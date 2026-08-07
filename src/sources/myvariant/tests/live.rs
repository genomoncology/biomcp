//! Tier 4 — real round-trips against myvariant.info. `#[ignore]`d: excluded from the
//! routine `make test` gate. Run in the verify lane / coverage-parity check with
//! `cargo nextest run --run-ignored all -E 'test(/sources::myvariant::/)'`.
//!
//! These exercise the thin async glue (plan -> request_from_plan -> get_json -> parse)
//! end to end and catch upstream drift.

use crate::error::BioMcpError;
use crate::sources::myvariant::{MYVARIANT_FIELDS_SEARCH, MyVariantClient, VariantSearchParams};

fn client() -> MyVariantClient {
    MyVariantClient::new().expect("construct live myvariant client")
}

fn search_params() -> VariantSearchParams {
    VariantSearchParams {
        gene: Some("BRAF".into()),
        hgvsp: None,
        hgvsc: None,
        rsid: None,
        protein_alias: None,
        significance: None,
        max_frequency: None,
        min_cadd: None,
        consequence: None,
        review_status: None,
        population: None,
        revel_min: None,
        gerp_min: None,
        tumor_site: None,
        condition: None,
        impact: None,
        lof: false,
        has: None,
        missing: None,
        therapy: None,
        limit: 5,
        offset: 0,
    }
}

#[tokio::test]
#[ignore = "live network"]
async fn live_get_braf_v600e_returns_hit() {
    let hit = client()
        .get("chr7:g.140453136A>T", None)
        .await
        .expect("live get BRAF V600E");
    assert_eq!(hit.id, "chr7:g.140453136A>T");
    assert_eq!(
        hit.dbnsfp.as_ref().and_then(|d| d.genename.first()),
        Some("BRAF")
    );
}

#[tokio::test]
#[ignore = "live network"]
async fn live_get_unknown_variant_is_not_found() {
    let err = client()
        .get("chr1:g.999999999999A>T", None)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        BioMcpError::NotFound { .. } | BioMcpError::Api { .. }
    ));
}

#[tokio::test]
#[ignore = "live network"]
async fn live_query_with_fields_returns_hits() {
    let resp = client()
        .query_with_fields("dbnsfp.genename:BRAF", 3, 0, MYVARIANT_FIELDS_SEARCH)
        .await
        .expect("live query_with_fields BRAF");
    assert!(!resp.hits.is_empty());
}

#[tokio::test]
#[ignore = "live network"]
async fn live_search_braf_returns_hits() {
    let resp = client()
        .search(&search_params())
        .await
        .expect("live search BRAF");
    assert!(!resp.hits.is_empty());
}
