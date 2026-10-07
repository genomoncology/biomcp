//! Source presence plans hydration independently of cached display admission.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};

async fn response(civic: Option<serde_json::Value>) -> serde_json::Value {
    let mut hit = serde_json::json!({"_id":"chr1:g.101A>T","dbsnp":{"rsid":"rs101"}});
    if let Some(civic) = civic {
        hit["civic"] = civic;
    }
    let fixture = TestHttpFixture::spawn(move |request| {
        let body = if request.starts_with("GET /query?") {
            serde_json::to_vec(&serde_json::json!({"total":1,"hits":[hit]})).unwrap()
        } else if request.starts_with("GET /variant/") {
            serde_json::to_vec(&hit).unwrap()
        } else {
            return TestHttpReply::Bytes(test_http_response(
                "400 Bad Request",
                "application/json",
                b"{}",
            ));
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
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
    serde_json::to_value(outcome.response).unwrap()
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cached_source_presence_plans_only_missing_hydration() {
    for (civic, planned) in [
        (None, 1),
        (Some(serde_json::json!(null)), 1),
        (Some(serde_json::json!(false)), 0),
        (Some(serde_json::json!([])), 0),
        (Some(serde_json::json!({})), 0),
        (
            Some(serde_json::json!({"molecularProfiles":[{"name":"P","evidenceItems":[]}]})),
            0,
        ),
    ] {
        let actual = response(civic).await;
        let status = actual["source_status"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["route"] == "source_citation" && s["source"] == "myvariant")
            .unwrap();
        assert_eq!(status["work"]["planned"], planned, "{actual}");
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citations_beyond_cached_display_cap_reach_article_route() {
    let mut items = (1..=20)
        .map(|id| serde_json::json!({"id":id}))
        .collect::<Vec<_>>();
    items.push(serde_json::json!({"id":21,"source":{"sourceType":"PUBMED","citation":"333"}}));
    let actual = response(Some(serde_json::json!({"molecularProfiles":[{"name":"P","evidenceItems":items},{"evidenceItems":[{"source":{"sourceType":"PUBMED","citation":"444"}}]}]}))).await;
    let rows = actual["results"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| r["pmid"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["333", "444"]
    );
    for row in rows {
        assert_eq!(row["provenance"][0]["route"], "source_citation");
        assert_eq!(row["provenance"][0]["source"], "civic");
    }
    let status = actual["source_status"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["route"] == "source_citation" && s["source"] == "myvariant")
        .unwrap();
    assert_eq!(status["work"]["planned"], 0);
}
