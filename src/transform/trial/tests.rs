use super::*;
use serde_json::json;
#[path = "tests/ticket_1107.rs"]
mod ticket_1107;
#[path = "tests/ticket_1132.rs"]
mod ticket_1132;
use ticket_1107::{IntoTrialSearchTestResult, IntoTrialTestResult};
#[test]
fn truncate_summary_two_sentences_and_length() {
    let s = "Sentence one. Sentence two. Sentence three.";
    let out = truncate_summary(s);
    assert_eq!(out, "Sentence one. Sentence two.");

    let long = "€".repeat(400);
    let out2 = truncate_summary(&long);
    assert!(out2.ends_with("..."));
    assert!(out2.len() <= 503);
}

#[test]
fn format_age_range_handles_missing_bounds() {
    assert_eq!(
        format_age_range(Some("18 Years"), Some("65 Years")).as_deref(),
        Some("18 Years to 65 Years")
    );
    assert_eq!(
        format_age_range(Some("18 Years"), None).as_deref(),
        Some("18 Years to Any age")
    );
    assert_eq!(
        format_age_range(None, Some("65 Years")).as_deref(),
        Some("Any age to 65 Years")
    );
    assert_eq!(format_age_range(None, None), None);
}

#[test]
fn from_ctgov_study_extracts_age_and_locations_sorted() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT01234567", "briefTitle": "Test Trial"},
            "statusModule": {"overallStatus": "RECRUITING"},
            "designModule": {"phases": ["PHASE2"]},
            "eligibilityModule": {"minimumAge": "18 Years", "maximumAge": "75 Years"},
            "contactsLocationsModule": {
                "locations": [
                    {
                        "facility": "Site B",
                        "city": "Boston",
                        "country": "USA",
                        "status": "COMPLETED",
                        "contacts": [{"name": "Late Contact", "phone": "333"}]
                    },
                    {
                        "facility": "Site A",
                        "city": "New York",
                        "country": "USA",
                        "status": "RECRUITING",
                        "contacts": [{"name": "Lead Contact", "phone": "111"}]
                    }
                ]
            }
        }
    }))
    .unwrap();

    let trial = from_ctgov_study(&study);
    assert_eq!(trial.age_range.as_deref(), Some("18 Years to 75 Years"));
    let locations = trial.locations.expect("locations");
    assert_eq!(locations.len(), 2);
    assert_eq!(locations[0].facility, "Site A");
    assert_eq!(locations[0].contact_name.as_deref(), Some("Lead Contact"));
}

#[test]
fn from_ctgov_study_preserves_contacts_and_structured_eligibility() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT41300001", "briefTitle": "Contact Trial"},
            "eligibilityModule": {
                "minimumAge": "2 Years",
                "maximumAge": "18 Years",
                "sex": "FEMALE"
            },
            "contactsLocationsModule": {
                "centralContacts": [{
                    "name": "Central Coordinator",
                    "role": "CONTACT",
                    "phone": "555-0100",
                    "email": "central@example.test"
                }],
                "locations": [{
                    "facility": "Rare Disease Center",
                    "city": "Ann Arbor",
                    "state": "Michigan",
                    "country": "United States",
                    "contacts": [{
                        "name": "Site Coordinator",
                        "role": "CONTACT",
                        "phone": "555-0199",
                        "email": "site@example.test"
                    }]
                }]
            }
        }
    }))
    .unwrap();

    let trial = from_ctgov_study(&study);
    let eligibility = trial.eligibility.expect("eligibility");
    assert_eq!(eligibility.sex.as_deref(), Some("Female"));
    assert_eq!(eligibility.minimum_age.as_deref(), Some("2 Years"));
    assert_eq!(eligibility.maximum_age.as_deref(), Some("18 Years"));

    let contacts = trial.contacts.expect("contacts");
    assert_eq!(contacts[0].level, "central");
    assert_eq!(contacts[0].email.as_deref(), Some("central@example.test"));
    assert_eq!(contacts[1].level, "site");
    assert_eq!(contacts[1].facility.as_deref(), Some("Rare Disease Center"));
    assert_eq!(contacts[1].email.as_deref(), Some("site@example.test"));

    let locations = trial.locations.expect("locations");
    assert_eq!(locations[0].contact_role.as_deref(), Some("CONTACT"));
    assert_eq!(
        locations[0].contact_email.as_deref(),
        Some("site@example.test")
    );
}

#[test]
fn from_ctgov_study_preserves_provider_type_fields_in_json() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {"nctId": "NCT09876543", "briefTitle": "Arms Trial"},
            "statusModule": {"overallStatus": "ACTIVE"},
            "armsInterventionsModule": {
                "interventions": [
                    {
                        "name": "Pembrolizumab",
                        "type": "BIOLOGICAL",
                        "armGroupLabels": ["Experimental Arm"]
                    }
                ],
                "armGroups": [
                    {
                        "label": "Experimental Arm",
                        "type": "EXPERIMENTAL",
                        "description": "Experimental group",
                        "interventionNames": []
                    }
                ]
            },
            "referencesModule": {
                "references": [{
                    "pmid": "12345678",
                    "type": "BACKGROUND",
                    "citation": "Prior evidence"
                }]
            },
            "outcomesModule": {
                "primaryOutcomes": [{"measure": "Overall survival"}],
                "secondaryOutcomes": [{"measure": "Progression-free survival"}]
            }
        }
    }))
    .unwrap();

    let trial = from_ctgov_study(&study);
    let arms = trial.arms.as_ref().expect("arms");
    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0].label, "Experimental Arm");
    assert_eq!(arms[0].interventions, vec!["Pembrolizumab"]);
    let outcomes = trial.outcomes.as_ref().expect("outcomes");
    assert_eq!(outcomes.primary.len(), 1);
    assert_eq!(outcomes.primary[0].measure, "Overall survival");
    assert_eq!(outcomes.secondary.len(), 1);
    assert_eq!(outcomes.secondary[0].measure, "Progression-free survival");

    let json = serde_json::to_value(trial).expect("trial JSON");
    assert_eq!(
        [
            &json["intervention_details"][0]["intervention_type"],
            &json["arms"][0]["arm_type"],
            &json["references"][0]["reference_type"],
        ],
        [
            &json!("BIOLOGICAL"),
            &json!("EXPERIMENTAL"),
            &json!("BACKGROUND"),
        ]
    );
}

#[test]
fn stopped_ctgov_trials_explain_their_status_in_json_and_markdown() {
    for status in ["TERMINATED", "WITHDRAWN", "SUSPENDED"] {
        let reason = format!("Registry reason for {status}");
        let study: CtGovStudy = serde_json::from_value(json!({
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT03515785",
                    "briefTitle": "Stopped trial"
                },
                "statusModule": {
                    "overallStatus": status,
                    "whyStopped": reason
                }
            }
        }))
        .expect("stopped study");

        let trial = from_ctgov_study(&study);
        let json = serde_json::to_value(&trial).expect("trial JSON");
        assert_eq!(json["why_stopped"], reason);

        let markdown =
            crate::render::markdown::trial_markdown(&trial, &[]).expect("trial markdown");
        assert!(markdown.contains(&format!("Status: {status} | Why stopped: {reason}")));
    }
}

#[test]
fn stopped_ctgov_trial_without_reason_reports_checked_absence() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": "NCT03515785",
                "briefTitle": "Stopped trial without a reason"
            },
            "statusModule": {"overallStatus": "WITHDRAWN"}
        }
    }))
    .expect("stopped study without reason");
    let trial = from_ctgov_study(&study);
    let json = serde_json::to_value(&trial).expect("trial JSON");
    assert!(
        json.as_object()
            .is_some_and(|object| object.contains_key("why_stopped")),
        "stopped trial JSON must distinguish a missing registry reason from an unrequested field"
    );
    assert!(json["why_stopped"].is_null());

    let markdown = crate::render::markdown::trial_markdown(&trial, &[]).expect("trial markdown");
    assert!(
        markdown.contains("Status: WITHDRAWN | Why stopped: Not provided by ClinicalTrials.gov")
    );

    let ordinary_study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": "NCT03515786",
                "briefTitle": "Recruiting trial"
            },
            "statusModule": {"overallStatus": "RECRUITING"}
        }
    }))
    .expect("ordinary study");
    let ordinary_trial = from_ctgov_study(&ordinary_study);
    let ordinary_json = serde_json::to_value(&ordinary_trial).expect("ordinary trial JSON");
    assert!(
        !ordinary_json
            .as_object()
            .expect("ordinary trial object")
            .contains_key("why_stopped")
    );
    let ordinary_markdown = crate::render::markdown::trial_markdown(&ordinary_trial, &[])
        .expect("ordinary trial Markdown");
    assert!(!ordinary_markdown.contains("Why stopped:"));
}

#[test]
fn from_nci_trial_maps_attested_fields() {
    let trial = from_nci_trial(&json!({
        "nct_id": "NCT11111111",
        "brief_title": "NCI trial",
        "current_trial_status": "RECRUITING",
        "phase": "PHASE3",
        "lead_org": "NCI",
        "start_date": "2020-01-01",
        "completion_date": "2024-12-31",
        "brief_summary": "Sentence one. Sentence two. Sentence three.",
        "diseases": ["Melanoma"]
    }))
    .into_test_result()
    .expect("valid NCI trial");

    assert_eq!(trial.nct_id, "NCT11111111");
    assert_eq!(trial.phase.as_deref(), Some("PHASE3"));
    assert_eq!(trial.conditions, vec!["Melanoma"]);
    assert!(
        trial
            .summary
            .as_deref()
            .is_some_and(|v| v.contains("Sentence one. Sentence two."))
    );
}

#[test]
fn trial_sections_maps_supported_nci_fields() {
    let trial = from_nci_trial(&json!({
        "nct_id": "NCT02296125",
        "brief_title": "Osimertinib in EGFR-mutant NSCLC",
        "current_trial_status": "ACTIVE",
        "phase": "PHASE3",
        "lead_org": "AstraZeneca",
        "diseases": ["Non-small cell lung cancer"]
    }))
    .into_test_result()
    .expect("valid NCI trial");

    assert_eq!(trial.nct_id, "NCT02296125");
    assert_eq!(trial.phase.as_deref(), Some("PHASE3"));
    assert_eq!(trial.sponsor.as_deref(), Some("AstraZeneca"));
    assert_eq!(trial.conditions, vec!["Non-small cell lung cancer"]);
}

#[test]
fn trial_status_normalization_variants() {
    let hit_a = from_nci_hit(&json!({
        "nct_id": "NCT02000622",
        "brief_title": "Olaparib Study",
        "current_trial_status": "recruiting"
    }))
    .into_test_result()
    .expect("valid NCI hit");
    let hit_b = from_nci_hit(&json!({
        "nct_id": "NCT04303780",
        "brief_title": "KRAS G12C Study",
        "current_trial_status": "RECRUITING"
    }))
    .into_test_result()
    .expect("valid NCI hit");

    assert_eq!(hit_a.status.to_ascii_uppercase(), "RECRUITING");
    assert_eq!(hit_b.status.to_ascii_uppercase(), "RECRUITING");
}
