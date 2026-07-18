//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::*;

#[test]
fn map_search_hits_filters_non_human_invalid_and_duplicate_rows() {
    let response: WikiPathwaysSearchResponse = serde_json::from_str(
        r#"{
          "pathwayInfo": [
            {"id": "WP111", "name": "Alpha", "species": "Homo sapiens"},
            {"id": "WP111", "name": "Alpha duplicate", "species": "Homo sapiens"},
            {"id": "WP222", "name": "Mouse only", "species": "Mus musculus"},
            {"id": "BAD", "name": "Bad", "species": "Homo sapiens"},
            {"id": "WP333", "name": "", "species": "Homo sapiens"},
            {"id": "WP444", "name": "Alpha beta", "species": "Homo sapiens"}
          ]
        }"#,
    )
    .unwrap();

    let hits = WikiPathwaysClient::map_search_hits(response, "alpha", 10);

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, "WP111");
    assert_eq!(hits[1].id, "WP444");
}

#[test]
fn map_pathway_record_parses_minimal_detail_payload() {
    let response: WikiPathwaysGetResponse = serde_json::from_str(
        r#"{"pathwayInfo":[{"id":"WP254","name":"Apoptosis","species":"Homo sapiens","revision":"140926"}]}"#,
    )
    .unwrap();

    let record = WikiPathwaysClient::map_pathway_record(response, "WP254").unwrap();

    assert_eq!(record.id, "WP254");
    assert_eq!(record.name, "Apoptosis");
    assert_eq!(record.species.as_deref(), Some("Homo sapiens"));
}

#[test]
fn map_pathway_entrez_gene_ids_dedupes_and_filters_non_numeric_rows() {
    let response: WikiPathwaysXrefResponse = serde_json::from_str(
        r#"{"pathwayInfo":[{"id":"WP254","ncbigene":"ncbigene:7157, ncbigene:1956, ncbigene:7157; ncbigene:BAD, , ncbigene:672"}]}"#,
    )
    .unwrap();

    let ids = WikiPathwaysClient::map_pathway_entrez_gene_ids(response, "WP254");

    assert_eq!(ids, vec!["7157", "1956", "672"]);
}

#[test]
fn decode_json_response_rejects_html_content_type_before_json_parse() {
    let content_type = HeaderValue::from_static("text/html");
    let err = WikiPathwaysClient::decode_json_response::<WikiPathwaysSearchResponse>(
        StatusCode::OK,
        Some(&content_type),
        b"<html><body>error page</body></html>",
    )
    .unwrap_err();

    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}

#[test]
fn decode_json_response_sanitizes_404_html_error_body() {
    let content_type = HeaderValue::from_static("text/html; charset=utf-8");
    let err = WikiPathwaysClient::decode_json_response::<WikiPathwaysSearchResponse>(
        StatusCode::NOT_FOUND,
        Some(&content_type),
        b"<!DOCTYPE html><html><head><title>404</title></head><body>File not found</body></html>",
    )
    .unwrap_err();
    let msg = err.to_string();

    assert!(msg.contains("WikiPathways"));
    assert!(msg.to_ascii_lowercase().contains("retry"));
    assert!(!msg.contains("HTTP 404"));
    assert!(!msg.contains("HTML error page"));
    assert!(!msg.contains("<!DOCTYPE"));
    assert!(!msg.contains("<html"));
    assert!(!msg.contains("<head"));
}
