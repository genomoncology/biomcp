//! Response parsing and local result-shaping tests. Pure: feed status,
//! content type, and bytes into decode helpers. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::*;

#[test]
fn article_response_normalizes_files_and_license() {
    let article = FigshareClient::decode_article_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        &article_response_bytes(),
        &article_reference(),
    )
    .unwrap();

    assert_eq!(article.files.len(), 2);
    assert_eq!(article.files[0].filename, "figshare-supplement.pdf");
    assert_eq!(
        article
            .license
            .as_ref()
            .and_then(|license| license.name.as_deref()),
        Some("CC BY 4.0")
    );
}

#[test]
fn search_response_normalizes_rows() {
    let rows = FigshareClient::decode_search_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        &search_response_bytes(),
    )
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].article_id, 22474817);
    assert_eq!(rows[0].title.as_deref(), Some("Example"));
    assert_eq!(rows[0].doi.as_deref(), Some("10.1000/example"));
}

#[test]
fn download_response_returns_bytes_for_success() {
    let response = FigshareClient::decode_download_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/pdf")),
        b"PDF bytes".to_vec(),
        0,
    )
    .unwrap();

    match response {
        DownloadResponse::Bytes(bytes) => assert_eq!(bytes, b"PDF bytes"),
        DownloadResponse::Retry => panic!("expected bytes"),
    }
}

#[test]
fn download_response_rejects_oversized_file_bytes() {
    let err = FigshareClient::decode_download_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/pdf")),
        vec![b'x'; MAX_FIGSHARE_FILE_BYTES + 1],
        0,
    )
    .unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("exceeded"));
}

#[test]
fn download_error_sanitizes_html_body() {
    let err = FigshareClient::decode_download_response(
        StatusCode::SERVICE_UNAVAILABLE,
        Some(&HeaderValue::from_static("text/html; charset=utf-8")),
        b"<html><body>upstream detail</body></html>".to_vec(),
        0,
    )
    .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("Figshare"));
    assert!(message.to_ascii_lowercase().contains("retry"));
    assert!(!message.contains("HTML error page"));
    assert!(!message.contains("<html"));
    assert!(!message.contains("upstream detail"));
}

#[test]
fn download_accepted_response_requests_retry_before_limit() {
    let response = FigshareClient::decode_download_response(
        StatusCode::ACCEPTED,
        None,
        Vec::new(),
        FIGSHARE_DOWNLOAD_ACCEPTED_RETRIES - 1,
    )
    .unwrap();

    assert!(matches!(response, DownloadResponse::Retry));
}

#[test]
fn download_errors_after_repeated_accepted_responses() {
    let err = FigshareClient::decode_download_response(
        StatusCode::ACCEPTED,
        None,
        Vec::new(),
        FIGSHARE_DOWNLOAD_ACCEPTED_RETRIES,
    )
    .unwrap_err();

    assert!(format!("{err:?}").contains("still staging"));
}

#[test]
fn article_error_sanitizes_html_body() {
    let err = FigshareClient::decode_article_response(
        StatusCode::SERVICE_UNAVAILABLE,
        Some(&HeaderValue::from_static("text/html; charset=utf-8")),
        b"<html><body>upstream detail</body></html>",
        &article_reference(),
    )
    .unwrap_err();
    let message = err.to_string();

    assert!(message.contains("Figshare"));
    assert!(message.to_ascii_lowercase().contains("retry"));
    assert!(!message.contains("HTML error page"));
    assert!(!message.contains("<html"));
    assert!(!message.contains("upstream detail"));
}
