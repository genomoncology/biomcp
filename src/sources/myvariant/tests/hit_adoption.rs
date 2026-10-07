//! Complete source ownership and product normalization.
use super::super::{MyVariantClient, MyVariantHit, VariantSearchParams};
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};

const NORMALIZED: &str = r#"{"_id":"x","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,"cosmic":null,"cgi":{"a":[false,"  raw  ",null],"z":1},"civic":[3,{"unknown":{"x":true}}],"snpeff":null}"#;

#[test]
fn shared_hit_keeps_original_fragments_and_normalizes_product_once() {
    let raw = r#"{"_id":"x","cgi":{"z":1,"a":[false,"  raw  ",null]},"civic":[3,{"unknown":{"x":true}}]}"#;
    let stream: MyVariantHit = serde_json::from_str(raw).unwrap();
    assert_eq!(
        stream.source().cgi_json().unwrap().get(),
        r#"{"z":1,"a":[false,"  raw  ",null]}"#
    );
    let value = MyVariantHit::from_value(serde_json::from_str(raw).unwrap()).unwrap();
    for hit in [stream, value] {
        assert_eq!(serde_json::to_string(&hit).unwrap(), NORMALIZED);
    }
    for (input, output) in [
        ("1", "1"),
        ("1.0", "1.0"),
        ("1e2", "100.0"),
        ("-0.0", "-0.0"),
        ("0.12345678912345002", "0.12345678912345003"),
        ("0.12345678912345003", "0.12345678912345004"),
    ] {
        for name in ["cgi", "civic"] {
            let raw = format!(r#"{{"_id":"x","{name}":{{"k":{input}}}}}"#);
            let stream: MyVariantHit = serde_json::from_str(&raw).unwrap();
            let original = if name == "cgi" {
                stream.source().cgi_json()
            } else {
                stream.source().civic_json()
            };
            assert_eq!(original.unwrap().get(), format!(r#"{{"k":{input}}}"#));
            for hit in [
                stream,
                MyVariantHit::from_value(serde_json::from_str(&raw).unwrap()).unwrap(),
            ] {
                assert!(
                    serde_json::to_string(&hit)
                        .unwrap()
                        .contains(&format!(r#""{name}":{{"k":{output}}}"#))
                );
            }
        }
    }
    let raw = r#"{"_id":"x","cgi":{"k":1,"k":2}}"#;
    let hit: MyVariantHit = serde_json::from_str(raw).unwrap();
    assert!(
        serde_json::to_string(&hit)
            .unwrap()
            .contains(r#""cgi":{"k":2}"#)
    );
}

#[test]
fn value_first_opaque_positions_keep_arity_and_independent_presence() {
    for length in 0..=14 {
        let mut positions = vec![serde_json::Value::Null; length];
        if length > 0 {
            positions[0] = serde_json::Value::String("x".into());
        }
        if length > 10 {
            positions[10] = serde_json::Value::Bool(false);
        }
        if length > 11 {
            positions[11] = serde_json::json!([]);
        }
        let hit = MyVariantHit::from_value(serde_json::Value::Array(positions));
        assert_eq!(hit.is_ok(), length == 12 || length == 13, "length {length}");
        if let Ok(hit) = hit {
            assert_eq!(hit.source().cgi_json().unwrap().get(), "false");
            assert_eq!(hit.source().civic_json().unwrap().get(), "[]");
            assert_eq!(
                serde_json::to_string(&hit).unwrap(),
                r#"{"_id":"x","cadd":null,"clinvar":null,"dbnsfp":null,"dbsnp":null,"gnomad_exome":null,"gnomad":null,"exac":null,"exac_nontcga":null,"cosmic":null,"cgi":false,"civic":[],"snpeff":null}"#
            );
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
async fn callable_hits_keep_stored_numeric_precision_and_independent_depth() {
    for (member, fragment, expected) in [
        (
            "cgi",
            "{\"k\":0.12345678912345002}",
            "\"cgi\":{\"k\":0.12345678912345003}",
        ),
        (
            "civic",
            "{\"k\":0.12345678912345002}",
            "\"civic\":{\"k\":0.12345678912345003}",
        ),
        (
            "cadd",
            "{\"phred\":0.12345678912345002}",
            "\"cadd\":{\"phred\":0.12345678912345003,\"consequence\":null}",
        ),
        (
            "gnomad_exome",
            "{\"af\":{\"af\":0.12345678912345002}}",
            "\"gnomad_exome\":{\"af\":{\"af\":0.12345678912345003,",
        ),
    ] {
        let raw = format!(r#"{{"_id":"x","{member}":{fragment}}}"#);
        let fixture = TestHttpFixture::spawn(move |request| {
            let body = if request.starts_with("GET /query?") {
                format!(r#"{{"total":17,"hits":[{raw}]}}"#)
            } else {
                raw.clone()
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
        let alias = client.search_gene_alias(&params()).await.unwrap();
        let query = client
            .query_with_fields(
                "dbnsfp.genename:BRAF",
                5,
                0,
                "_id,cgi,civic,cadd,gnomad_exome",
            )
            .await
            .unwrap();
        assert_eq!(alias.total, Some(17));
        assert_eq!(query.total, Some(17));
        let get = client.get("x", None).await.unwrap();
        let all = client.get_all("x").await.unwrap();
        for hit in search
            .hits
            .into_iter()
            .chain(alias.hits)
            .chain(query.hits)
            .chain([get])
            .chain(all)
        {
            let encoded = serde_json::to_string(&hit).unwrap();
            assert!(encoded.contains(expected), "{encoded}");
        }
    }
    for member in ["cgi", "civic"] {
        for (arrays, search_ok, direct_ok, array_ok) in [
            (124, true, true, true),
            (125, true, true, true),
            (126, true, true, false),
            (127, true, false, false),
            (128, false, false, false),
        ] {
            for array_body in [false, true] {
                let fragment = format!("{}false{}", "[".repeat(arrays), "]".repeat(arrays));
                let raw = format!(r#"{{"_id":"x","{member}":{fragment}}}"#);
                let fixture = TestHttpFixture::spawn(move |request| {
                    let body = if request.starts_with("GET /query?") {
                        format!(r#"{{"hits":[{raw}]}}"#)
                    } else if array_body {
                        format!("[{raw}]")
                    } else {
                        raw.clone()
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
                assert_eq!(
                    client.search(&params()).await.is_ok(),
                    search_ok,
                    "search {member} {arrays}"
                );
                let get_ok = if array_body { array_ok } else { direct_ok };
                assert_eq!(
                    client.get("x", None).await.is_ok(),
                    get_ok,
                    "get {member} {arrays}"
                );
                assert_eq!(
                    client.get_all("x").await.is_ok(),
                    get_ok,
                    "get_all {member} {arrays}"
                );
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn complete_hit_owned_errors_remain_private_in_callable_clients() {
    use crate::error::BioMcpError;
    fn private(error: &BioMcpError) {
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
            assert!(!diagnostic.contains("PRIVATE_HIT_MARKER"), "{diagnostic}");
            assert!(!diagnostic.contains("987654321"), "{diagnostic}");
            cause = error.source();
        }
    }
    for raw in [
        r#"{"_id":{"PRIVATE_HIT_MARKER":987654321}}"#,
        r#"{"_id":"x","cadd":{"phred":"PRIVATE_HIT_MARKER"}}"#,
        r#"{"_id":"x","cgi":1e999,"PRIVATE_HIT_MARKER":987654321}"#,
        r#"{"_id":"x","_id":"y","PRIVATE_HIT_MARKER":987654321}"#,
        r#"{"_id":"x","PRIVATE_HIT_MARKER":1,"PRIVATE_HIT_MARKER":987654321}"#,
    ] {
        let fixture = TestHttpFixture::spawn(move |request| {
            let body = if request.starts_with("GET /query?") {
                format!(r#"{{"hits":[{raw}]}}"#)
            } else {
                raw.to_owned()
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
        let unknown = raw.contains(r#""PRIVATE_HIT_MARKER":1,"#);
        let duplicate = raw.contains(r#""_id":"y""#);
        let search = client.search(&params()).await;
        if unknown {
            assert!(search.is_ok());
        } else {
            private(&search.unwrap_err());
        }
        // Initial whole-Value errors (including 1e999) keep their existing owner.
        if !raw.contains("1e999") {
            let get = client.get("x", None).await;
            let all = client.get_all("x").await;
            if unknown || duplicate {
                assert!(get.is_ok());
                assert!(all.is_ok());
            } else {
                private(&get.unwrap_err());
                private(&all.unwrap_err());
            }
        }
    }
    let hit: MyVariantHit = serde_json::from_str(
        r#"{"_id":"PRIVATE_HIT_MARKER","cgi":{"k":987654321},"civic":"PRIVATE_HIT_MARKER"}"#,
    )
    .unwrap();
    let debug = format!("{hit:?}");
    assert!(!debug.contains("PRIVATE_HIT_MARKER"));
    assert!(!debug.contains("987654321"));
}
