//! Tests for trial detail helpers.

use super::*;
use crate::error::BioMcpError;
use axum::{Json, Router, routing::get as axum_get};

const RECEIPTED_NCI_CAPTURE: &str =
    include_str!("../../../../testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json");

fn receipted_nci_record() -> serde_json::Value {
    let response: serde_json::Value =
        serde_json::from_str(RECEIPTED_NCI_CAPTURE).expect("receipted NCI capture");
    response["data"][0].clone()
}

fn assert_nci_eligibility_error(value: serde_json::Value) {
    let error = nci_eligibility_text(&value).expect_err("malformed eligibility must fail");
    match error {
        BioMcpError::Api { api, message } => {
            assert_eq!(api, "nci_cts");
            assert_eq!(message, "NCI eligibility structure is invalid");
        }
        other => panic!("expected NCI API conversion error, got {other:?}"),
    }
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

#[test]
fn nci_eligibility_renders_receipted_criteria_in_provider_order() {
    let record = receipted_nci_record();
    let text = nci_eligibility_text(&record)
        .expect("valid eligibility")
        .expect("recorded eligibility text");
    let first_inclusion = "The patient or a legally authorized representative must provide study-specific informed consent";
    let first_exclusion = "Definitive clinical or radiologic evidence of metastatic disease";
    let last_exclusion = "Other conditions that, in the opinion of the investigator";

    assert!(text.starts_with("Inclusion Criteria:\n- "));
    assert_eq!(text.matches("Inclusion Criteria:").count(), 1);
    assert_eq!(text.matches("Exclusion Criteria:").count(), 1);
    assert_eq!(
        text.lines().filter(|line| line.starts_with("- ")).count(),
        36
    );
    let inclusion_position = text.find(first_inclusion).expect("first inclusion");
    let exclusion_heading_position = text.find("Exclusion Criteria:").expect("exclusion heading");
    let first_exclusion_position = text.find(first_exclusion).expect("first exclusion");
    let last_exclusion_position = text.find(last_exclusion).expect("last exclusion");
    assert!(inclusion_position < exclusion_heading_position);
    assert!(exclusion_heading_position < first_exclusion_position);
    assert!(first_exclusion_position < last_exclusion_position);
}

#[test]
fn nci_eligibility_distinguishes_checked_absence_from_malformed_presence() {
    for value in [
        serde_json::json!({}),
        serde_json::json!({"eligibility": null}),
        serde_json::json!({"eligibility": {"structured": {"min_age": "18 Years"}}}),
        serde_json::json!({"eligibility": {"unstructured": null}}),
        serde_json::json!({"eligibility": {"unstructured": []}}),
    ] {
        assert_eq!(nci_eligibility_text(&value).expect("checked absence"), None);
    }

    for value in [
        serde_json::json!({"eligibility": "published criteria"}),
        serde_json::json!({"eligibility": {"unstructured": {}}}),
        serde_json::json!({"eligibility": {"unstructured": ["criterion"]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "display_order": 1,
            "inclusion_indicator": true
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": 7,
            "display_order": 1,
            "inclusion_indicator": true
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": "",
            "display_order": 1,
            "inclusion_indicator": true
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": "secret criterion",
            "inclusion_indicator": true
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": "secret criterion",
            "display_order": 1
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": "secret criterion",
            "display_order": 1.5,
            "inclusion_indicator": true
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [{
            "description": "secret criterion",
            "display_order": 1,
            "inclusion_indicator": "yes"
        }]}}),
        serde_json::json!({"eligibility": {"unstructured": [
            {
                "description": "valid first criterion",
                "display_order": 1,
                "inclusion_indicator": true
            },
            {
                "description": "secret malformed criterion",
                "display_order": "second",
                "inclusion_indicator": false
            }
        ]}}),
    ] {
        assert_nci_eligibility_error(value);
    }
}

#[test]
fn nci_eligibility_sorting_is_stable_and_headings_follow_transitions() {
    let value = serde_json::json!({"eligibility": {"unstructured": [
        {"description": "third\ninternal line", "display_order": 3, "inclusion_indicator": true},
        {"description": "equal first", "display_order": 2, "inclusion_indicator": false},
        {"description": "equal second", "display_order": 2, "inclusion_indicator": true},
        {"description": "first", "display_order": 1, "inclusion_indicator": true}
    ]}});

    let text = nci_eligibility_text(&value)
        .expect("valid eligibility")
        .expect("eligibility text");
    assert_eq!(
        text,
        "Inclusion Criteria:\n- first\n\nExclusion Criteria:\n- equal first\n\nInclusion Criteria:\n- equal second\n- third\ninternal line"
    );
}

#[test]
fn nci_eligibility_truncates_only_after_composing_labeled_text() {
    let description = "x".repeat(ELIGIBILITY_MAX_CHARS);
    let value = serde_json::json!({"eligibility": {"unstructured": [{
        "description": description,
        "display_order": 1,
        "inclusion_indicator": true
    }]}});
    let complete = format!(
        "Inclusion Criteria:\n- {}",
        "x".repeat(ELIGIBILITY_MAX_CHARS)
    );

    let text = nci_eligibility_text(&value)
        .expect("valid eligibility")
        .expect("eligibility text");
    assert!(text.starts_with("Inclusion Criteria:\n- "));
    assert!(text.ends_with(&format!(
        "\n\n(truncated, {} chars total)",
        complete.chars().count()
    )));
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
        "/trials/{nct_id}",
        axum_get(move || {
            let record = record.clone();
            async move { Json(record) }
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

    let overview = get("NCT05879926", &[], TrialSource::NciCts)
        .await
        .expect("NCI overview");
    server.abort();
    assert!(overview.eligibility.is_none());
    assert_eq!(overview.age_range.as_deref(), Some("18 Years to Any age"));
}
