//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to decoders
//! and response types. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;

fn synthetic_detail(protocol_members: serde_json::Value) -> Vec<u8> {
    let mut protocol = serde_json::json!({
        "identificationModule": {
            "nctId": "NCT00000001",
            "briefTitle": "Synthetic validation study"
        },
        "statusModule": {"overallStatus": "RECRUITING"},
        "sponsorCollaboratorsModule": {"leadSponsor": {"name": "Example sponsor"}},
        "conditionsModule": {"conditions": ["Example condition"]},
        "designModule": {"studyType": "INTERVENTIONAL"}
    });
    protocol
        .as_object_mut()
        .unwrap()
        .extend(protocol_members.as_object().unwrap().clone());
    serde_json::to_vec(&serde_json::json!({"protocolSection": protocol})).unwrap()
}

#[test]
fn biodata_detail_response_redacts_field_distinct_site_values_but_getters_retain_them() {
    let body = serde_json::to_vec(&serde_json::json!({"protocolSection": {
        "identificationModule": {"nctId":"NCT60000015","briefTitle":"Wrapper trial"},
        "statusModule": {"overallStatus":"RECRUITING"},
        "sponsorCollaboratorsModule": {"leadSponsor":{"name":"Wrapper sponsor"}},
        "conditionsModule": {"conditions":["Wrapper condition"]},
        "designModule": {"studyType":"INTERVENTIONAL","phases":[]},
        "contactsLocationsModule": {
            "centralContacts": [{
                "name":"central-name-sentinel", "role":"central-role-sentinel",
                "phone":"central-phone-sentinel", "phoneExt":"central-extension-sentinel",
                "email":"central-email@example.test"
            }],
            "locations": [{
                "facility":"facility-sentinel", "status":"location-status-sentinel",
                "city":"city-sentinel", "state":"state-sentinel",
                "zip":"postal-sentinel", "country":"country-sentinel",
                "geoPoint":{"lat":12.5,"lon":-45.25},
                "contacts":[{"name":"site-name-sentinel","role":"site-role-sentinel",
                    "phone":"site-phone-sentinel","phoneExt":"site-extension-sentinel",
                    "email":"site-email@example.test"}]
            }]
        }
    }}))
    .unwrap();
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT60000015",
        &["contacts".into(), "locations".into()],
        StatusCode::OK,
        &body,
    )
    .unwrap();
    let debug = format!("{response:?}");
    let forbidden = [
        "central-name-sentinel",
        "central-role-sentinel",
        "central-phone-sentinel",
        "central-extension-sentinel",
        "central-email@example.test",
        "facility-sentinel",
        "location-status-sentinel",
        "city-sentinel",
        "state-sentinel",
        "postal-sentinel",
        "country-sentinel",
        "site-name-sentinel",
        "site-role-sentinel",
        "site-phone-sentinel",
        "site-extension-sentinel",
        "site-email@example.test",
    ];
    for sentinel in forbidden {
        assert!(!debug.contains(sentinel), "Debug leaked {sentinel}");
    }
    let biodata::ClinicalTrialSection::Present(directory) = response.site_directory() else {
        panic!("selected directory must be present");
    };
    assert_eq!(
        directory.central_contacts().unwrap()[0].name(),
        Some("central-name-sentinel")
    );
    let site = &directory.sites().unwrap()[0];
    assert_eq!(site.facility(), Some("facility-sentinel"));
    assert_eq!(
        site.contacts().unwrap()[0].email(),
        Some("site-email@example.test")
    );
    assert_eq!(site.coordinates().unwrap().latitude(), 12.5);
    assert_eq!(site.coordinates().unwrap().longitude(), -45.25);
}

#[test]
fn parses_search_response_fixture() {
    let response: CtGovSearchResponse = ClinicalTrialsClient::decode_json_response(
        StatusCode::OK,
        include_bytes!("../../../../testdata/sources/clinicaltrials/search.json"),
    )
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
fn legacy_detail_decode_ignores_contacts_without_consuming_eligibility() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT41300001",
        StatusCode::OK,
        include_bytes!("../../../../testdata/sources/clinicaltrials/study_contacts.json"),
    )
    .unwrap();

    let protocol = study.protocol_section.expect("protocol");
    assert!(protocol.eligibility_module.is_some());
    assert!(protocol.contacts_locations_module.is_some());
}

#[test]
fn legacy_search_fixture_keeps_only_retained_location_fields() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT00000000",
        StatusCode::OK,
        include_bytes!(
            "../../../../testdata/sources/clinicaltrials/case-13-location-contacts.json"
        ),
    )
    .unwrap();
    let module = study
        .protocol_section
        .and_then(|protocol| protocol.contacts_locations_module)
        .expect("location module");
    assert!(!module.locations.is_empty());
}

#[test]
fn legacy_source_aggregates_redact_ignored_contact_sentinels() {
    const SENTINEL: &str = "CONTACT-PRIVACY-SENTINEL-0115";
    let response: CtGovSearchResponse = serde_json::from_value(serde_json::json!({
        "studies": [{
            "protocolSection": {
                "contactsLocationsModule": {
                    "centralContacts": [{
                        "name": SENTINEL,
                        "phone": SENTINEL,
                        "email": SENTINEL
                    }],
                    "locations": [{
                        "facility": SENTINEL,
                        "contacts": [{"name": SENTINEL}]
                    }]
                }
            }
        }]
    }))
    .unwrap();
    let study = &response.studies[0];
    let protocol = study.protocol_section.as_ref().unwrap();
    let module = protocol.contacts_locations_module.as_ref().unwrap();
    let location = &module.locations[0];
    for diagnostic in [
        format!("{response:?}"),
        format!("{study:?}"),
        format!("{protocol:?}"),
        format!("{module:?}"),
        format!("{location:?}"),
    ] {
        assert!(!diagnostic.contains(SENTINEL));
    }
    let retained = serde_json::to_value(&response).unwrap();
    let contacts_module = &retained["studies"][0]["protocolSection"]["contactsLocationsModule"];
    assert!(contacts_module.get("centralContacts").is_none());
    assert!(contacts_module["locations"][0].get("contacts").is_none());
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

fn assert_detail_error(
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
                        format!("response validation failed: {expected_code}")
                    );
                }
                other => panic!("expected sanitized API error, got {other:?}"),
            }
        }
        other => panic!("expected source context, got {other:?}"),
    }
}

#[test]
fn detail_response_maps_every_stable_validation_code() {
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
        let error = ClinicalTrialsClient::decode_biodata_detail_response(
            "NCT00000001",
            &["references".to_string()],
            StatusCode::OK,
            bytes,
        )
        .expect_err(code);
        assert_detail_error(error, code, RecoveryAction::RetryRemoteSource);
    }

    let oversized = vec![b' '; 8 * 1024 * 1024 + 1];
    let error = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT00000001",
        &["references".to_string()],
        StatusCode::OK,
        &oversized,
    )
    .expect_err("resource limit");
    assert_detail_error(error, "json_resource_limit", RecoveryAction::NarrowRequest);
}

#[test]
fn detail_response_checks_http_status_before_valid_json() {
    let body = br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"}}}"#;
    for status in [StatusCode::BAD_GATEWAY, StatusCode::SERVICE_UNAVAILABLE] {
        let error =
            ClinicalTrialsClient::decode_biodata_detail_response("NCT00000001", &[], status, body)
                .expect_err("HTTP failure");
        assert_eq!(error.code(), "api");
        assert!(format!("{error:?}").contains(status.as_str()));
    }
}

#[test]
fn detail_response_returns_shared_references() {
    let body = synthetic_detail(serde_json::json!({
        "referencesModule": {"references": []}
    }));
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT00000001",
        &["references".to_string()],
        StatusCode::OK,
        &body,
    )
    .expect("valid BioData response");

    assert!(matches!(
        response.references(),
        biodata::ClinicalTrialSection::Present(references) if references.is_empty()
    ));
    assert_eq!(response.capture().provider_record_identity(), "NCT00000001");
}

#[test]
fn recorded_ctgov_eligibility_reaches_the_shared_projection_without_loss() {
    let bytes =
        include_bytes!("../../../../testdata/sources/ctgov/get_nct02576665_full_20260903.json");
    let source: serde_json::Value = serde_json::from_slice(bytes).expect("recorded response");
    let expected_text = source["protocolSection"]["eligibilityModule"]["eligibilityCriteria"]
        .as_str()
        .expect("recorded registry text");
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT02576665",
        &["eligibility".to_string()],
        StatusCode::OK,
        bytes,
    )
    .expect("valid recorded BioData response");
    let biodata::ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present shared eligibility")
    };
    assert_eq!(eligibility.registry_text(), Some(expected_text));
    let age = eligibility.age_range().expect("recorded age range");
    assert_eq!(age.minimum().unwrap().source().source(), "18 Years");
    assert_eq!(age.maximum().unwrap().source().source(), "75 Years");
    let sex = &eligibility.sexes().expect("recorded sex")[0];
    assert_eq!(sex.authority(), "clinicaltrials.gov");
    assert_eq!(sex.code(), "ALL");
    assert_eq!(eligibility.includes_healthy_subjects(), Some(false));
}

#[test]
fn ctgov_eligibility_mutations_change_only_the_shared_values() {
    let source =
        include_bytes!("../../../../testdata/sources/ctgov/get_nct02576665_full_20260903.json");
    let mut changed: serde_json::Value = serde_json::from_slice(source).unwrap();
    let module = &mut changed["protocolSection"]["eligibilityModule"];
    module["eligibilityCriteria"] = serde_json::json!("Mutated β criteria");
    module["sex"] = serde_json::json!("FEMALE");
    module["healthyVolunteers"] = serde_json::json!(true);
    let bytes = serde_json::to_vec(&changed).unwrap();
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT02576665",
        &["eligibility".to_string()],
        StatusCode::OK,
        &bytes,
    )
    .expect("valid mutated BioData response");
    let biodata::ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present shared eligibility")
    };
    assert_eq!(eligibility.registry_text(), Some("Mutated β criteria"));
    assert_eq!(eligibility.sexes().unwrap()[0].code(), "FEMALE");
    assert_eq!(eligibility.includes_healthy_subjects(), Some(true));
}

#[test]
fn detail_response_sanitizes_ambiguous_arm_labels() {
    let body = br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"},"armsInterventionsModule":{"armGroups":[{"label":"same"},{"label":"same"}],"interventions":[{"name":"I","armGroupLabels":["same"]}]}}}"#;
    let error = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT00000001",
        &["arms".to_string()],
        StatusCode::OK,
        body,
    )
    .expect_err("ambiguous label");
    assert_detail_error(
        error,
        "invalid_projection",
        crate::error::RecoveryAction::RetryRemoteSource,
    );
}

#[test]
fn product_arm_states_survive_dedicated_mixed_and_all_routes() {
    let states = [
        (synthetic_detail(serde_json::json!({})), None),
        (
            synthetic_detail(
                serde_json::json!({"armsInterventionsModule":{"armGroups":null,"interventions":null}}),
            ),
            None,
        ),
        (
            synthetic_detail(
                serde_json::json!({"armsInterventionsModule":{"armGroups":[],"interventions":[]}}),
            ),
            Some(0),
        ),
        (
            synthetic_detail(
                serde_json::json!({"armsInterventionsModule":{"armGroups":[{"label":"A"}],"interventions":[{"name":"I","armGroupLabels":["A"]}]}}),
            ),
            Some(1),
        ),
    ];
    for sections in [
        vec!["arms".to_string()],
        vec!["arms".to_string(), "outcomes".to_string()],
        vec!["all".to_string()],
    ] {
        for (body, expected_arms) in &states {
            let response = ClinicalTrialsClient::decode_biodata_detail_response(
                "NCT00000001",
                &sections,
                StatusCode::OK,
                body,
            )
            .unwrap_or_else(|error| panic!("valid arm state for {sections:?}: {error:?}"));
            let design =
                crate::entities::trial::product_design(response.interventions(), response.arms())
                    .expect("product arm state");
            assert_eq!(design.arms().map(<[_]>::len), *expected_arms);
            assert_eq!(design.assignments().map(<[_]>::len), *expected_arms);
        }
    }
}

#[test]
fn legacy_capture_keeps_missing_null_and_empty_arm_arrays_distinct() {
    let missing = ClinicalTrialsClient::decode_get_response(
        "NCT00000001",
        StatusCode::OK,
        br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"}}}"#,
    )
    .unwrap();
    assert!(
        missing
            .protocol_section
            .unwrap()
            .arms_interventions_module
            .is_none()
    );

    let decode_module = |body: &[u8]| {
        ClinicalTrialsClient::decode_get_response("NCT00000001", StatusCode::OK, body)
            .unwrap()
            .protocol_section
            .unwrap()
            .arms_interventions_module
            .unwrap()
    };
    let null = decode_module(br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"},"armsInterventionsModule":{"armGroups":null,"interventions":null}}}"#);
    assert!(null.arm_groups.is_none());
    assert!(null.interventions.is_none());

    let empty = decode_module(br#"{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"},"armsInterventionsModule":{"armGroups":[],"interventions":[]}}}"#);
    assert!(empty.arm_groups.is_some_and(|values| values.is_empty()));
    assert!(empty.interventions.is_some_and(|values| values.is_empty()));
}

#[test]
fn arm_label_mutations_rebuild_or_reject_assignments_without_leaking_values() {
    let before = synthetic_detail(
        serde_json::json!({"armsInterventionsModule":{"armGroups":[{"label":"Arm A"},{"label":"Arm B"}],"interventions":[{"name":"I","armGroupLabels":["Arm A"]},{"name":"J","armGroupLabels":["Arm B"]}]}}),
    );
    let after = synthetic_detail(
        serde_json::json!({"armsInterventionsModule":{"armGroups":[{"label":"Arm A"},{"label":"Arm B"}],"interventions":[{"name":"I","armGroupLabels":["Arm B"]},{"name":"J","armGroupLabels":["Arm B"]}]}}),
    );
    let parse_design = |body: &[u8]| {
        let response = ClinicalTrialsClient::decode_biodata_detail_response(
            "NCT00000001",
            &["arms".to_string()],
            StatusCode::OK,
            body,
        )
        .expect("forward label resolves");
        crate::entities::trial::product_design(response.interventions(), response.arms())
            .expect("forward relationship")
    };
    let before = parse_design(&before);
    let after = parse_design(&after);
    let entity_names = |design: &crate::entities::trial::TrialDesign| {
        (
            design
                .arms()
                .unwrap()
                .iter()
                .map(|value| value.name().to_string())
                .collect::<Vec<_>>(),
            design
                .interventions()
                .iter()
                .map(|value| value.name().to_string())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(entity_names(&before), entity_names(&after));
    let assigned_arm = |design: &crate::entities::trial::TrialDesign| {
        design
            .assignments()
            .unwrap()
            .iter()
            .find(|value| value.intervention_id().get() == 1)
            .unwrap()
            .arm_id()
            .get()
    };
    assert_eq!(assigned_arm(&before), 1);
    assert_eq!(assigned_arm(&after), 2);

    for invalid in [
        synthetic_detail(
            serde_json::json!({"armsInterventionsModule":{"armGroups":[{"label":"known-label-secret"}],"interventions":[{"name":"I","armGroupLabels":["unknown-label-secret"]}]}}),
        ),
        synthetic_detail(
            serde_json::json!({"armsInterventionsModule":{"armGroups":[{"label":"ambiguous-label-secret"},{"label":"ambiguous-label-secret"}],"interventions":[{"name":"I","armGroupLabels":["ambiguous-label-secret"]}]}}),
        ),
    ] {
        let error = ClinicalTrialsClient::decode_biodata_detail_response(
            "NCT00000001",
            &["arms".to_string()],
            StatusCode::OK,
            &invalid,
        )
        .expect_err("invalid label relationship");
        assert!(matches!(
            &error,
            BioMcpError::WithSourceContext { source, .. }
                if matches!(source.as_ref(), BioMcpError::Api { message, .. }
                    if message.ends_with("invalid_projection"))
        ));
        for rendered in [error.to_string(), format!("{error:?}")] {
            assert!(!rendered.contains("label-secret"));
            assert!(!rendered.contains("unknown-label"));
            assert!(!rendered.contains("ambiguous-label"));
        }
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
