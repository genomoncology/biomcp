//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to ClinGen
//! decoders and CSV parsers. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;
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
fn validity_keeps_newest_five_with_deterministic_ties() {
    let csv = concat!(
        "GENE SYMBOL,GENE ID (HGNC),DISEASE LABEL,CLASSIFICATION,CLASSIFICATION DATE,MOI\n",
        "TP53,HGNC:11998,Zeta,Definitive,2026-01-01,AD\n",
        "TP53,HGNC:11998,Alpha,Limited,2026-01-01,AD\n",
        "TP53,HGNC:11998,Alpha,Definitive,2026-01-01,AD\n",
        "TP53,HGNC:11998,D4,Definitive,2025-01-01,AD\n",
        "TP53,HGNC:11998,D5,Definitive,2024-01-01,AD\n",
        "TP53,HGNC:11998,D6,Definitive,2023-01-01,AD\n",
        "TP53,HGNC:11998,D7,Definitive,2022-01-01,AD\n",
    );
    let rows = parse_validity_csv(csv, "TP53", Some("HGNC:11998")).unwrap();

    assert_eq!(rows.len(), 5);
    assert_eq!(
        rows.iter()
            .map(|row| (row.disease.as_str(), row.classification.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("Alpha", "Definitive"),
            ("Alpha", "Limited"),
            ("Zeta", "Definitive"),
            ("D4", "Definitive"),
            ("D5", "Definitive"),
        ]
    );
}

#[test]
fn dosage_newest_row_preserves_a_literal_no_evidence_and_omits_missing_side() {
    let csv = concat!(
        "GENE SYMBOL,HGNC ID,HAPLOINSUFFICIENCY,TRIPLOSENSITIVITY,DATE\n",
        "TP53,HGNC:11998,Sufficient Evidence for Haploinsufficiency,Sufficient Evidence for Triplosensitivity,2020-01-01\n",
        "TP53,HGNC:11998,,No Evidence for Triplosensitivity,2026-01-01\n",
    );
    let (haplo, triplo) = parse_dosage_csv(csv, "TP53", Some("HGNC:11998")).unwrap();

    assert_eq!(haplo, None);
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
        validity_status: ClinGenFamilyStatus::data(ClinGenOperation::GeneValidityDownload),
        dosage_status: ClinGenFamilyStatus::data(ClinGenOperation::GeneDosageDownload),
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
fn family_status_serialization_is_closed_and_omits_healthy_messages() {
    let context = GeneClinGen {
        validity: Vec::new(),
        haploinsufficiency: None,
        triplosensitivity: None,
        validity_status: ClinGenFamilyStatus::timed_out(
            ClinGenOperation::GeneValidityDownload,
            VALIDITY_TIMEOUT_MESSAGE,
        ),
        dosage_status: ClinGenFamilyStatus::empty(ClinGenOperation::GeneDosageDownload),
    };

    assert_eq!(
        serde_json::to_value(context).unwrap(),
        serde_json::json!({
            "validity_status": {
                "status": "timed_out",
                "op": "gene_validity_download",
                "message": "ClinGen gene-validity download timed out."
            },
            "dosage_status": {
                "status": "empty",
                "op": "gene_dosage_download"
            }
        })
    );
}

#[test]
fn downloads_fail_closed_on_unrecognized_schema_html_and_invalid_encoding() {
    let missing_validity_header =
        "GENE SYMBOL,DISEASE LABEL,CLASSIFICATION\nTP53,cancer,Definitive\n";
    assert!(parse_validity_csv(missing_validity_header, "TP53", None).is_err());

    let missing_dosage_header = "GENE SYMBOL,HGNC ID,HAPLOINSUFFICIENCY,TRIPLOSENSITIVITY\nTP53,HGNC:11998,Sufficient Evidence,\n";
    assert!(parse_dosage_csv(missing_dosage_header, "TP53", None).is_err());

    let malformed_validity = "GENE SYMBOL,GENE ID (HGNC),DISEASE LABEL,CLASSIFICATION,CLASSIFICATION DATE,MOI\nTP53,HGNC:11998,Li-Fraumeni syndrome\n";
    assert!(parse_validity_csv(malformed_validity, "TP53", None).is_err());

    assert!(
        ClinGenClient::decode_text_response(
            CLINGEN_API,
            StatusCode::OK,
            b"<!doctype html><html><body>maintenance</body></html>",
        )
        .is_err()
    );
    assert!(ClinGenClient::decode_text_response(CLINGEN_API, StatusCode::OK, b"\xff\xfe").is_err());
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
fn clingen_plans_set_lookup_and_download_paths() {
    let lookup = ClinGenClient::gene_lookup_plan(" braf ").unwrap();
    assert_eq!(lookup.method, HttpMethod::Get);
    assert_eq!(lookup.path, "api/genes/look/BRAF");
    assert!(lookup.query.is_empty());

    let validity = ClinGenClient::validity_download_plan();
    assert_eq!(validity.method, HttpMethod::Get);
    assert_eq!(validity.path, "kb/gene-validity/download");

    let dosage = ClinGenClient::dosage_download_plan();
    assert_eq!(dosage.method, HttpMethod::Get);
    assert_eq!(dosage.path, "kb/gene-dosage/download");
}

#[test]
fn lookup_plan_rejects_invalid_gene_symbols() {
    for gene in ["", "BR AF", "BRAF/ALK"] {
        assert!(
            matches!(
                ClinGenClient::gene_lookup_plan(gene),
                Err(BioMcpError::InvalidArgument(_))
            ),
            "expected invalid argument for {gene:?}"
        );
    }
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
