//! Tier 2 — local bundle construction checks. GTR is file-backed, so these tests
//! assert the required local file contract and root-bound client construction.

use super::super::*;
use crate::test_support::TempDirGuard;

#[test]
fn gtr_missing_files_tracks_required_contract() {
    let root = TempDirGuard::new("gtr-missing-files");
    let missing = gtr_missing_files(root.path());
    assert_eq!(
        missing,
        vec![
            GTR_TEST_VERSION_FILE.to_string(),
            GTR_CONDITION_GENE_FILE.to_string()
        ]
    );

    std::fs::write(
        root.path().join(GTR_TEST_VERSION_FILE),
        super::parsing::test_version_gz_bytes(),
    )
    .expect("write partial bundle");
    let missing = gtr_missing_files(root.path());
    assert_eq!(missing, vec![GTR_CONDITION_GENE_FILE.to_string()]);
}

#[test]
fn client_from_root_uses_supplied_root_without_env() {
    let root = TempDirGuard::new("gtr-client-root");
    let client = GtrClient::from_root(root.path());

    assert_eq!(client.root, root.path());
}

#[test]
fn sync_state_marks_missing_fresh_stale_and_force() {
    let root = TempDirGuard::new("gtr-sync-state");
    assert!(matches!(
        sync_state(root.path(), GtrSyncMode::Auto),
        SyncState::Missing
    ));

    std::fs::write(
        root.path().join(GTR_TEST_VERSION_FILE),
        super::parsing::test_version_gz_bytes(),
    )
    .expect("write test version");
    std::fs::write(
        root.path().join(GTR_CONDITION_GENE_FILE),
        super::parsing::condition_gene_bytes(),
    )
    .expect("write condition gene");

    assert!(matches!(
        sync_state(root.path(), GtrSyncMode::Auto),
        SyncState::Fresh
    ));

    let stale_path = root.path().join(GTR_CONDITION_GENE_FILE);
    let file = std::fs::OpenOptions::new()
        .write(true)
        .open(&stale_path)
        .expect("stale file should open");
    file.set_modified(
        std::time::SystemTime::now()
            .checked_sub(GTR_STALE_AFTER + std::time::Duration::from_secs(60))
            .expect("stale time should be valid"),
    )
    .expect("stale mtime should update");

    assert!(matches!(
        sync_state(root.path(), GtrSyncMode::Auto),
        SyncState::Stale
    ));
    assert!(matches!(
        sync_state(root.path(), GtrSyncMode::Force),
        SyncState::Stale
    ));
}

#[test]
fn sync_intro_matches_missing_stale_and_force_modes() {
    assert_eq!(
        sync_intro(SyncState::Missing, GtrSyncMode::Auto),
        "Downloading"
    );
    assert_eq!(
        sync_intro(SyncState::Stale, GtrSyncMode::Auto),
        "Refreshing stale"
    );
    assert_eq!(sync_intro(SyncState::Fresh, GtrSyncMode::Auto), "Checking");
    assert_eq!(
        sync_intro(SyncState::Fresh, GtrSyncMode::Force),
        "Refreshing"
    );
}

#[test]
fn gtr_sync_error_mentions_recovery_paths() {
    let root = TempDirGuard::new("gtr-sync-error");
    let err = gtr_sync_error(root.path(), "test_condition_gene.txt: HTTP 503");
    let message = format!("{err:?}");

    assert!(message.contains("GTR"));
    assert!(message.contains("test_condition_gene.txt: HTTP 503"));
    assert!(message.contains("biomcp gtr sync"));
    assert!(message.contains("BIOMCP_GTR_DIR"));
    assert!(message.contains(&root.path().display().to_string()));
}
