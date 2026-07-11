//! Tier 2 - local-data construction. Pure: checks DDInter bundle state and
//! identity terms without network.

use super::super::*;

#[test]
fn bundle_freshness_requires_all_files_to_be_fresh() {
    let root = tempfile::tempdir().expect("tempdir");
    assert_eq!(bundle_freshness(root.path()), DdinterBundleFreshness::Stale);

    for file_name in DDINTER_REQUIRED_FILES {
        std::fs::write(root.path().join(file_name), b"ok").expect("write");
    }
    assert_eq!(bundle_freshness(root.path()), DdinterBundleFreshness::Fresh);
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
