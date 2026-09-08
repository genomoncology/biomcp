use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};

use fs2::FileExt;

use crate::error::BioMcpError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ClearReport {
    pub(crate) bytes_freed: Option<u64>,
    pub(crate) entries_removed: usize,
}

#[derive(Debug, Default)]
struct ClearPlan {
    regular_files: Vec<(PathBuf, u64)>,
    symlinks: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

pub(crate) fn execute_cache_clear(cache_path: &Path) -> Result<ClearReport, BioMcpError> {
    let metadata = match fs::symlink_metadata(cache_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(ClearReport {
                bytes_freed: Some(0),
                entries_removed: 0,
            });
        }
        Err(err) => return Err(BioMcpError::Io(err)),
    };

    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        fs::remove_file(cache_path)?;
        return Ok(ClearReport {
            bytes_freed: None,
            entries_removed: 1,
        });
    }

    if !file_type.is_dir() {
        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear requires '{}' to be a directory or symlink, found {}",
            cache_path.display(),
            describe_file_type(&file_type)
        )));
    }

    super::secure_managed_tree(cache_path, true, None)
        .map_err(|error| BioMcpError::InvalidArgument(error.to_string()))?;
    let mut plan = ClearPlan::default();
    scan_directory(cache_path, &mut plan)?;
    plan.directories.push(cache_path.to_path_buf());

    let file_count = plan.regular_files.len();
    let symlink_count = plan.symlinks.len();
    let directory_count = plan.directories.len();
    let saw_symlink = symlink_count > 0;

    let mut bytes_freed = 0u64;
    for (path, size_bytes) in plan.regular_files {
        reject_linked_file(&path, &fs::symlink_metadata(&path)?)?;
        fs::remove_file(path)?;
        bytes_freed += size_bytes;
    }

    for path in plan.symlinks {
        fs::remove_file(path)?;
    }

    plan.directories
        .sort_by_key(|path| Reverse(path.components().count()));
    for path in plan.directories {
        fs::remove_dir(path)?;
    }

    Ok(ClearReport {
        bytes_freed: (!saw_symlink).then_some(bytes_freed),
        entries_removed: file_count + symlink_count + directory_count,
    })
}

#[cfg(unix)]
fn reject_linked_file(path: &Path, metadata: &fs::Metadata) -> Result<(), BioMcpError> {
    use std::os::unix::fs::MetadataExt;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear refuses linked or replaced file '{}'",
            path.display()
        )));
    }
    Ok(())
}

const CACHE_OPERATION_LOCK: &str = ".biomcp-operation.lock";
pub(super) const BODY_LIMIT_CACHE_MARKER: &[u8] = b"bounded-response-body-v1\n";
type LeaseMap = HashMap<PathBuf, usize>;
static SHARED_LEASES: OnceLock<(Mutex<LeaseMap>, Condvar)> = OnceLock::new();

pub(super) fn epoch_state(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
) -> io::Result<super::migration::EpochState> {
    use super::migration::EpochState;
    let marker =
        super::migration::validated_marker_exists(&cache_root.join(".body-limit-cache-v1"))?;
    let legacy = match fs::symlink_metadata(cache_root.join("http-cacache")) {
        Ok(_) => true,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => return Err(error),
    };
    Ok(match (marker, legacy_cache_was_renamed, legacy) {
        (true, false, false) => EpochState::Current,
        (true, false, true) => EpochState::RemoveLegacy,
        _ => EpochState::Rebuild,
    })
}

pub(crate) struct CacheOperationGuard {
    file: File,
    _shared_lease: Option<SharedLease>,
}

impl Drop for CacheOperationGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

struct SharedLease {
    root: PathBuf,
}

impl Drop for SharedLease {
    fn drop(&mut self) {
        unregister_shared(&self.root);
    }
}

pub(crate) struct CacheKeyGuard {
    _shared: CacheOperationGuard,
    _key: CacheOperationGuard,
}

fn shared_leases() -> &'static (Mutex<LeaseMap>, Condvar) {
    SHARED_LEASES.get_or_init(|| (Mutex::new(HashMap::new()), Condvar::new()))
}

fn lease_registry() -> io::Result<MutexGuard<'static, LeaseMap>> {
    shared_leases()
        .0
        .lock()
        .map_err(|_| io::Error::other("cache operation lease registry is poisoned"))
}

fn register_shared(root: &Path) -> io::Result<SharedLease> {
    let root = root.to_path_buf();
    *lease_registry()?.entry(root.clone()).or_default() += 1;
    Ok(SharedLease { root })
}

fn unregister_shared(root: &Path) {
    if let Ok(mut leases) = shared_leases().0.lock()
        && let Some(count) = leases.get_mut(root)
    {
        *count -= 1;
        if *count == 0 {
            leases.remove(root);
            shared_leases().1.notify_all();
        }
    }
}

fn operation_lock_file(cache_root: &Path) -> io::Result<File> {
    super::private::secure_managed_dir(cache_root)?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    super::private::open_private(&mut options, &cache_root.join(CACHE_OPERATION_LOCK))
}

fn local_shared_upgrade_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::WouldBlock,
        "cache maintenance cannot upgrade an active in-process shared operation",
    )
}

fn try_exclusive_without_local_shared(cache_root: &Path, file: &File) -> io::Result<bool> {
    let leases = lease_registry()?;
    if leases.get(cache_root).copied().unwrap_or_default() > 0 {
        return Err(local_shared_upgrade_error());
    }
    match file.try_lock_exclusive() {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn lock_cache_maintenance(cache_root: &Path) -> io::Result<CacheOperationGuard> {
    let file = operation_lock_file(cache_root)?;
    if crate::sources::current_variant_article_deadline().is_some() {
        return lock_maintenance_with_deadline(cache_root, file);
    }
    let leases = lease_registry()?;
    if leases.get(cache_root).copied().unwrap_or_default() > 0 {
        return Err(local_shared_upgrade_error());
    }
    file.lock_exclusive()?;
    drop(leases);
    Ok(exclusive_guard(file))
}

pub(crate) fn lock_cache_maintenance_after_shared(
    cache_root: &Path,
) -> io::Result<CacheOperationGuard> {
    let file = operation_lock_file(cache_root)?;
    let mut leases = lease_registry()?;
    while leases.get(cache_root).copied().unwrap_or_default() > 0 {
        leases = shared_leases()
            .1
            .wait(leases)
            .map_err(|_| io::Error::other("cache operation lease registry is poisoned"))?;
    }
    file.lock_exclusive()?;
    drop(leases);
    Ok(exclusive_guard(file))
}

fn lock_maintenance_with_deadline(
    cache_root: &Path,
    file: File,
) -> io::Result<CacheOperationGuard> {
    let deadline = crate::sources::current_variant_article_deadline().expect("deadline exists");
    loop {
        match try_exclusive_without_local_shared(cache_root, &file)? {
            true => return Ok(exclusive_guard(file)),
            false => {
                deadline.ensure_time_io()?;
                std::thread::sleep(
                    deadline
                        .remaining()
                        .min(std::time::Duration::from_millis(10)),
                );
            }
        }
    }
}

pub(crate) fn try_lock_cache_maintenance(
    cache_root: &Path,
) -> io::Result<Option<CacheOperationGuard>> {
    let file = operation_lock_file(cache_root)?;
    match try_exclusive_without_local_shared(cache_root, &file) {
        Ok(true) => Ok(Some(exclusive_guard(file))),
        Ok(false) => Ok(None),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn lock_cache_shared(cache_root: &Path) -> io::Result<CacheOperationGuard> {
    let file = operation_lock_file(cache_root)?;
    let lease = register_shared(cache_root)?;
    let result = if let Some(deadline) = crate::sources::current_variant_article_deadline() {
        loop {
            match FileExt::try_lock_shared(&file) {
                Ok(()) => break Ok(()),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(error) = deadline.ensure_time_io() {
                        break Err(error);
                    }
                    std::thread::sleep(
                        deadline
                            .remaining()
                            .min(std::time::Duration::from_millis(10)),
                    );
                }
                Err(error) => break Err(error),
            }
        }
    } else {
        FileExt::lock_shared(&file)
    };
    result.map(|()| shared_guard(file, lease))
}

fn shared_guard(file: File, lease: SharedLease) -> CacheOperationGuard {
    CacheOperationGuard {
        file,
        _shared_lease: Some(lease),
    }
}

fn exclusive_guard(file: File) -> CacheOperationGuard {
    CacheOperationGuard {
        file,
        _shared_lease: None,
    }
}

fn lock_cache_key(
    cache_root: &Path,
    key: &str,
    before_lock_dir_create: &dyn Fn(&Path),
) -> io::Result<CacheKeyGuard> {
    let shared = lock_cache_shared(cache_root)?;
    let lock_dir = cache_root.join(super::KEY_LOCK_DIR);
    super::private::secure_managed_dir_with(&lock_dir, || before_lock_dir_create(&lock_dir))?;
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    let file = super::private::open_private(&mut options, &super::key_lock_path(cache_root, key))?;
    file.lock_exclusive()?;
    Ok(CacheKeyGuard {
        _shared: shared,
        _key: exclusive_guard(file),
    })
}

pub(crate) async fn lock_cache_key_async(
    cache_root: PathBuf,
    key: String,
    before_lock_dir_create: Arc<dyn Fn(&Path) + Send + Sync>,
) -> io::Result<CacheKeyGuard> {
    if let Some(deadline) = crate::sources::current_variant_article_deadline() {
        let shared = lock_cache_shared_until(&cache_root, &deadline).await?;
        let lock_dir = cache_root.join(super::KEY_LOCK_DIR);
        super::private::secure_managed_dir_with(&lock_dir, || before_lock_dir_create(&lock_dir))?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        let key_file =
            super::private::open_private(&mut options, &super::key_lock_path(&cache_root, &key))?;
        let key_guard = lock_file_until(key_file, deadline).await?;
        return Ok(CacheKeyGuard {
            _shared: shared,
            _key: key_guard,
        });
    }
    tokio::task::spawn_blocking(move || {
        lock_cache_key(&cache_root, &key, before_lock_dir_create.as_ref())
    })
    .await
    .map_err(|error| io::Error::other(format!("cache key lock task failed: {error}")))?
}

pub(crate) async fn lock_cache_shared_until(
    cache_root: &Path,
    deadline: &crate::sources::VariantArticleDeadline,
) -> io::Result<CacheOperationGuard> {
    lock_cache_shared_until_with(cache_root, deadline, || {}).await
}

async fn lock_cache_shared_until_with<F>(
    cache_root: &Path,
    deadline: &crate::sources::VariantArticleDeadline,
    waiting: F,
) -> io::Result<CacheOperationGuard>
where
    F: FnMut(),
{
    let file = operation_lock_file(cache_root)?;
    let lease = register_shared(cache_root)?;
    lock_file_shared_until(file, lease, deadline, waiting).await
}

pub(crate) async fn lock_cache_maintenance_until(
    cache_root: &Path,
    deadline: &crate::sources::VariantArticleDeadline,
) -> io::Result<CacheOperationGuard> {
    let file = operation_lock_file(cache_root)?;
    loop {
        match try_exclusive_without_local_shared(cache_root, &file)? {
            true => return Ok(exclusive_guard(file)),
            false => wait(deadline).await?,
        }
    }
}

async fn lock_file_shared_until<F>(
    file: File,
    lease: SharedLease,
    deadline: &crate::sources::VariantArticleDeadline,
    mut waiting: F,
) -> io::Result<CacheOperationGuard>
where
    F: FnMut(),
{
    loop {
        match FileExt::try_lock_shared(&file) {
            Ok(()) => return Ok(shared_guard(file, lease)),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                waiting();
                wait(deadline).await?;
            }
            Err(error) => return Err(error),
        }
    }
}

async fn lock_file_until(
    file: File,
    deadline: crate::sources::VariantArticleDeadline,
) -> io::Result<CacheOperationGuard> {
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => return Ok(exclusive_guard(file)),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => wait(&deadline).await?,
            Err(error) => return Err(error),
        }
    }
}

async fn wait(deadline: &crate::sources::VariantArticleDeadline) -> io::Result<()> {
    deadline
        .run(tokio::time::sleep(std::time::Duration::from_millis(10)))
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "variant article invocation deadline exceeded",
            )
        })
}

#[cfg(test)]
mod operation_lock_tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn constructor_repairs_and_independent_key_operations_do_not_serialize_globally() {
        let root = tempfile::tempdir().expect("temporary root");
        let constructor_a = lock_cache_shared(root.path()).expect("first shared constructor lock");
        let (constructor_tx, constructor_rx) = mpsc::channel();
        let root_path = root.path().to_path_buf();
        let constructor_b = std::thread::spawn(move || {
            let guard = lock_cache_shared(&root_path).expect("second shared constructor lock");
            constructor_tx.send(()).expect("report shared acquisition");
            guard
        });
        constructor_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("constructor repairs must overlap while first lock remains held");
        drop(constructor_a);
        drop(constructor_b.join().expect("constructor thread"));

        let key_a = "independent-key-a";
        let key_b = (0..10_000)
            .map(|candidate| format!("independent-key-{candidate}"))
            .find(|candidate| {
                super::super::key_lock_path(root.path(), candidate)
                    != super::super::key_lock_path(root.path(), key_a)
            })
            .expect("key in another lock shard");
        let operation_a = lock_cache_key(root.path(), key_a, &|_| {}).expect("first key lock");
        let (key_tx, key_rx) = mpsc::channel();
        let root_path = root.path().to_path_buf();
        let operation_b = std::thread::spawn(move || {
            let guard = lock_cache_key(&root_path, &key_b, &|_| {}).expect("independent key lock");
            key_tx.send(()).expect("report independent acquisition");
            guard
        });
        key_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("independent key operations must overlap");
        drop(operation_a);
        drop(operation_b.join().expect("key operation thread"));
    }

    #[test]
    fn same_process_readers_never_block_on_an_exclusive_upgrade() {
        let root = tempfile::tempdir().expect("temporary cache root");
        let first = lock_cache_shared(root.path()).expect("first shared lease");
        let second = lock_cache_shared(root.path()).expect("second shared lease");
        let error = match lock_cache_maintenance(root.path()) {
            Ok(_) => panic!("maintenance must reject an in-process shared-to-exclusive upgrade"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        drop(first);
        drop(second);
        drop(lock_cache_maintenance(root.path()).expect("maintenance after shared release"));
        drop(lock_cache_shared(root.path()).expect("shared lease after maintenance release"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancelled_pending_shared_lock_releases_its_provisional_lease() {
        let root = tempfile::tempdir().expect("temporary cache root");
        let held = try_lock_cache_maintenance(root.path())
            .expect("maintenance lock attempt")
            .expect("maintenance lock");
        let root_path = root.path().to_path_buf();
        let (waiting_tx, waiting_rx) = tokio::sync::oneshot::channel();
        let waiter = tokio::spawn(async move {
            let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(5));
            let mut waiting_tx = Some(waiting_tx);
            lock_cache_shared_until_with(&root_path, &deadline, move || {
                if let Some(waiting_tx) = waiting_tx.take() {
                    waiting_tx.send(()).expect("report blocked shared flock");
                }
            })
            .await
        });

        waiting_rx
            .await
            .expect("shared waiter reached flock contention");
        waiter.abort();
        let cancelled = match waiter.await {
            Ok(_) => panic!("shared waiter must be cancelled"),
            Err(cancelled) => cancelled,
        };
        assert!(cancelled.is_cancelled());
        drop(held);

        drop(
            try_lock_cache_maintenance(root.path())
                .expect("maintenance attempt after cancellation")
                .expect("cancelled waiter must not retain a provisional lease"),
        );
        drop(
            lock_cache_maintenance_after_shared(root.path())
                .expect("background maintenance after cancellation"),
        );
        drop(lock_cache_shared(root.path()).expect("shared lock after cancellation"));
    }
}

#[cfg(not(unix))]
fn reject_linked_file(_path: &Path, metadata: &fs::Metadata) -> Result<(), BioMcpError> {
    if metadata.is_file() {
        Ok(())
    } else {
        Err(BioMcpError::InvalidArgument(
            "cache file changed during clear".into(),
        ))
    }
}

fn scan_directory(path: &Path, plan: &mut ClearPlan) -> Result<(), BioMcpError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)?;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            plan.regular_files.push((entry_path, metadata.len()));
            continue;
        }

        if file_type.is_symlink() {
            plan.symlinks.push(entry_path);
            continue;
        }

        if file_type.is_dir() {
            scan_directory(&entry_path, plan)?;
            plan.directories.push(entry_path);
            continue;
        }

        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear found unsupported entry at '{}': {}",
            entry_path.display(),
            describe_file_type(&file_type)
        )));
    }

    Ok(())
}

fn describe_file_type(file_type: &fs::FileType) -> &'static str {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

        if file_type.is_socket() {
            return "socket";
        }
        if file_type.is_fifo() {
            return "fifo";
        }
        if file_type.is_block_device() {
            return "block device";
        }
        if file_type.is_char_device() {
            return "char device";
        }
    }

    if file_type.is_dir() {
        "directory"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_symlink() {
        "symlink"
    } else {
        "unsupported file type"
    }
}

#[cfg(test)]
mod tests {
    use super::{ClearReport, execute_cache_clear};
    use crate::error::BioMcpError;
    use crate::test_support::TempDirGuard;
    use std::fs;

    #[test]
    fn clear_missing_path_returns_zero_report() {
        let root = TempDirGuard::new("missing");

        let report =
            execute_cache_clear(&root.path().join("http")).expect("missing path should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: Some(0),
                entries_removed: 0,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn clear_root_symlink_is_unlinked_without_traversing_target() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("root-symlink");
        let target_dir = root.path().join("target");
        let target_file = target_dir.join("outside.txt");
        fs::create_dir_all(&target_dir).expect("create target dir");
        fs::write(&target_file, b"outside").expect("seed target file");
        symlink(&target_dir, root.path().join("http")).expect("create root symlink");

        let report = execute_cache_clear(&root.path().join("http")).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: None,
                entries_removed: 1,
            }
        );
        assert!(
            !root.path().join("http").exists(),
            "root symlink should be removed"
        );
        assert!(target_file.is_file(), "target file should remain untouched");
    }

    #[cfg(unix)]
    #[test]
    fn clear_nested_symlink_unlinks_entry_and_sets_bytes_to_none() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("nested-symlink");
        let http_dir = root.path().join("http");
        let target_dir = root.path().join("target");
        let target_file = target_dir.join("outside.txt");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::create_dir_all(&target_dir).expect("create target dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");
        fs::write(&target_file, b"outside").expect("seed target file");
        symlink(&target_dir, http_dir.join("nested").join("link")).expect("create nested symlink");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: None,
                entries_removed: 4,
            }
        );
        assert!(!http_dir.exists(), "http dir should be removed");
        assert!(target_file.is_file(), "symlink target should remain");
    }

    #[test]
    fn clear_removes_directory_tree_and_root_http_dir() {
        let root = TempDirGuard::new("remove-tree");
        let http_dir = root.path().join("http");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: Some(3),
                entries_removed: 3,
            }
        );
        assert!(!http_dir.exists(), "http dir should be removed");
    }

    #[test]
    fn clear_preserves_sibling_downloads_directory() {
        let root = TempDirGuard::new("preserve-downloads");
        let http_dir = root.path().join("http");
        let downloads_dir = root.path().join("downloads");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::create_dir_all(&downloads_dir).expect("create downloads dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");
        fs::write(downloads_dir.join("keep.bin"), b"keep").expect("seed downloads file");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(report.bytes_freed, Some(3));
        assert!(!http_dir.exists(), "http dir should be removed");
        assert!(downloads_dir.is_dir(), "downloads dir should remain");
        assert_eq!(
            fs::read(downloads_dir.join("keep.bin")).expect("downloads file should remain"),
            b"keep"
        );
    }

    #[test]
    fn clear_rejects_root_regular_file_before_mutation() {
        let root = TempDirGuard::new("root-file");
        let http_path = root.path().join("http");
        fs::write(&http_path, b"not-a-dir").expect("seed file");

        let err = execute_cache_clear(&http_path).expect_err("root file should be rejected");

        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(http_path.is_file(), "root file should remain untouched");
    }

    #[cfg(unix)]
    #[test]
    fn clear_rejects_special_file_before_mutation() {
        use std::os::unix::net::UnixListener;

        let root = tempfile::Builder::new()
            .prefix("biomcp-sock-")
            .tempdir_in("/tmp")
            .expect("create short socket root");
        let http_dir = root.path().join("http");
        let socket_path = http_dir.join("special.sock");
        assert!(
            socket_path.as_os_str().len() < 100,
            "socket fixture path is too long"
        );
        let regular_file = http_dir.join("entry.bin");
        fs::create_dir_all(&http_dir).expect("create http dir");
        fs::write(&regular_file, b"abc").expect("seed file");
        let _listener = UnixListener::bind(&socket_path).expect("bind unix socket");

        let err = execute_cache_clear(&http_dir).expect_err("special file should be rejected");

        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(http_dir.is_dir(), "http dir should remain");
        assert!(regular_file.is_file(), "regular file should remain");
        assert!(socket_path.exists(), "socket should remain");
    }
}
