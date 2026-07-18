//! Tier 3 - response parsing. Pure: decodes XML fixture bodies and maps them
//! into aggregate tables. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

#[test]
fn decode_aggregate_response_accepts_wonder_html_content_type_with_xml_body() {
    let content_type = HeaderValue::from_static("text/html; charset=ISO-8859-1");
    let xml = decode_aggregate_response(
        StatusCode::OK,
        Some(&content_type),
        super::REACTIONS_RESPONSE_FIXTURE.as_bytes().to_vec(),
    )
    .expect("decode xml response");
    let table = parse_aggregate_response(&xml).expect("parse reactions");

    assert_eq!(table.total_events, 83_359);
    assert_eq!(
        table.rows.first().map(|row| row.label.as_str()),
        Some("ABASIA")
    );
    assert_eq!(table.rows.first().map(|row| row.count), Some(179));
    assert_eq!(table.rows.first().map(|row| row.percentage), Some(0.21));
}

#[test]
fn parse_serious_response_extracts_yes_and_no_rows() {
    let table = parse_aggregate_response(super::SERIOUS_RESPONSE_FIXTURE).expect("parse serious");

    assert_eq!(table.total_events, 83_359);
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].label, "Yes");
    assert_eq!(table.rows[0].count, 5_795);
    assert_eq!(table.rows[1].label, "No");
    assert_eq!(table.rows[1].count, 77_564);
}

#[test]
fn parse_age_response_extracts_buckets_and_skips_total_row() {
    let table = parse_aggregate_response(super::AGE_RESPONSE_FIXTURE).expect("parse age");

    assert_eq!(table.total_events, 83_359);
    assert_eq!(
        table.rows.first().map(|row| row.label.as_str()),
        Some("< 6 months")
    );
    assert_eq!(
        table.rows.last().map(|row| row.label.as_str()),
        Some("Unknown")
    );
    assert_eq!(table.rows.last().map(|row| row.count), Some(10_133));
}

#[test]
fn parse_processing_error_returns_api_message() {
    let err = parse_aggregate_response(
        r#"<?xml version="1.0"?><page><title>Processing Error</title><message>Request rate exceeded.</message></page>"#,
    )
    .expect_err("processing error should fail");

    assert!(format!("{err:?}").contains("Request rate exceeded"));
}

#[test]
fn decode_aggregate_response_rejects_http_html_and_non_utf8_errors() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_aggregate_response(
        StatusCode::BAD_GATEWAY,
        Some(&html),
        b"<html>bad gateway</html>".to_vec(),
    )
    .unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("502"));

    let err = decode_aggregate_response(StatusCode::OK, Some(&html), b"<html></html>".to_vec())
        .unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("HTML"));

    let xml_content_type = HeaderValue::from_static("text/xml");
    let err =
        decode_aggregate_response(StatusCode::OK, Some(&xml_content_type), vec![0xff]).unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("UTF-8"));
}
