//! Callable search retains ordered cached evidence without a dbNSFP gene.
use super::*;
use crate::entities::article::test_support::TestEnv;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn clinvar_search_preserves_fallback_gene_order_and_page() {
    for significance in ["Pathogenic", "Likely pathogenic"] {
        let row = json!({"_id":"chr7:g.101A>T","cadd":{"consequence":"synonymous"},
            "dbnsfp":{"revel":{"score":[0.94,0.11]}},
            "snpeff":{"ann":{"feature_id":"NM_999999.1","hgvs_p":"p.Other"}},
            "clinvar":{"gene":{"symbol":" braf "},"rcv":[
                {"preferred_name":"NM_012345.7:c.1A>G (p.Val1Gly)","clinical_significance":"Benign",
                    "review_status":"criteria provided, single submitter","last_evaluated":"2024-01-01"},
                {"preferred_name":"NM_654321.2:c.2A>G (p.Val2Gly)","clinical_significance":significance,
                    "review_status":"reviewed by expert panel","last_evaluated":"2025-01-01"}]}});
        let (fixture, requests) =
            super::exact_scan_tests::fixture(vec![json!({"total":1,"hits":[row]})]).await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let page = search_page(
            &VariantSearchFilters {
                gene: Some("BRAF".into()),
                ..Default::default()
            },
            1,
            0,
        )
        .await
        .unwrap();
        assert_eq!(
            serde_json::to_value(&page.results).unwrap(),
            json!([{
                "id":"chr7:g.101A>T","genome_build":"GRCh37","genome_build_provenance":"MyVariant.info provider default",
                "gene":"BRAF","hgvs_p":"p.Val1Gly","hgvs_c":"c.1A>G","transcript":"NM_012345.7",
                "legacy_name":"BRAF V1G","significance":significance,"significance_source":"MyVariant.info",
                "significance_evaluated":"2025-01-01","clinvar_stars":3,"revel":0.94,"gerp":null
            }])
        );
        assert_eq!(page.total, Some(1));
        assert!(page.requested_variant.is_none());
        assert!(page.resolution.is_none());
        assert!(page.has_more.is_none());
        assert!(page.diagnostics.is_empty());
        assert_eq!(
            serde_json::to_value(page.filter_evaluation).unwrap(),
            json!({"gene":"evaluated"})
        );
        let log = requests.lock().unwrap();
        assert_eq!(log.len(), 1);
        let words: Vec<_> = log[0].lines().next().unwrap().split_whitespace().collect();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/query");
        assert_eq!(
            url.query_pairs().into_owned().collect::<Vec<_>>(),
            [
                ("q".into(), "dbnsfp.genename:BRAF".into()),
                ("size".into(), "40".into()),
                ("from".into(), "0".into()),
                (
                    "fields".into(),
                    crate::sources::myvariant::MYVARIANT_FIELDS_SEARCH.into()
                )
            ]
        );
        assert_eq!(log[0].split_once("\r\n\r\n").unwrap().1, "");
    }
}
