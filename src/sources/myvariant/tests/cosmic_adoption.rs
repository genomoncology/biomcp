//! Shared COSMIC embedding, callable clients and retained product errors.
use super::super::{MyVariantClient, MyVariantHit, MyVariantSearchResponse, VariantSearchParams};
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::error::BioMcpError;

const EXPECTED: &str = r#"{"_id":"safe-id","cadd":{"phred":0.0,"consequence":["","synonymous"]},"clinvar":null,"dbnsfp":null,"dbsnp":{"rsid":"rs123"},"gnomad_exome":null,"gnomad":null,"exac":{"af":0.1},"exac_nontcga":{"af":0.2},"cosmic":{"cosmic_id":"COSM1","mut_freq":null,"tumor_site":["","skin"],"mut_nt":null},"cgi":{"opaque":true},"civic":{"opaque":[]},"snpeff":null}"#;

const SECOND: &str = r#"{"_id":"second-id","cadd":{"phred":0.0,"consequence":["","synonymous"]},"clinvar":null,"dbnsfp":null,"dbsnp":{"rsid":"rs123"},"gnomad_exome":null,"gnomad":null,"exac":{"af":0.1},"exac_nontcga":{"af":0.2},"cosmic":{"cosmic_id":["","COSM2","COSM2"],"mut_freq":2.83,"tumor_site":["","skin"],"mut_nt":null},"cgi":{"opaque":true},"civic":{"opaque":[]},"snpeff":null}"#;

#[test]
fn cosmic_preserves_complete_hit_order_presence_and_carriers() {
    let positional = r#"{"_id":"safe-id","cadd":[0,["","synonymous"]],"dbsnp":{"rsid":"rs123"},"exac":[0.1],"exac_nontcga":[0.2],"cosmic":["COSM1",null,["","skin"]],"cgi":{"opaque":true},"civic":{"opaque":[]},"ignored":true}"#;
    let list = EXPECTED
        .replace(
            r#""cosmic_id":"COSM1""#,
            r#""cosmic_id":[""," ID "," ID "]"#,
        )
        .replace(r#""mut_freq":null"#, r#""mut_freq":-1.0"#)
        .replace(r#""mut_nt":null"#, r#""mut_nt":[" c.1A>T "]"#);
    for (raw, expected) in [
        (EXPECTED, EXPECTED),
        (positional, EXPECTED),
        (list.as_str(), list.as_str()),
    ] {
        let bytes: MyVariantHit = serde_json::from_slice(raw.as_bytes()).unwrap();
        let value: MyVariantHit =
            MyVariantHit::from_value(serde_json::from_str(raw).unwrap()).unwrap();
        for hit in [bytes, value] {
            assert_eq!(serde_json::to_string(&hit.clone()).unwrap(), expected);
        }
    }
    for (member, cosmic) in [
        ("", "null"),
        (r#", "cosmic":null"#, "null"),
        (
            r#", "cosmic":{}"#,
            r#"{"cosmic_id":null,"mut_freq":null,"tumor_site":null,"mut_nt":null}"#,
        ),
        (
            r#", "cosmic":{"cosmic_id":"","mut_freq":null,"tumor_site":[],"mut_nt":null}"#,
            r#"{"cosmic_id":"","mut_freq":null,"tumor_site":[],"mut_nt":null}"#,
        ),
    ] {
        let raw = format!(r#"{{"_id":"safe-id"{member}}}"#);
        let expected = format!(
            r#"{{"_id":"safe-id","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,"cosmic":{cosmic},"cgi":null,"civic":null,"snpeff":null}}"#
        );
        for hit in [
            serde_json::from_str::<MyVariantHit>(&raw).unwrap(),
            MyVariantHit::from_value(serde_json::from_str(&raw).unwrap()).unwrap(),
        ] {
            assert_eq!(serde_json::to_string(&hit).unwrap(), expected);
        }
    }
    for field in ["cosmic_id", "mut_freq", "tumor_site", "mut_nt"] {
        let raw = format!(r#"{{"_id":"safe-id","cosmic":{{"{field}":null,"{field}":null}}}}"#);
        assert!(serde_json::from_str::<MyVariantHit>(&raw).is_err());
        let erased: MyVariantHit =
            MyVariantHit::from_value(serde_json::from_str(&raw).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_string(&erased).unwrap(),
            r#"{"_id":"safe-id","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,"cosmic":{"cosmic_id":null,"mut_freq":null,"tumor_site":null,"mut_nt":null},"cgi":null,"civic":null,"snpeff":null}"#
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
        assert!(
            !diagnostic.contains("PRIVATE_COSMIC_MARKER"),
            "{diagnostic}"
        );
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
async fn cosmic_owned_failures_are_private_through_search_get_and_get_all() {
    for owned in [
        r#"{"mut_freq":"PRIVATE_COSMIC_MARKER"}"#,
        "987654321",
        r#"{"cosmic_id":{"PRIVATE_COSMIC_MARKER":987654321}}"#,
        r#"{"tumor_site":{"PRIVATE_COSMIC_MARKER":987654321}}"#,
        r#"{"mut_nt":{"PRIVATE_COSMIC_MARKER":987654321}}"#,
        r#"{"mut_freq":null,"mut_freq":null}"#,
        r#"{"cosmic_id":null,"cosmic_id":null}"#,
        r#"{"tumor_site":null,"tumor_site":null}"#,
        r#"{"mut_nt":null,"mut_nt":null}"#,
    ] {
        let raw = format!(r#"{{"_id":"safe-id","cosmic":{owned}}}"#);
        private_error(&decode::<MyVariantHit>(&raw).unwrap_err());
        let search = format!(r#"{{"total":1,"hits":[{raw}]}}"#);
        private_error(&decode::<MyVariantSearchResponse>(&search).unwrap_err());
        let duplicate = owned.matches(":null").count() == 2;
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
        if duplicate {
            assert!(client.get("safe-id", None).await.is_ok());
            assert_eq!(client.get_all("safe-id").await.unwrap().len(), 1);
        } else {
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
async fn cosmic_callable_clients_preserve_rows_masks_and_canonical_hits() {
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
    let cosmic: &biodata::MyVariantCosmicProjection = first.source().cosmic().unwrap();
    assert_eq!(cosmic.cosmic_id().unwrap().first(), Some("COSM1"));
    assert_eq!(cosmic.tumor_site().unwrap().values(), &["", "skin"]);
    assert_eq!(cosmic.mut_freq(), None);
    assert_eq!(cosmic.mut_nt(), None);
    assert_eq!(serde_json::to_string(&first).unwrap(), EXPECTED);
    for hits in [search.hits, rows] {
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].source().cosmic().unwrap().mut_freq(), Some(2.83));
        assert_eq!(
            hits[1]
                .source()
                .cosmic()
                .unwrap()
                .cosmic_id()
                .unwrap()
                .values(),
            &["", "COSM2", "COSM2"]
        );
        assert_eq!(serde_json::to_string(&hits[0]).unwrap(), EXPECTED);
        assert_eq!(serde_json::to_string(&hits[1]).unwrap(), SECOND);
    }
}
