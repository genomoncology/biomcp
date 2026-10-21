//! The panic route exists only in this test build and is never registered by production.
use super::{BioMcpServer, CallToolRequestParams, CallToolResult, McpError};
use rmcp::{ServiceExt, tool, tool_router};

#[tool_router(router = panic_test_router)]
impl BioMcpServer {
    #[tool(
        name = "__biomcp_test_panic",
        description = "Internal panic recovery test hook"
    )]
    async fn test_panic(&self) -> Result<CallToolResult, McpError> {
        panic!("injected MCP tool panic")
    }
}

// Exercise the real protocol and server dispatch. A shipped subprocess cannot
// expose this route because this entire module is compiled only for tests.
#[tokio::test]
async fn rmcp_stdio_recovers_from_tool_panic_on_the_same_session() -> anyhow::Result<()> {
    let mut server = BioMcpServer::new();
    server.tool_router += BioMcpServer::panic_test_router();
    let (server_io, client_io) = tokio::io::duplex(4096);
    let serving = tokio::spawn(async move {
        server.serve(server_io).await?.waiting().await?;
        Ok::<_, anyhow::Error>(())
    });
    let client = ().serve(client_io).await?;
    let panic_result = client
        .peer()
        .call_tool(CallToolRequestParams::new("__biomcp_test_panic"))
        .await?;
    assert_eq!(panic_result.is_error, Some(true));
    assert!(panic_result.content.iter().any(|content| content
        .as_text()
        .is_some_and(|text| text.text.contains("injected MCP tool panic"))));

    let success = client
        .peer()
        .call_tool(CallToolRequestParams::new("biomcp").with_arguments(
            serde_json::json!({"command": "biomcp version"})
                .as_object()
                .unwrap()
                .clone(),
        ))
        .await?;
    assert_eq!(success.is_error, Some(false));
    client.cancel().await?;
    serving.await??;
    Ok(())
}
