//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the decoder
//! and GWAS error mapper. No network, no server.

use super::super::*;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gwas/",
            $name
        ))
    };
}

#[test]
fn de_opt_f64_accepts_string_numbers() {
    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(deserialize_with = "de_opt_f64")]
        value: Option<f64>,
    }

    let parsed: Wrapper = serde_json::from_str("{\"value\":\"8e-12\"}").expect("parse");
    assert_eq!(parsed.value, Some(8e-12));
}

#[test]
fn associations_response_parses_rows() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: GwasAssociationsResponse = GwasClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("associations_rsid.json"),
    )
    .unwrap()
    .expect("associations response");
    let rows = resp.embedded.associations;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].pvalue, Some(8e-12));
    assert_eq!(rows[0].or_per_copy_num, Some(1.54));
    assert_eq!(rows[0].risk_frequency, Some(0.04));
}

#[test]
fn v2_association_search_response_keeps_rows_and_provider_total() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: GwasV2AssociationsResponse = GwasClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("associations_gene_tcf7l2_v2_20260812.json"),
    )
    .unwrap()
    .expect("v2 association response");

    assert_eq!(resp.embedded.associations.len(), 2);
    assert_eq!(resp.page.total_elements, 902);
    assert_eq!(
        resp.embedded.associations[0].snp_allele[0].rs_id.as_deref(),
        Some("rs7903146")
    );
    assert_eq!(resp.embedded.associations[0].risk_frequency, Some(0.22941));
}

#[test]
fn associations_by_study_fallback_parse_path_can_read_fallback_response() {
    let content_type = HeaderValue::from_static("application/json");
    let search: GwasAssociationsResponse = GwasClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("associations_empty.json"),
    )
    .unwrap()
    .expect("search response");
    assert!(search.embedded.associations.is_empty());

    let fallback: GwasAssociationsResponse = GwasClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("associations_study_fallback.json"),
    )
    .unwrap()
    .expect("fallback response");
    assert_eq!(fallback.embedded.associations.len(), 1);
    assert_eq!(fallback.embedded.associations[0].pvalue, Some(1.0e-8));
}

#[test]
fn decode_json_optional_returns_none_on_not_found() {
    let decoded: Option<GwasAssociationsResponse> =
        GwasClient::decode_json_optional(StatusCode::NOT_FOUND, None, b"").unwrap();
    assert!(decoded.is_none());
}

#[test]
fn decode_failures_remap_to_source_unavailable() {
    let content_type = HeaderValue::from_static("application/json");
    let err = GwasClient::decode_json_optional::<GwasAssociationsResponse>(
        StatusCode::OK,
        Some(&content_type),
        b"{not-json",
    )
    .map_err(remap_gwas_error)
    .expect_err("decode failure should be remapped");

    assert_eq!(err.code(), "source_unavailable");
    let detail = format!("{err:?}");
    assert!(detail.contains("GWAS Catalog"));
    assert!(detail.contains("could not decode"));
}

#[test]
fn transient_http_failures_remap_to_source_unavailable() {
    let content_type = HeaderValue::from_static("application/json");
    let err = GwasClient::decode_json_optional::<GwasAssociationsResponse>(
        StatusCode::SERVICE_UNAVAILABLE,
        Some(&content_type),
        b"maintenance",
    )
    .map_err(remap_gwas_error)
    .expect_err("503 should be remapped");

    assert_eq!(err.code(), "source_unavailable");
    let detail = format!("{err:?}");
    assert!(detail.contains("GWAS Catalog"));
    assert!(detail.contains("temporarily unavailable"));
}
