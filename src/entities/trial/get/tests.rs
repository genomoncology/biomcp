//! Tests for trial detail helpers.

use super::*;
use crate::error::BioMcpError;
use axum::{Json, Router, routing::get as axum_get};

const RECEIPTED_NCI_CAPTURE: &str =
    include_str!("../../../../testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json");

fn receipted_nci_record() -> serde_json::Value {
    let response: serde_json::Value =
        serde_json::from_str(RECEIPTED_NCI_CAPTURE).expect("receipted NCI capture");
    let recorded = response["data"][0]
        .as_object()
        .expect("receipted NCI record");
    let mut selected = serde_json::Map::new();
    for field in [
        "nci_id",
        "nct_id",
        "brief_title",
        "official_title",
        "current_trial_status",
        "why_study_stopped",
        "study_protocol_type",
        "phase",
        "diseases",
        "minimum_target_accrual_number",
        "arms",
        "lead_org",
        "start_date",
        "completion_date",
        "eligibility",
        "brief_summary",
    ] {
        if let Some(value) = recorded.get(field) {
            selected.insert(field.to_string(), value.clone());
        }
    }
    serde_json::Value::Object(selected)
}

fn plan_bound_nci_response(
    record: serde_json::Value,
    eligibility_requested: bool,
) -> (NciCtsV2DetailPlan, NciCtsV2DetailResponse) {
    let plan = NciCtsV2DetailPlan::new("NCT05879926", eligibility_requested).unwrap();
    let bytes = serde_json::to_vec(&serde_json::json!({"total": 1, "data": [record]})).unwrap();
    let response =
        NciCtsV2DetailResponse::parse(&plan, &bytes, &biodata::NciCtsV2Limits::default()).unwrap();
    (plan, response)
}

#[test]
fn normalize_nct_id_uppercases_prefix() {
    assert_eq!(normalize_nct_id("nct06162221"), "NCT06162221");
    assert_eq!(normalize_nct_id("NCT06162221"), "NCT06162221");
}

#[test]
fn parse_sections_accepts_contacts_and_all_includes_contacts() {
    for section in ["arms", "contacts", "locations", "outcomes", "references"] {
        let parsed = parse_sections(&[section.to_string()]).unwrap();
        assert!(!parsed.request_eligibility, "{section}");
    }

    for sections in [
        vec![],
        vec!["--json".to_string()],
        vec!["eligibility".to_string()],
    ] {
        assert!(parse_sections(&sections).unwrap().request_eligibility);
    }

    let all = parse_sections(&["all".to_string()]).unwrap();
    assert!(all.include_contacts);
    assert!(all.request_eligibility);
    assert!(!all.include_eligibility_provenance);
    assert!(all.include_locations);
}

#[test]
fn nci_request_state_table_requests_eligibility_exactly_when_selected() {
    for (sections, expected) in [
        (vec![], 1),
        (vec!["eligibility"], 1),
        (vec!["all"], 1),
        (vec!["arms"], 0),
        (vec!["contacts"], 0),
        (vec!["locations"], 0),
        (vec!["outcomes"], 0),
        (vec!["references"], 0),
    ] {
        let sections = sections.into_iter().map(str::to_owned).collect::<Vec<_>>();
        let flags = parse_sections(&sections).expect("valid ordinary sections");
        let plan = NciCtsV2DetailPlan::new("NCT05879926", flags.request_eligibility).unwrap();
        assert_eq!(
            plan.query_pairs()
                .iter()
                .filter(|(name, value)| *name == "include" && *value == "eligibility")
                .count(),
            expected,
            "sections {sections:?}"
        );
    }
}

#[test]
fn product_references_maps_each_section_state() {
    use biodata::{ClinicalTrialReference, ClinicalTrialSection};

    assert!(
        product_references(ClinicalTrialSection::Absent)
            .expect("absent references")
            .is_empty()
    );
    assert!(
        product_references(ClinicalTrialSection::Present(Vec::new()))
            .expect("present empty references")
            .is_empty()
    );
    assert!(matches!(
        product_references(ClinicalTrialSection::NotRequested),
        Err(BioMcpError::InternalProcessing)
    ));
    assert!(matches!(
        product_references(ClinicalTrialSection::Unavailable),
        Err(BioMcpError::InternalProcessing)
    ));

    let without_citation = ClinicalTrialReference::new(Some("123".to_string()), None, None)
        .expect("source-stated reference");
    let retained =
        ClinicalTrialReference::new(Some("456".to_string()), Some("Citation".to_string()), None)
            .expect("source-stated reference");
    assert_eq!(
        product_references(ClinicalTrialSection::Present(vec![
            without_citation.clone(),
            retained.clone(),
        ]))
        .expect("present references"),
        vec![without_citation, retained]
    );
}

#[test]
fn nci_product_conversion_checks_enrollment_and_preserves_source_presence() {
    let mut record = receipted_nci_record();
    record["minimum_target_accrual_number"] = serde_json::json!(2_147_483_648_u64);
    record["why_study_stopped"] = serde_json::json!("  Enrollment target was not met  ");
    record["brief_summary"] = serde_json::json!("  Source summary.  ");
    let (plan, response) = plan_bound_nci_response(record, true);

    let trial = product_from_nci_response(&plan, &response, true, false).unwrap();
    assert_eq!(trial.enrollment, None);
    assert_eq!(
        trial.why_stopped,
        Some(Some("Enrollment target was not met".to_string()))
    );
    assert_eq!(trial.summary.as_deref(), Some("Source summary."));
}

#[test]
fn nci_arm_conversion_preserves_every_occurrence_and_assignment() {
    let (plan, response) = plan_bound_nci_response(receipted_nci_record(), true);
    let trial = product_from_nci_response(&plan, &response, true, true).unwrap();
    assert_eq!(trial.design.arms().map(<[_]>::len), Some(2));
    assert_eq!(trial.design.interventions().len(), 53);
    assert_eq!(trial.design.assignments().map(<[_]>::len), Some(53));
    let mut names = std::collections::HashSet::new();
    assert!(
        trial
            .design
            .interventions()
            .iter()
            .any(|value| !names.insert(value.name()))
    );
    let encoded = serde_json::to_value(&trial.design).unwrap();
    assert_eq!(
        encoded["interventions"][0]["type"],
        serde_json::json!({
            "authority": "nci", "code": "Other", "display": null,
            "vocabulary_version": null, "recognized_meaning": null
        })
    );
    assert_eq!(
        encoded["arms"][0]["type"],
        serde_json::json!({
            "authority": "nci", "code": "EXPERIMENTAL", "display": null,
            "vocabulary_version": null, "recognized_meaning": null
        })
    );
    assert!(
        encoded["interventions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|value| { value["type"].is_null() || value["type"]["authority"] == "nci" })
    );
    let decoded: crate::entities::trial::TrialDesign =
        serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), encoded);
}

#[test]
fn product_design_retains_a_relationship_failure_from_mismatched_sections() {
    let arm_id = biodata::ClinicalTrialArmId::new(1).unwrap();
    let expected_intervention_id = biodata::ClinicalTrialInterventionId::new(1).unwrap();
    let original_intervention = biodata::ClinicalTrialIntervention::new(
        expected_intervention_id,
        "original",
        None,
        None,
        None,
    )
    .unwrap();
    let arm = biodata::ClinicalTrialArm::new(arm_id, "arm", None, None).unwrap();
    let assignment =
        biodata::ClinicalTrialArmInterventionAssignment::new(arm_id, expected_intervention_id);
    let arms =
        biodata::ClinicalTrialArms::new(vec![arm], &[original_intervention], vec![assignment])
            .unwrap();
    let replacement_intervention = biodata::ClinicalTrialIntervention::new(
        biodata::ClinicalTrialInterventionId::new(2).unwrap(),
        "replacement",
        None,
        None,
        None,
    )
    .unwrap();

    let error = product_design(
        &ClinicalTrialSection::Present(vec![replacement_intervention]),
        &ClinicalTrialSection::Present(arms),
    )
    .expect_err("mismatched product sections");
    let BioMcpError::TrialDesign(crate::error::TrialDesignError::InvalidRelationship(relationship)) =
        error
    else {
        panic!("expected typed trial design failure")
    };
    assert_eq!(
        relationship,
        biodata::ClinicalTrialArmRelationshipError::MissingInterventionEndpoint {
            intervention_id: expected_intervention_id
        }
    );
}

#[test]
fn nci_eligibility_keeps_absence_and_an_explicit_empty_list_distinct() {
    let mut absent = receipted_nci_record();
    absent.as_object_mut().unwrap().remove("eligibility");
    let (_, absent) = plan_bound_nci_response(absent, true);
    assert!(matches!(absent.eligibility(), ClinicalTrialSection::Absent));

    let mut null = receipted_nci_record();
    null["eligibility"] = serde_json::Value::Null;
    let (_, null) = plan_bound_nci_response(null, true);
    assert!(matches!(null.eligibility(), ClinicalTrialSection::Absent));

    let mut missing_criteria = receipted_nci_record();
    missing_criteria["eligibility"]
        .as_object_mut()
        .unwrap()
        .remove("unstructured");
    let (_, missing_criteria) = plan_bound_nci_response(missing_criteria, true);
    let ClinicalTrialSection::Present(eligibility) = missing_criteria.eligibility() else {
        panic!("present eligibility object");
    };
    assert!(eligibility.criteria().is_none());

    let mut empty = receipted_nci_record();
    empty["eligibility"]["unstructured"] = serde_json::json!([]);
    let (_, empty) = plan_bound_nci_response(empty, true);
    let ClinicalTrialSection::Present(eligibility) = empty.eligibility() else {
        panic!("present eligibility object");
    };
    assert!(eligibility.criteria().is_some_and(<[_]>::is_empty));

    let record = receipted_nci_record();
    let (unrequested_plan, unrequested) = plan_bound_nci_response(record, false);
    assert!(matches!(
        product_from_nci_response(&unrequested_plan, &unrequested, true, false),
        Err(BioMcpError::InternalProcessing)
    ));
}

#[test]
fn nci_criterion_sorting_preserves_source_occurrence_identity_and_classification() {
    let original = receipted_nci_record();
    let rows = original["eligibility"]["unstructured"]
        .as_array()
        .expect("recorded criteria");
    let mut expected = rows.iter().enumerate().collect::<Vec<_>>();
    expected.sort_by_key(|(_, row)| row["display_order"].as_u64().unwrap());
    let (_, response) = plan_bound_nci_response(original.clone(), true);
    let ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present eligibility")
    };
    let criteria = eligibility.criteria().expect("criteria");
    assert_eq!(criteria.len(), expected.len());
    for (sorted_index, (source_index, row)) in expected.into_iter().enumerate() {
        let criterion = &criteria[sorted_index];
        assert_eq!(
            criterion.description(),
            row["description"].as_str().unwrap(),
            "description at sorted position {sorted_index}"
        );
        assert_eq!(
            criterion.id().get(),
            source_index as u64 + 1,
            "occurrence identity at sorted position {sorted_index}"
        );
        assert_eq!(
            row["display_order"].as_u64(),
            Some(sorted_index as u64 + 1),
            "recorded display order at sorted position {sorted_index}"
        );
        let expected_inclusion = row["inclusion_indicator"].as_bool().unwrap();
        assert_eq!(
            matches!(
                criterion.classification(),
                biodata::ClinicalTrialEligibilityClassification::Inclusion
            ),
            expected_inclusion,
            "classification at sorted position {sorted_index}"
        );
        assert_eq!(
            matches!(
                criterion.classification(),
                biodata::ClinicalTrialEligibilityClassification::Exclusion
            ),
            !expected_inclusion,
            "classification at sorted position {sorted_index}"
        );
    }
    let expected_first = original["eligibility"]["unstructured"][0]["description"]
        .as_str()
        .unwrap()
        .to_string();
    let mut reversed = original;
    reversed["eligibility"]["unstructured"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let (_, response) = plan_bound_nci_response(reversed, true);
    let ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present eligibility")
    };
    let criteria = eligibility.criteria().expect("criteria");
    assert_eq!(criteria[0].description(), expected_first);
    assert_eq!(criteria[0].id().get(), 36);
    assert!(matches!(
        criteria[0].classification(),
        biodata::ClinicalTrialEligibilityClassification::Inclusion
    ));

    let mut changed = receipted_nci_record();
    changed["eligibility"]["unstructured"][0]["inclusion_indicator"] = serde_json::json!(false);
    let (_, response) = plan_bound_nci_response(changed, true);
    let ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present eligibility")
    };
    assert!(matches!(
        eligibility.criteria().unwrap()[0].classification(),
        biodata::ClinicalTrialEligibilityClassification::Exclusion
    ));
}

#[tokio::test]
async fn get_rejects_non_nct_id_with_format_hint() {
    let err = get("WRONG", &[], TrialSource::ClinicalTrialsGov)
        .await
        .expect_err("invalid trial id should fail before API call");

    match err {
        BioMcpError::InvalidArgument(message) => {
            assert!(message.contains("Expected an NCT ID like NCT02576665"));
            assert!(message.contains("got 'WRONG'"));
        }
        other => panic!("expected InvalidArgument, got: {other}"),
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ctgov_eligibility_provenance_exists_only_for_explicit_text() {
    use crate::entities::trial::test_support::{CtGovFixtureEnv, ctgov_json_fixture};

    let recorded =
        include_str!("../../../../testdata/sources/ctgov/get_nct02576665_full_20260903.json");
    let (base, _, server) = ctgov_json_fixture(recorded).await;
    let _env = CtGovFixtureEnv::set(&base);
    let explicit = get(
        "NCT02576665",
        &["eligibility".to_string()],
        TrialSource::ClinicalTrialsGov,
    )
    .await
    .expect("explicit eligibility");
    let default = get("NCT02576665", &[], TrialSource::ClinicalTrialsGov)
        .await
        .expect("default detail");
    let all = get(
        "NCT02576665",
        &["all".to_string()],
        TrialSource::ClinicalTrialsGov,
    )
    .await
    .expect("all detail");
    server.abort();
    assert!(explicit.eligibility_provenance.is_some());
    assert!(default.eligibility.is_some());
    assert!(default.eligibility_provenance.is_none());
    assert!(all.eligibility.is_some());
    assert!(all.eligibility_provenance.is_none());

    for (label, mutation) in [
        ("without text", Some(serde_json::Value::Null)),
        ("without eligibility", None),
    ] {
        let mut body: serde_json::Value = serde_json::from_str(recorded).unwrap();
        if let Some(text) = mutation {
            body["protocolSection"]["eligibilityModule"]["eligibilityCriteria"] = text;
        } else {
            body["protocolSection"]
                .as_object_mut()
                .unwrap()
                .remove("eligibilityModule");
        }
        let (base, _, server) = ctgov_json_fixture(serde_json::to_string(&body).unwrap()).await;
        let _env = CtGovFixtureEnv::set(&base);
        let trial = get(
            "NCT02576665",
            &["eligibility".to_string()],
            TrialSource::ClinicalTrialsGov,
        )
        .await
        .unwrap_or_else(|error| panic!("{label}: {error}"));
        server.abort();
        assert!(trial.eligibility_provenance.is_none(), "{label}");
    }
}

struct NciFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl NciFixtureEnv {
    fn set(&mut self, name: &'static str, value: &str) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe { std::env::set_var(name, value) };
    }
}

impl Drop for NciFixtureEnv {
    fn drop(&mut self) {
        for (name, prior) in self.0.drain(..).rev() {
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe {
                if let Some(value) = prior {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_get_eligibility_uses_receipted_trial_record_shape() {
    let record = receipted_nci_record();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind NCI detail fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let router = Router::new().route(
        "/trials",
        axum_get(move || {
            let record = record.clone();
            async move { Json(serde_json::json!({"total": 1, "data": [record]})) }
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve NCI detail fixture");
    });
    let mut env = NciFixtureEnv(Vec::new());
    env.set("NCI_API_KEY", "fixture-key");
    env.set("BIOMCP_NCI_CTS_BASE", &base);

    let trial = get(
        "NCT05879926",
        &["eligibility".to_string()],
        TrialSource::NciCts,
    )
    .await
    .expect("NCI trial detail");
    let eligibility = serde_json::to_value(&trial).expect("trial JSON")["eligibility"].clone();
    assert_eq!(
        eligibility["age_range"]["minimum"]["source"],
        serde_json::json!("18 Years")
    );
    assert_eq!(
        eligibility["age_range"]["maximum"]["kind"],
        serde_json::json!("source_stated_no_limit")
    );
    assert_eq!(eligibility["criteria"].as_array().unwrap().len(), 36);
    assert_eq!(eligibility["sexes"][0]["authority"], "nci");
    assert_eq!(eligibility["sexes"][0]["code"], "FEMALE");
    assert_eq!(eligibility["includes_healthy_subjects"], false);
    assert_eq!(eligibility["criteria"][0]["id"], 1);
    assert_eq!(
        eligibility["criteria"][0]["classification"]["kind"],
        "inclusion"
    );
    assert_eq!(trial.source.as_deref(), Some("NCI CTS"));
    assert_eq!(trial.status, "Active");
    assert_eq!(trial.phase.as_deref(), Some("III"));
    assert_eq!(trial.study_type.as_deref(), Some("Interventional"));
    assert_eq!(trial.sponsor.as_deref(), Some("NRG Oncology"));
    assert_eq!(trial.enrollment, Some(3960));
    assert_eq!(trial.start_date.as_deref(), Some("2023-10-18"));
    assert_eq!(trial.completion_date.as_deref(), Some("2030-02-28"));
    assert_eq!(trial.why_stopped, Some(None));
    assert_eq!(trial.design.interventions().len(), 53);
    assert!(
        trial
            .summary
            .as_deref()
            .is_some_and(|value| !value.is_empty())
    );
    assert!(!trial.conditions.is_empty());
    assert!(trial.contacts.is_none());
    assert!(trial.locations.is_none());
    assert!(trial.outcomes.is_none());
    assert!(trial.design.arms().is_none());

    let overview = get("NCT05879926", &[], TrialSource::NciCts)
        .await
        .expect("NCI overview");
    let references = get(
        "NCT05879926",
        &["references".to_string()],
        TrialSource::NciCts,
    )
    .await
    .expect("NCI references");
    let all = get("NCT05879926", &["all".to_string()], TrialSource::NciCts)
        .await
        .expect("NCI all sections");
    server.abort();
    assert!(overview.eligibility.is_some());
    assert!(overview.eligibility_provenance.is_none());
    assert!(references.references.as_ref().is_some_and(Vec::is_empty));
    assert!(all.eligibility.is_some());
    assert!(all.eligibility_provenance.is_none());
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_not_found_status_wins_before_an_oversized_body_is_read() {
    const OVERSIZED_BODY_LEN: usize = 8 * 1024 * 1024 + 1;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind NCI detail fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let router = Router::new().route(
        "/trials",
        axum_get(|| async {
            (
                axum::http::StatusCode::NOT_FOUND,
                vec![b'x'; OVERSIZED_BODY_LEN],
            )
        }),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve NCI detail fixture");
    });
    let mut env = NciFixtureEnv(Vec::new());
    env.set("NCI_API_KEY", "fixture-key");
    env.set("BIOMCP_NCI_CTS_BASE", &base);

    let error = get("NCT05879926", &[], TrialSource::NciCts)
        .await
        .expect_err("NCI not found");
    server.abort();

    match error {
        BioMcpError::NotFound {
            entity,
            id,
            suggestion,
        } => {
            assert_eq!(entity, "trial");
            assert_eq!(id, "NCT05879926");
            assert_eq!(
                suggestion,
                "Try searching: biomcp search trial -c \"NCT05879926\""
            );
        }
        other => panic!("expected NotFound, got: {other:?}"),
    }
}
