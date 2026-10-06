//! Thin native channel contracts use original synthetic MIT provider rows.
use super::tests::{AMBIGUOUS, detail, fixture, hit, ledger};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

fn native_card() -> Value {
    let mut expected = detail("c.215C>G", "chr17:g.101C>T");
    expected["_meta"] = json!({"evidence_urls":[],
        "next_commands":["biomcp get gene TP53","biomcp search drug --target TP53",
            "biomcp variant trials \"chr17:g.101C>T\"","biomcp variant articles \"chr17:g.101C>T\""],
        "section_sources":[{"key":"identity","label":"Identity","outcome":"data","sources":["MyVariant.info","ClinVar"]}]});
    expected
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn gene_coding_get_cli_typed_and_raw_preserve_native_contracts() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let harness = ContractHarness::new(std::env::var_os("BIOMCP_BIN").unwrap(), root);
    let row = hit("c.215C>G", "chr17:g.101C>T");
    let mut other = row.clone();
    other["_id"] = json!("chr17:g.102C>T");
    let over_limit = format!("{}TP53 c.215C>G", " ".repeat(500));
    assert_eq!(over_limit.len(), 513);
    for (channel, input, pages, offsets) in [
        (
            "cli-json",
            "TP53 c.215C>G",
            vec![json!({"total":1,"hits":[row.clone()]})],
            vec![0],
        ),
        (
            "cli-markdown",
            "TP53 c.215C>G",
            vec![json!({"total":1,"hits":[row.clone()]})],
            vec![0],
        ),
        ("cli-assembly", "TP53 c.215C>G", vec![], vec![]),
        (
            "typed-success",
            "TP53 c.215C>G",
            vec![json!({"total":1,"hits":[row.clone()]})],
            vec![0],
        ),
        ("typed-original-limit", over_limit.as_str(), vec![], vec![]),
        ("typed-assembly", "TP53 c.215C>G", vec![], vec![]),
        (
            "raw-success",
            "TP53 c.215C>G",
            vec![json!({"total":1,"hits":[row.clone()]})],
            vec![0],
        ),
        (
            "raw-ambiguous",
            "TP53 c.215C>G",
            vec![json!({"total":2,"hits":[row.clone(),other]})],
            vec![0],
        ),
        (
            "raw-failure",
            "TP53 c.215C>G",
            vec![json!({"total":2,"hits":[row]}), json!("failure")],
            vec![0, 1],
        ),
    ] {
        let (fixture, requests) = fixture(pages).await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            (
                "RUST_LOG",
                "off,biomcp_cli=trace,reqwest_retry=error".into(),
            ),
            ("ONCOKB_TOKEN", "".into()),
        ];
        if channel.starts_with("cli") {
            let mut args = vec!["get", "variant", input];
            if channel != "cli-markdown" {
                args.push("--json");
            }
            if channel == "cli-assembly" {
                args.extend(["--assembly", "grch38"]);
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(env)
                .output()
                .await
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(if channel == "cli-assembly" { 2 } else { 0 })
            );
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            assert_eq!(output.stdout.last(), Some(&b'\n'));
            if channel == "cli-markdown" {
                let text = String::from_utf8(output.stdout).unwrap();
                assert!(text.starts_with("# TP53\n"));
                for line in [
                    "cDNA: c.215C>G",
                    "Transcript: NM_012345.7",
                    "Genomic coordinate (GRCh37, provider default): chr17:g.101C>T",
                ] {
                    assert!(text.lines().any(|actual| actual == line), "{text}");
                }
            } else if channel == "cli-assembly" {
                assert_eq!(
                    serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                    json!({
                    "error":{"code":"invalid_argument","message":"Invalid argument: --assembly only applies to chromosome-prefixed genomic coordinates"},"_meta":{"not_found":false}})
                );
            } else {
                assert_eq!(
                    serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                    native_card()
                );
            }
        } else {
            let mut child = tokio::process::Command::new(&harness.biomcp_bin)
                .arg("serve")
                .current_dir(&harness.repo_root)
                .envs(env)
                .env("UMLS_API_KEY", "")
                .env_remove("BIOMCP_TEST_PANIC_TOOL")
                .env_remove("RUST_MIN_STACK")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .unwrap();
            let mut stderr = child.stderr.take().unwrap();
            let stderr = tokio::spawn(async move {
                let mut text = String::new();
                stderr.read_to_string(&mut text).await.unwrap();
                text
            });
            let client =
                ().serve((child.stdout.take().unwrap(), child.stdin.take().unwrap()))
                    .await
                    .unwrap();
            let result = if channel.starts_with("raw") {
                biomcp_mcp_contract_client::call_biomcp_json(
                    &client,
                    &format!("get variant '{input}' --json"),
                )
                .await
                .unwrap()
            } else {
                let mut arguments = json!({"entity":"variant","id":input,"json":true});
                if channel == "typed-assembly" {
                    arguments["assembly"] = json!("grch38");
                }
                client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new("get")
                            .with_arguments(arguments.as_object().unwrap().clone()),
                    )
                    .await
                    .unwrap()
            };
            if channel.ends_with("success") {
                let mut actual = serde_json::to_value(&result).unwrap();
                actual["content"][0]["text"] =
                    serde_json::from_str::<Value>(first_text(&result.content)).unwrap();
                assert_eq!(
                    actual,
                    json!({"content":[{"type":"text","text":native_card()}],"isError":false})
                );
            } else {
                let text = match channel {
                    "typed-original-limit" => "Gene coding lookup exceeds its input limit.".to_string(),
                    "typed-assembly" => "Error: Invalid argument: --assembly only applies to chromosome-prefixed genomic coordinates".to_string(),
                    "raw-ambiguous" => format!("Error: Invalid argument: {AMBIGUOUS}"),
                    "raw-failure" => "Error: API request to MyVariant.info failed. Retry the remote source.".to_string(),
                    _ => unreachable!(),
                };
                assert_eq!(
                    serde_json::to_value(result).unwrap(),
                    json!({"content":[{"type":"text","text":text}],"isError":true})
                );
            }
            client.cancel().await.unwrap();
            assert_eq!(
                tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
                    .await
                    .unwrap()
                    .unwrap()
                    .code(),
                Some(0)
            );
            let logs = stderr.await.unwrap();
            assert!(!logs.contains("credential-input-canary"));
            assert!(!logs.contains("c.215C>G"));
        }
        ledger(&requests, "c.215C>G", &offsets);
    }
}
