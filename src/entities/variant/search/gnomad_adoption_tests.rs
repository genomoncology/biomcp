//! Callable search owns AF object precedence, ranking and external presence.
use super::exact_scan_tests::fixture;
use super::*;
use crate::entities::article::test_support::TestEnv;
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

fn expected_row(id: &str, af: Option<f64>) -> Value {
    let mut row = json!({"id":id,"genome_build":"GRCh37",
        "genome_build_provenance":"MyVariant.info provider default","gene":"BRAF",
        "clinvar_stars":null,"revel":null,"gerp":null});
    if let Some(af) = af {
        row["gnomad_af"] = json!(af);
    }
    row
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn gnomad_search_selects_objects_before_reading_frequency() {
    for (members, af) in [
        (
            json!({"gnomad_exome":{"af":{"af":0.1}},"gnomad":{"exomes":{"af":{"af":0.2}},"genomes":{"af":{"af":0.3}}}}),
            Some(0.1),
        ),
        (
            json!({"gnomad":{"exomes":{"af":{"af":0.2}},"genomes":{"af":{"af":0.3}}}}),
            Some(0.2),
        ),
        (
            json!({"gnomad_exome":{"af":null},"gnomad":{"exomes":{"af":{"af":0.2}}}}),
            Some(0.2),
        ),
        (json!({"gnomad":{"genomes":{"af":{"af":0.3}}}}), Some(0.3)),
        (
            json!({"gnomad":{"exomes":{"af":null},"genomes":{"af":{"af":0.3}}}}),
            Some(0.3),
        ),
        (
            json!({"gnomad_exome":{"af":{}},"gnomad":{"exomes":{"af":{"af":0.2}},"genomes":{"af":{"af":0.3}}}}),
            None,
        ),
        (
            json!({"gnomad_exome":{"af":{"af":null}},"gnomad":{"exomes":{"af":{"af":0.2}}}}),
            None,
        ),
        (
            json!({"gnomad":{"exomes":{"af":{}},"genomes":{"af":{"af":0.3}}}}),
            None,
        ),
        (
            json!({"gnomad_exome":{"af":{"af":0.0}},"gnomad":{"exomes":{"af":{"af":0.2}}}}),
            Some(0.0),
        ),
        (json!({}), None),
    ] {
        let mut row = members;
        row["_id"] = json!("chr7:g.101A>T");
        row["dbnsfp"] = json!({"genename":"BRAF"});
        let (fixture, requests) = fixture(vec![json!({"total":1,"hits":[row]})]).await;
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
        assert_eq!(page.results[0].gnomad_af, af);
        let encoded = serde_json::to_value(&page.results).unwrap();
        assert_eq!(encoded, json!([expected_row("chr7:g.101A>T", af)]));
        assert_eq!(encoded[0].get("gnomad_af").is_some(), af.is_some());
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
                ),
            ]
        );
        assert_eq!(log[0].split_once("\r\n\r\n").unwrap().1, "");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn gnomad_frequency_alone_promotes_a_later_id_and_preserves_ties() {
    for (first_af, ids) in [
        (None, ["chr7:g.102A>T", "chr7:g.101A>T"]),
        (Some(0.2), ["chr7:g.101A>T", "chr7:g.102A>T"]),
    ] {
        let mut first = json!({"_id":"chr7:g.101A>T","dbnsfp":{"genename":"BRAF"}});
        if let Some(af) = first_af {
            first["gnomad_exome"] = json!({"af":{"af":af}});
        }
        let second = json!({"_id":"chr7:g.102A>T","dbnsfp":{"genename":"BRAF"},"gnomad_exome":{"af":{"af":0.0}}});
        let (fixture, _) = fixture(vec![json!({"total":2,"hits":[first,second]})]).await;
        let mut env = TestEnv::new();
        env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_CACHE_MODE", "off");
        let page = search_page(
            &VariantSearchFilters {
                gene: Some("BRAF".into()),
                ..Default::default()
            },
            2,
            0,
        )
        .await
        .unwrap();
        assert_eq!(
            page.results
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(page.total, Some(2));
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn gnomad_exact_search_keeps_selected_frequency_and_page_identity() {
    let mut row = super::exact_scan_tests::hit();
    row["gnomad"] = json!({"genomes":{"af":{"af":0.3}}});
    let (fixture, requests) = fixture(vec![json!({"total":1,"hits":[row]})]).await;
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_MODE", "off");
    let filters = super::exact_scan_tests::filters();
    let page = search_page(&filters, 1, 0).await.unwrap();
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].gnomad_af, Some(0.3));
    assert_eq!(page.results[0].id, "chr7:g.101A>T");
    assert_eq!(page.requested_variant, filters.requested_identity);
    assert_eq!(page.total, Some(1));
    assert_eq!(page.has_more, Some(false));
    let resolution = page.resolution.unwrap();
    assert_eq!(resolution.status, VariantResolutionStatus::Resolved);
    assert!(resolution.exhaustive);
    assert!(page.diagnostics.is_empty());
    super::exact_scan_tests::ledger(
        &requests,
        &[0],
        "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.Val600Glu\"",
    );
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn gnomad_cli_zero_and_typed_mcp_absence_preserve_json_contracts() {
    let harness = ContractHarness::new(
        std::path::PathBuf::from(std::env::var_os("BIOMCP_BIN").unwrap()),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    for (channel, af) in [("cli", Some(0.0)), ("typed", None)] {
        let mut row = json!({"_id":"chr7:g.101A>T","dbnsfp":{"genename":"BRAF"}});
        row["gnomad_exome"] = if af.is_some() {
            json!({"af":{"af":0.0}})
        } else {
            json!({"af":{}})
        };
        row["gnomad"] = json!({"exomes":{"af":{"af":0.2}}});
        let (fixture, requests) = fixture(vec![json!({"total":1,"hits":[row]})]).await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        let value = if channel == "cli" {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(["search", "variant", "-g", "BRAF", "--limit", "1", "--json"])
                .envs(env)
                .output()
                .await
                .unwrap();
            assert_eq!(output.status.code(), Some(0));
            assert!(output.stderr.is_empty());
            assert_eq!(output.stdout.last(), Some(&b'\n'));
            serde_json::from_slice::<Value>(&output.stdout).unwrap()
        } else {
            let client = harness.spawn_stdio_client(&env).await.unwrap();
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new("search").with_arguments(
                        json!({"entity":"variant","gene":"BRAF","limit":1,"json":true})
                            .as_object()
                            .unwrap()
                            .clone(),
                    ),
                )
                .await
                .unwrap();
            let text = first_text(&result.content);
            assert_eq!(
                serde_json::to_value(&result).unwrap(),
                json!({"content":[{"type":"text","text":text}],"isError":false})
            );
            let value = serde_json::from_str::<Value>(text).unwrap();
            client.cancel().await.unwrap();
            value
        };
        assert_eq!(value["results"], json!([expected_row("chr7:g.101A>T", af)]));
        assert_eq!(value["results"][0].get("gnomad_af").is_some(), af.is_some());
        assert_eq!(value["count"], 1);
        assert_eq!(
            value["pagination"],
            json!({"offset":0,"limit":1,"returned":1,"total":1,"has_more":false,"next_page_token":null})
        );
        assert!(value["_meta"]["next_commands"].is_array());
        assert_eq!(requests.lock().unwrap().len(), 1);
    }
}
