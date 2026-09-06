//! Tier 4 — real round-trips against mygene.info. `#[ignore]`d: excluded from the
//! routine `make test` gate. Run in the verify lane / coverage-parity check with
//! `cargo nextest run --run-ignored all -E 'test(/sources::mygene::/)'`.
//!
//! These exercise the thin async glue (plan -> request_from_plan -> get_json -> parse)
//! end to end and catch upstream drift.

use crate::error::BioMcpError;
use crate::sources::mygene::MyGeneClient;

fn client() -> MyGeneClient {
    MyGeneClient::new().expect("construct live mygene client")
}

#[tokio::test]
#[ignore = "live network"]
async fn live_get_braf_returns_symbol_and_ensembl() {
    let resp = client().get("BRAF", true).await.expect("live get BRAF");
    assert_eq!(resp.symbol.as_deref(), Some("BRAF"));
    assert!(resp.ensembl.and_then(|e| e.gene().cloned()).is_some());
}

#[tokio::test]
#[ignore = "live network"]
async fn live_search_egfr_returns_hits() {
    let resp = client()
        .search("EGFR", 3, 0, None)
        .await
        .expect("live search EGFR");
    assert!(!resp.hits.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn live_resolve_uniprot_for_braf() {
    let acc = client()
        .resolve_uniprot_accession("BRAF")
        .await
        .expect("live uniprot BRAF");
    assert!(!acc.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn live_get_unknown_symbol_is_not_found() {
    let err = client().get("ZZZNOTAREALGENE", false).await.unwrap_err();
    assert!(matches!(err, BioMcpError::NotFound { .. }));
}

#[tokio::test]
#[ignore = "live network"]
async fn live_symbols_for_entrez_ids_resolves_known_ids() {
    let symbols = client()
        .symbols_for_entrez_ids(&["1956".to_string(), "7157".to_string()])
        .await
        .expect("live batch symbols");
    assert!(symbols.contains(&"EGFR".to_string()));
}
