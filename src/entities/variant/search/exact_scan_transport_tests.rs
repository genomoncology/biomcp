//! Distinct actual CLI and native MCP output composition claims.
use super::exact_scan_tests::{fixture, ledger};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

fn json_page(value: &Value) {
    assert_eq!(value["resolution"]["status"], "ambiguous");
    assert_eq!(value["resolution"]["exhaustive"], false);
    assert_eq!(value["count"], 1);
    assert_eq!(value["results"][0]["id"], "chr7:g.101A>T");
    assert_eq!(
        value["pagination"],
        json!({"offset":0,"limit":1,"returned":1,"total":null,"has_more":true,"next_page_token":null})
    );
    assert_eq!(value["results"][0]["transcript"], "NM_004333.6");
    assert_eq!(value["results"][0]["hgvs_c"], "c.1799T>A");
    assert_eq!(value["results"][0]["transcript_annotations_complete"], true);
    assert_eq!(
        value["results"][0]["transcript_annotations"],
        json!([{
            "source":"myvariant.info/snpeff.ann", "gene":"BRAF", "transcript":"NM_004333.6",
            "hgvs_c":"c.1799T>A", "hgvs_p":"p.Val600Glu", "roles":["displayed","matched"]
        }])
    );
    assert!(value["requested_variant"].is_object());
    assert!(value["filter_evaluation"].is_object());
    assert_eq!(value["diagnostics"], json!([]));
    assert!(value["_meta"]["next_commands"].is_array());
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn incomplete_search_cli_json_typed_json_and_raw_markdown() {
    let harness = ContractHarness::new(
        std::path::PathBuf::from(std::env::var_os("BIOMCP_BIN").unwrap()),
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    for channel in ["cli", "typed", "raw"] {
        let (fixture, requests) = fixture(vec![
            json!({"total":2,"hits":[{
                "_id":"chr7:g.101A>T", "dbnsfp":{"genename":"BRAF","hgvsp":"p.Val600Glu"},
                "snpeff":{"ann":{"feature_id":"NM_004333.6", "genename":"BRAF",
                    "hgvs_c":"c.1799T>A", "hgvs_p":"p.Val600Glu"}}
            }]}),
            json!({"total":2,"hits":[]}),
        ])
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        if channel == "cli" {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args([
                    "search", "variant", "-g", "BRAF", "--hgvsp", "V600E", "--limit", "1", "--json",
                ])
                .envs(env)
                .output()
                .await
                .unwrap();
            assert_eq!(output.status.code(), Some(0));
            assert!(output.stderr.is_empty());
            assert_eq!(output.stdout.last(), Some(&b'\n'));
            json_page(&serde_json::from_slice::<Value>(&output.stdout).unwrap());
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
            let result = if channel == "typed" {
                client.peer().call_tool(CallToolRequestParams::new("search").with_arguments(json!({"entity":"variant","gene":"BRAF","hgvsp":"V600E","limit":1,"json":true}).as_object().unwrap().clone())).await.unwrap()
            } else {
                biomcp_mcp_contract_client::call_biomcp(
                    &client,
                    "search variant -g BRAF --hgvsp V600E --limit 1",
                )
                .await
                .unwrap()
            };
            let text = first_text(&result.content);
            assert_eq!(
                serde_json::to_value(&result).unwrap(),
                json!({"content":[{"type":"text","text":text}],"isError":false})
            );
            if channel == "typed" {
                json_page(&serde_json::from_str::<Value>(text).unwrap());
            } else {
                assert!(
                    text.starts_with(
                        "Requested variant: BRAF V600E\n\nVariant identity: ambiguous\n\n"
                    ),
                    "{text}"
                );
                assert!(
                    text.contains("Showing 1 results (total unknown). Use --offset 1 for more."),
                    "{text}"
                );
                assert!(
                    text.contains("| BRAF | NM_004333.6 | c.1799T>A | p.Val600Glu |"),
                    "{text}"
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
            assert_eq!(stderr.await.unwrap(), "");
        }
        ledger(
            &requests,
            &[0, 1],
            "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.V600E\"",
        );
    }
}
