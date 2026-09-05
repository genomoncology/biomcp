use super::*;
use crate::render::markdown::trial_markdown;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use reqwest::StatusCode;
use serde_json::json;

#[test]
fn receipted_ctgov_location_preserves_postal_code_in_json_and_markdown() {
    let study = ClinicalTrialsClient::decode_get_response(
        "NCT02576665",
        StatusCode::OK,
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/ctgov/get_nct02576665_20260811.json"
        )),
    )
    .expect("receipted NCT02576665 capture");

    let trial = from_ctgov_study(&study).expect("valid trial fixture");
    let sarah_cannon = trial
        .locations
        .as_ref()
        .expect("locations")
        .iter()
        .find(|location| location.facility.as_deref() == Some("Sarah Cannon Research Institute"))
        .expect("Sarah Cannon location");

    assert_eq!(sarah_cannon.postal_code.as_deref(), Some("80218"));

    let structured = serde_json::to_value(&trial).expect("structured trial JSON");
    let structured_sarah_cannon = structured["locations"]
        .as_array()
        .expect("serialized locations")
        .iter()
        .find(|location| location["facility"] == "Sarah Cannon Research Institute")
        .expect("serialized Sarah Cannon location");
    assert_eq!(structured_sarah_cannon["postal_code"], "80218");

    let markdown = trial_markdown(&trial, &["locations".to_string()]).expect("locations Markdown");
    assert!(markdown.contains("| Facility | City | Postal code | Country | Status | Contact |"));
    assert!(markdown.contains(
        "| Sarah Cannon Research Institute | Denver, Colorado | 80218 | United States |"
    ));
}

#[test]
fn ctgov_postal_codes_are_trimmed_and_blank_values_are_omitted() {
    let study: CtGovStudy = serde_json::from_value(json!({
        "protocolSection": {
            "identificationModule": {
                "nctId": "NCT01114000",
                "briefTitle": "Postal code cleanup"
            },
            "contactsLocationsModule": {
                "locations": [
                    {
                        "facility": "Trimmed Postal Site",
                        "city": "London",
                        "zip": "  SW1A 1AA  ",
                        "country": "United Kingdom"
                    },
                    {
                        "facility": "Blank Postal Site",
                        "city": "Paris",
                        "zip": "  \t ",
                        "country": "France"
                    },
                    {
                        "facility": "Absent Postal Site",
                        "city": "Berlin",
                        "country": "Germany"
                    }
                ]
            }
        }
    }))
    .expect("provider-shaped study");

    let trial = from_ctgov_study(&study).expect("valid trial fixture");
    let locations = trial.locations.as_ref().expect("locations");
    let trimmed = locations
        .iter()
        .find(|location| location.facility.as_deref() == Some("Trimmed Postal Site"))
        .expect("trimmed postal location");
    let blank = locations
        .iter()
        .find(|location| location.facility.as_deref() == Some("Blank Postal Site"))
        .expect("blank postal location");
    let absent = locations
        .iter()
        .find(|location| location.facility.as_deref() == Some("Absent Postal Site"))
        .expect("absent postal location");

    assert_eq!(trimmed.postal_code.as_deref(), Some("SW1A 1AA"));
    assert_eq!(blank.postal_code, None);
    assert_eq!(absent.postal_code, None);

    let structured = serde_json::to_value(&trial).expect("structured trial JSON");
    let serialized_locations = structured["locations"]
        .as_array()
        .expect("serialized locations");
    let serialized_blank = serialized_locations
        .iter()
        .find(|location| location["facility"] == "Blank Postal Site")
        .expect("serialized blank postal location");
    let serialized_absent = serialized_locations
        .iter()
        .find(|location| location["facility"] == "Absent Postal Site")
        .expect("serialized absent postal location");
    assert!(
        !serialized_blank
            .as_object()
            .unwrap()
            .contains_key("postal_code")
    );
    assert!(
        !serialized_absent
            .as_object()
            .unwrap()
            .contains_key("postal_code")
    );
}
