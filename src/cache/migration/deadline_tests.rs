//! Deadline result-ordering and settlement tests for cache migration I/O.

use super::*;
use crate::test_support::TempDirGuard;
use fs2::FileExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[tokio::test]
async fn body_limit_epoch_lock_contention_obeys_variant_article_deadline() {
    let root = TempDirGuard::new("body-limit-epoch-deadline");
    fs::create_dir_all(root.path()).expect("cache root");
    let held = open_epoch_lock(root.path()).expect("epoch lock");
    held.lock_exclusive().expect("hold epoch lock");
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(20));

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
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(20));

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
    let retry = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(1));
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

#[tokio::test(start_paused = true)]
async fn epoch_cleanup_stops_mutating_after_a_mid_traversal_deadline() {
    let root = TempDirGuard::new("epoch-mid-cleanup-deadline");
    let cache = root.path().join("http");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("a"), b"first").unwrap();
    fs::write(cache.join("b"), b"second").unwrap();
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(10));
    let epoch_lock = open_epoch_lock(root.path()).unwrap();
    epoch_lock.try_lock_exclusive().unwrap();
    let maintenance = super::super::lock_cache_maintenance_until(root.path(), &deadline)
        .await
        .unwrap();
    let paused = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let driving = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let driver = tokio::spawn({
        let driving = Arc::clone(&driving);
        async move {
            while driving.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        }
    });
    let cleanup = ensure_body_limited_cache_epoch_async(
        root.path(),
        epoch_lock,
        maintenance,
        EpochState::Rebuild,
        &deadline,
        {
            let (paused, release) = (Arc::clone(&paused), Arc::clone(&release));
            move |path| {
                let (paused, release) = (Arc::clone(&paused), Arc::clone(&release));
                async move {
                    if path.file_name().is_some_and(|name| name == "a") {
                        paused.notify_one();
                        release.notified().await;
                    }
                }
            }
        },
    );
    tokio::pin!(cleanup);
    tokio::select! {
        () = paused.notified() => {}
        result = &mut cleanup => panic!("cleanup settled before injected pause: {result:?}"),
    }
    driving.store(false, Ordering::SeqCst);
    driver.await.unwrap();
    tokio::time::advance(Duration::from_secs(11)).await;
    assert!(cache.join("b").is_file());
    release.notify_one();
    assert_eq!(cleanup.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
    assert!(cache.join("b").is_file());
    assert!(!root.path().join(BODY_LIMIT_CACHE_EPOCH).exists());
}

#[tokio::test]
async fn synchronous_staging_window_is_deadline_raced_responsive_and_orphan_free() {
    let root = TempDirGuard::new("epoch-staging-window-deadline");
    fs::create_dir_all(root.path()).unwrap();
    let entered = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(20));
    let task = tokio::spawn({
        let root = root.path().to_path_buf();
        let entered = Arc::clone(&entered);
        let barrier = Arc::clone(&barrier);
        async move {
            deadline_blocking_io(&deadline, move |cancelled| {
                entered.store(true, Ordering::Release);
                barrier.wait();
                if cancelled.load(Ordering::Acquire) {
                    return Err(deadline_elapsed());
                }
                create_epoch_staging(&root)
            })
            .await
        }
    });
    while !entered.load(Ordering::Acquire) {
        tokio::task::yield_now().await;
    }
    let responsive = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let heartbeat = tokio::spawn({
        let responsive = Arc::clone(&responsive);
        async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            responsive.store(true, Ordering::Release);
        }
    });
    tokio::time::sleep(Duration::from_millis(30)).await;
    assert!(responsive.load(Ordering::Acquire));
    barrier.wait();
    heartbeat.await.unwrap();
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert!(!root.path().join(BODY_LIMIT_CACHE_EPOCH).exists());
    assert!(fs::read_dir(root.path()).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp-")
    }));
}

#[tokio::test(start_paused = true)]
async fn deadline_io_returns_a_timely_success() {
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(10));

    let value = deadline_io(&deadline, async { Ok::<_, io::Error>("timely") })
        .await
        .unwrap();

    assert_eq!(value, "timely");
}

#[tokio::test]
async fn deadline_io_converts_an_admitted_late_success_to_timed_out() {
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(5));

    let error = deadline_io(&deadline, async {
        std::thread::sleep(Duration::from_millis(20));
        Ok(())
    })
    .await
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
}

#[tokio::test]
async fn deadline_io_preserves_an_admitted_late_io_error() {
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_millis(5));

    let error = deadline_io(&deadline, async {
        std::thread::sleep(Duration::from_millis(20));
        Err::<(), _>(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "admitted file operation failed",
        ))
    })
    .await
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    assert_eq!(error.to_string(), "admitted file operation failed");
}

#[tokio::test(start_paused = true)]
async fn deadline_io_timeout_settles_before_return_and_cannot_mutate_later() {
    let entered = Arc::new(tokio::sync::Notify::new());
    let release = Arc::new(tokio::sync::Notify::new());
    let mutations = Arc::new(AtomicUsize::new(0));
    let deadline = crate::sources::VariantArticleDeadline::from_now(Duration::from_secs(10));
    let task = tokio::spawn({
        let entered = Arc::clone(&entered);
        let release = Arc::clone(&release);
        let mutations = Arc::clone(&mutations);
        async move {
            deadline_io(&deadline, async move {
                entered.notify_one();
                release.notified().await;
                mutations.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await
        }
    });
    entered.notified().await;
    tokio::time::advance(Duration::from_secs(11)).await;
    tokio::task::yield_now().await;
    assert!(!task.is_finished(), "timeout must settle admitted I/O");
    assert_eq!(mutations.load(Ordering::SeqCst), 0);

    release.notify_one();
    assert_eq!(
        task.await.unwrap().unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(mutations.load(Ordering::SeqCst), 1);
    tokio::task::yield_now().await;
    assert_eq!(mutations.load(Ordering::SeqCst), 1);
}
