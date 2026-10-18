//! Cross-process byte reservations and atomic totals records.
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::{Mutex, TryLockError};

use super::CONTENT_TOTALS_MARKER_FILE;

static TOTALS_MUTEX: Mutex<()> = Mutex::new(());

pub(super) fn with_lock<T>(
    cache_path: &Path,
    try_only: bool,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    let try_only = try_only || crate::sources::current_variant_article_deadline().is_some();
    // No future is suspended while either lock is held. The local mutex also
    // serializes threads on platforms whose file locks belong to the process.
    let _local = if try_only {
        match TOTALS_MUTEX.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(io::ErrorKind::WouldBlock.into()),
        }
    } else {
        crate::utils::sync::recover_poison(TOTALS_MUTEX.lock())
    };
    super::super::private::secure_managed_dir(cache_path)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    let lock =
        super::super::private::open_private(&mut options, &cache_path.join(".content-bytes.lock"))?;
    if try_only {
        crate::utils::fs_lock::try_lock_exclusive(&lock)?;
    } else {
        lock.lock()?;
    }
    operation()
}

pub(super) fn add_content_bytes(cache_path: &Path, bytes: u64, try_only: bool) -> io::Result<u64> {
    with_lock(cache_path, try_only, || {
        let previous = match read_content_totals_marker(cache_path) {
            Some(total) => total,
            None => super::sum_content_tree_bytes(cache_path)?,
        };
        let total = previous.saturating_add(bytes);
        write_content_totals_marker(cache_path, total)?;
        Ok(total)
    })
}

pub(super) fn read_content_totals_marker(cache_path: &Path) -> Option<u64> {
    let mut file =
        super::super::private::open_managed_read(&cache_path.join(CONTENT_TOTALS_MARKER_FILE))
            .ok()?;
    let mut text = String::new();
    file.read_to_string(&mut text).ok()?;
    // A decimal prefix from an interrupted legacy in-place write is never a
    // valid total. Even the terminal newline must survive for a v2 record.
    text.strip_prefix("v2 ")?.strip_suffix('\n')?.parse().ok()
}

pub(super) fn write_content_totals_marker(cache_path: &Path, total_bytes: u64) -> io::Result<()> {
    super::super::private::secure_managed_dir(cache_path)?;
    let temporary = cache_path.join(format!(".content-bytes-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = super::super::private::open_private(&mut options, &temporary)?;
        writeln!(file, "v2 {total_bytes}")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, cache_path.join(CONTENT_TOTALS_MARKER_FILE))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
