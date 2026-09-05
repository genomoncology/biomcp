#![cfg(unix)]

use std::os::unix::fs::{PermissionsExt, symlink};

use super::*;

#[test]
fn concurrent_constructor_does_not_wait_for_an_existing_shared_operation() {
    let root = TempDirGuard::new("parallel-http-cache-construction");
    crate::cache::ensure_body_limited_cache_epoch(root.path(), false).expect("seed cache epoch");
    let existing = crate::cache::lock_cache_shared(root.path()).expect("shared cache operation");
    let config = test_cache_config(root.path());
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();

    let constructor = std::thread::spawn(move || {
        let result = build_http_client_with_config(SharedHttpClientKind::Default, config, None);
        finished_tx
            .send(result.map(|_| ()))
            .expect("report constructor result");
    });
    finished_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("constructor must not wait for an unrelated shared operation")
        .expect("construct HTTP client");

    drop(existing);
    constructor.join().expect("constructor thread");
}

#[test]
fn build_http_client_repairs_unrelated_permissive_cache_state() {
    let root = TempDirGuard::new("http-cache-permission-repair");
    let cache_root = root.path().join("cache-root");
    let sentinel = cache_root.join("http/unrelated/sentinel");
    std::fs::create_dir_all(sentinel.parent().expect("sentinel parent")).expect("cache tree");
    std::fs::write(&sentinel, b"cached response").expect("sentinel");
    std::fs::write(
        cache_root.join(".body-limit-cache-v1"),
        b"bounded-response-body-v1\n",
    )
    .expect("current cache epoch");
    std::fs::set_permissions(&sentinel, std::fs::Permissions::from_mode(0o644))
        .expect("permissive sentinel");

    build_http_client_with_config(
        SharedHttpClientKind::Default,
        test_cache_config(&cache_root),
        None,
    )
    .expect("client construction repairs cache state");

    assert_eq!(
        std::fs::metadata(&sentinel)
            .expect("sentinel metadata")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}

#[test]
fn build_http_client_rejects_directory_symlink_inside_content_tree() {
    let root = TempDirGuard::new("http-cache-content-link");
    let cache_root = root.path().join("cache-root");
    let outside = root.path().join("outside");
    std::fs::create_dir_all(cache_root.join("http/content-v2/sha256")).expect("content tree");
    std::fs::write(
        cache_root.join(".body-limit-cache-v1"),
        b"bounded-response-body-v1\n",
    )
    .expect("current cache epoch");
    std::fs::create_dir(&outside).expect("outside directory");
    symlink(&outside, cache_root.join("http/content-v2/sha256/aa"))
        .expect("content directory symlink");

    let error = match build_http_client_with_config(
        SharedHttpClientKind::Default,
        test_cache_config(&cache_root),
        None,
    ) {
        Ok(_) => panic!("content directory symlink must be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("managed content directory"));
    assert_eq!(
        std::fs::read_dir(&outside)
            .expect("outside directory")
            .count(),
        0
    );
}
