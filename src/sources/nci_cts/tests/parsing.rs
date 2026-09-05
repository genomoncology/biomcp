//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
//! the `NciSearchResponse` shape. No network, no server.

use crate::error::{BioMcpError, RecoveryAction};
use crate::sources::decode_json;
use crate::sources::nci_cts::{NciCtsClient, NciSearchResponse};
use reqwest::StatusCode;
use sha2::{Digest, Sha256};

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/nci_cts/",
            $name
        ))
    };
}

#[test]
fn parses_real_search_response_total_and_hits() {
    let resp: NciSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS),
        StatusCode::OK,
        None,
        fixture!("search_melanoma.json"),
        false,
    )
    .unwrap();
    assert!(resp.total.is_some());
    assert!(!resp.hits().is_empty());
}

#[test]
fn hits_prefers_data_over_trials() {
    let resp: NciSearchResponse = serde_json::from_str(
        r#"{"data":[{"nci_id":"NCI-1"}],"trials":[{"nci_id":"OLD"}],"total":1}"#,
    )
    .unwrap();
    assert_eq!(resp.hits().len(), 1);
    assert_eq!(
        resp.hits()[0].get("nci_id").and_then(|v| v.as_str()),
        Some("NCI-1")
    );
}

#[test]
fn hits_falls_back_to_trials_when_data_empty() {
    let resp: NciSearchResponse =
        serde_json::from_str(r#"{"data":[],"trials":[{"nci_id":"T-1"}],"total":1}"#).unwrap();
    assert_eq!(resp.hits().len(), 1);
    assert_eq!(
        resp.hits()[0].get("nci_id").and_then(|v| v.as_str()),
        Some("T-1")
    );
}

#[test]
fn total_accepts_total_count_alias() {
    let resp: NciSearchResponse = serde_json::from_str(r#"{"data":[],"total_count":42}"#).unwrap();
    assert_eq!(resp.total, Some(42));
}

#[test]
fn decode_json_maps_http_error_for_nci() {
    let err = decode_json::<NciSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        false,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("NCI Clinical Trials Search"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

fn detail_plan() -> biodata::NciCtsV2DetailPlan {
    biodata::NciCtsV2DetailPlan::new("NCT00000001", true).unwrap()
}

fn valid_detail(identity: &str) -> Vec<u8> {
    format!(
        concat!(
            "{{\"total\":1,\"data\":[{{\"nci_id\":\"NCI-SYNTHETIC\",",
            "\"nct_id\":\"{}\",\"brief_title\":\"Synthetic trial\",",
            "\"official_title\":null,\"current_trial_status\":\"Active\",",
            "\"why_study_stopped\":null,\"study_protocol_type\":\"Interventional\",",
            "\"phase\":\"Phase 1\",\"diseases\":[{{\"name\":\"Cancer\"}}],",
            "\"minimum_target_accrual_number\":10,\"arms\":[],\"lead_org\":\"NCI\",",
            "\"start_date\":null,\"completion_date\":null,",
            "\"eligibility\":{{\"structured\":{{\"min_age\":\"18 Years\",",
            "\"max_age\":\"999 Years\",\"sex\":\"ALL\"}},\"unstructured\":[]}},",
            "\"brief_summary\":\" summary \"}}]}}"
        ),
        identity
    )
    .into_bytes()
}

fn assert_sanitized(error: &BioMcpError, forbidden: &[&str]) {
    let diagnostic = format!("{error} {error:?} {}", error.public_projection().message);
    for value in forbidden {
        assert!(!diagnostic.contains(value), "leaked {value}: {diagnostic}");
    }
}

#[test]
fn detail_response_checks_status_before_parsing_and_never_stores_the_body() {
    for status in [
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
    ] {
        let error = NciCtsClient::decode_detail_response(
            &detail_plan(),
            status,
            br#"{"secret":"provider-value","nct_id":"NCT99999999"}"#,
        )
        .unwrap_err();
        assert_eq!(error.code(), "api");
        assert_eq!(
            error.public_projection().recovery,
            Some(RecoveryAction::RetryRemoteSource.message())
        );
        assert_sanitized(&error, &["provider-value", "NCT99999999", "NCT00000001"]);
    }

    let error = NciCtsClient::decode_detail_response(
        &detail_plan(),
        StatusCode::NOT_FOUND,
        br#"{"secret":"provider-value"}"#,
    )
    .unwrap_err();
    assert!(matches!(error, BioMcpError::NotFound { .. }));
    assert!(!format!("{error} {error:?}").contains("provider-value"));
}

#[test]
fn detail_response_maps_biodata_failures_without_source_values() {
    let plan = detail_plan();
    let cases = [
        (br#"{"total":0,"data":[]}"#.as_slice(), "not_found"),
        (b"{".as_slice(), "malformed_json"),
        (
            br#"{"total":1,"total":1,"data":[]}"#.as_slice(),
            "unsupported_json",
        ),
        (
            br#"{"total":2,"data":[{},{}]}"#.as_slice(),
            "unexpected_row_count",
        ),
    ];
    for (bytes, code) in cases {
        let error = NciCtsClient::decode_detail_response(&plan, StatusCode::OK, bytes).unwrap_err();
        if code == "not_found" {
            assert!(matches!(error, BioMcpError::NotFound { .. }));
        } else {
            assert!(format!("{error:?}").contains(code));
            assert_sanitized(&error, &["NCT00000001", "secret"]);
        }
    }

    let wrong = valid_detail("NCT00000002");
    let error = NciCtsClient::decode_detail_response(&plan, StatusCode::OK, &wrong).unwrap_err();
    assert!(format!("{error:?}").contains("identity_mismatch"));
    assert_sanitized(&error, &["NCT00000001", "NCT00000002"]);
}

#[test]
fn detail_response_rejects_identity_and_old_lenient_shapes() {
    let plan = detail_plan();
    let valid = String::from_utf8(valid_detail("NCT00000001")).unwrap();
    let malformed = [
        (
            "missing identity",
            valid.replace("\"nct_id\":\"NCT00000001\",", ""),
            "invalid_projection",
        ),
        (
            "null identity",
            valid.replace("\"nct_id\":\"NCT00000001\"", "\"nct_id\":null"),
            "invalid_projection",
        ),
        (
            "numeric identity",
            valid.replace("\"nct_id\":\"NCT00000001\"", "\"nct_id\":7"),
            "invalid_projection",
        ),
        (
            "missing title",
            valid.replace("\"brief_title\":\"Synthetic trial\",", ""),
            "invalid_projection",
        ),
        (
            "invalid age",
            valid.replace("\"min_age\":\"18 Years\"", "\"min_age\":\"18 Years old\""),
            "invalid_projection",
        ),
        (
            "non-object structured eligibility",
            valid.replace(
            "\"structured\":{\"min_age\":\"18 Years\",\"max_age\":\"999 Years\",\"sex\":\"ALL\"}",
            "\"structured\":true",
        ),
            "unsupported_json",
        ),
        (
            "string enrollment",
            valid.replace(
            "\"minimum_target_accrual_number\":10",
            "\"minimum_target_accrual_number\":\"10\"",
        ),
            "invalid_projection",
        ),
    ];
    for (shape, bytes, expected_code) in malformed {
        let error = NciCtsClient::decode_detail_response(&plan, StatusCode::OK, bytes.as_bytes())
            .unwrap_err();
        assert_eq!(error.code(), "api");
        assert!(
            format!("{error:?}").contains(&format!(
                "BioData response validation failed: {expected_code}"
            )),
            "{shape} returned {error:?}"
        );
        assert_sanitized(&error, &["NCT00000001", "Synthetic trial"]);
    }
}

#[test]
fn detail_response_maps_the_biodata_resource_limit_to_narrow_recovery() {
    let bytes = vec![b' '; 8 * 1024 * 1024 + 1];
    let error =
        NciCtsClient::decode_detail_response(&detail_plan(), StatusCode::OK, &bytes).unwrap_err();
    assert!(format!("{error:?}").contains("json_resource_limit"));
    assert_eq!(
        error.public_projection().recovery,
        Some(RecoveryAction::NarrowRequest.message())
    );
}

#[test]
fn detail_response_passes_untouched_success_bytes_to_biodata() {
    let bytes = valid_detail("NCT00000001");
    let response =
        NciCtsClient::decode_detail_response(&detail_plan(), StatusCode::OK, &bytes).unwrap();
    assert_eq!(
        response.projection().capture().digest(),
        format!("sha256:{:x}", Sha256::digest(&bytes))
    );
}
