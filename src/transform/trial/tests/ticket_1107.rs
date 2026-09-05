use super::super::*;
use serde_json::json;
use std::fmt::Display;

pub(super) trait IntoTrialSearchTestResult {
    fn into_test_result(self) -> Result<TrialSearchResult, String>;
}

impl IntoTrialSearchTestResult for TrialSearchResult {
    fn into_test_result(self) -> Result<TrialSearchResult, String> {
        Ok(self)
    }
}

impl<E: Display> IntoTrialSearchTestResult for Result<TrialSearchResult, E> {
    fn into_test_result(self) -> Result<TrialSearchResult, String> {
        self.map_err(|error| error.to_string())
    }
}

#[test]
fn recorded_nci_hit_reports_its_disease_names() {
    let response: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../testdata/sources/nci_cts/search_melanoma_20260811.json"
    ))
    .expect("recorded NCI response");
    let record = &response["data"][0];
    let names = record["diseases"]
        .as_array()
        .expect("recorded diseases")
        .iter()
        .map(|disease| {
            disease["name"]
                .as_str()
                .expect("recorded disease name")
                .to_string()
        })
        .collect::<Vec<_>>();

    let hit = from_nci_hit(record)
        .into_test_result()
        .expect("valid recorded NCI hit");

    assert_eq!(hit.conditions, names);
}

#[test]
fn unreadable_nci_disease_is_an_error() {
    let mut diseases = (0..25)
        .map(|index| json!({"name": format!("Condition {index}")}))
        .collect::<Vec<_>>();
    diseases.push(json!({"nci_thesaurus_concept_id": "C000000"}));
    let record = json!({
        "nct_id": "NCT00000001",
        "diseases": diseases
    });

    let hit = from_nci_hit(&record).into_test_result();

    assert!(hit.is_err(), "an unreadable disease must reject the hit");
}
