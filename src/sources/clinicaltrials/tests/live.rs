//! Tier 4 — live upstream smoke. Ignored so normal gates stay pure and fast.

use crate::sources::clinicaltrials::{ClinicalTrialsClient, CtGovSearchParams};

#[tokio::test]
#[ignore = "live network"]
async fn live_search_returns_cancer_trials() {
    let client = ClinicalTrialsClient::new().expect("client");
    let response = client
        .search(&CtGovSearchParams {
            condition: Some("melanoma".into()),
            page_size: 1,
            ..Default::default()
        })
        .await
        .expect("live ClinicalTrials.gov search");
    assert!(!response.results().unwrap_or_default().is_empty());
}
