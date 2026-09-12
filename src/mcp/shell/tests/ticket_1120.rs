use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct CtGovMcpFixtureEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _cache: tempfile::TempDir,
}

impl CtGovMcpFixtureEnv {
    fn set(base: &str) -> Self {
        let cache = tempfile::tempdir().expect("CTGov MCP fixture cache");
        let mut previous = Vec::new();
        for (key, value) in [
            ("BIOMCP_CTGOV_BASE", base.to_owned()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_owned()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
        ] {
            previous.push((key, std::env::var_os(key)));
            // SAFETY: callers hold the serial-test process-wide environment lock.
            unsafe { std::env::set_var(key, value) };
        }
        Self {
            previous,
            _cache: cache,
        }
    }
}

impl Drop for CtGovMcpFixtureEnv {
    fn drop(&mut self) {
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe {
            for (key, previous) in self.previous.drain(..).rev() {
                if let Some(previous) = previous {
                    std::env::set_var(key, previous);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }
}

async fn omitted_total_mcp_fixture()
-> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind synthetic CTGov MCP fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut request).await.expect("read CTGov request");
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(String::from_utf8_lossy(&request[..len]).into_owned());
                // Synthetic envelope: it exercises observed optionality but
                // is not represented as a captured provider response.
                let body = r#"{"studies":[]}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write CTGov response");
            });
        }
    });
    (base, requests, task)
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn raw_biomcp_tool_preserves_an_omitted_ctgov_total_as_null() {
    let (base, requests, server) = omitted_total_mcp_fixture().await;
    let _env = CtGovMcpFixtureEnv::set(&base);

    let result = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp search trial --condition melanoma --count-only".into(),
            json: true,
        }))
        .await
        .expect("raw MCP count response");
    let markdown = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp search trial --condition melanoma --count-only".into(),
            json: false,
        }))
        .await
        .expect("raw MCP Markdown count response");
    server.abort();

    let value = serde_json::to_value(result).expect("serialize raw MCP result");
    let text = value["content"][0]["text"]
        .as_str()
        .expect("raw MCP count text");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text).expect("raw MCP count JSON"),
        json!({
            "total": null,
            "total_precision": "unknown",
            "total_reason": "provider_omitted_total"
        })
    );
    let markdown_value = serde_json::to_value(markdown).expect("serialize Markdown response");
    assert_eq!(
        markdown_value["content"][0]["text"],
        "Total: unknown (provider_omitted_total)"
    );
    let requests = requests.lock().expect("lock fixture requests");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].contains("countTotal=true"));
    assert!(requests[0].contains("pageSize=1"));
}

#[tokio::test]
async fn raw_biomcp_reports_reversed_search_before_the_generic_fallback() {
    for (command, expected) in [
        (
            "biomcp article search",
            "Error: reversed search syntax; use `biomcp search article`",
        ),
        (
            "trial search --json",
            "Error: reversed search syntax; use `biomcp search trial --json`",
        ),
    ] {
        let result = BioMcpServer::new()
            .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
                command: command.into(),
                json: true,
            }))
            .await
            .expect("raw MCP correction");
        let value = serde_json::to_value(result).expect("serialize correction");
        assert_eq!(value["isError"], true);
        assert_eq!(value["content"][0]["text"], expected);
        assert_eq!(value["content"].as_array().unwrap().len(), 1);
    }
}
