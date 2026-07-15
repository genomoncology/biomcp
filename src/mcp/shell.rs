use std::collections::BTreeSet;
use std::future::Future;
use std::time::Duration;

use axum::{Json, Router, routing::get};
use base64::Engine;
use clap::CommandFactory;
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
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub struct BioMcpServer {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ShellCommand {
    command: String,
    #[serde(default)]
    json: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedSearch {
    #[schemars(transform = add_search_entity_enum)]
    entity: String,
    #[serde(default)]
    query: Option<String>,
    #[serde(default = "default_typed_limit")]
    #[schemars(range(min = 1, max = 25))]
    limit: usize,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    json: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedGet {
    #[schemars(transform = add_get_entity_enum)]
    entity: String,
    id: String,
    #[serde(default)]
    sections: Vec<McpGetSection>,
    #[serde(default)]
    json: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
struct McpGetSection(#[schemars(transform = add_get_section_enum)] String);

fn default_typed_limit() -> usize {
    10
}

fn add_string_enum(schema: &mut schemars::Schema, values: &[String]) {
    schema.insert(
        "enum".to_string(),
        Value::Array(
            values
                .iter()
                .map(|value| Value::String(value.clone()))
                .collect(),
        ),
    );
}

fn add_search_entity_enum(schema: &mut schemars::Schema) {
    add_string_enum(schema, &subcommand_names("search"));
}

fn add_get_entity_enum(schema: &mut schemars::Schema) {
    add_string_enum(schema, &subcommand_names("get"));
}

fn add_get_section_enum(schema: &mut schemars::Schema) {
    add_string_enum(schema, &all_get_sections());
}

const RESOURCE_HELP_URI: &str = "biomcp://help";
const GENERIC_MCP_REJECTION_MESSAGE: &str = "Error: BioMCP allows read-only commands only. Allowed families are search/get/helpers/list/version/health/batch/enrich/discover/skill plus MCP-safe study commands (`study list`, `study download --list`, `study top-mutated`, `study query`, `study filter`, `study cohort`, `study survival`, `study compare`, `study co-occurrence`).";
const CACHE_FAMILY_MCP_REJECTION_MESSAGE: &str = "Error: biomcp cache commands are CLI-only over MCP because they reveal workstation-local filesystem paths.";

impl BioMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn tool_error(message: impl Into<String>) -> CallToolResult {
        CallToolResult::error(vec![Content::text(message.into())])
    }

    async fn execute_args(args: Vec<String>, json: bool) -> Result<CallToolResult, McpError> {
        match crate::cli::execute_mcp(args.clone()).await {
            Ok(output) => {
                let text = if json || args_include_json(&args) {
                    redact_mcp_json_text(&output.text).map_err(|err| {
                        McpError::internal_error(
                            format!("Failed to sanitize MCP JSON response: {err}"),
                            None,
                        )
                    })?
                } else {
                    match crate::cli::execute_mcp(args_with_json(&args)).await {
                        Ok(json_output) => match serde_json::from_str::<Value>(&json_output.text) {
                            Ok(value) => {
                                let text = redact_mcp_text(output.text, &value);
                                append_default_mcp_footer(text, &json_output.text)
                            }
                            Err(err) if args_may_return_article_fulltext(&args) => {
                                return Err(McpError::internal_error(
                                    format!(
                                        "Failed to inspect MCP full-text response fields: {err}"
                                    ),
                                    None,
                                ));
                            }
                            Err(_) => output.text,
                        },
                        Err(err) if args_may_return_article_fulltext(&args) => {
                            return Err(McpError::internal_error(
                                format!("Failed to prepare safe MCP full-text response: {err}"),
                                None,
                            ));
                        }
                        Err(_) => output.text,
                    }
                };
                let mut content = vec![Content::text(text)];
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
        | "protein" | "list" | "version" | "health" | "batch" | "enrich" | "discover" => true,
        "study" => {
            let Some(sub) = args.get(2).map(|s| s.trim().to_ascii_lowercase()) else {
                return false;
            };
            match sub.as_str() {
                "list" | "top-mutated" | "query" | "filter" | "cohort" | "survival" | "compare"
                | "co-occurrence" => true,
                "download" => args.len() == 4 && args[3] == "--list",
                _ => false,
            }
        }
        "skill" => {
            let Some(sub) = args.get(2).map(|s| s.trim().to_ascii_lowercase()) else {
                return true;
            };
            if args.len() != 3 {
                return false;
            }
            matches!(sub.as_str(), "list" | "render")
                || crate::cli::skill::show_use_case(&sub).is_ok()
        }
        _ => false,
    }
}

fn mcp_rejection_message(args: &[String]) -> &'static str {
    if args
        .get(1)
        .is_some_and(|cmd| cmd.trim().eq_ignore_ascii_case("cache"))
    {
        CACHE_FAMILY_MCP_REJECTION_MESSAGE
    } else {
        GENERIC_MCP_REJECTION_MESSAGE
    }
}

fn args_include_json(args: &[String]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.as_str(), "--json" | "-j"))
}

fn args_with_json(args: &[String]) -> Vec<String> {
    let mut with_json = args.to_vec();
    if !args_include_json(&with_json) {
        with_json.push("--json".to_string());
    }
    with_json
}

fn args_may_return_article_fulltext(args: &[String]) -> bool {
    args.get(1).is_some_and(|arg| arg == "get") && args.get(2).is_some_and(|arg| arg == "article")
}

fn get_section_groups() -> &'static [&'static [&'static str]] {
    &[
        crate::entities::gene::GENE_SECTION_NAMES,
        crate::entities::article::ARTICLE_SECTION_NAMES,
        crate::entities::disease::DISEASE_SECTION_NAMES,
        crate::entities::diagnostic::DIAGNOSTIC_SECTION_NAMES,
        crate::entities::pgx::PGX_SECTION_NAMES,
        crate::entities::trial::TRIAL_SECTION_NAMES,
        crate::entities::variant::VARIANT_SECTION_NAMES,
        crate::entities::drug::DRUG_SECTION_NAMES,
        crate::entities::pathway::PATHWAY_SECTION_NAMES,
        crate::entities::protein::PROTEIN_SECTION_NAMES,
        crate::entities::adverse_event::ADVERSE_EVENT_SECTION_NAMES,
    ]
}

fn subcommand_names(name: &str) -> Vec<String> {
    crate::cli::Cli::command()
        .find_subcommand(name)
        .expect("top-level subcommand exists")
        .get_subcommands()
        .map(|cmd| cmd.get_name().to_string())
        .collect()
}

fn all_get_sections() -> Vec<String> {
    get_section_groups()
        .iter()
        .flat_map(|group| group.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn normalize_token(raw: &str, allowed: &[String], field: &str) -> Result<String, McpError> {
    let token = raw.trim();
    if allowed.iter().any(|allowed| allowed == token) {
        Ok(token.to_string())
    } else {
        Err(McpError::invalid_params(
            format!(
                "invalid {field}: {token}; allowed values: {}",
                allowed.join(", ")
            ),
            None,
        ))
    }
}

fn search_args(input: TypedSearch) -> Result<Vec<String>, McpError> {
    let search_entities = subcommand_names("search");
    let entity = normalize_token(&input.entity, &search_entities, "search entity")?;
    if input.limit == 0 || input.limit > 25 {
        return Err(McpError::invalid_params(
            "invalid limit: typed search limit must be between 1 and 25",
            None,
        ));
    }

    let mut args = vec![
        "biomcp".to_string(),
        "search".to_string(),
        entity.to_string(),
    ];
    if let Some(query) = input
        .query
        .as_deref()
        .map(str::trim)
        .filter(|q| !q.is_empty())
    {
        match entity.as_str() {
            "article" => args.extend(["--keyword".to_string(), query.to_string()]),
            "author" => args.extend(["--query".to_string(), query.to_string()]),
            "diagnostic" | "gwas" | "pgx" => {
                args.extend(["--gene".to_string(), query.to_string()]);
            }
            "trial" => args.extend(["--condition".to_string(), query.to_string()]),
            "all" => args.extend(["--keyword".to_string(), query.to_string()]),
            _ => args.push(query.to_string()),
        }
    }
    args.extend(["--limit".to_string(), input.limit.to_string()]);
    if input.offset > 0 {
        args.extend(["--offset".to_string(), input.offset.to_string()]);
    }
    if input.json {
        args = args_with_json(&args);
    }
    Ok(args)
}

fn get_args(input: TypedGet) -> Result<Vec<String>, McpError> {
    let get_entities = subcommand_names("get");
    let entity = normalize_token(&input.entity, &get_entities, "get entity")?;
    let allowed_sections = all_get_sections();
    for section in &input.sections {
        normalize_token(&section.0, &allowed_sections, "get section")?;
    }

    let mut args = vec![
        "biomcp".to_string(),
        "get".to_string(),
        entity.to_string(),
        input.id,
    ];
    args.extend(input.sections.into_iter().map(|section| section.0));
    if input.json {
        args = args_with_json(&args);
    }
    Ok(args)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct McpSectionSource {
    label: String,
    sources: Vec<String>,
}

#[derive(Debug, Default)]
struct McpMetaFooter {
    section_sources: Vec<McpSectionSource>,
    next_commands: Vec<String>,
}

fn push_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn collect_meta_footer(value: &Value, footer: &mut McpMetaFooter) {
    if let Some(meta) = value.get("_meta").and_then(Value::as_object) {
        if let Some(sections) = meta.get("section_sources").and_then(Value::as_array) {
            for section in sections {
                let Some(label) = section.get("label").and_then(Value::as_str) else {
                    continue;
                };
                let sources = section
                    .get("sources")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if !sources.is_empty() {
                    push_unique(
                        &mut footer.section_sources,
                        McpSectionSource {
                            label: label.to_string(),
                            sources,
                        },
                    );
                }
            }
        }

        if let Some(commands) = meta.get("next_commands").and_then(Value::as_array) {
            for command in commands.iter().filter_map(Value::as_str) {
                push_unique(&mut footer.next_commands, command.to_string());
            }
        }
    }

    match value {
        Value::Array(values) => {
            for item in values {
                collect_meta_footer(item, footer);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                collect_meta_footer(item, footer);
            }
        }
        _ => {}
    }
}

fn mcp_meta_footer_from_json(json_text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json_text).ok()?;
    let mut footer = McpMetaFooter::default();
    collect_meta_footer(&value, &mut footer);

    if footer.section_sources.is_empty() && footer.next_commands.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    if !footer.section_sources.is_empty() {
        lines.push("## Sources".to_string());
        for section in footer.section_sources {
            lines.push(format!(
                "- {}: {}",
                section.label,
                section.sources.join(", ")
            ));
        }
    }
    if !footer.next_commands.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("## Next commands".to_string());
        for command in footer.next_commands {
            lines.push(format!("- `{command}`"));
        }
    }
    Some(lines.join("\n"))
}

fn collect_full_text_paths(value: &Value, paths: &mut Vec<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_full_text_paths(value, paths);
            }
        }
        Value::Object(map) => {
            if let Some(path) = map.get("full_text_path").and_then(Value::as_str) {
                push_unique(paths, path.to_string());
            }
            for value in map.values() {
                collect_full_text_paths(value, paths);
            }
        }
        _ => {}
    }
}

fn redact_mcp_text(mut text: String, value: &Value) -> String {
    let mut paths = Vec::new();
    collect_full_text_paths(value, &mut paths);
    for path in paths {
        text = text.replace(
            &format!("Saved to: {path}"),
            "Full text: available (local cache path withheld over MCP)",
        );
        text = text.replace(&path, "[local path withheld over MCP]");
    }
    text
}

fn redact_mcp_json_value(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                redact_mcp_json_value(value);
            }
        }
        Value::Object(map) => {
            let full_text_available = map
                .remove("full_text_path")
                .is_some_and(|path| !path.is_null());
            if full_text_available {
                map.insert("full_text_available".to_string(), Value::Bool(true));
            }
            for value in map.values_mut() {
                redact_mcp_json_value(value);
            }
        }
        _ => {}
    }
}

fn redact_mcp_json_text(text: &str) -> Result<String, serde_json::Error> {
    let mut value: Value = serde_json::from_str(text)?;
    redact_mcp_json_value(&mut value);
    serde_json::to_string_pretty(&value)
}

fn append_default_mcp_footer(text: String, json_text: &str) -> String {
    match mcp_meta_footer_from_json(json_text) {
        Some(footer) => format!("{text}\n\n{footer}"),
        None => text,
    }
}

#[tool_router]
impl BioMcpServer {
    #[doc = include_str!(concat!(env!("OUT_DIR"), "/mcp_shell_description.txt"))]
    #[tool(annotations(title = "BioMCP", read_only_hint = true))]
    async fn biomcp(
        &self,
        Parameters(ShellCommand { command, json }): Parameters<ShellCommand>,
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
            return Ok(Self::tool_error(mcp_rejection_message(&args)));
        }

        if json {
            args = args_with_json(&args);
        }

        Self::execute_args(args, json).await
    }

    /// Search a biomedical entity with typed MCP parameters instead of a shell command string.
    #[tool(annotations(title = "BioMCP typed search", read_only_hint = true))]
    async fn search(
        &self,
        Parameters(input): Parameters<TypedSearch>,
    ) -> Result<CallToolResult, McpError> {
        let json = input.json;
        let args = search_args(input)?;
        Self::execute_args(args, json).await
    }

    /// Get one biomedical entity record with typed entity, id, and section parameters.
    #[tool(annotations(title = "BioMCP typed get", read_only_hint = true))]
    async fn get(
        &self,
        Parameters(input): Parameters<TypedGet>,
    ) -> Result<CallToolResult, McpError> {
        let json = input.json;
        let args = get_args(input)?;
        Self::execute_args(args, json).await
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
            "BioMCP provides biomedical data from leading public biomedical data sources \
             (PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome, UniProt, \
             PharmGKB, OpenFDA, and more). \
             Prefer the typed `search` and `get` tools for structured entity lookup; use the raw `biomcp` command tool as an escape hatch for long-tail commands. \
             Start with `biomcp skill list` when you need the right playbook, \
             `biomcp list` for a command reference, \
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
    msg.contains("expect initialize")
        || msg.contains("unexpected eof")
        || (msg.contains("connection closed") && msg.contains("initialize"))
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

pub async fn run_http(host: &str, port: u16, allowed_hosts: Vec<String>) -> anyhow::Result<()> {
    let ip: std::net::IpAddr = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host address: {e}"))?;
    let bind = std::net::SocketAddr::new(ip, port);
    let shutdown = CancellationToken::new();

    #[allow(clippy::field_reassign_with_default)]
    let http_config = {
        let mut http_config = StreamableHttpServerConfig::default();
        http_config.stateful_mode = true;
        http_config.cancellation_token = shutdown.child_token();
        http_config.allowed_hosts = allowed_hosts;
        http_config
    };

    let service: StreamableHttpService<BioMcpServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(BioMcpServer::new()), Default::default(), http_config);

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
    use std::collections::BTreeSet;

    use super::{
        CACHE_FAMILY_MCP_REJECTION_MESSAGE, GENERIC_MCP_REJECTION_MESSAGE, TypedGet, TypedSearch,
        all_get_sections, get_args, get_section_groups, index_handler, is_allowed_mcp_command,
        mcp_rejection_message, redact_mcp_json_text, redact_mcp_text, search_args,
        subcommand_names,
    };
    use axum::Json;

    fn section_names_from_sources() -> BTreeSet<&'static str> {
        get_section_groups()
            .iter()
            .flat_map(|group| group.iter().copied())
            .collect()
    }

    #[test]
    fn mcp_full_text_path_redaction_is_field_driven_for_text_and_json() {
        let path = "/tmp/BioMCP cache/naïve article.md";
        let value = serde_json::json!({
            "title": "Example",
            "full_text_path": path,
            "full_text_source": {"source": "Europe PMC"}
        });
        let text = redact_mcp_text(format!("## Full Text\nSaved to: {path}"), &value);
        assert_eq!(
            text,
            "## Full Text\nFull text: available (local cache path withheld over MCP)"
        );
        assert!(!text.contains(path));
        assert!(!text.contains("Saved to:"));

        let json = redact_mcp_json_text(&value.to_string()).expect("valid JSON");
        let redacted: serde_json::Value = serde_json::from_str(&json).expect("redacted JSON");
        assert_eq!(redacted["full_text_available"], true);
        assert_eq!(redacted["full_text_source"]["source"], "Europe PMC");
        assert!(redacted.get("full_text_path").is_none());
        assert!(!json.contains(path));
    }

    #[test]
    fn typed_schema_sources_match_cli_entities_and_sections() {
        assert!(subcommand_names("search").contains(&"pathway".to_string()));
        assert!(subcommand_names("search").contains(&"author".to_string()));
        assert!(subcommand_names("get").contains(&"author".to_string()));
        assert!(subcommand_names("get").contains(&"gene".to_string()));
        assert_eq!(
            all_get_sections().into_iter().collect::<BTreeSet<String>>(),
            section_names_from_sources()
                .into_iter()
                .map(str::to_string)
                .collect::<BTreeSet<String>>()
        );
    }

    #[test]
    fn typed_search_and_get_build_cli_args() {
        let search = search_args(TypedSearch {
            entity: "pathway".to_string(),
            query: Some("MAPK signaling".to_string()),
            limit: 5,
            offset: 0,
            json: true,
        })
        .expect("typed search args");
        assert_eq!(
            search,
            [
                "biomcp",
                "search",
                "pathway",
                "MAPK signaling",
                "--limit",
                "5",
                "--json"
            ]
        );

        let author = search_args(TypedSearch {
            entity: "author".to_string(),
            query: Some("Louis Williams".to_string()),
            limit: 5,
            offset: 0,
            json: false,
        })
        .expect("typed author search args");
        assert_eq!(
            author,
            [
                "biomcp",
                "search",
                "author",
                "--query",
                "Louis Williams",
                "--limit",
                "5"
            ]
        );

        let get = get_args(TypedGet {
            entity: "gene".to_string(),
            id: "BRAF".to_string(),
            sections: vec![super::McpGetSection("pathways".to_string())],
            json: false,
        })
        .expect("typed get args");
        assert_eq!(get, ["biomcp", "get", "gene", "BRAF", "pathways"]);
    }

    #[test]
    fn typed_search_rejects_out_of_schema_limit_before_cli_dispatch() {
        let err = search_args(TypedSearch {
            entity: "pathway".to_string(),
            query: Some("MAPK".to_string()),
            limit: 50,
            offset: 0,
            json: false,
        })
        .expect_err("limit over schema cap should be invalid params");
        assert!(err.message.contains("typed search limit"));
    }

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
            "render".into()
        ]));
        assert!(is_allowed_mcp_command(&["biomcp".into(), "skill".into()]));
        // Numeric and slug skill lookups are read-only when they name embedded skills.
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "03".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "gene-disease-orientation".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "03-gene-disease-orientation".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "list".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "--list".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "cache".into(),
            "path".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "cache".into(),
            "stats".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "top-mutated".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--limit".into(),
            "10".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "query".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into(),
            "--type".into(),
            "mutations".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "filter".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "cohort".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--cohort".into(),
            "tp53".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "survival".into(),
            "--study".into(),
            "msk_impact_2017".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "compare".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "co-occurrence".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "suggest".into(),
            "What drugs treat melanoma?".into()
        ]));
        assert!(is_allowed_mcp_command(&[
            "biomcp".into(),
            "discover".into(),
            "BRCA1".into()
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
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "not-a-real-skill".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "skill".into(),
            "render".into(),
            "extra".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "ema".into(),
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "who-ivd".into(),
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "msk_impact_2017".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "--list".into(),
            "msk_impact_2017".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "study".into(),
            "download".into()
        ]));
    }

    #[test]
    fn cache_family_rejection_message_mentions_local_path_disclosure() {
        let args = vec!["biomcp".into(), "cache".into(), "path".into()];
        assert_eq!(
            mcp_rejection_message(&args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );

        let stats_args = vec!["biomcp".into(), "cache".into(), "stats".into()];
        assert_eq!(
            mcp_rejection_message(&stats_args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );

        let clear_args = vec!["biomcp".into(), "cache".into(), "clear".into()];
        assert_eq!(
            mcp_rejection_message(&clear_args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );
    }

    #[test]
    fn generic_mcp_rejection_message_stays_read_only_for_mutating_commands() {
        let args = vec!["biomcp".into(), "update".into()];
        assert_eq!(mcp_rejection_message(&args), GENERIC_MCP_REJECTION_MESSAGE);
    }

    #[tokio::test]
    async fn index_handler_reports_streamable_http_surface() {
        let Json(payload) = index_handler().await;
        assert_eq!(payload["name"], "biomcp");
        assert_eq!(payload["transport"], "streamable-http");
        assert_eq!(payload["mcp"], "/mcp");
    }
}
