//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to text/XML
//! helpers. No network, no server.

use crate::error::BioMcpError;
use crate::sources::ncbi_efetch::{NcbiEfetchClient, normalize_article_xml};
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/ncbi_efetch/",
            $name
        ))
    };
}

#[test]
fn normalize_article_xml_extracts_article_from_wrapped_fixture() {
    let xml = normalize_article_xml(fixture!("pmc_wrapped_article.xml"))
        .unwrap()
        .expect("article");

    assert!(xml.starts_with("<article"));
    assert!(xml.contains("<article-title>Wrapped</article-title>"));
    assert!(xml.contains("Body text at 70 &#181;m."));
    assert!(!xml.contains("<pmc-articleset>"));
    assert!(!xml.contains("<!DOCTYPE"));
}

#[test]
fn normalize_article_xml_returns_none_for_blank_input() {
    assert_eq!(normalize_article_xml("   ").unwrap(), None);
}

#[test]
fn normalize_article_xml_keeps_unwrapped_xml() {
    let xml = normalize_article_xml("<root><value>Body</value></root>")
        .unwrap()
        .expect("xml");
    assert_eq!(xml, "<root><value>Body</value></root>");
}

#[test]
fn normalize_article_xml_keeps_original_malformed_and_entity_bearing_xml() {
    for input in [
        "<article><body>",
        r#"<!DOCTYPE article [<!ENTITY unsafe "value">]><article>&unsafe;</article>"#,
    ] {
        assert_eq!(
            normalize_article_xml(input).unwrap().as_deref(),
            Some(input)
        );
    }
}

#[test]
fn decode_text_treats_documented_absence_statuses_as_empty() {
    assert_eq!(
        NcbiEfetchClient::decode_text(StatusCode::NOT_FOUND, b"not found").unwrap(),
        ""
    );
    assert_eq!(
        NcbiEfetchClient::decode_text(StatusCode::NO_CONTENT, b"").unwrap(),
        ""
    );
}

#[test]
fn decode_text_maps_http_error_status_with_excerpt() {
    let err = NcbiEfetchClient::decode_text(StatusCode::INTERNAL_SERVER_ERROR, b"upstream failure")
        .unwrap_err();
    let msg = format!("{err:?}");
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("pubmed-eutils"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_text_rejects_invalid_utf8() {
    let err = NcbiEfetchClient::decode_text(StatusCode::OK, b"<article>\xff</article>")
        .expect_err("invalid XML bytes must remain a source failure");
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(!err.to_string().contains('�'));
}
