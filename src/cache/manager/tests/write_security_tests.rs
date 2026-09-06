#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::Path;
use std::sync::{Arc, Barrier, Mutex, mpsc};
use std::time::Duration;

use http_cache::CacheManager;

use super::*;

fn inert_manager(cache_root: &Path) -> SizeAwareCacheManager {
    SizeAwareCacheManager::new_with_services(
        cache_root.join("http"),
        test_config(cache_root, u64::MAX / 2, DiskFreeThreshold::Percent(1)),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
    )
}

fn unix_mode(path: &Path) -> u32 {
    fs::symlink_metadata(path)
        .expect("path metadata")
        .permissions()
        .mode()
        & 0o777
}

fn wait_until_exhausted(deadline: &crate::sources::VariantArticleDeadline) {
    while !deadline.is_exhausted() {
        std::thread::yield_now();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn put_deadline_transition_is_exactly_at_writer_commit() {
    let before_root = TempDirGuard::new("put-before-publication-deadline");
    let before_deadline =
        crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(50));
    let wait_deadline = before_deadline.clone();
    let before_commit = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed_commit = Arc::clone(&before_commit);
    let before_manager = SizeAwareCacheManager::new_with_services_and_observers(
        before_root.path().join("http"),
        test_config(
            before_root.path(),
            u64::MAX / 2,
            DiskFreeThreshold::Percent(1),
        ),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
        |_, _| unreachable!("timed-out pre-publication put must not finalize"),
        move |_| wait_until_exhausted(&wait_deadline),
    )
    .with_safe_return_observer(move || {
        let observed_commit = Arc::clone(&observed_commit);
        async move { observed_commit.store(true, std::sync::atomic::Ordering::SeqCst) }
    });
    let result = crate::sources::with_variant_article_deadline(
        before_deadline,
        before_manager.put(
            "before".into(),
            test_http_response(b"before"),
            test_policy(),
        ),
    )
    .await;
    assert!(result.is_err(), "pre-publication work remains cancellable");
    assert!(!before_commit.load(std::sync::atomic::Ordering::SeqCst));
    assert!(
        cacache::metadata(before_root.path().join("http"), "before")
            .await
            .expect("metadata lookup")
            .is_none(),
        "pre-publication timeout must not publish an index entry"
    );

    let commit_root = TempDirGuard::new("put-at-publication-deadline");
    let commit_deadline =
        crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(50));
    let wait_deadline = commit_deadline.clone();
    let commit_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed_commit = Arc::clone(&commit_started);
    let published_commit = Arc::clone(&commit_started);
    let commit_manager = SizeAwareCacheManager::new_with_services_and_observers(
        commit_root.path().join("http"),
        test_config(
            commit_root.path(),
            u64::MAX / 2,
            DiskFreeThreshold::Percent(1),
        ),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
        move |path, key| {
            assert!(crate::sources::current_variant_article_deadline().is_none());
            assert!(published_commit.load(std::sync::atomic::Ordering::SeqCst));
            assert!(cacache::metadata_sync(path, key).unwrap().is_some());
            wait_until_exhausted(&wait_deadline);
        },
        |_| {},
    )
    .with_safe_return_observer(move || {
        let observed_commit = Arc::clone(&observed_commit);
        async move {
            assert!(crate::sources::current_variant_article_deadline().is_some());
            observed_commit.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
    let result = crate::sources::with_variant_article_deadline(
        commit_deadline,
        commit_manager.put(
            "commit".into(),
            test_http_response(b"commit"),
            test_policy(),
        ),
    )
    .await;
    assert!(
        result.is_ok(),
        "commit and finalization must settle safely: {result:?}"
    );
    assert!(commit_started.load(std::sync::atomic::Ordering::SeqCst));
    let cached = commit_manager
        .get("commit")
        .await
        .expect("cache get")
        .expect("published entry");
    assert_eq!(cached.0.body, b"commit");
}

fn assert_write_path_modes(cache_path: &Path, key: &str, integrity: &ssri::Integrity) {
    let index_bucket = crate::cache::index_bucket_path(cache_path, key);
    let blob = crate::cache::content_path(cache_path, integrity);
    let content_root = crate::cache::content_root(cache_path);
    let (algorithm, _) = integrity.to_hex();
    let directories = [
        cache_path.to_path_buf(),
        cache_path.join("tmp"),
        cache_path.join("index-v5"),
        index_bucket
            .parent()
            .and_then(Path::parent)
            .expect("first index shard")
            .to_path_buf(),
        index_bucket
            .parent()
            .expect("second index shard")
            .to_path_buf(),
        content_root.clone(),
        content_root.join(algorithm.to_string()),
        blob.parent()
            .and_then(Path::parent)
            .expect("first content shard")
            .to_path_buf(),
        blob.parent().expect("second content shard").to_path_buf(),
    ];
    assert_eq!(unix_mode(&index_bucket), 0o600);
    assert_eq!(unix_mode(&blob), 0o600);
    let cache_root = cache_path.parent().expect("cache root");
    assert_eq!(unix_mode(&cache_root.join(".biomcp-operation.lock")), 0o600);
    assert_eq!(
        unix_mode(&crate::cache::key_lock_path(cache_root, key)),
        0o600
    );
    assert_eq!(unix_mode(&cache_root.join(".biomcp-key-locks")), 0o700);
    for directory in directories {
        assert_eq!(unix_mode(&directory), 0o700, "{}", directory.display());
    }
}

#[test]
fn estimate_cache_bytes_fast_handles_missing_and_populated_content_trees() {
    let root = TempDirGuard::new("estimate-fast");
    let cache_path = root.path().join("http");
    assert_eq!(estimate_cache_bytes_fast(&cache_path).unwrap(), 0);
    let content = cache_path.join("content-v2/sha256/aa/bb");
    fs::create_dir_all(&content).unwrap();
    fs::write(content.join("blob-a"), b"abc").unwrap();
    fs::write(content.join("blob-b"), b"defgh").unwrap();
    assert_eq!(estimate_cache_bytes_fast(&cache_path).unwrap(), 8);
}

fn faulting_manager(
    cache_root: &Path,
    delegated: Arc<std::sync::atomic::AtomicBool>,
    break_metadata_read: bool,
) -> SizeAwareCacheManager {
    SizeAwareCacheManager::new_with_services_and_observers(
        cache_root.join("http"),
        test_config(cache_root, u64::MAX / 2, DiskFreeThreshold::Percent(1)),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
        move |path, key| {
            assert!(
                cacache::metadata_sync(path, key)
                    .expect("delegated metadata read")
                    .is_some(),
                "the delegated CACache write must complete before fault injection"
            );
            delegated.store(true, std::sync::atomic::Ordering::SeqCst);
            if break_metadata_read {
                let bucket = crate::cache::index_bucket_path(path, key);
                fs::remove_file(&bucket).expect("remove written metadata bucket");
                fs::write(&bucket, b"not cacache metadata").expect("corrupt metadata bucket");
            } else {
                cacache::remove_sync(path, key).expect("remove written metadata");
            }
        },
        |_| {},
    )
}

#[tokio::test(flavor = "current_thread")]
async fn successful_put_with_missing_metadata_fails_closed_with_stable_context() {
    let root = TempDirGuard::new("missing-post-put-metadata");
    let delegated = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let error = faulting_manager(root.path(), Arc::clone(&delegated), false)
        .put(
            "sensitive-cache-key".into(),
            test_http_response(b"secret body"),
            test_policy(),
        )
        .await
        .expect_err("missing metadata after a delegated write must fail closed");

    assert!(delegated.load(std::sync::atomic::Ordering::SeqCst));
    assert!(
        error
            .to_string()
            .contains("cache security finalization failed after successful put")
    );
    assert!(!error.to_string().contains("sensitive-cache-key"));
    assert!(!error.to_string().contains("secret body"));
    assert!(
        !error
            .to_string()
            .contains(&root.path().display().to_string())
    );
}

#[tokio::test(flavor = "current_thread")]
async fn successful_put_with_metadata_read_error_fails_closed_with_stable_context() {
    let root = TempDirGuard::new("failed-post-put-metadata-read");
    let delegated = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let error = faulting_manager(root.path(), Arc::clone(&delegated), true)
        .put(
            "sensitive-cache-key".into(),
            test_http_response(b"secret body"),
            test_policy(),
        )
        .await
        .expect_err("metadata read error after a delegated write must fail closed");

    assert!(delegated.load(std::sync::atomic::Ordering::SeqCst));
    assert!(
        error
            .to_string()
            .contains("cache security finalization failed after successful put")
    );
    assert!(!error.to_string().contains("sensitive-cache-key"));
    assert!(!error.to_string().contains("secret body"));
    assert!(
        !error
            .to_string()
            .contains(&root.path().display().to_string())
    );
}

#[test]
fn contended_constructor_defers_cleanup_until_a_later_uncontended_constructor() {
    let root = TempDirGuard::new("deferred-open-age-maintenance");
    let cache_path = root.path().join("http");
    write_entry(&cache_path, "expired", b"old", 1_000);
    let mut config = test_config(root.path(), 1_000_000, DiskFreeThreshold::Percent(1));
    config.max_age = Duration::from_secs(10);
    let active = crate::cache::lock_cache_shared(root.path()).expect("active shared operation");

    SizeAwareCacheManager::new_at(cache_path.clone(), config.clone(), 20_000)
        .expect("contended manager construction");
    assert!(
        cacache::metadata_sync(&cache_path, "expired")
            .expect("deferred metadata")
            .is_some(),
        "contended constructor must defer destructive cleanup"
    );

    drop(active);
    SizeAwareCacheManager::new_at(cache_path.clone(), config, 20_000)
        .expect("uncontended manager construction");
    assert!(
        cacache::metadata_sync(&cache_path, "expired")
            .expect("reclaimed metadata")
            .is_none(),
        "later uncontended constructor must reclaim the expired entry"
    );
}

struct UmaskGuard(libc::mode_t);

impl Drop for UmaskGuard {
    fn drop(&mut self) {
        // SAFETY: these tests are serialized with every test that changes the process umask.
        unsafe { libc::umask(self.0) };
    }
}

#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(cache_epoch_umask)]
async fn put_secures_only_its_exact_paths_under_a_permissive_umask() {
    // SAFETY: this test is serialized with every test that changes the process umask.
    let original = unsafe { libc::umask(0) };
    let _guard = UmaskGuard(original);
    let root = TempDirGuard::new("bounded-put-hardening");
    let manager = inert_manager(root.path());
    let unrelated = root.path().join("http/unrelated/nested/sentinel");
    fs::create_dir_all(unrelated.parent().expect("sentinel parent")).expect("unrelated tree");
    fs::write(&unrelated, b"must remain untouched").expect("sentinel");
    fs::set_permissions(
        unrelated.parent().expect("sentinel parent"),
        fs::Permissions::from_mode(0o755),
    )
    .expect("permissive sentinel parent");
    fs::set_permissions(&unrelated, fs::Permissions::from_mode(0o644))
        .expect("permissive sentinel");
    fs::hard_link(&unrelated, root.path().join("outside-link")).expect("unrelated hard link");

    let key = "https://example.test/cache?q=é";
    manager
        .put(
            key.into(),
            test_http_response(b"private body"),
            test_policy(),
        )
        .await
        .expect("put must not inspect unrelated hard link");

    assert_eq!(
        fs::read(&unrelated).expect("sentinel bytes"),
        b"must remain untouched"
    );
    assert_eq!(unix_mode(&unrelated), 0o644);
    assert_eq!(
        unix_mode(unrelated.parent().expect("sentinel parent")),
        0o755
    );

    let cache_path = root.path().join("http");
    let metadata = cacache::metadata_sync(&cache_path, key)
        .expect("cache metadata")
        .expect("written metadata");
    assert_write_path_modes(&cache_path, key, &metadata.integrity);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[serial_test::serial(cache_epoch_umask)]
async fn concurrent_same_key_puts_keep_metadata_attributable_until_hardening_finishes() {
    // SAFETY: this test is serialized with every test that changes the process umask.
    let original = unsafe { libc::umask(0) };
    let _guard = UmaskGuard(original);
    let root = TempDirGuard::new("coordinated-same-key-put");
    let cache_path = root.path().join("http");
    let key = "same-key";
    let (a_delegated_tx, a_delegated_rx) = mpsc::channel();
    let (release_a_tx, release_a_rx) = mpsc::channel();
    let release_a_rx = Arc::new(Mutex::new(release_a_rx));
    let manager_a = SizeAwareCacheManager::new_with_services_and_observers(
        cache_path.clone(),
        test_config(root.path(), u64::MAX / 2, DiskFreeThreshold::Percent(1)),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
        {
            let release_a_rx = Arc::clone(&release_a_rx);
            move |path, cache_key| {
                let integrity = cacache::metadata_sync(path, cache_key)
                    .expect("A metadata")
                    .expect("A written metadata")
                    .integrity;
                a_delegated_tx.send(integrity).expect("report A delegation");
                release_a_rx
                    .lock()
                    .expect("release receiver")
                    .recv()
                    .expect("release A");
            }
        },
        |_| {},
    );
    let (b_delegated_tx, b_delegated_rx) = mpsc::channel();
    let manager_b = SizeAwareCacheManager::new_with_services_and_observers(
        cache_path.clone(),
        test_config(root.path(), u64::MAX / 2, DiskFreeThreshold::Percent(1)),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
        move |_, _| {
            let _ = b_delegated_tx.send(());
        },
        |_| {},
    );

    let a = tokio::spawn(async move {
        manager_a
            .put(key.into(), test_http_response(b"body-a"), test_policy())
            .await
    });
    let a_integrity = tokio::task::spawn_blocking(move || {
        a_delegated_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("A reaches post-delegation seam")
    })
    .await
    .expect("A observer task");
    let b = tokio::spawn(async move {
        manager_b
            .put(key.into(), test_http_response(b"body-b"), test_policy())
            .await
    });
    let b_was_blocked = tokio::task::spawn_blocking(move || {
        b_delegated_rx
            .recv_timeout(Duration::from_millis(200))
            .is_err()
    })
    .await
    .expect("B observer task");
    assert!(
        b_was_blocked,
        "B must not delegate while A attributes and secures its write"
    );
    release_a_tx.send(()).expect("release A");
    a.await.expect("A task").expect("A put");
    b.await.expect("B task").expect("B put");

    assert_write_path_modes(&cache_path, key, &a_integrity);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_cache_different_shard_puts_tolerate_common_directory_creation_races() {
    let root = TempDirGuard::new("cold-different-shard-puts");
    let cache_path = root.path().join("http");
    let key_a = "cold-key-a";
    let key_b = (0..10_000)
        .map(|candidate| format!("cold-key-{candidate}"))
        .find(|candidate| {
            crate::cache::key_lock_path(root.path(), candidate)
                != crate::cache::key_lock_path(root.path(), key_a)
        })
        .expect("key in another lock shard");
    let barrier = Arc::new(Barrier::new(2));
    let manager = |barrier: Arc<Barrier>| {
        SizeAwareCacheManager::new_with_services_and_observers(
            cache_path.clone(),
            test_config(root.path(), u64::MAX / 2, DiskFreeThreshold::Percent(1)),
            |_| Ok(0),
            |_| {
                Ok(FilesystemSpace {
                    available_bytes: 99,
                    total_bytes: 100,
                })
            },
            |_, _, _, _| unreachable!("test cache must not schedule eviction"),
            |_, _| {},
            move |_| {
                barrier.wait();
            },
        )
    };
    let manager_a = manager(Arc::clone(&barrier));
    let manager_b = manager(barrier);
    let put_a = tokio::spawn(async move {
        manager_a
            .put(key_a.into(), test_http_response(b"body-a"), test_policy())
            .await
    });
    let put_b = tokio::spawn(async move {
        manager_b
            .put(key_b.clone(), test_http_response(b"body-b"), test_policy())
            .await
            .map(|response| (response, key_b))
    });

    put_a.await.expect("first put task").expect("first put");
    let (_, key_b) = put_b.await.expect("second put task").expect("second put");
    assert!(
        cacache::metadata_sync(&cache_path, key_a)
            .expect("first metadata")
            .is_some()
    );
    assert!(
        cacache::metadata_sync(&cache_path, &key_b)
            .expect("second metadata")
            .is_some()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn eviction_waits_for_an_active_cache_operation() {
    let root = TempDirGuard::new("coordinated-eviction");
    let cache_path = root.path().join("http");
    let config = test_config(root.path(), 1_000, DiskFreeThreshold::Percent(1));
    let operation = crate::cache::lock_cache_shared(root.path()).expect("hold operation lock");
    let (started_tx, started_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let eviction = tokio::task::spawn_blocking(move || {
        started_tx.send(()).expect("report eviction start");
        let result = super::super::run_eviction_cycle(
            &cache_path,
            &config,
            &std::sync::atomic::AtomicU64::new(0),
        );
        finished_tx.send(()).expect("report eviction finish");
        result
    });
    started_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("eviction task started");
    assert!(
        finished_rx
            .recv_timeout(Duration::from_millis(200))
            .is_err(),
        "eviction must wait for the active cache operation"
    );
    drop(operation);
    eviction
        .await
        .expect("eviction task")
        .expect("eviction cycle");
}

#[test]
#[serial_test::serial(cache_epoch_umask)]
fn cacache_atomic_temporary_file_is_born_private() {
    // SAFETY: this test is serialized with every test that changes the process umask.
    let original = unsafe { libc::umask(0) };
    let _guard = UmaskGuard(original);
    let root = TempDirGuard::new("cacache-private-temp");
    let cache_path = root.path().join("http");
    crate::cache::prepare_write_paths(&cache_path, "temporary-file-test")
        .expect("prepare cache paths");
    let writer = cacache::WriteOpts::new()
        .size(1)
        .open_sync(&cache_path, "temporary-file-test")
        .expect("open writer");
    let entries = fs::read_dir(cache_path.join("tmp"))
        .expect("read temp directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("temp entries");
    assert_eq!(
        entries.len(),
        1,
        "writer should own one live temporary file"
    );
    assert_eq!(unix_mode(&entries[0].path()), 0o600);
    drop(writer);
}

#[tokio::test(flavor = "current_thread")]
async fn put_rejects_linked_index_bucket_and_each_linked_ancestor() {
    for hostile_level in 0..4 {
        let root = TempDirGuard::new("hostile-index-path");
        let cache_path = root.path().join("http");
        let key = format!("hostile-index-{hostile_level}");
        let bucket = crate::cache::index_bucket_path(&cache_path, &key);
        let paths = [
            cache_path.join("index-v5"),
            bucket
                .parent()
                .and_then(Path::parent)
                .expect("first shard")
                .to_path_buf(),
            bucket.parent().expect("second shard").to_path_buf(),
            bucket.clone(),
        ];
        fs::create_dir_all(paths[hostile_level].parent().expect("hostile parent"))
            .expect("hostile parent tree");
        let outside = root.path().join("outside");
        if hostile_level == 3 {
            fs::write(&outside, b"outside bytes").expect("outside file");
            symlink(&outside, &paths[hostile_level]).expect("hostile file symlink");
        } else {
            fs::create_dir(&outside).expect("outside directory");
            symlink(&outside, &paths[hostile_level]).expect("hostile directory symlink");
        }

        let error = inert_manager(root.path())
            .put(key, test_http_response(b"body"), test_policy())
            .await
            .expect_err("linked derived path must fail before delegation");
        assert!(
            error.to_string().contains("not a directory") || error.to_string().contains("symlink")
        );
        if hostile_level == 3 {
            assert_eq!(
                fs::read(&outside).expect("outside unchanged"),
                b"outside bytes"
            );
        } else {
            assert_eq!(
                fs::read_dir(&outside).expect("outside directory").count(),
                0
            );
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn put_rejects_a_multiply_linked_index_bucket() {
    let root = TempDirGuard::new("hard-linked-index-bucket");
    let cache_path = root.path().join("http");
    let key = "hard-linked-index";
    let bucket = crate::cache::index_bucket_path(&cache_path, key);
    fs::create_dir_all(bucket.parent().expect("bucket parent")).expect("index tree");
    fs::write(&bucket, b"outside bytes").expect("bucket seed");
    let outside = root.path().join("outside-index");
    fs::hard_link(&bucket, &outside).expect("hard link");

    inert_manager(root.path())
        .put(key.into(), test_http_response(b"body"), test_policy())
        .await
        .expect_err("multiply-linked bucket must fail before delegation");

    assert_eq!(
        fs::read(&outside).expect("outside unchanged"),
        b"outside bytes"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn put_atomically_replaces_a_hostile_content_destination() {
    let root = TempDirGuard::new("hostile-content-destination");
    let cache_path = root.path().join("http");
    let manager = inert_manager(root.path());
    let key = "hostile-content";
    let response = test_http_response(b"body");
    let policy = test_policy();
    manager
        .put(key.into(), response.clone(), policy.clone())
        .await
        .expect("seed put");
    let metadata = cacache::metadata_sync(&cache_path, key)
        .expect("metadata")
        .expect("seed metadata");
    let blob = crate::cache::content_path(&cache_path, &metadata.integrity);
    fs::remove_file(&blob).expect("remove seed blob");
    let outside = root.path().join("outside-content");
    fs::write(&outside, b"outside bytes").expect("outside file");
    symlink(&outside, &blob).expect("hostile content symlink");

    manager
        .put(key.into(), response, policy)
        .await
        .expect("replacement put");

    assert_eq!(
        fs::read(&outside).expect("outside unchanged"),
        b"outside bytes"
    );
    let blob_metadata = fs::symlink_metadata(&blob).expect("replacement blob");
    assert!(blob_metadata.is_file());
    assert_eq!(blob_metadata.nlink(), 1);
    assert_eq!(unix_mode(&blob), 0o600);
}
