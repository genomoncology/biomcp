//! Three distinct process claims keep the existing CLI and MCP contracts.
use super::rsid_lookup_tests::{AMBIGUOUS, detail, fixture, hit, ledger};
use biomcp_mcp_contract_client::ContractHarness;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn rsid_get_cli_detail_typed_ambiguity_and_raw_source_failure() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let harness = ContractHarness::new(
        std::path::PathBuf::from(std::env::var_os("BIOMCP_BIN").unwrap()),
        root,
    );
    let mut other = hit();
    other["_id"] = json!("chr1:g.102A>T");
    for (channel, pages, offsets) in [
        ("cli", vec![json!({"total":1,"hits":[hit()]})], vec![0]),
        (
            "typed",
            vec![json!({"total":2,"hits":[hit(),other]})],
            vec![0],
        ),
        (
            "raw",
            vec![json!({"hits":[hit()]}), json!("failure")],
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
            ("RUST_LOG", "off,reqwest_retry=error".into()),
            ("ONCOKB_TOKEN", "".into()),
        ];
        if channel == "cli" {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(["get", "variant", "rs101", "--json"])
                .envs(env)
                .output()
                .await
                .unwrap();
            assert_eq!(output.status.code(), Some(0));
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
            assert_eq!(output.stdout.last(), Some(&b'\n'));
            let mut expected = detail();
            expected["_meta"] = json!({
                "evidence_urls":[{"label":"dbSNP","url":"https://www.ncbi.nlm.nih.gov/snp/rs101"}],
                "next_commands":["biomcp get gene GENE","biomcp search drug --target GENE",
                    "biomcp variant trials \"chr1:g.101A>T\"","biomcp variant articles \"chr1:g.101A>T\""],
                "section_sources":[{"key":"identity","label":"Identity","outcome":"data","sources":["MyVariant.info","ClinVar"]}]});
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                expected
            );
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
                client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new("get").with_arguments(
                            json!({"entity":"variant","id":"rs101","json":true})
                                .as_object()
                                .unwrap()
                                .clone(),
                        ),
                    )
                    .await
                    .unwrap()
            } else {
                biomcp_mcp_contract_client::call_biomcp_json(&client, "get variant rs101 --json")
                    .await
                    .unwrap()
            };
            let expected = if channel == "typed" {
                json!({"content":[{"type":"text","text":format!("Error: Invalid argument: {AMBIGUOUS}")}],"isError":true})
            } else {
                json!({"content":[{"type":"text","text":"Error: API request to MyVariant.info failed. Retry the remote source."}],"isError":true})
            };
            assert_eq!(serde_json::to_value(result).unwrap(), expected);
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
        ledger(&requests, &offsets, 50);
    }
}
