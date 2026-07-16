use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use fs2::FileExt;

const BODY_LIMIT_CACHE_EPOCH: &str = ".body-limit-cache-v1";
const BODY_LIMIT_CACHE_LOCK: &str = ".body-limit-cache-v1.lock";

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
    fs::create_dir_all(cache_root)?;
    let marker = cache_root.join(BODY_LIMIT_CACHE_EPOCH);
    let legacy_cache = cache_root.join("http-cacache");
    if marker.is_file() && !legacy_cache_was_renamed && !legacy_cache.exists() {
        return Ok(());
    }

    let lock_path = cache_root.join(BODY_LIMIT_CACHE_LOCK);
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)?;
    lock.lock_exclusive()?;

    let result = (|| {
        if marker.is_file() && !legacy_cache_was_renamed {
            remove_cache_directory(&legacy_cache)?;
            return Ok(());
        }

        let cache_path = cache_root.join("http");
        remove_cache_directory(&cache_path)?;
        remove_cache_directory(&legacy_cache)?;
        fs::create_dir_all(&cache_path)?;

        let temporary = cache_root.join(format!(
            ".{BODY_LIMIT_CACHE_EPOCH}.tmp-{}",
            std::process::id()
        ));
        let mut file = File::create(&temporary)?;
        file.write_all(b"bounded-response-body-v1\n")?;
        file.sync_all()?;
        if marker.exists() {
            fs::remove_file(&marker)?;
        }
        fs::rename(&temporary, &marker)?;
        Ok(())
    })();
    let unlock_result = FileExt::unlock(&lock);
    result.and(unlock_result)
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
    use std::os::unix::fs::symlink;

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
