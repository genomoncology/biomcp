//! Top-level CLI parsing and command execution.

mod adverse_event;
mod article;
#[cfg(test)]
#[allow(dead_code)]
mod benchmark;
pub mod cache;
pub mod chart;
mod commands;
pub mod debug_plan;
mod diagnostic;
pub mod discover;
mod disease;
mod drug;
mod gene;
mod gwas;
pub mod health;
pub mod list;
mod outcome;
mod pathway;
mod pgx;
mod phenotype;
mod protein;
pub mod search_all;
mod search_all_command;
mod shared;
pub mod skill;
mod study;
pub(crate) mod suggest;
mod system;
#[cfg(test)]
mod test_support;
mod trial;
mod types;
pub mod update;
mod variant;

pub use self::article::ArticleCommand;
pub use self::commands::{Commands, GetEntity, SearchEntity};
pub use self::disease::DiseaseCommand;
pub use self::drug::DrugCommand;
pub use self::gene::GeneCommand;
#[cfg(test)]
use self::outcome::{McpChartPass, rewrite_mcp_chart_args};
pub use self::outcome::{execute, execute_mcp, run, run_outcome};
pub use self::pathway::PathwayCommand;
pub use self::protein::ProteinCommand;
pub use self::shared::{build_cli, parse_cli_from_env, try_parse_cli};
pub use self::study::StudyCommand;
pub use self::system::{CvxCommand, DdinterCommand, EmaCommand, GtrCommand, WhoCommand};
pub use self::types::{
    ChartArgs, ChartType, Cli, CliOutput, CommandOutcome, DrugRegionArg, OutputStream,
};
pub use self::variant::VariantCommand;

#[cfg(test)]
use self::shared::RUNTIME_HELP_SUBCOMMANDS;
#[cfg(test)]
use self::shared::search_meta_with_suggestions;
use self::shared::{
    PaginationMeta, SearchJsonMeta, empty_sections, extract_json_from_sections,
    log_pagination_truncation, normalize_cli_query, normalize_cli_tokens, normalize_next_commands,
    paged_fetch_limit, paginate_results, pagination_footer_cursor, pagination_footer_offset,
    related_article_filters, render_batch_json, resolve_query_input, search_json,
    search_json_with_meta, search_json_with_meta_and_suggestions, search_meta,
    search_meta_with_workflow, try_alias_fallback_outcome,
};

#[cfg(test)]
mod tests;
