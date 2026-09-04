use super::super::*;
use super::ticket_1107::IntoTrialTestResult;

#[test]
fn recorded_nci_trial_maps_provider_field_locations() {
    let response: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../testdata/sources/nci_cts/search_melanoma.json"
    ))
    .expect("recorded NCI response");
    let record = &response["data"][0];
    let expected_interventions = record["arms"]
        .as_array()
        .expect("recorded arms")
        .iter()
        .flat_map(|arm| {
            arm["interventions"]
                .as_array()
                .expect("recorded arm interventions")
        })
        .map(|intervention| {
            intervention["name"]
                .as_str()
                .expect("recorded intervention name")
                .to_string()
        })
        .take(25)
        .collect::<Vec<_>>();
    assert!(!expected_interventions.is_empty());

    let trial = from_nci_trial(record)
        .into_test_result()
        .expect("valid recorded NCI trial");

    assert_eq!(
        (
            trial.interventions,
            trial.age_range.as_deref(),
            trial.study_type.as_deref(),
            trial.enrollment,
        ),
        (
            expected_interventions,
            Some("18 Years to Any age"),
            Some("Interventional"),
            Some(2400),
        )
    );
}
