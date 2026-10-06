//! Callable cards preserve COSMIC ID, first context and section policy.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::entities::variant;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

#[tokio::test]
#[serial_test::serial(source_env)]
async fn cosmic_callable_detail_preserves_id_context_and_section_policy() {
    for (cosmic, id, context, sections) in [
        (None, None, None, vec!["cosmic"]),
        (Some(Value::Null), None, None, vec!["cosmic"]),
        (Some(json!({})), None, None, vec!["cosmic"]),
        (
            Some(json!({"cosmic_id":null,"mut_freq":null,"tumor_site":null,"mut_nt":null})),
            None,
            None,
            vec!["cosmic"],
        ),
        (
            Some(json!({"cosmic_id":" ID "})),
            Some("ID"),
            None,
            vec!["cosmic"],
        ),
        (
            Some(
                json!({"cosmic_id":[" "," ID ","OTHER"],"tumor_site":" site ","mut_nt":" c.1A>T "}),
            ),
            Some("ID"),
            Some(json!({"tumor_site":"site","mut_nt":"c.1A>T"})),
            vec!["cosmic"],
        ),
        (
            Some(
                json!({"cosmic_id":" ","tumor_site":["site","later"],"mut_nt":["c.1A>T","later"]}),
            ),
            None,
            Some(json!({"tumor_site":"site","mut_nt":"c.1A>T"})),
            vec!["cosmic"],
        ),
        (
            Some(json!({"cosmic_id":[],"tumor_site":[" ","site"],"mut_nt":["","c.1A>T"]})),
            None,
            None,
            vec!["cosmic"],
        ),
        (
            Some(json!({"cosmic_id":"","tumor_site":[],"mut_nt":" "})),
            None,
            None,
            vec!["cosmic"],
        ),
        (
            Some(json!({"mut_freq":0})),
            None,
            Some(json!({"mut_freq":0.0})),
            vec!["cosmic"],
        ),
        (
            Some(json!({"mut_freq":-1})),
            None,
            Some(json!({"mut_freq":-1.0})),
            vec!["cosmic"],
        ),
        (
            Some(json!({"tumor_site":" site ","mut_freq":null})),
            None,
            Some(json!({"tumor_site":"site"})),
            vec!["cosmic"],
        ),
        (
            Some(json!({"mut_nt":" c.1A>T "})),
            None,
            Some(json!({"mut_nt":"c.1A>T"})),
            vec!["cosmic"],
        ),
        (
            Some(json!({"cosmic_id":" ID ","mut_freq":0,"tumor_site":"site"})),
            Some("ID"),
            None,
            vec![],
        ),
        (
            Some(json!({"cosmic_id":" ID ","mut_freq":0,"tumor_site":"site"})),
            Some("ID"),
            Some(json!({"mut_freq":0.0,"tumor_site":"site"})),
            vec!["all"],
        ),
    ] {
        let mut body = json!({"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE"}});
        if let Some(cosmic) = cosmic {
            body["cosmic"] = cosmic;
        }
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_string());
            if !request.starts_with("GET /variant/") {
                return TestHttpReply::Bytes(test_http_response(
                    "400 Bad Request",
                    "application/json",
                    b"{}",
                ));
            }
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
        env.set("BIOMCP_GNOMAD_BASE", &fixture.base);
        env.set("BIOMCP_CBIOPORTAL_BASE", &fixture.base);
        let actual = serde_json::to_value(
            variant::get(
                "chr1:g.101A>T",
                &sections.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
            )
            .await
            .unwrap(),
        )
        .unwrap();
        let mut expected = json!({"gene":"GENE","id":"chr1:g.101A>T","genome_build":"GRCh38",
            "section_outcomes":{
                "cancerhotspots":{"outcome":"not_requested","sources":[]},
                "cbioportal":{"outcome":"not_requested","sources":[]},
                "civic":{"outcome":"not_requested","sources":[]},
                "clinvar":{"outcome":"not_requested","sources":[]},
                "gwas":{"outcome":"not_requested","sources":[]},
                "population":{"outcome":"not_requested","sources":[]},
                "predict":{"outcome":"not_requested","sources":[]}}});
        if let Some(id) = id {
            expected["cosmic_id"] = json!(id);
        }
        if let Some(context) = context {
            expected["cosmic_context"] = context;
        }
        if sections == ["all"] {
            expected["supporting_pmids"] = json!([]);
            expected["population"] = json!({"status":"unavailable","dataset":"gnomad_r4","release":"gnomAD v4","message":"gnomAD population data is temporarily unavailable.","exome":null,"genome":null,"faf_caveat":"gnomAD excludes bottlenecked genetic ancestry groups when selecting grpmax FAF."});
            expected["section_outcomes"] = json!({
                "cancerhotspots":{"outcome":"inapplicable","sources":[],"message":"A gene and normalizable protein change are required for Cancer Hotspots."},
                "cbioportal":{"outcome":"unavailable","sources":[],"message":"Requested variant source data is temporarily unavailable."},
                "civic":{"outcome":"inapplicable","sources":[],"message":"A gene and protein change are required for clinical evidence lookup."},
                "clinvar":{"outcome":"inapplicable","sources":[],"message":"Direct ClinVar retrieval requires a resolved numeric Variation ID."},
                "gwas":{"outcome":"inapplicable","sources":[],"message":"An rsID is required for GWAS associations."},
                "population":{"outcome":"unavailable","sources":[],"message":"gnomAD population data is temporarily unavailable."},
                "predict":{"outcome":"not_requested","sources":[]}});
        }
        assert_eq!(actual, expected);
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), if sections == ["all"] { 4 } else { 2 });
        if sections == ["all"] {
            assert!(requests[2].starts_with("POST / HTTP/1.1\r\n"));
            assert!(
                requests[3]
                    .starts_with("GET /genes?keyword=GENE&pageSize=1&pageNumber=0 HTTP/1.1\r\n")
            );
        }
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
