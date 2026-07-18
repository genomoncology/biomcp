//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
//! the `NciSearchResponse` shape. No network, no server.

use crate::sources::decode_json;
use crate::sources::nci_cts::NciSearchResponse;
use reqwest::StatusCode;

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
