use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::PathBuf;

use futures::future::try_join_all;
use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::europepmc::{
    EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse, EuropePmcSort,
};
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_oa::PmcOaClient;
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{
    SemanticScholarCitationEdge, SemanticScholarClient, SemanticScholarPaper,
    SemanticScholarReferenceEdge,
};
use crate::transform;
use crate::utils::date::validate_since;
use crate::utils::download;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ArticleAnnotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_scholar: Option<ArticleSemanticScholar>,
    #[serde(default)]
    pub pubtator_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSemanticScholar {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access_pdf: Option<ArticleSemanticScholarPdf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSemanticScholarPdf {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAnnotations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genes: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diseases: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chemicals: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<AnnotationCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationCount {
    pub text: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleBatchItem {
    pub requested_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_summary: Option<ArticleBatchEntitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tldr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_citation_count: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleBatchEntitySummary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genes: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diseases: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chemicals: Vec<AnnotationCount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mutations: Vec<AnnotationCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSearchResult {
    pub pmid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub influential_citation_count: Option<u64>,
    pub source: ArticleSource,
    #[serde(default)]
    pub matched_sources: Vec<ArticleSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(default)]
    pub is_retracted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_snippet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ranking: Option<ArticleRankingMetadata>,
    #[serde(skip)]
    pub normalized_title: String,
    #[serde(skip)]
    pub normalized_abstract: String,
    #[serde(skip)]
    pub publication_type: Option<String>,
    #[serde(skip)]
    pub insertion_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleRankingMetadata {
    pub directness_tier: u8,
    pub anchor_count: u8,
    pub title_anchor_hits: u8,
    pub abstract_anchor_hits: u8,
    pub combined_anchor_hits: u8,
    pub all_anchors_in_title: bool,
    pub all_anchors_in_text: bool,
    pub study_or_review_cue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleRelatedPaper {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleGraphEdge {
    pub paper: ArticleRelatedPaper,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intents: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<String>,
    pub is_influential: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleGraphResult {
    pub article: ArticleRelatedPaper,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ArticleGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleRecommendationsResult {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positive_seeds: Vec<ArticleRelatedPaper>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_seeds: Vec<ArticleRelatedPaper>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommendations: Vec<ArticleRelatedPaper>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleSource {
    PubTator,
    EuropePmc,
    SemanticScholar,
}

impl ArticleSource {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::PubTator => "PubTator3",
            Self::EuropePmc => "Europe PMC",
            Self::SemanticScholar => "Semantic Scholar",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleSourceFilter {
    #[default]
    All,
    PubTator,
    EuropePmc,
}

impl ArticleSourceFilter {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "all" => Ok(Self::All),
            "pubtator" => Ok(Self::PubTator),
            "europepmc" | "europe-pmc" => Ok(Self::EuropePmc),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected one of: all, pubtator, europepmc."
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::PubTator => "pubtator",
            Self::EuropePmc => "europepmc",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleSort {
    Date,
    Citations,
    #[default]
    Relevance,
}

impl ArticleSort {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        let value = value.trim();
        match value.to_ascii_lowercase().as_str() {
            "date" => Ok(Self::Date),
            "citations" => Ok(Self::Citations),
            "relevance" => Ok(Self::Relevance),
            _ => Err(BioMcpError::InvalidArgument(
                "Invalid article sort. Expected one of: date, citations, relevance".into(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::Citations => "citations",
            Self::Relevance => "relevance",
        }
    }

    fn as_europepmc_sort(self) -> EuropePmcSort {
        match self {
            Self::Date => EuropePmcSort::Date,
            Self::Citations => EuropePmcSort::Citations,
            Self::Relevance => EuropePmcSort::Relevance,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArticleSearchFilters {
    pub gene: Option<String>,
    pub gene_anchored: bool,
    pub disease: Option<String>,
    pub drug: Option<String>,
    pub author: Option<String>,
    pub keyword: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub article_type: Option<String>,
    pub journal: Option<String>,
    pub open_access: bool,
    pub no_preprints: bool,
    pub exclude_retracted: bool,
    pub sort: ArticleSort,
}

const ARTICLE_SECTION_ANNOTATIONS: &str = "annotations";
const ARTICLE_SECTION_FULLTEXT: &str = "fulltext";
const ARTICLE_SECTION_TLDR: &str = "tldr";
const ARTICLE_SECTION_ALL: &str = "all";

pub const ARTICLE_SECTION_NAMES: &[&str] = &[
    ARTICLE_SECTION_ANNOTATIONS,
    ARTICLE_SECTION_FULLTEXT,
    ARTICLE_SECTION_TLDR,
    ARTICLE_SECTION_ALL,
];

const MAX_SEARCH_LIMIT: usize = 50;
pub const ARTICLE_BATCH_MAX_IDS: usize = 20;
const EUROPE_PMC_PAGE_SIZE: usize = 25;
const PUBTATOR_PAGE_SIZE: usize = 25;
const MAX_PAGE_FETCHES: usize = 50;
const WARN_PAGE_THRESHOLD: usize = 20;
const FEDERATED_PAGE_SIZE_CAP: usize = if EUROPE_PMC_PAGE_SIZE < PUBTATOR_PAGE_SIZE {
    EUROPE_PMC_PAGE_SIZE
} else {
    PUBTATOR_PAGE_SIZE
};
const MAX_FEDERATED_FETCH_RESULTS: usize = MAX_PAGE_FETCHES * FEDERATED_PAGE_SIZE_CAP;
const FULLTEXT_CACHE_VERSION: &str = "jats-v1";
const INVALID_ARTICLE_ID_MSG: &str = "\
Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), \
PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., \
e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not \
indexed by PubMed or Europe PMC and cannot be resolved.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendPlan {
    EuropeOnly,
    PubTatorOnly,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntityBiotype {
    Gene,
    Disease,
    Chemical,
}

fn is_doi(id: &str) -> bool {
    let id = id.trim();
    if !id.starts_with("10.") {
        return false;
    }
    id.contains('/')
}

fn parse_pmid(id: &str) -> Option<u32> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    id.parse::<u32>().ok()
}

fn parse_pmcid(id: &str) -> Option<String> {
    let mut id = id.trim();
    if id.len() > 6
        && let Some(prefix) = id.get(..6)
        && prefix.eq_ignore_ascii_case("PMCID:")
        && let Some(rest) = id.get(6..)
    {
        id = rest.trim();
    }
    if id.len() < 4 {
        return None;
    }
    let prefix = id.get(..3)?;
    if !prefix.eq_ignore_ascii_case("PMC") {
        return None;
    }
    let rest = id.get(3..)?.trim();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("PMC{rest}"))
}

#[derive(Debug, Clone)]
enum ArticleIdType {
    Pmc(String),
    Doi(String),
    Pmid(u32),
    Invalid,
}

fn parse_article_id(id: &str) -> ArticleIdType {
    let id = id.trim();
    if let Some(pmcid) = parse_pmcid(id) {
        return ArticleIdType::Pmc(pmcid);
    }
    if is_doi(id) {
        return ArticleIdType::Doi(id.to_string());
    }
    if let Some(pmid) = parse_pmid(id) {
        return ArticleIdType::Pmid(pmid);
    }
    ArticleIdType::Invalid
}

fn parse_arxiv_id(id: &str) -> Option<String> {
    let id = id.trim();
    if id.len() <= 6 {
        return None;
    }
    let prefix = id.get(..6)?;
    if !prefix.eq_ignore_ascii_case("arxiv:") {
        return None;
    }
    let rest = id.get(6..)?.trim();
    if rest.is_empty() {
        return None;
    }
    Some(format!("ARXIV:{rest}"))
}

fn fulltext_cache_key(id: &str) -> String {
    format!("article-fulltext-{FULLTEXT_CACHE_VERSION}:{id}")
}

async fn render_fulltext_xml(xml: String) -> Result<String, BioMcpError> {
    tokio::task::spawn_blocking(move || transform::article::extract_text_from_xml(&xml))
        .await
        .map_err(|err| BioMcpError::Api {
            api: "article".to_string(),
            message: format!("Full text render worker failed: {err}"),
        })
}

fn is_semantic_scholar_paper_id(id: &str) -> bool {
    id.len() == 40 && id.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn semantic_scholar_lookup_id(id: &str) -> Option<String> {
    let id = id.trim();
    if let Some(pmid) = parse_pmid(id) {
        return Some(format!("PMID:{pmid}"));
    }
    if is_doi(id) {
        return Some(format!("DOI:{id}"));
    }
    if let Some(arxiv) = parse_arxiv_id(id) {
        return Some(arxiv);
    }
    if is_semantic_scholar_paper_id(id) {
        return Some(id.to_string());
    }
    None
}

fn is_preprint_journal(journal: &str) -> bool {
    let j = journal.to_ascii_lowercase();
    j.contains("biorxiv") || j.contains("medrxiv") || j.contains("arxiv")
}

fn europepmc_escape(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '\"'
                | '+'
                | '-'
                | '!'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '^'
                | '~'
                | '*'
                | '?'
                | ':'
                | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }

    escaped
}

fn europepmc_phrase(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }
    let escaped = europepmc_escape(value);
    if value.chars().any(|c| c.is_whitespace()) || value.contains('/') {
        format!("\"{escaped}\"")
    } else {
        escaped
    }
}

fn europepmc_keyword(value: &str) -> String {
    europepmc_escape(value)
}

fn normalize_article_type(value: &str) -> Result<&'static str, BioMcpError> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "review" => Ok("review"),
        "research" | "research-article" => Ok("research-article"),
        "case-reports" => Ok("case-reports"),
        "meta-analysis" | "metaanalysis" => Ok("meta-analysis"),
        _ => Err(BioMcpError::InvalidArgument(
            "--type must be one of: review, research, research-article, case-reports, meta-analysis".into(),
        )),
    }
}

fn validate_required_search_filters(filters: &ArticleSearchFilters) -> Result<(), BioMcpError> {
    if filters.gene.is_none()
        && filters.disease.is_none()
        && filters.drug.is_none()
        && filters.author.is_none()
        && filters.keyword.is_none()
        && filters.article_type.is_none()
        && !filters.open_access
    {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search article -g BRAF".into(),
        ));
    }
    Ok(())
}

fn normalized_date_bounds(
    filters: &ArticleSearchFilters,
) -> Result<(Option<String>, Option<String>), BioMcpError> {
    let normalized_date_from = filters
        .date_from
        .as_deref()
        .map(validate_since)
        .transpose()?;
    let normalized_date_to = filters.date_to.as_deref().map(validate_since).transpose()?;
    if let (Some(from), Some(to)) = (
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
    ) && from > to
    {
        return Err(BioMcpError::InvalidArgument(
            "--date-from must be <= --date-to".into(),
        ));
    }
    Ok((normalized_date_from, normalized_date_to))
}

fn has_strict_europepmc_filters(filters: &ArticleSearchFilters) -> bool {
    filters.open_access
        || filters
            .article_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
}

fn plan_backends(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> Result<BackendPlan, BioMcpError> {
    match source {
        ArticleSourceFilter::EuropePmc => Ok(BackendPlan::EuropeOnly),
        ArticleSourceFilter::PubTator => {
            if has_strict_europepmc_filters(filters) {
                return Err(BioMcpError::InvalidArgument(
                    "--source pubtator does not support strict filters --open-access or --type. Use --source europepmc or --source all.".into(),
                ));
            }
            Ok(BackendPlan::PubTatorOnly)
        }
        ArticleSourceFilter::All => {
            if has_strict_europepmc_filters(filters) {
                Ok(BackendPlan::EuropeOnly)
            } else {
                Ok(BackendPlan::Both)
            }
        }
    }
}

pub fn semantic_scholar_search_enabled(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
) -> bool {
    if source != ArticleSourceFilter::All || has_strict_europepmc_filters(filters) {
        return false;
    }
    SemanticScholarClient::new()
        .map(|client| client.is_configured())
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleSearchDebugSummary {
    pub routing: Vec<String>,
    pub sources: Vec<String>,
    pub matched_sources: Vec<String>,
}

pub(crate) fn summarize_debug_plan(
    filters: &ArticleSearchFilters,
    source: ArticleSourceFilter,
    results: &[ArticleSearchResult],
) -> Result<ArticleSearchDebugSummary, BioMcpError> {
    let plan = plan_backends(filters, source)?;
    let planner = match (plan, source) {
        (BackendPlan::EuropeOnly, ArticleSourceFilter::All)
            if has_strict_europepmc_filters(filters) =>
        {
            "planner=europe_only_strict_filters"
        }
        (BackendPlan::EuropeOnly, _) => "planner=europe_only",
        (BackendPlan::PubTatorOnly, _) => "planner=pubtator_only",
        (BackendPlan::Both, _) => "planner=federated",
    };

    let mut sources = match plan {
        BackendPlan::EuropeOnly => vec!["Europe PMC".to_string()],
        BackendPlan::PubTatorOnly => vec!["PubTator3".to_string()],
        BackendPlan::Both => vec!["PubTator3".to_string(), "Europe PMC".to_string()],
    };
    if semantic_scholar_search_enabled(filters, source) {
        sources.push("Semantic Scholar".to_string());
    }

    let matched_sources = [
        ArticleSource::PubTator,
        ArticleSource::EuropePmc,
        ArticleSource::SemanticScholar,
    ]
    .into_iter()
    .filter(|candidate| {
        results.iter().any(|row| {
            row.source == *candidate || row.matched_sources.iter().any(|source| source == candidate)
        })
    })
    .map(|source| source.display_name().to_string())
    .collect();

    Ok(ArticleSearchDebugSummary {
        routing: vec![planner.to_string()],
        sources,
        matched_sources,
    })
}

fn matches_entity_biotype(value: Option<&str>, expected: EntityBiotype) -> bool {
    let Some(value) = value else {
        return false;
    };
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return false;
    }
    match expected {
        EntityBiotype::Gene => normalized.contains("gene"),
        EntityBiotype::Disease => normalized.contains("disease"),
        EntityBiotype::Chemical => normalized.contains("chemical") || normalized.contains("drug"),
    }
}

async fn normalize_entity_token(
    pubtator: &PubTatorClient,
    token: Option<&str>,
    expected: EntityBiotype,
) -> Option<String> {
    let token = token.map(str::trim).filter(|value| !value.is_empty())?;
    match pubtator.entity_autocomplete(token).await {
        Ok(rows) => rows
            .iter()
            .find(|row| matches_entity_biotype(row.biotype.as_deref(), expected))
            .and_then(|row| row.id.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .or_else(|| Some(token.to_string())),
        Err(err) => {
            warn!(
                ?err,
                token, "pubtator autocomplete failed; falling back to raw token"
            );
            Some(token.to_string())
        }
    }
}

fn pubtator_sort(sort: ArticleSort) -> Option<&'static str> {
    match sort {
        ArticleSort::Date => Some("date desc"),
        ArticleSort::Citations | ArticleSort::Relevance => None,
    }
}

fn pubtator_source_can_satisfy(filters: &ArticleSearchFilters) -> bool {
    !filters.exclude_retracted
}

fn parse_row_date(value: Option<&str>) -> Option<String> {
    let value = value.map(str::trim).filter(|v| !v.is_empty())?;
    let truncated = value.get(0..10).unwrap_or(value);
    match truncated.len() {
        4 => Some(format!("{truncated}-01-01")),
        7 => Some(format!("{truncated}-01")),
        _ => Some(truncated.to_string()),
    }
}

fn matches_optional_journal_filter(
    row_journal: Option<&str>,
    expected_journal: Option<&str>,
) -> bool {
    let Some(expected) = expected_journal
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return true;
    };
    let Some(actual) = row_journal.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    actual
        .to_ascii_lowercase()
        .contains(&expected.to_ascii_lowercase())
}

fn matches_optional_date_filter(
    row_date: Option<&str>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> bool {
    if date_from.is_none() && date_to.is_none() {
        return true;
    }
    let Some(value) = parse_row_date(row_date) else {
        return false;
    };
    if let Some(from) = date_from
        && value.as_str() < from
    {
        return false;
    }
    if let Some(to) = date_to
        && value.as_str() > to
    {
        return false;
    }
    true
}

fn matches_result_filters(
    row: &ArticleSearchResult,
    filters: &ArticleSearchFilters,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> bool {
    if filters.no_preprints && row.journal.as_deref().is_some_and(is_preprint_journal) {
        return false;
    }
    if filters.exclude_retracted && row.is_retracted.unwrap_or(true) {
        return false;
    }
    if !matches_optional_journal_filter(row.journal.as_deref(), filters.journal.as_deref()) {
        return false;
    }
    if !matches_optional_date_filter(row.date.as_deref(), date_from, date_to) {
        return false;
    }
    true
}

fn normalize_row_identifier(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn article_source_priority(source: ArticleSource) -> u8 {
    match source {
        ArticleSource::PubTator => 0,
        ArticleSource::EuropePmc => 1,
        ArticleSource::SemanticScholar => 2,
    }
}

fn stable_article_identifier(row: &ArticleSearchResult) -> String {
    normalize_row_identifier(Some(&row.pmid))
        .or_else(|| normalize_row_identifier(row.pmcid.as_deref()))
        .or_else(|| normalize_row_identifier(row.doi.as_deref()))
        .unwrap_or_else(|| row.title.to_ascii_lowercase())
}

fn ensure_matched_sources(row: &mut ArticleSearchResult) {
    if !row.matched_sources.contains(&row.source) {
        row.matched_sources.push(row.source);
    }
    row.matched_sources
        .sort_by_key(|source| article_source_priority(*source));
    row.matched_sources.dedup();
}

fn article_rows_overlap(left: &ArticleSearchResult, right: &ArticleSearchResult) -> bool {
    let left_pmid = normalize_row_identifier(Some(&left.pmid));
    let right_pmid = normalize_row_identifier(Some(&right.pmid));
    let left_pmcid = normalize_row_identifier(left.pmcid.as_deref());
    let right_pmcid = normalize_row_identifier(right.pmcid.as_deref());
    let left_doi = normalize_row_identifier(left.doi.as_deref());
    let right_doi = normalize_row_identifier(right.doi.as_deref());

    left_pmid.is_some() && left_pmid == right_pmid
        || left_pmcid.is_some() && left_pmcid == right_pmcid
        || left_doi.is_some() && left_doi == right_doi
}

fn merge_missing_string(target: &mut Option<String>, incoming: Option<String>) {
    if target
        .as_deref()
        .map(str::trim)
        .is_none_or(|value| value.is_empty())
    {
        *target = incoming
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
}

fn merge_missing_u64(target: &mut Option<u64>, incoming: Option<u64>) {
    if target.is_none() {
        *target = incoming;
    }
}

fn merge_article_candidate(target: &mut ArticleSearchResult, incoming: ArticleSearchResult) {
    merge_missing_string(&mut target.pmcid, incoming.pmcid);
    merge_missing_string(&mut target.doi, incoming.doi);
    if target.pmid.trim().is_empty() && !incoming.pmid.trim().is_empty() {
        target.pmid = incoming.pmid;
    }
    if target.title.trim().is_empty() && !incoming.title.trim().is_empty() {
        target.title = incoming.title;
    }
    merge_missing_string(&mut target.journal, incoming.journal);
    merge_missing_string(&mut target.date, incoming.date);
    merge_missing_u64(&mut target.citation_count, incoming.citation_count);
    merge_missing_u64(
        &mut target.influential_citation_count,
        incoming.influential_citation_count,
    );
    if target.score.is_none() {
        target.score = incoming.score;
    }
    if target.is_retracted.is_none() && incoming.is_retracted.is_some() {
        target.is_retracted = incoming.is_retracted;
    }
    merge_missing_string(&mut target.abstract_snippet, incoming.abstract_snippet);
    if target.ranking.is_none() {
        target.ranking = incoming.ranking;
    }
    if target.normalized_title.is_empty() && !incoming.normalized_title.is_empty() {
        target.normalized_title = incoming.normalized_title;
    }
    if target.normalized_abstract.is_empty() && !incoming.normalized_abstract.is_empty() {
        target.normalized_abstract = incoming.normalized_abstract;
    }
    merge_missing_string(&mut target.publication_type, incoming.publication_type);
    target.insertion_index = target.insertion_index.min(incoming.insertion_index);
    target.matched_sources.extend(incoming.matched_sources);
    ensure_matched_sources(target);
}

fn merge_article_candidates(results: Vec<ArticleSearchResult>) -> Vec<ArticleSearchResult> {
    let mut merged: Vec<ArticleSearchResult> = Vec::with_capacity(results.len());

    for mut row in results {
        ensure_matched_sources(&mut row);
        let matches = merged
            .iter()
            .enumerate()
            .filter_map(|(idx, existing)| article_rows_overlap(existing, &row).then_some(idx))
            .collect::<Vec<_>>();

        if matches.is_empty() {
            merged.push(row);
            continue;
        }

        let keep_idx = matches[0];
        merge_article_candidate(&mut merged[keep_idx], row);
        for idx in matches.into_iter().skip(1).rev() {
            let duplicate = merged.remove(idx);
            merge_article_candidate(&mut merged[keep_idx], duplicate);
        }
    }

    merged
}

fn compare_optional_dates_desc(
    left: Option<&ArticleSearchResult>,
    right: Option<&ArticleSearchResult>,
) -> Ordering {
    match (
        left.and_then(|row| parse_row_date(row.date.as_deref())),
        right.and_then(|row| parse_row_date(row.date.as_deref())),
    ) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_citations_desc(
    left: Option<&ArticleSearchResult>,
    right: Option<&ArticleSearchResult>,
) -> Ordering {
    match (
        left.and_then(|row| row.citation_count),
        right.and_then(|row| row.citation_count),
    ) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn build_anchor_set(filters: &ArticleSearchFilters) -> Vec<String> {
    let mut anchors = Vec::new();
    let mut seen = HashSet::new();
    for value in [
        filters.gene.as_deref(),
        filters.disease.as_deref(),
        filters.drug.as_deref(),
        filters.keyword.as_deref(),
    ] {
        let Some(anchor) = value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(transform::article::normalize_article_search_text)
        else {
            continue;
        };
        if seen.insert(anchor.clone()) {
            anchors.push(anchor);
        }
    }
    anchors
}

fn anchor_matches_text(text: &str, anchor: &str) -> bool {
    if anchor.is_empty() || text.is_empty() {
        return false;
    }
    if anchor.chars().any(|ch| ch.is_whitespace()) {
        return text.contains(anchor);
    }

    for (idx, _) in text.match_indices(anchor) {
        let start_ok = text[..idx]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric());
        let end_idx = idx + anchor.len();
        let end_ok = text[end_idx..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric());
        if start_ok && end_ok {
            return true;
        }
    }
    false
}

fn has_study_or_review_cue(row: &ArticleSearchResult) -> bool {
    let title = row.normalized_title.as_str();
    let publication_type = row
        .publication_type
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    [
        "review",
        "meta-analysis",
        "meta analysis",
        "systematic review",
        "clinical trial",
    ]
    .into_iter()
    .any(|cue| title.contains(cue) || publication_type.contains(cue))
}

fn rank_articles_by_directness(rows: &mut [ArticleSearchResult], filters: &ArticleSearchFilters) {
    let anchors = build_anchor_set(filters);

    for row in rows.iter_mut() {
        ensure_matched_sources(row);
        let title_hits = anchors
            .iter()
            .filter(|anchor| anchor_matches_text(&row.normalized_title, anchor))
            .count();
        let abstract_hits = anchors
            .iter()
            .filter(|anchor| anchor_matches_text(&row.normalized_abstract, anchor))
            .count();
        let combined_hits = anchors
            .iter()
            .filter(|anchor| {
                anchor_matches_text(&row.normalized_title, anchor)
                    || anchor_matches_text(&row.normalized_abstract, anchor)
            })
            .count();
        let anchor_count = anchors.len();
        let all_anchors_in_title = anchor_count > 0 && title_hits == anchor_count;
        let all_anchors_in_text = anchor_count > 0 && combined_hits == anchor_count;
        let directness_tier = if all_anchors_in_title {
            3
        } else if all_anchors_in_text {
            2
        } else if combined_hits > 0 {
            1
        } else {
            0
        };

        row.ranking = Some(ArticleRankingMetadata {
            directness_tier,
            anchor_count: anchor_count.min(u8::MAX as usize) as u8,
            title_anchor_hits: title_hits.min(u8::MAX as usize) as u8,
            abstract_anchor_hits: abstract_hits.min(u8::MAX as usize) as u8,
            combined_anchor_hits: combined_hits.min(u8::MAX as usize) as u8,
            all_anchors_in_title,
            all_anchors_in_text,
            study_or_review_cue: has_study_or_review_cue(row),
        });
    }

    rows.sort_by(|left, right| {
        let left_ranking = left.ranking.as_ref();
        let right_ranking = right.ranking.as_ref();
        right_ranking
            .map(|ranking| ranking.directness_tier)
            .cmp(&left_ranking.map(|ranking| ranking.directness_tier))
            .then_with(|| {
                right_ranking
                    .map(|ranking| ranking.title_anchor_hits)
                    .cmp(&left_ranking.map(|ranking| ranking.title_anchor_hits))
            })
            .then_with(|| {
                right_ranking
                    .map(|ranking| ranking.combined_anchor_hits)
                    .cmp(&left_ranking.map(|ranking| ranking.combined_anchor_hits))
            })
            .then_with(|| {
                right_ranking
                    .map(|ranking| ranking.study_or_review_cue)
                    .cmp(&left_ranking.map(|ranking| ranking.study_or_review_cue))
            })
            .then_with(|| compare_optional_citations_desc(Some(left), Some(right)))
            .then_with(|| {
                right
                    .influential_citation_count
                    .cmp(&left.influential_citation_count)
            })
            .then_with(|| left.insertion_index.cmp(&right.insertion_index))
            .then_with(|| stable_article_identifier(left).cmp(&stable_article_identifier(right)))
    });
}

fn sort_article_rows(
    rows: &mut [ArticleSearchResult],
    sort: ArticleSort,
    filters: &ArticleSearchFilters,
) {
    match sort {
        ArticleSort::Relevance => rank_articles_by_directness(rows, filters),
        ArticleSort::Citations => rows.sort_by(|left, right| {
            compare_optional_citations_desc(Some(left), Some(right))
                .then_with(|| compare_optional_dates_desc(Some(left), Some(right)))
                .then_with(|| left.pmid.cmp(&right.pmid))
        }),
        ArticleSort::Date => rows.sort_by(|left, right| {
            compare_optional_dates_desc(Some(left), Some(right))
                .then_with(|| compare_optional_citations_desc(Some(left), Some(right)))
                .then_with(|| left.pmid.cmp(&right.pmid))
        }),
    }
}

fn build_search_query(filters: &ArticleSearchFilters) -> Result<String, BioMcpError> {
    validate_required_search_filters(filters)?;
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let mut terms: Vec<String> = Vec::new();

    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if filters.gene_anchored {
            terms.push(format!("GENE_PROTEIN:{}", europepmc_phrase(gene)));
        } else {
            terms.push(europepmc_phrase(gene));
        }
    }
    if let Some(disease) = filters
        .disease
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(disease));
    }
    if let Some(drug) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_phrase(drug));
    }
    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("AUTH:{}", europepmc_phrase(author)));
    }
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(europepmc_keyword(keyword));
    }

    if let Some(article_type) = filters
        .article_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let normalized = normalize_article_type(article_type)?;
        terms.push(format!("PUB_TYPE:\"{normalized}\""));
    }

    if let Some(journal) = filters
        .journal
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!("JOURNAL:{}", europepmc_phrase(journal)));
    }

    if filters.open_access {
        terms.push("OPEN_ACCESS:y".into());
    }

    if let Some(from) = normalized_date_from.as_deref() {
        let to = normalized_date_to.as_deref().unwrap_or("*");
        terms.push(format!("FIRST_PDATE:[{from} TO {to}]"));
    } else if let Some(to) = normalized_date_to.as_deref() {
        terms.push(format!("FIRST_PDATE:[* TO {to}]"));
    }

    if filters.no_preprints {
        terms.push("NOT SRC:PPR".into());
    }
    if filters.exclude_retracted {
        terms.push("NOT PUB_TYPE:\"retracted publication\"".into());
    }

    Ok(terms.join(" AND "))
}

async fn build_pubtator_query(
    filters: &ArticleSearchFilters,
    pubtator: &PubTatorClient,
) -> Result<String, BioMcpError> {
    validate_required_search_filters(filters)?;
    let gene = normalize_entity_token(pubtator, filters.gene.as_deref(), EntityBiotype::Gene).await;
    let disease =
        normalize_entity_token(pubtator, filters.disease.as_deref(), EntityBiotype::Disease).await;
    let drug =
        normalize_entity_token(pubtator, filters.drug.as_deref(), EntityBiotype::Chemical).await;

    let mut terms: Vec<String> = Vec::new();
    if let Some(gene) = gene {
        terms.push(gene);
    }
    if let Some(disease) = disease {
        terms.push(disease);
    }
    if let Some(drug) = drug {
        terms.push(drug);
    }
    if let Some(author) = filters
        .author
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(author.to_string());
    }
    if let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(keyword.to_string());
    }

    if terms.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one queryable token is required for --source pubtator.".into(),
        ));
    }

    Ok(terms.join(" "))
}

#[derive(Debug, Clone, Copy, Default)]
struct ArticleSections {
    include_annotations: bool,
    include_fulltext: bool,
    include_tldr: bool,
    include_all: bool,
}

fn parse_sections(sections: &[String]) -> Result<ArticleSections, BioMcpError> {
    let mut out = ArticleSections::default();

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }

        match section.as_str() {
            ARTICLE_SECTION_ANNOTATIONS => out.include_annotations = true,
            ARTICLE_SECTION_FULLTEXT => out.include_fulltext = true,
            ARTICLE_SECTION_TLDR => out.include_tldr = true,
            ARTICLE_SECTION_ALL => out.include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for article. Available: {}",
                    ARTICLE_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if out.include_all {
        out.include_annotations = true;
        out.include_fulltext = true;
        out.include_tldr = true;
    }

    Ok(out)
}

fn is_section_only_request(sections: &[String], include_all: bool) -> bool {
    if include_all {
        return false;
    }
    sections.iter().any(|s| {
        let value = s.trim().to_ascii_lowercase();
        !value.is_empty() && value != "--json" && value != "-j"
    })
}

fn article_not_found(id: &str, suggestion_id: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "article".into(),
        id: id.to_string(),
        suggestion: format!("Try searching: biomcp search article -q \"{suggestion_id}\""),
    }
}

fn semantic_scholar_invalid_id(id: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unsupported identifier format for Semantic Scholar article helpers: '{id}'. Supported: PMID, PMCID, DOI, arXiv, or a Semantic Scholar paper ID."
    ))
}

fn first_europepmc_hit(search: EuropePmcSearchResponse) -> Option<EuropePmcResult> {
    search.result_list.and_then(|l| l.result.into_iter().next())
}

fn related_paper_from_semantic_scholar(paper: &SemanticScholarPaper) -> ArticleRelatedPaper {
    let external_ids = paper.external_ids.as_ref();
    ArticleRelatedPaper {
        paper_id: paper.paper_id.clone(),
        pmid: external_ids.and_then(|ids| ids.pubmed.clone()),
        doi: external_ids.and_then(|ids| ids.doi.clone()),
        arxiv_id: external_ids.and_then(|ids| ids.arxiv.clone()),
        title: paper.title.clone().unwrap_or_default().trim().to_string(),
        journal: paper
            .venue
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        year: paper.year,
    }
}

fn semantic_scholar_enrichment_from_paper(
    paper: &SemanticScholarPaper,
) -> Option<ArticleSemanticScholar> {
    let open_access_pdf = paper.open_access_pdf.as_ref().and_then(|pdf| {
        let url = pdf
            .url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        Some(ArticleSemanticScholarPdf {
            url: url.to_string(),
            status: pdf
                .status
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            license: pdf
                .license
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    });
    let tldr = paper
        .tldr
        .as_ref()
        .and_then(|value| value.text.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if paper.paper_id.is_none()
        && tldr.is_none()
        && paper.citation_count.is_none()
        && paper.influential_citation_count.is_none()
        && paper.reference_count.is_none()
        && paper.is_open_access.is_none()
        && open_access_pdf.is_none()
    {
        return None;
    }

    Some(ArticleSemanticScholar {
        paper_id: paper.paper_id.clone(),
        tldr,
        citation_count: paper.citation_count,
        influential_citation_count: paper.influential_citation_count,
        reference_count: paper.reference_count,
        is_open_access: paper.is_open_access,
        open_access_pdf,
    })
}

async fn resolve_semantic_scholar_input_id(
    id: &str,
    europe: &EuropePmcClient,
) -> Result<String, BioMcpError> {
    if let Some(id) = semantic_scholar_lookup_id(id) {
        return Ok(id);
    }

    if let Some(pmcid) = parse_pmcid(id) {
        let search = europe.search_by_pmcid(&pmcid).await?;
        let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&pmcid, id))?;
        if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
            return Ok(format!("PMID:{pmid}"));
        }
        if let Some(doi) = hit
            .doi
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(format!("DOI:{doi}"));
        }
        return Err(article_not_found(&pmcid, id));
    }

    Err(semantic_scholar_invalid_id(id))
}

async fn resolve_semantic_scholar_seed(
    id: &str,
    client: &SemanticScholarClient,
    europe: &EuropePmcClient,
) -> Result<ArticleRelatedPaper, BioMcpError> {
    let lookup_id = resolve_semantic_scholar_input_id(id, europe).await?;
    let mut rows = client.paper_batch(&[lookup_id]).await?;
    let paper = rows
        .pop()
        .flatten()
        .ok_or_else(|| article_not_found(id, id))?;
    Ok(related_paper_from_semantic_scholar(&paper))
}

fn dedup_related_papers(rows: Vec<ArticleRelatedPaper>) -> Vec<ArticleRelatedPaper> {
    let mut seen: HashSet<String> = HashSet::with_capacity(rows.len());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key = row
            .paper_id
            .as_deref()
            .map(str::to_string)
            .or_else(|| row.pmid.as_deref().map(|value| format!("pmid:{value}")))
            .or_else(|| row.doi.as_deref().map(|value| format!("doi:{value}")))
            .or_else(|| {
                row.arxiv_id
                    .as_deref()
                    .map(|value| format!("arxiv:{value}"))
            })
            .unwrap_or_else(|| row.title.clone());
        if seen.insert(key) {
            out.push(row);
        }
    }
    out
}

async fn resolve_semantic_scholar_seeds(
    ids: &[String],
    client: &SemanticScholarClient,
    europe: &EuropePmcClient,
) -> Result<Vec<ArticleRelatedPaper>, BioMcpError> {
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(resolve_semantic_scholar_seed(id, client, europe).await?);
    }
    Ok(dedup_related_papers(out))
}

fn is_pubtator_lag_error(err: &BioMcpError) -> bool {
    matches!(
        err,
        BioMcpError::Api { api, message }
            if api == "pubtator3" && (message.contains("HTTP 400") || message.contains("HTTP 404"))
    )
}

async fn resolve_article_from_pmid(
    pmid: u32,
    not_found_id: &str,
    suggestion_id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
    europe_hint: Option<&EuropePmcResult>,
) -> Result<Article, BioMcpError> {
    match pubtator.export_biocjson(pmid).await {
        Ok(resp) => {
            let doc = resp
                .documents
                .into_iter()
                .next()
                .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?;

            let mut article = transform::article::from_pubtator_document(&doc);
            if let Some(hit) = europe_hint {
                transform::article::merge_europepmc_metadata(&mut article, hit);
            } else if let Ok(search) = europe.search_by_pmid(&pmid.to_string()).await
                && let Some(hit) = first_europepmc_hit(search)
            {
                transform::article::merge_europepmc_metadata(&mut article, &hit);
            }
            article.annotations = transform::article::extract_annotations(&doc);
            Ok(article)
        }
        Err(err) => {
            if !is_pubtator_lag_error(&err) {
                return Err(err);
            }

            let hit = match europe_hint.cloned() {
                Some(hit) => hit,
                None => {
                    let search = europe.search_by_pmid(&pmid.to_string()).await?;
                    first_europepmc_hit(search)
                        .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?
                }
            };
            let mut article = transform::article::from_europepmc_result(&hit);
            article.pubtator_fallback = true;
            Ok(article)
        }
    }
}

async fn get_article_base_with_clients(
    id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
) -> Result<Article, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "ID is required. Example: biomcp get article 22663011".into(),
        ));
    }
    if id.len() > 512 {
        return Err(BioMcpError::InvalidArgument("ID is too long.".into()));
    }

    match parse_article_id(id) {
        ArticleIdType::Pmid(pmid) => {
            resolve_article_from_pmid(pmid, id, id, pubtator, europe, None).await
        }
        ArticleIdType::Doi(doi) => {
            let search = europe.search_by_doi(&doi).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&doi, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&doi, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &doi, id, pubtator, europe, Some(&hit)).await
            } else {
                Ok(transform::article::from_europepmc_result(&hit))
            }
        }
        ArticleIdType::Pmc(pmcid) => {
            let search = europe.search_by_pmcid(&pmcid).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&pmcid, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&pmcid, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &pmcid, id, pubtator, europe, Some(&hit)).await
            } else {
                Ok(transform::article::from_europepmc_result(&hit))
            }
        }
        ArticleIdType::Invalid => Err(BioMcpError::InvalidArgument(INVALID_ARTICLE_ID_MSG.into())),
    }
}

async fn get_article_base(id: &str) -> Result<Article, BioMcpError> {
    let pubtator = PubTatorClient::new()?;
    let europe = EuropePmcClient::new()?;
    get_article_base_with_clients(id, &pubtator, &europe).await
}

fn trimmed_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn article_batch_title(article: &Article, requested_id: &str) -> String {
    let title = article.title.trim();
    if !title.is_empty() {
        return title.to_string();
    }
    article
        .pmid
        .as_deref()
        .or(article.pmcid.as_deref())
        .or(article.doi.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(requested_id)
        .to_string()
}

fn article_batch_entity_summary(
    annotations: &ArticleAnnotations,
) -> Option<ArticleBatchEntitySummary> {
    fn top_three(rows: &[AnnotationCount]) -> Vec<AnnotationCount> {
        rows.iter().take(3).cloned().collect()
    }

    let summary = ArticleBatchEntitySummary {
        genes: top_three(&annotations.genes),
        diseases: top_three(&annotations.diseases),
        chemicals: top_three(&annotations.chemicals),
        mutations: top_three(&annotations.mutations),
    };

    if summary.genes.is_empty()
        && summary.diseases.is_empty()
        && summary.chemicals.is_empty()
        && summary.mutations.is_empty()
    {
        None
    } else {
        Some(summary)
    }
}

fn article_batch_year(article: &Article) -> Option<u32> {
    let normalized = parse_row_date(article.date.as_deref())?;
    normalized.get(..4)?.parse::<u32>().ok()
}

fn article_batch_semantic_scholar_lookup_id(item: &ArticleBatchItem) -> Option<String> {
    item.pmid
        .as_deref()
        .map(|pmid| format!("PMID:{pmid}"))
        .or_else(|| item.doi.as_deref().map(|doi| format!("DOI:{doi}")))
}

fn article_batch_item_from_article(requested_id: &str, article: &Article) -> ArticleBatchItem {
    let requested_id = requested_id.trim();
    ArticleBatchItem {
        requested_id: requested_id.to_string(),
        pmid: trimmed_opt(article.pmid.as_deref()),
        pmcid: trimmed_opt(article.pmcid.as_deref()),
        doi: trimmed_opt(article.doi.as_deref()),
        title: article_batch_title(article, requested_id),
        journal: trimmed_opt(article.journal.as_deref()),
        year: article_batch_year(article),
        entity_summary: article
            .annotations
            .as_ref()
            .and_then(article_batch_entity_summary),
        tldr: None,
        citation_count: None,
        influential_citation_count: None,
    }
}

fn merge_semantic_scholar_compact_rows(
    items: &mut [ArticleBatchItem],
    item_positions: &[usize],
    rows: Vec<Option<SemanticScholarPaper>>,
) {
    for (idx, paper) in item_positions.iter().zip(rows.into_iter()) {
        let Some(paper) = paper else {
            continue;
        };
        let item = &mut items[*idx];
        item.tldr = paper
            .tldr
            .as_ref()
            .and_then(|value| value.text.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        item.citation_count = paper.citation_count;
        item.influential_citation_count = paper.influential_citation_count;
    }
}

async fn enrich_article_batch_with_semantic_scholar(
    items: &mut [ArticleBatchItem],
) -> Result<(), BioMcpError> {
    let client = SemanticScholarClient::new()?;
    if !client.is_configured() {
        return Ok(());
    }

    let mut lookup_ids = Vec::new();
    let mut item_positions = Vec::new();
    for (idx, item) in items.iter().enumerate() {
        if let Some(lookup_id) = article_batch_semantic_scholar_lookup_id(item) {
            item_positions.push(idx);
            lookup_ids.push(lookup_id);
        }
    }
    if lookup_ids.is_empty() {
        return Ok(());
    }

    match client.paper_batch_compact(&lookup_ids).await {
        Ok(rows) => merge_semantic_scholar_compact_rows(items, &item_positions, rows),
        Err(err) => warn!(?err, "Semantic Scholar batch enrichment failed"),
    }

    Ok(())
}

pub async fn get_batch_compact(ids: &[String]) -> Result<Vec<ArticleBatchItem>, BioMcpError> {
    if ids.len() > ARTICLE_BATCH_MAX_IDS {
        return Err(BioMcpError::InvalidArgument(format!(
            "Article batch is limited to {ARTICLE_BATCH_MAX_IDS} IDs"
        )));
    }

    let pubtator = PubTatorClient::new()?;
    let europe = EuropePmcClient::new()?;
    let articles = try_join_all(
        ids.iter()
            .map(|id| get_article_base_with_clients(id, &pubtator, &europe)),
    )
    .await?;

    let mut items = ids
        .iter()
        .zip(articles.iter())
        .map(|(requested_id, article)| article_batch_item_from_article(requested_id, article))
        .collect::<Vec<_>>();
    enrich_article_batch_with_semantic_scholar(&mut items).await?;
    Ok(items)
}

async fn enrich_article_with_semantic_scholar(article: &mut Article) -> Result<(), BioMcpError> {
    let client = SemanticScholarClient::new()?;
    if !client.is_configured() {
        return Ok(());
    }

    let lookup_id = article
        .pmid
        .as_deref()
        .map(|pmid| format!("PMID:{pmid}"))
        .or_else(|| article.doi.as_deref().map(|doi| format!("DOI:{doi}")));
    let Some(lookup_id) = lookup_id else {
        return Ok(());
    };

    match client.paper_detail(&lookup_id).await {
        Ok(paper) => article.semantic_scholar = semantic_scholar_enrichment_from_paper(&paper),
        Err(err) => warn!(?err, lookup_id, "Semantic Scholar enrichment failed"),
    }

    Ok(())
}

fn semantic_scholar_api_key_required(client: &SemanticScholarClient) -> Result<(), BioMcpError> {
    if client.is_configured() {
        Ok(())
    } else {
        Err(SemanticScholarClient::api_key_required())
    }
}

fn graph_edge_from_citation(edge: SemanticScholarCitationEdge) -> ArticleGraphEdge {
    ArticleGraphEdge {
        paper: related_paper_from_semantic_scholar(&edge.citing_paper),
        intents: edge.intents,
        contexts: edge.contexts,
        is_influential: edge.is_influential.unwrap_or(false),
    }
}

fn graph_edge_from_reference(edge: SemanticScholarReferenceEdge) -> ArticleGraphEdge {
    ArticleGraphEdge {
        paper: related_paper_from_semantic_scholar(&edge.cited_paper),
        intents: edge.intents,
        contexts: edge.contexts,
        is_influential: edge.is_influential.unwrap_or(false),
    }
}

pub async fn search(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0, ArticleSourceFilter::All)
        .await?
        .results)
}

async fn search_europepmc_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let europe = EuropePmcClient::new()?;
    let query = build_search_query(filters)?;
    let europepmc_sort = filters.sort.as_europepmc_sort();
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut page: usize = (offset / EUROPE_PMC_PAGE_SIZE) + 1;
    let mut local_skip = offset % EUROPE_PMC_PAGE_SIZE;
    let mut fetched_pages = 0usize;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages == WARN_PAGE_THRESHOLD + 1 {
            tracing::warn!(
                "article search is deep (>{WARN_PAGE_THRESHOLD} page fetches); continuing up to {MAX_PAGE_FETCHES} — consider narrowing your query"
            );
        }
        let resp = europe
            .search_query_with_sort(&query, page, EUROPE_PMC_PAGE_SIZE, europepmc_sort)
            .await?;
        if total.is_none() {
            total = resp.hit_count.map(|v| v as usize);
            if total.is_some_and(|value| offset >= value) {
                return Ok(SearchPage::offset(Vec::new(), total));
            }
        }
        let Some(results) = resp.result_list.map(|v| v.result) else {
            break;
        };
        if results.is_empty() {
            break;
        }

        for hit in results {
            if local_skip > 0 {
                local_skip -= 1;
                continue;
            }

            let Some(row) = transform::article::from_europepmc_search_result(&hit) else {
                continue;
            };
            if !matches_result_filters(
                &row,
                filters,
                normalized_date_from.as_deref(),
                normalized_date_to.as_deref(),
            ) {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        page += 1;
    }

    // Safety-first default: when date-sorted results contain no visible retraction marker,
    // try adding one matched retracted publication if available.
    if !filters.exclude_retracted
        && filters.sort == ArticleSort::Date
        && !out.iter().any(|row| row.is_retracted == Some(true))
    {
        let retracted_query = format!("({query}) AND PUB_TYPE:\"retracted publication\"");
        if let Ok(resp) = europe
            .search_query_with_sort(&retracted_query, 1, 10, europepmc_sort)
            .await
        {
            let replacement = resp
                .result_list
                .map(|v| v.result)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|hit| transform::article::from_europepmc_search_result(&hit))
                .find(|row| {
                    row.is_retracted == Some(true)
                        && !seen_pmids.contains(&row.pmid)
                        && matches_result_filters(
                            row,
                            filters,
                            normalized_date_from.as_deref(),
                            normalized_date_to.as_deref(),
                        )
                });
            if let Some(row) = replacement {
                if out.len() >= limit && !out.is_empty() {
                    out.pop();
                }
                if out.len() < limit {
                    seen_pmids.insert(row.pmid.clone());
                    out.push(row);
                }
            }
        }
    }

    Ok(SearchPage::offset(out, total))
}

async fn search_pubtator_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    if !pubtator_source_can_satisfy(filters) {
        return Ok(SearchPage::offset(Vec::new(), Some(0)));
    }

    let pubtator = PubTatorClient::new()?;
    let query = build_pubtator_query(filters, &pubtator).await?;
    let sort = pubtator_sort(filters.sort);
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut page: usize = (offset / PUBTATOR_PAGE_SIZE) + 1;
    let mut local_skip = offset % PUBTATOR_PAGE_SIZE;
    let mut fetched_pages = 0usize;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        let resp = pubtator
            .search(&query, page, PUBTATOR_PAGE_SIZE, sort)
            .await?;
        if total.is_none() {
            total = resp.count.map(|v| v as usize);
            if total.is_some_and(|value| offset >= value) {
                return Ok(SearchPage::offset(Vec::new(), total));
            }
        }

        if resp.results.is_empty() {
            break;
        }

        for hit in resp.results {
            if local_skip > 0 {
                local_skip -= 1;
                continue;
            }
            let Some(row) = transform::article::from_pubtator_search_result(&hit) else {
                continue;
            };
            if !matches_result_filters(
                &row,
                filters,
                normalized_date_from.as_deref(),
                normalized_date_to.as_deref(),
            ) {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }
        page += 1;
    }

    Ok(SearchPage::offset(out, total))
}

fn build_semantic_scholar_query(filters: &ArticleSearchFilters) -> String {
    [
        filters.gene.as_deref(),
        filters.disease.as_deref(),
        filters.drug.as_deref(),
        filters.keyword.as_deref(),
        filters.author.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

async fn search_semantic_scholar_candidates(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    if !client.is_configured() {
        return Ok(Vec::new());
    }

    let query = build_semantic_scholar_query(filters);
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let response = match client.paper_search(&query, limit).await {
        Ok(response) => response,
        Err(err) => {
            warn!(?err, query, "Semantic Scholar article search leg failed");
            return Ok(Vec::new());
        }
    };

    let mut rows = Vec::with_capacity(response.data.len());
    for paper in response.data {
        let external_ids = paper.external_ids.as_ref();
        let title = paper
            .title
            .as_deref()
            .map(transform::article::clean_title)
            .unwrap_or_default();
        let abstract_text = paper
            .abstract_text
            .as_deref()
            .map(transform::article::clean_abstract);
        let row = ArticleSearchResult {
            pmid: external_ids
                .and_then(|ids| ids.pubmed.clone())
                .unwrap_or_default()
                .trim()
                .to_string(),
            pmcid: external_ids
                .and_then(|ids| ids.pmcid.clone())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            doi: external_ids
                .and_then(|ids| ids.doi.clone())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            title,
            journal: paper
                .venue
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            date: paper.year.map(|year| year.to_string()),
            citation_count: paper.citation_count,
            influential_citation_count: paper.influential_citation_count,
            source: ArticleSource::SemanticScholar,
            matched_sources: vec![ArticleSource::SemanticScholar],
            score: None,
            is_retracted: None,
            abstract_snippet: abstract_text
                .as_deref()
                .and_then(transform::article::article_search_abstract_snippet),
            ranking: None,
            normalized_title: paper
                .title
                .as_deref()
                .map(transform::article::normalize_article_search_text)
                .unwrap_or_default(),
            normalized_abstract: abstract_text
                .as_deref()
                .map(transform::article::normalize_article_search_text)
                .unwrap_or_default(),
            publication_type: None,
            insertion_index: 0,
        };
        if matches_result_filters(
            &row,
            filters,
            normalized_date_from.as_deref(),
            normalized_date_to.as_deref(),
        ) {
            rows.push(row);
        }
    }

    Ok(rows)
}

fn finalize_article_candidates(
    mut rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
) -> SearchPage<ArticleSearchResult> {
    for (idx, row) in rows.iter_mut().enumerate() {
        row.insertion_index = idx;
        ensure_matched_sources(row);
    }

    let mut rows = merge_article_candidates(rows);
    sort_article_rows(&mut rows, filters.sort, filters);
    rows.retain(|row| !row.pmid.trim().is_empty());
    rows.drain(0..offset.min(rows.len()));
    rows.truncate(limit);
    SearchPage::offset(rows, total)
}

async fn search_federated_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }
    let (pubtator_leg, europe_leg, semantic_scholar_leg) = tokio::join!(
        search_pubtator_page(filters, fetch_count, 0),
        search_europepmc_page(filters, fetch_count, 0),
        search_semantic_scholar_candidates(filters, fetch_count)
    );

    merge_federated_pages(
        pubtator_leg,
        europe_leg,
        semantic_scholar_leg,
        limit,
        offset,
        filters,
    )
}

fn merge_federated_pages(
    pubtator_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    europe_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    semantic_scholar_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
    limit: usize,
    offset: usize,
    filters: &ArticleSearchFilters,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let semantic_scholar_rows = match semantic_scholar_leg {
        Ok(rows) => rows,
        Err(err) => {
            warn!(
                ?err,
                "Semantic Scholar search leg failed; continuing without it"
            );
            Vec::new()
        }
    };

    match (pubtator_leg, europe_leg) {
        (Ok(pubtator_page), Ok(europe_page)) => {
            let mut merged = pubtator_page.results;
            merged.extend(europe_page.results);
            merged.extend(semantic_scholar_rows);
            Ok(finalize_article_candidates(
                merged, limit, offset, None, filters,
            ))
        }
        (Ok(pubtator_page), Err(err)) => {
            warn!(
                ?err,
                "Europe PMC search leg failed; returning PubTator-only results"
            );
            let mut rows = pubtator_page.results;
            rows.extend(semantic_scholar_rows);
            Ok(finalize_article_candidates(
                rows, limit, offset, None, filters,
            ))
        }
        (Err(err), Ok(europe_page)) => {
            warn!(
                ?err,
                "PubTator search leg failed; returning Europe PMC-only results"
            );
            let mut rows = europe_page.results;
            rows.extend(semantic_scholar_rows);
            Ok(finalize_article_candidates(
                rows, limit, offset, None, filters,
            ))
        }
        (Err(pubtator_err), Err(europe_err)) => {
            warn!(?europe_err, "Europe PMC leg also failed");
            Err(pubtator_err)
        }
    }
}

async fn search_relevance_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    plan: BackendPlan,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }

    match plan {
        BackendPlan::EuropeOnly => {
            let page = search_europepmc_page(filters, fetch_count, 0).await?;
            Ok(finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
            ))
        }
        BackendPlan::PubTatorOnly => {
            let page = search_pubtator_page(filters, fetch_count, 0).await?;
            Ok(finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
            ))
        }
        BackendPlan::Both => search_federated_page(filters, limit, offset).await,
    }
}

pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    source: ArticleSourceFilter,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    let plan = plan_backends(filters, source)?;
    if filters.sort == ArticleSort::Relevance {
        return search_relevance_page(filters, limit, offset, plan).await;
    }
    match plan {
        BackendPlan::EuropeOnly => search_europepmc_page(filters, limit, offset).await,
        BackendPlan::PubTatorOnly => search_pubtator_page(filters, limit, offset).await,
        BackendPlan::Both => search_federated_page(filters, limit, offset).await,
    }
}

pub async fn get(id: &str, sections: &[String]) -> Result<Article, BioMcpError> {
    let id = id.trim();
    let section_flags = parse_sections(sections)?;
    let full_text = section_flags.include_fulltext;
    let section_only = is_section_only_request(sections, section_flags.include_all);
    let europe = EuropePmcClient::new()?;
    let mut article = get_article_base(id).await?;

    enrich_article_with_semantic_scholar(&mut article).await?;

    if section_only && !section_flags.include_annotations {
        article.annotations = None;
    }
    if section_only && !section_flags.include_tldr {
        article.semantic_scholar = None;
    }

    if full_text {
        let mut full_text_err: Option<BioMcpError> = None;
        let mut resolved_pmcid = article.pmcid.clone();

        if resolved_pmcid.is_none() {
            match NcbiIdConverterClient::new() {
                Ok(ncbi) => {
                    if let Some(pmid) = article.pmid.as_deref() {
                        match ncbi.pmid_to_pmcid(pmid).await {
                            Ok(v) => resolved_pmcid = v,
                            Err(err) => full_text_err = Some(err),
                        }
                    } else if let Some(doi) = article.doi.as_deref() {
                        match ncbi.doi_to_pmcid(doi).await {
                            Ok(v) => resolved_pmcid = v,
                            Err(err) => full_text_err = Some(err),
                        }
                    }
                }
                Err(err) => full_text_err = Some(err),
            }
        }

        if article.pmcid.is_none() {
            article.pmcid = resolved_pmcid.clone();
        }

        let mut xml: Option<String> = None;
        if let Some(pmcid) = resolved_pmcid.as_deref() {
            match europe.get_full_text_xml("PMC", pmcid).await {
                Ok(v) => xml = v,
                Err(err) => full_text_err = Some(err),
            }
        }
        if xml.is_none()
            && let Some(pmcid) = resolved_pmcid.as_deref()
        {
            match PmcOaClient::new() {
                Ok(pmc_oa) => match pmc_oa.get_full_text_xml(pmcid).await {
                    Ok(v) => xml = v,
                    Err(err) => full_text_err = Some(err),
                },
                Err(err) => full_text_err = Some(err),
            }
        }
        if xml.is_none()
            && let Some(pmid) = article.pmid.as_deref()
        {
            match europe.get_full_text_xml("MED", pmid).await {
                Ok(v) => xml = v,
                Err(err) => full_text_err = Some(err),
            }
        }

        if let Some(xml) = xml {
            let text = render_fulltext_xml(xml).await?;
            let key = article
                .pmid
                .as_deref()
                .or(article.doi.as_deref())
                .or(article.pmcid.as_deref())
                .unwrap_or(id);
            let path = download::save_atomic(&fulltext_cache_key(key), &text).await?;
            article.full_text_path = Some(path);
            article.full_text_note = None;
        } else if let Some(err) = full_text_err {
            warn!(?err, id, "Full text retrieval failed");
            article.full_text_note = Some("Full text not available: API error".into());
        } else if article.pmcid.is_none() {
            article.full_text_note =
                Some("Full text not available: Article not in PubMed Central".into());
        } else {
            article.full_text_note =
                Some("Full text not available: Full text not available from Europe PMC".into());
        }
    }

    Ok(article)
}

pub async fn citations(id: &str, limit: usize) -> Result<ArticleGraphResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    semantic_scholar_api_key_required(&client)?;
    let europe = EuropePmcClient::new()?;
    let article = resolve_semantic_scholar_seed(id, &client, &europe).await?;
    let graph_id = article
        .paper_id
        .as_deref()
        .map(str::to_string)
        .ok_or_else(|| article_not_found(id, id))?;
    let response = client.paper_citations(&graph_id, limit).await?;

    Ok(ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_citation)
            .collect(),
    })
}

pub async fn references(id: &str, limit: usize) -> Result<ArticleGraphResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    semantic_scholar_api_key_required(&client)?;
    let europe = EuropePmcClient::new()?;
    let article = resolve_semantic_scholar_seed(id, &client, &europe).await?;
    let graph_id = article
        .paper_id
        .as_deref()
        .map(str::to_string)
        .ok_or_else(|| article_not_found(id, id))?;
    let response = client.paper_references(&graph_id, limit).await?;

    Ok(ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_reference)
            .collect(),
    })
}

pub async fn recommendations(
    ids: &[String],
    negative: &[String],
    limit: usize,
) -> Result<ArticleRecommendationsResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    semantic_scholar_api_key_required(&client)?;
    let europe = EuropePmcClient::new()?;
    let positive_seeds = resolve_semantic_scholar_seeds(ids, &client, &europe).await?;
    let negative_seeds = resolve_semantic_scholar_seeds(negative, &client, &europe).await?;
    if positive_seeds.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one positive article seed is required. Example: biomcp article recommendations 22663011".into(),
        ));
    }

    let positive_ids: Vec<String> = positive_seeds
        .iter()
        .filter_map(|paper| paper.paper_id.clone())
        .collect();
    let negative_ids: Vec<String> = negative_seeds
        .iter()
        .filter_map(|paper| paper.paper_id.clone())
        .collect();
    let positive_set: HashSet<&str> = positive_ids.iter().map(String::as_str).collect();
    if let Some(conflict) = negative_ids
        .iter()
        .map(String::as_str)
        .find(|paper_id| positive_set.contains(paper_id))
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "The same paper cannot appear in both positive and negative recommendation seeds ({conflict})"
        )));
    }

    let response = if positive_ids.len() == 1 && negative_ids.is_empty() {
        client
            .recommendations_for_paper(&positive_ids[0], limit)
            .await?
    } else {
        client
            .recommendations(&positive_ids, &negative_ids, limit)
            .await?
    };

    Ok(ArticleRecommendationsResult {
        positive_seeds,
        negative_seeds,
        recommendations: response
            .recommended_papers
            .into_iter()
            .map(|paper| related_paper_from_semantic_scholar(&paper))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_filters() -> ArticleSearchFilters {
        ArticleSearchFilters {
            gene: None,
            gene_anchored: false,
            disease: None,
            drug: None,
            author: None,
            keyword: None,
            date_from: None,
            date_to: None,
            article_type: None,
            journal: None,
            open_access: false,
            no_preprints: false,
            exclude_retracted: false,
            sort: ArticleSort::Relevance,
        }
    }

    #[test]
    fn article_sort_default_is_relevance() {
        let default: ArticleSort = Default::default();
        assert_eq!(default, ArticleSort::Relevance);
    }

    #[test]
    fn pubtator_sort_omits_param_for_relevance() {
        assert_eq!(pubtator_sort(ArticleSort::Relevance), None);
    }

    #[test]
    fn pubtator_sort_sends_param_for_date() {
        assert_eq!(pubtator_sort(ArticleSort::Date), Some("date desc"));
    }

    #[test]
    fn empty_filters_default_sort_is_relevance() {
        let filters = empty_filters();
        assert_eq!(filters.sort, ArticleSort::Relevance);
    }

    #[test]
    fn article_section_names_include_tldr() {
        assert!(ARTICLE_SECTION_NAMES.contains(&"tldr"));
    }

    #[test]
    fn fulltext_cache_key_is_versioned() {
        let key = fulltext_cache_key("22663011");
        assert!(key.starts_with("article-fulltext-jats-v1:"));
        assert!(key.ends_with("22663011"));
    }

    #[test]
    fn parse_sections_supports_tldr_and_all() {
        let tldr_only = parse_sections(&["tldr".to_string()]).expect("tldr should parse");
        assert!(tldr_only.include_tldr);
        assert!(!tldr_only.include_annotations);
        assert!(!tldr_only.include_fulltext);

        let all = parse_sections(&["all".to_string()]).expect("all should parse");
        assert!(all.include_tldr);
        assert!(all.include_annotations);
        assert!(all.include_fulltext);
    }

    #[test]
    fn semantic_scholar_lookup_id_supports_arxiv_and_paper_ids() {
        assert_eq!(
            semantic_scholar_lookup_id("arXiv:2401.01234"),
            Some("ARXIV:2401.01234".to_string())
        );
        assert_eq!(
            semantic_scholar_lookup_id("0123456789abcdef0123456789abcdef01234567"),
            Some("0123456789abcdef0123456789abcdef01234567".to_string())
        );
    }

    #[test]
    fn is_doi_basic() {
        assert!(is_doi("10.1056/NEJMoa1203421"));
        assert!(is_doi("10.1056/nejmoa1203421"));
        assert!(!is_doi("22663011"));
        assert!(!is_doi("doi:10.1056/NEJMoa1203421"));
    }

    #[test]
    fn parse_pmid_basic() {
        assert_eq!(parse_pmid("22663011"), Some(22663011));
        assert_eq!(parse_pmid(" 22663011 "), Some(22663011));
        assert_eq!(parse_pmid(""), None);
        assert_eq!(parse_pmid("10.1056/NEJMoa1203421"), None);
        assert_eq!(parse_pmid("abc"), None);
    }

    #[test]
    fn parse_pmcid_basic() {
        assert_eq!(parse_pmcid("PMC9984800"), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid("pmc9984800"), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid("PMCID:PMC9984800"), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid(" PMC9984800 "), Some("PMC9984800".into()));
        assert_eq!(parse_pmcid("PMC"), None);
        assert_eq!(parse_pmcid("PMCX"), None);
        assert_eq!(parse_pmcid("PMC-123"), None);
        assert_eq!(parse_pmcid("22663011"), None);
    }

    #[test]
    fn parse_article_id_basic() {
        match parse_article_id("PMC9984800") {
            ArticleIdType::Pmc(v) => assert_eq!(v, "PMC9984800"),
            _ => panic!("expected PMCID"),
        }
        match parse_article_id("10.1056/NEJMoa1203421") {
            ArticleIdType::Doi(v) => assert_eq!(v, "10.1056/NEJMoa1203421"),
            _ => panic!("expected DOI"),
        }
        match parse_article_id("22663011") {
            ArticleIdType::Pmid(v) => assert_eq!(v, 22663011),
            _ => panic!("expected PMID"),
        }
        assert!(matches!(
            parse_article_id("doi:10.1056/NEJMoa1203421"),
            ArticleIdType::Invalid
        ));
    }

    #[test]
    fn parse_article_id_publisher_pii_is_invalid() {
        assert!(matches!(
            parse_article_id("S1535610826000103"),
            ArticleIdType::Invalid
        ));
    }

    #[test]
    fn article_error_copy_and_warn_threshold_match_contract() {
        assert_eq!(WARN_PAGE_THRESHOLD, 20);
        assert_eq!(
            INVALID_ARTICLE_ID_MSG,
            "Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
        );
    }

    #[test]
    fn invalid_article_id_error_names_supported_types_and_publisher_limit() {
        assert!(INVALID_ARTICLE_ID_MSG.contains("PMID"));
        assert!(INVALID_ARTICLE_ID_MSG.contains("PMCID"));
        assert!(INVALID_ARTICLE_ID_MSG.contains("DOI"));
        assert!(
            INVALID_ARTICLE_ID_MSG.contains("PII") || INVALID_ARTICLE_ID_MSG.contains("publisher")
        );
    }

    #[test]
    fn europepmc_keyword_does_not_quote_whitespace() {
        let term = europepmc_keyword("large language model clinical trials");
        assert_eq!(term, "large language model clinical trials");
    }

    #[test]
    fn build_search_query_keeps_phrase_quoting_for_entity_filters() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF V600E".into());
        filters.author = Some("Jane Doe".into());

        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("\"BRAF V600E\""));
        assert!(query.contains("AUTH:\"Jane Doe\""));
    }

    #[test]
    fn build_search_query_uses_gene_anchor_field_when_requested() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.gene_anchored = true;
        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("GENE_PROTEIN:BRAF"));
    }

    #[test]
    fn build_search_query_combines_keyword_and_since() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.keyword = Some("large language model".into());
        filters.date_from = Some("2024-01-01".into());
        filters.no_preprints = true;

        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("BRAF"));
        assert!(query.contains("large language model"));
        assert!(query.contains("FIRST_PDATE:[2024-01-01 TO *]"));
        assert!(query.contains("NOT SRC:PPR"));
    }

    #[test]
    fn build_search_query_excludes_retracted_when_requested() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.exclude_retracted = true;
        let query = build_search_query(&filters).expect("query should build");
        assert!(query.contains("NOT PUB_TYPE:\"retracted publication\""));
    }

    #[test]
    fn normalize_article_type_accepts_aliases() {
        assert_eq!(
            normalize_article_type("review").expect("review should normalize"),
            "review"
        );
        assert_eq!(
            normalize_article_type("research").expect("research alias should normalize"),
            "research-article"
        );
        assert_eq!(
            normalize_article_type("research-article").expect("research-article should normalize"),
            "research-article"
        );
        assert_eq!(
            normalize_article_type("case-reports").expect("case-reports should normalize"),
            "case-reports"
        );
        assert_eq!(
            normalize_article_type("metaanalysis").expect("metaanalysis alias should normalize"),
            "meta-analysis"
        );
    }

    #[test]
    fn build_search_query_rejects_unknown_article_type() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.article_type = Some("invalid".into());

        let err = build_search_query(&filters).expect_err("invalid article type should fail");
        let msg = err.to_string();
        assert!(msg.contains("Invalid argument"));
        assert!(msg.contains("case-reports"));
    }

    #[test]
    fn article_sort_parses_supported_values() {
        assert_eq!(
            ArticleSort::from_flag("date").expect("date should parse"),
            ArticleSort::Date
        );
        assert_eq!(
            ArticleSort::from_flag("citations").expect("citations should parse"),
            ArticleSort::Citations
        );
        assert_eq!(
            ArticleSort::from_flag("relevance").expect("relevance should parse"),
            ArticleSort::Relevance
        );
        assert!(ArticleSort::from_flag("newest").is_err());
    }

    #[test]
    fn article_source_filter_parses_supported_values() {
        assert_eq!(
            ArticleSourceFilter::from_flag("all").expect("all should parse"),
            ArticleSourceFilter::All
        );
        assert_eq!(
            ArticleSourceFilter::from_flag("pubtator").expect("pubtator should parse"),
            ArticleSourceFilter::PubTator
        );
        assert_eq!(
            ArticleSourceFilter::from_flag("europepmc").expect("europepmc should parse"),
            ArticleSourceFilter::EuropePmc
        );
        assert!(ArticleSourceFilter::from_flag("pubmed").is_err());
    }

    #[test]
    fn planner_routes_all_to_europepmc_for_strict_filters() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.open_access = true;

        let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
        assert!(matches!(plan, BackendPlan::EuropeOnly));
    }

    #[test]
    fn planner_rejects_pubtator_with_unsupported_strict_filters() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.article_type = Some("review".into());

        let err = plan_backends(&filters, ArticleSourceFilter::PubTator)
            .expect_err("planner should reject strict-only filter on pubtator");
        assert!(err.to_string().contains("--type"));
    }

    #[test]
    fn summarize_debug_plan_reports_federated_sources_and_matches() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        let results = vec![ArticleSearchResult {
            pmid: "22663011".into(),
            pmcid: None,
            doi: None,
            title: "BRAF melanoma".into(),
            journal: None,
            date: None,
            citation_count: None,
            influential_citation_count: None,
            source: ArticleSource::PubTator,
            matched_sources: vec![ArticleSource::PubTator, ArticleSource::SemanticScholar],
            score: None,
            is_retracted: Some(false),
            abstract_snippet: None,
            ranking: None,
            normalized_title: "braf melanoma".into(),
            normalized_abstract: String::new(),
            publication_type: None,
            insertion_index: 0,
        }];

        let summary =
            summarize_debug_plan(&filters, ArticleSourceFilter::All, &results).expect("summary");

        assert_eq!(summary.routing, vec!["planner=federated".to_string()]);
        assert!(summary.sources.contains(&"PubTator3".to_string()));
        assert!(summary.sources.contains(&"Europe PMC".to_string()));
        assert_eq!(
            summary.matched_sources,
            vec!["PubTator3".to_string(), "Semantic Scholar".to_string()]
        );
    }

    #[test]
    fn summarize_debug_plan_strict_filter_emits_europe_only_strict() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.open_access = true;

        let summary =
            summarize_debug_plan(&filters, ArticleSourceFilter::All, &[]).expect("summary");

        assert_eq!(
            summary.routing,
            vec!["planner=europe_only_strict_filters".to_string()]
        );
        assert_eq!(summary.sources, vec!["Europe PMC".to_string()]);
        assert!(summary.matched_sources.is_empty());
    }

    #[test]
    fn summarize_debug_plan_explicit_pubtator_emits_pubtator_only() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());

        let summary =
            summarize_debug_plan(&filters, ArticleSourceFilter::PubTator, &[]).expect("summary");

        assert_eq!(summary.routing, vec!["planner=pubtator_only".to_string()]);
        assert_eq!(summary.sources, vec!["PubTator3".to_string()]);
        assert!(summary.matched_sources.is_empty());
    }

    #[test]
    fn pubtator_source_cannot_satisfy_exclude_retracted() {
        let include_filters = empty_filters();
        let exclude_filters = ArticleSearchFilters {
            exclude_retracted: true,
            ..empty_filters()
        };

        assert!(pubtator_source_can_satisfy(&include_filters));
        assert!(!pubtator_source_can_satisfy(&exclude_filters));
    }

    #[test]
    fn pubtator_lag_error_is_400_or_404_only() {
        let err_400 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 400 Bad Request: pending".into(),
        };
        let err_404 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 404 Not Found: pending".into(),
        };
        let err_500 = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 500 Internal Server Error".into(),
        };
        let other_api_400 = BioMcpError::Api {
            api: "europepmc".into(),
            message: "HTTP 400 Bad Request".into(),
        };

        assert!(is_pubtator_lag_error(&err_400));
        assert!(is_pubtator_lag_error(&err_404));
        assert!(!is_pubtator_lag_error(&err_500));
        assert!(!is_pubtator_lag_error(&other_api_400));
    }

    fn row(pmid: &str, source: ArticleSource) -> ArticleSearchResult {
        row_with(pmid, source, Some("2025-01-01"), Some(1), Some(false))
    }

    fn row_with(
        pmid: &str,
        source: ArticleSource,
        date: Option<&str>,
        citation_count: Option<u64>,
        is_retracted: Option<bool>,
    ) -> ArticleSearchResult {
        ArticleSearchResult {
            pmid: pmid.to_string(),
            pmcid: None,
            doi: None,
            title: format!("title-{pmid}"),
            journal: Some("Journal".into()),
            date: date.map(str::to_string),
            citation_count,
            influential_citation_count: None,
            source,
            matched_sources: vec![source],
            score: (source == ArticleSource::PubTator).then_some(42.0),
            is_retracted,
            abstract_snippet: None,
            ranking: None,
            normalized_title: format!("title-{pmid}"),
            normalized_abstract: String::new(),
            publication_type: None,
            insertion_index: 0,
        }
    }

    #[test]
    fn merge_federated_pages_dedups_with_pubtator_priority() {
        let pubtator_page = SearchPage::offset(
            vec![
                row("100", ArticleSource::PubTator),
                row("200", ArticleSource::PubTator),
            ],
            Some(2),
        );
        let europe_page = SearchPage::offset(
            vec![
                row("200", ArticleSource::EuropePmc),
                row("300", ArticleSource::EuropePmc),
            ],
            Some(2),
        );

        let merged = merge_federated_pages(
            Ok(pubtator_page),
            Ok(europe_page),
            Ok(Vec::new()),
            3,
            0,
            &empty_filters(),
        )
        .expect("federated merge should succeed");
        assert_eq!(merged.results.len(), 3);
        assert_eq!(merged.results[0].pmid, "100");
        assert_eq!(merged.results[1].pmid, "200");
        assert_eq!(merged.results[2].pmid, "300");
        assert_eq!(merged.results[1].source, ArticleSource::PubTator);
        assert_eq!(merged.total, None);
    }

    #[test]
    fn merge_federated_pages_returns_surviving_pubtator_leg() {
        let pubtator_page = SearchPage::offset(
            vec![
                row("100", ArticleSource::PubTator),
                row("200", ArticleSource::PubTator),
            ],
            Some(50),
        );
        let europe_err = BioMcpError::Api {
            api: "europepmc".into(),
            message: "HTTP 500: upstream".into(),
        };

        let merged = merge_federated_pages(
            Ok(pubtator_page),
            Err(europe_err),
            Ok(Vec::new()),
            2,
            0,
            &empty_filters(),
        )
        .expect("fallback should return pubtator rows");
        assert_eq!(merged.results.len(), 2);
        assert!(
            merged
                .results
                .iter()
                .all(|r| r.source == ArticleSource::PubTator)
        );
        assert_eq!(merged.total, None);
    }

    #[test]
    fn merge_federated_pages_returns_surviving_europe_leg() {
        let pubtator_err = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 500: upstream".into(),
        };
        let europe_page = SearchPage::offset(
            vec![
                row("100", ArticleSource::EuropePmc),
                row("200", ArticleSource::EuropePmc),
                row("300", ArticleSource::EuropePmc),
            ],
            Some(50),
        );

        let merged = merge_federated_pages(
            Err(pubtator_err),
            Ok(europe_page),
            Ok(Vec::new()),
            2,
            0,
            &empty_filters(),
        )
        .expect("fallback should return europe rows");
        assert_eq!(merged.results.len(), 2);
        assert!(
            merged
                .results
                .iter()
                .all(|r| r.source == ArticleSource::EuropePmc)
        );
        assert_eq!(merged.total, None);
    }

    #[test]
    fn merge_federated_pages_sorts_surviving_leg_before_offset() {
        let pubtator_err = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 500: upstream".into(),
        };
        let europe_page = SearchPage::offset(
            vec![
                row_with(
                    "100",
                    ArticleSource::EuropePmc,
                    Some("2024-01-01"),
                    Some(1),
                    Some(false),
                ),
                row_with(
                    "200",
                    ArticleSource::EuropePmc,
                    Some("2025-01-01"),
                    Some(1),
                    Some(false),
                ),
                row_with(
                    "300",
                    ArticleSource::EuropePmc,
                    Some("2023-01-01"),
                    Some(1),
                    Some(false),
                ),
            ],
            Some(3),
        );

        let merged = merge_federated_pages(
            Err(pubtator_err),
            Ok(europe_page),
            Ok(Vec::new()),
            1,
            1,
            &ArticleSearchFilters {
                sort: ArticleSort::Date,
                ..empty_filters()
            },
        )
        .expect("fallback should sort surviving rows before offset");
        assert_eq!(merged.results.len(), 1);
        assert_eq!(merged.results[0].pmid, "100");
    }

    #[test]
    fn merge_federated_pages_returns_first_error_when_both_fail() {
        let pubtator_err = BioMcpError::Api {
            api: "pubtator3".into(),
            message: "HTTP 500: pubtator failed".into(),
        };
        let europe_err = BioMcpError::Api {
            api: "europepmc".into(),
            message: "HTTP 500: europe failed".into(),
        };

        let err = merge_federated_pages(
            Err(pubtator_err),
            Err(europe_err),
            Ok(Vec::new()),
            10,
            0,
            &empty_filters(),
        )
        .expect_err("both failing legs should return first error");
        let msg = err.to_string();
        assert!(msg.contains("pubtator"));
    }

    #[test]
    fn federated_offset_applied_after_merge_not_per_leg() {
        let pubtator_page = SearchPage::offset(
            vec![
                row("100", ArticleSource::PubTator),
                row("200", ArticleSource::PubTator),
                row("300", ArticleSource::PubTator),
                row("400", ArticleSource::PubTator),
                row("500", ArticleSource::PubTator),
            ],
            Some(5),
        );
        let europe_page = SearchPage::offset(
            vec![
                row("600", ArticleSource::EuropePmc),
                row("700", ArticleSource::EuropePmc),
            ],
            Some(2),
        );

        let merged = merge_federated_pages(
            Ok(pubtator_page),
            Ok(europe_page),
            Ok(Vec::new()),
            2,
            3,
            &empty_filters(),
        )
        .expect("federated merge should succeed");

        let pmids: Vec<&str> = merged.results.iter().map(|row| row.pmid.as_str()).collect();
        assert_eq!(pmids, vec!["400", "500"]);
    }

    #[test]
    fn federated_sort_orders_merged_results_for_citations_and_date() {
        let citation_pubtator_page = SearchPage::offset(
            vec![
                row_with(
                    "100",
                    ArticleSource::PubTator,
                    Some("2025-02-01"),
                    Some(50),
                    Some(false),
                ),
                row_with(
                    "200",
                    ArticleSource::PubTator,
                    Some("2024-01-01"),
                    Some(5),
                    Some(false),
                ),
            ],
            Some(2),
        );
        let citation_europe_page = SearchPage::offset(
            vec![
                row_with(
                    "300",
                    ArticleSource::EuropePmc,
                    Some("2025-03-01"),
                    Some(100),
                    Some(false),
                ),
                row_with(
                    "400",
                    ArticleSource::EuropePmc,
                    Some("2024-06-01"),
                    Some(10),
                    Some(false),
                ),
            ],
            Some(2),
        );

        let citation_merged = merge_federated_pages(
            Ok(citation_pubtator_page),
            Ok(citation_europe_page),
            Ok(Vec::new()),
            10,
            0,
            &ArticleSearchFilters {
                sort: ArticleSort::Citations,
                ..empty_filters()
            },
        )
        .expect("citation merge should succeed");
        let citation_pmids: Vec<&str> = citation_merged
            .results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect();
        assert_eq!(citation_pmids, vec!["300", "100", "400", "200"]);

        let date_pubtator_page = SearchPage::offset(
            vec![
                row_with(
                    "500",
                    ArticleSource::PubTator,
                    Some("2025"),
                    Some(25),
                    Some(false),
                ),
                row_with(
                    "600",
                    ArticleSource::PubTator,
                    Some("2024-12-31"),
                    Some(30),
                    Some(false),
                ),
            ],
            Some(2),
        );
        let date_europe_page = SearchPage::offset(
            vec![
                row_with(
                    "700",
                    ArticleSource::EuropePmc,
                    Some("2025-06-01"),
                    Some(10),
                    Some(false),
                ),
                row_with("800", ArticleSource::EuropePmc, None, Some(99), Some(false)),
            ],
            Some(2),
        );

        let date_merged = merge_federated_pages(
            Ok(date_pubtator_page),
            Ok(date_europe_page),
            Ok(Vec::new()),
            10,
            0,
            &ArticleSearchFilters {
                sort: ArticleSort::Date,
                ..empty_filters()
            },
        )
        .expect("date merge should succeed");
        let date_pmids: Vec<&str> = date_merged
            .results
            .iter()
            .map(|row| row.pmid.as_str())
            .collect();
        assert_eq!(date_pmids, vec!["700", "500", "600", "800"]);
    }

    #[test]
    fn partial_date_normalization_and_filtering_are_consistent() {
        assert_eq!(parse_row_date(Some("2024")), Some("2024-01-01".into()));
        assert_eq!(parse_row_date(Some("2024-06")), Some("2024-06-01".into()));
        assert_eq!(
            parse_row_date(Some("2024-06-15")),
            Some("2024-06-15".into())
        );

        assert!(matches_optional_date_filter(
            Some("2024"),
            Some("2024-01-01"),
            None,
        ));
        assert!(!matches_optional_date_filter(
            Some("2023"),
            Some("2024-01-01"),
            None,
        ));
        assert!(matches_optional_date_filter(
            Some("2024-06"),
            None,
            Some("2024-12-31"),
        ));
    }

    #[test]
    fn article_batch_item_projection_keeps_requested_id_year_and_top_entities() {
        let article = Article {
            pmid: Some("22663011".to_string()),
            pmcid: Some("PMC9984800".to_string()),
            doi: Some("10.1056/NEJMoa1203421".to_string()),
            title: " Improved survival with vemurafenib ".to_string(),
            authors: Vec::new(),
            journal: Some("NEJM".to_string()),
            date: Some("2012-06-07".to_string()),
            citation_count: Some(77),
            publication_type: None,
            open_access: None,
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            annotations: Some(ArticleAnnotations {
                genes: vec![
                    AnnotationCount {
                        text: "BRAF".to_string(),
                        count: 4,
                    },
                    AnnotationCount {
                        text: "NRAS".to_string(),
                        count: 3,
                    },
                    AnnotationCount {
                        text: "MAP2K1".to_string(),
                        count: 2,
                    },
                    AnnotationCount {
                        text: "PTEN".to_string(),
                        count: 1,
                    },
                ],
                diseases: vec![AnnotationCount {
                    text: "melanoma".to_string(),
                    count: 2,
                }],
                chemicals: vec![AnnotationCount {
                    text: "vemurafenib".to_string(),
                    count: 2,
                }],
                mutations: vec![AnnotationCount {
                    text: "V600E".to_string(),
                    count: 3,
                }],
            }),
            semantic_scholar: Some(ArticleSemanticScholar {
                paper_id: Some("paper-1".to_string()),
                tldr: Some("BRAF inhibitor benefit in melanoma.".to_string()),
                citation_count: Some(120),
                influential_citation_count: Some(18),
                reference_count: None,
                is_open_access: None,
                open_access_pdf: None,
            }),
            pubtator_fallback: false,
        };

        let item = article_batch_item_from_article(" 10.1056/NEJMoa1203421 ", &article);
        assert_eq!(item.requested_id, "10.1056/NEJMoa1203421");
        assert_eq!(item.pmid.as_deref(), Some("22663011"));
        assert_eq!(item.pmcid.as_deref(), Some("PMC9984800"));
        assert_eq!(item.doi.as_deref(), Some("10.1056/NEJMoa1203421"));
        assert_eq!(item.title, "Improved survival with vemurafenib");
        assert_eq!(item.journal.as_deref(), Some("NEJM"));
        assert_eq!(item.year, Some(2012));
        assert_eq!(item.tldr, None);
        assert_eq!(item.citation_count, None);
        assert_eq!(item.influential_citation_count, None);

        let entity_summary = item.entity_summary.expect("entity summary");
        assert_eq!(entity_summary.genes.len(), 3);
        assert_eq!(
            entity_summary
                .genes
                .iter()
                .map(|row| row.text.as_str())
                .collect::<Vec<_>>(),
            vec!["BRAF", "NRAS", "MAP2K1"]
        );
        assert_eq!(entity_summary.diseases[0].text, "melanoma");
        assert_eq!(entity_summary.chemicals[0].text, "vemurafenib");
        assert_eq!(entity_summary.mutations[0].text, "V600E");
    }

    #[tokio::test]
    async fn article_batch_rejects_more_than_max_ids_before_network() {
        let ids = (0..ARTICLE_BATCH_MAX_IDS + 1)
            .map(|idx| format!("{}", 22000000 + idx))
            .collect::<Vec<_>>();

        let err = get_batch_compact(&ids)
            .await
            .expect_err("batch over the max should fail");
        assert_eq!(
            err.to_string(),
            format!("Invalid argument: Article batch is limited to {ARTICLE_BATCH_MAX_IDS} IDs")
        );
    }

    #[test]
    fn batch_semantic_scholar_merge_fills_fields_and_skips_none_rows_and_pmcid_only() {
        use crate::sources::semantic_scholar::{SemanticScholarPaper, SemanticScholarTldr};

        fn blank_item(requested_id: &str) -> ArticleBatchItem {
            ArticleBatchItem {
                requested_id: requested_id.to_string(),
                pmid: None,
                pmcid: None,
                doi: None,
                title: String::new(),
                journal: None,
                year: None,
                entity_summary: None,
                tldr: None,
                citation_count: None,
                influential_citation_count: None,
            }
        }

        let mut items = vec![
            ArticleBatchItem {
                pmid: Some("22663011".to_string()),
                ..blank_item("22663011")
            },
            // PMCID-only: not in the S2 lookup list (no PMID or DOI)
            ArticleBatchItem {
                pmcid: Some("PMC9984800".to_string()),
                ..blank_item("PMC9984800")
            },
            // Second PMID lookup — S2 returns None (paper not found)
            ArticleBatchItem {
                pmid: Some("00000000".to_string()),
                ..blank_item("00000000")
            },
        ];

        // positions 0 and 2 have PMIDs; position 1 is PMCID-only and not looked up
        let item_positions = vec![0usize, 2usize];
        let rows: Vec<Option<SemanticScholarPaper>> = vec![
            Some(SemanticScholarPaper {
                tldr: Some(SemanticScholarTldr {
                    text: Some("  Compact summary  ".to_string()),
                    model: None,
                }),
                citation_count: Some(120),
                influential_citation_count: Some(18),
                ..Default::default()
            }),
            None, // S2 returned no match for position 2
        ];

        merge_semantic_scholar_compact_rows(&mut items, &item_positions, rows);

        // Item 0: enriched
        assert_eq!(items[0].tldr.as_deref(), Some("Compact summary")); // whitespace trimmed
        assert_eq!(items[0].citation_count, Some(120));
        assert_eq!(items[0].influential_citation_count, Some(18));

        // Item 1: PMCID-only, not in lookup, untouched
        assert_eq!(items[1].tldr, None);
        assert_eq!(items[1].citation_count, None);

        // Item 2: None row, fields stay unset
        assert_eq!(items[2].tldr, None);
        assert_eq!(items[2].citation_count, None);
    }

    #[test]
    fn exclude_retracted_filters_pubtator_source_rows() {
        let row = row_with(
            "100",
            ArticleSource::PubTator,
            Some("2025-01-01"),
            Some(1),
            Some(true),
        );
        let exclude_filters = ArticleSearchFilters {
            exclude_retracted: true,
            ..empty_filters()
        };
        let include_filters = ArticleSearchFilters {
            exclude_retracted: false,
            ..empty_filters()
        };

        assert!(!matches_result_filters(&row, &exclude_filters, None, None));
        assert!(matches_result_filters(&row, &include_filters, None, None));
    }

    #[test]
    fn exclude_retracted_excludes_unknown_retraction_status_by_default() {
        let row = row_with(
            "100",
            ArticleSource::PubTator,
            Some("2025-01-01"),
            Some(1),
            None,
        );
        let exclude_filters = ArticleSearchFilters {
            exclude_retracted: true,
            ..empty_filters()
        };
        let include_filters = ArticleSearchFilters {
            exclude_retracted: false,
            ..empty_filters()
        };

        assert!(!matches_result_filters(&row, &exclude_filters, None, None));
        assert!(matches_result_filters(&row, &include_filters, None, None));
    }

    #[test]
    fn merge_federated_pages_preserves_known_retraction_status_from_later_duplicate() {
        let pubtator_page = SearchPage::offset(
            vec![row_with(
                "200",
                ArticleSource::PubTator,
                Some("2025-01-01"),
                Some(1),
                None,
            )],
            Some(1),
        );
        let europe_page = SearchPage::offset(
            vec![row_with(
                "200",
                ArticleSource::EuropePmc,
                Some("2025-01-01"),
                Some(10),
                Some(true),
            )],
            Some(1),
        );

        let merged = merge_federated_pages(
            Ok(pubtator_page),
            Ok(europe_page),
            Ok(Vec::new()),
            10,
            0,
            &empty_filters(),
        )
        .expect("federated merge should succeed");

        assert_eq!(merged.results.len(), 1);
        assert_eq!(merged.results[0].source, ArticleSource::PubTator);
        assert_eq!(merged.results[0].is_retracted, Some(true));
    }

    #[test]
    fn article_search_result_serializes_unknown_retraction_as_null() {
        let row = row_with(
            "100",
            ArticleSource::PubTator,
            Some("2025-01-01"),
            Some(1),
            None,
        );

        let value = serde_json::to_value(&row).expect("search row should serialize");
        assert!(value.get("is_retracted").is_some());
        assert!(value["is_retracted"].is_null());
    }

    #[test]
    fn merge_article_candidates_dedups_transitively_across_identifiers() {
        let merged = merge_article_candidates(vec![
            ArticleSearchResult {
                pmid: "100".into(),
                pmcid: Some("PMC100".into()),
                doi: None,
                title: "Primary PMID row".into(),
                journal: Some("Journal".into()),
                date: Some("2025-01-01".into()),
                citation_count: None,
                influential_citation_count: None,
                source: ArticleSource::PubTator,
                score: Some(42.0),
                is_retracted: None,
                abstract_snippet: None,
                ranking: None,
                matched_sources: vec![ArticleSource::PubTator],
                normalized_title: "primary pmid row".into(),
                normalized_abstract: String::new(),
                publication_type: None,
                insertion_index: 0,
            },
            ArticleSearchResult {
                pmid: String::new(),
                pmcid: Some("PMC100".into()),
                doi: Some("10.1000/example".into()),
                title: "Europe metadata".into(),
                journal: Some("Journal".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(15),
                influential_citation_count: None,
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: Some("Europe abstract".into()),
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "europe metadata".into(),
                normalized_abstract: "europe abstract".into(),
                publication_type: Some("Review".into()),
                insertion_index: 1,
            },
            ArticleSearchResult {
                pmid: String::new(),
                pmcid: None,
                doi: Some("10.1000/example".into()),
                title: "Semantic Scholar metadata".into(),
                journal: Some("Journal".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(99),
                influential_citation_count: Some(7),
                source: ArticleSource::SemanticScholar,
                score: None,
                is_retracted: None,
                abstract_snippet: Some("Semantic Scholar abstract".into()),
                ranking: None,
                matched_sources: vec![ArticleSource::SemanticScholar],
                normalized_title: "semantic scholar metadata".into(),
                normalized_abstract: "semantic scholar abstract".into(),
                publication_type: None,
                insertion_index: 2,
            },
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source, ArticleSource::PubTator);
        assert_eq!(merged[0].pmid, "100");
        assert_eq!(merged[0].pmcid.as_deref(), Some("PMC100"));
        assert_eq!(merged[0].doi.as_deref(), Some("10.1000/example"));
        assert_eq!(
            merged[0].matched_sources,
            vec![
                ArticleSource::PubTator,
                ArticleSource::EuropePmc,
                ArticleSource::SemanticScholar,
            ]
        );
        assert_eq!(merged[0].citation_count, Some(15));
        assert_eq!(merged[0].influential_citation_count, Some(7));
        assert_eq!(
            merged[0].abstract_snippet.as_deref(),
            Some("Europe abstract")
        );
        assert_eq!(merged[0].is_retracted, Some(false));
    }

    #[test]
    fn directness_ranking_uses_full_title_and_token_boundaries() {
        let mut filters = empty_filters();
        filters.gene = Some("MET".into());
        filters.keyword = Some("ALL".into());

        let long_prefix =
            "This intentionally long prefix exists to push the anchors well past sixty bytes";
        let mut rows = vec![
            ArticleSearchResult {
                pmid: "100".into(),
                pmcid: None,
                doi: None,
                title: format!("{long_prefix} MET ALL response study"),
                journal: Some("Journal A".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(10),
                influential_citation_count: Some(1),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: Some("Direct abstract".into()),
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: format!(
                    "{} met all response study",
                    long_prefix.to_ascii_lowercase()
                ),
                normalized_abstract: "direct abstract".into(),
                publication_type: None,
                insertion_index: 0,
            },
            ArticleSearchResult {
                pmid: "200".into(),
                pmcid: None,
                doi: None,
                title: "Meta-analysis of small molecule therapy".into(),
                journal: Some("Journal B".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(500),
                influential_citation_count: Some(50),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: None,
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "meta-analysis of small molecule therapy".into(),
                normalized_abstract: String::new(),
                publication_type: Some("Meta-Analysis".into()),
                insertion_index: 1,
            },
            ArticleSearchResult {
                pmid: "300".into(),
                pmcid: None,
                doi: None,
                title: "ALL biomarker response study".into(),
                journal: Some("Journal C".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(100),
                influential_citation_count: Some(5),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: Some("MET is discussed in the abstract".into()),
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "all biomarker response study".into(),
                normalized_abstract: "met is discussed in the abstract".into(),
                publication_type: None,
                insertion_index: 2,
            },
        ];

        rank_articles_by_directness(&mut rows, &filters);

        assert_eq!(rows[0].pmid, "100");
        assert_eq!(
            rows[0]
                .ranking
                .as_ref()
                .map(|ranking| ranking.directness_tier),
            Some(3)
        );
        assert_eq!(
            rows[1]
                .ranking
                .as_ref()
                .map(|ranking| ranking.directness_tier),
            Some(2)
        );
        assert_eq!(
            rows[2]
                .ranking
                .as_ref()
                .map(|ranking| ranking.directness_tier),
            Some(0)
        );
        assert_eq!(
            rows[2]
                .ranking
                .as_ref()
                .map(|ranking| ranking.combined_anchor_hits),
            Some(0)
        );
    }

    #[test]
    fn directness_ranking_prefers_cue_then_citation_then_insertion() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.keyword = Some("melanoma".into());

        let mut rows = vec![
            ArticleSearchResult {
                pmid: "100".into(),
                pmcid: None,
                doi: None,
                title: "BRAF melanoma study".into(),
                journal: Some("Journal A".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(10),
                influential_citation_count: Some(1),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: None,
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "braf melanoma study".into(),
                normalized_abstract: String::new(),
                publication_type: None,
                insertion_index: 0,
            },
            ArticleSearchResult {
                pmid: "200".into(),
                pmcid: None,
                doi: None,
                title: "BRAF melanoma systematic review".into(),
                journal: Some("Journal B".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(5),
                influential_citation_count: Some(0),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: None,
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "braf melanoma systematic review".into(),
                normalized_abstract: String::new(),
                publication_type: Some("Review".into()),
                insertion_index: 1,
            },
            ArticleSearchResult {
                pmid: "300".into(),
                pmcid: None,
                doi: None,
                title: "BRAF melanoma clinical trial review".into(),
                journal: Some("Journal C".into()),
                date: Some("2025-01-01".into()),
                citation_count: Some(50),
                influential_citation_count: Some(7),
                source: ArticleSource::EuropePmc,
                score: None,
                is_retracted: Some(false),
                abstract_snippet: None,
                ranking: None,
                matched_sources: vec![ArticleSource::EuropePmc],
                normalized_title: "braf melanoma clinical trial review".into(),
                normalized_abstract: String::new(),
                publication_type: Some("Clinical Trial".into()),
                insertion_index: 2,
            },
        ];

        rank_articles_by_directness(&mut rows, &filters);

        let pmids: Vec<&str> = rows.iter().map(|row| row.pmid.as_str()).collect();
        assert_eq!(pmids, vec!["300", "200", "100"]);
        assert_eq!(
            rows[0]
                .ranking
                .as_ref()
                .map(|ranking| ranking.study_or_review_cue),
            Some(true)
        );
    }
}
