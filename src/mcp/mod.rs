//! MCP server entrypoints for stdio and HTTP transports.

mod shell;

/// Runs the BioMCP MCP server over stdio.
///
/// # Errors
///
/// Returns an error when stdio transport setup or MCP server startup fails.
pub async fn run_stdio() -> anyhow::Result<()> {
    shell::run_stdio().await
}

/// Runs the BioMCP MCP server over HTTP with SSE transport.
///
/// Starts an HTTP server on `host:port` with:
/// - `GET /sse` — SSE stream for server-to-client messages
/// - `POST /message?sessionId=<id>` — client-to-server JSON-RPC messages
///
/// # Errors
///
/// Returns an error when TCP bind or server startup fails.
pub async fn run_http(host: &str, port: u16) -> anyhow::Result<()> {
    shell::run_http(host, port).await
}
