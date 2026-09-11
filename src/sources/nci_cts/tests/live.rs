//! Human-only live NCI CTS smoke. These tests require the credential and network.
use crate::sources::nci_cts::NciCtsClient;

fn client() -> NciCtsClient {
    NciCtsClient::new().expect("NCI_API_KEY must be set")
}
fn melanoma(size: usize) -> biodata::NciCtsV2SearchPlan {
    let filters = biodata::ClinicalTrialSearchFilters::new(Default::default(), Default::default())
        .expect("filters");
    biodata::NciCtsV2SearchPlan::new(
        &filters,
        Some(biodata::NciCtsV2DiseaseSelection::Keyword(
            "melanoma".into(),
        )),
        size,
        0,
    )
    .expect("plan")
}

#[tokio::test]
#[ignore = "live network + NCI_API_KEY"]
async fn live_search_melanoma_returns_hits() {
    let response = client().search(&melanoma(2)).await.expect("live search");
    assert!(!response.results().unwrap_or_default().is_empty());
}

#[tokio::test]
#[ignore = "live network + NCI_API_KEY"]
async fn live_get_trial_by_id_round_trips() {
    let response = client().search(&melanoma(1)).await.expect("live search");
    let id = response
        .results()
        .unwrap_or_default()
        .first()
        .and_then(|r| {
            r.projection()
                .value()
                .identities()
                .first()
                .map(|i| i.identifier())
        })
        .expect("NCT identity");
    let plan = biodata::NciCtsV2DetailPlan::new(id, true).expect("detail plan");
    let trial = client().get(&plan).await.expect("live get");
    assert_eq!(trial.projection().trial().identities().len(), 2);
}
