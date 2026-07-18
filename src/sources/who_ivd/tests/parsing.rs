//! Tier 3 - CSV parsing and local result shaping. Pure: reads committed WHO IVD
//! fixtures and validates output rows. No network.

use super::super::*;
use crate::test_support::TempDirGuard;
use reqwest::header::HeaderValue;

#[test]
fn parse_who_ivd_csv_requires_expected_headers() {
    let err = parse_who_ivd_csv("wrong,header\n1,2\n").expect_err("parse should fail");
    let message = format!("{err:?}");
    assert!(message.contains("missing required column"));
}

#[test]
fn ensure_csv_content_type_rejects_html_response() {
    let content_type = HeaderValue::from_static("text/html; charset=utf-8");
    let err = ensure_csv_content_type(Some(&content_type), b"<html><body>not csv</body></html>")
        .expect_err("html should be rejected");
    let message = format!("{err:?}");

    assert!(message.contains("Unexpected HTML response"));
    assert!(!message.contains("<html>"));
}

#[test]
fn parse_who_ivd_csv_reads_fixture_rows() {
    let rows = parse_who_ivd_csv(&super::fixture_csv()).expect("fixture should parse");

    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0],
        WhoIvdRecord {
            product_code: "ITPW02232- TC40".to_string(),
            product_name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
            target_marker: "HIV".to_string(),
            manufacturer_name: "InTec Products, Inc.".to_string(),
            assay_format: "Immunochromatographic (lateral flow)".to_string(),
            regulatory_version: "Rest-of-World".to_string(),
            prequalification_year: "2019".to_string(),
        }
    );
}

#[test]
fn parse_who_ivd_csv_deduplicates_first_product_code() {
    let payload = "\"Product name\",\"Product Code\",\"WHO Product ID\",\"Assay Format\",\"Regulatory Version\",\"Manufacturer name\",\"Pathogen/Disease/Marker\",\"Year prequalification\"\n\
\"First\",\"ABC 123\",\"1\",\"Lateral flow\",\"ROW\",\"Maker A\",\"HIV\",\"2024\"\n\
\"Second\",\"ABC 123\",\"2\",\"NAT\",\"EU\",\"Maker B\",\"TB\",\"2025\"\n";

    let rows = parse_who_ivd_csv(payload).expect("payload should parse");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].product_name, "First");
    assert_eq!(rows[0].target_marker, "HIV");
}

#[test]
fn who_ivd_client_get_matches_exact_trimmed_product_code() {
    let root = TempDirGuard::new("who-ivd-read-rows");
    std::fs::write(root.path().join(WHO_IVD_CSV_FILE), super::fixture_csv())
        .expect("write fixture");
    let client = WhoIvdClient::from_root(root.path());

    let row = client
        .get(" ITPW02232- TC40 ")
        .expect("lookup should work")
        .expect("row should exist");

    assert_eq!(row.product_name, "ONE STEP Anti-HIV (1&2) Test");
}
