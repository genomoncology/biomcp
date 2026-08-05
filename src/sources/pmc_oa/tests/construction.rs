//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::pmc_oa::PmcOaClient;

#[test]
fn oa_archive_manifest_plan_lists_versioned_s3_prefix() {
    let plan = PmcOaClient::oa_archive_manifest_plan(" PMC9984800 ", None)
        .unwrap()
        .expect("plan");

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "");
    assert_eq!(plan.query_value("list-type"), Some("2"));
    assert_eq!(plan.query_value("prefix"), Some("PMC9984800."));
    assert_eq!(plan.query_value("delimiter"), Some("/"));
    assert!(!plan.has_query("id"));
    assert!(!plan.has_query("api_key"));
}

#[test]
fn oa_archive_manifest_plan_adds_api_key_when_configured() {
    let plan = PmcOaClient::oa_archive_manifest_plan("PMC123", Some(" test-key "))
        .unwrap()
        .expect("plan");
    assert_eq!(plan.query_value("api_key"), Some("test-key"));
}

#[test]
fn oa_archive_manifest_plan_empty_input_returns_none() {
    assert!(
        PmcOaClient::oa_archive_manifest_plan("   ", None)
            .unwrap()
            .is_none()
    );
}

#[test]
fn oa_archive_manifest_plan_rejects_overlong_id() {
    let err = PmcOaClient::oa_archive_manifest_plan(&"P".repeat(65), None).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("too long"));
}
