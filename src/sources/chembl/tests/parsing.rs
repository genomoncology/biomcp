//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to ChEMBL
//! decoders and mappers. No network, no server.

use super::super::*;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/chembl/",
            $name
        ))
    };
}

#[test]
fn drug_targets_response_maps_targets_and_defaults() {
    let resp: ChemblMechanismResponse =
        ChemblClient::decode_json_response(StatusCode::OK, fixture!("mechanisms_chembl25.json"))
            .unwrap();
    let targets = ChemblClient::targets_from_response(resp);

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].target, "BRAF");
    assert_eq!(targets[0].action, "INHIBITOR");
    assert!(targets[0].mechanism.is_none());
    assert_eq!(targets[0].target_chembl_id.as_deref(), Some("CHEMBL1824"));
    assert_eq!(targets[1].target, "Unknown target");
    assert_eq!(targets[1].action, "Mechanism");
}

#[test]
fn target_summary_response_maps_pref_name_and_target_type() {
    let resp: ChemblTargetSummaryResponse =
        ChemblClient::decode_json_response(StatusCode::OK, fixture!("target_chembl3390820.json"))
            .unwrap();
    let summary = ChemblClient::summary_from_response(resp);

    assert_eq!(summary.pref_name, "PARP 1, 2 and 3");
    assert_eq!(summary.target_type, "PROTEIN FAMILY");
}

// Ticket 1214: the cell line record, the assay count, and the release.

#[test]
fn cell_line_response_maps_the_molm13_record() {
    let resp: ChemblCellLineResponse = ChemblClient::decode_json_response(
        StatusCode::OK,
        fixture!("cell_line_cvcl_2119_20260918.json"),
    )
    .unwrap();
    let records = ChemblClient::cell_lines_from_response(resp);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].chembl_id, "CHEMBL3706573");
    assert_eq!(records[0].name, "MOLM-13");
    assert_eq!(records[0].tax_id, Some(9606));
    assert_eq!(records[0].efo_id, None);
    assert_eq!(records[0].clo_id, None);
}

#[test]
fn cell_line_response_maps_the_k562_record_whose_name_chembl_spells_differently() {
    let resp: ChemblCellLineResponse = ChemblClient::decode_json_response(
        StatusCode::OK,
        fixture!("cell_line_cvcl_0004_20260918.json"),
    )
    .unwrap();
    let records = ChemblClient::cell_lines_from_response(resp);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].chembl_id, "CHEMBL3308378");
    assert_eq!(records[0].name, "K562");
    assert_eq!(records[0].efo_id.as_deref(), Some("EFO_0002067"));
    assert_eq!(records[0].clo_id.as_deref(), Some("CLO_0007059"));
}

#[test]
fn an_accession_chembl_does_not_list_maps_to_no_record() {
    let resp: ChemblCellLineResponse = ChemblClient::decode_json_response(
        StatusCode::OK,
        fixture!("cell_line_none_20260918.json"),
    )
    .unwrap();

    assert!(ChemblClient::cell_lines_from_response(resp).is_empty());
}

#[test]
fn assay_page_reports_the_total_count() {
    let resp: ChemblPageResponse = ChemblClient::decode_json_response(
        StatusCode::OK,
        fixture!("assay_count_chembl3706573_20260918.json"),
    )
    .unwrap();

    assert_eq!(ChemblClient::total_count_from_response(resp), 1408);
}

#[test]
fn status_reports_the_release_name_and_date() {
    let resp: ChemblStatusResponse =
        ChemblClient::decode_json_response(StatusCode::OK, fixture!("status_20260918.json"))
            .unwrap();
    let release = ChemblClient::release_from_response(resp).unwrap();

    assert_eq!(release.version, "ChEMBL_37");
    assert_eq!(release.released, "2026-05-01");
}

// Ticket 1214 acceptance 8: an HTML body with HTTP 200 is a provider error.

#[test]
fn an_html_body_served_with_http_200_names_the_url() {
    let url = "https://www.ebi.ac.uk/chembl/api/data/cell_line.json";
    let err = ChemblClient::decode_body::<ChemblCellLineResponse>(
        url,
        StatusCode::OK,
        b"<!DOCTYPE html><html><body>Verify you are human</body></html>",
    )
    .unwrap_err();
    let msg = format!("{err:?}");

    assert_eq!(err.code(), "api");
    assert!(msg.contains(url), "got: {msg}");
    assert!(
        msg.contains("HTML page where JSON was expected"),
        "got: {msg}"
    );
}

#[test]
fn decode_json_response_maps_http_and_json_errors() {
    let err = ChemblClient::decode_json_response::<ChemblMechanismResponse>(
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");

    let err =
        ChemblClient::decode_json_response::<ChemblMechanismResponse>(StatusCode::OK, b"not json")
            .unwrap_err();
    assert_eq!(err.code(), "api_json");
}
