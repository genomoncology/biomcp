//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response types. No network, no server.

use crate::sources::decode_json;
use crate::sources::litsense2::LitSense2SearchHit;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/litsense2/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_sentence_response_from_real_fixture() {
    let hits: Vec<LitSense2SearchHit> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITSENSE2),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("sentence_hirschsprung.json"),
        true,
    )
    .unwrap();

    assert!(!hits.is_empty());
    assert_eq!(hits[0].pmid, 12_939_702);
    assert!(hits[0].text.contains("Hirschsprung"));
    assert_eq!(hits[0].section.as_deref(), Some("abstract"));
    assert!(
        hits[0]
            .annotations
            .iter()
            .any(|value| value.contains("MESH:D006627"))
    );
}

#[test]
fn paragraph_shape_tolerates_null_annotations_and_trimmed_optionals() {
    let hits: Vec<LitSense2SearchHit> = serde_json::from_value(serde_json::json!([
        {
            "pmid": 123,
            "pmcid": "  PMC123  ",
            "text": "Paragraph match",
            "score": 0.5,
            "section": "  INTRO  ",
            "annotations": null
        },
        {
            "pmid": 456,
            "pmcid": null,
            "text": "Second match",
            "score": 0.4,
            "section": null,
            "annotations": ["0|1|gene|BRAF"]
        }
    ]))
    .unwrap();

    assert_eq!(hits[0].pmcid.as_deref(), Some("PMC123"));
    assert_eq!(hits[0].section.as_deref(), Some("INTRO"));
    assert!(hits[0].annotations.is_empty());
    assert!(hits[1].pmcid.is_none());
    assert!(hits[1].section.is_none());
    assert_eq!(hits[1].annotations, vec!["0|1|gene|BRAF"]);
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<Vec<LitSense2SearchHit>>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITSENSE2),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert_eq!(err.code(), "api");
    assert!(msg.contains("LitSense 2"), "got: {msg}");
    assert!(!msg.contains("500"), "got: {msg}");
    assert!(!msg.contains("upstream failure"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<Vec<LitSense2SearchHit>>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::LITSENSE2),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("LitSense 2"), "got: {msg}");
    assert!(!msg.contains("HTML"), "got: {msg}");
}
