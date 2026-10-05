//! Actual executable and typed MCP checks against accepted complete 0668 objects.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;

fn decoded_requests(requests: &Arc<Mutex<Vec<String>>>) -> Value {
    let items: Vec<_> = requests
        .lock()
        .unwrap()
        .iter()
        .map(|request| {
            let words: Vec<_> = request.lines().next().unwrap().split_whitespace().collect();
            assert_eq!(words.len(), 3);
            assert_eq!(words[2], "HTTP/1.1");
            let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
            json!({"method":words[0],"path":url.path(),
            "query_pairs":url.query_pairs().into_owned().collect::<Vec<_>>(),
            "body":request.split_once("\r\n\r\n").unwrap().1})
        })
        .collect();
    json!(items)
}

pub(super) async fn search_row(response: Value, request: Value) -> Value {
    let raw = serde_json::to_vec(&response).unwrap();
    let fixture = TestHttpFixture::spawn(move |_| {
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &raw))
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYVARIANT_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_CACHE_MODE", "off");
    let identity: RequestedVariantIdentity = serde_json::from_value(request).unwrap();
    let filters = crate::entities::variant::VariantSearchFilters {
        gene: identity.gene.clone(),
        hgvsc: identity.coding_change.clone(),
        hgvsp: identity.protein_change.clone(),
        requested_identity: Some(identity),
        ..Default::default()
    };
    let page = crate::entities::variant::search_page(&filters, 5, 0)
        .await
        .unwrap();
    assert_eq!(page.results.len(), 1);
    serde_json::to_value(&page.results[0]).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn coding_cli_and_mcp_table() {
    let rows = oracle("transports");
    assert_eq!(rows["cases"].as_array().unwrap().len(), 18);
    let interval: Value =
        serde_json::from_str(include_str!("interval_search_oracles.json")).unwrap();
    let original = rows["cases"].as_array().unwrap();
    let added = interval["transport_cases"].as_array().unwrap();
    assert_eq!(added.len(), 8);
    for (index, row) in original.iter().enumerate() {
        let (prefix, number) = if index < 6 {
            ("T", index + 1)
        } else if index < 14 {
            ("CT", index - 5)
        } else {
            ("GT", index - 13)
        };
        assert_eq!(row["id"], format!("{prefix}{number:02}"));
    }
    for (index, row) in added.iter().enumerate() {
        assert_eq!(row["id"], format!("IT{:02}", index + 1));
    }
    assert_eq!(original.len() + added.len(), 26);
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root().join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root());
    for row in original.iter().chain(added) {
        let id = row["id"].as_str().unwrap();
        let raw = if let Some(file) = row["stub_response_file"].as_str() {
            std::fs::read(root().join(file)).unwrap()
        } else {
            serde_json::to_vec(&row["stub_response"]).unwrap()
        };
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.to_owned());
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &raw))
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("BIOMCP_CACHE_MODE", "off".into()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        if row["channel"] == "cli" {
            let args: Vec<_> = row["input"]["argv"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect();
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(env.iter().cloned())
                .output()
                .await
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(row["expected_exit"].as_i64().unwrap() as i32),
                "{id}"
            );
            assert_eq!(
                String::from_utf8(output.stderr).unwrap(),
                row["expected_stderr"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                row["expected_payload"],
                "{id}"
            );
        } else {
            let mut child = tokio::process::Command::new(&harness.biomcp_bin)
                .arg("serve")
                .current_dir(&harness.repo_root)
                .envs(env.iter().cloned())
                .env("UMLS_API_KEY", "")
                .env_remove("RUST_MIN_STACK")
                .env_remove("BIOMCP_TEST_PANIC_TOOL")
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
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(row["input"]["tool"].as_str().unwrap().to_owned())
                        .with_arguments(row["input"]["arguments"].as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            let mut actual = serde_json::to_value(&result).unwrap();
            assert_eq!(actual["content"].as_array().unwrap().len(), 1, "{id}");
            let payload = serde_json::from_str::<Value>(first_text(&result.content)).unwrap();
            let expected = if id == "T06" {
                let mut expected = read_json(row["expected_full_wrapper_file"].as_str().unwrap());
                expected["content"][0]["decoded_json"] =
                    serde_json::from_str::<Value>(expected["content"][0]["text"].as_str().unwrap())
                        .unwrap();
                expected["content"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("text");
                expected
            } else {
                assert_eq!(payload, row["expected_payload"], "{id}");
                row["expected_mcp"].clone()
            };
            actual["content"][0]["decoded_json"] = payload;
            actual["content"][0].as_object_mut().unwrap().remove("text");
            assert_eq!(actual, expected, "complete wrapper {id}");
            client.cancel().await.unwrap();
            let status = tokio::time::timeout(std::time::Duration::from_secs(5), child.wait())
                .await
                .unwrap()
                .unwrap();
            let stderr = stderr.await.unwrap();
            if let Some(expected) = row.get("expected_process") {
                assert_eq!(
                    json!({"exit":status.code(),"stderr":stderr}),
                    *expected,
                    "process {id}"
                );
            }
        }
        let expected = if id == "T06" {
            let mut expected = read_json(row["expected_request_file"].as_str().unwrap());
            for request in expected.as_array_mut().unwrap() {
                assert_eq!(request["base"], "https://myvariant.info/v1");
                assert!(request["body"].is_null());
                let object = request.as_object_mut().unwrap();
                object.remove("base");
                let pairs = object.remove("ordered_decoded_query_pairs").unwrap();
                object.insert("query_pairs".into(), pairs);
                object.insert("body".into(), json!(""));
            }
            expected
        } else {
            row["expected_requests"].clone()
        };
        assert_eq!(
            decoded_requests(&requests),
            expected,
            "complete requests {id}"
        );
    }
}
