//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the decoder
//! and local row processors. No network, no server.

use super::super::*;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gtex/",
            $name
        ))
    };
}

fn decode<T: serde::de::DeserializeOwned>(name: &str, bytes: &[u8]) -> T {
    let content_type = HeaderValue::from_static("application/json");
    GtexClient::decode_json_response(StatusCode::OK, Some(&content_type), bytes)
        .unwrap_or_else(|err| panic!("decode {name}: {err}"))
}

#[test]
fn resolve_versioned_id_uses_gene_search_response() {
    let resp: GtexGeneSearchResponse =
        decode("gene_search_braf", fixture!("gene_search_braf.json"));
    let resolved =
        GtexClient::resolve_versioned_gencode_id_from_response("ENSG00000157764", resp).unwrap();

    assert_eq!(resolved.as_deref(), Some("ENSG00000157764.12"));
}

#[test]
fn resolve_versioned_id_returns_first_non_empty_fallback() {
    let resp: GtexGeneSearchResponse = decode(
        "gene_search_fallback",
        fixture!("gene_search_fallback.json"),
    );
    let resolved =
        GtexClient::resolve_versioned_gencode_id_from_response("ENSG00000999999", resp).unwrap();

    assert_eq!(resolved.as_deref(), Some("ENSG00000157764.12"));
}

#[test]
fn median_expression_sorts_and_compacts_to_top_and_low_tissues() {
    let resp: GtexMedianExpressionResponse =
        decode("median_expression", fixture!("median_expression.json"));
    let rows = GtexClient::median_expression_rows_from_response(resp);
    let tissues = compact_tissue_rows(rows);

    assert_eq!(tissues.len(), 13);
    assert_eq!(
        tissues.first().map(|row| row.tissue.as_str()),
        Some("Tissue 14")
    );
    assert!(tissues.iter().any(|row| row.tissue == "Tissue 1"));
    assert!(!tissues.iter().any(|row| row.tissue == "Tissue 4"));
}

#[test]
fn median_expression_returns_empty_when_gene_search_has_no_match() {
    let resp: GtexGeneSearchResponse =
        decode("gene_search_empty", fixture!("gene_search_empty.json"));
    let resolved =
        GtexClient::resolve_versioned_gencode_id_from_response("ENSG00000157764", resp).unwrap();

    assert!(resolved.is_none());
}

#[test]
fn decode_json_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = GtexClient::decode_json_response::<GtexGeneSearchResponse>(
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
    let err = GtexClient::decode_json_response::<GtexGeneSearchResponse>(
        StatusCode::OK,
        Some(&html),
        b"<html></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
}
