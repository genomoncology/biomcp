//! Tier 2 - local-data construction. Pure: checks the EMA feed contract,
//! missing-file detection, and sync/download plan. No network.

use super::super::*;
use crate::test_support::TempDirGuard;
use http_cache_reqwest::CacheMode;

#[test]
fn ema_feed_table_matches_required_file_contract() {
    let required = EMA_FEEDS
        .iter()
        .map(|feed| feed.local_name)
        .collect::<Vec<_>>();
    assert_eq!(required, EMA_REQUIRED_FILES);
}

#[test]
fn ema_missing_files_tracks_required_file_contract_in_order() {
    let root = TempDirGuard::new("missing-files");
    std::fs::write(root.path().join(MEDICINES_FILE), b"{}").expect("write medicines fixture");

    let missing = ema_missing_files(root.path(), EMA_REQUIRED_FILES);

    assert_eq!(missing, EMA_REQUIRED_FILES[1..].to_vec());
}

#[test]
fn sync_plan_marks_missing_and_stale_feeds() {
    let root = TempDirGuard::new("sync-plan");
    for feed in EMA_FEEDS {
        std::fs::write(root.path().join(feed.local_name), br#"{"data":[]}"#)
            .expect("fixture write should succeed");
    }
    let stale_path = root.path().join("medicines.json");
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&stale_path)
        .expect("stale file should open");
    file.set_modified(
        std::time::SystemTime::now()
            .checked_sub(std::time::Duration::from_secs(73 * 60 * 60))
            .expect("stale time should be valid"),
    )
    .expect("stale mtime should update");
    std::fs::remove_file(root.path().join("shortages.json"))
        .expect("missing file should be removable");

    let plan = sync_plan(root.path(), EmaSyncMode::Auto);
    let files = plan
        .iter()
        .map(|entry| entry.feed.local_name)
        .collect::<Vec<_>>();

    assert_eq!(files, vec!["medicines.json", "shortages.json"]);
    assert!(matches!(plan[0].state, FeedSyncState::Stale));
    assert_eq!(plan[0].cache_mode, CacheMode::Default);
    assert!(matches!(plan[1].state, FeedSyncState::Missing));
    assert_eq!(plan[1].cache_mode, CacheMode::Default);
}

#[test]
fn sync_intro_matches_download_refresh_and_force_modes() {
    let root = TempDirGuard::new("sync-intro");
    let missing_plan = sync_plan(root.path(), EmaSyncMode::Auto);
    assert_eq!(sync_intro(&missing_plan, EmaSyncMode::Auto), "Downloading");

    for feed in EMA_FEEDS {
        std::fs::write(root.path().join(feed.local_name), br#"{"data":[]}"#)
            .expect("fixture write should succeed");
    }
    let stale_path = root.path().join(MEDICINES_FILE);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&stale_path)
        .expect("stale file should open");
    file.set_modified(
        std::time::SystemTime::now()
            .checked_sub(EMA_STALE_AFTER + std::time::Duration::from_secs(60))
            .expect("stale time should be valid"),
    )
    .expect("stale mtime should update");

    let stale_plan = sync_plan(root.path(), EmaSyncMode::Auto);
    assert_eq!(sync_intro(&stale_plan, EmaSyncMode::Auto), "Refreshing");

    let force_plan = sync_plan(root.path(), EmaSyncMode::Force);
    assert_eq!(sync_intro(&force_plan, EmaSyncMode::Force), "Refreshing");
}

#[test]
fn ema_sync_error_mentions_recovery_paths() {
    let root = TempDirGuard::new("ema-sync-error");
    let err = ema_sync_error(root.path(), "medicines.json: HTTP 503");
    let message = format!("{err:?}");

    assert!(message.contains("EMA"));
    assert!(message.contains("medicines.json: HTTP 503"));
    assert!(message.contains("biomcp ema sync"));
    assert!(message.contains("BIOMCP_EMA_DIR"));
    assert!(message.contains(&root.path().display().to_string()));
}
