//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::OlsClient;

#[test]
fn decode_search_response_maps_docs() {
    let content_type = HeaderValue::from_static("application/json");
    let rows = OlsClient::decode_search_response(
        StatusCode::OK,
        Some(&content_type),
        br#"{
            "response": {
                "docs": [
                    {
                        "iri": "http://example.org/hgnc/3236",
                        "ontology_name": "hgnc",
                        "ontology_prefix": "hgnc",
                        "short_form": "hgnc:3236",
                        "obo_id": "HGNC:3236",
                        "label": "EGFR",
                        "description": [],
                        "exact_synonyms": ["ERBB1"],
                        "type": "class"
                    }
                ]
            }
        }"#,
    )
    .expect("search response should decode");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "EGFR");
    assert_eq!(rows[0].exact_synonyms, vec!["ERBB1"]);
}

#[test]
fn decode_search_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = OlsClient::decode_search_response(
        StatusCode::BAD_GATEWAY,
        Some(&content_type),
        br#"{"error":"upstream"}"#,
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));

    let content_type = HeaderValue::from_static("text/html");
    let err = OlsClient::decode_search_response(
        StatusCode::OK,
        Some(&content_type),
        b"<html>not json</html>",
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}
