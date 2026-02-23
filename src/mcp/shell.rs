use std::future::Future;

use rmcp::model::{
    AnnotateAble, Implementation, ListResourcesResult, PaginatedRequestParam, RawResource,
    ReadResourceRequestParam, ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{Error as McpError, ServerHandler, ServiceExt, tool};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct BioMcpServer;

const RESOURCE_HELP_URI: &str = "biomcp://help";
const SHELL_DESCRIPTION_MARKERS: &str = "SEARCH FILTERS:\nAGENT GUIDANCE:\n--lat --lon --distance";

fn shell_description() -> &'static str {
    let _markers = SHELL_DESCRIPTION_MARKERS;
    include_str!(concat!(env!("OUT_DIR"), "/mcp_shell_description.txt"))
}

fn is_allowed_mcp_command(args: &[String]) -> bool {
    // args[0] is the binary name ("biomcp")
    let Some(cmd) = args.get(1).map(|s| s.trim().to_ascii_lowercase()) else {
        return false;
    };

    match cmd.as_str() {
        "search" | "get" | "variant" | "drug" | "disease" | "article" | "gene" | "pathway"
        | "protein" | "list" | "version" | "health" | "batch" | "enrich" => true,
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

#[tool(tool_box)]
impl BioMcpServer {
    #[tool(description = shell_description())]
    async fn shell(&self, #[tool(param)] command: String) -> Result<String, String> {
        if command.len() > 1024 {
            return Err("Error: command is too long".to_string());
        }

        let split = match shlex::split(&command) {
            Some(args) => args,
            None => return Err(format!("Error: Invalid command syntax: {command}")),
        };

        let mut args = vec!["biomcp".to_string()];
        if split.first().is_some_and(|s| s == "biomcp") {
            args.extend(split.into_iter().skip(1));
        } else {
            args.extend(split);
        }

        if !is_allowed_mcp_command(&args) {
            return Err(
                "Error: MCP shell allows read-only commands only (search/get/helpers/list/version/health/batch/enrich/skill)."
                    .to_string(),
            );
        }

        crate::cli::execute(args)
            .await
            .map_err(|e| format!("Error: {e}"))
    }
}

#[tool(tool_box)]
impl ServerHandler for BioMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: Implementation {
                name: "biomcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            instructions: Some(
                "BioMCP provides biomedical data from 15 sources (PubMed, ClinicalTrials.gov, \
                 ClinVar, gnomAD, OncoKB, Reactome, UniProt, PharmGKB, OpenFDA, and more). \
                 Use the `shell` tool to run BioMCP CLI commands. \
                 Start with `biomcp list` for a command reference, \
                 or `biomcp skill list` for guided investigation workflows."
                    .to_string(),
            ),
            ..Default::default()
        }
    }

    fn list_resources(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListResourcesResult {
            next_cursor: None,
            resources: build_resource_list()
                .into_iter()
                .map(|r| r.no_annotation())
                .collect(),
        }))
    }

    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
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
    let mut resources = vec![RawResource {
        uri: RESOURCE_HELP_URI.to_string(),
        name: "BioMCP Overview".to_string(),
        description: None,
        mime_type: Some("text/markdown".to_string()),
        size: None,
    }];

    if let Ok(skills) = crate::cli::skill::list_use_case_refs() {
        for skill in skills {
            let title = skill.title.trim();
            let name = if title.to_ascii_lowercase().starts_with("pattern:") {
                title.to_string()
            } else {
                format!("Pattern: {title}")
            };
            resources.push(RawResource {
                uri: format!("biomcp://skill/{}", skill.slug),
                name,
                description: None,
                mime_type: Some("text/markdown".to_string()),
                size: None,
            });
        }
    }

    resources
}

fn to_resource_result(uri: &str, content: String) -> ReadResourceResult {
    ReadResourceResult {
        contents: vec![ResourceContents::TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some("text/markdown".to_string()),
            text: content,
        }],
    }
}

pub async fn run_stdio() -> anyhow::Result<()> {
    let shutdown = CancellationToken::new();

    let cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.cancel();
        }
    });

    let running = BioMcpServer
        .serve_with_ct(rmcp::transport::stdio(), shutdown)
        .await?;
    let _reason = running.waiting().await?;
    Ok(())
}

pub async fn run_http(host: &str, port: u16) -> anyhow::Result<()> {
    use rmcp::transport::sse_server::SseServer;

    let ip: std::net::IpAddr = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host address: {e}"))?;
    let bind = std::net::SocketAddr::new(ip, port);

    tracing::info!("BioMCP HTTP server listening on http://{bind}");
    tracing::info!("  SSE endpoint:  GET  http://{bind}/sse");
    tracing::info!("  Post endpoint: POST http://{bind}/message");

    let ct = SseServer::serve(bind)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind HTTP server: {e}"))?
        .with_service(|| BioMcpServer);

    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down…");
    ct.cancel();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_allowed_mcp_command;

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
}
