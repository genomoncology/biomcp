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
fn ctgov_age_wire_round_trips_only_provider_strings() {
    let module: CtGovEligibilityModule = serde_json::from_value(serde_json::json!({
        "minimumAge": " 6 Months ",
        "maximumAge": "N/A"
    }))
    .unwrap();
    let minimum = module.minimum_age.as_ref().unwrap();
    assert_eq!(minimum.original(), " 6 Months ");
    assert_eq!(minimum.parsed().unwrap().original(), "6 Months");
    assert_eq!(
        serde_json::to_value(&module).unwrap(),
        serde_json::json!({
            "eligibilityCriteria": null,
            "sex": null,
            "minimumAge": " 6 Months ",
            "maximumAge": "N/A"
        })
    );
    assert!(
        serde_json::from_value::<CtGovEligibilityModule>(serde_json::json!({
            "minimumAge": {"number": 6.0, "unit": "months", "original": "6 Months"}
        }))
        .is_err()
    );
}

#[test]
fn ctgov_age_wire_distinguishes_absent_null_and_blank() {
    for input in [
        serde_json::json!({}),
        serde_json::json!({"minimumAge": null}),
    ] {
        let module: CtGovEligibilityModule = serde_json::from_value(input).unwrap();
        assert!(module.minimum_age.is_none());
    }
    let module: CtGovEligibilityModule =
        serde_json::from_value(serde_json::json!({"minimumAge": " \t"})).unwrap();
    let wire = module.minimum_age.as_ref().unwrap();
    assert_eq!(wire.original(), " \t");
    assert!(wire.parsed().is_none());
    assert_eq!(serde_json::to_value(wire).unwrap(), " \t");
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

fn assert_biodata_error(
    error: BioMcpError,
    expected_code: &str,
    expected_recovery: crate::error::RecoveryAction,
) {
    match error {
        BioMcpError::WithSourceContext { context, source } => {
            assert_eq!(
                context.provider(),
                crate::error::SourceProvider::CLINICAL_TRIALS
            );
            assert_eq!(context.recovery(), expected_recovery);
            match *source {
                BioMcpError::Api { api, message } => {
                    assert_eq!(api, "ClinicalTrials.gov");
                    assert_eq!(
                        message,
                        format!("BioData response validation failed: {expected_code}")
                    );
                }
                other => panic!("expected sanitized API error, got {other:?}"),
            }
        }
        other => panic!("expected source context, got {other:?}"),
    }
}

#[test]
fn biodata_reference_response_maps_every_stable_validation_code() {
    use crate::error::RecoveryAction;

    for (bytes, code) in [
        (b"{".as_slice(), "malformed_json"),
        (b"[]".as_slice(), "unsupported_json"),
        (
            br#"{"protocolSection":{"identificationModule":{}}}"#.as_slice(),
            "invalid_projection",
        ),
        (
            br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000002"}}}"#.as_slice(),
            "identity_mismatch",
        ),
    ] {
        let error = ClinicalTrialsClient::decode_biodata_reference_response(
            "NCT00000001",
            StatusCode::OK,
            bytes,
        )
        .expect_err(code);
        assert_biodata_error(error, code, RecoveryAction::RetryRemoteSource);
    }

    let oversized = vec![b' '; 8 * 1024 * 1024 + 1];
    let error = ClinicalTrialsClient::decode_biodata_reference_response(
        "NCT00000001",
        StatusCode::OK,
        &oversized,
    )
    .expect_err("resource limit");
    assert_biodata_error(error, "json_resource_limit", RecoveryAction::NarrowRequest);
}

#[test]
fn biodata_reference_response_checks_http_status_before_valid_json() {
    let body = br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"}}}"#;
    for status in [StatusCode::BAD_GATEWAY, StatusCode::SERVICE_UNAVAILABLE] {
        let error =
            ClinicalTrialsClient::decode_biodata_reference_response("NCT00000001", status, body)
                .expect_err("HTTP failure");
        assert_eq!(error.code(), "api");
        assert!(format!("{error:?}").contains(status.as_str()));
    }
}

#[test]
fn biodata_reference_response_returns_one_reference_owner() {
    let body = br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"},"referencesModule":{"references":[]}}}"#;
    let response = ClinicalTrialsClient::decode_biodata_reference_response(
        "NCT00000001",
        StatusCode::OK,
        body,
    )
    .expect("valid BioData response");

    assert!(matches!(
        response.references,
        biodata::ClinicalTrialSection::Present(ref references) if references.is_empty()
    ));
    assert!(
        response
            .study
            .protocol_section
            .as_ref()
            .and_then(|protocol| protocol.references_module.as_ref())
            .is_none()
    );
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
