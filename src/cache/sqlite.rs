//! Indexed reusable cache. SQLite serializes publication, eviction and blob reads.
mod blobs;
pub(crate) mod request_key;
mod schema;

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

pub(crate) struct Store {
    connection: Connection,
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
        super::private::secure_managed_dir(root)?;
        // macOS exposes its temporary directory through /var, an ancestor
        // symlink. Resolve ancestors after rejecting a linked cache root so
        // SQLite's NOFOLLOW flag can protect the actual database path.
        let root = fs::canonicalize(root)?;
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
        for suffix in ["-wal", "-shm", "-journal"] {
            let sidecar = root.join(format!("{DATABASE_NAME}{suffix}"));
            if sidecar.exists() || fs::symlink_metadata(&sidecar).is_ok() {
                match validate_database_file(&sidecar, true) {
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    result => result?,
                }
            }
        }
        let connection = Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::default() | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(sqlite_error)?;
        connection
            .busy_timeout(Duration::from_millis(100))
            .map_err(sqlite_error)?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(sqlite_error)?;
        if version > 1 {
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
        Ok(Self { connection, root })
    }

    pub(crate) fn transaction<T>(
        &mut self,
        action: impl FnOnce(&Transaction<'_>, &Path) -> Result<T>,
    ) -> Result<T> {
        // Blob readers take the same short writer lease as publishers. WAL readers
        // alone cannot prevent an evictor unlinking a referenced external file.
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sqlite_error)?;
        let result = action(&tx, &self.root)?;
        tx.commit().map_err(sqlite_error)?;
        Ok(result)
    }

    pub(crate) fn get(&mut self, namespace: &str, key: &str) -> Result<Option<Record>> {
        self.transaction(|tx, root| read_record(tx, root, namespace, key))
    }

    pub(crate) fn variants(&mut self, namespace: &str, base: &str) -> Result<Vec<Record>> {
        let mut statement = self
            .connection
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

    pub(crate) fn connection(&self) -> &Connection {
        &self.connection
    }
}

pub(crate) fn read_record(
    tx: &Transaction<'_>,
    root: &Path,
    namespace: &str,
    key: &str,
) -> Result<Option<Record>> {
    let record = tx
        .query_row(
            "SELECT metadata, body, blob, body_hash FROM entries WHERE namespace=?1 AND key=?2",
            params![namespace, key],
            |r| {
                Ok((
                    r.get::<_, Vec<u8>>(0)?,
                    r.get::<_, Option<Vec<u8>>>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, String>(3)?,
                ))
            },
        )
        .optional()
        .map_err(sqlite_error)?;
    let Some((metadata, body, blob, body_hash)) = record else {
        return Ok(None);
    };
    let body = match (body, blob) {
        (Some(body), None) => body,
        (None, Some(blob)) => blobs::read(root, &blob)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid cache body",
            ));
        }
    };
    if format!("{:x}", Sha256::digest(&body)) != body_hash {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cache body integrity",
        ));
    }
    let now = now_ms();
    tx.execute("UPDATE entries SET last_access=?3 WHERE namespace=?1 AND key=?2 AND last_access<=?3-3600000", params![namespace, key, now]).map_err(sqlite_error)?;
    Ok(Some(Record {
        key: key.to_owned(),
        metadata,
        body,
    }))
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
        let tx = self
            .connection
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

#[derive(Clone, Copy)]
pub(crate) enum Maintenance {
    Startup,
    Pressure,
    Clean,
}

pub(crate) fn maintain(
    store: &mut Store,
    config: &super::ResolvedCacheConfig,
    max_age: Option<Duration>,
    max_size: Option<u64>,
    dry_run: bool,
    mode: Maintenance,
) -> Result<super::CleanReport> {
    let now = now_ms();
    let space = super::inspect_filesystem_space(&config.cache_root).map_err(io::Error::other)?;
    let mut report=store.transaction(|tx, root| {
        let last: i64 = tx.query_row("SELECT last_clean FROM cache_meta WHERE id=1", [], |r|r.get(0)).map_err(sqlite_error)?;
        let due = match mode {Maintenance::Startup=>now.saturating_sub(last)>=86_400_000,Maintenance::Pressure=>false,Maintenance::Clean=>true};
        let age = max_age.or(if due { Some(config.max_age) } else { None });
        let cutoff = age.map(|age|now.saturating_sub(age.as_millis().min(i64::MAX as u128) as i64));
        let mut total: u64 = tx.query_row("SELECT optional_bytes FROM cache_meta WHERE id=1",[],|r|read_u64(r,0)).map_err(sqlite_error)?;
        let limit = max_size.unwrap_or(config.max_size);
        let expired = if due {
            let mut statement = tx.prepare("SELECT namespace,key FROM entries WHERE (namespace!='capture' AND expires IS NOT NULL AND expires<=?1) OR (namespace='http' AND ?2 IS NOT NULL AND created<=?2)").map_err(sqlite_error)?;
            statement.query_map(params![now,cutoff],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(sqlite_error)?
                .collect::<std::result::Result<Vec<_>,_>>().map_err(sqlite_error)?
        } else { Vec::new() };
        let mut selected = expired;
        let mut planned: u64 = 0;
        for (namespace,key) in &selected {
            planned += tx.query_row("SELECT bytes FROM entries WHERE namespace=?1 AND key=?2",params![namespace,key],|r|read_u64(r,0)).map_err(sqlite_error)?;
        }
        total=total.saturating_sub(planned);
        let mut limits=config.clone();limits.max_size=limit;
        let estimated_space=super::FilesystemSpace{available_bytes:space.available_bytes.saturating_add(planned),total_bytes:space.total_bytes};
        let evaluation=super::evaluate_cache_usage(super::CacheUsage{referenced_blob_bytes:total,orphan_blob_bytes:0},&limits,estimated_space);
        let target=evaluation.effective_max_size;
        let mut selected_set=selected.iter().cloned().collect::<std::collections::HashSet<_>>();
        if total>target {
            let mut statement=tx.prepare("SELECT namespace,key,bytes FROM entries WHERE namespace!='capture' ORDER BY last_access").map_err(sqlite_error)?;
            let rows=statement.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,read_u64(r,2)?))).map_err(sqlite_error)?;
            for row in rows {
                if total<=target { break; }
                let (namespace,key,bytes)=row.map_err(sqlite_error)?;
                if !selected_set.insert((namespace.clone(),key.clone())) {continue;}
                selected.push((namespace,key)); total=total.saturating_sub(bytes);planned+=bytes;
            }
        }
        let mut capture_bytes=0;
        for (namespace,key) in &selected {
            if namespace=="capture" {
                capture_bytes+=tx.query_row("SELECT bytes FROM entries WHERE namespace=?1 AND key=?2",params![namespace,key],|r|read_u64(r,0)).map_err(sqlite_error)?;
            }
            if !dry_run { delete_record(tx,root,namespace,key)?; }
        }
        if !dry_run && due && matches!(mode,Maintenance::Startup) { tx.execute("UPDATE cache_meta SET last_clean=?1 WHERE id=1",[now]).map_err(sqlite_error)?; }
        Ok(super::CleanReport { dry_run,orphans_removed:0,entries_removed:selected.len(),bytes_freed:planned,provider_capture_bytes_freed:capture_bytes,errors:Vec::new() })
    })?;
    if matches!(mode, Maintenance::Clean) {
        let captures = super::provider_capture::ProviderCaptureStore::new(&config.cache_root);
        let freed = captures.clean_in_store(store, dry_run)?;
        report.provider_capture_bytes_freed = freed;
        report.bytes_freed = report.bytes_freed.saturating_add(freed);
    }
    if !dry_run {
        store.collect_blobs()?;
        if report.bytes_freed > 0 {
            store.release_space()?;
        }
    }
    Ok(report)
}

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
        let mut vacuum = self
            .connection
            .prepare("PRAGMA incremental_vacuum")
            .map_err(sqlite_error)?;
        let mut pages = vacuum.query([]).map_err(sqlite_error)?;
        while pages.next().map_err(sqlite_error)?.is_some() {}
        drop(pages);
        drop(vacuum);
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(sqlite_error)
    }
}
