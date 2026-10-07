//! Actual constraint retrieval, product decoding and shipped channel contracts.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn expected_constraint(transcript: Option<&str>, metrics: bool) -> Value {
    let mut value = json!({"source":"gnomAD","source_version":"v4","reference_genome":"GRCh38"});
    if let Some(transcript) = transcript {
        value["transcript"] = json!(transcript);
    }
    if metrics {
        for (key, number) in [
            ("pli", 0.9979),
            ("loeuf", 0.449),
            ("mis_z", 1.1539),
            ("syn_z", 0.9583),
        ] {
            value[key] = json!(number);
        }
    }
    value
}

fn check_output(
    text: &str,
    json_mode: bool,
    state: &str,
    transcript: Option<&str>,
    metrics: bool,
    meta: bool,
) {
    assert!(!text.contains("PRIVATE-CANARY"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["constraint"], expected_constraint(transcript, metrics));
        assert_eq!(card["section_outcomes"]["constraint"]["outcome"], state);
        let outcome = &card["section_outcomes"]["constraint"];
        if state == "unavailable" {
            assert_eq!(outcome["message"], "gnomAD gene constraint is unavailable.");
            assert!(outcome.get("sources").is_none() || outcome["sources"] == json!([]));
        } else {
            assert_eq!(outcome["sources"], json!(["gnomAD"]));
        }
        if meta {
            let sources = card["_meta"]["section_sources"].as_array().unwrap();
            let constraint = sources.iter().find(|source| source["key"] == "constraint");
            assert!(constraint.is_some(), "{text}");
            if let Some(source) = constraint {
                assert_eq!(source["outcome"], state);
                assert_eq!(
                    source["sources"],
                    if state == "unavailable" {
                        json!([])
                    } else {
                        json!(["gnomAD"])
                    }
                );
            }
        }
    } else {
        assert!(text.contains("# TP53 - constraint"), "{text}");
        assert!(text.contains("## Constraint (gnomAD)"));
        if state == "unavailable" {
            assert!(
                text.contains("gnomAD gene constraint is unavailable."),
                "{text}"
            );
            assert!(!text.contains("Transcript:"));
        } else {
            for label in ["Source: gnomAD", "Version: v4", "Reference genome: GRCh38"] {
                assert!(text.contains(label), "{text}");
            }
            if let Some(transcript) = transcript {
                assert!(text.contains(transcript), "{text}");
            }
            if metrics {
                for label in ["pLI: 0.998", "LOEUF: 0.449", "mis_z: 1.154", "syn_z: 0.958"] {
                    assert!(text.contains(label), "{text}");
                }
            }
        }
    }
}

#[test]
fn flattened_gene_constraint_preserves_literal_target_and_omission() {
    for fields in [
        json!({"transcript":"  ","pli":0.0,"loeuf":-2.0,"mis_z":3.0,"syn_z":4.0}),
        json!({"transcript":"","pli":null,"loeuf":null,"mis_z":null,"syn_z":null}),
        json!({}),
    ] {
        let mut target = fields.clone();
        for (key, value) in expected_constraint(None, false).as_object().unwrap() {
            target[key] = value.clone();
        }
        target["canonical_transcript_id"] = json!("PRIVATE-CANARY");
        target["pLI"] = json!(99);
        let gene: Gene = serde_json::from_value(json!({
            "symbol":"TP53","name":"tumor protein p53","entrez_id":"7157",
            "aliases":[],"clinical_diseases":[],"clinical_drugs":[],"constraint":target
        }))
        .unwrap();
        let encoded = serde_json::to_value(&gene).unwrap();
        let mut expected = expected_constraint(None, false);
        for (key, value) in fields.as_object().unwrap() {
            if !value.is_null() {
                expected[key] = value.clone();
            }
        }
        assert_eq!(encoded["constraint"], expected);
        let text = crate::render::markdown::gene_markdown(&gene, &["constraint".into()]).unwrap();
        assert!(!text.contains("PRIVATE-CANARY"));
        assert!(text.contains("## Constraint (gnomAD)"));
    }
    assert!(serde_json::from_value::<GeneConstraint>(json!({"pli":1})).is_err());
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn direct_constraint_reaches_both_strategies_and_cli_raw_typed_channels() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root);
    let cases: Vec<(&str, Vec<u8>, &str, Option<&str>, bool)> = vec![
        ("data", include_bytes!("../../../testdata/sources/gnomad/constraint_tp53.json").to_vec(), "data", Some("ENST00000269305"), true),
        ("null metrics", include_bytes!("../../../testdata/sources/gnomad/constraint_ddx3x_null.json").to_vec(), "data", Some("ENST00000644876"), false),
        ("not found", include_bytes!("../../../testdata/sources/gnomad/constraint_not_found.json").to_vec(), "empty", None, false),
        ("empty", br#"{"data":{"gene":{"canonical_transcript_id":"  ","gnomad_constraint":{}}}}"#.to_vec(), "empty", None, false),
        ("trim source", br#"{"data":{"gene":{"canonical_transcript_id":"  ENST00000269305  ","gnomad_constraint":null,"ignored":"PRIVATE-CANARY"}}}"#.to_vec(), "data", Some("ENST00000269305"), false),
        ("graphql", br#"{"data":{"gene":{"canonical_transcript_id":"PRIVATE-CANARY"}},"errors":[{"message":"PRIVATE-CANARY"}]}"#.to_vec(), "unavailable", None, false),
        ("no message", br#"{"errors":[{}]}"#.to_vec(), "unavailable", None, false),
        ("wrong type", br#"{"data":{"gene":{"gnomad_constraint":{"pLI":"PRIVATE-CANARY"}}}}"#.to_vec(), "unavailable", None, false),
        ("http", b"PRIVATE-CANARY".to_vec(), "unavailable", None, false),
        ("timeout", b"".to_vec(), "unavailable", None, false),
    ];
    let response = Arc::new(Mutex::new(("data", Vec::<u8>::new())));
    let selected = Arc::clone(&response);
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let (release, wait) = std::sync::mpsc::channel();
    let hold = Arc::new(Mutex::new(wait));
    let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_owned());
            if request.starts_with("POST /gnomad") {
                let selected = selected.lock().unwrap();
                if selected.0 == "timeout" { return TestHttpReply::Hold(Arc::clone(&hold)); }
                return TestHttpReply::Bytes(test_http_response(
                    if selected.0 == "http" { "400 Bad Request" } else { "200 OK" },
                    "application/json", &selected.1,
                ));
            }
            let (status, bytes) = if request.starts_with("GET /query?") {
                ("200 OK", br#"{"total":1,"hits":[{"symbol":"TP53","name":"tumor protein p53","entrezgene":7157,"ensembl":{"gene":"ENSG00000141510"}}]}"#.as_slice())
            } else if request.starts_with("POST /graphql") {
                ("200 OK", br#"{"data":{"search":{"hits":[]},"target":{"associatedDiseases":{"rows":[]},"knownDrugs":{"rows":[]}}}}"#.as_slice())
            } else if request.starts_with("POST /addList") {
                ("200 OK", br#"{"userListId":1}"#.as_slice())
            } else if request.starts_with("GET /enrich?") {
                ("200 OK", b"{}".as_slice())
            } else if request.starts_with("GET /api/") {
                ("200 OK", br#"{"response":{"numFound":0,"start":0,"docs":[]}}"#.as_slice())
            } else { panic!("unrouted fixture request: {request}") };
            TestHttpReply::Bytes(test_http_response(status, "application/json", bytes))
        }).await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYGENE_BASE", fixture.base.clone()),
        ("BIOMCP_OLS4_BASE", fixture.base.clone()),
        ("BIOMCP_OPENTARGETS_BASE", fixture.base.clone()),
        ("BIOMCP_ENRICHR_BASE", fixture.base.clone()),
        ("BIOMCP_GNOMAD_BASE", format!("{}/gnomad", fixture.base)),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("BIOMCP_GENE_OPTIONAL_TIMEOUT_MS", "100".into()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for (label, body, state, transcript, metrics) in cases {
        *response.lock().unwrap() = (label, body);
        let before_default = requests.lock().unwrap().len();
        let default = crate::sources::with_no_cache_flag(
            true,
            get_with_report("TP53", &GeneGetOptions::default()),
        )
        .await
        .unwrap();
        assert!(default.gene.constraint.is_none());
        assert!(
            !requests.lock().unwrap()[before_default..]
                .iter()
                .any(|r| r.starts_with("POST /gnomad"))
        );
        for strategy in [GeneGetStrategy::Baseline, GeneGetStrategy::ParallelTop] {
            let options = GeneGetOptions::default()
                .with_sections(vec![GeneSection::Constraint, GeneSection::Ontology])
                .with_strategy(strategy)
                .with_optional_timeout(Duration::from_millis(100));
            assert!(should_use_parallel_top(&options.sections));
            let before = requests.lock().unwrap().len();
            let result =
                crate::sources::with_no_cache_flag(true, get_with_report("TP53", &options))
                    .await
                    .unwrap();
            assert_eq!(result.timing.strategy, strategy.as_str());
            assert!(
                result
                    .timing
                    .sections
                    .iter()
                    .any(|entry| entry.section == "constraint")
            );
            assert!(
                requests.lock().unwrap()[before..]
                    .iter()
                    .any(|r| r.starts_with("POST /gnomad")),
                "{label}: {strategy:?}"
            );
            let record: &biodata::GnomadGeneConstraintProjection =
                &result.gene.constraint.as_ref().unwrap().record;
            assert_eq!(record.transcript(), transcript, "{label}");
            check_output(
                &serde_json::to_string(&result.gene).unwrap(),
                true,
                state,
                transcript,
                metrics,
                false,
            );
        }
        if label == "data" {
            let before = requests.lock().unwrap().len();
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(["--json", "--no-cache", "get", "gene", "TP53"])
                .envs(envs.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success());
            let mut texts = vec![String::from_utf8(output.stdout).unwrap()];
            for tool in ["get", "biomcp"] {
                let arguments = if tool == "get" {
                    json!({"entity":"gene","id":"TP53","json":true})
                } else {
                    json!({"command":"biomcp --no-cache get gene TP53","json":true})
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
                texts.push(first_text(&result.content).to_owned());
            }
            for text in texts {
                let card: Value = serde_json::from_str(&text).unwrap();
                assert!(card.get("constraint").is_none());
                assert!(
                    !card["_meta"]["section_sources"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|source| source["key"] == "constraint")
                );
            }
            assert!(
                !requests.lock().unwrap()[before..]
                    .iter()
                    .any(|request| request.starts_with("POST /gnomad"))
            );
        }
        for json_mode in [true, false] {
            let mut args = vec!["--no-cache", "get", "gene", "TP53", "constraint"];
            if json_mode {
                args.push("--json");
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(envs.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success(), "{label}: {output:?}");
            check_output(
                std::str::from_utf8(&output.stdout).unwrap(),
                json_mode,
                state,
                transcript,
                metrics,
                true,
            );
            for tool in ["get", "biomcp"] {
                let arguments = if tool == "get" {
                    json!({"entity":"gene","id":"TP53","sections":["constraint"],"json":json_mode})
                } else {
                    json!({"command":"biomcp --no-cache get gene TP53 constraint","json":json_mode})
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
                check_output(
                    first_text(&result.content),
                    json_mode,
                    state,
                    transcript,
                    metrics,
                    true,
                );
            }
        }
    }
    client.cancel().await.unwrap();
    drop(release);
}
