//! Transport expectations are literals authored before interval admission.
use super::tests::{fixture, ledger, oracle, point_pages};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn interval_get_cli_typed_and_raw_mcp_preserve_transport_contracts() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root);
    let data = oracle();
    let row = &data["admission"][1];
    let mut h2 = row["hit"].clone();
    h2["_id"] = json!("chr1:g.102A>T");
    h2["dbsnp"]["rsid"] = json!("rs102");
    let original_over_limit = format!("{}GENE p.A11del", " ".repeat(500));
    assert_eq!(original_over_limit.len(), 513);
    let rows = [
        (
            "typed-original-limit",
            original_over_limit.as_str(),
            vec![json!({"total":1,"hits":[data["admission"][0]["hit"]]})],
            json!({"content":[{"type":"text","text":"Protein interval lookup exceeds its input limit."}],"isError":true}),
            vec![],
        ),
        (
            "cli",
            "GENE p.Ala11_Gly12del",
            vec![json!({"total":1,"hits":[row["hit"]]})],
            data["cli_success"].clone(),
            vec![0],
        ),
        (
            "assembly",
            "GENE p.Ala11_Gly12del",
            vec![json!({"total":0,"hits":[]})],
            data["assembly_payload"].clone(),
            vec![],
        ),
        (
            "raw",
            "GENE p.A11_G12del",
            vec![json!({"total":2,"hits":[row["hit"],h2]})],
            data["ambiguous_mcp"].clone(),
            vec![0],
        ),
        (
            "typed-success",
            "GENE p.A11_G12insV*",
            vec![json!({"total":1,"hits":[data["admission"][10]["hit"]]})],
            data["typed_success"].clone(),
            vec![0],
        ),
        (
            "raw-malformed",
            "GENE p.Ala11_Gly12del",
            vec![json!("malformed")],
            data["malformed_mcp"].clone(),
            vec![0],
        ),
        (
            "typed",
            "GENE p.Ala11_Gly12del",
            vec![json!({"total":1001,"hits":vec![row["hit"].clone();50]}); 20],
            data["incomplete_mcp"].clone(),
            (0..20).map(|n| n * 50).collect(),
        ),
    ];
    run_transport(
        &harness,
        &data,
        row,
        rows.into_iter()
            .map(|(c, i, p, e, o)| (c.to_string(), i.to_string(), p, e, o))
            .collect(),
    )
    .await;
}

async fn run_transport(
    harness: &ContractHarness,
    data: &Value,
    row: &Value,
    rows: Vec<(String, String, Vec<Value>, Value, Vec<usize>)>,
) {
    for (channel, input, pages, expected, offsets) in rows {
        let (fixture, requests) = fixture(pages).await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
            ("ONCOKB_TOKEN", "".into()),
        ];
        if channel.starts_with("cli") || channel == "assembly" {
            let mut args = vec!["get", "variant", input.as_str(), "--json"];
            if channel == "assembly" {
                args.extend(["--assembly", "grch38"]);
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(env.iter().cloned())
                .output()
                .await
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(if channel == "assembly" {
                    2
                } else if channel == "cli" || channel.ends_with("success") {
                    0
                } else if expected["error"]["code"] == "invalid_argument" {
                    2
                } else {
                    1
                })
            );
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            assert_eq!(output.stdout.last(), Some(&b'\n'));
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                expected
            );
        } else {
            let mut child = tokio::process::Command::new(&harness.biomcp_bin)
                .arg("serve")
                .current_dir(&harness.repo_root)
                .envs(env.iter().cloned())
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
                client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new("get").with_arguments(
                            json!({"entity":"variant","id":input,"json":true})
                                .as_object()
                                .unwrap()
                                .clone(),
                        ),
                    )
                    .await
                    .unwrap()
            };
            if channel == "typed-original-limit" {
                ledger(&requests, row["query"].as_str().unwrap(), &offsets);
            }
            let mut actual = serde_json::to_value(&result).unwrap();
            if channel.ends_with("success") {
                actual["content"][0]["text"] =
                    serde_json::from_str::<Value>(first_text(&result.content)).unwrap();
                assert_eq!(
                    actual,
                    json!({"content":[{"type":"text","text":expected}],"isError":false})
                );
            } else {
                assert_eq!(actual, expected);
            }
            client.cancel().await.unwrap();
            let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(status.code(), Some(0));
            assert_eq!(stderr.await.unwrap(), "");
        }
        ledger(
            &requests,
            if channel == "typed-success" {
                data["admission"][10]["query"].as_str().unwrap()
            } else {
                row["query"].as_str().unwrap()
            },
            &offsets,
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn point_get_cli_typed_and_raw_mcp_preserve_complete_transport_contracts() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root);
    let data = oracle();
    let point = &data["point"];
    let mut rows = Vec::new();
    for (name, channels) in [
        (
            "later protein assertion",
            vec!["cli-success", "typed-point-success", "raw-success"],
        ),
        (
            "later ambiguity",
            vec!["cli-ambiguous", "typed-point", "raw-point"],
        ),
        (
            "missing protein",
            vec!["cli-evidence", "typed-point", "raw-point"],
        ),
        (
            "bound 1001",
            vec!["cli-incomplete", "typed-point", "raw-point"],
        ),
        ("absence", vec!["cli-absent"]),
        ("late failure", vec!["cli-failure", "typed-point"]),
        ("malformed", vec!["cli-malformed", "raw-point"]),
    ] {
        let case = point["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        for channel in channels {
            let outcome = case["outcome"].as_str().unwrap();
            let expected = if outcome == "success" {
                point["cli_success"].clone()
            } else if channel.starts_with("cli") {
                point["payloads"][outcome].clone()
            } else {
                point["wrappers"][outcome].clone()
            };
            rows.push((
                channel.into(),
                point["input"].as_str().unwrap().into(),
                point_pages(case, point),
                expected,
                case["offsets"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| n.as_u64().unwrap() as usize)
                    .collect(),
            ));
        }
    }
    run_transport(&harness, &data, point, rows).await;
}
