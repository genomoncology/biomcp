//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response helpers. No network, no server.

use crate::error::BioMcpError;
use crate::sources::decode_json;
use crate::sources::europepmc::{
    EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse, parse_supplementary_zip,
    parse_supplementary_zip_with_limits, supplementary_status_has_package,
};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

const EUROPE_PMC_API: &str = "europepmc";

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/europepmc/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

fn zip_bytes(entries: &[(&str, &[u8])], directories: &[&str]) -> Vec<u8> {
    use std::io::{Cursor, Write};

    let mut out = Cursor::new(Vec::new());
    {
        let mut archive = zip::ZipWriter::new(&mut out);
        let options = zip::write::FileOptions::default();
        for directory in directories {
            archive.add_directory(*directory, options).unwrap();
        }
        for (name, bytes) in entries {
            archive.start_file(*name, options).unwrap();
            archive.write_all(bytes).unwrap();
        }
        archive.finish().unwrap();
    }
    out.into_inner()
}

#[test]
fn parses_search_response_from_real_fixture() {
    let resp: EuropePmcSearchResponse = decode_json(
        EUROPE_PMC_API,
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_pmid_22663011.json"),
        false,
    )
    .unwrap();

    assert_eq!(resp.hit_count, Some(1));
    let result = resp
        .result_list
        .expect("result list")
        .result
        .into_iter()
        .next()
        .expect("result");
    assert_eq!(result.id.as_deref(), Some("22663011"));
    assert_eq!(result.pmid.as_deref(), Some("22663011"));
    assert_eq!(result.doi.as_deref(), Some("10.1056/nejmoa1203421"));
}

#[test]
fn europepmc_result_deserializes_first_index_date() {
    let result: EuropePmcResult = serde_json::from_value(serde_json::json!({
        "id": "22663011",
        "pmid": "22663011",
        "firstPublicationDate": "2025-01-14",
        "firstIndexDate": "2025-01-15"
    }))
    .expect("europepmc result should deserialize");

    assert_eq!(result.first_index_date.as_deref(), Some("2025-01-15"));
}

#[test]
fn decode_full_text_xml_returns_none_on_not_found() {
    let xml = EuropePmcClient::decode_full_text_xml(StatusCode::NOT_FOUND, b"missing").unwrap();
    assert!(xml.is_none());
}

#[test]
fn decode_full_text_xml_returns_body_on_success() {
    let xml = EuropePmcClient::decode_full_text_xml(StatusCode::OK, b"<article/>").unwrap();
    assert_eq!(xml, Some("<article/>".to_string()));
}

#[test]
fn decode_full_text_xml_maps_http_error_status_with_excerpt() {
    let err = EuropePmcClient::decode_full_text_xml(
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("europepmc"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn supplementary_status_distinguishes_absence_from_failure() {
    assert!(!supplementary_status_has_package(StatusCode::NOT_FOUND).unwrap());
    assert!(!supplementary_status_has_package(StatusCode::NO_CONTENT).unwrap());
    assert!(supplementary_status_has_package(StatusCode::OK).unwrap());
    assert!(matches!(
        supplementary_status_has_package(StatusCode::BAD_GATEWAY),
        Err(BioMcpError::Api { .. })
    ));
}

#[test]
fn supplementary_zip_preserves_normalized_names_and_exact_bytes() {
    let bytes = zip_bytes(&[("folder\\asset.docx", b"exact bytes")], &["empty/"]);
    let package = parse_supplementary_zip(&bytes).unwrap();
    assert_eq!(package.entries.len(), 1);
    assert_eq!(package.entries[0].filename, "folder/asset.docx");
    assert_eq!(package.entries[0].bytes, b"exact bytes");
}

#[test]
fn supplementary_zip_rejects_unsafe_and_duplicate_names() {
    for name in [
        "../asset.txt",
        "/asset.txt",
        "C:\\asset.txt",
        "C:asset.txt",
        "folder/../asset.txt",
        "folder//asset.txt",
        "folder/./asset.txt",
        " leading.txt",
        "trailing.txt ",
        "control\nname.txt",
    ] {
        let bytes = zip_bytes(&[(name, b"x")], &[]);
        assert!(
            parse_supplementary_zip(&bytes).is_err(),
            "accepted {name:?}"
        );
    }

    let bytes = zip_bytes(&[("a\\b.txt", b"one"), ("a/b.txt", b"two")], &[]);
    assert!(parse_supplementary_zip(&bytes).is_err());

    let bytes = zip_bytes(&[("folder", b"file")], &["folder/"]);
    assert!(parse_supplementary_zip(&bytes).is_err());
}

#[test]
fn supplementary_zip_rejects_empty_directory_only_and_malformed_archives() {
    assert!(parse_supplementary_zip(&zip_bytes(&[], &[])).is_err());
    assert!(parse_supplementary_zip(&zip_bytes(&[], &["folder/"])).is_err());
    assert!(parse_supplementary_zip(&zip_bytes(&[("file.txt", b"x")], &["../"])).is_err());
    assert!(parse_supplementary_zip(b"not a zip").is_err());
}

#[test]
fn supplementary_zip_enforces_all_size_and_count_limits() {
    let one = zip_bytes(&[("one.txt", b"1234")], &[]);
    assert!(parse_supplementary_zip_with_limits(&one, one.len() - 1, 10, 10, 1).is_err());
    assert!(parse_supplementary_zip_with_limits(&one, one.len(), 3, 10, 1).is_err());

    let two = zip_bytes(&[("one.txt", b"1234"), ("two.txt", b"5678")], &[]);
    assert!(parse_supplementary_zip_with_limits(&two, two.len(), 4, 7, 2).is_err());
    assert!(parse_supplementary_zip_with_limits(&two, two.len(), 4, 8, 1).is_err());
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<EuropePmcSearchResponse>(
        EUROPE_PMC_API,
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        false,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("europepmc"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}
