//! Public article route binds citations to the confirmed source before spelling selection.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use std::sync::{Arc, Mutex};

async fn article_response(hydrated: String, hydrate: bool) -> serde_json::Value {
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
        if hydrate { r#"{"total":1,"hits":[{"_id":"chr1:g.101A>T","dbsnp":{"rsid":"rs101"}}]}"#.to_owned() }
        else { format!(r#"{{"total":3,"hits":{hydrated}}}"#) }
            } else { hydrated.clone() };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body.as_bytes()))
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
    let log = requests.lock().unwrap();
    assert_eq!(log.iter().filter(|r| r.starts_with("GET /variant/")).count(), usize::from(hydrate));
    response
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn complete_hit_normalization_selects_confirmed_citations_in_both_orders() {
    for (field, a, z, winner) in [
        ("cgi", r#""a""#, r#""z""#, "111"),
        ("cgi", r#"{"z":0,"a":1}"#, r#"{"a":2,"z":0}"#, "111"),
        ("cgi", r#"{"k":9,"k":1}"#, r#"{"k":2}"#, "111"),
        ("cgi", r#"{"k":1e2}"#, r#"{"k":11}"#, "111"),
        ("cgi", r#"{"k":0.12345678912345002}"#, r#"{"k":0.12345678912345003}"#, "222"),
        ("cadd", r#"{"phred":0.12345678912345002}"#, r#"{"phred":0.12345678912345003}"#, "222"),
    ] {
        for reversed in [false, true] {
            for hydrate in [false, true] {
            let hit = |rsid: &str, fragment: &str, citation: &str| {
                format!(r#"{{"_id":"chr1:g.101A>T","dbsnp":{{"rsid":"{rsid}"}},"{field}":{fragment},"civic":{{"molecularProfiles":[{{"name":"Profile","evidenceItems":[{{"source":{{"sourceType":"PUBMED","citation":"PMID:{citation}"}}}}]}}]}}}}"#)
            };
            let loser = if winner == "111" { "222" } else { "111" };
            let mut hits = vec![hit("rs101", a, winner), hit("rs101", z, loser), hit("rs000", r#""0""#, "333")];
            // CADD needs a valid decoy shape too.
            if field == "cadd" { hits[2] = hit("rs000", r#"{"phred":0}"#, "333"); }
            if reversed { hits.reverse(); }
            let hydrated = format!("[{}]", hits.join(","));
            let response = article_response(hydrated, hydrate).await;
            assert_eq!(
                response["results"].as_array().unwrap().len(),
                1,
                "{response}"
            );
            assert_eq!(response["results"][0]["pmid"], winner, "{response}");
            assert_eq!(
                response["results"][0]["provenance"][0]["route"], "source_citation",
                "{response}"
            );
            assert!(response["source_status"].as_array().unwrap().iter().any(
                |status| status["route"] == "source_citation"
                    && status["source"] == "myvariant"
                    && status["status"] == "ok"
            ));
        }
    }
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn opaque_civic_presence_controls_hydration_and_citation_provenance() {
    let civic = r#"{"molecularProfiles":[{"name":"Profile","evidenceItems":[{"source":{"sourceType":"PUBMED","citation":"PMID:111"}},{"source":{"sourceType":"PUBMED","citation":222}},{"source":{"sourceType":"PUBMED","citation":" 003 "}},{"source":{"sourceType":"PUBMED","citation":"PMID:111"}},{"source":{"sourceType":"PUBMED","citation":0}},{"source":{"sourceType":"PUBMED","citation":-1}},{"source":{"sourceType":"PUBMED","citation":1.5}},{"source":{"sourceType":"PUBMED","citation":4294967296}},{"source":{"sourceType":"PUBMED","citation":" "}},{"source":{"sourceType":"OTHER","citation":"999"}}]}]}"#;
    for companion in [None, Some("null"), Some("{}"), Some("[]"), Some("false"), Some(r#""opaque""#), Some(civic)] {
        let hydrate = companion.is_none() || companion == Some("null");
        let opaque = companion.map(|c| format!(r#", "civic":{c}"#)).unwrap_or_default();
        let hit = format!(r#"{{"_id":"chr1:g.101A>T","dbsnp":{{"rsid":"rs101"}}{opaque}}}"#);
        // The helper uses a minimal query hit for hydration cases and the supplied complete hit otherwise.
        let response = article_response(format!("[{hit}]"), hydrate).await;
        let rows = response["results"].as_array().unwrap();
        if companion == Some(civic) {
            let mut pmids = rows.iter().map(|r| r["pmid"].as_str().unwrap()).collect::<Vec<_>>();
            pmids.sort();
            assert_eq!(pmids, ["111", "222", "3"]);
            for row in rows {
                assert_eq!(row["provenance"][0]["route"], "source_citation");
                assert_eq!(row["provenance"][0]["source"], "civic");
            }
        } else { assert!(rows.is_empty(), "{response}"); }
    }
}
