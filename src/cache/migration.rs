use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use fs2::FileExt;

use super::private::open_private;

const BODY_LIMIT_CACHE_EPOCH: &str = ".body-limit-cache-v1";
const BODY_LIMIT_CACHE_LOCK: &str = ".body-limit-cache-v1.lock";
const BODY_LIMIT_CACHE_MARKER: &[u8] = b"bounded-response-body-v1\n";
static BODY_LIMIT_STAGE_NONCE: AtomicU64 = AtomicU64::new(0);

struct EpochStaging {
    path: std::path::PathBuf,
    file: File,
}

impl Drop for EpochStaging {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug)]
pub(crate) enum MigrationOutcome {
    Renamed,
    SkippedOldMissing,
    SkippedTargetPresent,
}

fn directory_exists(path: &Path, label: &str) -> Result<bool, io::Error> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(true),
        Ok(metadata) if metadata.file_type().is_symlink() => match fs::metadata(path) {
            Ok(target_metadata) if target_metadata.is_dir() => Ok(true),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{label} {} exists but is not a directory", path.display()),
            )),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{label} {} exists but points to a missing target",
                    path.display()
                ),
            )),
            Err(err) => Err(err),
        },
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{label} {} exists but is not a directory", path.display()),
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

pub(crate) fn migrate_http_cache(cache_root: &Path) -> Result<MigrationOutcome, io::Error> {
    let old = cache_root.join("http-cacache");
    if !directory_exists(&old, "legacy cache path")? {
        return Ok(MigrationOutcome::SkippedOldMissing);
    }
    let _maintenance = super::lock_cache_maintenance(cache_root)?;
    migrate_http_cache_locked(cache_root)
}

pub(crate) async fn migrate_http_cache_with_deadline(
    cache_root: &Path,
    deadline: &crate::sources::VariantArticleDeadline,
) -> Result<MigrationOutcome, io::Error> {
    if !directory_exists(&cache_root.join("http-cacache"), "legacy cache path")? {
        return Ok(MigrationOutcome::SkippedOldMissing);
    }
    let _maintenance = super::lock_cache_maintenance_until(cache_root, deadline).await?;
    deadline.ensure_time_io()?;
    let result = migrate_http_cache_locked(cache_root);
    deadline.ensure_time_io()?;
    result
}

fn migrate_http_cache_locked(cache_root: &Path) -> Result<MigrationOutcome, io::Error> {
    let old = cache_root.join("http-cacache");
    let new = cache_root.join("http");

    if !directory_exists(&old, "legacy cache path")? {
        return Ok(MigrationOutcome::SkippedOldMissing);
    }

    if directory_exists(&new, "runtime cache target")? {
        return Ok(MigrationOutcome::SkippedTargetPresent);
    }

    fs::rename(&old, &new)?;
    Ok(MigrationOutcome::Renamed)
}

pub(crate) fn ensure_body_limited_cache_epoch(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
) -> Result<(), io::Error> {
    ensure_body_limited_cache_epoch_with_observer(
        cache_root,
        legacy_cache_was_renamed,
        |_file, _path| Ok(()),
    )
}

fn ensure_body_limited_cache_epoch_with_observer<F>(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
    mut before_publish: F,
) -> Result<(), io::Error>
where
    F: FnMut(&File, &Path) -> io::Result<()>,
{
    fs::create_dir_all(cache_root)?;
    let lock = open_epoch_lock(cache_root)?;
    lock.lock_exclusive()?;
    ensure_body_limited_cache_epoch_locked(
        cache_root,
        legacy_cache_was_renamed,
        lock,
        &mut before_publish,
    )
}

pub(crate) async fn ensure_body_limited_cache_epoch_until(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
    deadline: &crate::sources::VariantArticleDeadline,
) -> Result<(), io::Error> {
    fs::create_dir_all(cache_root)?;
    let lock = open_epoch_lock(cache_root)?;
    loop {
        match lock.try_lock_exclusive() {
            Ok(()) => break,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                deadline
                    .run(tokio::time::sleep(std::time::Duration::from_millis(10)))
                    .await
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::TimedOut,
                            "variant article invocation deadline exceeded",
                        )
                    })?;
            }
            Err(error) => return Err(error),
        }
    }
    let maintenance = super::lock_cache_maintenance_until(cache_root, deadline).await?;
    deadline.ensure_time_io()?;
    ensure_body_limited_cache_epoch_locked_with_guard(
        cache_root,
        legacy_cache_was_renamed,
        lock,
        maintenance,
        &mut |_file, _path| Ok(()),
    )
}

fn open_epoch_lock(cache_root: &Path) -> io::Result<File> {
    let lock_path = cache_root.join(BODY_LIMIT_CACHE_LOCK);
    open_private(
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true),
        &lock_path,
    )
}

fn ensure_body_limited_cache_epoch_locked<F>(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
    lock: File,
    before_publish: &mut F,
) -> Result<(), io::Error>
where
    F: FnMut(&File, &Path) -> io::Result<()>,
{
    let maintenance = super::lock_cache_maintenance(cache_root)?;
    ensure_body_limited_cache_epoch_locked_with_guard(
        cache_root,
        legacy_cache_was_renamed,
        lock,
        maintenance,
        before_publish,
    )
}

fn ensure_body_limited_cache_epoch_locked_with_guard<F>(
    cache_root: &Path,
    legacy_cache_was_renamed: bool,
    lock: File,
    _maintenance: super::private::CacheOperationGuard,
    before_publish: &mut F,
) -> Result<(), io::Error>
where
    F: FnMut(&File, &Path) -> io::Result<()>,
{
    let marker = cache_root.join(BODY_LIMIT_CACHE_EPOCH);
    let legacy_cache = cache_root.join("http-cacache");

    let result = (|| {
        let marker_exists = validated_marker_exists(&marker)?;

        let legacy_cache_exists = match fs::symlink_metadata(&legacy_cache) {
            Ok(_) => true,
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };
        if marker_exists && !legacy_cache_was_renamed && !legacy_cache_exists {
            return Ok(());
        }

        if marker_exists && !legacy_cache_was_renamed {
            remove_cache_directory(&legacy_cache)?;
            return Ok(());
        }

        let cache_path = cache_root.join("http");
        remove_cache_directory(&cache_path)?;
        remove_cache_directory(&legacy_cache)?;
        fs::create_dir_all(&cache_path)?;

        let mut staging = create_epoch_staging(cache_root)?;
        staging.file.write_all(BODY_LIMIT_CACHE_MARKER)?;
        staging.file.sync_all()?;
        before_publish(&staging.file, &staging.path)?;
        if marker_exists {
            fs::remove_file(&marker)?;
        }
        fs::rename(&staging.path, &marker)?;
        if !validated_marker_exists(&marker)? {
            return Err(io::Error::other(
                "cache epoch marker disappeared after publication",
            ));
        }
        Ok(())
    })();
    let unlock_result = FileExt::unlock(&lock);
    result.and(unlock_result)
}

fn create_epoch_staging(cache_root: &Path) -> io::Result<EpochStaging> {
    for _ in 0..32 {
        let nonce = BODY_LIMIT_STAGE_NONCE.fetch_add(1, Ordering::Relaxed);
        let path = cache_root.join(format!(
            ".{BODY_LIMIT_CACHE_EPOCH}.tmp-{}-{nonce}",
            std::process::id()
        ));
        let opened = open_private(
            OpenOptions::new().read(true).write(true).create_new(true),
            &path,
        );
        match opened {
            Ok(file) => return Ok(EpochStaging { path, file }),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                let _ = fs::remove_file(path);
                return Err(error);
            }
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique cache epoch staging file",
    ))
}

fn open_existing_private(path: &Path) -> io::Result<Option<File>> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    match open_private(&mut options, path) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn validated_marker_exists(path: &Path) -> io::Result<bool> {
    let Some(mut marker) = open_existing_private(path)? else {
        return Ok(false);
    };
    marker.seek(SeekFrom::Start(0))?;
    let mut contents = Vec::with_capacity(BODY_LIMIT_CACHE_MARKER.len() + 1);
    marker
        .take((BODY_LIMIT_CACHE_MARKER.len() + 1) as u64)
        .read_to_end(&mut contents)?;
    if contents == BODY_LIMIT_CACHE_MARKER {
        Ok(true)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "cache epoch marker has invalid contents: {}",
                path.display()
            ),
        ))
    }
}

fn remove_cache_directory(path: &Path) -> Result<(), io::Error> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDirGuard;
    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn assert_invalid_input_contains(
        result: Result<MigrationOutcome, io::Error>,
        expected: &[&str],
    ) {
        match result {
            Err(err) => {
                assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
                let message = err.to_string();
                for needle in expected {
                    assert!(
                        message.contains(needle),
                        "expected error to contain {needle:?}, got: {message}"
                    );
                }
            }
            Ok(_) => panic!("expected invalid-input error"),
        }
    }

    #[test]
    fn renames_legacy_http_cache_directory_when_only_legacy_dir_exists() {
        let root = TempDirGuard::new("rename");
        let legacy_dir = root.path().join("http-cacache");
        let target_dir = root.path().join("http");
        std::fs::create_dir_all(&legacy_dir).expect("create legacy dir");
        let sentinel = legacy_dir.join("sentinel.txt");
        std::fs::write(&sentinel, b"cached payload").expect("write sentinel");

        let result = migrate_http_cache(root.path());

        assert!(matches!(result, Ok(MigrationOutcome::Renamed)));
        assert!(target_dir.is_dir(), "runtime dir should exist after rename");
        assert!(
            target_dir.join("sentinel.txt").is_file(),
            "sentinel file should move into runtime dir"
        );
        assert_eq!(
            std::fs::read(target_dir.join("sentinel.txt")).expect("read sentinel"),
            b"cached payload"
        );
        assert!(
            !legacy_dir.exists(),
            "legacy dir should not remain after successful rename"
        );
    }

    #[test]
    fn skips_when_legacy_http_cache_directory_is_missing() {
        let root = TempDirGuard::new("old-missing");

        let result = migrate_http_cache(root.path());

        assert!(matches!(result, Ok(MigrationOutcome::SkippedOldMissing)));
    }

    #[test]
    fn skips_when_runtime_http_directory_already_exists() {
        let root = TempDirGuard::new("target-present");
        let legacy_dir = root.path().join("http-cacache");
        let target_dir = root.path().join("http");
        std::fs::create_dir_all(&legacy_dir).expect("create legacy dir");
        std::fs::create_dir_all(&target_dir).expect("create target dir");
        std::fs::write(legacy_dir.join("legacy.txt"), b"legacy").expect("write legacy file");
        std::fs::write(target_dir.join("runtime.txt"), b"runtime").expect("write runtime file");

        let result = migrate_http_cache(root.path());

        assert!(matches!(result, Ok(MigrationOutcome::SkippedTargetPresent)));
        assert!(legacy_dir.join("legacy.txt").is_file());
        assert!(target_dir.join("runtime.txt").is_file());
    }

    #[test]
    fn errors_when_legacy_path_is_not_a_directory() {
        let root = TempDirGuard::new("legacy-file");
        std::fs::write(root.path().join("http-cacache"), b"not a dir").expect("write legacy file");

        assert_invalid_input_contains(
            migrate_http_cache(root.path()),
            &["legacy cache path", "not a directory"],
        );
    }

    #[test]
    fn errors_when_runtime_http_target_is_not_a_directory() {
        let root = TempDirGuard::new("target-file");
        std::fs::create_dir_all(root.path().join("http-cacache")).expect("create legacy dir");
        std::fs::write(root.path().join("http"), b"not a dir").expect("write target file");

        assert_invalid_input_contains(
            migrate_http_cache(root.path()),
            &["runtime cache target", "not a directory"],
        );
    }

    #[cfg(unix)]
    #[test]
    fn errors_when_legacy_path_is_a_dangling_symlink() {
        let root = TempDirGuard::new("legacy-dangling-symlink");
        symlink(
            root.path().join("missing-legacy"),
            root.path().join("http-cacache"),
        )
        .expect("create dangling legacy symlink");

        assert_invalid_input_contains(
            migrate_http_cache(root.path()),
            &["legacy cache path", "missing target"],
        );
    }

    #[test]
    fn body_limit_epoch_clears_legacy_entries_once() {
        let root = TempDirGuard::new("body-limit-epoch");
        let cache = root.path().join("http");
        std::fs::create_dir_all(&cache).expect("create cache");
        std::fs::write(cache.join("legacy-entry"), b"legacy").expect("seed legacy entry");

        ensure_body_limited_cache_epoch(root.path(), false).expect("migrate cache epoch");
        assert!(!cache.join("legacy-entry").exists());

        std::fs::write(cache.join("bounded-entry"), b"bounded").expect("seed bounded entry");
        ensure_body_limited_cache_epoch(root.path(), false).expect("repeat cache epoch");
        assert!(cache.join("bounded-entry").is_file());
    }

    #[test]
    fn body_limit_epoch_is_concurrent_and_idempotent() {
        let root = TempDirGuard::new("body-limit-epoch-concurrent");
        let root_path = std::sync::Arc::new(root.path().to_path_buf());
        let workers = (0..8)
            .map(|_| {
                let root_path = std::sync::Arc::clone(&root_path);
                std::thread::spawn(move || ensure_body_limited_cache_epoch(&root_path, false))
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker
                .join()
                .expect("worker should finish")
                .expect("epoch migration");
        }
        assert!(root.path().join(BODY_LIMIT_CACHE_EPOCH).is_file());
        assert!(root.path().join("http").is_dir());
    }

    #[tokio::test]
    async fn body_limit_epoch_lock_contention_obeys_variant_article_deadline() {
        let root = TempDirGuard::new("body-limit-epoch-deadline");
        fs::create_dir_all(root.path()).expect("cache root");
        let held = open_epoch_lock(root.path()).expect("epoch lock");
        held.lock_exclusive().expect("hold epoch lock");
        let deadline =
            crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_millis(20));

        let error = ensure_body_limited_cache_epoch_until(root.path(), false, &deadline)
            .await
            .expect_err("contended epoch lock must not outlive the invocation");

        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(!root.path().join(BODY_LIMIT_CACHE_EPOCH).exists());
        FileExt::unlock(&held).expect("release epoch lock");
    }

    #[tokio::test(start_paused = true)]
    async fn maintenance_contention_expires_without_partial_migration_and_releases_for_retry() {
        let root = TempDirGuard::new("migration-maintenance-deadline");
        fs::create_dir_all(root.path().join("http-cacache")).unwrap();
        fs::write(root.path().join("http-cacache/sentinel"), b"legacy").unwrap();
        let held = super::super::try_lock_cache_maintenance(root.path())
            .unwrap()
            .expect("hold maintenance lock");
        let deadline =
            crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_millis(20));

        let error = migrate_http_cache_with_deadline(root.path(), &deadline)
            .await
            .expect_err("contended migration must expire");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(
            fs::read(root.path().join("http-cacache/sentinel")).unwrap(),
            b"legacy"
        );
        assert!(!root.path().join("http").exists());

        drop(held);
        let retry =
            crate::sources::VariantArticleDeadline::from_now(std::time::Duration::from_secs(1));
        assert!(matches!(
            migrate_http_cache_with_deadline(root.path(), &retry)
                .await
                .unwrap(),
            MigrationOutcome::Renamed
        ));
        assert_eq!(
            fs::read(root.path().join("http/sentinel")).unwrap(),
            b"legacy"
        );
    }

    #[cfg(unix)]
    #[test]
    fn body_limit_epoch_observes_a_private_temp_and_removes_it_after_publication() {
        let root = TempDirGuard::new("body-limit-epoch-temp-observer");
        let mut observed = None;
        ensure_body_limited_cache_epoch_with_observer(root.path(), false, |file, path| {
            observed = Some(path.to_path_buf());
            assert_eq!(
                file.metadata()?.permissions().mode() & 0o777,
                0o600,
                "actual epoch staging file must be private before publication"
            );
            assert!(
                path.is_file(),
                "staging pathname must identify the opened file"
            );
            Ok(())
        })
        .expect("publish observed epoch marker");

        let staging = observed.expect("observe actual staging file");
        assert_eq!(
            fs::read(root.path().join(BODY_LIMIT_CACHE_EPOCH)).expect("published marker"),
            b"bounded-response-body-v1\n"
        );
        assert!(
            !staging.exists(),
            "successful publication must leave no staging entry"
        );
    }

    #[test]
    fn body_limit_epoch_ignores_crash_orphans_and_later_succeeds() {
        let root = TempDirGuard::new("body-limit-epoch-orphan");
        let orphan = root
            .path()
            .join(format!(".{BODY_LIMIT_CACHE_EPOCH}.tmp-crashed-process"));
        fs::write(&orphan, b"interrupted publication").expect("seed crash orphan");

        ensure_body_limited_cache_epoch(root.path(), false)
            .expect("orphan must not block a later publication");
        assert_eq!(
            fs::read(root.path().join(BODY_LIMIT_CACHE_EPOCH)).expect("published marker"),
            b"bounded-response-body-v1\n"
        );
        assert_eq!(
            fs::read(orphan).expect("crash orphan remains untouched"),
            b"interrupted publication"
        );
    }

    #[test]
    fn body_limit_epoch_cleans_its_staging_file_after_an_ordinary_error() {
        let root = TempDirGuard::new("body-limit-epoch-error-cleanup");
        let mut staging = None;
        let error =
            ensure_body_limited_cache_epoch_with_observer(root.path(), false, |_file, path| {
                staging = Some(path.to_path_buf());
                Err(io::Error::other("injected publication failure"))
            })
            .expect_err("injected error must be returned");

        assert_eq!(error.to_string(), "injected publication failure");
        assert!(!staging.expect("observe staging path").exists());
        assert!(!root.path().join(BODY_LIMIT_CACHE_EPOCH).exists());
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial(cache_epoch_umask)]
    fn body_limit_epoch_files_are_private_and_existing_modes_are_repaired() {
        struct UmaskGuard(libc::mode_t);
        impl Drop for UmaskGuard {
            fn drop(&mut self) {
                // SAFETY: this test is serialized with every test that changes the process umask.
                unsafe { libc::umask(self.0) };
            }
        }

        // SAFETY: this test is serialized with every test that changes the process umask.
        let original = unsafe { libc::umask(0) };
        let _guard = UmaskGuard(original);
        let root = TempDirGuard::new("body-limit-epoch-private");

        ensure_body_limited_cache_epoch(root.path(), false).expect("create private epoch files");
        for name in [BODY_LIMIT_CACHE_LOCK, BODY_LIMIT_CACHE_EPOCH] {
            let path = root.path().join(name);
            assert_eq!(
                fs::metadata(&path)
                    .expect("epoch metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600,
                "{name} should be private"
            );
            fs::set_permissions(&path, fs::Permissions::from_mode(0o664))
                .expect("broaden epoch mode");
        }

        ensure_body_limited_cache_epoch(root.path(), false).expect("repair epoch files");
        for name in [BODY_LIMIT_CACHE_LOCK, BODY_LIMIT_CACHE_EPOCH] {
            assert_eq!(
                fs::metadata(root.path().join(name))
                    .expect("repaired epoch metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600,
                "{name} should be repaired on the current fast path"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn body_limit_epoch_rejects_linked_control_files_without_touching_targets() {
        for name in [
            BODY_LIMIT_CACHE_LOCK.to_string(),
            BODY_LIMIT_CACHE_EPOCH.to_string(),
        ] {
            for hard_link in [false, true] {
                let root = TempDirGuard::new("body-limit-epoch-linked");
                let outside = root.path().join("outside");
                fs::write(&outside, b"outside sentinel").expect("outside target");
                let entry = root.path().join(&name);
                if hard_link {
                    fs::hard_link(&outside, &entry).expect("hard link control entry");
                } else {
                    symlink(&outside, &entry).expect("symlink control entry");
                }

                let error = ensure_body_limited_cache_epoch(root.path(), false)
                    .expect_err("linked control entry must be rejected");
                assert!(
                    matches!(
                        error.kind(),
                        io::ErrorKind::InvalidInput | io::ErrorKind::AlreadyExists
                    ),
                    "unexpected error for {name}: {error}"
                );
                assert_eq!(
                    fs::read(&outside).expect("outside target"),
                    b"outside sentinel"
                );
            }
        }
    }

    #[cfg(windows)]
    #[test]
    fn body_limit_epoch_rejects_windows_hard_links_and_reparse_points() {
        use std::os::windows::fs::symlink_file;

        for name in [
            BODY_LIMIT_CACHE_LOCK.to_string(),
            BODY_LIMIT_CACHE_EPOCH.to_string(),
        ] {
            for hard_link in [false, true] {
                let root = TempDirGuard::new("body-limit-epoch-windows-linked");
                let outside = root.path().join("outside");
                fs::write(&outside, b"outside sentinel").expect("outside target");
                let entry = root.path().join(&name);
                if hard_link {
                    fs::hard_link(&outside, &entry).expect("hard link control entry");
                } else {
                    symlink_file(&outside, &entry).expect("reparse control entry");
                }

                ensure_body_limited_cache_epoch(root.path(), false)
                    .expect_err("linked Windows control entry must be rejected");
                assert_eq!(
                    fs::read(&outside).expect("outside target"),
                    b"outside sentinel"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn errors_when_runtime_http_target_is_a_dangling_symlink() {
        let root = TempDirGuard::new("target-dangling-symlink");
        std::fs::create_dir_all(root.path().join("http-cacache")).expect("create legacy dir");
        symlink(root.path().join("missing-target"), root.path().join("http"))
            .expect("create dangling runtime symlink");

        assert_invalid_input_contains(
            migrate_http_cache(root.path()),
            &["runtime cache target", "missing target"],
        );
    }
}
