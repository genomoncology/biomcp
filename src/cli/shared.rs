use std::ffi::{OsStr, OsString};
use std::io::Write;

use clap::{CommandFactory, FromArgMatches, error::ErrorKind};
use tracing::{debug, warn};

use super::types::{Cli, CommandOutcome};

pub(super) const RUNTIME_HELP_SUBCOMMANDS: [&str; 4] = ["mcp", "serve", "serve-http", "serve-sse"];
const HIDDEN_GLOBAL_FLAGS: [&str; 3] = ["--json", "-j", "--no-cache"];

fn hide_runtime_help_globals(
    command: clap::Command,
    subcommand_name: &'static str,
    json_arg: &clap::Arg,
    no_cache_arg: &clap::Arg,
) -> clap::Command {
    command.mut_subcommand(subcommand_name, |runtime| {
        runtime.arg(json_arg.clone()).arg(no_cache_arg.clone())
    })
}

pub fn build_cli() -> clap::Command {
    let mut command = Cli::command().version(crate::build_identity::current().version);
    let json_arg = command
        .get_arguments()
        .find(|arg| arg.get_id() == "json")
        .cloned()
        .expect("json arg should exist")
        .hide(true);
    let no_cache_arg = command
        .get_arguments()
        .find(|arg| arg.get_id() == "no_cache")
        .cloned()
        .expect("no_cache arg should exist")
        .hide(true);

    for subcommand_name in RUNTIME_HELP_SUBCOMMANDS {
        command = hide_runtime_help_globals(command, subcommand_name, &json_arg, &no_cache_arg);
    }
    command
}

fn reject_reserved_skill_subcommand(args: &[OsString]) -> Result<(), clap::Error> {
    let mut tokens = args.iter().skip(1).filter_map(|arg| {
        let value = arg.to_string_lossy();
        let trimmed = value.trim();
        if trimmed.is_empty() || HIDDEN_GLOBAL_FLAGS.contains(&trimmed) {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    if matches!(tokens.next().as_deref(), Some("skill"))
        && matches!(tokens.next().as_deref(), Some("uninstall"))
    {
        return Err(build_cli().error(
            ErrorKind::InvalidSubcommand,
            "unrecognized subcommand 'uninstall'. Use `biomcp uninstall`.",
        ));
    }

    Ok(())
}

pub fn try_parse_cli<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
    reject_reserved_skill_subcommand(&args)?;
    let matches = build_cli().try_get_matches_from(args)?;
    Cli::from_arg_matches(&matches)
}

fn args_request_json(args: &[OsString]) -> bool {
    args.iter()
        .any(|arg| arg == OsStr::new("--json") || arg == OsStr::new("-j"))
}

pub(crate) fn render_human_clap_error(error: &clap::Error, args: &[OsString]) -> String {
    let mut message = error.render().to_string();
    for arg in args {
        let arg = arg.to_string_lossy();
        let sanitized = crate::render::human::sanitize_inline(&arg);
        if sanitized != arg {
            message = message.replace(arg.as_ref(), &sanitized);
        }
    }
    crate::render::human::sanitize_document(&message)
}

fn exit_human_clap_error(error: clap::Error, args: &[OsString]) -> ! {
    let exit_code = error.exit_code();
    let message = render_human_clap_error(&error, args);
    let mut stream: Box<dyn Write> = if error.use_stderr() {
        Box::new(std::io::stderr())
    } else {
        Box::new(std::io::stdout())
    };
    let _ = stream.write_all(message.as_bytes());
    let _ = stream.flush();
    std::process::exit(exit_code);
}

pub fn parse_cli_from_env() -> Cli {
    let args: Vec<OsString> = std::env::args_os().collect();
    match try_parse_cli(args.clone()) {
        Ok(cli) => cli,
        Err(err) if args_request_json(&args) => {
            let exit_code = err.exit_code();
            let bio_err = crate::error::BioMcpError::InvalidArgument(err.to_string());
            let json = crate::render::json::to_error_json(&bio_err)
                .expect("clap parse errors should render as JSON");
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(json.as_bytes());
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
            std::process::exit(exit_code);
        }
        Err(err) => exit_human_clap_error(err, &args),
    }
}

pub(super) fn empty_sections() -> &'static [String] {
    &[]
}

pub(super) fn related_article_filters() -> crate::entities::article::ArticleSearchFilters {
    crate::entities::article::ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: None,
        date_from: None,
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: true,
        exclude_retracted: true,
        max_per_source: None,
        sort: crate::entities::article::ArticleSort::Relevance,
        ranking: crate::entities::article::ArticleRankingOptions::default(),
    }
}

pub(super) fn extract_json_from_sections(sections: &[String]) -> (Vec<String>, bool) {
    let mut json_override = false;
    let cleaned = sections
        .iter()
        .filter_map(|raw| {
            let trimmed = raw.trim();
            let normalized = trimmed.to_ascii_lowercase();
            if normalized == "--json" || normalized == "-j" {
                json_override = true;
                return None;
            }
            if trimmed.is_empty() {
                return None;
            }
            Some(trimmed.to_string())
        })
        .collect();
    (cleaned, json_override)
}

pub(super) fn normalize_cli_query(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn normalize_cli_tokens(values: Vec<String>) -> Option<String> {
    let joined = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    normalize_cli_query(Some(joined))
}

pub(super) fn resolve_query_input(
    flag_query: Option<String>,
    positional_query: Option<String>,
    flag_names: &str,
) -> Result<Option<String>, crate::error::BioMcpError> {
    let flag_query = normalize_cli_query(flag_query);
    let positional_query = normalize_cli_query(positional_query);
    match (flag_query, positional_query) {
        (Some(_), Some(_)) => Err(crate::error::BioMcpError::InvalidArgument(format!(
            "Use either positional QUERY or {flag_names}, not both"
        ))),
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn alias_suggestion_markdown(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    decision: &crate::entities::discover::AliasFallbackDecision,
) -> String {
    let err = crate::error::BioMcpError::NotFound {
        entity: requested_entity.cli_name().to_string(),
        id: query.trim().to_string(),
        suggestion: crate::render::markdown::alias_fallback_suggestion(decision),
    };
    format!("Error: {err}")
}

pub(super) fn alias_suggestion_outcome(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    decision: &crate::entities::discover::AliasFallbackDecision,
    json_output: bool,
) -> anyhow::Result<CommandOutcome> {
    if json_output {
        return Ok(CommandOutcome::stdout_with_exit(
            crate::render::json::to_alias_suggestion_json(decision)?,
            1,
        ));
    }
    Ok(CommandOutcome::stderr_with_exit(
        alias_suggestion_markdown(query, requested_entity, decision),
        1,
    ))
}

#[cfg(test)]
pub(super) fn render_batch_json<T, F>(
    results: &[T],
    wrap: F,
) -> Result<String, crate::error::BioMcpError>
where
    F: Fn(&T) -> Result<serde_json::Value, crate::error::BioMcpError>,
{
    let items = results.iter().map(wrap).collect::<Result<Vec<_>, _>>()?;
    crate::render::json::to_pretty(&items)
}

pub(super) async fn try_alias_fallback_outcome(
    query: &str,
    requested_entity: crate::entities::discover::DiscoverType,
    json_output: bool,
) -> anyhow::Result<Option<CommandOutcome>> {
    match crate::entities::discover::resolve_query(
        query,
        crate::entities::discover::DiscoverMode::AliasFallback,
    )
    .await
    {
        Ok(result) => {
            let decision =
                crate::entities::discover::classify_alias_fallback(&result, requested_entity);
            match decision {
                crate::entities::discover::AliasFallbackDecision::None => Ok(None),
                other => Ok(Some(alias_suggestion_outcome(
                    query,
                    requested_entity,
                    &other,
                    json_output,
                )?)),
            }
        }
        Err(err) => {
            warn!(
                query = query.trim(),
                entity = requested_entity.cli_name(),
                "alias fallback discovery unavailable: {err}"
            );
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct PaginationMeta {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
}

impl PaginationMeta {
    pub(super) fn offset(
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
    ) -> Self {
        let has_more = total
            .map(|value| offset.saturating_add(returned) < value)
            .unwrap_or(returned == limit);
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_page_token: None,
        }
    }

    pub(super) fn cursor(
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
        next_page_token: Option<String>,
    ) -> Self {
        let has_token = next_page_token
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());
        let has_more = match total {
            Some(value) => has_token && offset.saturating_add(returned) < value,
            None => has_token,
        };
        Self {
            offset,
            limit,
            returned,
            total,
            has_more,
            next_page_token: has_more.then_some(next_page_token).flatten(),
        }
    }
}

// dead-code reason: shared::SearchJsonResponse is exercised by binary dispatch or CLI contracts
#[cfg_attr(not(test), allow(dead_code))]
#[derive(serde::Serialize)]
struct SearchJsonResponse<T: serde::Serialize> {
    pagination: PaginationMeta,
    count: usize,
    results: Vec<T>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct SearchJsonMeta {
    pub(super) next_commands: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) suggestions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) ladder: Vec<crate::workflow_ladders::WorkflowLadderStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) section_sources: Vec<crate::render::provenance::SectionSource>,
}

impl SearchJsonMeta {
    pub(super) fn with_section_sources(
        mut self,
        section_sources: Vec<crate::render::provenance::SectionSource>,
    ) -> Self {
        self.section_sources = section_sources;
        self
    }
}

#[derive(serde::Serialize)]
struct SearchJsonResponseWithMeta<T: serde::Serialize> {
    pagination: PaginationMeta,
    count: usize,
    results: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _meta: Option<SearchJsonMeta>,
}

// dead-code reason: shared::search_json is exercised by binary dispatch or CLI contracts
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn search_json<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
) -> anyhow::Result<String> {
    let count = results.len();
    crate::render::json::to_pretty(&SearchJsonResponse {
        pagination,
        count,
        results,
    })
    .map_err(Into::into)
}

pub(super) fn normalize_next_commands(next_commands: Vec<String>) -> Vec<String> {
    next_commands
        .into_iter()
        .map(|command| command.trim().to_string())
        .filter(|command| !command.is_empty())
        .collect()
}

pub(super) fn search_meta(next_commands: Vec<String>) -> Option<SearchJsonMeta> {
    search_meta_with_suggestions(next_commands, None)
}

pub(super) fn search_meta_with_section_sources(
    next_commands: Vec<String>,
    section_sources: Vec<crate::render::provenance::SectionSource>,
) -> Option<SearchJsonMeta> {
    let meta = search_meta(next_commands).unwrap_or(SearchJsonMeta {
        next_commands: Vec::new(),
        suggestions: None,
        workflow: None,
        ladder: Vec::new(),
        section_sources: Vec::new(),
    });
    (!meta.next_commands.is_empty() || !section_sources.is_empty())
        .then(|| meta.with_section_sources(section_sources))
}

pub(super) fn search_meta_with_suggestions(
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
) -> Option<SearchJsonMeta> {
    search_meta_with_workflow(next_commands, suggestions, None)
}

pub(super) fn search_meta_with_workflow(
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
    workflow: Option<crate::workflow_ladders::WorkflowMeta>,
) -> Option<SearchJsonMeta> {
    let next_commands = normalize_next_commands(next_commands);
    let suggestions = suggestions.map(normalize_next_commands);
    let (workflow, ladder) = workflow
        .map(|meta| (Some(meta.workflow), meta.ladder))
        .unwrap_or_else(|| (None, Vec::new()));
    (!next_commands.is_empty() || suggestions.is_some() || workflow.is_some()).then_some(
        SearchJsonMeta {
            next_commands,
            suggestions,
            workflow,
            ladder,
            section_sources: Vec::new(),
        },
    )
}

pub(super) fn search_json_with_meta<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
    next_commands: Vec<String>,
) -> anyhow::Result<String> {
    search_json_with_meta_and_suggestions(results, pagination, next_commands, None)
}

pub(super) fn search_json_with_meta_and_suggestions<T: serde::Serialize>(
    results: Vec<T>,
    pagination: PaginationMeta,
    next_commands: Vec<String>,
    suggestions: Option<Vec<String>>,
) -> anyhow::Result<String> {
    let count = results.len();
    crate::render::json::to_pretty(&SearchJsonResponseWithMeta {
        pagination,
        count,
        results,
        _meta: search_meta_with_suggestions(next_commands, suggestions),
    })
    .map_err(Into::into)
}

pub(super) fn pagination_footer_offset(meta: &PaginationMeta) -> String {
    crate::render::markdown::pagination_footer(
        crate::render::markdown::PaginationFooterMode::Offset,
        meta.offset,
        meta.limit,
        meta.returned,
        meta.total,
        None,
    )
}

pub(super) fn pagination_footer_cursor(meta: &PaginationMeta) -> String {
    crate::render::markdown::pagination_footer(
        crate::render::markdown::PaginationFooterMode::Cursor,
        meta.offset,
        meta.limit,
        meta.returned,
        meta.total,
        meta.next_page_token.as_deref(),
    )
}

pub(super) fn paged_fetch_limit(
    limit: usize,
    offset: usize,
    max_limit: usize,
) -> Result<usize, crate::error::BioMcpError> {
    if limit == 0 || limit > max_limit {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {max_limit}"
        )));
    }
    Ok(limit.saturating_add(offset).min(max_limit))
}

pub(super) fn paged_fetch_limit_for(
    command_name: &str,
    limit: usize,
    offset: usize,
    max_limit: usize,
) -> Result<usize, crate::error::BioMcpError> {
    if limit == 0 || limit > max_limit {
        return Err(crate::error::BioMcpError::InvalidArgument(format!(
            "--limit for {command_name} must be 1-{max_limit}"
        )));
    }
    Ok(limit.saturating_add(offset).min(max_limit))
}

pub(crate) fn paginate_results<T>(rows: Vec<T>, offset: usize, limit: usize) -> (Vec<T>, usize) {
    let total = rows.len();
    let paged = rows.into_iter().skip(offset).take(limit).collect();
    (paged, total)
}

pub(super) fn log_pagination_truncation(observed_total: usize, offset: usize, returned: usize) {
    if offset.saturating_add(returned) < observed_total {
        debug!(
            total = observed_total,
            offset, returned, "Results truncated by --limit"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::section_outcome::SectionOutcomeState;

    #[test]
    fn cursor_total_suppresses_stale_token_past_the_end() {
        let pagination = PaginationMeta::cursor(4_000, 5, 0, Some(3_738), Some("stale".into()));

        assert!(!pagination.has_more);
        assert_eq!(pagination.next_page_token, None);
    }

    #[test]
    fn cursor_without_token_never_promises_a_next_page() {
        let pagination = PaginationMeta::cursor(0, 5, 5, Some(10), None);

        assert!(!pagination.has_more);
        assert_eq!(pagination.next_page_token, None);
    }

    #[test]
    fn section_provenance_keeps_meta_when_search_has_no_next_commands() {
        let meta = search_meta_with_section_sources(
            Vec::new(),
            vec![crate::render::provenance::SectionSource {
                key: "faers".to_string(),
                label: "Adverse events (OpenFDA FAERS)".to_string(),
                outcome: SectionOutcomeState::Unavailable,
                sources: Vec::new(),
            }],
        )
        .expect("section provenance should create metadata");
        let value = serde_json::to_value(meta).expect("metadata JSON");

        assert_eq!(value["section_sources"][0]["key"], "faers");
        assert_eq!(value["section_sources"][0]["outcome"], "unavailable");
    }
}
