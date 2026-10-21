//! Drop old reusable bytes without decoding credential-bearing metadata.
use std::io;
use std::path::Path;

use rusqlite::Connection;

use super::{Result, sqlite_error};

pub(super) fn remove_legacy_stores(root: &Path) -> Result<()> {
    for name in crate::cache::LEGACY_STORES {
        crate::cache::execute_cache_clear(&root.join(name))
            .map_err(|_| io::Error::other("failed to remove obsolete cache store"))?;
    }
    Ok(())
}

pub(super) fn discard_credential_capable_cache(connection: &Connection, root: &Path) -> Result<()> {
    // DELETE alone leaves old metadata in SQLite free pages and its WAL. No
    // inspection is needed: every pre-fix reusable entry is disposable.
    connection
        .execute_batch("PRAGMA secure_delete=ON; BEGIN IMMEDIATE; DELETE FROM entries; DELETE FROM blob_gc; COMMIT;")
        .map_err(sqlite_error)?;
    crate::cache::execute_cache_clear(&root.join("blobs"))
        .map_err(|_| io::Error::other("failed to remove obsolete cache blobs"))?;
    connection.execute_batch("VACUUM;").map_err(sqlite_error)?;
    checkpoint(connection)?;
    // Mark complete only after old database and WAL bytes have been removed.
    // An interrupted cleanup remains version 1 and retries on the next open.
    connection
        .execute_batch("PRAGMA user_version=2;")
        .map_err(sqlite_error)?;
    checkpoint(connection)
}

fn checkpoint(connection: &Connection) -> Result<()> {
    let busy: i64 = connection
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
        .map_err(sqlite_error)?;
    if busy != 0 {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "obsolete cache cleanup is waiting for a database reader",
        ));
    }
    Ok(())
}
