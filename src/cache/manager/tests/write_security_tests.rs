#![cfg(unix)]

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::Path;

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
    let index_bucket = crate::cache::index_bucket_path(&cache_path, key);
    let metadata = cacache::metadata_sync(&cache_path, key)
        .expect("cache metadata")
        .expect("written metadata");
    let blob = crate::cache::content_path(&cache_path, &metadata.integrity);
    assert_eq!(unix_mode(&index_bucket), 0o600);
    assert_eq!(unix_mode(&blob), 0o600);
    for directory in [
        cache_path.clone(),
        cache_path.join("tmp"),
        cache_path.join("index-v5"),
        index_bucket.parent().expect("bucket parent").to_path_buf(),
        blob.parent().expect("blob parent").to_path_buf(),
    ] {
        assert_eq!(unix_mode(&directory), 0o700, "{}", directory.display());
    }
}

#[test]
#[serial_test::serial(cache_epoch_umask)]
fn cacache_atomic_temporary_file_is_born_private() {
    // SAFETY: this test is serialized with every test that changes the process umask.
    let original = unsafe { libc::umask(0) };
    let _guard = UmaskGuard(original);
    let root = TempDirGuard::new("cacache-private-temp");
    let cache_path = root.path().join("http");
    super::super::prepare_write_paths(&cache_path, "temporary-file-test")
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
