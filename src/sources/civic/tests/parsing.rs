//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the CIViC
//! GraphQL decoder and mapper. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/civic/",
            $name
        ))
    };
}

#[test]
fn context_response_maps_evidence_and_assertions() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: GraphQlResponse<CivicContextData> = CivicClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("context_braf_v600e.json"),
    )
    .expect("graphql response");
    let out = CivicClient::context_from_response(resp).expect("context");

    assert_eq!(out.evidence_total_count, 12);
    assert_eq!(out.assertion_total_count, 2);
    assert_eq!(out.evidence_items.len(), 1);
    assert_eq!(out.assertions.len(), 1);
    assert_eq!(out.evidence_items[0].therapies, vec!["Vemurafenib"]);
    assert_eq!(out.assertions[0].approvals_count, 1);
}

#[test]
fn context_response_surfaces_graphql_errors() {
    let resp: GraphQlResponse<CivicContextData> = serde_json::from_value(serde_json::json!({
        "errors": [{"message": "Bad query"}]
    }))
    .unwrap();

    let err = CivicClient::context_from_response(resp).unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("Bad query"));
}

#[test]
fn decode_json_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = CivicClient::decode_json_response::<GraphQlResponse<CivicContextData>>(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&content_type),
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let err = CivicClient::decode_json_response::<GraphQlResponse<CivicContextData>>(
        StatusCode::OK,
        Some(&html),
        b"<html></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
}

#[test]
fn context_response_rejects_a_missing_data_block() {
    // A schema change that renames the data block must not read as
    // "no evidence": the envelope parses, the data block is absent, and
    // no GraphQL error was surfaced.
    let resp: GraphQlResponse<CivicContextData> =
        serde_json::from_value(serde_json::json!({})).unwrap();

    let err = CivicClient::context_from_response(resp).unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("no data block"));
}

#[test]
fn context_response_rejects_a_total_without_parsed_rows() {
    // A renamed evidence node field leaves the total intact but the rows
    // empty: that shape is a schema change, not a healthy empty page.
    let resp: GraphQlResponse<CivicContextData> = serde_json::from_value(serde_json::json!({
        "data": {
            "evidenceItems": {"totalCount": 17, "nodes": []},
            "assertions": {"totalCount": 0, "nodes": []}
        }
    }))
    .unwrap();

    let err = CivicClient::context_from_response(resp).unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("evidence total is 17"));
}

#[test]
fn context_response_accepts_a_genuine_empty_page() {
    // totalCount=0 with no rows is a healthy empty: no error, real zeros.
    let resp: GraphQlResponse<CivicContextData> = serde_json::from_value(serde_json::json!({
        "data": {
            "evidenceItems": {"totalCount": 0, "nodes": []},
            "assertions": {"totalCount": 0, "nodes": []}
        }
    }))
    .unwrap();

    let out = CivicClient::context_from_response(resp).expect("healthy empty context");
    assert_eq!(out.evidence_total_count, 0);
    assert_eq!(out.assertion_total_count, 0);
    assert!(out.evidence_items.is_empty());
    assert!(out.assertions.is_empty());
}
