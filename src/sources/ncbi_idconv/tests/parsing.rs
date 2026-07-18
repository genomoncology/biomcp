//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response helpers. No network, no server.

use crate::sources::decode_json;
use crate::sources::ncbi_idconv::{NcbiIdConvResponse, NcbiIdConverterClient};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/ncbi_idconv/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_lookup_response_from_real_fixture() {
    let resp: NcbiIdConvResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NCBI_ID_CONVERTER),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("pmid_22663011.json"),
        false,
    )
    .unwrap();

    assert_eq!(resp.status.as_deref(), Some("ok"));
    let record = resp.records.first().expect("record");
    assert_eq!(record.pmid, Some(22663011));
    assert_eq!(record.requested_id.as_deref(), Some("22663011"));
}

#[test]
fn extract_first_pmcid_trims_non_empty_value() {
    let resp: NcbiIdConvResponse =
        serde_json::from_str(r#"{"records":[{"pmcid":" PMC123456 "},{"pmcid":"PMC999999"}]}"#)
            .unwrap();

    assert_eq!(
        NcbiIdConverterClient::extract_first_pmcid(resp).as_deref(),
        Some("PMC123456")
    );
}

#[test]
fn extract_first_pmcid_returns_none_for_missing_or_blank_value() {
    let resp: NcbiIdConvResponse =
        serde_json::from_str(r#"{"records":[{"pmcid":"   "}]}"#).unwrap();
    assert_eq!(NcbiIdConverterClient::extract_first_pmcid(resp), None);

    let resp: NcbiIdConvResponse = serde_json::from_str(r#"{"records":[]}"#).unwrap();
    assert_eq!(NcbiIdConverterClient::extract_first_pmcid(resp), None);
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<NcbiIdConvResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NCBI_ID_CONVERTER),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        false,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("NCBI ID Converter"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}
