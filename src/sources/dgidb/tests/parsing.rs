//! Tier 3 - response parsing. Pure: feeds committed fixture bytes to the DGIdb
//! GraphQL decoder and mapper. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/dgidb/",
            $name
        ))
    };
}

#[test]
fn gene_interactions_response_aggregates_categories_and_interactions() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: GraphQlResponse<DgidbGeneData> = DgidbClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("gene_braf_interactions.json"),
    )
    .expect("graphql response");
    let out = DgidbClient::druggability_from_response(resp).expect("druggability");

    assert_eq!(out.categories, vec!["Kinase", "Protein Kinase"]);
    assert_eq!(out.interactions.len(), 2);

    let first = &out.interactions[0];
    assert_eq!(first.drug, "DABRAFENIB");
    assert_eq!(first.score, Some(1.2));
    assert_eq!(first.approved, Some(true));
    assert_eq!(first.source_count, 2);
    assert_eq!(first.interaction_types, vec!["antagonist", "inhibitor"]);

    let second = &out.interactions[1];
    assert_eq!(second.drug, "SORAFENIB");
    assert_eq!(second.score, Some(0.4));
}

#[test]
fn gene_interactions_response_surfaces_graphql_errors() {
    let resp: GraphQlResponse<DgidbGeneData> = serde_json::from_value(serde_json::json!({
        "errors": [{"message": "GraphQL validation failed"}]
    }))
    .unwrap();

    let err = DgidbClient::druggability_from_response(resp).unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("GraphQL validation failed"));
}

#[test]
fn decode_json_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = DgidbClient::decode_json_response::<GraphQlResponse<DgidbGeneData>>(
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
    let err = DgidbClient::decode_json_response::<GraphQlResponse<DgidbGeneData>>(
        StatusCode::OK,
        Some(&html),
        b"<html></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
}
