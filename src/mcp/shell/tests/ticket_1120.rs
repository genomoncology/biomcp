use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;

struct CtGovMcpFixtureEnv(Option<std::ffi::OsString>);

impl CtGovMcpFixtureEnv {
    fn set(base: &str) -> Self {
        let previous = std::env::var_os("BIOMCP_CTGOV_BASE");
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe { std::env::set_var("BIOMCP_CTGOV_BASE", base) };
        Self(previous)
    }
}

impl Drop for CtGovMcpFixtureEnv {
    fn drop(&mut self) {
        // SAFETY: callers hold the serial-test process-wide environment lock.
        unsafe {
            if let Some(previous) = self.0.take() {
                std::env::set_var("BIOMCP_CTGOV_BASE", previous);
            } else {
                std::env::remove_var("BIOMCP_CTGOV_BASE");
            }
        }
    }
}

async fn omitted_total_mcp_fixture() -> (
    String,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        let captured = captured.clone();
        async move {
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };

            captured
                .lock()
                .expect("lock fixture requests")
                .push(String::from_utf8_lossy(&request).into_owned());
            // Synthetic envelope: it exercises observed optionality but
            // is not represented as a captured provider response.
            let body = r#"{"studies":[]}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let base = task.base.clone();
    (base, requests, task)
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn raw_biomcp_tool_preserves_an_omitted_ctgov_total_as_null() {
    let (base, requests, mut server) = omitted_total_mcp_fixture().await;
    let _env = CtGovMcpFixtureEnv::set(&base);

    let result = BioMcpServer::new()
        .biomcp(crate::mcp::shell::arguments::Parameters(ShellCommand {
            command: "biomcp search trial --condition melanoma --count-only".into(),
            json: true,
        }))
        .await
        .expect("raw MCP count response");
    server.finish().unwrap();

    let value = serde_json::to_value(result).expect("serialize raw MCP result");
    let text = value["content"][0]["text"]
        .as_str()
        .expect("raw MCP count text");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text).expect("raw MCP count JSON"),
        json!({"total": null})
    );
    let requests = requests.lock().expect("lock fixture requests");
    assert_eq!(requests.len(), 1);
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
            .biomcp(crate::mcp::shell::arguments::Parameters(ShellCommand {
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
