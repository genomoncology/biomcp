//! Public article route binds citations to the confirmed source before spelling selection.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[tokio::test]
#[serial_test::serial(source_env)]
async fn confirmed_citations_use_canonical_hit_order_and_exclude_decoys() {
    for positional in [false, true] {
        for reversed in [false, true] {
            let hit = |rsid: &str, citation: &str| {
                json!({"_id":"chr1:g.101A>T",
                "dbsnp": if positional {json!([rsid])} else {json!({"rsid":rsid})},
                "civic":{"molecularProfiles":[{"evidenceItems":[{"source":{
                    "sourceType":"PUBMED","citation":citation}}]}]}})
            };
            // Canonical dbsnp precedes civic: leading space selects 111 over 222.
            // The decoy sorts earlier still, so omitting the confirmed-key filter selects 333.
            let mut hydrated = vec![
                hit(" RS101 ", "PMID:111"),
                hit("rs101", "PMID:222"),
                hit("  RS000 ", "PMID:333"),
            ];
            if reversed {
                hydrated.reverse();
            }
            let requests = Arc::new(Mutex::new(Vec::new()));
            let captured = requests.clone();
            let fixture = TestHttpFixture::spawn(move |request| {
                let mut log = captured.lock().unwrap();
                let target = request
                    .lines()
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .nth(1)
                    .unwrap();
                let url = reqwest::Url::parse(&format!("http://localhost{target}")).unwrap();
                log.push(request.to_string());
                if url.path() == "/query" || url.path() == "/variant/chr1:g.101A%3ET" {
                    let body = if url.path() == "/query" {
                        json!({"total":1,"hits":[{"_id":"chr1:g.101A>T","dbsnp":{"rsid":"rs101"}}]})
                    } else {
                        json!(hydrated)
                    };
                    TestHttpReply::Bytes(test_http_response(
                        "200 OK",
                        "application/json",
                        &serde_json::to_vec(&body).unwrap(),
                    ))
                } else {
                    // Other route failures are already owned; keep this case about citation binding.
                    TestHttpReply::Bytes(test_http_response(
                        "400 Bad Request",
                        "application/json",
                        b"{}",
                    ))
                }
            })
            .await;
            let mut env = TestEnv::new();
            for name in [
                "BIOMCP_MYVARIANT_BASE",
                "BIOMCP_PUBTATOR_BASE",
                "BIOMCP_PUBMED_BASE",
                "BIOMCP_EUROPEPMC_BASE",
                "BIOMCP_S2_BASE",
                "BIOMCP_CLINGEN_CAR_BASE",
                "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
                "BIOMCP_NCBI_IDCONV_BASE",
            ] {
                env.set(name, &fixture.base);
            }
            env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
            env.set("BIOMCP_CACHE_MODE", "off");
            let outcome = search_variant_articles_with_options(
                "rs101",
                VariantArticleStrategy::Union,
                5,
                0,
                true,
                VariantArticleVerificationOptions::default(),
            )
            .await
            .unwrap();
            assert!(!outcome.hard_error, "{:?}", outcome.response);
            let response = serde_json::to_value(outcome.response).unwrap();
            assert_eq!(
                response["results"].as_array().unwrap().len(),
                1,
                "{response}"
            );
            assert_eq!(response["results"][0]["pmid"], "111", "{response}");
            assert_eq!(
                response["results"][0]["provenance"][0]["route"], "source_citation",
                "{response}"
            );
            let log = requests.lock().unwrap();
            let variant_requests = log
                .iter()
                .filter(|line| line.starts_with("GET /variant/"))
                .collect::<Vec<_>>();
            assert_eq!(variant_requests.len(), 1, "{log:?}");
            for request in variant_requests {
                let target = request
                    .lines()
                    .next()
                    .unwrap()
                    .split_whitespace()
                    .nth(1)
                    .unwrap();
                let url = reqwest::Url::parse(&format!("http://localhost{target}")).unwrap();
                assert_eq!(
                    url.query_pairs().into_owned().collect::<Vec<_>>(),
                    vec![(
                        "fields".into(),
                        crate::sources::myvariant::MYVARIANT_FIELDS_GET.into()
                    )]
                );
                assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
            }
            assert!(response["source_status"].as_array().unwrap().iter().any(
                |status| status["route"] == "source_citation"
                    && status["source"] == "myvariant"
                    && status["status"] == "ok"
            ));
        }
    }
}
