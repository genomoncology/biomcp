//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to decoders
//! and response types. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clinicaltrials/",
            $name
        ))
    };
}

#[test]
fn parses_search_response_fixture() {
    let response: CtGovSearchResponse =
        ClinicalTrialsClient::decode_json_response(StatusCode::OK, fixture!("search.json"))
            .unwrap();

    assert_eq!(response.total_count, Some(1));
    assert_eq!(response.studies.len(), 1);
    let protocol = response.studies[0]
        .protocol_section
        .as_ref()
        .expect("protocol");
    assert_eq!(
        protocol
            .identification_module
            .as_ref()
            .and_then(|module| module.nct_id.as_deref()),
        Some("NCT41300001")
    );
}

#[test]
fn parses_contacts_and_eligibility_fixture() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT41300001",
        StatusCode::OK,
        fixture!("study_contacts.json"),
    )
    .unwrap();

    let protocol = study.protocol_section.expect("protocol");
    assert_eq!(
        protocol
            .eligibility_module
            .expect("eligibility")
            .sex
            .as_deref(),
        Some("FEMALE")
    );
    assert_eq!(
        protocol
            .contacts_locations_module
            .expect("contacts")
            .central_contacts[0]
            .email
            .as_deref(),
        Some("central@example.test")
    );
}

#[test]
fn parses_large_document_module() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT03361748",
        StatusCode::OK,
        br#"{"documentSection":{"largeDocumentModule":{"largeDocs":[{"typeAbbrev":"Prot_SAP","filename":"Prot_SAP_000.pdf","size":50,"hasProtocol":true,"hasSap":true,"hasIcf":false}]}}}"#,
    )
    .unwrap();

    let document = &study
        .document_section
        .expect("document section")
        .large_document_module
        .expect("large document module")
        .large_docs[0];
    assert_eq!(document.type_abbrev.as_deref(), Some("Prot_SAP"));
    assert_eq!(document.filename.as_deref(), Some("Prot_SAP_000.pdf"));
    assert_eq!(document.size, Some(50));
}

#[test]
fn get_response_maps_not_found_to_trial_not_found() {
    let err =
        ClinicalTrialsClient::decode_get_response("NCT404", StatusCode::NOT_FOUND, b"not found")
            .unwrap_err();

    match err {
        BioMcpError::NotFound { entity, id, .. } => {
            assert_eq!(entity, "trial");
            assert_eq!(id, "NCT404");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn decode_json_classifies_only_intervention_parser_bad_requests() {
    let signature = b"Error parsing query in Intervention / treatment: invalid expression";
    let err = ClinicalTrialsClient::decode_json_response::<CtGovSearchResponse>(
        StatusCode::BAD_REQUEST,
        signature,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        BioMcpError::CtGovInterventionQueryRejected { .. }
    ));

    for (status, body) in [
        (StatusCode::BAD_REQUEST, b"unrelated bad request".as_slice()),
        (StatusCode::INTERNAL_SERVER_ERROR, signature.as_slice()),
    ] {
        let err = ClinicalTrialsClient::decode_json_response::<CtGovSearchResponse>(status, body)
            .unwrap_err();
        assert_eq!(err.code(), "api");
    }
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = ClinicalTrialsClient::decode_json_response::<CtGovSearchResponse>(
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failure",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("ClinicalTrials.gov"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");
}
