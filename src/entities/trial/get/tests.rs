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
    let contacts = parse_sections(&["contacts".to_string()]).unwrap();
    assert!(contacts.include_contacts);
    assert!(!contacts.include_eligibility);

    let all = parse_sections(&["all".to_string()]).unwrap();
    assert!(all.include_contacts);
    assert!(all.include_eligibility);
    assert!(!all.include_eligibility_provenance);
    assert!(all.include_locations);
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
            without_citation,
            retained.clone(),
        ]))
        .expect("present references"),
        vec![retained]
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
fn nci_eligibility_uses_stable_criterion_order_and_heading_transitions() {
    let mut record = receipted_nci_record();
    record["eligibility"]["unstructured"] = serde_json::json!([
        {"description": "third\ninternal line", "display_order": 3, "inclusion_indicator": true},
        {"description": "equal first", "display_order": 2, "inclusion_indicator": false},
        {"description": "equal second", "display_order": 2, "inclusion_indicator": true},
        {"description": "first", "display_order": 1, "inclusion_indicator": true}
    ]);
    let (_, response) = plan_bound_nci_response(record, true);
    let ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present eligibility object");
    };

    assert_eq!(
        nci_eligibility_text(eligibility),
        Some(
            "Inclusion Criteria:\n- first\n\nExclusion Criteria:\n- equal first\n\nInclusion Criteria:\n- equal second\n- third\ninternal line"
                .to_string()
        )
    );
}

#[test]
fn nci_eligibility_truncates_the_composed_text_at_exactly_12000_characters() {
    let mut record = receipted_nci_record();
    record["eligibility"]["unstructured"] = serde_json::json!([
        {"description": "x".repeat(12_000), "display_order": 1, "inclusion_indicator": true}
    ]);
    let (_, response) = plan_bound_nci_response(record, true);
    let ClinicalTrialSection::Present(eligibility) = response.eligibility() else {
        panic!("present eligibility object");
    };
    let complete = format!("Inclusion Criteria:\n- {}", "x".repeat(12_000));
    let expected = format!(
        "{}\n\n(truncated, {} chars total)",
        complete.chars().take(12_000).collect::<String>(),
        complete.chars().count()
    );

    assert_eq!(nci_eligibility_text(eligibility), Some(expected));
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
    let eligibility = serde_json::to_value(trial.eligibility.as_ref().expect("typed eligibility"))
        .expect("eligibility JSON");
    assert_eq!(
        eligibility["minimum_age"],
        serde_json::json!({
            "number": 18.0, "unit": "years", "original": "18 Years"
        })
    );
    assert_eq!(
        eligibility["maximum_age"],
        serde_json::json!({
            "number": null, "unit": null, "original": "999 Years"
        })
    );
    let text = trial.eligibility_text.expect("NCI eligibility text");
    assert!(text.starts_with("Inclusion Criteria:\n- "));
    assert!(text.contains("Exclusion Criteria:\n- Definitive clinical or radiologic evidence"));
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
    assert!(overview.eligibility.is_none());
    assert!(overview.eligibility_text.is_none());
    assert_eq!(overview.age_range.as_deref(), Some("18 Years to Any age"));
    assert!(references.references.as_ref().is_some_and(Vec::is_empty));
    assert!(all.eligibility.is_some());
    assert!(all.eligibility_text.is_some());
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
