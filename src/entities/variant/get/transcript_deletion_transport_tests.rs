//! Thin native channel contracts use original synthetic MIT provider rows.
use super::tests::{AMBIGUOUS, card, fixture, hit, ledger};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::process::Stdio;
use tokio::io::AsyncReadExt;

fn native_card() -> Value {
    let mut expected = card("c.17_18del");
    expected["_meta"] = json!({"evidence_urls":[],
        "next_commands":["biomcp get gene TP53","biomcp search drug --target TP53",
            "biomcp variant trials \"chr17:g.101C>T\"","biomcp variant articles \"chr17:g.101C>T\""],
        "section_sources":[{"key":"identity","label":"Identity","outcome":"data","sources":["MyVariant.info","ClinVar"]}]});
    expected
}

async fn exercise(channel: &str) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let harness = ContractHarness::new(std::env::var_os("BIOMCP_BIN").unwrap(), root);
    let row = hit("c.17_18del");
    let mut other = row.clone();
    other["_id"] = json!("chr17:g.102C>T");
    let over_limit = format!(
        "{}NM_012345.7(TP53):c.17_18del",
        " ".repeat(513 - "NM_012345.7(TP53):c.17_18del".len())
    );
    let input = match channel {
        "typed-original-limit" => over_limit.as_str(),
        "cli-markdown" | "cli-split" => "NM_012345.7(TP53) c.17_18del",
        _ => "NM_012345.7(TP53):c.17_18del",
    };
    let pages = match channel {
        "typed-original-limit" | "cli-split" => vec![],
        "raw-ambiguous" => vec![json!({"total":2,"hits":[row,other]})],
        _ => vec![json!({"total":1,"hits":[row]})],
    };
    let offsets = if pages.is_empty() { vec![] } else { vec![0] };
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
        let mut args = if channel == "cli-split" {
            vec!["get", "variant", "NM_012345.7(TP53)", "c.17_18del"]
        } else {
            vec!["get", "variant", input]
        };
        if channel != "cli-markdown" {
            args.push("--json");
        }
        let output = tokio::process::Command::new(&harness.biomcp_bin)
            .args(args)
            .envs(env)
            .output()
            .await
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(if channel == "cli-split" { 2 } else { 0 })
        );
        assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        assert_eq!(output.stdout.last(), Some(&b'\n'));
        if channel == "cli-markdown" {
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.starts_with("# TP53 p.Gly6del\n"));
            for line in [
                "cDNA: c.17_18del",
                "Transcript: NM_012345.7",
                "Genomic coordinate (GRCh37, provider default): chr17:g.101C>T",
            ] {
                assert!(text.lines().any(|actual| actual == line), "{text}");
            }
        } else if channel == "cli-split" {
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                json!({
                    "error":{"code":"invalid_argument","message":"Invalid argument: Unknown section \"c.17_18del\" for variant. Available: predict, predictions, clinvar, population, population-details, conservation, cosmic, cgi, civic, cbioportal, gwas, all"},"_meta":{"not_found":false}})
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
            let arguments = json!({"entity":"variant","id":input,"json":true});
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
                "typed-original-limit" => {
                    "Transcript gene deletion lookup exceeds its input limit.".to_string()
                }
                "raw-ambiguous" => format!("Error: Invalid argument: {AMBIGUOUS}"),
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
        assert!(!logs.contains("c.17_18del"));
    }
    ledger(&requests, "c.17_18del", "NM_012345.7", &offsets);
}

macro_rules! channel_case {
    ($name:ident, $channel:literal) => {
        #[tokio::test(flavor = "multi_thread")]
        #[serial_test::parallel(source_env)]
        async fn $name() {
            exercise($channel).await;
        }
    };
}
channel_case!(transcript_deletion_34_cli_json, "cli-json");
channel_case!(transcript_deletion_35_cli_split_section, "cli-split");
channel_case!(transcript_deletion_36_cli_markdown, "cli-markdown");
channel_case!(transcript_deletion_37_typed_json, "typed-success");
channel_case!(
    transcript_deletion_38_typed_original_limit,
    "typed-original-limit"
);
channel_case!(transcript_deletion_39_raw_json, "raw-success");
channel_case!(transcript_deletion_40_raw_ambiguity, "raw-ambiguous");
