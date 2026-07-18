//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
//! the response types, plus the pure post-processing helpers. No network, no server.

use crate::sources::decode_json;
use crate::sources::mygene::{
    MyGeneBatchGeneHit, MyGeneClient, MyGeneGetQueryResponse, MyGeneSearchResponse,
    extract_uniprot_accession,
};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/mygene/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_search_response_from_real_fixture() {
    let resp: MyGeneSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_egfr.json"),
        true,
    )
    .unwrap();
    assert!(!resp.hits.is_empty());
    assert!(resp.hits[0].symbol.is_some());
}

#[test]
fn parses_get_response_fields_from_real_fixture() {
    let resp: MyGeneGetQueryResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_braf.json"),
        true,
    )
    .unwrap();
    let hit = resp.hits.into_iter().next().expect("a hit");
    assert_eq!(hit.symbol.as_deref(), Some("BRAF"));
    assert_eq!(
        hit.ensembl
            .as_ref()
            .and_then(|e| e.gene())
            .map(String::as_str),
        Some("ENSG00000157764")
    );
    assert_eq!(
        hit.genomic_pos
            .as_ref()
            .and_then(|g| g.chr())
            .map(String::as_str),
        Some("7")
    );
}

#[test]
fn extract_uniprot_prefers_swiss_prot_from_real_fixture() {
    let resp: MyGeneGetQueryResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_braf.json"),
        true,
    )
    .unwrap();
    let hit = resp.hits.into_iter().next().expect("a hit");
    let uniprot = hit.uniprot.as_ref().expect("real BRAF carries uniprot");
    assert_eq!(
        extract_uniprot_accession(uniprot).as_deref(),
        Some("P15056")
    );
}

#[test]
fn extract_uniprot_prefers_swiss_prot_over_trembl_synthetic() {
    let value = serde_json::json!({ "Swiss-Prot": ["P15056"], "TrEMBL": ["A0A0A0"] });
    assert_eq!(extract_uniprot_accession(&value).as_deref(), Some("P15056"));
}

#[test]
fn extract_uniprot_returns_none_for_empty_object() {
    assert_eq!(extract_uniprot_accession(&serde_json::json!({})), None);
}

#[test]
fn dedupe_symbols_maps_real_batch_in_input_order() {
    let rows: Vec<MyGeneBatchGeneHit> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("batch_symbols.json"),
        true,
    )
    .unwrap();
    // fixture maps 1956 -> EGFR, 7157 -> TP53, 673 -> BRAF
    let ids = vec!["7157".to_string(), "1956".to_string(), "673".to_string()];
    assert_eq!(
        MyGeneClient::dedupe_symbols_in_order(rows, &ids),
        vec!["TP53", "EGFR", "BRAF"]
    );
}

#[test]
fn dedupe_symbols_dedupes_repeated_ids_keeping_first_position() {
    let rows: Vec<MyGeneBatchGeneHit> = serde_json::from_str(
        r#"[{"query":"1956","_id":"1956","symbol":"EGFR"},{"query":"7157","_id":"7157","symbol":"TP53"}]"#,
    )
    .unwrap();
    let ids = vec!["1956".to_string(), "7157".to_string(), "1956".to_string()];
    assert_eq!(
        MyGeneClient::dedupe_symbols_in_order(rows, &ids),
        vec!["EGFR", "TP53"]
    );
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<MyGeneSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("MyGene.info"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<MyGeneSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("MyGene.info"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
