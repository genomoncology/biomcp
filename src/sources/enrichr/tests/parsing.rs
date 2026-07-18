//! Tier 3 - response parsing and local result shaping. Pure: feeds response
//! bytes into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::*;

#[test]
fn decode_add_list_response_parses_user_list_id() {
    let id =
        EnrichrClient::decode_add_list_response(StatusCode::OK, br#"{"userListId":42}"#).unwrap();

    assert_eq!(id, 42);
}

#[test]
fn decode_enrich_response_gracefully_handles_bad_request() {
    let value =
        EnrichrClient::decode_enrich_response(StatusCode::BAD_REQUEST, None, b"bad request")
            .unwrap();

    assert_eq!(value, serde_json::json!({}));
}

#[test]
fn decode_enrich_response_parses_json_and_rejects_html() {
    let content_type = HeaderValue::from_static("application/json");
    let value = EnrichrClient::decode_enrich_response(
        StatusCode::OK,
        Some(&content_type),
        br#"{"KEGG_2021_Human":[]}"#,
    )
    .unwrap();
    assert_eq!(value["KEGG_2021_Human"], serde_json::json!([]));

    let content_type = HeaderValue::from_static("text/html");
    let err = EnrichrClient::decode_enrich_response(
        StatusCode::OK,
        Some(&content_type),
        b"<html>not json</html>",
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}
