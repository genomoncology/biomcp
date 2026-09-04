use std::collections::BTreeSet;
use std::future::Future;
use std::time::Duration;

use base64::Engine;
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{
    AnnotateAble, CallToolResult, Content, Implementation, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, RawResource, ReadResourceRequestParams, ReadResourceResult,
    ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::schemars;
use rmcp::service::RequestContext;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

mod typed_get;
use self::typed_get::{typed_get_capabilities, typed_get_schema};
mod http_server;
pub(super) use self::http_server::run_http;
mod pre_session;
mod trial_phase;
#[derive(Debug, Clone)]
pub struct BioMcpServer {
    pub(super) tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ShellCommand {
    command: String,
    #[serde(default)]
    json: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
#[schemars(transform = typed_search_schema)]
struct TypedSearch(Value);

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
#[schemars(transform = typed_get_schema)]
struct TypedGet(Value);

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TypedVariantCar {
    #[schemars(length(min = 1, max = 50))]
    inputs: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(transform = typed_variant_erepo_schema)]
struct TypedVariantErepo {
    #[serde(default)]
    caid: Option<String>,
    #[serde(default)]
    #[schemars(length(min = 1, max = 50))]
    caids: Option<Vec<String>>,
    #[serde(default)]
    gene: Option<String>,
    #[serde(default = "default_cspec_limit")]
    #[schemars(range(min = 1, max = 100))]
    limit: usize,
    #[serde(default)]
    offset: usize,
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
    files: bool,
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

fn short_string_schema() -> Value {
    json!({"type":"string","minLength":1,"maxLength":256})
}

fn string_array_schema() -> Value {
    json!({"type":"array","minItems":1,"maxItems":3,"uniqueItems":true,"items":short_string_schema()})
}

fn typed_search_branch(entity: &str) -> Value {
    let (fields, required): (&[(&str, &str)], &[&str]) = match entity {
        "author" => (
            &[("query", "text"), ("source", "author_source")],
            &["query"],
        ),
        "gene" => (
            &[
                ("query", "text"),
                ("gene_type", "text"),
                ("chromosome", "text"),
                ("region", "text"),
            ],
            &["query", "gene_type", "chromosome", "region"],
        ),
        "pgx" => (
            &[("gene", "text"), ("drug", "text"), ("cpic_level", "cpic")],
            &["gene", "drug"],
        ),
        "gwas" => (
            &[
                ("gene", "text"),
                ("trait", "text"),
                ("p_value", "probability"),
            ],
            &["gene", "trait"],
        ),
        "article" => (
            &[
                ("keyword", "array"),
                ("gene", "text"),
                ("disease", "array"),
                ("drug", "array"),
                ("author", "array"),
                ("journal", "array"),
                ("date_from", "date"),
                ("date_to", "date"),
                ("article_type", "article_type"),
                ("source", "article_source"),
                ("open_access", "bool"),
                ("no_preprints", "bool"),
                ("sort", "sort"),
            ],
            &["keyword", "gene", "disease", "drug", "author"],
        ),
        "trial" => (
            &[
                ("condition", "array"),
                ("intervention", "array"),
                ("mutation", "array"),
                ("criteria", "array"),
                ("biomarker", "array"),
                ("phase", "phase"),
                ("status", "status"),
                ("source", "trial_source"),
            ],
            &[
                "condition",
                "intervention",
                "mutation",
                "criteria",
                "biomarker",
            ],
        ),
        "variant" => (
            &[
                ("query", "text"),
                ("gene", "text"),
                ("hgvsp", "text"),
                ("significance", "text"),
                ("max_frequency", "unit"),
                ("consequence", "text"),
                ("review_status", "review"),
                ("revel_min", "unit"),
            ],
            &["query", "gene", "hgvsp"],
        ),
        "protein" => (
            &[
                ("query", "text"),
                ("all_species", "bool"),
                ("reviewed", "bool"),
                ("disease", "text"),
                ("existence", "existence"),
            ],
            &["query"],
        ),
        _ => unreachable!(),
    };
    let mut properties = serde_json::Map::from_iter([
        ("entity".into(), json!({"const":entity})),
        (
            "limit".into(),
            json!({"type":"integer","minimum":1,"maximum":25,"default":10}),
        ),
        (
            "offset".into(),
            json!({"type":"integer","minimum":0,"maximum":1000,"default":0}),
        ),
        ("json".into(), json!({"type":"boolean","default":false})),
    ]);
    for &(name, kind) in fields {
        let value = match kind {
            "array" => string_array_schema(),
            "bool" => json!({"type":"boolean"}),
            "probability" => json!({"type":"number","exclusiveMinimum":0,"maximum":1}),
            "unit" => json!({"type":"number","minimum":0,"maximum":1}),
            "existence" => json!({"type":"integer","minimum":1,"maximum":5}),
            "date" => json!({"type":"string","pattern":"^[0-9]{4}(-[0-9]{2}(-[0-9]{2})?)?$"}),
            "author_source" => json!({"const":"semanticscholar"}),
            "cpic" => json!({"enum":["A","B","C","D"]}),
            "article_type" => {
                json!({"enum":["research-article","review","case-reports","meta-analysis"]})
            }
            "article_source" => {
                json!({"enum":["all","pubtator","europepmc","pubmed","semanticscholar","litsense2"]})
            }
            "sort" => json!({"enum":["date","citations","relevance"]}),
            "phase" => trial_phase::schema(),
            "status" => {
                json!({"enum":["recruiting","not_yet_recruiting","enrolling_by_invitation","active_not_recruiting","completed","suspended","terminated","withdrawn"]})
            }
            "trial_source" => json!({"enum":["ctgov","nci"]}),
            "review" => {
                json!({"enum":["0","1","2","3","4","none","criteria_provided","expert_panel"]})
            }
            _ => short_string_schema(),
        };
        properties.insert(name.into(), value);
    }
    let any_of = required
        .iter()
        .map(|name| json!({"required":[name]}))
        .collect::<Vec<_>>();
    json!({"type":"object","additionalProperties":false,"properties":properties,"required":["entity"],"anyOf":any_of})
}

fn typed_search_schema(schema: &mut schemars::Schema) {
    let branches = [
        "author", "gene", "pgx", "gwas", "article", "trial", "variant", "protein",
    ]
    .into_iter()
    .map(typed_search_branch)
    .collect::<Vec<_>>();
    *schema = serde_json::from_value(json!({"oneOf":branches})).expect("valid typed search schema");
}

fn typed_variant_erepo_schema(schema: &mut schemars::Schema) {
    let branches = [
        json!({"type":"object","additionalProperties":false,"properties":{"caid":{"type":"string","minLength":1},"detail":{"type":"boolean"},"assertion_id":{"type":"string"},"version":{"type":"string"}},"required":["caid"]}),
        json!({"type":"object","additionalProperties":false,"properties":{"caids":{"type":"array","minItems":1,"maxItems":50,"items":{"type":"string","minLength":1}}},"required":["caids"]}),
        json!({"type":"object","additionalProperties":false,"properties":{"gene":{"type":"string","minLength":1},"limit":{"type":"integer","minimum":1,"maximum":100,"default":25},"offset":{"type":"integer","minimum":0,"default":0}},"required":["gene"]}),
    ];
    *schema = serde_json::from_value(json!({"type":"object","oneOf":branches}))
        .expect("valid typed ERepo schema");
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
const LOCAL_INPUT_MCP_REJECTION_MESSAGE: &str = "Error: --input file and stdin arguments are CLI-only over raw MCP because they read server-local state; use the matching typed MCP tool instead.";
impl BioMcpServer {
    pub fn new() -> Self {
        let mut tool_router = Self::tool_router();
        super::catalog::apply(&mut tool_router);
        Self { tool_router }
    }

    fn tool_error(message: impl Into<String>) -> CallToolResult {
        let message = crate::render::human::sanitize_inline(&message.into());
        CallToolResult::error(vec![Content::text(message)])
    }

    async fn execute_args(args: Vec<String>, json: bool) -> Result<CallToolResult, McpError> {
        let cli = match crate::cli::try_parse_cli(args.clone()) {
            Ok(cli) => cli,
            Err(error) => return Ok(Self::tool_error(format!("Error: {error}"))),
        };
        Self::execute_cli(cli, args, json).await
    }

    async fn execute_cli(
        cli: crate::cli::Cli,
        args: Vec<String>,
        json: bool,
    ) -> Result<CallToolResult, McpError> {
        if let Some(message) = binary_download_rejection(&cli, &args) {
            return Ok(Self::tool_error(message));
        }
        let command_requests_json = json || cli.json;
        let may_return_article_fulltext = cli_may_return_article_fulltext(&cli);
        match crate::cli::execute_mcp_cli(cli).await {
            Ok(output) => {
                let text = if command_requests_json {
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
                            Err(err) if may_return_article_fulltext => {
                                return Err(McpError::internal_error(
                                    format!(
                                        "Failed to inspect MCP full-text response fields: {err}"
                                    ),
                                    None,
                                ));
                            }
                            Err(_) => output.text,
                        },
                        None if may_return_article_fulltext => {
                            return Err(McpError::internal_error(
                                "Failed to prepare safe MCP full-text response metadata",
                                None,
                            ));
                        }
                        None => output.text,
                    }
                };
                let text = if command_requests_json {
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

fn binary_download_rejection(cli: &crate::cli::Cli, args: &[String]) -> Option<String> {
    let (entity, section) = match &cli.command {
        crate::cli::Commands::Get {
            entity: crate::cli::GetEntity::Trial(args),
        } if args
            .sections
            .first()
            .is_some_and(|section| section == "document") =>
        {
            ("trial", "document")
        }
        crate::cli::Commands::Get {
            entity: crate::cli::GetEntity::Article(args),
        } if args
            .sections
            .first()
            .is_some_and(|section| section == "asset") =>
        {
            ("article", "asset")
        }
        _ => return None,
    };
    Some(binary_download_message(entity, section, args))
}

fn binary_download_message(entity: &str, section: &str, args: &[String]) -> String {
    let command = args
        .iter()
        .take_while(|arg| !matches!(arg.as_str(), "--json" | "-j"))
        .map(|arg| shlex::try_quote(arg).unwrap_or_else(|_| "<value>".into()))
        .collect::<Vec<_>>()
        .join(" ");
    format!("Binary {entity} {section} downloads are CLI-only. Run `{command}` from a terminal.")
}

impl Default for BioMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

fn variant_command_reads_local_input(command: &crate::cli::VariantCommand) -> bool {
    match command {
        crate::cli::VariantCommand::Articles { input, .. }
        | crate::cli::VariantCommand::Erepo { input, .. }
        | crate::cli::VariantCommand::Normalize { input, .. } => input.is_some(),
        crate::cli::VariantCommand::Trials { .. }
        | crate::cli::VariantCommand::Structure { .. }
        | crate::cli::VariantCommand::Oncokb { .. }
        | crate::cli::VariantCommand::External(_) => false,
    }
}

fn is_allowed_mcp_command(cli: &crate::cli::Cli) -> bool {
    use crate::cli::{
        ArticleCommand, Commands, DiseaseCommand, DrugCommand, GeneCommand, PathwayCommand,
        ProteinCommand, StudyCommand,
    };

    match &cli.command {
        Commands::Search { .. } | Commands::Get { .. } | Commands::Author { .. } => true,
        Commands::Variant { cmd } => !variant_command_reads_local_input(cmd),
        Commands::Drug {
            cmd:
                DrugCommand::Trials { .. }
                | DrugCommand::AdverseEvents { .. }
                | DrugCommand::Interactions { .. },
        }
        | Commands::Disease {
            cmd:
                DiseaseCommand::Trials { .. }
                | DiseaseCommand::Articles { .. }
                | DiseaseCommand::Drugs { .. },
        }
        | Commands::Article {
            cmd:
                ArticleCommand::Authors { .. }
                | ArticleCommand::Entities { .. }
                | ArticleCommand::Batch { .. }
                | ArticleCommand::Citations { .. }
                | ArticleCommand::References { .. }
                | ArticleCommand::Recommendations { .. },
        }
        | Commands::Gene {
            cmd:
                GeneCommand::Definition { .. }
                | GeneCommand::Trials { .. }
                | GeneCommand::Drugs { .. }
                | GeneCommand::Articles { .. }
                | GeneCommand::Pathways { .. }
                | GeneCommand::Cspec(_),
        }
        | Commands::Pathway {
            cmd:
                PathwayCommand::Drugs { .. }
                | PathwayCommand::Articles { .. }
                | PathwayCommand::Trials { .. },
        }
        | Commands::Protein {
            cmd: ProteinCommand::Structures { .. },
        }
        | Commands::Health(_)
        | Commands::List(_)
        | Commands::Batch(_)
        | Commands::Enrich(_)
        | Commands::Discover(_)
        | Commands::Version(_) => true,
        Commands::Study {
            cmd:
                StudyCommand::List
                | StudyCommand::TopMutated { .. }
                | StudyCommand::Query { .. }
                | StudyCommand::Filter { .. }
                | StudyCommand::Cohort { .. }
                | StudyCommand::Survival { .. }
                | StudyCommand::Compare { .. }
                | StudyCommand::CoOccurrence { .. },
        }
        | Commands::Study {
            cmd: StudyCommand::Download { list: true, .. },
        } => true,
        Commands::Study {
            cmd: StudyCommand::Download { list: false, .. },
        } => false,
        Commands::Skill { command: None }
        | Commands::Skill {
            command:
                Some(crate::cli::skill::SkillCommand::List | crate::cli::skill::SkillCommand::Render),
        } => true,
        Commands::Skill {
            command: Some(crate::cli::skill::SkillCommand::Show(parts)),
        } => parts.len() == 1 && crate::cli::skill::show_use_case(&parts[0]).is_ok(),
        Commands::Skill {
            command:
                Some(crate::cli::skill::SkillCommand::Status { .. })
                | Some(crate::cli::skill::SkillCommand::Install { .. }),
        }
        | Commands::Cache { .. }
        | Commands::Ema { .. }
        | Commands::Who { .. }
        | Commands::Cvx { .. }
        | Commands::Ddinter { .. }
        | Commands::Gtr { .. }
        | Commands::WhoIvd { .. }
        | Commands::Mcp(_)
        | Commands::Serve
        | Commands::McpConfig(_)
        | Commands::ServeHttp(_)
        | Commands::ServeSse
        | Commands::Chart { .. }
        | Commands::Update(_)
        | Commands::Uninstall
        | Commands::Drug {
            cmd: DrugCommand::External(_),
        }
        | Commands::Gene {
            cmd: GeneCommand::External(_),
        } => false,
    }
}

fn mcp_rejection_message(cli: &crate::cli::Cli) -> &'static str {
    match &cli.command {
        crate::cli::Commands::Cache { .. } => CACHE_FAMILY_MCP_REJECTION_MESSAGE,
        crate::cli::Commands::Variant { cmd } if variant_command_reads_local_input(cmd) => {
            LOCAL_INPUT_MCP_REJECTION_MESSAGE
        }
        _ => GENERIC_MCP_REJECTION_MESSAGE,
    }
}

fn args_with_json(mut args: Vec<String>) -> Vec<String> {
    if !args
        .iter()
        .any(|arg| matches!(arg.as_str(), "--json" | "-j"))
    {
        args.push("--json".to_string());
    }
    args
}

fn cli_may_return_article_fulltext(cli: &crate::cli::Cli) -> bool {
    matches!(
        cli.command,
        crate::cli::Commands::Get {
            entity: crate::cli::GetEntity::Article(_),
        }
    )
}

#[cfg(test)]
fn is_allowed_mcp_args(args: &[String]) -> bool {
    crate::cli::try_parse_cli(args.to_vec()).is_ok_and(|cli| is_allowed_mcp_command(&cli))
}

#[cfg(test)]
fn mcp_rejection_message_for_args(args: &[String]) -> &'static str {
    let cli = crate::cli::try_parse_cli(args.to_vec()).expect("test command parses");
    mcp_rejection_message(&cli)
}

#[cfg(test)]
fn binary_download_rejection_for_args(args: &[String]) -> Option<String> {
    crate::cli::try_parse_cli(args.to_vec())
        .ok()
        .and_then(|cli| binary_download_rejection(&cli, args))
}

fn input_error(message: impl Into<String>) -> McpError {
    McpError::invalid_params(message.into(), None)
}

fn checked_text(value: &Value, field: &str, max: usize) -> Result<String, McpError> {
    let text = value
        .as_str()
        .ok_or_else(|| input_error(format!("{field} must be a string")))?
        .trim();
    if text.is_empty() || text.chars().count() > max {
        return Err(input_error(format!(
            "{field} must contain 1-{max} characters"
        )));
    }
    Ok(text.into())
}

fn search_args(input: TypedSearch) -> Result<Vec<String>, McpError> {
    let object = input
        .0
        .as_object()
        .ok_or_else(|| input_error("typed search input must be an object"))?;
    let entity = checked_text(object.get("entity").unwrap_or(&Value::Null), "entity", 256)?;
    if ![
        "author", "gene", "pgx", "gwas", "article", "trial", "variant", "protein",
    ]
    .contains(&entity.as_str())
    {
        return Err(input_error("invalid typed search entity"));
    }
    let branch = typed_search_branch(&entity);
    let allowed = branch["properties"].as_object().expect("branch properties");
    if let Some(key) = object.keys().find(|key| !allowed.contains_key(*key)) {
        return Err(input_error(format!("unknown {entity} search field: {key}")));
    }
    let required = branch["anyOf"].as_array().expect("required choices");
    if !required.iter().any(|choice| {
        choice["required"][0]
            .as_str()
            .is_some_and(|key| object.contains_key(key))
    }) {
        return Err(input_error(format!(
            "{entity} search requires at least one identity field"
        )));
    }
    let limit = object.get("limit").and_then(Value::as_u64).unwrap_or(10) as usize;
    let offset = object.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
    if !(1..=25).contains(&limit)
        || offset > 1000
        || (entity == "gwas" && offset.checked_add(limit).is_none_or(|end| end > 50))
    {
        return Err(input_error(
            "typed search pagination is outside its supported bounds",
        ));
    }
    if entity == "trial" && object.get("source").and_then(Value::as_str) == Some("nci") {
        let nci = ["mutation", "criteria", "biomarker"]
            .into_iter()
            .filter(|key| object.contains_key(*key))
            .collect::<Vec<_>>();
        if nci.len() > 1
            || nci.first().is_some_and(|key| {
                object[*key]
                    .as_array()
                    .is_none_or(|values| values.len() != 1)
            })
        {
            return Err(input_error(
                "NCI typed search accepts exactly one mutation, criteria, or biomarker value",
            ));
        }
    }
    let mut args = vec!["biomcp".into(), "search".into(), entity.clone()];
    for (field, value) in object {
        if matches!(field.as_str(), "entity" | "limit" | "offset" | "json") {
            continue;
        }
        let field_schema = &allowed[field];
        if let Some(values) = field_schema.get("enum").and_then(Value::as_array)
            && !values.contains(value)
        {
            return Err(input_error(format!("invalid {field} value")));
        }
        if field_schema.get("type").and_then(Value::as_str) == Some("boolean")
            && !value.is_boolean()
        {
            return Err(input_error(format!("{field} must be a boolean")));
        }
        if field_schema.get("type").and_then(Value::as_str) == Some("number") {
            let number = value
                .as_f64()
                .filter(|number| number.is_finite())
                .ok_or_else(|| input_error(format!("{field} must be a finite number")))?;
            let minimum = field_schema.get("minimum").and_then(Value::as_f64);
            let exclusive = field_schema.get("exclusiveMinimum").and_then(Value::as_f64);
            let maximum = field_schema.get("maximum").and_then(Value::as_f64);
            if minimum.is_some_and(|min| number < min)
                || exclusive.is_some_and(|min| number <= min)
                || maximum.is_some_and(|max| number > max)
            {
                return Err(input_error(format!(
                    "{field} is outside its supported range"
                )));
            }
        }
        let flag = match (entity.as_str(), field.as_str()) {
            ("gene", "gene_type") | ("article", "article_type") => "--type",
            ("gwas", "trait") => "--trait",
            ("gwas", "p_value") => "--p-value",
            ("variant", "max_frequency") => "--max-frequency",
            ("variant", "review_status") => "--review-status",
            ("variant", "revel_min") => "--revel-min",
            ("pgx", "cpic_level") => "--cpic-level",
            ("article", "date_from") => "--date-from",
            ("article", "date_to") => "--date-to",
            ("article", "open_access") => "--open-access",
            ("article", "no_preprints") => "--no-preprints",
            ("protein", "all_species") => "--all-species",
            (_, "query") if matches!(entity.as_str(), "gene" | "variant") => "",
            (_, name) => Box::leak(format!("--{}", name.replace('_', "-")).into_boxed_str()),
        };
        if value.is_boolean() {
            if value.as_bool() == Some(true) {
                args.push(flag.into());
            }
        } else if let Some(values) = value.as_array() {
            if values.is_empty() || values.len() > 3 {
                return Err(input_error(format!("{field} must contain 1-3 values")));
            }
            let mut seen = BTreeSet::new();
            for value in values {
                let text = checked_text(value, field, 256)?;
                if !seen.insert(text.clone()) {
                    return Err(input_error(format!("{field} values must be unique")));
                }
                args.extend([flag.into(), text]);
            }
        } else {
            let text = if value.is_string() {
                checked_text(value, field, 256)?
            } else {
                value.to_string()
            };
            if flag.is_empty() {
                args.push(text);
            } else {
                args.extend([flag.into(), text]);
            }
        }
    }
    args.extend(["--limit".into(), limit.to_string()]);
    if offset > 0 {
        args.extend(["--offset".into(), offset.to_string()]);
    }
    if object.get("json").and_then(Value::as_bool) == Some(true) {
        args = args_with_json(args);
    }
    Ok(args)
}

/// Maps typed get input onto the existing CLI grammar without doing provider work.
///
/// The accepted entity, section, and duplicate policies come from the same MCP
/// capability projection that generates the schema. That projection starts from
/// the CLI catalog but deliberately excludes article `asset`: it is a variadic
/// binary download that can name server-local output and is therefore CLI-only.
/// The safe article `assets` manifest remains an ordinary typed section. Trial
/// terminal document forms remain outside the typed projection as well. Their
/// explicit checks below run before ordinary section validation so callers keep
/// the established CLI-only guidance instead of receiving a generic bad-section
/// error. Adverse-event's repeated-section behavior remains intentionally
/// idempotent; all other section-bearing entities reject duplicates.
fn get_args(input: TypedGet) -> Result<Vec<String>, McpError> {
    let object = input
        .0
        .as_object()
        .ok_or_else(|| input_error("typed get input must be an object"))?;
    let entity = checked_text(object.get("entity").unwrap_or(&Value::Null), "entity", 256)?;
    let capabilities = typed_get_capabilities();
    let capability = capabilities
        .iter()
        .find(|capability| capability.entity == entity)
        .ok_or_else(|| input_error("invalid typed get entity"))?;
    let id = checked_text(object.get("id").unwrap_or(&Value::Null), "id", 512)?;
    let allowed_keys = if capability.sections.is_none() {
        &["entity", "id", "json"][..]
    } else if entity == "variant" {
        &["entity", "id", "sections", "assembly", "json"][..]
    } else {
        &["entity", "id", "sections", "json"][..]
    };
    if let Some(key) = object
        .keys()
        .find(|key| !allowed_keys.contains(&key.as_str()))
    {
        return Err(input_error(format!("unknown {entity} get field: {key}")));
    }
    let mut args = vec!["biomcp".into(), "get".into(), entity.clone()];
    if let Some(assembly) = object.get("assembly") {
        let assembly = checked_text(assembly, "assembly", 256)?;
        if !["grch37", "hg19", "grch38", "hg38"].contains(&assembly.as_str()) {
            return Err(input_error("invalid variant assembly"));
        }
        args.extend(["--assembly".into(), assembly]);
    }
    args.push(id);
    let sections = object
        .get("sections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if sections.len() > 16 {
        return Err(input_error("sections accepts at most 16 values"));
    }
    let sections = sections
        .iter()
        .map(|section| checked_text(section, "section", 256))
        .collect::<Result<Vec<_>, _>>()?;
    if matches!(
        (entity.as_str(), sections.first().map(String::as_str)),
        ("trial", Some("document")) | ("article", Some("asset"))
    ) {
        let section = sections.first().expect("binary section").clone();
        args.extend(sections);
        return Err(McpError::invalid_params(
            binary_download_message(entity.as_str(), &section, &args),
            None,
        ));
    }
    let allowed_sections = capability.sections.as_deref().unwrap_or_default();
    let mut seen = BTreeSet::new();
    for section in sections {
        if !allowed_sections.contains(&section.as_str()) {
            return Err(input_error(format!("invalid {entity} section: {section}")));
        }
        if !seen.insert(section.clone()) {
            if !capability.reject_duplicate_sections {
                continue;
            }
            return Err(input_error(format!(
                "duplicate {entity} section: {section}"
            )));
        }
        args.push(section);
    }
    if object.get("json").and_then(Value::as_bool) == Some(true) {
        args = args_with_json(args);
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
    #[tool]
    async fn biomcp(
        &self,
        Parameters(ShellCommand { command, json }): Parameters<ShellCommand>,
    ) -> Result<CallToolResult, McpError> {
        if command.len() > 1024 {
            return Ok(Self::tool_error("Error: command is too long"));
        }

        let split = match shlex::split(&command) {
            Some(args) => args,
            None => return Ok(Self::tool_error(GENERIC_MCP_REJECTION_MESSAGE)),
        };

        let mut args = vec!["biomcp".to_string()];
        if split.first().is_some_and(|s| s == "biomcp") {
            args.extend(split.into_iter().skip(1));
        } else {
            args.extend(split);
        }
        if json {
            args = args_with_json(args);
        }

        let cli = match crate::cli::try_parse_cli(args.clone()) {
            Ok(cli) => cli,
            Err(_) => return Ok(Self::tool_error(GENERIC_MCP_REJECTION_MESSAGE)),
        };
        if !is_allowed_mcp_command(&cli) {
            return Ok(Self::tool_error(mcp_rejection_message(&cli)));
        }

        Self::execute_cli(cli, args, json).await
    }

    #[tool]
    async fn search(
        &self,
        Parameters(input): Parameters<TypedSearch>,
    ) -> Result<CallToolResult, McpError> {
        let json = input
            .0
            .get("json")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let args = search_args(input)?;
        Self::execute_args(args, json).await
    }

    #[tool]
    async fn get(
        &self,
        Parameters(input): Parameters<TypedGet>,
    ) -> Result<CallToolResult, McpError> {
        let json = input
            .0
            .get("json")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let args = get_args(input)?;
        Self::execute_args(args, json).await
    }

    #[tool]
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

    #[tool]
    async fn variant_erepo(
        &self,
        Parameters(input): Parameters<TypedVariantErepo>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(gene) = input.gene {
            if input.caid.is_some()
                || input.caids.is_some()
                || input.detail
                || input.assertion_id.is_some()
                || input.version.is_some()
                || input.limit == 0
                || input.limit > 100
            {
                return Err(McpError::invalid_params(
                    "variant_erepo gene mode cannot use CAID or detail selectors; limit must be 1-100",
                    None,
                ));
            }
            return match crate::entities::variant::search_erepo_gene(
                &gene,
                input.limit,
                input.offset,
            )
            .await
            {
                Ok(response) => Ok(CallToolResult::success(vec![Content::text(
                    crate::render::json::to_pretty(&response).map_err(|error| {
                        McpError::internal_error(
                            format!("Failed to serialize ERepo response: {error}"),
                            None,
                        )
                    })?,
                )])),
                Err(error) => Ok(Self::tool_error(format!("Error: {error}"))),
            };
        }
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

    #[tool]
    async fn gene_cspec(
        &self,
        Parameters(input): Parameters<TypedGeneCspec>,
    ) -> Result<CallToolResult, McpError> {
        if input.version_iri.is_some() && input.capture_id.is_some()
            || input.limit == 0
            || input.limit > 50
            || input.files && input.version_iri.is_none() && input.capture_id.is_none()
        {
            return Err(McpError::invalid_params(
                "gene_cspec version_iri and capture_id are mutually exclusive; files requires one of them; limit must be 1-50",
                None,
            ));
        }
        let result = if input.files {
            match input.capture_id {
                Some(capture_id) => {
                    crate::entities::gene::cspec::files_capture(&capture_id, &input.gene)
                        .and_then(|response| crate::render::json::to_pretty(&response))
                }
                None => crate::entities::gene::cspec::retrieve_files(
                    &input.gene,
                    input.version_iri.as_deref().expect("validated version"),
                )
                .await
                .and_then(|response| crate::render::json::to_pretty(&response)),
            }
        } else {
            match input.capture_id {
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
            }
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

    #[tool]
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
        .with_instructions(super::catalog::instructions())
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(super::catalog::list(
            &self.tool_router,
        ))))
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

pub(super) fn read_resource_markdown(uri: &str) -> Result<ReadResourceResult, McpError> {
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

pub(super) fn build_resource_list() -> Vec<RawResource> {
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

pub async fn run_stdio() -> anyhow::Result<()> {
    let shutdown = CancellationToken::new();

    let cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.cancel();
        }
    });
    let startup = tokio::time::timeout(Duration::from_secs(5), async {
        let transport = pre_session::stdio_transport().await?;
        BioMcpServer::new()
            .serve_with_ct(transport, shutdown)
            .await
            .map_err(anyhow::Error::new)
    })
    .await;
    let running = match startup {
        Ok(Ok(running)) => running,
        Ok(Err(err)) => {
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

#[cfg(test)]
#[path = "shell/typed_get_tests.rs"]
mod typed_get_tests;

#[cfg(test)]
mod tests {
    use super::{
        BioMcpServer, CACHE_FAMILY_MCP_REJECTION_MESSAGE, GENERIC_MCP_REJECTION_MESSAGE,
        LOCAL_INPUT_MCP_REJECTION_MESSAGE, ShellCommand, TypedGeneCspec, TypedGet, TypedSearch,
        TypedVariantArticles, TypedVariantCar, binary_download_rejection_for_args,
        cli_may_return_article_fulltext, get_args, is_allowed_mcp_args,
        mcp_rejection_message_for_args, redact_mcp_json_text, redact_mcp_text, search_args,
    };
    use serde_json::json;
    mod ticket_1120;

    #[test]
    fn binary_downloads_are_rejected_but_manifests_remain_allowed() {
        for (args, label) in [
            (
                [
                    "biomcp",
                    "get",
                    "--json",
                    "trial",
                    "NCT1",
                    "document",
                    "protocol.pdf",
                ],
                "trial document",
            ),
            (
                [
                    "biomcp",
                    "get",
                    "--no-cache",
                    "article",
                    "1",
                    "asset",
                    "table.xlsx",
                ],
                "article asset",
            ),
        ] {
            let args = args.into_iter().map(String::from).collect::<Vec<_>>();
            let message =
                binary_download_rejection_for_args(&args).expect("binary route is rejected");
            assert!(message.contains(label));
            assert!(message.contains("CLI-only"));
            assert!(message.contains("biomcp get"));
        }
        for args in [
            ["biomcp", "get", "trial", "NCT1", "documents"],
            ["biomcp", "get", "article", "1", "assets"],
        ] {
            let args = args.into_iter().map(String::from).collect::<Vec<_>>();
            assert!(binary_download_rejection_for_args(&args).is_none());
        }

        let error = get_args(TypedGet(json!({
            "entity": "article",
            "id": "22663011",
            "sections": ["asset", "supplement.xlsx"]
        })))
        .expect_err("typed binary route is rejected before section validation");
        assert!(error.to_string().contains("CLI-only"));
    }

    #[tokio::test]
    async fn raw_mcp_rejects_unparseable_commands_before_execution() {
        let result = BioMcpServer::new()
            .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
                command: "biomcp get --json article".into(),
                json: false,
            }))
            .await
            .expect("raw MCP rejection");
        let value = serde_json::to_value(result).expect("serialize raw MCP result");
        let text = value["content"][0]["text"]
            .as_str()
            .expect("raw MCP rejection text");

        assert_eq!(text, GENERIC_MCP_REJECTION_MESSAGE);
    }

    #[test]
    fn mcp_human_error_and_resource_boundaries_remove_terminal_controls() {
        let error = serde_json::to_value(BioMcpServer::tool_error(
            "Error: bad\u{9b}31m identifier\u{202e}",
        ))
        .expect("serialize MCP error");
        let resource = serde_json::to_value(super::to_resource_result(
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
        let cli = crate::cli::try_parse_cli([
            "biomcp",
            "get",
            "--no-cache",
            "article",
            "22663011",
            "fulltext",
        ])
        .expect("full-text command parses");
        assert!(cli_may_return_article_fulltext(&cli));

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
    fn typed_schemas_are_entity_specific() {
        let search = serde_json::to_value(rmcp::schemars::schema_for!(TypedSearch)).unwrap();
        assert_eq!(search["oneOf"].as_array().unwrap().len(), 8);
        let gwas = search["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == "gwas")
            .unwrap();
        assert!(gwas["properties"].get("trait").is_some());
        assert!(gwas["properties"].get("region").is_none());
        let get = serde_json::to_value(rmcp::schemars::schema_for!(TypedGet)).unwrap();
        assert_eq!(get["oneOf"].as_array().unwrap().len(), 12);
        let author = get["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == "author")
            .unwrap();
        assert!(author["properties"].get("sections").is_none());
        let gene = get["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == "gene")
            .unwrap();
        assert!(
            gene["properties"]["sections"]["items"]["enum"]
                .as_array()
                .unwrap()
                .contains(&json!("pathways"))
        );
        assert!(
            !gene["properties"]["sections"]["items"]["enum"]
                .as_array()
                .unwrap()
                .contains(&json!("population"))
        );
        let variant = get["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|branch| branch["properties"]["entity"]["const"] == "variant")
            .unwrap();
        assert_eq!(
            variant["properties"]["assembly"]["enum"],
            json!(["grch37", "hg19", "grch38", "hg38"])
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
                files: false,
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
    fn typed_search_maps_each_published_entity_and_rejects_schema_mismatches() {
        for (input, expected) in [
            (
                json!({"entity":"article","keyword":["BRAF"],"gene":"BRAF","source":"pubmed"}),
                "--keyword",
            ),
            (
                json!({"entity":"trial","condition":["melanoma"],"phase":"2"}),
                "--condition",
            ),
            (
                json!({"entity":"variant","gene":"BRAF","hgvsp":"V600E"}),
                "--hgvsp",
            ),
            (json!({"entity":"gene","region":"7:1-2"}), "--region"),
            (
                json!({"entity":"protein","query":"BRAF","reviewed":true}),
                "--reviewed",
            ),
            (
                json!({"entity":"pgx","gene":"CYP2D6","cpic_level":"A"}),
                "--cpic-level",
            ),
            (
                json!({"entity":"gwas","gene":"TCF7L2","trait":"diabetes"}),
                "--trait",
            ),
            (
                json!({"entity":"author","query":"Jane Doe","source":"semanticscholar"}),
                "--source",
            ),
        ] {
            let args = search_args(TypedSearch(input)).expect("published typed search");
            assert!(
                args.iter().any(|arg| arg == expected),
                "missing {expected} in {args:?}"
            );
        }

        for input in [
            json!({"entity":"gwas","gene":"BRAF","region":"7:1-2"}),
            json!({"entity":"pathway","query":"MAPK"}),
            json!({"entity":"protein","query":"BRAF","reviewed":"yes"}),
            json!({"entity":"gwas","gene":"BRAF","offset":49,"limit":2}),
            json!({"entity":"gene","query":"BRAF","limit":50}),
            json!({"entity":"trial","condition":["x"],"source":"nci","mutation":["a"],"criteria":["b"]}),
        ] {
            assert!(search_args(TypedSearch(input)).is_err());
        }
    }

    #[test]
    fn mcp_allowlist_blocks_mutating_commands() {
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "search".into(),
            "gene".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "variant".into(),
            "articles".into(),
            "BRAF p.V600E".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "variant".into(),
            "--json".into(),
            "articles".into(),
            "--input".into(),
            "/server/private.json".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "variant".into(),
            "articles".into(),
            "--input=-".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "--json".into(),
            "list".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "render".into()
        ]));
        assert!(is_allowed_mcp_args(&["biomcp".into(), "skill".into()]));
        // Numeric and slug skill lookups are read-only when they name embedded skills.
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "03".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "gene-disease-orientation".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "03-gene-disease-orientation".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "--no-cache".into(),
            "list".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "--list".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "cache".into(),
            "path".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "cache".into(),
            "stats".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "top-mutated".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--limit".into(),
            "10".into()
        ]));
        assert!(is_allowed_mcp_args(&[
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
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "filter".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--mutated".into(),
            "TP53".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "cohort".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "survival".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "compare".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--gene".into(),
            "TP53".into(),
            "--type".into(),
            "mutations".into(),
            "--target".into(),
            "KRAS".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "co-occurrence".into(),
            "--study".into(),
            "msk_impact_2017".into(),
            "--genes".into(),
            "TP53,KRAS".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "suggest".into(),
            "What drugs treat melanoma?".into()
        ]));
        assert!(is_allowed_mcp_args(&[
            "biomcp".into(),
            "discover".into(),
            "BRCA1".into()
        ]));
        assert!(!is_allowed_mcp_args(&["biomcp".into(), "update".into()]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "install".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "status".into(),
            "/home/operator/private/skills".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "not-a-real-skill".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "skill".into(),
            "render".into(),
            "extra".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "ema".into(),
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "who-ivd".into(),
            "sync".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "msk_impact_2017".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "download".into(),
            "--list".into(),
            "msk_impact_2017".into()
        ]));
        assert!(!is_allowed_mcp_args(&[
            "biomcp".into(),
            "study".into(),
            "download".into()
        ]));
    }

    #[test]
    fn raw_local_input_rejection_covers_every_spelling_and_variant_route() {
        for prefix in [
            &["biomcp", "variant", "--no-cache", "articles"][..],
            &["biomcp", "variant", "--no-cache", "erepo"][..],
            &["biomcp", "variant", "--no-cache", "normalize", "car"][..],
        ] {
            for input in [["--input", "/server/private.json"], ["--input=-", ""]] {
                let mut args = prefix
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect::<Vec<_>>();
                args.extend(
                    input
                        .into_iter()
                        .filter(|value| !value.is_empty())
                        .map(String::from),
                );
                assert!(!is_allowed_mcp_args(&args));
                assert_eq!(
                    mcp_rejection_message_for_args(&args),
                    LOCAL_INPUT_MCP_REJECTION_MESSAGE
                );
            }
        }
    }

    #[test]
    fn raw_mcp_local_input_inventory_matches_the_cli_surface() {
        use clap::CommandFactory;

        fn collect(command: &clap::Command, path: &mut Vec<String>, found: &mut Vec<String>) {
            if command
                .get_arguments()
                .any(|argument| argument.get_id().as_str() == "input")
            {
                found.push(path.join(" "));
            }
            for child in command.get_subcommands() {
                path.push(child.get_name().to_string());
                collect(child, path, found);
                path.pop();
            }
        }

        let command = crate::cli::Cli::command();
        let mut found = Vec::new();
        collect(&command, &mut Vec::new(), &mut found);
        found.sort();
        assert_eq!(
            found,
            ["variant articles", "variant erepo", "variant normalize"]
        );
    }

    #[test]
    fn cache_family_rejection_message_mentions_local_path_disclosure() {
        let args = vec![
            "biomcp".into(),
            "--no-cache".into(),
            "cache".into(),
            "path".into(),
        ];
        assert_eq!(
            mcp_rejection_message_for_args(&args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );

        let stats_args = vec!["biomcp".into(), "cache".into(), "stats".into()];
        assert_eq!(
            mcp_rejection_message_for_args(&stats_args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );

        let clear_args = vec!["biomcp".into(), "cache".into(), "clear".into()];
        assert_eq!(
            mcp_rejection_message_for_args(&clear_args),
            CACHE_FAMILY_MCP_REJECTION_MESSAGE
        );
    }

    #[test]
    fn generic_mcp_rejection_message_for_args_stays_read_only_for_mutating_commands() {
        let args = vec!["biomcp".into(), "--json".into(), "update".into()];
        assert_eq!(
            mcp_rejection_message_for_args(&args),
            GENERIC_MCP_REJECTION_MESSAGE
        );

        let private = "/home/operator/private/skills";
        let status_args = vec![
            "biomcp".into(),
            "skill".into(),
            "status".into(),
            private.into(),
        ];
        let message = mcp_rejection_message_for_args(&status_args);
        assert_eq!(message, GENERIC_MCP_REJECTION_MESSAGE);
        assert!(!message.contains(private));
    }
}
