use std::future::Future;
use std::time::Duration;

use axum::{Json, Router, routing::get};
use base64::Engine;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    AnnotateAble, CallToolResult, Content, Implementation, ListResourcesResult,
    PaginatedRequestParams, RawResource, ReadResourceRequestParams, ReadResourceResult,
    ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::schemars;
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::json;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct BioMcpServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ShellCommand {
    command: String,
}

const RESOURCE_HELP_URI: &str = "biomcp://help";

impl BioMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn tool_error(message: impl Into<String>) -> CallToolResult {
        CallToolResult::error(vec![Content::text(message.into())])
    }
}

impl Default for BioMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn is_allowed_mcp_command(args: &[String]) -> bool {
    // args[0] is the binary name ("biomcp")
    let Some(cmd) = args.get(1).map(|s| s.trim().to_ascii_lowercase()) else {
        return false;
    };

    match cmd.as_str() {
        "search" | "get" | "variant" | "drug" | "disease" | "article" | "gene" | "pathway"
        | "protein" | "study" | "list" | "version" | "health" | "batch" | "enrich" => true,
        "skill" => {
            // Allow read-only skill commands: list, show, numeric lookup
            // (e.g. "skill 03"), and slug lookup (e.g. "skill variant-to-treatment").
            // Block only mutating commands.
            let sub = args
                .get(2)
                .map(|s| s.trim().to_ascii_lowercase())
                .unwrap_or_else(|| "list".to_string());
            !matches!(sub.as_str(), "install" | "uninstall")
        }
        _ => false,
    }
}

#[tool_router]
impl BioMcpServer {
    #[doc = include_str!(concat!(env!("OUT_DIR"), "/mcp_shell_description.txt"))]
    #[tool]
    async fn biomcp(
        &self,
        Parameters(ShellCommand { command }): Parameters<ShellCommand>,
    ) -> Result<CallToolResult, McpError> {
        if command.len() > 1024 {
            return Ok(Self::tool_error("Error: command is too long"));
        }

        let split = match shlex::split(&command) {
            Some(args) => args,
            None => {
                return Ok(Self::tool_error(format!(
                    "Error: Invalid command syntax: {command}"
                )));
            }
        };

        let mut args = vec!["biomcp".to_string()];
        if split.first().is_some_and(|s| s == "biomcp") {
            args.extend(split.into_iter().skip(1));
        } else {
            args.extend(split);
        }

        if !is_allowed_mcp_command(&args) {
            return Ok(Self::tool_error(
                "Error: BioMCP allows read-only commands only (search/get/helpers/study/list/version/health/batch/enrich/skill)."
                    .to_string(),
            ));
        }

        match crate::cli::execute_mcp(args).await {
            Ok(output) => {
                let mut content = vec![Content::text(output.text)];
                if let Some(svg) = output.svg {
                    let encoded = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
                    content.push(Content::image(encoded, "image/svg+xml"));
                }
                Ok(CallToolResult::success(content))
            }
            Err(err) => Ok(Self::tool_error(format!("Error: {err}"))),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BioMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("biomcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "BioMCP provides biomedical data from 15 sources (PubMed, ClinicalTrials.gov, \
             ClinVar, gnomAD, OncoKB, Reactome, UniProt, PharmGKB, OpenFDA, and more). \
             Use the `biomcp` tool to run BioMCP CLI commands. \
             Start with `biomcp list` for a command reference, \
             or `biomcp skill` for guided investigation workflows."
                .to_string(),
        )
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult::with_all_items(
            build_resource_list()
                .into_iter()
                .map(|r| r.no_annotation())
                .collect(),
        )))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        std::future::ready(read_resource_markdown(&request.uri))
    }
}

fn read_resource_markdown(uri: &str) -> Result<ReadResourceResult, McpError> {
    if uri == RESOURCE_HELP_URI {
        let content = crate::cli::skill::show_overview()
            .map_err(|e| McpError::internal_error(format!("Failed to render {uri}: {e}"), None))?;
        return Ok(to_resource_result(uri, content));
    }

    if let Some(slug) = uri.strip_prefix("biomcp://skill/") {
        let content = crate::cli::skill::show_use_case(slug)
            .map_err(|_e| McpError::resource_not_found(format!("Unknown resource: {uri}"), None))?;
        return Ok(to_resource_result(uri, content));
    }

    Err(McpError::resource_not_found(
        format!("Unknown resource: {uri}"),
        None,
    ))
}

fn build_resource_list() -> Vec<RawResource> {
    let mut resources = vec![
        RawResource::new(RESOURCE_HELP_URI, "BioMCP Overview").with_mime_type("text/markdown"),
    ];

    if let Ok(skills) = crate::cli::skill::list_use_case_refs() {
        for skill in skills {
            let title = skill.title.trim();
            let name = if title.to_ascii_lowercase().starts_with("pattern:") {
                title.to_string()
            } else {
                format!("Pattern: {title}")
            };
            resources.push(
                RawResource::new(format!("biomcp://skill/{}", skill.slug), name)
                    .with_mime_type("text/markdown"),
            );
        }
    }

    resources
}

fn to_resource_result(uri: &str, content: String) -> ReadResourceResult {
    ReadResourceResult::new(vec![
        ResourceContents::text(content, uri).with_mime_type("text/markdown"),
    ])
}

fn mcp_stdio_guidance() -> &'static str {
    "This command expects an MCP client on stdin (initialize handshake). Use `biomcp serve-http` for manual testing."
}

fn is_handshake_startup_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_ascii_lowercase();
    msg.contains("expect initialize") || msg.contains("unexpected eof")
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

async fn index_handler() -> Json<serde_json::Value> {
    Json(json!({
        "name": "biomcp",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": "streamable-http",
        "mcp": "/mcp"
    }))
}

pub async fn run_stdio() -> anyhow::Result<()> {
    let shutdown = CancellationToken::new();

    let cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.cancel();
        }
    });

    let startup = tokio::time::timeout(
        Duration::from_secs(5),
        BioMcpServer::new().serve_with_ct(rmcp::transport::stdio(), shutdown),
    )
    .await;

    let running = match startup {
        Ok(Ok(running)) => running,
        Ok(Err(err)) => {
            let err = anyhow::Error::new(err);
            if is_handshake_startup_error(&err) {
                anyhow::bail!("{}", mcp_stdio_guidance());
            }
            return Err(err);
        }
        Err(_) => {
            anyhow::bail!("{}", mcp_stdio_guidance());
        }
    };
    let _reason = running.waiting().await?;
    Ok(())
}

pub async fn run_http(host: &str, port: u16) -> anyhow::Result<()> {
    let ip: std::net::IpAddr = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host address: {e}"))?;
    let bind = std::net::SocketAddr::new(ip, port);
    let shutdown = CancellationToken::new();

    let service: StreamableHttpService<BioMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(BioMcpServer::new()),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                cancellation_token: shutdown.child_token(),
                ..Default::default()
            },
        );

    let router = Router::new()
        .nest_service("/mcp", service)
        .route("/health", get(health_handler))
        .route("/readyz", get(health_handler))
        .route("/", get(index_handler));
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind HTTP server: {e}"))?;

    tracing::info!("BioMCP Streamable HTTP server listening on http://{bind}");
    tracing::info!("  MCP endpoint:   POST/GET http://{bind}/mcp");
    tracing::info!("  Health probe:   GET      http://{bind}/health");
    tracing::info!("  Ready probe:    GET      http://{bind}/readyz");
    tracing::info!("  Status:         GET      http://{bind}/");

    let cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.cancel();
        }
    });

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown.cancelled_owned().await;
        })
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server exited: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::Json;

    use super::{index_handler, is_allowed_mcp_command};

    #[test]
    fn mcp_allowlist_blocks_mutating_commands() {
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "search".into(),
            "gene".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "list".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "show".into()
        ]));
        // Numeric and slug skill lookups are read-only
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "03".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "variant-to-treatment".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "list".into()
        ]));
        assert!(!is_allowed_mcp_command(&["biomcp".into(), "update".into()]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "install".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "uninstall".into()
        ]));
    }

    #[tokio::test]
    async fn index_handler_reports_streamable_http_surface() {
        let Json(payload) = index_handler().await;
        assert_eq!(payload["name"], "biomcp");
        assert_eq!(payload["transport"], "streamable-http");
        assert_eq!(payload["mcp"], "/mcp");
    }
}
