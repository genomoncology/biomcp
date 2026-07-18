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
