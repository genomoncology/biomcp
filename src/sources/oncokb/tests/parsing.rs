//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the decoder.
//! No network, no server, no token needed.

use super::super::*;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/oncokb/",
            $name
        ))
    };
}

#[test]
fn parses_annotation_fixture() {
    let ann: OncoKBAnnotation =
        OncoKBClient::decode_json_response(StatusCode::OK, fixture!("annotation_braf_v600e.json"))
            .unwrap();

    assert_eq!(ann.oncogenic.as_deref(), Some("Oncogenic"));
    assert_eq!(
        ann.mutation_effect
            .as_ref()
            .and_then(|effect| effect.known_effect.as_deref()),
        Some("Gain-of-function")
    );
    assert_eq!(ann.highest_sensitive_level.as_deref(), Some("LEVEL_1"));
    assert_eq!(ann.treatments.len(), 1);
    assert_eq!(ann.treatments[0].level.as_deref(), Some("LEVEL_1"));
    assert_eq!(
        ann.treatments[0].drugs[0].drug_name.as_deref(),
        Some("Dabrafenib")
    );
}

#[test]
fn decode_json_response_maps_http_errors_with_excerpt() {
    let err = OncoKBClient::decode_json_response::<OncoKBAnnotation>(
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");

    assert_eq!(err.code(), "api");
    assert!(msg.contains("OncoKB"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");
}

#[test]
fn decode_json_response_maps_invalid_json() {
    let err = OncoKBClient::decode_json_response::<OncoKBAnnotation>(StatusCode::OK, b"not json")
        .unwrap_err();

    assert_eq!(err.code(), "api_json");
}
