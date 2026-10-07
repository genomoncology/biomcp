//! Cached sections retain their public channel representation.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

async fn channels(request_cgi: bool) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", br#"{"_id":"chr1:g.101A>T","dbnsfp":{"genename":"GENE"},"cgi":{"drug":" DrugA ","association":" Responsive "},"civic":{"molecularProfiles":[{"name":" P ","evidenceItems":[{"evidenceType":" Predictive ","therapies":[{"name":" A "}],"source":{"sourceType":"PUBMED","citation":"PMID:111"}}]}]}}"#))
    }).await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
        ("BIOMCP_CIVIC_BASE", fixture.base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("ONCOKB_TOKEN", "".into()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let harness = ContractHarness::new(std::env::var_os("BIOMCP_BIN").unwrap(), root);
    let mut command = vec!["get", "variant", "chr1:g.101A>T"];
    if request_cgi {
        command.push("cgi");
    }
    let expected = json!({"cached_evidence":[{"id":0,"name":"cached","molecular_profile":"P","evidence_type":"Predictive","evidence_level":"-","significance":"-","therapies":["A"],"status":"-"}]});
    let check_json = |text: &str| {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["civic"], expected);
        if request_cgi {
            assert_eq!(
                card["cgi_associations"],
                json!([{"drug":"DrugA","association":"Responsive"}])
            );
        } else {
            assert!(card.get("cgi_associations").is_none());
        }
    };
    let check_markdown = |text: &str| {
        if request_cgi {
            assert!(text.contains("| DrugA | Responsive | - | - |"), "{text}");
        } else {
            assert!(
                text.contains("Therapeutic evidence: 1 CIViC predictive item(s) / 0 assertion(s)"),
                "{text}"
            );
            assert!(
                text.contains("get variant \"chr1:g.101A>T\" civic"),
                "{text}"
            );
        }
    };
    for json_mode in [true, false] {
        let output = tokio::process::Command::new(&harness.biomcp_bin)
            .args(&command)
            .args(if json_mode { vec!["--json"] } else { vec![] })
            .envs(envs.iter().cloned())
            .output()
            .await
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = std::str::from_utf8(&output.stdout).unwrap();
        if json_mode {
            check_json(text);
        } else {
            check_markdown(text);
        }
    }
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for json_mode in [true, false] {
        let sections = if request_cgi { vec!["cgi"] } else { vec![] };
        let raw = format!(
            "biomcp {}{}",
            command.join(" "),
            if json_mode { " --json" } else { "" }
        );
        for (tool, args) in [
            (
                "get",
                json!({"entity":"variant","id":"chr1:g.101A>T","sections":sections,"json":json_mode}),
            ),
            ("biomcp", json!({"json":json_mode,"command":raw})),
        ] {
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(tool)
                        .with_arguments(args.as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            assert!(!result.is_error.unwrap_or_default(), "{result:?}");
            let text = first_text(&result.content);
            if json_mode {
                check_json(text);
            } else {
                check_markdown(text);
            }
        }
    }
    client.cancel().await.unwrap();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 12);
    assert!(
        requests
            .iter()
            .all(|r| r.starts_with("GET /variant/chr1:g.101A%3ET?")),
        "{requests:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn default_cached_sections_keep_cli_and_mcp_channels() {
    channels(false).await;
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn requested_cgi_keeps_cli_and_mcp_channels() {
    channels(true).await;
}
