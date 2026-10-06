//! Complete native cards retain distinct cached and recursive condition policies.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn row(present: bool, name: &str) -> Value {
    let mut row = json!({"_id":"chr7:g.140453136A>T","cadd":{"phred":32.0,"consequence":"missense"},
        "dbnsfp":{"genename":"BRAF","sift":{"pred":"D"}},
        "clinvar":{"variant_id":123,"rcv":{"accession":"RCV123","version":4,
            "clinical_significance":"Pathogenic","review_status":"reviewed by expert panel",
            "last_evaluated":"2024-02-03","number_submitters":3,"preferred_name":"Preferred",
            "conditions":[{"name":format!(" {name} "),"preferred_name":"Beta"},["Gamma"],"Alpha"]}}});
    if !present {
        row["clinvar"] = json!({"variant_id":123});
    }
    row
}

fn card(mode: &str, name: &str) -> Value {
    let mut card = json!({"id":"chr7:g.140453136A>T","gene":"BRAF","genome_build":"GRCh38",
        "cadd_score":32.0,"consequence":"missense","sift_pred":"Deleterious",
        "section_outcomes":{
            "cancerhotspots":{"outcome":"not_requested","sources":[]},
            "cbioportal":{"outcome":"not_requested","sources":[]},
            "civic":{"outcome":"not_requested","sources":[]},
            "clinvar":{"outcome":"not_requested","sources":[]},
            "gwas":{"outcome":"not_requested","sources":[]},
            "population":{"outcome":"not_requested","sources":[]},
            "predict":{"outcome":"not_requested","sources":[]}}});
    if mode != "missing" {
        card["significance"] = json!("Pathogenic");
        card["significance_source"] = json!("MyVariant.info");
        card["significance_evaluated"] = json!("2024-02-03");
        card["significance_note"] = json!(
            "Most severe RCV classification in MyVariant.info's cached ClinVar copy; run `biomcp get variant \"chr7:g.140453136A>T\" clinvar` for the current NCBI ClinVar record-level classification."
        );
    }
    match mode {
        "empty" => {
            card["section_outcomes"]["clinvar"] =
                json!({"outcome":"empty","sources":["NCBI ClinVar"]})
        }
        "fallback" => {
            card["section_outcomes"]["clinvar"] = json!({"outcome":"degraded","sources":["MyVariant.info"],
                "message":"Direct ClinVar retrieval is unavailable; showing MyVariant.info fallback data."});
            card["clinvar"] = json!({"source":"MyVariant.info","variation_id":123,"submissions":[],
                "aggregates":[{"source":"MyVariant.info","accession":"RCV123","version":4,
                    "classification_domain":"germline","classification":"Pathogenic","review_status":"reviewed by expert panel",
                    "evaluation_date":"2024-02-03","number_submitters":3,"conditions":[format!(" {name} "),"Alpha","Beta","Gamma","Preferred"]}]});
        }
        "missing" => {
            card["section_outcomes"]["clinvar"] = json!({"outcome":"unavailable","sources":[],"message":"ClinVar data is temporarily unavailable."})
        }
        _ => {}
    }
    card
}

fn native_card(mode: &str, name: &str) -> Value {
    let mut card = card(mode, name);
    let mut sources = json!([{"key":"identity","label":"Identity","outcome":"data","sources":["MyVariant.info","ClinVar"]}, {"key":"expanded_predictions","label":"Expanded Predictions","outcome":"data","sources":["MyVariant.info"]}]);
    if mode != "default" {
        sources.as_array_mut().unwrap().push(json!({"key":"clinvar","label":"ClinVar",
            "outcome":card["section_outcomes"]["clinvar"]["outcome"],"sources":card["section_outcomes"]["clinvar"]["sources"]}));
    }
    card["_meta"] = json!({"evidence_urls":[],"next_commands":["biomcp get gene BRAF","biomcp search drug --target BRAF",
        "biomcp variant trials \"chr7:g.140453136A>T\"","biomcp variant articles \"chr7:g.140453136A>T\""],"section_sources":sources});
    card["_meta"]["workflow"] = json!("variant-pathogenicity");
    card["_meta"]["workflow_rationale"] = json!(
        "Start with ClinVar, prediction, and population context, then widen to cancer evidence, trials, and literature."
    );
    card["_meta"]["workflow_playbook"] = json!("biomcp skill variant-pathogenicity");
    card
}

fn ledger(requests: &Arc<Mutex<Vec<String>>>, requested: bool) {
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), if requested { 3 } else { 2 }, "{log:?}");
    for (request, assembly) in log.iter().take(2).zip(["hg38", "hg19"]) {
        let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/variant/chr7:g.140453136A%3ET");
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
    if requested {
        let words: Vec<_> = log[2].lines().next().unwrap().split_whitespace().collect();
        assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
        let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
        assert_eq!(url.path(), "/efetch.fcgi");
        assert_eq!(
            url.query_pairs().into_owned().collect::<Vec<_>>(),
            [
                ("db".into(), "clinvar".into()),
                ("rettype".into(), "vcv".into()),
                ("is_variationid".into(), "true".into()),
                ("id".into(), "123".into())
            ]
        );
        assert_eq!(log[2].split_once("\r\n\r\n").unwrap().1, "");
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn clinvar_native_cards_keep_conditions_fallback_and_channels() {
    let state = Arc::new(Mutex::new(("default", "Alpha")));
    let served = state.clone();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        let (mode, name) = *served.lock().unwrap();
        if request.starts_with("GET /efetch.fcgi") {
            // The checked-empty and wrong-content-type replies reuse existing direct contracts.
            let (content, body) = if mode == "empty" {
                ("application/xml", "<ClinVarResult-Set/>")
            } else {
                ("text/html", "<html>private-provider-marker</html>")
            };
            TestHttpReply::Bytes(test_http_response("200 OK", content, body.as_bytes()))
        } else {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/json",
                &serde_json::to_vec(&row(mode != "missing", name)).unwrap(),
            ))
        }
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
        ("BIOMCP_CLINVAR_BASE", fixture.base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
        ("ONCOKB_TOKEN", "".into()),
        ("NCBI_API_KEY", "".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let harness = ContractHarness::new(
        std::env::var_os("BIOMCP_BIN").unwrap(),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for (mode, name) in [
        ("default", "Alpha"),
        ("empty", "Alpha"),
        ("fallback", "Alpha"),
        ("fallback", "Delta"),
        ("missing", "Alpha"),
    ] {
        *state.lock().unwrap() = (mode, name);
        let sections = if mode == "default" {
            vec![]
        } else {
            vec!["clinvar".into()]
        };
        requests.lock().unwrap().clear();
        let actual = super::get("chr7:g.140453136A>T", &sections).await.unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), card(mode, name));
        ledger(&requests, mode != "default");
        // The unstripped cached mapping counts only the top-level object and string names.
        if mode == "default" {
            let hit = serde_json::from_value(row(true, name)).unwrap();
            let cached = crate::transform::variant::from_myvariant_hit(&hit);
            assert_eq!(
                serde_json::to_value(cached.clinvar_conditions).unwrap(),
                json!([{"condition":"Alpha","reports":2}])
            );
            assert_eq!(cached.clinvar_condition_reports, Some(2));
        }
        requests.lock().unwrap().clear();
        let mut args = vec!["get", "variant", "chr7:g.140453136A>T"];
        if mode != "default" {
            args.push("clinvar");
        }
        args.push("--json");
        let output = tokio::process::Command::new(&harness.biomcp_bin)
            .args(args)
            .envs(envs.iter().cloned())
            .output()
            .await
            .unwrap();
        assert!(output.status.success());
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            native_card(mode, name)
        );
        ledger(&requests, mode != "default");
        for tool in ["get", "biomcp"] {
            requests.lock().unwrap().clear();
            let args = if tool == "get" {
                json!({"entity":"variant","id":"chr7:g.140453136A>T","sections":sections,"json":true})
            } else {
                json!({"json":true,"command":format!("biomcp get variant 'chr7:g.140453136A>T' {} --json",if mode=="default" {""} else {"clinvar"})})
            };
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(tool)
                        .with_arguments(args.as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            let mut actual = serde_json::to_value(&result).unwrap();
            actual["content"][0]["text"] =
                serde_json::from_str::<Value>(first_text(&result.content)).unwrap();
            assert_eq!(
                actual,
                json!({"content":[{"type":"text","text":native_card(mode,name)}],"isError":false})
            );
            ledger(&requests, mode != "default");
        }
        requests.lock().unwrap().clear();
        let command = format!(
            "biomcp get variant 'chr7:g.140453136A>T' {}",
            if mode == "default" { "" } else { "clinvar" }
        );
        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("biomcp")
                    .with_arguments(json!({"command":command}).as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        let text = first_text(&result.content);
        assert!(!text.contains("private-provider-marker"));
        assert!(!result.is_error.unwrap_or_default());
        match mode {
            "fallback" => {
                assert!(text.contains("MyVariant.info fallback"));
                assert!(text.contains("Gamma"));
                assert!(text.contains("Beta"));
                assert!(text.contains("Preferred"));
            }
            "empty" => {
                assert!(!text.contains("Gamma"));
                assert!(!text.contains("RCV123"));
            }
            "missing" => assert!(text.contains("temporarily unavailable")),
            _ => {
                assert!(!text.contains("RCV123"));
                assert!(!text.contains("Gamma"));
            }
        }
        ledger(&requests, mode != "default");
    }
    client.cancel().await.unwrap();
}
