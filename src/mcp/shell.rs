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
struct TypedVariantCar {
    #[schemars(length(min = 1, max = 50))]
    inputs: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedVariantErepo {
    #[serde(default)]
    caid: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 50))]
    caids: Option<Vec<String>>,
    #[serde(default)]
    detail: bool,
    #[serde(default)]
    assertion_id: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedGeneCspec {
    gene: String,
    #[serde(default)]
    version_iri: Option<String>,
    #[serde(default)]
    capture_id: Option<String>,
    #[serde(default)]
    offset: usize,
    #[serde(default = "default_cspec_limit")]
    #[schemars(range(min = 1, max = 50))]
    limit: usize,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedVariantArticles {
    #[schemars(length(min = 1, max = 10))]
    items: Vec<crate::entities::variant::VariantArticleRequest>,
    #[serde(default = "default_variant_article_strategy")]
    #[schemars(transform = add_variant_article_strategy_enum)]
    strategy: String,
    #[serde(default = "default_typed_limit")]
    #[schemars(range(min = 1, max = 50))]
    limit: usize,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    debug_plan: bool,
    #[serde(default)]
    verify_identity: bool,
    #[serde(default)]
    confirmed_only: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
struct McpGetSection(#[schemars(transform = add_get_section_enum)] String);

fn default_typed_limit() -> usize {
    10
}

fn default_cspec_limit() -> usize {
    25
}

fn default_variant_article_strategy() -> String {
    "union".into()
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

fn add_variant_article_strategy_enum(schema: &mut schemars::Schema) {
    add_string_enum(
        schema,
        &["union".into(), "annotation".into(), "lexical".into()],
    );
}

fn variant_article_strategy(
    value: &str,
) -> Result<crate::entities::article::VariantArticleStrategy, McpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "union" => Ok(crate::entities::article::VariantArticleStrategy::Union),
        "annotation" => Ok(crate::entities::article::VariantArticleStrategy::Annotation),
        "lexical" => Ok(crate::entities::article::VariantArticleStrategy::Lexical),
        _ => Err(McpError::invalid_params(
            "variant_articles strategy must be union, annotation, or lexical",
            None,
        )),
    }
}

const RESOURCE_HELP_URI: &str = "biomcp://help";
const GENERIC_MCP_REJECTION_MESSAGE: &str = "Error: BioMCP allows read-only commands only. Allowed families are search/get/helpers/list/version/health/batch/enrich/discover/skill plus MCP-safe study commands (`study list`, `study download --list`, `study top-mutated`, `study query`, `study filter`, `study cohort`, `study survival`, `study compare`, `study co-occurrence`).";
const CACHE_FAMILY_MCP_REJECTION_MESSAGE: &str = "Error: biomcp cache commands are CLI-only over MCP because they reveal workstation-local filesystem paths.";
const VARIANT_ARTICLE_INPUT_MCP_REJECTION_MESSAGE: &str = "Error: variant articles --input is CLI-only over raw MCP because it reads server-local files or stdin; use the typed variant_articles tool instead.";

impl BioMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn tool_error(message: impl Into<String>) -> CallToolResult {
        let message = crate::render::human::sanitize_inline(&message.into());
        CallToolResult::error(vec![Content::text(message)])
    }

    async fn execute_args(args: Vec<String>, json: bool) -> Result<CallToolResult, McpError> {
        if let Some(message) = binary_download_rejection(&args) {
            return Ok(Self::tool_error(message));
        }
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
                    match output.metadata_json.as_deref() {
                        Some(json_text) => match serde_json::from_str::<Value>(json_text) {
                            Ok(value) => {
                                let text = redact_mcp_text(output.text, &value);
                                append_default_mcp_footer(text, json_text)
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
                        None if args_may_return_article_fulltext(&args) => {
                            return Err(McpError::internal_error(
                                "Failed to prepare safe MCP full-text response metadata",
                                None,
                            ));
                        }
                        None => output.text,
                    }
                };
                let text = if json || args_include_json(&args) {
                    text
                } else {
                    crate::render::human::sanitize_document(&text)
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

fn binary_download_rejection(args: &[String]) -> Option<String> {
    if args.get(1).is_none_or(|value| value != "get") {
        return None;
    }
    let entity = args.get(2)?.as_str();
    let section = args.get(4)?.as_str();
    let is_binary = matches!(
        (entity, section),
        ("trial", "document") | ("article", "asset")
    );
    if !is_binary {
        return None;
    }
    let command = args
        .iter()
        .take_while(|arg| !matches!(arg.as_str(), "--json" | "-j"))
        .map(|arg| shlex::try_quote(arg).unwrap_or_else(|_| "<value>".into()))
        .collect::<Vec<_>>()
        .join(" ");
    Some(format!(
        "Binary {entity} {section} downloads are CLI-only. Run `{command}` from a terminal."
    ))
}

impl Default for BioMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn raw_variant_articles_reads_input(args: &[String]) -> bool {
    args.get(1)
        .is_some_and(|value| value.eq_ignore_ascii_case("variant"))
        && args
            .get(2)
            .is_some_and(|value| value.eq_ignore_ascii_case("articles"))
        && args
            .iter()
            .skip(3)
            .any(|value| value == "--input" || value.starts_with("--input="))
}

fn is_allowed_mcp_command(args: &[String]) -> bool {
    // args[0] is the binary name ("biomcp")
    let Some(cmd) = args.get(1).map(|s| s.trim().to_ascii_lowercase()) else {
        return false;
    };

    match cmd.as_str() {
        "variant" => !raw_variant_articles_reads_input(args),
        "search" | "get" | "drug" | "disease" | "article" | "gene" | "pathway" | "protein"
        | "list" | "version" | "health" | "batch" | "enrich" | "discover" => true,
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
    } else if raw_variant_articles_reads_input(args) {
        VARIANT_ARTICLE_INPUT_MCP_REJECTION_MESSAGE
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
    let mut args = vec![
        "biomcp".to_string(),
        "get".to_string(),
        entity.to_string(),
        input.id.clone(),
    ];
    args.extend(input.sections.iter().map(|section| section.0.clone()));
    if let Some(message) = binary_download_rejection(&args) {
        return Err(McpError::invalid_params(message, None));
    }
    let allowed_sections = all_get_sections();
    for section in &input.sections {
        normalize_token(&section.0, &allowed_sections, "get section")?;
    }
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

fn redact_mcp_json_text(text: &str) -> Result<String, crate::error::BioMcpError> {
    let mut value: Value = serde_json::from_str(text)?;
    redact_mcp_json_value(&mut value);
    crate::render::json::to_pretty(&value)
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

    /// Normalize 1-50 versioned RefSeq HGVS values through the read-only ClinGen Allele Registry.
    #[tool(annotations(
        title = "BioMCP ClinGen Allele Registry normalization",
        read_only_hint = true
    ))]
    async fn variant_normalize_car(
        &self,
        Parameters(input): Parameters<TypedVariantCar>,
    ) -> Result<CallToolResult, McpError> {
        if input.inputs.is_empty() || input.inputs.len() > 50 {
            return Err(McpError::invalid_params(
                "variant_normalize_car inputs must contain 1-50 HGVS strings",
                None,
            ));
        }
        match crate::entities::variant::normalize_car_batch(input.inputs).await {
            Ok(response) => Ok(CallToolResult::success(vec![Content::text(
                crate::render::json::to_pretty(&response).map_err(|error| {
                    McpError::internal_error(
                        format!("Failed to serialize CAR response: {error}"),
                        None,
                    )
                })?,
            )])),
            Err(error) => Ok(Self::tool_error(format!("Error: {error}"))),
        }
    }

    /// Retrieve versioned ClinGen ERepo expert assertions for one CAid or a bounded CAid batch.
    #[tool(annotations(title = "BioMCP ClinGen ERepo assertions", read_only_hint = true))]
    async fn variant_erepo(
        &self,
        Parameters(input): Parameters<TypedVariantErepo>,
    ) -> Result<CallToolResult, McpError> {
        let caids = match (input.caid, input.caids) {
            (Some(caid), None) => vec![caid],
            (None, Some(caids)) if !caids.is_empty() && caids.len() <= 50 => caids,
            _ => {
                return Err(McpError::invalid_params(
                    "variant_erepo requires exactly one of caid or caids (1-50)",
                    None,
                ));
            }
        };
        if caids.len() != 1
            && (input.detail || input.assertion_id.is_some() || input.version.is_some())
        {
            return Err(McpError::invalid_params(
                "variant_erepo detail selectors require singular caid",
                None,
            ));
        }
        match crate::entities::variant::retrieve_erepo(
            caids,
            input.detail,
            input.assertion_id.as_deref(),
            input.version.as_deref(),
        )
        .await
        {
            Ok(response) => Ok(CallToolResult::success(vec![Content::text(
                redact_mcp_json_text(&crate::render::json::to_pretty(&response).map_err(
                    |error| {
                        McpError::internal_error(
                            format!("Failed to serialize ERepo response: {error}"),
                            None,
                        )
                    },
                )?)
                .map_err(|error| {
                    McpError::internal_error(
                        format!("Failed to sanitize ERepo response: {error}"),
                        None,
                    )
                })?,
            )])),
            Err(error) => Ok(Self::tool_error(format!("Error: {error}"))),
        }
    }

    /// Retrieve ClinGen CSpec manifests or bounded pages from one captured exact document.
    #[tool(annotations(title = "BioMCP ClinGen CSpec", read_only_hint = true))]
    async fn gene_cspec(
        &self,
        Parameters(input): Parameters<TypedGeneCspec>,
    ) -> Result<CallToolResult, McpError> {
        if input.version_iri.is_some() && input.capture_id.is_some()
            || input.limit == 0
            || input.limit > 50
        {
            return Err(McpError::invalid_params(
                "gene_cspec version_iri and capture_id are mutually exclusive; limit must be 1-50",
                None,
            ));
        }
        let result = match input.capture_id {
            Some(capture_id) => crate::entities::gene::cspec::page_capture(
                &capture_id,
                &input.gene,
                input.offset,
                input.limit,
            )
            .and_then(|response| crate::render::json::to_pretty(&response)),
            None => crate::entities::gene::cspec::retrieve(
                &input.gene,
                input.version_iri.as_deref(),
                input.offset,
                input.limit,
            )
            .await
            .and_then(|response| crate::render::json::to_pretty(&response)),
        };
        match result {
            Ok(text) => Ok(CallToolResult::success(vec![Content::text(
                redact_mcp_json_text(&text).map_err(|error| {
                    McpError::internal_error(
                        format!("Failed to sanitize CSpec response: {error}"),
                        None,
                    )
                })?,
            )])),
            Err(error) => Ok(Self::tool_error(format!("Error: {error}"))),
        }
    }

    /// Retrieve compact variant-literature shortlists for 1-10 structured identities.
    #[tool(annotations(title = "BioMCP variant literature batch", read_only_hint = true))]
    async fn variant_articles(
        &self,
        Parameters(input): Parameters<TypedVariantArticles>,
    ) -> Result<CallToolResult, McpError> {
        if input.items.is_empty() || input.items.len() > 10 {
            return Err(McpError::invalid_params(
                "variant_articles requires between 1 and 10 items",
                None,
            ));
        }
        if input.limit == 0 || input.limit > 50 {
            return Err(McpError::invalid_params(
                "variant_articles limit must be between 1 and 50",
                None,
            ));
        }
        if input.confirmed_only && !input.verify_identity {
            return Err(McpError::invalid_params(
                "variant_articles confirmed_only requires verify_identity",
                None,
            ));
        }
        let strategy = variant_article_strategy(&input.strategy)?;
        match crate::entities::article::search_variant_article_batch_with_options(
            input.items,
            strategy,
            input.limit,
            input.offset,
            input.debug_plan,
            crate::entities::article::VariantArticleVerificationOptions {
                verify_identity: input.verify_identity,
                confirmed_only: input.confirmed_only,
            },
        )
        .await
        {
            Ok(outcome) => {
                let text = crate::render::json::to_pretty(&outcome.response).map_err(|error| {
                    McpError::internal_error(
                        format!("Failed to serialize variant article response: {error}"),
                        None,
                    )
                })?;
                let text = redact_mcp_json_text(&text).map_err(|error| {
                    McpError::internal_error(
                        format!("Failed to sanitize variant article response: {error}"),
                        None,
                    )
                })?;
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(error) => Ok(Self::tool_error(format!("Error: {error}"))),
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
            "BioMCP provides biomedical data from leading public biomedical data sources \
             (PubMed, ClinicalTrials.gov, ClinVar, gnomAD, OncoKB, Reactome, UniProt, \
             PharmGKB, OpenFDA, and more). \
             Prefer the typed `search`, `get`, and `variant_articles` tools for structured entity lookup; use the raw `biomcp` command tool as an escape hatch for long-tail commands. \
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
    let content = crate::render::human::sanitize_document(&content);
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

fn http_allowed_hosts(
    ip: std::net::IpAddr,
    allowed_hosts: Vec<String>,
    unsafe_allow_any_host: bool,
) -> anyhow::Result<Vec<String>> {
    let allowed_hosts = allowed_hosts
        .into_iter()
        .map(|host| host.trim().to_string())
        .filter(|host| !host.is_empty())
        .collect::<Vec<_>>();
    if unsafe_allow_any_host && !allowed_hosts.is_empty() {
        anyhow::bail!("--allowed-hosts cannot be combined with --unsafe-allow-any-host");
    }
    if unsafe_allow_any_host {
        return Ok(Vec::new());
    }
    if !allowed_hosts.is_empty() {
        return Ok(allowed_hosts);
    }
    if ip.is_loopback() {
        return Ok(vec!["localhost".into(), "127.0.0.1".into(), "::1".into()]);
    }
    anyhow::bail!(
        "A non-loopback serve-http bind requires --allowed-hosts or --unsafe-allow-any-host"
    )
}

pub async fn run_http(
    host: &str,
    port: u16,
    allowed_hosts: Vec<String>,
    unsafe_allow_any_host: bool,
) -> anyhow::Result<()> {
    let ip: std::net::IpAddr = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host address: {e}"))?;
    let bind = std::net::SocketAddr::new(ip, port);
    let allowed_hosts = http_allowed_hosts(ip, allowed_hosts, unsafe_allow_any_host)?;
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
    if unsafe_allow_any_host {
        tracing::warn!(
            "Host header checks are disabled; this does not provide authentication or encryption"
        );
    }
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
        BioMcpServer, CACHE_FAMILY_MCP_REJECTION_MESSAGE, GENERIC_MCP_REJECTION_MESSAGE,
        TypedGeneCspec, TypedGet, TypedSearch, TypedVariantArticles, TypedVariantCar,
        VARIANT_ARTICLE_INPUT_MCP_REJECTION_MESSAGE, all_get_sections, binary_download_rejection,
        get_args, get_section_groups, http_allowed_hosts, index_handler, is_allowed_mcp_command,
        mcp_rejection_message, redact_mcp_json_text, redact_mcp_text, search_args,
        subcommand_names, to_resource_result,
    };
    use axum::Json;

    fn section_names_from_sources() -> BTreeSet<&'static str> {
        get_section_groups()
            .iter()
            .flat_map(|group| group.iter().copied())
            .collect()
    }

    #[test]
    fn binary_downloads_are_rejected_but_manifests_remain_allowed() {
        for (args, label) in [
            (
                ["biomcp", "get", "trial", "NCT1", "document", "protocol.pdf"],
                "trial document",
            ),
            (
                ["biomcp", "get", "article", "1", "asset", "table.xlsx"],
                "article asset",
            ),
        ] {
            let args = args.into_iter().map(String::from).collect::<Vec<_>>();
            let message = binary_download_rejection(&args).expect("binary route is rejected");
            assert!(message.contains(label));
            assert!(message.contains("CLI-only"));
            assert!(message.contains("biomcp get"));
        }
        for args in [
            ["biomcp", "get", "trial", "NCT1", "documents"],
            ["biomcp", "get", "article", "1", "assets"],
        ] {
            let args = args.into_iter().map(String::from).collect::<Vec<_>>();
            assert!(binary_download_rejection(&args).is_none());
        }
    }

    #[test]
    fn loopback_http_defaults_to_local_host_headers() {
        let hosts = http_allowed_hosts("127.0.0.1".parse().unwrap(), vec![], false).unwrap();
        assert_eq!(hosts, ["localhost", "127.0.0.1", "::1"]);
    }

    #[test]
    fn non_loopback_http_requires_an_explicit_policy() {
        let error = http_allowed_hosts("0.0.0.0".parse().unwrap(), vec![], false).unwrap_err();
        assert!(error.to_string().contains("--allowed-hosts"));
        assert!(error.to_string().contains("--unsafe-allow-any-host"));

        assert!(
            http_allowed_hosts("0.0.0.0".parse().unwrap(), vec![], true)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            http_allowed_hosts(
                "0.0.0.0".parse().unwrap(),
                vec!["api.example".into()],
                false
            )
            .unwrap(),
            ["api.example"]
        );
    }

    #[test]
    fn mcp_human_error_and_resource_boundaries_remove_terminal_controls() {
        let error = serde_json::to_value(BioMcpServer::tool_error(
            "Error: bad\u{9b}31m identifier\u{202e}",
        ))
        .expect("serialize MCP error");
        let resource = serde_json::to_value(to_resource_result(
            "biomcp://help",
            "# Help\nBad\u{1b}]8;;https://example.test\u{7}label\u{1b}]8;;\u{7}".into(),
        ))
        .expect("serialize MCP resource");

        assert_eq!(error["content"][0]["text"], "Error: bad identifier");
        assert_eq!(resource["contents"][0]["text"], "# Help\nBadlabel");
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
    fn typed_gene_cspec_schema_exposes_bounded_capture_paging_without_raw_bytes() {
        let schema = serde_json::to_value(rmcp::schemars::schema_for!(TypedGeneCspec))
            .expect("CSpec schema");
        let properties = &schema["properties"];

        assert!(properties.get("gene").is_some());
        assert!(properties.get("version_iri").is_some());
        assert!(properties.get("capture_id").is_some());
        assert!(properties.get("offset").is_some());
        assert_eq!(properties["limit"]["minimum"], 1);
        assert_eq!(properties["limit"]["maximum"], 50);
        assert!(properties.get("raw_bytes").is_none());
    }

    #[tokio::test]
    async fn typed_gene_cspec_rejects_version_and_capture_together_before_network_access() {
        let result = BioMcpServer::new()
            .gene_cspec(rmcp::handler::server::wrapper::Parameters(TypedGeneCspec {
                gene: "ATM".into(),
                version_iri: Some("https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1".into()),
                capture_id: Some("capture:cspec:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
                offset: 0,
                limit: 25,
            }))
            .await;

        assert!(
            result.is_err(),
            "mutually exclusive CSpec selectors must fail"
        );
    }

    #[test]
    fn typed_variant_car_schema_is_bounded() {
        let schema =
            serde_json::to_value(rmcp::schemars::schema_for!(TypedVariantCar)).expect("CAR schema");
        assert_eq!(schema["properties"]["inputs"]["minItems"], 1);
        assert_eq!(schema["properties"]["inputs"]["maxItems"], 50);
    }

    #[test]
    fn typed_variant_articles_schema_is_bounded_and_has_structured_identity_fields() {
        let schema = serde_json::to_value(rmcp::schemars::schema_for!(TypedVariantArticles))
            .expect("variant article schema");
        let items = &schema["properties"]["items"];
        assert_eq!(items["minItems"], 1);
        assert_eq!(items["maxItems"], 10);
        let item_schema = &schema["$defs"]["VariantArticleRequest"]["properties"];
        for field in [
            "request_id",
            "gene",
            "protein",
            "coding",
            "transcript",
            "genomic",
            "accession",
            "build",
            "position",
            "ref",
            "alt",
            "rsid",
        ] {
            assert!(item_schema.get(field).is_some(), "missing field {field}");
        }
        assert_eq!(schema["properties"]["limit"]["minimum"], 1);
        assert_eq!(schema["properties"]["limit"]["maximum"], 50);
    }

    #[test]
    fn typed_variant_articles_preserves_article_resolution_fields_and_nullability() {
        let text = redact_mcp_json_text(
            r#"{"items":[{"resolution":{"status":"resolved","basis":"caller_supplied","exhaustive":true,"normalized_aliases":{"protein_changes":[],"coding_changes":["c.1066-6T>G"],"genomic_ids":["NC_000011.10:g.108248927T>G"],"rsids":[]},"provider_validation":{"source":"myvariant","status":"not_found","matched_alias":null,"contradictory_field":null}},"canonical_equivalence":{"status":"confirmed","caid":"CA900000000002","exhaustive":true,"complete":true,"applicable_identity_count":2,"observations":[{"basis":"transcript_coding","query":"NM_000051.4:c.1066-6T>G","status":"resolved","caid":"CA900000000002","provider_exhaustive":true,"comparison_complete":true,"source":"clingen_car","request_template_version":"1","car_version":null,"provider_response_sha256":"23930aafbb13d87cda75bba884ca09a706e4112a029c71416fc0b669fedae75d"}],"message":"all independently supplied CAR identities resolved to one CAid"}}]}"#,
        )
        .expect("valid MCP JSON");
        let response: serde_json::Value = serde_json::from_str(&text).expect("response JSON");
        let resolution = &response["items"][0]["resolution"];
        assert_eq!(resolution["basis"], "caller_supplied");
        assert_eq!(resolution["provider_validation"]["status"], "not_found");
        assert!(resolution["provider_validation"]["matched_alias"].is_null());
        assert!(resolution["provider_validation"]["contradictory_field"].is_null());
        let equivalence = &response["items"][0]["canonical_equivalence"];
        assert_eq!(equivalence["status"], "confirmed");
        assert_eq!(equivalence["observations"][0]["basis"], "transcript_coding");
        assert!(equivalence["observations"][0]["car_version"].is_null());
        assert_eq!(
            equivalence["observations"][0]["provider_response_sha256"],
            "23930aafbb13d87cda75bba884ca09a706e4112a029c71416fc0b669fedae75d"
        );
    }

    #[tokio::test]
    async fn typed_variant_articles_executes_in_memory_without_stdin_or_paths() {
        let items = serde_json::from_value(serde_json::json!([
            {"request_id":"invalid","gene":"BRAF"}
        ]))
        .expect("typed variant requests");
        let result = BioMcpServer::new()
            .variant_articles(rmcp::handler::server::wrapper::Parameters(
                TypedVariantArticles {
                    items,
                    strategy: "union".into(),
                    limit: 3,
                    offset: 0,
                    debug_plan: true,
                    verify_identity: false,
                    confirmed_only: false,
                },
            ))
            .await
            .expect("typed MCP response");
        let value = serde_json::to_value(result).expect("MCP result JSON");
        let text = value["content"][0]["text"].as_str().expect("text response");
        let response: serde_json::Value = serde_json::from_str(text).expect("response JSON");

        assert_eq!(response["items"][0]["request_id"], "invalid");
        assert_eq!(response["items"][0]["resolution"], serde_json::Value::Null);
        assert!(response["items"][0]["debug_plan"].is_object());
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
            "variant".into(),
            "articles".into(),
            "BRAF p.V600E".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "variant".into(),
            "articles".into(),
            "--input".into(),
            "/server/private.json".into()
        ]));
        assert!(!is_allowed_mcp_command(&[
            "biomcp".into(),
            "variant".into(),
            "articles".into(),
            "--input=-".into()
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
            "status".into(),
            "/home/operator/private/skills".into()
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
    fn raw_variant_article_input_rejection_directs_callers_to_the_typed_tool() {
        for input in [["--input", "/server/private.json"], ["--input=-", ""]] {
            let mut args = vec!["biomcp".into(), "variant".into(), "articles".into()];
            args.extend(
                input
                    .into_iter()
                    .filter(|value| !value.is_empty())
                    .map(String::from),
            );
            assert_eq!(
                mcp_rejection_message(&args),
                VARIANT_ARTICLE_INPUT_MCP_REJECTION_MESSAGE
            );
        }
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

        let private = "/home/operator/private/skills";
        let status_args = vec![
            "biomcp".into(),
            "skill".into(),
            "status".into(),
            private.into(),
        ];
        let message = mcp_rejection_message(&status_args);
        assert_eq!(message, GENERIC_MCP_REJECTION_MESSAGE);
        assert!(!message.contains(private));
    }

    #[tokio::test]
    async fn index_handler_reports_streamable_http_surface() {
        let Json(payload) = index_handler().await;
        assert_eq!(payload["name"], "biomcp");
        assert_eq!(payload["transport"], "streamable-http");
        assert_eq!(payload["mcp"], "/mcp");
    }
}
