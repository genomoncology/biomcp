//! Tier 4 — live upstream smoke. Ignored so normal gates stay pure and fast.

use crate::sources::pubmed::{PubMedClient, PubMedESearchParams};

#[tokio::test]
#[ignore = "live network"]
async fn live_esearch_returns_braf_hits() {
    let client = PubMedClient::new().expect("client");
    let response = client
        .esearch(&PubMedESearchParams {
            term: "BRAF melanoma".into(),
            sort: None,
            retstart: 0,
            retmax: 1,
            date_from: None,
            date_to: None,
        })
        .await
        .expect("live PubMed ESearch");
    assert!(response.count > 0);
}
