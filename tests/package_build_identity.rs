//! Package-safe smoke target for the published library boundary.

#[test]
fn packaged_library_reports_the_manifest_version() {
    let identity = biomcp_cli::build_identity::current();
    assert_eq!(identity.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(identity.git_revision, "unknown");
    assert_eq!(identity.build_date, "unknown");
}
