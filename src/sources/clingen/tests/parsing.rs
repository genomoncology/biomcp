//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to ClinGen
//! decoders and CSV parsers. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen/",
            $name
        ))
    };
}

fn lookup_rows(bytes: &[u8]) -> Vec<ClinGenLookupGeneRow> {
    ClinGenClient::decode_json_response(
        CLINGEN_API,
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        bytes,
    )
    .expect("lookup rows")
}

#[test]
fn lookup_accepts_json_with_html_content_type() {
    let rows: Vec<ClinGenLookupGeneRow> = ClinGenClient::decode_json_response(
        CLINGEN_API,
        StatusCode::OK,
        Some(&HeaderValue::from_static("text/html; charset=UTF-8")),
        fixture!("lookup_braf.json"),
    )
    .expect("mislabeled lookup json");

    assert_eq!(
        hgnc_id_from_lookup_rows("BRAF", &rows).as_deref(),
        Some("HGNC:1097")
    );
}

#[test]
fn gene_validity_parses_csv_with_metadata_rows() {
    let rows = lookup_rows(fixture!("lookup_braf.json"));
    let hgnc_id = hgnc_id_from_lookup_rows("BRAF", &rows);
    let csv_payload =
        ClinGenClient::decode_text_response(CLINGEN_API, StatusCode::OK, fixture!("validity.csv"))
            .expect("validity csv");
    let validity = parse_validity_csv(&csv_payload, "BRAF", hgnc_id.as_deref()).unwrap();

    assert_eq!(validity.len(), 2);
    assert_eq!(validity[0].disease, "cardiofaciocutaneous syndrome");
    assert_eq!(validity[0].classification, "Definitive");
    assert_eq!(validity[0].review_date.as_deref(), Some("2024-01-12"));
    assert_eq!(validity[1].review_date.as_deref(), Some("2023-05-01"));
}

#[test]
fn dosage_sensitivity_parses_csv_and_picks_latest_row() {
    let rows = lookup_rows(fixture!("lookup_braf.json"));
    let hgnc_id = hgnc_id_from_lookup_rows("BRAF", &rows);
    let csv_payload =
        ClinGenClient::decode_text_response(CLINGEN_API, StatusCode::OK, fixture!("dosage.csv"))
            .expect("dosage csv");
    let (haplo, triplo) = parse_dosage_csv(&csv_payload, "BRAF", hgnc_id.as_deref()).unwrap();

    assert_eq!(
        haplo.as_deref(),
        Some("Sufficient Evidence for Haploinsufficiency")
    );
    assert_eq!(triplo.as_deref(), Some("No Evidence for Triplosensitivity"));
}

#[test]
fn gene_context_can_be_built_from_one_lookup_and_both_csv_payloads() {
    let rows = lookup_rows(fixture!("lookup_braf.json"));
    let hgnc_id = hgnc_id_from_lookup_rows("BRAF", &rows);
    let validity = parse_validity_csv(
        std::str::from_utf8(fixture!("validity.csv")).unwrap(),
        "BRAF",
        hgnc_id.as_deref(),
    )
    .unwrap();
    let (haploinsufficiency, triplosensitivity) = parse_dosage_csv(
        std::str::from_utf8(fixture!("dosage.csv")).unwrap(),
        "BRAF",
        hgnc_id.as_deref(),
    )
    .unwrap();
    let context = GeneClinGen {
        validity,
        haploinsufficiency,
        triplosensitivity,
    };

    assert_eq!(context.validity.len(), 2);
    assert_eq!(
        context.haploinsufficiency.as_deref(),
        Some("Sufficient Evidence for Haploinsufficiency")
    );
    assert_eq!(
        context.triplosensitivity.as_deref(),
        Some("No Evidence for Triplosensitivity")
    );
}

#[test]
fn clingen_parsers_handle_missing_gene_rows_cleanly() {
    let validity = parse_validity_csv(
        std::str::from_utf8(fixture!("validity.csv")).unwrap(),
        "NRAS",
        None,
    )
    .unwrap();
    let dosage = parse_dosage_csv(
        std::str::from_utf8(fixture!("dosage.csv")).unwrap(),
        "NRAS",
        None,
    )
    .unwrap();

    assert!(validity.is_empty());
    assert_eq!(dosage, (None, None));
}

#[test]
fn hgnc_lookup_allows_hgnc_only_validity_match() {
    let rows = lookup_rows(fixture!("lookup_braf.json"));
    let hgnc_id = hgnc_id_from_lookup_rows("BRAF", &rows);
    let validity = parse_validity_csv(
        std::str::from_utf8(fixture!("validity_hgnc_only.csv")).unwrap(),
        "BRAF",
        hgnc_id.as_deref(),
    )
    .unwrap();

    assert_eq!(validity.len(), 1);
    assert_eq!(validity[0].disease, "Noonan syndrome");
}

#[test]
fn decode_text_and_json_map_http_errors() {
    let err = ClinGenClient::decode_text_response(CLINGEN_API, StatusCode::BAD_GATEWAY, b"down")
        .unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("502"));

    let err = ClinGenClient::decode_json_response::<Vec<ClinGenLookupGeneRow>>(
        CLINGEN_API,
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&HeaderValue::from_static("application/json")),
        b"upstream failed",
    )
    .unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("500"));
}
