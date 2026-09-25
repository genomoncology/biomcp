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

fn write_valid_bundle(root: &std::path::Path) {
    let body = b"DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level\nDDI1A,aspirin,DDI1B,warfarin,Major\n";
    for file_name in DDINTER_REQUIRED_FILES {
        std::fs::write(root.join(file_name), body).expect("write bundle file");
    }
}

fn set_all_modified(root: &std::path::Path, mtime: std::time::SystemTime) {
    for file_name in DDINTER_REQUIRED_FILES {
        let path = root.join(file_name);
        let file = std::fs::File::options()
            .write(true)
            .open(&path)
            .expect("open");
        file.set_modified(mtime).expect("set_modified");
    }
}

#[test]
fn cached_index_freshness_follows_the_cached_basis_not_the_files() {
    // Through cached_index_for_root itself (ticket 1254): the basis is
    // captured at load, so replacing the bundle afterwards cannot flip
    // a loaded index's label — in either direction.
    let root = tempfile::tempdir().expect("tempdir");
    write_valid_bundle(root.path());
    let now = std::time::SystemTime::now();
    let old_mtime = now - DDINTER_STALE_AFTER - Duration::from_secs(60);

    // Load stale: later fresh files cannot make the cached label Fresh.
    set_all_modified(root.path(), old_mtime);
    let cached = cached_index_for_root(root.path()).expect("cache");
    assert_eq!(
        basis_freshness(cached.oldest_mtime),
        DdinterBundleFreshness::Stale
    );
    set_all_modified(root.path(), now);
    let cached = cached_index_for_root(root.path()).expect("cache");
    assert_eq!(
        basis_freshness(cached.oldest_mtime),
        DdinterBundleFreshness::Stale,
        "a loaded stale basis stays stale after fresh replacement"
    );
}

#[test]
fn cached_index_freshness_keeps_a_fresh_load_fresh_after_aging_the_files() {
    // The other direction: a basis captured fresh stays fresh even
    // after the files on disk age past the window (ticket 1254). The
    // server outlives the bundle files; only sync evicts the cache.
    let root = tempfile::tempdir().expect("tempdir");
    write_valid_bundle(root.path());
    let now = std::time::SystemTime::now();
    let old_mtime = now - DDINTER_STALE_AFTER - Duration::from_secs(60);

    set_all_modified(root.path(), now);
    let cached = cached_index_for_root(root.path()).expect("cache");
    assert_eq!(
        basis_freshness(cached.oldest_mtime),
        DdinterBundleFreshness::Fresh
    );
    set_all_modified(root.path(), old_mtime);
    let cached = cached_index_for_root(root.path()).expect("cache");
    assert_eq!(
        basis_freshness(cached.oldest_mtime),
        DdinterBundleFreshness::Fresh,
        "a loaded fresh basis stays fresh after the files age"
    );
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
