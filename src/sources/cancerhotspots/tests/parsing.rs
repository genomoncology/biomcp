//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the decoder
//! and recurrence logic. No network, no server.

use super::super::*;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cancerhotspots/",
            $name
        ))
    };
}

#[test]
fn parses_by_gene_fixture() {
    let content_type = HeaderValue::from_static("application/json");
    let rows = CancerHotspotsClient::decode_by_gene_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("by_gene_braf.json"),
    )
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].residue.as_deref(), Some("V600"));
    assert_eq!(rows[0].tumor_count, Some(64));
    assert_eq!(rows[0].variant_amino_acid.get("K"), Some(&64));
}

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

#[test]
fn recurrence_maps_counts_and_transcript_for_exact_alt() {
    let rows = CancerHotspotsClient::decode_by_gene_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        fixture!("by_gene_braf.json"),
    )
    .unwrap();

    let recurrence = recurrence_for_change(&rows, "V600E");
    assert_eq!(recurrence.source, "cancerhotspots.org");
    assert_eq!(recurrence.position_count, Some(897));
    assert_eq!(recurrence.same_aa_count, Some(833));
    assert_eq!(
        recurrence.matched_transcript.as_deref(),
        Some("ENST00000288602")
    );
}

#[test]
fn recurrence_serializes_checked_absence_with_nulls() {
    let recurrence = recurrence_for_change(&[], "G12D");
    let json = serde_json::to_value(&recurrence).unwrap();

    assert_eq!(json["source"], "cancerhotspots.org");
    assert!(json.get("position_count").is_some());
    assert!(json["position_count"].is_null());
    assert!(json["same_aa_count"].is_null());
    assert!(json["matched_transcript"].is_null());
}

#[test]
fn recurrence_treats_missing_exact_alt_as_checked_absence() {
    let rows: Vec<CancerHotspotRow> = serde_json::from_value(serde_json::json!([
        {
            "hugoSymbol": "KRAS",
            "residue": "G12",
            "tumorCount": 100,
            "transcriptId": "ENST00000256078",
            "aminoAcidPosition": 12,
            "variantAminoAcid": {"D": 25}
        }
    ]))
    .unwrap();

    let json = serde_json::to_value(recurrence_for_change(&rows, "G12V")).unwrap();
    assert!(json["position_count"].is_null());
    assert!(json["same_aa_count"].is_null());
    assert!(json["matched_transcript"].is_null());
}

#[test]
fn recurrence_checks_later_matching_residue_rows_for_exact_alt() {
    let rows = CancerHotspotsClient::decode_by_gene_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        fixture!("by_gene_braf.json"),
    )
    .unwrap();

    let recurrence = recurrence_for_change(&rows, "V600E");
    assert!(recurrence.position_count.is_some());
    assert!(recurrence.same_aa_count.is_some());
    assert_eq!(
        recurrence.matched_transcript.as_deref(),
        Some("ENST00000288602")
    );
}
