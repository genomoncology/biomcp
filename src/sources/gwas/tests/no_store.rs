//! The runtime no-store pin for GWAS (ticket 1268 follow-up,
//! 2026-09-29 second review): reqwest-middleware 0.4.2 exposes
//! `RequestBuilder::extensions()` publicly, so the invariant is
//! proven on the middleware builder the cache layer reads — before
//! `build()`, which hands off to reqwest and drops the middleware's
//! extension view (the first attempt read `Request::extensions()`
//! after `build()` and hit reqwest's private method; CI run
//! 36619750222, error E0624).

use super::super::*;
use http_cache_reqwest::CacheMode;

/// Every GWAS request carries `CacheMode::NoStore`, whatever
/// `BIOMCP_CACHE_MODE` says: both plan entry points route through
/// `request_no_store`, and the extension sits on the middleware
/// builder the cache middleware inspects.
#[test]
#[serial_test::serial(source_env)]
fn gwas_requests_carry_no_store_on_the_middleware_builder() {
    let _mode = crate::sources::test_cache_mode::infinite();
    assert_eq!(
        crate::sources::current_cache_mode(),
        Some(CacheMode::ForceCache)
    );
    let client = GwasClient {
        client: crate::sources::shared_client().expect("shared client"),
        base: std::borrow::Cow::Borrowed("http://gwas.test"),
    };

    let plans = [
        GwasClient::associations_by_rsid_plan("rs36053993", 10).expect("rsid plan"),
        GwasClient::association_search_plan(Some("BRAF"), None, 5).expect("search plan"),
    ];
    for plan in plans {
        let mut builder = client.request_no_store(&plan);
        assert_eq!(
            builder.extensions().get::<CacheMode>(),
            Some(&CacheMode::NoStore),
            "the GWAS request must carry CacheMode::NoStore into the cache layer"
        );
    }
}
