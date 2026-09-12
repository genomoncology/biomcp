use super::*;
use crate::entities::trial::{TrialResponse, TrialSectionState, TrialSectionStates};

fn response() -> TrialResponse {
    let plan = biodata::ClinicalTrialsGovApiV2DetailPlan::new("NCT41300001", false)
        .unwrap()
        .with_contacts()
        .with_locations();
    let response = biodata::ClinicalTrialsGovApiV2Response::parse(
            &plan,
            br#"{"protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Paging trial"},"statusModule":{"overallStatus":"RECRUITING"},"sponsorCollaboratorsModule":{"leadSponsor":{"name":"Sponsor"}},"conditionsModule":{"conditions":["Condition"]},"designModule":{"studyType":"INTERVENTIONAL"},"contactsLocationsModule":{"centralContacts":[{"name":"Central Contact"}],"locations":[{"facility":"Site 0","city":"City 0","country":"Example Country","contacts":[{"name":"Site Contact 0"}]},{"facility":"Site 1","city":"City 1","country":"Example Country","contacts":[{"name":"Site Contact 1"}]},{"facility":"Site 2","city":"City 2","country":"Example Country","contacts":[{"name":"Site Contact 2"}]}]}}}"#,
            &Default::default(),
        )
        .unwrap();
    TrialResponse::new(
        response.into_projection().unwrap(),
        "ClinicalTrials.gov",
        None,
        TrialSectionStates {
            arms: TrialSectionState::NotRequested,
            eligibility: TrialSectionState::NotRequested,
            outcomes: TrialSectionState::NotRequested,
            references: TrialSectionState::NotRequested,
            contacts: TrialSectionState::Present,
            locations: TrialSectionState::Present,
        },
    )
}

#[test]
fn location_page_filters_sites_and_site_contacts_but_keeps_central_first() {
    let mut response = response();
    let page = paginate_trial_locations(&mut response, 1, 1);
    let encoded = trial_response_locations_json(&response, page).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["location_pagination"]["total"], 3);
    assert_eq!(value["locations"][0]["facility"], "Site 1");
    assert_eq!(value["contacts"][0]["level"], "central");
    assert_eq!(value["contacts"][1]["level"], "site");
    assert_eq!(value["contacts"][1]["facility"], "Site 1");
    assert_eq!(value["contacts"].as_array().unwrap().len(), 2);
}

#[test]
fn standalone_pagination_wrapper_serializes_only_the_current_directory_page() {
    let mut response = response();
    let page = paginate_trial_locations(&mut response, 2, 1);
    let encoded = trial_locations_json(&response, page).unwrap();
    let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(value["locations"][0]["facility"], "Site 2");
    assert_eq!(value["contacts"][1]["facility"], "Site 2");
}
