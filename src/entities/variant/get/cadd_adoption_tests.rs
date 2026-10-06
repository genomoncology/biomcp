//! Callable cards preserve source score and first consequence policy.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cadd_callable_detail_preserves_scores_first_selection_and_public_omission() {
    for (cadd, score, consequence) in [
        (None, None, None),
        (Some(Value::Null), None, None),
        (Some(json!({})), None, None),
        (Some(json!({"phred":null,"consequence":null})), None, None),
        (
            Some(json!({"phred":0,"consequence":" NON_SYNONYMOUS "})),
            Some(0.0),
            Some("missense_variant"),
        ),
        (
            Some(json!({"phred":32,"consequence":"nonsynonymous"})),
            Some(32.0),
            Some("missense_variant"),
        ),
        (
            Some(json!({"phred":-1,"consequence":"non-synonymous"})),
            Some(-1.0),
            Some("missense_variant"),
        ),
        (
            Some(json!({"consequence":"synonymous"})),
            None,
            Some("synonymous_variant"),
        ),
        (
            Some(json!({"consequence":" splice region "})),
            None,
            Some("splice_region"),
        ),
        (
            Some(json!({"consequence":["synonymous","non_synonymous"]})),
            None,
            Some("synonymous_variant"),
        ),
        (Some(json!({"consequence":[" ","synonymous"]})), None, None),
        (Some(json!({"consequence":""})), None, None),
        (Some(json!({"consequence":[]})), None, None),
    ] {
        let mut body = json!({"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE"}});
        if let Some(cadd) = cadd {
            body["cadd"] = cadd;
        }
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_string());
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                &serde_json::to_vec(&body).unwrap(),
            ))
        })
        .await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let actual =
            serde_json::to_value(variant::get("chr1:g.101A>T", &[]).await.unwrap()).unwrap();
        let mut expected = json!({"gene":"GENE","id":"chr1:g.101A>T","genome_build":"GRCh38",
            "section_outcomes":{
                "cancerhotspots":{"outcome":"not_requested","sources":[]},
                "cbioportal":{"outcome":"not_requested","sources":[]},
                "civic":{"outcome":"not_requested","sources":[]},
                "clinvar":{"outcome":"not_requested","sources":[]},
                "gwas":{"outcome":"not_requested","sources":[]},
                "population":{"outcome":"not_requested","sources":[]},
                "predict":{"outcome":"not_requested","sources":[]}}});
        if let Some(score) = score {
            expected["cadd_score"] = json!(score);
        }
        if let Some(consequence) = consequence {
            expected["consequence"] = json!(consequence);
        }
        assert_eq!(actual, expected);
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        for (request, assembly) in requests.iter().zip(["hg38", "hg19"]) {
            let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
            assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
            let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
            assert_eq!(url.path(), "/variant/chr1:g.101A%3ET");
            assert_eq!(
                url.query_pairs().into_owned().collect::<Vec<_>>(),
                [
                    (
                        "fields".into(),
                        include_str!("../resolution/point_oracles/GET_FIELDS.txt")
                            .trim_end()
                            .into()
                    ),
                    ("assembly".into(), assembly.into())
                ]
            );
            assert_eq!(request.split_once("\r\n\r\n").unwrap().1, "");
        }
    }
}
