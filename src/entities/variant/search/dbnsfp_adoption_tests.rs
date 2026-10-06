//! Callable diagnostics preserve requested identity and enumerate independent aliases.
use super::*;
use crate::entities::article::test_support::TestEnv;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn dbnsfp_zero_result_reports_all_positions_without_renumbering() {
    let (fixture, requests) = super::exact_scan_tests::fixture(vec![
        json!({"total":0,"hits":[]}),
        json!({"total":1,"hits":[]}),
        json!({"total":0,"hits":[]}),
        json!({"total":2,"hits":[
            {"_id":"neighbor-a","dbnsfp":{"hgvsp":["p.V601E","p.V599E","p.V601E"]}},
            {"_id":"neighbor-b","dbnsfp":{"hgvsp":"p.Val599Glu"}}]}),
    ])
    .await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let requested = RequestedVariantIdentity::for_search(
        Some("BRAF".into()),
        Some("p.V600E".into()),
        None,
        None,
    );
    let filters = VariantSearchFilters {
        gene: Some("BRAF".into()),
        hgvsp: Some("p.V600E".into()),
        requested_identity: Some(requested.clone()),
        ..Default::default()
    };
    let page = search_page(&filters, 5, 0).await.unwrap();
    assert!(page.results.is_empty());
    assert_eq!(page.requested_variant, Some(requested));
    assert_eq!(
        page.resolution.unwrap().status,
        VariantResolutionStatus::Unresolved
    );
    assert_eq!(
        page.diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["no dbNSFP record for BRAF p.V600E; dbNSFP holds V to E at positions 599, 601"]
    );
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), 4, "{log:?}");
    let words: Vec<_> = log[3].lines().next().unwrap().split_whitespace().collect();
    let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
    assert_eq!(url.path(), "/query");
    assert_eq!(
        url.query_pairs().into_owned().collect::<Vec<_>>(),
        vec![
            (
                "q".into(),
                "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:p.V*E".into()
            ),
            ("size".into(), "50".into()),
            ("from".into(), "0".into()),
            (
                "fields".into(),
                crate::sources::myvariant::MYVARIANT_FIELDS_SEARCH.into()
            )
        ]
    );
    assert_eq!(log[3].split_once("\r\n\r\n").unwrap().1, "");
}
