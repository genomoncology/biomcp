use super::*;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use reqwest::StatusCode;
use serde_json::json;

const TOCA_511_DESCRIPTION: &str = "Toca 511 consists of a purified retroviral replicating vector encoding a modified yeast cytosine deaminase (CD) gene. The CD gene converts the antifungal 5-fluorocytosine (5FC) to the anticancer drug 5-FU in cells that have been infected by the Toca 511 vector";
const TOCA_FC_DESCRIPTION: &str = "Toca FC is an extended-release formulation of flucytosine. Toca FC is supplied as 500 mg white, oblong tablets with \"TOCA FC\" embossed on one side and \"500\" embossed on the other side";

#[path = "tests/ticket_1107.rs"]
mod ticket_1107;
#[path = "tests/ticket_1111.rs"]
mod ticket_1111;
#[path = "tests/ticket_1115.rs"]
mod ticket_1115;
use ticket_1107::IntoTrialSearchTestResult;

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
            "outcomesModule": {
                "primaryOutcomes": [{"measure": "Overall survival"}],
                "secondaryOutcomes": [{"measure": "Progression-free survival"}]
            }
        }
    }))
    .unwrap();

    let trial = from_ctgov_study(&study).expect("valid trial fixture");
    assert!(trial.design.interventions().is_empty());
    assert!(trial.design.arms().is_none());
    assert!(trial.outcomes.is_none());
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

        let trial = from_ctgov_study(&study).expect("valid trial fixture");
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
    let trial = from_ctgov_study(&study).expect("valid trial fixture");
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
    let ordinary_trial = from_ctgov_study(&ordinary_study).expect("valid trial fixture");
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

#[test]
fn receipted_ctgov_intervention_descriptions_keep_their_associations_in_json() {
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT02576665",
        &["arms".to_string()],
        StatusCode::OK,
        include_bytes!("../../../testdata/sources/ctgov/get_nct02576665_full_20260903.json"),
    )
    .expect("receipted unrestricted NCT02576665 capture");
    let design = crate::entities::trial::product_design(response.interventions(), response.arms())
        .expect("shared trial design");
    assert_eq!(design.interventions().len(), 2);
    assert_eq!(design.assignments().map(<[_]>::len), Some(2));
    let first = &design.interventions()[0];
    assert_eq!(first.name(), "Toca 511");
    assert_eq!(
        first.source_type().map(|value| value.code()),
        Some("BIOLOGICAL")
    );
    assert_eq!(first.description(), Some(TOCA_511_DESCRIPTION));
    assert_eq!(
        first.other_names().unwrap(),
        &[
            "vocimagene amiretrorepvec",
            "RRV",
            "retroviral replicating viral"
        ]
    );
    let second = &design.interventions()[1];
    assert_eq!(second.name(), "Toca FC");
    assert_eq!(second.source_type().map(|value| value.code()), Some("DRUG"));
    assert_eq!(second.description(), Some(TOCA_FC_DESCRIPTION));
    assert_eq!(
        second.other_names().unwrap(),
        &["Flucytosine", "5-FC", "5-Fluorocytosine"]
    );
    let json = serde_json::to_value(&design).expect("structured trial JSON");
    assert_eq!(
        json["interventions"][0]["type"],
        serde_json::json!({
            "authority": "clinicaltrials.gov", "code": "BIOLOGICAL",
            "display": null, "vocabulary_version": null, "recognized_meaning": null
        })
    );
    assert_eq!(
        json["arms"][0]["type"],
        serde_json::json!({
            "authority": "clinicaltrials.gov", "code": "EXPERIMENTAL",
            "display": null, "vocabulary_version": null, "recognized_meaning": null
        })
    );
    assert!(json.get("intervention_details").is_none());
    assert_eq!(
        json["arm_intervention_assignments"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    let decoded: crate::entities::trial::TrialDesign =
        serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), json);
}

#[test]
fn receipted_ctgov_two_arm_trial_keeps_independent_typed_assignments() {
    let response = ClinicalTrialsClient::decode_biodata_detail_response(
        "NCT00791778",
        &["arms".to_string()],
        StatusCode::OK,
        include_bytes!("../../../testdata/sources/ctgov/get_nct00791778_20260902.json"),
    )
    .expect("receipted NCT00791778 capture");
    let design = crate::entities::trial::product_design(response.interventions(), response.arms())
        .expect("shared trial design");
    assert_eq!(design.arms().map(<[_]>::len), Some(2));
    assert_eq!(design.interventions().len(), 2);
    assert_eq!(design.assignments().map(<[_]>::len), Some(2));
    let assignments = design.assignments().unwrap();
    assert_ne!(assignments[0].arm_id(), assignments[1].arm_id());
    assert_ne!(
        assignments[0].intervention_id(),
        assignments[1].intervention_id()
    );
}
