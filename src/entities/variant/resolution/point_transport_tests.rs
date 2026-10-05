//! Search replays admitted A01 bytes; bounded get owns synthetic later-array success.
use super::*;
use crate::entities::article::test_support::{TestHttpFixture, TestHttpReply, test_http_response};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use futures::FutureExt;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

fn request_list(requests: &Arc<Mutex<Vec<String>>>, fields: &str, query: &str, size: &str) {
    let requests = requests.lock().unwrap().clone();
    assert_eq!(requests.len(), 1, "complete outbound list: {requests:?}");
    let request = &requests[0];
    let mut first = request.lines().next().unwrap().split_whitespace();
    assert_eq!(first.next(), Some("GET"));
    let url = reqwest::Url::parse(&format!("http://localhost{}", first.next().unwrap())).unwrap();
    assert_eq!(first.next(), Some("HTTP/1.1"));
    assert_eq!(first.next(), None);
    assert_eq!(url.path(), "/query");
    let literal = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/entities/variant/resolution/point_oracles")
            .join(fields),
    )
    .unwrap();
    assert_eq!(
        url.query_pairs().into_owned().collect::<Vec<_>>(),
        vec![
            ("q".into(), query.into()),
            ("size".into(), size.into()),
            ("from".into(), "0".into()),
            ("fields".into(), literal.trim_end().into())
        ]
    );
    assert_eq!(
        request.split_once("\r\n\r\n").unwrap().1,
        "",
        "request body must be absent"
    );
}

fn transport_oracle(case: &str) -> Value {
    if case == "T02" {
        oracle("correction_retained_transport/T02.output.json")
    } else {
        oracle(&format!("{case}.json"))
    }
}

fn wrapper(result: &CallToolResult, payload: Value, error: bool) {
    let mut actual = serde_json::to_value(result).unwrap();
    assert_eq!(actual["content"].as_array().unwrap().len(), 1);
    if payload.is_string() {
        assert_eq!(
            actual,
            json!({"content":[{"type":"text","text":payload}],"isError":error})
        );
    } else {
        let text = first_text(&result.content);
        assert_eq!(serde_json::from_str::<Value>(text).unwrap(), payload);
        actual["content"][0]["text"] = payload.clone();
        assert_eq!(
            actual,
            json!({"content":[{"type":"text","text":payload}],"isError":error})
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn accepted_point_cli_and_mcp_transport_table() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root.clone());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let raw =
        std::fs::read(root.join("testdata/sources/myvariant/search_braf_v600e_20260806.json"))
            .unwrap();
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
    let mut failures = Vec::new();
    for (case, args, code, fields, query, size) in [
        (
            "T02",
            vec![
                "search",
                "variant",
                "BRAF p.Val600Glu",
                "--limit",
                "5",
                "--json",
            ],
            0,
            "SEARCH_FIELDS.txt",
            "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.V600E\"",
            "50",
        ),
        (
            "T03",
            vec!["get", "variant", "p.Val600Glu", "--json"],
            1,
            "",
            "",
            "",
        ),
    ] {
        requests.lock().unwrap().clear();
        let result = AssertUnwindSafe(async {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(env.iter().cloned())
                .output()
                .await
                .unwrap();
            assert_eq!(
                output.status.code(),
                Some(code),
                "{case}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stderr.is_empty(), "{case}: {:?}", output.stderr);
            let output_matches = std::panic::catch_unwind(|| {
                assert_eq!(
                    serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                    transport_oracle(case)
                );
            })
            .is_ok();
            let requests_match = std::panic::catch_unwind(|| {
                if fields.is_empty() {
                    assert!(requests.lock().unwrap().is_empty());
                } else {
                    request_list(&requests, fields, query, size);
                }
            })
            .is_ok();
            assert!(
                output_matches && requests_match,
                "{case}: output={output_matches}; requests={requests_match}"
            );
        })
        .catch_unwind()
        .await;
        if result.is_err() {
            failures.push(case);
        }
    }
    // T04 calls the private dispatch in cli::variant::tests.
    let client = harness.spawn_stdio_client(&env).await.unwrap();
    for (tool, arguments, fields, query, size) in [(
        "search",
        json!({"entity":"variant","gene":"BRAF","hgvsp":"p.Val600Glu","limit":5,"json":true}),
        "SEARCH_FIELDS.txt",
        "dbnsfp.genename:BRAF AND dbnsfp.hgvsp:\"p.V600E\"",
        "50",
    )] {
        requests.lock().unwrap().clear();
        let result = AssertUnwindSafe(async {
            let result = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(tool)
                        .with_arguments(arguments.as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            let output_matches = std::panic::catch_unwind(|| {
                let mut expected = oracle("correction_retained_transport/T06.output.json");
                let payload: Value =
                    serde_json::from_str(expected["content"][0]["text"].as_str().unwrap()).unwrap();
                expected["content"][0]["text"] = payload.clone();
                let mut actual = serde_json::to_value(&result).unwrap();
                actual["content"][0]["text"] =
                    serde_json::from_str::<Value>(first_text(&result.content)).unwrap();
                assert_eq!(actual, expected);
            })
            .is_ok();
            let requests_match = std::panic::catch_unwind(|| {
                request_list(&requests, fields, query, size);
            })
            .is_ok();
            assert!(
                output_matches && requests_match,
                "typed {tool}: output={output_matches}; requests={requests_match}"
            );
        })
        .catch_unwind()
        .await;
        if result.is_err() {
            failures.push("T06");
        }
    }
    for (arguments, message) in [
        (
            json!({"entity":"variant","id":"BRAF p.Val600Glu","json":true,"unexpected":true}),
            "unknown variant get field: unexpected",
        ),
        (
            json!({"entity":"variant","id":"V".repeat(513),"json":true}),
            "id must contain 1-512 characters",
        ),
    ] {
        requests.lock().unwrap().clear();
        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new("get")
                    .with_arguments(arguments.as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        wrapper(&result, json!(message), true);
        assert!(requests.lock().unwrap().is_empty());
    }
    requests.lock().unwrap().clear();
    let result = biomcp_mcp_contract_client::call_biomcp_json(
        &client,
        "biomcp get variant p.Val600Glu --json",
    )
    .await
    .unwrap();
    wrapper(&result, oracle("T03.json"), false);
    assert!(requests.lock().unwrap().is_empty());
    client.cancel().await.unwrap();
    assert!(
        failures.is_empty(),
        "accepted transport scenario failures: {failures:?}"
    );
}
