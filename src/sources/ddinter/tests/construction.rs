//! Tier 2 - local-data construction. Pure: checks DDInter bundle state and
//! identity terms without network.

use std::time::Duration;

use super::super::*;

#[test]
fn basis_freshness_is_stale_without_a_basis_and_fresh_from_recent_files() {
    // No basis (missing files) reads as stale, matching the all-files
    // semantics the root-reading check used to enforce.
    assert_eq!(basis_freshness(None), DdinterBundleFreshness::Stale);

    let root = tempfile::tempdir().expect("tempdir");
    for file_name in DDINTER_REQUIRED_FILES {
        std::fs::write(root.path().join(file_name), b"ok").expect("write");
    }
    assert_eq!(
        basis_freshness(oldest_bundle_mtime(root.path())),
        DdinterBundleFreshness::Fresh
    );
}

#[test]
fn cached_index_freshness_ignores_later_file_replacement() {
    // The basis is captured at load: replacing the bundle afterwards
    // cannot flip a loaded index's label (ticket 1241).
    let root = tempfile::tempdir().expect("tempdir");
    for file_name in DDINTER_REQUIRED_FILES {
        std::fs::write(root.path().join(file_name), b"ok").expect("write");
    }
    let old_mtime = std::time::SystemTime::now() - DDINTER_STALE_AFTER - Duration::from_secs(60);
    for file_name in DDINTER_REQUIRED_FILES {
        let path = root.path().join(file_name);
        let file = std::fs::File::options()
            .write(true)
            .open(&path)
            .expect("open");
        file.set_modified(old_mtime).expect("set_modified");
    }
    let basis = oldest_bundle_mtime(root.path()).expect("basis");
    // Aged basis reads stale even though the files now sit on disk.
    assert_eq!(basis_freshness(Some(basis)), DdinterBundleFreshness::Stale);
    // Rewriting the files fresh does not change the cached verdict.
    for file_name in DDINTER_REQUIRED_FILES {
        std::fs::write(root.path().join(file_name), b"ok").expect("write");
    }
    assert_eq!(basis_freshness(Some(basis)), DdinterBundleFreshness::Stale);
}

#[test]
fn ddinter_missing_files_reports_incomplete_bundle() {
    let root = tempfile::tempdir().expect("tempdir");
    std::fs::write(root.path().join(DDINTER_REQUIRED_FILES[0]), b"ok").expect("write");

    let missing = ddinter_missing_files(root.path(), DDINTER_REQUIRED_FILES);
    assert_eq!(missing.len(), DDINTER_REQUIRED_FILES.len() - 1);
    assert!(!missing.contains(&DDINTER_REQUIRED_FILES[0]));
}

#[test]
fn ddinter_identity_dedupes_alias_terms() {
    let aliases = vec!["Coumadin".to_string(), "WARFARIN".to_string()];
    let identity = DdinterIdentity::with_aliases("warfarin", Some("Warfarin"), &aliases);
    assert_eq!(
        identity.terms(),
        &["warfarin".to_string(), "coumadin".to_string()]
    );
}
