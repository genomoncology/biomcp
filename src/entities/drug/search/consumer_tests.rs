//! Complete ranking and EMA admission controls use accepted source pages.
use super::*;
use crate::entities::article::test_support::TestEnv;
use crate::entities::drug::get::consumer_tests::CaseHttp;
use crate::sources::mychem::consumer_tests::{bytes, profile, search_value, table};
use serde_json::json;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ranked_drug_paging_table() {
    let _cache_mode = crate::sources::test_cache_mode::off();
    for case in table(2)
        .into_iter()
        .filter(|case| case["input"]["operation"] == "search_ranked_name_us_page")
    {
        let fixture = CaseHttp::new(&case).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = TestEnv::new();
        fixture.environment(&mut env, cache.path());
        let input = &case["input"];
        let expected = &case["expected"];
        let filters: DrugSearchFilters = serde_json::from_value(input["filters"].clone()).unwrap();
        let result = search_ranked_name_us_page(
            &filters,
            input["query"].as_str().unwrap(),
            input["limit"].as_u64().unwrap() as usize,
            input["offset"].as_u64().unwrap() as usize,
        )
        .await;
        if expected["failure"].is_null() {
            let page = result.unwrap();
            assert_eq!(
                json!(page.results.iter().map(search_value).collect::<Vec<_>>()),
                expected["results"]
            );
            assert_eq!(json!(page.total), expected["total"]);
            assert_eq!(json!(page.match_kinds), expected["match_kinds"]);
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), expected["failure"]["code"]);
        }
        fixture.assert_requests(&case["id"]);
    }
}
#[test]
fn adopted_ema_source_admission_table() {
    for case in table(6).into_iter().filter(|case| {
        case["input"]["operation"] == "admitted_page_then_ema_identity_from_source_companion"
    }) {
        let response =
            crate::sources::mychem::projection::decode(&bytes(&case), profile(&case)).unwrap();
        let identity =
            ema_identity_from_mychem_hits(case["input"]["query"].as_str().unwrap(), &response.hits)
                .unwrap();
        let actual = identity
            .terms_for_test()
            .into_iter()
            .map(|(text, source)| json!({"text":text,"source":source}))
            .collect::<Vec<_>>();
        assert_eq!(json!(actual), case["expected"]["terms"]);
    }
}
