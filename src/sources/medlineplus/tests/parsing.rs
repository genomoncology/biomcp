//! Response parsing tests. Pure: feed status, content type, and XML bytes into
//! local helpers. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::*;

#[test]
fn decode_response_body_accepts_xml() {
    let xml = MedlinePlusClient::decode_response_body(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/xml")),
        topic_xml().as_bytes().to_vec(),
    )
    .unwrap();
    let topics = parse_topics(&xml).unwrap();

    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].title, "Chest Pain");
    assert_eq!(topics[0].summary_excerpt, "Summary");
}

#[test]
fn parse_topics_decodes_inline_markup() {
    let topics = parse_topics(marked_up_topic_xml()).expect("topics");

    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0].title, "Chest Pain");
    assert_eq!(topics[0].summary_excerpt, "Chest pain summary.");
}

#[test]
fn decode_response_body_rejects_html_content_type() {
    let err = MedlinePlusClient::decode_response_body(
        StatusCode::OK,
        Some(&HeaderValue::from_static("text/html; charset=utf-8")),
        b"<html><body>login</body></html>".to_vec(),
    )
    .unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}

#[test]
fn decode_response_body_reports_http_errors() {
    let err = MedlinePlusClient::decode_response_body(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&HeaderValue::from_static("application/xml")),
        b"upstream failure".to_vec(),
    )
    .unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("500"));
}

#[test]
fn decode_response_body_rejects_invalid_utf8() {
    let err = MedlinePlusClient::decode_response_body(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/xml")),
        vec![0xff, 0xfe, 0xfd],
    )
    .unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("valid UTF-8 XML"));
}
