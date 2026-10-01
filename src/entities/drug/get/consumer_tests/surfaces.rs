//! Execute frozen CLI and raw/typed MCP results against synthetic replies.
use super::*;
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use futures::FutureExt;
use rmcp::model::CallToolRequestParams;
use std::panic::AssertUnwindSafe;

fn environment(fixture: &CaseHttp, cache: &std::path::Path) -> Vec<(&'static str, String)> {
    let base = &fixture.fixture.base;
    vec![
        ("BIOMCP_MYCHEM_BASE", format!("{base}/v1")),
        ("BIOMCP_OPENFDA_BASE", base.clone()),
        ("BIOMCP_CTGOV_BASE", format!("{base}/api/v2")),
        ("BIOMCP_CIVIC_BASE", format!("{base}/api")),
        ("BIOMCP_OPENTARGETS_BASE", base.clone()),
        ("BIOMCP_CHEMBL_BASE", base.clone()),
        ("BIOMCP_OLS4_BASE", base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", base.clone()),
        ("BIOMCP_CACHE_DIR", cache.display().to_string()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
        ("NCI_API_KEY", "".into()),
        ("OPENFDA_API_KEY", "".into()),
        ("UMLS_API_KEY", "".into()),
        ("NCBI_API_KEY", "".into()),
        ("S2_API_KEY", "".into()),
    ]
}
fn content(actual: &str, wanted: &Value) {
    if let Some(path) = wanted
        .get("stdout_json")
        .or_else(|| wanted.get("text_json"))
        .and_then(Value::as_str)
    {
        let actual: Value = serde_json::from_str(actual).unwrap();
        assert_eq!(actual, asset(path));
    } else if let Some(path) = wanted["ordered_content"].as_str() {
        let wanted = std::fs::read_to_string(root().join(path)).unwrap();
        assert_eq!(
            actual.split_whitespace().collect::<Vec<_>>(),
            wanted.split_whitespace().collect::<Vec<_>>()
        );
    } else if let Some(text) = wanted["text"].as_str() {
        assert_eq!(actual, text);
    } else {
        panic!("unhandled authored content: {wanted}");
    }
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn cli_and_raw_typed_mcp_drug_table() {
    let root_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root_path.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root_path);
    let mut failures = Vec::new();
    for case in table(5).into_iter().filter(|case| {
        matches!(
            case["input"]["operation"].as_str(),
            Some("shipped_cli_dispatch" | "mcp_dispatch")
        )
    }) {
        let outcome = AssertUnwindSafe(async {
            let input = &case["input"];
            let expected = &case["expected"];
            if input["operation"] == "shipped_cli_dispatch" {
                for (argv_key, result_key) in [
                    ("argv", "cli"),
                    ("argv_json", "cli_json"),
                    ("argv_markdown", "cli_markdown"),
                ] {
                    let Some(argv) = input[argv_key].as_array() else {
                        continue;
                    };
                    let fixture = CaseHttp::new(&case).await;
                    let cache = tempfile::tempdir().unwrap();
                    let env = environment(&fixture, cache.path());
                    let args = argv
                        .iter()
                        .skip(1)
                        .map(|arg| arg.as_str().unwrap())
                        .collect::<Vec<_>>();
                    let output = tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        tokio::process::Command::new(&harness.biomcp_bin)
                            .args(&args)
                            .envs(env)
                            .env_remove("NCI_API_KEY")
                            .kill_on_drop(true)
                            .output(),
                    )
                    .await
                    .unwrap()
                    .unwrap();
                    assert_eq!(
                        output.status.code().unwrap(),
                        expected[result_key]["exit"].as_i64().unwrap() as i32,
                        "{}: {}",
                        case["id"],
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert_eq!(
                        String::from_utf8_lossy(&output.stderr),
                        expected[result_key]["stderr"].as_str().unwrap()
                    );
                    content(
                        &String::from_utf8(output.stdout).unwrap(),
                        &expected[result_key],
                    );
                    fixture.assert_requests(&case["id"]);
                }
            } else {
                let fixture = CaseHttp::new(&case).await;
                let cache = tempfile::tempdir().unwrap();
                let env = environment(&fixture, cache.path());
                let client = harness.spawn_stdio_client(&env).await.unwrap();
                let result = client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new(input["tool"].as_str().unwrap().to_owned())
                            .with_arguments(input["arguments"].as_object().unwrap().clone()),
                    )
                    .await;
                client.cancel().await.unwrap();
                match result {
                    Ok(result) => {
                        let wanted = &expected["mcp_result"];
                        assert_eq!(
                            result.is_error,
                            Some(wanted["isError"].as_bool().unwrap()),
                            "{}",
                            case["id"]
                        );
                        assert!(result.structured_content.is_none());
                        assert_eq!(result.content.len(), 1, "{}", case["id"]);
                        assert_eq!(serde_json::to_value(&result.content[0]).unwrap()["type"], "text");
                        let text = first_text(&result.content);
                        content(text, &wanted["content"][0]);
                        let wire = serde_json::to_value(&result).unwrap();
                        assert!(wire.get("_meta").is_none());
                    }
                    Err(error) => {
                        let rmcp::ServiceError::McpError(data) = error else {
                            panic!("{}: expected protocol error, got {error:?}", case["id"]);
                        };
                        assert_eq!(
                            serde_json::json!({"code": data.code.0, "message": data.message, "data": data.data}),
                            expected["mapper_error"],
                            "{}", case["id"]
                        );
                    }
                }
                fixture.assert_requests(&case["id"]);
            }
        })
        .catch_unwind()
        .await;
        if outcome.is_err() {
            failures.push(case["id"].clone());
        }
    }
    assert!(
        failures.is_empty(),
        "surface alternatives failed: {failures:?}"
    );
}

mod privacy;
