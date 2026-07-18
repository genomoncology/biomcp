//! Tier 2 - local-data construction. Pure: checks required-file detection and
//! sync state decisions without network or env mutation.

use super::super::*;
use crate::test_support::TempDirGuard;

#[test]
fn cvx_missing_files_tracks_required_contract() {
    let root = TempDirGuard::new("cvx-missing");
    std::fs::write(root.path().join("cvx.txt"), "fixture").expect("write file");

    let missing = cvx_missing_files(root.path(), CVX_REQUIRED_FILES);

    assert_eq!(missing, CVX_REQUIRED_FILES[1..].to_vec());
}

#[test]
fn sync_state_marks_missing_fresh_stale_and_force() {
    let root = TempDirGuard::new("cvx-sync-state");
    assert!(matches!(
        sync_state(root.path(), CvxSyncMode::Auto),
        SyncState::Missing
    ));

    super::write_fixture_bundle(root.path());
    assert!(matches!(
        sync_state(root.path(), CvxSyncMode::Auto),
        SyncState::Fresh
    ));

    let stale_path = root.path().join(CVX_FILE);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&stale_path)
        .expect("stale file should open");
    file.set_modified(
        std::time::SystemTime::now()
            .checked_sub(CVX_STALE_AFTER + std::time::Duration::from_secs(60))
            .expect("stale time should be valid"),
    )
    .expect("stale mtime should update");
    assert!(matches!(
        sync_state(root.path(), CvxSyncMode::Auto),
        SyncState::Stale
    ));
    assert!(matches!(
        sync_state(root.path(), CvxSyncMode::Force),
        SyncState::Stale
    ));
}

#[test]
fn sync_intro_matches_missing_stale_and_force_modes() {
    assert_eq!(
        sync_intro(SyncState::Missing, CvxSyncMode::Auto),
        "Downloading"
    );
    assert_eq!(
        sync_intro(SyncState::Stale, CvxSyncMode::Auto),
        "Refreshing stale"
    );
    assert_eq!(sync_intro(SyncState::Fresh, CvxSyncMode::Auto), "Checking");
    assert_eq!(
        sync_intro(SyncState::Fresh, CvxSyncMode::Force),
        "Refreshing"
    );
}

#[test]
fn cvx_read_error_mentions_recovery_paths() {
    let root = TempDirGuard::new("cvx-read-error");
    let err = cvx_read_error(root.path(), "mvx.txt: HTTP 503");
    let message = format!("{err:?}");

    assert!(message.contains("CDC CVX/MVX"));
    assert!(message.contains("mvx.txt: HTTP 503"));
    assert!(message.contains("biomcp cvx sync"));
    assert!(message.contains("BIOMCP_CVX_DIR"));
    assert!(message.contains(&root.path().display().to_string()));
}
