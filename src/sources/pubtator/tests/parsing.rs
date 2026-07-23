//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response types. No network, no server.

use super::super::*;
use crate::sources::decode_json;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pubtator/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_export_response_fixture() {
    let resp: PubTatorExportResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("export_22663011.json"),
        true,
    )
    .unwrap();

    assert_eq!(resp.documents.len(), 1);
    let doc = &resp.documents[0];
    assert_eq!(doc.pmid, Some(22663011));
    assert_eq!(doc.pmcid.as_deref(), Some("PMC3326122"));
    assert_eq!(doc.passages.len(), 1);
    assert_eq!(
        doc.passages[0]
            .infons
            .as_ref()
            .and_then(|i| i.kind.as_deref()),
        Some("abstract")
    );
    assert_eq!(doc.relations[0].id.as_deref(), Some("relation-1"));
    assert_eq!(doc.relations[0].nodes[0].refid.as_deref(), Some("gene-1"));
    assert!(doc.passages[0].annotations.iter().any(|annotation| {
        annotation.id.as_deref() == Some("gene-1") && annotation.text.as_deref() == Some("BRAF")
    }));
}

#[test]
fn parses_autocomplete_response_fixture() {
    let resp: Vec<PubTatorAutocompleteResult> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("autocomplete_braf.json"),
        true,
    )
    .unwrap();

    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].id.as_deref(), Some("@GENE_BRAF"));
    assert_eq!(resp[0].biotype.as_deref(), Some("gene"));
    assert_eq!(resp[0].db_id.as_deref(), Some("673"));
    assert_eq!(resp[0].name.as_deref(), Some("BRAF"));
}

#[test]
fn parses_search_response_fixture_and_stringifies_numeric_pmid() {
    let resp: PubTatorSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf.json"),
        true,
    )
    .unwrap();

    assert_eq!(resp.count, Some(1));
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].id.as_deref(), Some("123"));
    assert_eq!(resp.results[0].pmid.as_deref(), Some("123"));
    assert_eq!(
        resp.results[0].title.as_deref(),
        Some("BRAF alterations in melanoma")
    );
}

#[test]
fn search_result_trims_empty_string_pmids_to_none() {
    let result: PubTatorSearchResult = serde_json::from_value(serde_json::json!({
        "_id": "empty-pmid",
        "pmid": "   ",
        "title": "No PMID"
    }))
    .unwrap();

    assert_eq!(result.pmid, None);
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<PubTatorSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("PubTator 3"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<PubTatorSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("PubTator 3"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
