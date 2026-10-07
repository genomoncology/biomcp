//! Complete shared direct records survive the actual client and shipped channels.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

const ID: &str = "NC_000007.14:g.140753125T>C";
const SOURCE: &str = r#"{"data":{"variant":{"variant_id":"7-140753125-T-C",
"exome":{"allele_frequency":0.99,"ac":2,"an":10,"homozygote_count":1,"hemizygote_count":0,"filters":["AC0"],"faf95":{"popmax":0.12,"popmax_population":"nfe"},"populations":[{"id":"nfe","ac":1,"an":4,"homozygote_count":0,"hemizygote_count":0}]},
"genome":{"ac":3,"an":20,"homozygote_count":0,"hemizygote_count":1,"filters":["RF"],"faf95":{"popmax":0.07,"popmax_population":"afr"},"populations":[{"id":"afr","ac":3,"an":10,"homozygote_count":0,"hemizygote_count":1}]}}}}"#;
fn check_channel(text: &str, json_mode: bool, details: bool) {
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        let p = &card["population"];
        assert_eq!(p["status"], "data");
        assert_eq!(
            p["exome"],
            json!({"allele_frequency":0.2,"ac":2,"an":10,"homozygote_count":1,"hemizygote_count":0,"filters":["AC0"],"faf95":{"popmax":0.12,"popmax_population":"nfe"},"populations":[{"id":"nfe","allele_frequency":0.25,"ac":1,"an":4,"homozygote_count":0,"hemizygote_count":0}]})
        );
        assert_eq!(
            p["genome"],
            json!({"allele_frequency":0.15,"ac":3,"an":20,"homozygote_count":0,"hemizygote_count":1,"filters":["RF"],"faf95":{"popmax":0.07,"popmax_population":"afr"},"populations":[{"id":"afr","allele_frequency":0.3,"ac":3,"an":10,"homozygote_count":0,"hemizygote_count":1}]})
        );
        assert_eq!(
            card["section_outcomes"]["population"],
            json!({"outcome":"data","sources":["gnomAD v4"]})
        );
    } else {
        for literal in [
            "Exome highest observed population-row frequency: nfe (0.25; allele count 1 / allele number 4)",
            "Genome highest observed population-row frequency: afr (0.3; allele count 3 / allele number 10)",
            "0.12",
            "0.07",
        ] {
            assert!(text.contains(literal), "missing {literal:?}: {text}");
        }
        assert_eq!(text.contains("| nfe |"), details, "{text}");
    }
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn shared_direct_population_reaches_native_cli_and_typed_raw_mcp() {
    let fixture = TestHttpFixture::spawn(|request| {
        let bytes = if request.starts_with("POST /") {
            SOURCE.as_bytes()
        } else {
            br#"{"_id":"chr7:g.140753125T>C","dbnsfp":{"genename":"BRAF"}}"#
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", bytes))
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
        ("BIOMCP_GNOMAD_BASE", fixture.base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let native = super::get(ID, &["population".into()]).await.unwrap();
    let exome: &biodata::GnomadSequencingPopulationProjection =
        native.population.as_ref().unwrap().exome.as_ref().unwrap();
    assert_eq!(exome.allele_frequency(), Some(0.2));
    check_channel(&serde_json::to_string(&native).unwrap(), true, false);
    check_channel(
        &crate::render::markdown::variant_markdown(&native, &["population".into()]).unwrap(),
        false,
        false,
    );
    let harness = ContractHarness::new(
        std::env::var_os("BIOMCP_BIN").unwrap(),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for section in ["population", "population-details"] {
        for json_mode in [true, false] {
            let mut args = vec!["get", "variant", ID, section];
            if json_mode {
                args.push("--json");
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(envs.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
            check_channel(
                std::str::from_utf8(&output.stdout).unwrap(),
                json_mode,
                section == "population-details",
            );
            for tool in ["get", "biomcp"] {
                let arguments = if tool == "get" {
                    json!({"entity":"variant","id":ID,"sections":[section],"json":json_mode})
                } else {
                    json!({"command":format!("biomcp get variant '{ID}' {section}"),"json":json_mode})
                };
                let result = client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new(tool)
                            .with_arguments(arguments.as_object().unwrap().clone()),
                    )
                    .await
                    .unwrap();
                assert!(!result.is_error.unwrap_or_default());
                check_channel(
                    first_text(&result.content),
                    json_mode,
                    section == "population-details",
                );
            }
        }
    }
    client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn direct_population_section_admission_and_private_failures() {
    use std::sync::{Arc, Mutex};
    let one_sided = r#"{"data":{"variant":{"variant_id":"7-140753125-T-C","genome":{"ac":1,"an":0,"homozygote_count":0,"hemizygote_count":0,"filters":[],"populations":[{"id":"other","ac":0,"an":0,"homozygote_count":0,"hemizygote_count":0}]}}}}"#;
    let wrong_id = SOURCE.replace("7-140753125-T-C", "7-140753126-T-C");
    let malformed = SOURCE.replace("\"ac\":2", "\"ac\":\"private-provider-marker\"");
    let errors_with_data = SOURCE.replacen(
        "{\"data\":",
        "{\"errors\":[{\"message\":\"private-provider-marker\"}],\"data\":",
        1,
    );
    for (section, body, status) in [
        ("", SOURCE, "not_requested"),
        ("population", one_sided, "data"),
        (
            "population-details",
            r#"{"data":{"variant":null}}"#,
            "empty",
        ),
        (
            "all",
            r#"{"data":{"variant":{"variant_id":"7-140753125-T-C"}}}"#,
            "empty",
        ),
        (
            "population",
            r#"{"errors":[{"message":" Variant not found "},{"message":"VARIANT NOT FOUND"}],"data":{"variant":null}}"#,
            "empty",
        ),
        ("population", wrong_id.as_str(), "unavailable"),
        ("population", malformed.as_str(), "unavailable"),
        ("population", errors_with_data.as_str(), "unavailable"),
        (
            "population",
            r#"{"errors":[{}],"data":{"variant":null}}"#,
            "unavailable",
        ),
        (
            "population",
            r#"{"errors":[{"message":"Variant not found"},{"message":"private-provider-marker"}],"data":{"variant":null}}"#,
            "unavailable",
        ),
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let body = body.to_string();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_string());
            let bytes = if request.starts_with("POST /") {
                body.as_bytes()
            } else {
                br#"{"_id":"chr7:g.140753125T>C"}"#
            };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", bytes))
        })
        .await;
        let mut env = TestEnv::new();
        for key in [
            "BIOMCP_MYVARIANT_BASE",
            "BIOMCP_GNOMAD_BASE",
            "BIOMCP_CLINVAR_BASE",
            "BIOMCP_CBIOPORTAL_BASE",
            "BIOMCP_CIVIC_BASE",
            "BIOMCP_CANCERHOTSPOTS_BASE",
            "BIOMCP_GWAS_BASE",
            "BIOMCP_TEST_UNPACED_ORIGIN",
        ] {
            env.set(key, &fixture.base);
        }
        env.set("BIOMCP_CACHE_MODE", "off");
        let sections = if section.is_empty() {
            vec![]
        } else {
            vec![section.into()]
        };
        let native = super::get(ID, &sections).await.unwrap();
        let card = serde_json::to_value(&native).unwrap();
        let markdown = crate::render::markdown::variant_markdown(&native, &sections).unwrap();
        assert!(!card.to_string().contains("private-provider-marker"));
        assert!(!markdown.contains("private-provider-marker"));
        let log = requests.lock().unwrap();
        let post_count = log
            .iter()
            .filter(|request| request.starts_with("POST /"))
            .count();
        assert_eq!(
            post_count,
            usize::from(!section.is_empty()),
            "{section}: {log:?}"
        );
        if section.is_empty() {
            assert!(native.population.is_none());
        } else {
            assert_eq!(card["population"]["status"], status, "{section}: {card}");
            assert_eq!(card["section_outcomes"]["population"]["outcome"], status);
            assert!(
                log.iter()
                    .any(|request| request.contains("\"variantId\":\"7-140753125-T-C\""))
            );
            if status == "data" {
                assert_eq!(card["population"]["exome"], Value::Null);
                assert_eq!(
                    card["population"]["genome"]["allele_frequency"],
                    Value::Null
                );
                assert_eq!(
                    card["population"]["genome"]["populations"][0]["allele_frequency"],
                    Value::Null
                );
            } else {
                assert_eq!(card["population"]["exome"], Value::Null);
                assert_eq!(card["population"]["genome"], Value::Null);
            }
        }
    }
}
