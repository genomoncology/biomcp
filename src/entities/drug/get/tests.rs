//! Get-module tests split from the legacy drug facade.

use super::*;
use crate::test_support::set_env_var;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn mount_trial_alias_lookup(
    server: &MockServer,
    requested: &str,
    canonical: &str,
    aliases: &[&str],
) {
    Mock::given(method("GET"))
        .and(path("/v1/query"))
        .and(query_param("q", requested))
        .and(query_param("size", "25"))
        .and(query_param("from", "0"))
        .and(query_param(
            "fields",
            crate::sources::mychem::MYCHEM_FIELDS_GET,
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total": 1,
            "hits": [{
                "_id": "drug-test-id",
                "_score": 42.0,
                "drugbank": {
                    "id": "DBTEST",
                    "name": canonical,
                    "synonyms": aliases,
                }
            }]
        })))
        .expect(1)
        .mount(server)
        .await;
}

#[test]
fn parse_sections_supports_all_and_rejects_unknown() {
    let flags = parse_sections(&["all".to_string()]).unwrap();
    assert!(flags.include_label);
    assert!(flags.include_regulatory);
    assert!(flags.include_safety);
    assert!(flags.include_shortage);
    assert!(flags.include_targets);
    assert!(flags.include_indications);
    assert!(flags.include_interactions);
    assert!(flags.include_civic);
    assert!(!flags.include_approvals);

    let err = parse_sections(&["bad".to_string()]).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

#[test]
fn parse_sections_all_with_explicit_label_keeps_label() {
    let flags = parse_sections(&["all".to_string(), "label".to_string()]).unwrap();
    assert!(flags.include_label);
}

#[test]
fn parse_sections_default_card_includes_targets_enrichment() {
    let flags = parse_sections(&[]).unwrap();
    assert!(flags.include_targets);
}

#[test]
fn validate_region_usage_rejects_approvals_with_explicit_region() {
    let flags = parse_sections(&["approvals".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Us, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("approvals"));
}

#[test]
fn validate_region_usage_rejects_explicit_region_without_regional_sections() {
    let flags = parse_sections(&["targets".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Us, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--region can only be used"));
}

#[test]
fn validate_region_usage_rejects_who_safety_only_requests() {
    let flags = parse_sections(&["safety".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Who, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("WHO regional data currently supports regulatory only")
    );
}

#[test]
fn validate_region_usage_rejects_who_shortage_only_requests() {
    let flags = parse_sections(&["shortage".to_string()]).unwrap();
    let err = validate_region_usage(&flags, DrugRegion::Who, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("WHO regional data currently supports regulatory only")
    );
}

#[test]
fn validate_region_usage_allows_who_all_requests() {
    let flags = parse_sections(&["all".to_string()]).unwrap();
    validate_region_usage(&flags, DrugRegion::Who, true).expect("who all should be valid");
}

#[test]
fn validate_raw_usage_rejects_raw_without_label_section() {
    let flags = parse_sections(&["targets".to_string()]).unwrap();
    let err = validate_raw_usage(&flags, true).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--raw can only be used"));
}

#[test]
fn validate_raw_usage_allows_raw_with_label_section() {
    let flags = parse_sections(&["label".to_string()]).unwrap();
    validate_raw_usage(&flags, true).expect("raw label should be valid");
}

#[test]
fn trial_alias_filter_rejects_formulation_strength_variants() {
    assert!(looks_like_trial_formulation_variant("Keytruda 25 mg/mL"));
    assert!(looks_like_trial_formulation_variant(
        "Pembrolizumab injection"
    ));
}

#[test]
fn trial_alias_filter_keeps_sponsor_codes() {
    assert!(!looks_like_trial_formulation_variant("RMC-6236"));
}

#[test]
fn build_trial_aliases_preserves_requested_canonical_and_brand_order() {
    let aliases = build_trial_aliases(
        "RMC-6236",
        Some("daraxonrasib"),
        &[
            "RMC-6236".to_string(),
            "Keytruda 25 mg/mL".to_string(),
            "RMC-6236".to_string(),
            "daraxonrasib".to_string(),
            "RMC-9805".to_string(),
        ],
    );

    assert_eq!(aliases, vec!["RMC-6236", "daraxonrasib", "RMC-9805"]);
}

#[test]
fn trial_alias_cache_key_normalizes_requested_name() {
    assert_eq!(trial_alias_cache_key(" Daraxonrasib "), "daraxonrasib");
}

#[tokio::test]
async fn resolve_trial_aliases_retries_after_transient_lookup_failure() {
    let _env_lock = crate::test_support::env_lock().lock().await;
    // Isolate the shared MyChem HTTP cache while this test swaps BIOMCP_MYCHEM_BASE.
    let requested = "review-transient-alias-drug";

    let failing = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/query"))
        .and(query_param("q", requested))
        .and(query_param("size", "25"))
        .and(query_param("from", "0"))
        .and(query_param(
            "fields",
            crate::sources::mychem::MYCHEM_FIELDS_GET,
        ))
        .respond_with(ResponseTemplate::new(500))
        .mount(&failing)
        .await;

    let failing_base = format!("{}/v1", failing.uri());
    let failing_env = set_env_var("BIOMCP_MYCHEM_BASE", Some(&failing_base));
    assert_eq!(
        resolve_trial_aliases(requested)
            .await
            .expect("fallback aliases"),
        vec![requested.to_string()]
    );
    drop(failing_env);

    let success = MockServer::start().await;
    mount_trial_alias_lookup(&success, requested, requested, &["RMC-6236"]).await;

    let success_base = format!("{}/v1", success.uri());
    let _success_env = set_env_var("BIOMCP_MYCHEM_BASE", Some(&success_base));
    assert_eq!(
        resolve_trial_aliases(requested)
            .await
            .expect("resolved aliases after retry"),
        vec![requested.to_string(), "RMC-6236".to_string()]
    );
}

#[tokio::test]
async fn resolve_trial_canonical_name_keeps_generic_requests_canonical() {
    let _env_lock = crate::test_support::env_lock().lock().await;
    let requested = "pembrolizumab";

    let success = MockServer::start().await;
    mount_trial_alias_lookup(&success, requested, requested, &["Keytruda"]).await;

    let success_base = format!("{}/v1", success.uri());
    let _success_env = set_env_var("BIOMCP_MYCHEM_BASE", Some(&success_base));
    assert_eq!(
        resolve_trial_canonical_name(requested)
            .await
            .expect("resolved canonical trial name"),
        requested
    );
}
