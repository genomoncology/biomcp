//! Article CLI payloads and subcommands.

use clap::{Args, Subcommand};

mod annotations;
mod fulltext_view;
pub(super) use annotations::truncate_article_annotations;

fn parse_article_year(value: &str) -> Result<u16, String> {
    if value.len() != 4 || !value.chars().all(|ch| ch.is_ascii_digit()) {
        return Err("expected YYYY".to_string());
    }
    value.parse().map_err(|_| "expected YYYY".to_string())
}

#[derive(Args, Debug)]
pub struct ArticleSearchArgs {
    /// Filter by gene symbol
    #[arg(short, long)]
    pub gene: Option<String>,
    /// Filter by disease name
    #[arg(short, long, num_args = 1..)]
    pub disease: Vec<String>,
    /// Filter by drug/chemical name
    #[arg(long, num_args = 1..)]
    pub drug: Vec<String>,
    /// Filter by author name (default search uses compatible author-capable sources)
    #[arg(short = 'a', long, num_args = 1..)]
    pub author: Vec<String>,
    /// Provider-neutral free text (alias: -q, --query; use --author/--journal for fields)
    #[arg(
        short = 'k',
        long = "keyword",
        visible_short_alias = 'q',
        visible_alias = "query",
        num_args = 1..
    )]
    pub keyword: Vec<String>,
    /// Optional positional query alias for -k/--keyword/--query
    #[arg(value_name = "QUERY")]
    pub positional_query: Option<String>,
    /// Published after date (YYYY, YYYY-MM, or YYYY-MM-DD)
    #[arg(long = "date-from", visible_alias = "since")]
    pub date_from: Option<String>,
    /// Published before date (YYYY, YYYY-MM, or YYYY-MM-DD)
    #[arg(long = "date-to", visible_alias = "until")]
    pub date_to: Option<String>,
    /// Published from year (YYYY)
    #[arg(
        long = "year-min",
        value_name = "YYYY",
        value_parser = parse_article_year,
        conflicts_with = "date_from"
    )]
    pub year_min: Option<u16>,
    /// Published through year (YYYY)
    #[arg(
        long = "year-max",
        value_name = "YYYY",
        value_parser = parse_article_year,
        conflicts_with = "date_to"
    )]
    pub year_max: Option<u16>,
    /// Filter by publication type [values: research-article, review, case-reports, meta-analysis]
    #[arg(long = "type")]
    pub article_type: Option<String>,
    /// Filter by journal title
    #[arg(long, num_args = 1..)]
    pub journal: Vec<String>,
    /// Restrict to open-access articles (default: off, includes all access models)
    #[arg(long = "open-access")]
    pub open_access: bool,
    /// Exclude preprints (best-effort; default: off, includes preprints)
    #[arg(long)]
    pub no_preprints: bool,
    /// Exclude retracted publications from search results
    #[arg(long)]
    pub exclude_retracted: bool,
    /// Include retracted publications in search results (default excludes them)
    #[arg(long, conflicts_with = "exclude_retracted")]
    pub include_retracted: bool,
    /// Sort order [values: date, citations, relevance] (default: relevance)
    #[arg(long, default_value = "relevance", value_parser = ["date", "citations", "relevance"])]
    pub sort: String,
    /// Relevance ranking mode [values: lexical, semantic, hybrid] (default: hybrid with keyword, lexical otherwise)
    #[arg(long = "ranking-mode", value_parser = ["lexical", "semantic", "hybrid"])]
    pub ranking_mode: Option<String>,
    /// Hybrid semantic weight (default: 0.4; requires --sort relevance)
    #[arg(long = "weight-semantic")]
    pub weight_semantic: Option<f64>,
    /// Hybrid lexical weight (default: 0.3; requires --sort relevance)
    #[arg(long = "weight-lexical")]
    pub weight_lexical: Option<f64>,
    /// Hybrid citation weight (default: 0.2; requires --sort relevance)
    #[arg(long = "weight-citations")]
    pub weight_citations: Option<f64>,
    /// Hybrid source-position weight (default: 0.1; requires --sort relevance)
    #[arg(long = "weight-position")]
    pub weight_position: Option<f64>,
    /// Article source [values: all, pubtator, europepmc, pubmed, semanticscholar, litsense2] (default: all; LitSense2 explicit only)
    #[arg(
        long,
        default_value = "all",
        value_parser = ["all", "pubtator", "europepmc", "pubmed", "semanticscholar", "litsense2"]
    )]
    pub source: String,
    /// Cap each federated source's contribution after deduplication and before ranking (default: 40% of --limit on federated pools with at least three surviving primary sources; 0 uses the default cap; equal to --limit disables capping)
    #[arg(long = "max-per-source", value_name = "N")]
    pub max_per_source: Option<usize>,
    /// Local caller label for JSON loop-breaker suggestions across consecutive article keyword searches
    #[arg(long = "session", value_name = "TOKEN")]
    pub session: Option<String>,
    /// Maximum results, 1-50 (default: 10)
    #[arg(short, long, default_value = "10")]
    pub limit: usize,
    /// Skip the first N results
    #[arg(long, default_value = "0")]
    pub offset: usize,
    /// Include the executed search planner output and redacted source status in markdown or JSON output
    #[arg(long = "debug-plan")]
    pub debug_plan: bool,
    /// Return detailed JSON rows including abstracts, provenance, and ranking diagnostics
    #[arg(long)]
    pub full: bool,
}

#[derive(Clone, Copy)]
pub(super) enum ArticleSearchDetail {
    Compact,
    Full,
}

pub(super) const DATE_SORT_WARNING_MESSAGE: &str =
    "Date sort replaces relevance ranking; results are ordered by publication date.";

#[derive(Clone, Copy, serde::Serialize)]
pub(super) struct ArticleSearchWarning {
    pub(super) code: &'static str,
    pub(super) message: &'static str,
}

pub(super) fn article_search_warning(
    sort: crate::entities::article::ArticleSort,
) -> Option<ArticleSearchWarning> {
    (sort == crate::entities::article::ArticleSort::Date).then_some(ArticleSearchWarning {
        code: "date_sort_replaces_relevance",
        message: DATE_SORT_WARNING_MESSAGE,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) struct ArticleSuggestion {
    pub command: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<String>,
}

#[derive(serde::Serialize)]
pub(super) struct ArticleSearchJsonMeta {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<ArticleSearchWarning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    next_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    suggestions: Vec<ArticleSuggestion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    source_status: Vec<crate::entities::article::ArticleSourceStatus>,
}

pub(super) fn article_search_json_meta(
    warnings: Vec<ArticleSearchWarning>,
    next_commands: Vec<String>,
    suggestions: Vec<ArticleSuggestion>,
    source_status: Vec<crate::entities::article::ArticleSourceStatus>,
) -> Option<ArticleSearchJsonMeta> {
    let next_commands = super::normalize_next_commands(next_commands);
    let suggestions = suggestions
        .into_iter()
        .filter(|suggestion| {
            !suggestion.command.trim().is_empty() && !suggestion.reason.trim().is_empty()
        })
        .collect::<Vec<_>>();

    (!warnings.is_empty()
        || !next_commands.is_empty()
        || !suggestions.is_empty()
        || !source_status.is_empty())
    .then_some(ArticleSearchJsonMeta {
        warnings,
        next_commands,
        suggestions,
        source_status,
    })
}

#[derive(serde::Serialize)]
pub(super) struct ArticleSearchCompactResult<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pmid: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pmcid: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arxiv_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_scholar_id: Option<&'a str>,
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    journal: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    citation_count: Option<u64>,
    source: crate::entities::article::ArticleSource,
    is_retracted: Option<bool>,
}

impl<'a> From<&'a crate::entities::article::ArticleSearchResult>
    for ArticleSearchCompactResult<'a>
{
    fn from(row: &'a crate::entities::article::ArticleSearchResult) -> Self {
        let nonempty = |value: &'a str| (!value.trim().is_empty()).then_some(value);
        Self {
            pmid: nonempty(&row.pmid),
            pmcid: row.pmcid.as_deref().and_then(nonempty),
            doi: row.doi.as_deref().and_then(nonempty),
            arxiv_id: row.arxiv_id.as_deref().and_then(nonempty),
            semantic_scholar_id: row.semantic_scholar_id.as_deref().and_then(nonempty),
            title: &row.title,
            journal: row.journal.as_deref(),
            date: row.date.as_deref(),
            citation_count: row.citation_count,
            source: row.source,
            is_retracted: row.is_retracted,
        }
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub(super) enum ArticleSearchJsonRows<'a> {
    Compact(Vec<ArticleSearchCompactResult<'a>>),
    Full(&'a [crate::entities::article::ArticleSearchResult]),
}

#[derive(Args, Debug)]
pub struct ArticleGetArgs {
    /// PMID (e.g., 22663011), PMCID (e.g., PMC9984800), or DOI (e.g., 10.1056/NEJMoa1203421)
    pub id: String,
    /// Allow Semantic Scholar PDF as a final fulltext fallback (requires fulltext section)
    #[arg(long)]
    pub pdf: bool,
    /// Return a bounded heading outline from cached full text
    #[arg(long, conflicts_with = "lines")]
    pub outline: bool,
    /// Return an inclusive one-based cached full-text line range (START:END)
    #[arg(long, value_name = "START:END", conflicts_with = "outline")]
    pub lines: Option<String>,
    /// Write an article asset to an exact file instead of standard output
    #[arg(long, value_name = "FILE", conflicts_with = "out")]
    pub output: Option<std::path::PathBuf>,
    /// Export full text or one article asset into an existing directory
    #[arg(long, value_name = "DIR", conflicts_with = "output")]
    pub out: Option<std::path::PathBuf>,
    /// Asset manifest view
    #[arg(long = "asset-view", default_value = "compact", value_parser = ["compact", "retrievable", "coverage"])]
    pub asset_view: String,
    /// Page size for retrievable or coverage asset views (1-100)
    #[arg(long = "asset-limit")]
    pub asset_limit: Option<usize>,
    /// Zero-based asset manifest offset
    #[arg(long = "asset-offset", default_value_t = 0)]
    pub asset_offset: usize,
    /// Sections to include (annotations, indexing, fulltext, tldr, assets, asset <asset-key>, all)
    pub sections: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum ArticleCommand {
    /// List provider-exact Semantic Scholar authors for one article
    #[command(after_help = "\
EXAMPLES:
  biomcp article authors 22663011
  biomcp article authors arXiv:2110.01406

See also: biomcp list article")]
    Authors {
        /// PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID
        id: String,
    },
    /// Surface annotated entities from PubTator as discoverable commands
    #[command(after_help = "\
EXAMPLES:
  biomcp article entities 22663011
  biomcp article entities 22663011 --limit 5
  biomcp article entities 24200969

See also: biomcp list article")]
    Entities {
        /// PMID (e.g., 22663011)
        pmid: String,
        /// Maximum related entity commands to surface, 1-50 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Fetch compact summary cards for multiple known article IDs
    #[command(after_help = "\
EXAMPLES:
  biomcp article batch 22663011 24200969
  biomcp article batch 22663011 10.1056/NEJMoa1203421 --json

Returns compact multi-article summary cards for anchor selection.
Semantic Scholar enrichment is optional. With S2_API_KEY, BioMCP uses
authenticated requests at 1 req/sec; without it, BioMCP uses the shared pool at
1 req/2sec.
See also: biomcp list article")]
    Batch {
        /// PMIDs, PMCIDs, or DOIs (repeatable)
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
    },
    /// Traverse citing papers with Semantic Scholar contexts and intents
    #[command(after_help = "\
EXAMPLES:
  biomcp article citations 22663011 --limit 5
  biomcp article citations PMC9984800 --limit 5

Works without S2_API_KEY; authenticated requests are more reliable when the key
is set.
See also: biomcp list article")]
    Citations {
        /// PMID, PMCID, or DOI
        id: String,
        /// Maximum citing papers, 1-100 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Traverse referenced papers with Semantic Scholar contexts and intents
    #[command(after_help = "\
EXAMPLES:
  biomcp article references 22663011 --limit 5
  biomcp article references 10.1056/NEJMoa1203421 --limit 5

Works without S2_API_KEY; authenticated requests are more reliable when the key
is set.
See also: biomcp list article")]
    References {
        /// PMID, PMCID, or DOI
        id: String,
        /// Maximum referenced papers, 1-100 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Find related papers from one or more positive seeds
    #[command(after_help = "\
EXAMPLES:
  biomcp article recommendations 22663011 --limit 5
  biomcp article recommendations 22663011 24200969 --negative 39073865 --limit 5

Works without S2_API_KEY; authenticated requests are more reliable when the key
is set.
See also: biomcp list article")]
    Recommendations {
        /// Positive seed PMIDs, PMCIDs, or DOIs (repeatable)
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Negative seed PMIDs, PMCIDs, or DOIs to repel
        #[arg(long = "negative")]
        negative: Vec<String>,
        /// Maximum recommendations, 1-100 (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

mod assets;
mod dispatch;
mod export;
mod render;
pub(crate) mod session;
mod workflow;
pub(super) use self::dispatch::{handle_command, handle_get, handle_search};
#[cfg(test)]
pub(crate) use self::render::render_loaded_card;

#[cfg(test)]
mod tests;
