//! Tier 3 - response parsing. Pure: feeds committed fixture bytes to CPIC
//! decoders and typed rows. No network, no server.

use super::super::*;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue};

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cpic/",
            $name
        ))
    };
}

#[test]
fn pair_page_response_decodes_rows_and_total() {
    let mut headers = HeaderMap::new();
    headers.insert("content-range", HeaderValue::from_static("0-0/12"));
    let content_type = HeaderValue::from_static("application/json");

    let page: CpicPage<Vec<CpicPairRow>> = CpicClient::decode_json_page_response(
        StatusCode::OK,
        &headers,
        Some(&content_type),
        fixture!("pair_view_cyp2d6.json"),
    )
    .expect("pair page");

    assert_eq!(page.total, Some(12));
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].genesymbol, "CYP2D6");
    assert_eq!(page.rows[0].drugname, "codeine");
    assert_eq!(page.rows[0].cpiclevel.as_deref(), Some("A"));
}

#[test]
fn recommendation_and_guideline_responses_decode() {
    let content_type = HeaderValue::from_static("application/json");
    let recs: Vec<CpicRecommendationRow> = CpicClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("recommendations_codeine.json"),
    )
    .expect("recommendations");
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].drugname, "codeine");
    assert_eq!(recs[0].drugrecommendation.as_deref(), Some("Avoid codeine"));
    assert_eq!(recs[0].phenotypes["CYP2D6"], "Poor Metabolizer");

    let guidelines: Vec<CpicGuidelineSummaryRow> = CpicClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("guideline_cyp2d6.json"),
    )
    .expect("guidelines");
    assert_eq!(guidelines[0].guideline_name, "CYP2D6 and Opioids");
    assert_eq!(guidelines[0].genes[0].symbol, "CYP2D6");
}

#[test]
fn frequency_response_decodes_rows() {
    let content_type = HeaderValue::from_static("application/json");
    let rows: Vec<CpicFrequencyRow> = CpicClient::decode_json_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("frequency_cyp2d6.json"),
    )
    .expect("frequencies");

    assert_eq!(rows[0].genesymbol, "CYP2D6");
    assert_eq!(rows[0].name, "*1");
    assert_eq!(rows[0].freq_weighted_avg, Some(0.42));
}

#[test]
fn decode_json_response_maps_http_content_type_and_json_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = CpicClient::decode_json_response::<Vec<CpicPairRow>>(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&content_type),
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let err =
        CpicClient::decode_json_response::<Vec<CpicPairRow>>(StatusCode::OK, Some(&html), b"html")
            .unwrap_err();
    assert_eq!(err.code(), "api");

    let err = CpicClient::decode_json_response::<Vec<CpicPairRow>>(
        StatusCode::OK,
        Some(&content_type),
        b"not json",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api_json");
}
