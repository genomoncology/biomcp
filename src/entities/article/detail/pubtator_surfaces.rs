use std::path::{Path, PathBuf};

use biomcp_mcp_contract_client::{ContractHarness, call_biomcp_json, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

use super::super::test_support::{TestHttpFixture, TestHttpReply, test_http_response};

fn harness() -> ContractHarness {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    ContractHarness::new(binary, root)
}

fn record(pmid: &str, title: Option<&str>) -> Value {
    let mut passages = Vec::new();
    if let Some(title) = title {
        passages.push(json!({
            "infons": {"type": "title", "article-id_pmc": "PMC-PASSAGE-ONLY"},
            "offset": 0,
            "text": title,
            "sentences": [],
            "annotations": [{
                "id": "a1",
                "infons": {"type": "Gene"},
                "text": "BRAF",
                "locations": [{"offset": 0, "length": 4}]
            }],
            "relations": []
        }));
    }
    passages.push(json!({
        "infons": {"type": "abstract"},
        "offset": 20,
        "text": "A".repeat(1700),
        "sentences": [],
        "annotations": [],
        "relations": []
    }));
    json!({
        "id": pmid,
        "pmcid": null,
        "date": "2026-09-16T12:00:00Z",
        "journal": "Document Journal",
        "authors": ["First Author", "Second Author"],
        "passages": passages
    })
}

async fn fixture() -> TestHttpFixture {
    TestHttpFixture::spawn(|request| {
        let body = if request.starts_with("GET /publications/export/biocjson?") {
            assert!(!request.contains("api_key="));
            if request.contains("pmids=9") {
                br#"{"PubTator3":[{"id":"10","passages":[],"neighbor":true}]}"#.to_vec()
            } else {
                let (pmid, title) = if request.contains("pmids=8") {
                    ("8", Some("   "))
                } else if request.contains("pmids=12") {
                    ("12", None)
                } else {
                    ("7", Some("  Admitted title  "))
                };
                serde_json::to_vec(&json!({"PubTator3": [record(pmid, title)]}))
                    .expect("fixture JSON")
            }
        } else {
            br#"{"hitCount":1,"resultList":{"result":[{"id":"7","pmid":"7","pmcid":"PMC-EUROPE","doi":"10.1/fixture"}]}}"#.to_vec()
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &body))
    })
    .await
}

fn process_env(command: &mut tokio::process::Command, base: &str, cache: &Path) {
    command
        .env("BIOMCP_CACHE_DIR", cache)
        .env("BIOMCP_TEST_UNPACED_ORIGIN", base)
        .env("BIOMCP_PUBTATOR_BASE", base)
        .env("BIOMCP_EUROPEPMC_BASE", base)
        .env("NCBI_API_KEY", "")
        .env("RUST_LOG", "off");
}

async fn run_cli(
    harness: &ContractHarness,
    base: &str,
    cache: &Path,
    args: &[&str],
) -> std::process::Output {
    let mut command = tokio::process::Command::new(&harness.biomcp_bin);
    command.args(args);
    process_env(&mut command, base, cache);
    command.output().await.expect("CLI process")
}

fn json_output(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI JSON")
}

fn assert_detail(value: &Value) {
    assert_eq!(value["pmid"], "7");
    assert_eq!(value["title"], "Admitted title");
    assert_eq!(value["pmcid"], "PMC-EUROPE");
    assert_eq!(value["journal"], "Document Journal");
    assert_eq!(value["author_completeness"], "source_limited");
    assert_eq!(value["authors"], json!(["First Author", "Second Author"]));
    assert!(
        value["abstract_text"]
            .as_str()
            .is_some_and(|text| text.contains("truncated, 1700 chars total"))
    );
    assert_eq!(
        value["annotations"]["genes"],
        json!([{"text": "BRAF", "count": 1}])
    );
    assert!(!value.to_string().contains("PMC-PASSAGE-ONLY"));
}

#[tokio::test]
async fn actual_cli_detail_and_ordered_batches_use_adopted_pubtator() {
    let fixture = fixture().await;
    let cache = tempfile::tempdir().expect("cache");
    let harness = harness();
    let json = run_cli(
        &harness,
        &fixture.base,
        cache.path(),
        &["--json", "get", "article", "7"],
    )
    .await;
    assert_detail(&json_output(&json));

    let markdown = run_cli(
        &harness,
        &fixture.base,
        cache.path(),
        &["get", "article", "7"],
    )
    .await;
    assert!(markdown.status.success());
    let markdown = String::from_utf8_lossy(&markdown.stdout);
    assert!(markdown.contains("Admitted title"));
    assert!(markdown.contains("truncated, 1700 chars total"));

    for (mode, ids) in [("compact", "7,8,12"), ("detail", "12,8,7")] {
        let output = run_cli(
            &harness,
            &fixture.base,
            cache.path(),
            &["batch", "article", ids, "--mode", mode, "--json"],
        )
        .await;
        let value = json_output(&output);
        let inputs = value["items"]
            .as_array()
            .expect("batch items")
            .iter()
            .map(|item| item["input"].as_str().expect("batch input"))
            .collect::<Vec<_>>();
        let expected = ids.split(',').collect::<Vec<_>>();
        assert_eq!(inputs, expected);
        if mode == "compact" {
            assert_eq!(value["items"][1]["result"]["title"], "8");
            assert_eq!(value["items"][2]["result"]["title"], "12");
        } else {
            assert_eq!(value["items"][0]["result"]["title"], "");
            assert_eq!(value["items"][1]["result"]["title"], "");
        }
        let markdown = run_cli(
            &harness,
            &fixture.base,
            cache.path(),
            &["batch", "article", ids, "--mode", mode],
        )
        .await;
        assert!(markdown.status.success());
        assert!(String::from_utf8_lossy(&markdown.stdout).contains("Admitted title"));
    }

    let mixed = run_cli(
        &harness,
        &fixture.base,
        cache.path(),
        &["batch", "article", "7,bad,12", "--mode", "detail", "--json"],
    )
    .await;
    assert!(!mixed.status.success());
    let mixed: Value = serde_json::from_slice(&mixed.stdout).expect("mixed batch JSON");
    assert_eq!(mixed["items"][0]["status"], "ok");
    assert_eq!(mixed["items"][1]["status"], "error");
    assert_eq!(mixed["items"][2]["status"], "ok");
}

#[tokio::test(flavor = "multi_thread")]
async fn actual_raw_and_typed_mcp_preserve_detail_and_sanitize_refusal() {
    let fixture = fixture().await;
    let cache = tempfile::tempdir().expect("cache");
    let harness = harness();
    let env = [
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_PUBTATOR_BASE", fixture.base.clone()),
        ("BIOMCP_EUROPEPMC_BASE", fixture.base.clone()),
        ("NCBI_API_KEY", String::new()),
        ("RUST_LOG", "off".to_string()),
    ];
    let client = harness.spawn_stdio_client(&env).await.expect("MCP client");

    let raw = call_biomcp_json(&client, "biomcp get article 7")
        .await
        .expect("raw MCP");
    assert_eq!(raw.is_error, Some(false));
    assert_detail(&serde_json::from_str(first_text(&raw.content)).expect("raw detail"));

    let typed = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                json!({"entity": "article", "id": "7", "json": true})
                    .as_object()
                    .expect("typed arguments")
                    .clone(),
            ),
        )
        .await
        .expect("typed MCP");
    assert_eq!(typed.is_error, Some(false));
    assert_detail(&serde_json::from_str(first_text(&typed.content)).expect("typed detail"));

    let raw_rejected = call_biomcp_json(&client, "biomcp get article 9")
        .await
        .expect("raw refusal");
    let typed_rejected = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                json!({"entity": "article", "id": "9", "json": true})
                    .as_object()
                    .expect("typed refusal arguments")
                    .clone(),
            ),
        )
        .await
        .expect("typed refusal");
    for rejected in [raw_rejected, typed_rejected] {
        assert_eq!(rejected.is_error, Some(true));
        let refusal = first_text(&rejected.content);
        assert!(!refusal.contains("neighbor"));
        assert!(!refusal.contains("identity_mismatch"));
    }
    client.cancel().await.expect("cancel MCP client");
}
