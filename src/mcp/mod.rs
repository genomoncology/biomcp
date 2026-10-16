//! MCP server entrypoints for stdio and HTTP transports.

mod catalog;
mod shell;

/// Serializes the declared MCP tool catalog.
pub(crate) fn tool_catalog_json() -> anyhow::Result<String> {
    serde_json::to_string(&catalog::list_server_tools()).map_err(Into::into)
}

/// Runs the BioMCP MCP server over stdio.
///
/// `disable_sources` is the parsed `--disable-sources` value; the flag and
/// `BIOMCP_DISABLE_SOURCES` combine here exactly as they do for one-shot
/// CLI commands.
///
/// # Errors
///
/// Returns an error when stdio transport setup or MCP server startup fails.
pub async fn run_stdio(disable_sources: Option<&str>) -> anyhow::Result<()> {
    crate::sources::disable::init(disable_sources)?;
    shell::run_stdio().await
}

/// Runs the BioMCP MCP server over Streamable HTTP.
///
/// Starts an HTTP server on `host:port` with:
/// - `POST /mcp` — Streamable HTTP JSON-RPC requests
/// - `GET /mcp` — SSE stream managed by the Streamable HTTP session
/// - `GET /health` — liveness probe
/// - `GET /readyz` — readiness alias
/// - `GET /` — identity/status response
///
/// # Errors
///
/// Returns an error when TCP bind or server startup fails.
pub async fn run_http(
    host: &str,
    port: u16,
    allowed_hosts: Vec<String>,
    unsafe_allow_any_host: bool,
    disable_sources: Option<&str>,
) -> anyhow::Result<()> {
    crate::sources::disable::init(disable_sources)?;
    shell::run_http(host, port, allowed_hosts, unsafe_allow_any_host).await
}

/// Returns the deprecation guidance for the removed SSE transport command.
pub const fn sse_deprecation_message() -> &'static str {
    "The legacy SSE transport has been removed. Use `biomcp serve-http` and connect to `/mcp` instead."
}

/// Returns a deprecation error for the removed SSE transport command.
pub async fn run_sse() -> anyhow::Result<()> {
    anyhow::bail!("{}", sse_deprecation_message())
}
