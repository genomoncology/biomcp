//! Tier 2 - local-data construction. Pure: checks required-file detection and
//! stale/missing sync decisions. No network.

use std::time::{Duration, SystemTime};

use super::super::*;
use crate::test_support::TempDirGuard;

#[test]
fn who_pq_missing_files_tracks_required_file_contract() {
    let root = TempDirGuard::new("missing-files");
    let missing = who_pq_missing_files(root.path(), WHO_PQ_REQUIRED_FILES);
    assert_eq!(
        missing,
        vec![WHO_PQ_CSV_FILE, WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE]
    );
}

#[test]
fn file_is_stale_tracks_age_threshold() {
    let root = TempDirGuard::new("stale");
    let path = root.path().join(WHO_PQ_CSV_FILE);
    std::fs::write(&path, "header\n").expect("fixture should write");
    assert!(!file_is_stale(&path));

    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .expect("file should open");
    file.set_modified(
        SystemTime::now()
            .checked_sub(Duration::from_secs(73 * 60 * 60))
            .expect("stale time should be valid"),
    )
    .expect("mtime should update");

    assert!(file_is_stale(&path));
}

#[test]
fn sync_state_marks_missing_fresh_stale_and_force() {
    let root = TempDirGuard::new("sync-state");
    assert!(matches!(
        sync_state(root.path(), WhoPqSyncMode::Auto),
        SyncState::Missing
    ));

    std::fs::write(root.path().join(WHO_PQ_CSV_FILE), super::fixture_csv()).expect("write WHO CSV");
    std::fs::write(
        root.path().join(WHO_PQ_API_CSV_FILE),
        super::fixture_api_csv(),
    )
    .expect("write WHO API CSV");
    std::fs::write(
        root.path().join(WHO_VACCINES_CSV_FILE),
        super::fixture_vaccine_csv(),
    )
    .expect("write WHO vaccine CSV");
    assert!(matches!(
        sync_state(root.path(), WhoPqSyncMode::Auto),
        SyncState::Fresh
    ));

    let stale_path = root.path().join(WHO_PQ_API_CSV_FILE);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&stale_path)
        .expect("stale file should open");
    file.set_modified(
        SystemTime::now()
            .checked_sub(WHO_PQ_STALE_AFTER + Duration::from_secs(60))
            .expect("stale time should be valid"),
    )
    .expect("stale mtime should update");

    assert!(matches!(
        sync_state(root.path(), WhoPqSyncMode::Auto),
        SyncState::Stale
    ));
    assert!(matches!(
        sync_state(root.path(), WhoPqSyncMode::Force),
        SyncState::Stale
    ));
}

#[test]
fn sync_intro_matches_missing_stale_and_force_modes() {
    assert_eq!(
        sync_intro(SyncState::Missing, WhoPqSyncMode::Auto),
        "Downloading"
    );
    assert_eq!(
        sync_intro(SyncState::Stale, WhoPqSyncMode::Auto),
        "Refreshing stale"
    );
    assert_eq!(
        sync_intro(SyncState::Fresh, WhoPqSyncMode::Auto),
        "Checking"
    );
    assert_eq!(
        sync_intro(SyncState::Fresh, WhoPqSyncMode::Force),
        "Refreshing"
    );
}

#[test]
fn who_pq_sync_error_mentions_recovery_paths() {
    let root = TempDirGuard::new("who-pq-sync-error");
    let err = who_pq_sync_error(root.path(), "who_api.csv is missing required column: inn");
    let message = format!("{err:?}");

    assert!(message.contains("WHO Prequalification"));
    assert!(message.contains("who_api.csv is missing required column: inn"));
    assert!(message.contains("biomcp who sync"));
    assert!(message.contains("BIOMCP_WHO_DIR"));
    assert!(message.contains(&root.path().display().to_string()));
}
