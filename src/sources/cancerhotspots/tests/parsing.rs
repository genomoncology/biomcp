//! HTTP and content-type policy remains local to the application.
use super::super::*;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

#[test]
fn decode_by_gene_maps_http_and_html_errors() {
    let err = CancerHotspotsClient::decode_by_gene_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let err = CancerHotspotsClient::decode_by_gene_response(
        StatusCode::OK,
        Some(&html),
        b"<html><body>not json</body></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
}
