//! Indexed reusable cache. SQLite serializes publication, eviction and blob reads.
mod blobs;
mod identity;
mod maintenance;
mod pool;
pub(crate) use maintenance::{Maintenance, maintain};
mod read;
pub(crate) use read::read_record;
pub(crate) mod request_key;
mod schema;
mod upgrade;

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use sha2::{Digest, Sha256};

pub(crate) const DATABASE_NAME: &str = "biomcp.db";
pub(crate) type Result<T> = std::result::Result<T, io::Error>;

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

pub(crate) fn read_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

pub(crate) fn sqlite_error(error: rusqlite::Error) -> io::Error {
    let kind = match &error {
        rusqlite::Error::SqliteFailure(code, _)
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) =>
        {
            io::ErrorKind::WouldBlock
        }
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, error)
}

#[derive(Clone)]
pub(crate) struct Store {
    connection: pool::SharedConnection,
    root: PathBuf,
}

#[derive(Debug)]
pub(crate) struct Record {
    pub(crate) key: String,
    pub(crate) metadata: Vec<u8>,
    pub(crate) body: Vec<u8>,
}

impl Store {
    pub(crate) fn open(root: &Path) -> Result<Self> {
        let _span = super::timing::Span::new("open");
        super::private::secure_managed_dir(root)?;
        // macOS exposes its temporary directory through /var, an ancestor
        // symlink. Resolve ancestors after rejecting a linked cache root so
        // SQLite's NOFOLLOW flag can protect the actual database path.
        let root = fs::canonicalize(root)?;
        let path = root.join(DATABASE_NAME);
        if path.exists() || fs::symlink_metadata(&path).is_ok() {
            validate_database_file(&path, false)?;
        }
        validate_sidecars(&root)?;
        upgrade::remove_legacy_stores(&root)?;
        let connection = pool::open(&root, || Self::open_connection(&root))?;
        Ok(Self { connection, root })
    }

    fn open_connection(root: &Path) -> Result<(Connection, identity::Identity)> {
        let path = root.join(DATABASE_NAME);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        // Only the creator opens an ordinary descriptor. Closing any descriptor
        // for a database can release another SQLite connection's POSIX locks.
        static OPEN: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _opening = crate::utils::sync::recover_poison(OPEN.lock());
        match super::open_private(&mut options, &path) {
            Ok(file) => drop(file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                validate_database_file(&path, false)?
            }
            Err(error) => return Err(error),
        }
        let identity = identity::Identity::capture(root)?;
        let connection = Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::default() | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(sqlite_error)?;
        identity.validate(root, &connection)?;
        connection
            .busy_timeout(Duration::from_millis(100))
            .map_err(sqlite_error)?;
        if super::timing::enabled() {
            connection
                .busy_handler(Some(super::timing::busy_handler))
                .map_err(sqlite_error)?;
        }
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sqlite_error)?;
        if version > 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported cache schema",
            ));
        }
        if version == 0 {
            // WAL initialization allocates database pages. Select the vacuum
            // format first so deleted inline bodies can return disk space.
            connection
                .execute_batch("PRAGMA auto_vacuum=INCREMENTAL;")
                .map_err(sqlite_error)?;
        }
        connection
            .execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA trusted_schema=OFF;",
            )
            .map_err(sqlite_error)?;
        if version == 0 {
            connection
                .execute_batch(schema::SCHEMA)
                .map_err(sqlite_error)?;
        }
        if version == 1 {
            upgrade::discard_credential_capable_cache(&connection, root)?;
        }
        identity.validate(root, &connection)?;
        Ok((connection, identity))
    }

    pub(crate) fn transaction<T>(
        &mut self,
        action: impl FnOnce(&Transaction<'_>, &Path) -> Result<T>,
    ) -> Result<T> {
        // Publication, accounting and blob collection share the writer lease.
        // Snapshot readers instead revalidate a concurrently removed blob.
        let begin = super::timing::Span::new("begin");
        let mut connection = self.connection();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sqlite_error)?;
        drop(begin);
        let result = action(&tx, &self.root)?;
        tx.commit().map_err(sqlite_error)?;
        Ok(result)
    }

    pub(crate) fn variants(&mut self, namespace: &str, base: &str) -> Result<Vec<Record>> {
        let connection = self.connection();
        let mut statement = connection
            .prepare("SELECT key,metadata FROM entries WHERE namespace=?1 AND base_key=?2")
            .map_err(sqlite_error)?;
        statement
            .query_map(params![namespace, base], |r| {
                Ok(Record {
                    key: r.get(0)?,
                    metadata: r.get(1)?,
                    body: Vec::new(),
                })
            })
            .map_err(sqlite_error)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(sqlite_error)
    }

    pub(crate) fn put(
        &mut self,
        namespace: &str,
        key: &str,
        base: &str,
        metadata: &[u8],
        body: &[u8],
        expires: Option<i64>,
    ) -> Result<()> {
        self.transaction(|tx, root| {
            put_record(tx, root, namespace, key, base, metadata, body, expires)
        })?;
        self.collect_blobs()
    }

    pub(crate) fn delete(&mut self, namespace: &str, key: &str) -> Result<()> {
        self.transaction(|tx, root| delete_record(tx, root, namespace, key).map(|_| ()))?;
        self.collect_blobs()
    }

    pub(crate) fn validate(&self) -> Result<()> {
        super::private::secure_managed_dir(&self.root)?;
        validate_database_file(&self.root.join(DATABASE_NAME), false)?;
        validate_sidecars(&self.root)?;
        pool::validate(&self.connection, &self.root)
    }

    pub(crate) fn connection(&self) -> pool::ConnectionAccess<'_> {
        pool::lock(&self.connection)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn put_record(
    tx: &Transaction<'_>,
    root: &Path,
    namespace: &str,
    key: &str,
    base: &str,
    metadata: &[u8],
    body: &[u8],
    expires: Option<i64>,
) -> Result<()> {
    let old_blob: Option<String> = tx
        .query_row(
            "SELECT blob FROM entries WHERE namespace=?1 AND key=?2",
            params![namespace, key],
            |r| r.get(0),
        )
        .optional()
        .map_err(sqlite_error)?
        .flatten();
    let blob = if body.len() > blobs::INLINE_LIMIT {
        Some(blobs::publish(root, body)?)
    } else {
        None
    };
    let inline = if blob.is_none() { Some(body) } else { None };
    let now = now_ms();
    tx.execute("INSERT INTO entries(namespace,key,base_key,metadata,body,blob,bytes,created,last_access,expires,body_hash) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8,?9,?10) ON CONFLICT(namespace,key) DO UPDATE SET base_key=excluded.base_key,metadata=excluded.metadata,body=excluded.body,blob=excluded.blob,body_hash=excluded.body_hash,bytes=excluded.bytes,created=excluded.created,last_access=excluded.last_access,expires=excluded.expires",
        params![namespace,key,base,metadata,inline,blob,(metadata.len()+body.len()) as i64,now,expires,format!("{:x}",Sha256::digest(body))]).map_err(sqlite_error)?;
    if let Some(old) = old_blob {
        remove_unreferenced(tx, root, &old)?;
    }
    Ok(())
}

pub(crate) fn delete_record(
    tx: &Transaction<'_>,
    root: &Path,
    namespace: &str,
    key: &str,
) -> Result<u64> {
    let row = tx
        .query_row(
            "SELECT bytes,blob FROM entries WHERE namespace=?1 AND key=?2",
            params![namespace, key],
            |r| Ok((read_u64(r, 0)?, r.get::<_, Option<String>>(1)?)),
        )
        .optional()
        .map_err(sqlite_error)?;
    let Some((bytes, blob)) = row else {
        return Ok(0);
    };
    tx.execute(
        "DELETE FROM entries WHERE namespace=?1 AND key=?2",
        params![namespace, key],
    )
    .map_err(sqlite_error)?;
    if let Some(blob) = blob {
        remove_unreferenced(tx, root, &blob)?;
    }
    Ok(bytes)
}

fn remove_unreferenced(tx: &Transaction<'_>, _root: &Path, blob: &str) -> Result<()> {
    tx.execute("INSERT OR IGNORE INTO blob_gc(digest) VALUES(?1)", [blob])
        .map_err(sqlite_error)?;
    Ok(())
}

impl Store {
    pub(crate) fn collect_blobs(&mut self) -> Result<()> {
        let mut connection = self.connection();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sqlite_error)?;
        let digests = {
            let mut statement = tx
                .prepare("SELECT digest FROM blob_gc")
                .map_err(sqlite_error)?;
            statement
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(sqlite_error)?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(sqlite_error)?
        };
        for digest in digests {
            let count: i64 = tx
                .query_row(
                    "SELECT count(*) FROM entries WHERE blob=?1",
                    [&digest],
                    |r| r.get(0),
                )
                .map_err(sqlite_error)?;
            if count == 0 {
                blobs::remove(&self.root, &digest)?;
            }
            tx.execute("DELETE FROM blob_gc WHERE digest=?1", [&digest])
                .map_err(sqlite_error)?;
        }
        tx.commit().map_err(sqlite_error)
    }
}

#[cfg(test)]
mod stress_tests;
#[cfg(test)]
mod tests;

fn validate_database_file(path: &Path, transient: bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid SQLite cache file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        if metadata.nlink() != 1 {
            if transient && metadata.nlink() == 0 {
                return Ok(());
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "linked SQLite cache file",
            ));
        }
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "insecure SQLite cache file",
            ));
        }
    }
    #[cfg(windows)]
    {
        let mut options = OpenOptions::new();
        options.read(true).write(true);
        drop(super::open_private(&mut options, path)?);
    }
    Ok(())
}

impl Store {
    /// Explicit clean alone enumerates orphan external blobs. Ordinary reads,
    /// accounting, LRU eviction and startup never walk the blob directory.
    pub(crate) fn collect_orphans(&mut self, dry_run: bool) -> Result<(usize, u64)> {
        self.transaction(|tx, root| {
            let directory = root.join("blobs");
            let metadata = match fs::symlink_metadata(&directory) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok((0, 0)),
                result => result?,
            };
            if !metadata.is_dir() || super::private::is_link_or_reparse_point(&metadata) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid blob directory",
                ));
            }
            let mut count = 0;
            let mut bytes = 0;
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let name = entry.file_name();
                let Some(name) = name.to_str() else { continue };
                let references: i64 = tx
                    .query_row("SELECT count(*) FROM entries WHERE blob=?1", [name], |r| {
                        r.get(0)
                    })
                    .map_err(sqlite_error)?;
                if references != 0 {
                    continue;
                }
                let metadata = fs::symlink_metadata(entry.path())?;
                if metadata.is_dir() {
                    continue;
                }
                count += 1;
                bytes += metadata.len();
                if !dry_run {
                    fs::remove_file(entry.path())?;
                }
            }
            Ok((count, bytes))
        })
    }
}

impl Store {
    pub(crate) fn release_space(&self) -> Result<()> {
        // SQLite yields one row per reclaimed page. Step the pragma to
        // completion; execute_batch stops after its first yielded row.
        let connection = self.connection();
        let mut vacuum = connection
            .prepare("PRAGMA incremental_vacuum")
            .map_err(sqlite_error)?;
        let mut pages = vacuum.query([]).map_err(sqlite_error)?;
        while pages.next().map_err(sqlite_error)?.is_some() {}
        drop(pages);
        drop(vacuum);
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(sqlite_error)
    }
}

fn validate_sidecars(root: &Path) -> Result<()> {
    for suffix in ["-wal", "-shm", "-journal"] {
        let sidecar = root.join(format!("{DATABASE_NAME}{suffix}"));
        if sidecar.exists() || fs::symlink_metadata(&sidecar).is_ok() {
            match validate_database_file(&sidecar, true) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                result => result?,
            }
        }
    }
    Ok(())
}
