use std::ffi::{OsStr, OsString};
use std::io::Write;

use clap::{CommandFactory, FromArgMatches, error::ErrorKind};
use tracing::warn;

use super::commands::Commands;
use super::types::{Cli, CommandOutcome};

pub(super) const RUNTIME_HELP_SUBCOMMANDS: [&str; 4] = ["mcp", "serve", "serve-http", "serve-sse"];
const HIDDEN_GLOBAL_FLAGS: [&str; 3] = ["--json", "-j", "--no-cache"];
const SEARCH_ENTITY_NAMES: [&str; 15] = [
    "all",
    "author",
    "gene",
    "disease",
    "diagnostic",
    "pgx",
    "phenotype",
    "gwas",
    "article",
    "trial",
    "variant",
    "drug",
    "pathway",
    "protein",
    "adverse-event",
];
const SEARCH_CATCHALL_NAMES: [&str; 3] = ["gene", "drug", "variant"];
const RECOVERY_LIMITS: (usize, usize, usize) = (256, 16_384, 32_768);

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
        .expect("json arg should exist");
    let hidden_json_arg = json_arg.clone().hide(true);
    let no_cache_arg = command
        .get_arguments()
        .find(|arg| arg.get_id() == "no_cache")
        .cloned()
        .expect("no_cache arg should exist")
        .hide(true);

    for subcommand_name in RUNTIME_HELP_SUBCOMMANDS {
        command =
            hide_runtime_help_globals(command, subcommand_name, &hidden_json_arg, &no_cache_arg);
    }
    command = command.mut_subcommand("cache", |cache| cache.arg(json_arg).arg(no_cache_arg));
    command
}

pub(super) fn validate_cli_state_contract(cli: &Cli) -> Result<(), crate::error::BioMcpError> {
    if cli.no_cache && matches!(&cli.command, Commands::Cache { .. }) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--no-cache cannot be used with cache commands because they inspect or modify managed state"
                .into(),
        ));
    }
    Ok(())
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
    if let Some(sentence) = reversed_search_correction(&args) {
        return Err(build_cli().error(ErrorKind::InvalidSubcommand, sentence));
    }
    let matches = build_cli().try_get_matches_from(args)?;
    Cli::from_arg_matches(&matches)
}

enum ReversedSearch {
    NotReversed,
    Complete(String),
    Suppressed {
        entity: &'static str,
        reason: Suppression,
    },
}

#[derive(Clone, Copy)]
enum Suppression {
    UnsafeInput,
    InvalidCandidate,
}

fn classify_reversed_search<T: AsRef<OsStr>>(args: &[T]) -> ReversedSearch {
    let mut commands = args
        .iter()
        .enumerate()
        .skip(1)
        .take_while(|(_, value)| value.as_ref() != "--")
        .filter(|(_, value)| {
            !HIDDEN_GLOBAL_FLAGS
                .iter()
                .any(|flag| value.as_ref() == *flag)
        });
    let (Some((first, entity_arg)), Some((second, search_arg))) =
        (commands.next(), commands.next())
    else {
        return ReversedSearch::NotReversed;
    };
    let Some(entity) = SEARCH_ENTITY_NAMES
        .iter()
        .copied()
        .find(|name| entity_arg.as_ref() == OsStr::new(name))
    else {
        return ReversedSearch::NotReversed;
    };
    if search_arg.as_ref() != "search" {
        return ReversedSearch::NotReversed;
    }
    let complete = (|| -> Result<String, Suppression> {
        if args.len() > RECOVERY_LIMITS.0 {
            return Err(Suppression::UnsafeInput);
        }
        let values = args
            .iter()
            .map(|arg| arg.as_ref().to_str())
            .collect::<Option<Vec<_>>>()
            .ok_or(Suppression::UnsafeInput)?;
        let input_bytes = values
            .iter()
            .try_fold(0usize, |total, value| total.checked_add(value.len()))
            .ok_or(Suppression::UnsafeInput)?;
        if input_bytes > RECOVERY_LIMITS.1
            || values.iter().any(|value| {
                value.chars().any(|ch| {
                    ('\u{0}'..='\u{1f}').contains(&ch) || ('\u{7f}'..='\u{9f}').contains(&ch)
                })
            })
        {
            return Err(Suppression::UnsafeInput);
        }
        let mut candidate = args
            .iter()
            .map(|arg| arg.as_ref().to_os_string())
            .collect::<Vec<_>>();
        candidate.swap(first, second);
        match build_cli().try_get_matches_from(candidate.clone()) {
            Ok(_) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
                ) => {}
            Err(_) => return Err(Suppression::InvalidCandidate),
        }
        let rendered = shlex::try_join(
            std::iter::once("biomcp").chain(
                candidate
                    .iter()
                    .skip(1)
                    .map(|arg| arg.to_str().expect("argv was validated as UTF-8")),
            ),
        )
        .map_err(|_| Suppression::UnsafeInput)?;
        let sentence = format!(
            "reversed search syntax; use {}",
            crate::render::markdown::markdown_command_code_span(&rendered)
        );
        (sentence.len() <= RECOVERY_LIMITS.2)
            .then_some(sentence)
            .ok_or(Suppression::UnsafeInput)
    })();
    match complete {
        Ok(sentence) => ReversedSearch::Complete(sentence),
        Err(reason) => ReversedSearch::Suppressed { entity, reason },
    }
}

pub(crate) fn reversed_search_correction<T: AsRef<OsStr>>(args: &[T]) -> Option<String> {
    match classify_reversed_search(args) {
        ReversedSearch::Complete(sentence) => Some(sentence),
        ReversedSearch::Suppressed { entity, reason }
            if SEARCH_CATCHALL_NAMES.contains(&entity)
                && (!matches!(reason, Suppression::InvalidCandidate)
                    || build_cli()
                        .try_get_matches_from(args)
                        .and_then(|matches| Cli::from_arg_matches(&matches))
                        .is_ok()) =>
        {
            Some(format!(
                "reversed search syntax; use `biomcp search {entity}`; the supplied search arguments were not accepted"
            ))
        }
        ReversedSearch::NotReversed | ReversedSearch::Suppressed { .. } => None,
    }
}

fn args_request_json(args: &[OsString]) -> bool {
    args.iter()
        .any(|arg| arg == OsStr::new("--json") || arg == OsStr::new("-j"))
}

fn reversed_args_request_json(args: &[OsString]) -> bool {
    args.iter()
        .take_while(|arg| *arg != OsStr::new("--"))
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
    let requests_json = if reversed_search_correction(&args).is_some() {
        reversed_args_request_json(&args)
    } else {
        args_request_json(&args)
    };
    match try_parse_cli(args.clone()) {
        Ok(cli) => cli,
        Err(err)
            if requests_json
                && matches!(
                    err.kind(),
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
                ) =>
        {
            let json = match err.kind() {
                ErrorKind::DisplayHelp => crate::render::json::to_pretty(&serde_json::json!({
                    "kind": "help",
                    "content": render_human_clap_error(&err, &args),
                })),
                ErrorKind::DisplayVersion => super::system::version_identity_json(),
                _ => unreachable!("guarded display result"),
            }
            .expect("static CLI display result should render as JSON");
            let mut stdout = std::io::stdout();
            let _ = stdout.write_all(json.as_bytes());
            let _ = stdout.write_all(b"\n");
            let _ = stdout.flush();
            std::process::exit(0);
        }
        Err(err) if requests_json => {
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

    // dead-code reason: retained for generic cursor pagination outside the typed trial contract
    #[allow(dead_code)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) workflow_rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) workflow_playbook: Option<String>,
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
        workflow_rationale: None,
        workflow_playbook: None,
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
    let (workflow, workflow_rationale, workflow_playbook) = workflow
        .map(|meta| {
            (
                Some(meta.workflow),
                Some(meta.rationale),
                Some(meta.playbook),
            )
        })
        .unwrap_or((None, None, None));
    (!next_commands.is_empty() || suggestions.is_some() || workflow.is_some()).then_some(
        SearchJsonMeta {
            next_commands,
            suggestions,
            workflow,
            workflow_rationale,
            workflow_playbook,
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

// dead-code reason: retained for non-trial cursor-based entity pagination compatibility
#[allow(dead_code)]
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
