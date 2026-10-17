//! A held cache lock must enter the caller's deadline retry path on every platform.
// Include the production adapter without widening its visibility for tests.
#[path = "../src/utils/fs_lock.rs"]
mod fs_lock;

use std::fs::OpenOptions;
use std::io::ErrorKind;

#[test]
fn contended_cache_locks_report_would_block() {
    let root = tempfile::tempdir().expect("isolated lock directory");
    let path = root.path().join("cache.lock");
    let first = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .expect("first lock handle");
    let second = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .expect("independent second lock handle");
    first.lock().expect("hold the exclusive cache lock");

    for (mode, result) in [
        ("exclusive", fs_lock::try_lock_exclusive(&second)),
        ("shared", fs_lock::try_lock_shared(&second)),
    ] {
        let error = result.expect_err("the second handle must contend");
        assert_eq!(error.kind(), ErrorKind::WouldBlock, "{mode}: {error}");
    }
    first.unlock().expect("release the held cache lock");
}
