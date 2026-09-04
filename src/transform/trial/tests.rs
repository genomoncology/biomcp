use super::*;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use reqwest::StatusCode;
use serde_json::json;
#[path = "tests/ticket_1107.rs"]
mod ticket_1107;
#[path = "tests/ticket_1111.rs"]
mod ticket_1111;
#[path = "tests/ticket_1112.rs"]
mod ticket_1112;
#[path = "tests/ticket_1114.rs"]
mod ticket_1114;
#[path = "tests/ticket_1115.rs"]
mod ticket_1115;
#[path = "tests/ticket_1132.rs"]
mod ticket_1132;
use ticket_1107::{IntoTrialSearchTestResult, IntoTrialTestResult};

#[test]
fn receipted_ctgov_partial_sites_preserve_all_locations() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT00791778",
        StatusCode::OK,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/ctgov/get_nct00791778_20260902.json"
        )),
    )
    .expect("receipted NCT00791778 capture");

    let trial = from_ctgov_study(&study);
    let locations = trial.locations.as_ref().expect("locations");
    assert_eq!(locations.len(), 59);
    assert!(locations.iter().all(|location| location.facility.is_none()));
    assert!(locations.iter().all(|location| {
        location
            .city
            .as_deref()
            .is_some_and(|city| !city.is_empty())
            && location
                .country
                .as_deref()
                .is_some_and(|country| !country.is_empty())
    }));
    assert_eq!(
        locations
            .iter()
            .filter(|location| location.state.is_none())
            .count(),
        37
    );

    let structured = serde_json::to_value(&trial).expect("structured trial JSON");
    let serialized_locations = structured["locations"]
        .as_array()
        .expect("serialized locations");
    assert_eq!(serialized_locations.len(), 59);
    assert!(serialized_locations.iter().all(|location| {
        !location
            .as_object()
            .expect("serialized location object")
            .contains_key("facility")
            && location["city"]
                .as_str()
                .is_some_and(|city| !city.is_empty())
            && location["country"]
                .as_str()
                .is_some_and(|country| !country.is_empty())
    }));

    let markdown = crate::render::markdown::trial_markdown(&trial, &["locations".to_string()])
        .expect("locations Markdown");
    assert!(markdown.contains("| - | La Jolla, California | 92037 | United States |"));
}

#[test]
fn ctgov_meaningful_sites_keep_partial_identity_and_safe_markdown() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": "NCT11210000",
                "briefTitle": "Partial site boundary"
            },
            "contactsLocationsModule": {
                "locations": [
                    {"facility": "  Pipe | Facility\n\u{0007}  ", "status": "COMPLETED"},
                    {"city": "  City only  "},
                    {"state": "  State only  ", "status": "RECRUITING"},
                    {"zip": "  12345  "},
                    {"country": "  Country only  "},
                    {"geoPoint": {"lat": 42.0}},
                    {"contacts": [
                        {"name": "  First Contact  ", "email": "first@example.test"},
                        {"name": "Second Contact", "phone": "555-0102"}
                    ]},
                    {"status": "RECRUITING"},
                    {
                        "facility": "  ",
                        "city": "\n",
                        "state": "\t",
                        "zip": " ",
                        "country": "  ",
                        "contacts": [{"name": "  ", "email": "orphan@example.test"}]
                    }
                ]
            }
        }
    }))
    .expect("provider-shaped partial sites");

    let trial = from_ctgov_study(&study);
    let locations = trial.locations.as_ref().expect("meaningful locations");
    assert_eq!(locations.len(), 7);
    assert_eq!(locations[0].state.as_deref(), Some("State only"));
    assert_eq!(
        locations[1..]
            .iter()
            .map(|location| (
                location.facility.as_deref(),
                location.city.as_deref(),
                location.postal_code.as_deref(),
                location.country.as_deref(),
                location.latitude,
                location.contact_name.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some("Pipe | Facility\n\u{7}"), None, None, None, None, None),
            (None, Some("City only"), None, None, None, None),
            (None, None, Some("12345"), None, None, None),
            (None, None, None, Some("Country only"), None, None),
            (None, None, None, None, Some(42.0), None),
            (None, None, None, None, None, Some("First Contact")),
        ]
    );

    let contacts = trial.contacts.as_ref().expect("site contacts");
    assert_eq!(
        contacts
            .iter()
            .map(|contact| contact.name.as_str())
            .collect::<Vec<_>>(),
        vec!["First Contact", "Second Contact"]
    );

    let structured = serde_json::to_value(&trial).expect("structured trial JSON");
    let serialized = structured["locations"]
        .as_array()
        .expect("serialized meaningful locations");
    assert!(!serialized[0].as_object().unwrap().contains_key("facility"));
    assert!(!serialized[0].as_object().unwrap().contains_key("city"));
    assert!(!serialized[0].as_object().unwrap().contains_key("country"));
    assert_eq!(serialized[1]["facility"], "Pipe | Facility\n\u{7}");
    assert!(!serialized[1].as_object().unwrap().contains_key("city"));
    assert!(!serialized[1].as_object().unwrap().contains_key("country"));

    let markdown = crate::render::markdown::trial_markdown(&trial, &["locations".to_string()])
        .expect("locations Markdown");
    assert!(markdown.contains("| - | State only | - | - | RECRUITING | - |"));
    assert!(!markdown.contains("| - | , State only |"));
    let escaped_row = markdown
        .lines()
        .find(|line| line.contains("Pipe \\| Facility"))
        .expect("escaped facility row");
    assert!(!escaped_row.contains('\n'));
    assert!(!escaped_row.contains('\u{7}'));
    assert_eq!(escaped_row.matches(" | ").count(), 5);
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
    assert_eq!(locations[0].facility.as_deref(), Some("Site A"));
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
    assert_eq!(
        trial.summary.as_deref(),
        Some("Sentence one. Sentence two. Sentence three.")
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
