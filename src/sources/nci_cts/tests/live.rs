//! Tier 4 — real round-trips against the NCI CTS API. `#[ignore]`d: needs `NCI_API_KEY`
//! and network. Run in the verify lane / coverage-parity check with
//! `cargo nextest run --run-ignored all -E 'test(/sources::nci_cts::/)'`.
//!
//! Exercises the thin async glue (plan -> request_from_plan -> get_json -> parse).

use crate::sources::nci_cts::{NciCtsClient, NciDiseaseFilter, NciSearchParams};

fn client() -> NciCtsClient {
    NciCtsClient::new().expect("NCI_API_KEY must be set for live nci_cts tests")
}

fn melanoma(size: usize) -> NciSearchParams {
    NciSearchParams {
        disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
        size,
        from: 0,
        ..Default::default()
    }
}

#[tokio::test]
#[ignore = "live network + NCI_API_KEY"]
async fn live_search_melanoma_returns_hits() {
    let resp = client()
        .search(&melanoma(2))
        .await
        .expect("live nci search");
    assert!(!resp.hits().is_empty());
}

#[tokio::test]
#[ignore = "live network + NCI_API_KEY"]
async fn live_get_trial_by_id_round_trips() {
    let resp = client()
        .search(&melanoma(1))
        .await
        .expect("live nci search");
    let id = resp
        .hits()
        .first()
        .and_then(|t| t.get("nct_id"))
        .and_then(|v| v.as_str())
        .expect("a trial with an nct_id");
    let plan = biodata::NciCtsV2DetailPlan::new(id, true).expect("valid NCT identity");
    let trial = client().get(&plan).await.expect("live nci get");
    assert_eq!(trial.projection().trial().identities().len(), 2);
}
