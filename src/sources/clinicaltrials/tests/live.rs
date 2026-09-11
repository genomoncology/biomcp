//! Human-only live ClinicalTrials.gov smoke.
use crate::sources::clinicaltrials::ClinicalTrialsClient;

#[tokio::test]
#[ignore = "live network"]
async fn live_search_returns_cancer_trials() {
    let filters = biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("melanoma".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .expect("filters");
    let plan =
        biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, false).expect("plan");
    let response = ClinicalTrialsClient::new()
        .expect("client")
        .search(&plan)
        .await
        .expect("live search");
    assert!(!response.results().unwrap_or_default().is_empty());
}
