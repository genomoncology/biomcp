use serde_json::json;

use super::*;
use crate::entities::section_outcome::SectionOutcomeState;
use crate::sources::fhir::test_server::{FixtureServer, Reply, condition, condition_bundle};

const ID: &str = "SYNTH-PT-7Q.42";

fn patient_json() -> serde_json::Value {
    json!({"resourceType": "Patient", "id": ID, "gender": "female", "birthDate": "1970-01-01"})
}

async fn read(route: impl Fn(&str) -> Reply + Send + Sync + 'static) -> (Patient, FixtureServer) {
    let server = FixtureServer::start(route).await;
    let client = FhirClient::new(&format!("{}/fhir", server.base)).expect("client");
    let id = PatientId::parse(ID).expect("id");
    let patient = get_with_client(&client, &id, true).await.expect("patient");
    (patient, server)
}

fn outcome(patient: &Patient) -> &SectionOutcome {
    patient
        .section_outcomes
        .get(PATIENT_SECTION_CONDITIONS)
        .expect("conditions outcome")
}

fn assert_message_names_nothing(patient: &Patient, server: &FixtureServer) {
    let message = outcome(patient).message().unwrap_or_default();
    assert!(!message.contains(ID), "{message}");
    assert!(!message.contains("://"), "{message}");
    assert!(
        !message.contains(server.base.trim_start_matches("http://")),
        "{message}"
    );
}

#[tokio::test]
async fn the_card_reads_demographics_and_both_condition_pages() {
    let (patient, _server) = read(|target| {
        if target.starts_with("/fhir/Patient/") {
            return Reply::json(200, patient_json());
        }
        if target.starts_with("/fhir/Condition?") {
            return Reply::json(
                200,
                condition_bundle(&[condition("c1", "Synthetic one")], Some("/fhir?page=2")),
            );
        }
        Reply::json(
            200,
            condition_bundle(&[condition("c2", "Synthetic two")], None),
        )
    })
    .await;
    assert_eq!(patient.id, ID);
    assert_eq!(patient.gender.as_deref(), Some("female"));
    assert_eq!(patient.birth_date.as_deref(), Some("1970-01-01"));
    let conditions = patient.conditions.as_ref().expect("conditions");
    assert_eq!(conditions.len(), 2);
    assert_eq!(conditions[0].text.as_deref(), Some("Synthetic one"));
    assert_eq!(conditions[0].clinical_status.as_deref(), Some("active"));
    assert_eq!(outcome(&patient).outcome(), SectionOutcomeState::Data);
}

#[tokio::test]
async fn a_condition_without_clinical_status_is_degraded() {
    let (patient, server) = read(|target| {
        if target.starts_with("/fhir/Patient/") {
            return Reply::json(200, patient_json());
        }
        let mut bare = condition("c1", "Synthetic");
        bare.as_object_mut()
            .expect("object")
            .remove("clinicalStatus");
        Reply::json(200, condition_bundle(&[bare], None))
    })
    .await;
    assert_eq!(outcome(&patient).outcome(), SectionOutcomeState::Degraded);
    assert_eq!(patient.conditions.as_ref().map(Vec::len), Some(1));
    assert_message_names_nothing(&patient, &server);
}

#[tokio::test]
async fn zero_entries_is_empty() {
    let (patient, _server) = read(|target| {
        if target.starts_with("/fhir/Patient/") {
            return Reply::json(200, patient_json());
        }
        Reply::json(200, condition_bundle(&[], None))
    })
    .await;
    assert_eq!(outcome(&patient).outcome(), SectionOutcomeState::Empty);
}

#[tokio::test]
async fn a_server_error_is_unavailable() {
    let (patient, server) = read(|target| {
        if target.starts_with("/fhir/Patient/") {
            return Reply::json(200, patient_json());
        }
        Reply::json(500, json!({"resourceType": "OperationOutcome"}))
    })
    .await;
    assert_eq!(
        outcome(&patient).outcome(),
        SectionOutcomeState::Unavailable
    );
    assert_eq!(patient.conditions.as_ref().map(Vec::len), Some(0));
    assert_message_names_nothing(&patient, &server);
}

#[tokio::test]
async fn every_early_stop_is_degraded_with_a_message_naming_no_url() {
    // A repeated next link, a next link to another origin, and a 21st page.
    for case in ["repeat", "origin", "cap"] {
        let pages = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (patient, server) = read(move |target| {
            if target.starts_with("/fhir/Patient/") {
                return Reply::json(200, patient_json());
            }
            let page = pages.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let next = match case {
                "repeat" => "/fhir?page=1".to_string(),
                "origin" => "http://other.invalid/fhir?page=2".to_string(),
                _ => format!("/fhir?page={}", page + 1),
            };
            Reply::json(
                200,
                condition_bundle(&[condition(&page.to_string(), "x")], Some(&next)),
            )
        })
        .await;
        assert_eq!(
            outcome(&patient).outcome(),
            SectionOutcomeState::Degraded,
            "case={case}"
        );
        assert_message_names_nothing(&patient, &server);
    }
}

#[tokio::test]
async fn a_redirect_to_another_origin_is_degraded() {
    let (patient, server) = read(|target| {
        if target.starts_with("/fhir/Patient/") {
            return Reply::json(200, patient_json());
        }
        Reply::redirect(format!("http://other.invalid{target}"))
    })
    .await;
    assert_eq!(outcome(&patient).outcome(), SectionOutcomeState::Degraded);
    assert_message_names_nothing(&patient, &server);
}

#[test]
fn unknown_sections_are_refused() {
    assert!(include_conditions(&["conditions".into()]).unwrap());
    assert!(include_conditions(&["all".into()]).unwrap());
    assert!(!include_conditions(&[]).unwrap());
    assert!(include_conditions(&["labs".into()]).is_err());
}
