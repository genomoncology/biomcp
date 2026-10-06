//! Shared ExAC pair embedding, callable clients and retained product errors.
use super::super::{MyVariantClient, MyVariantHit, MyVariantSearchResponse, VariantSearchParams};
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::error::BioMcpError;

const EXPECTED: &str = r#"{"_id":"safe-id","cadd":{"phred":0.0,"consequence":["","synonymous"]},"clinvar":null,"dbnsfp":null,"dbsnp":{"rsid":"rs123"},"gnomad_exome":null,"gnomad":null,"exac":{"af":0.1},"exac_nontcga":{"af":0.2},"cosmic":{"cosmic_id":"COSM1","mut_freq":null,"tumor_site":["","skin"],"mut_nt":null},"cgi":{"opaque":true},"civic":{"opaque":[]},"snpeff":null}"#;

const SECOND: &str = r#"{"_id":"second-id","cadd":{"phred":0.0,"consequence":["","synonymous"]},"clinvar":null,"dbnsfp":null,"dbsnp":{"rsid":"rs123"},"gnomad_exome":null,"gnomad":null,"exac":{"af":0.3},"exac_nontcga":{"af":0.2},"cosmic":{"cosmic_id":"COSM1","mut_freq":null,"tumor_site":["","skin"],"mut_nt":null},"cgi":{"opaque":true},"civic":{"opaque":[]},"snpeff":null}"#;

#[test]
fn exac_pair_preserves_complete_hit_order_and_independent_presence() {
    let positional = r#"{"_id":"safe-id","cadd":[0,["","synonymous"]],"dbsnp":{"rsid":"rs123"},"exac":[0.1],"exac_nontcga":[0.2],"cosmic":{"cosmic_id":"COSM1","tumor_site":["","skin"]},"cgi":{"opaque":true},"civic":{"opaque":[]},"ignored":true}"#;
    for raw in [EXPECTED, positional] {
        let bytes: MyVariantHit = serde_json::from_slice(raw.as_bytes()).unwrap();
        let value: MyVariantHit =
            serde_json::from_value(serde_json::from_str(raw).unwrap()).unwrap();
        for hit in [bytes, value] {
            assert_eq!(serde_json::to_string(&hit.clone()).unwrap(), EXPECTED);
        }
    }
    for (members, exac, non_tcga) in [
        ("", "null", "null"),
        (r#", "exac":null, "exac_nontcga":null"#, "null", "null"),
        (
            r#", "exac":{}, "exac_nontcga":{"af":0}"#,
            r#"{"af":null}"#,
            r#"{"af":0.0}"#,
        ),
        (
            r#", "exac":{"af":0}, "exac_nontcga":{}"#,
            r#"{"af":0.0}"#,
            r#"{"af":null}"#,
        ),
        (
            r#", "exac":{}, "exac_nontcga":null"#,
            r#"{"af":null}"#,
            "null",
        ),
        (
            r#", "exac":null, "exac_nontcga":{}"#,
            "null",
            r#"{"af":null}"#,
        ),
    ] {
        let raw = format!(r#"{{"_id":"safe-id"{members}}}"#);
        let expected = format!(
            r#"{{"_id":"safe-id","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":{exac},"exac_nontcga":{non_tcga},"cosmic":null,"cgi":null,"civic":null,"snpeff":null}}"#
        );
        for hit in [
            serde_json::from_str::<MyVariantHit>(&raw).unwrap(),
            serde_json::from_value(serde_json::from_str(&raw).unwrap()).unwrap(),
        ] {
            assert_eq!(serde_json::to_string(&hit).unwrap(), expected);
        }
    }
    for member in ["exac", "exac_nontcga"] {
        assert!(
            serde_json::from_str::<MyVariantHit>(&format!(
                r#"{{"_id":"safe-id","{member}":{{"af":null,"af":0}}}}"#
            ))
            .is_err()
        );
    }
}

fn private_error(error: &BioMcpError) {
    let underlying = match error {
        BioMcpError::WithSourceContext { context, source } => {
            assert_eq!(context.provider().label(), "MyVariant.info");
            source.as_ref()
        }
        error => error,
    };
    assert!(
        matches!(underlying, BioMcpError::ApiJson { api, .. } if api.eq_ignore_ascii_case("myvariant.info"))
    );
    let mut cause: Option<&dyn std::error::Error> = Some(error);
    while let Some(error) = cause {
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("PRIVATE_EXAC_MARKER"), "{diagnostic}");
        assert!(!diagnostic.contains("987654321"), "{diagnostic}");
        cause = error.source();
    }
}

fn decode<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, BioMcpError> {
    crate::sources::decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        reqwest::StatusCode::OK,
        Some(&reqwest::header::HeaderValue::from_static(
            "application/json",
        )),
        raw.as_bytes(),
        true,
    )
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn exac_pair_owned_failures_are_private_through_search_get_and_get_all() {
    for member in ["exac", "exac_nontcga"] {
        for owned in [r#"{"af":"PRIVATE_EXAC_MARKER"}"#, "987654321"] {
            let raw = format!(r#"{{"_id":"safe-id","{member}":{owned}}}"#);
            private_error(&decode::<MyVariantHit>(&raw).unwrap_err());
            let search = format!(r#"{{"total":1,"hits":[{raw}]}}"#);
            private_error(&decode::<MyVariantSearchResponse>(&search).unwrap_err());
            let fixture = TestHttpFixture::spawn(move |request| {
                let body = if request.starts_with("GET /query?") {
                    &search
                } else {
                    &raw
                };
                TestHttpReply::Bytes(test_http_response(
                    "200 OK",
                    "application/json",
                    body.as_bytes(),
                ))
            })
            .await;
            let mut env = TestEnv::new();
            env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
            env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
            env.set("BIOMCP_CACHE_MODE", "off");
            let client = MyVariantClient::new().unwrap();
            private_error(&client.search(&params()).await.unwrap_err());
            private_error(&client.get("safe-id", None).await.unwrap_err());
            private_error(&client.get_all("safe-id").await.unwrap_err());
        }
    }
}

fn params() -> VariantSearchParams {
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

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn exac_pair_callable_clients_preserve_rows_masks_and_canonical_hits() {
    let fixture = TestHttpFixture::spawn(move |request| {
        let line = request.lines().next().unwrap();
        let target = line.split_whitespace().nth(1).unwrap();
        let url = reqwest::Url::parse(&format!("http://localhost{target}")).unwrap();
        let pairs: std::collections::BTreeMap<_, _> = url.query_pairs().into_owned().collect();
        let body = if url.path() == "/query" {
            assert_eq!(
                pairs.get("fields").unwrap(),
                super::super::MYVARIANT_FIELDS_SEARCH
            );
            assert_eq!(pairs.get("size").unwrap(), "5");
            assert_eq!(pairs.get("from").unwrap(), "0");
            assert_eq!(pairs.get("q").unwrap(), "dbnsfp.genename:BRAF");
            format!(r#"{{"total":17,"hits":[{EXPECTED},{SECOND}]}}"#)
        } else {
            assert_eq!(url.path(), "/variant/safe-id");
            assert_eq!(
                pairs.get("fields").unwrap(),
                super::super::MYVARIANT_FIELDS_GET
            );
            format!("[{EXPECTED},{SECOND}]")
        };
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/json",
            body.as_bytes(),
        ))
    })
    .await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let client = MyVariantClient::new().unwrap();
    let search = client.search(&params()).await.unwrap();
    assert_eq!(search.total, Some(17));
    let rows = client.get_all("safe-id").await.unwrap();
    let first = client.get("safe-id", None).await.unwrap();
    assert_eq!(first.exac.as_ref().unwrap().af(), Some(0.1));
    assert_eq!(first.exac_nontcga.as_ref().unwrap().af(), Some(0.2));
    assert_eq!(serde_json::to_string(&first).unwrap(), EXPECTED);
    for hits in [search.hits, rows] {
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].exac.as_ref().unwrap().af(), Some(0.3));
        assert_eq!(hits[1].exac_nontcga.as_ref().unwrap().af(), Some(0.2));
        assert_eq!(serde_json::to_string(&hits[0]).unwrap(), EXPECTED);
        assert_eq!(serde_json::to_string(&hits[1]).unwrap(), SECOND);
    }
}
