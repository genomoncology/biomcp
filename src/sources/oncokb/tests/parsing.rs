//! Transport error categories and sanitized shared-admission diagnostics.
//! The actual helper owns successful fixture claims; BioData owns admission.

use super::super::*;
use reqwest::StatusCode;

#[test]
fn decode_json_response_maps_http_errors_with_excerpt() {
    let err =
        OncoKBClient::decode_json_response(StatusCode::INTERNAL_SERVER_ERROR, b"upstream failed")
            .unwrap_err();
    let msg = format!("{err:?}");

    assert_eq!(err.code(), "api");
    assert!(msg.contains("OncoKB"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");
}

#[test]
fn decode_json_response_maps_invalid_json() {
    let err = OncoKBClient::decode_json_response(
        StatusCode::OK,
        br#"{"treatments":"synthetic-private-marker"}"#,
    )
    .unwrap_err();

    assert_eq!(err.code(), "api_json");
    let diagnostic = format!("{err:?}");
    assert!(
        diagnostic.contains("Invalid OncoKB response."),
        "{diagnostic}"
    );
    assert!(
        !diagnostic.contains("synthetic-private-marker"),
        "{diagnostic}"
    );
}
