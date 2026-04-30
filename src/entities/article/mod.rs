//! Article entity models and workflows exposed through the stable article facade.

mod backends;
mod batch;
mod candidates;
mod detail;
mod enrichment;
mod filters;
mod fulltext;
mod graph;
mod planner;
mod query;
mod ranking;
mod search;
#[cfg(test)]
mod test_support;

pub use self::batch::get_batch_compact;
pub use self::detail::get;
pub use self::graph::{citations, recommendations, references};
#[allow(unused_imports)]
pub(crate) use self::planner::{
    ArticleSearchDebugSummary, article_type_limitation_note, litsense2_search_enabled,
    semantic_scholar_search_enabled, summarize_debug_plan,
};
#[allow(unused_imports)]
pub(crate) use self::ranking::{article_effective_ranking_mode, article_relevance_ranking_policy};
pub use self::search::{search, search_page, validate_search_page_request};

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcSort;
use crate::sources::semantic_scholar::SemanticScholarAuthMode;

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
    pub full_text_source: Option<ArticleFulltextSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ArticleAnnotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_scholar: Option<ArticleSemanticScholar>,
    #[serde(default)]
    pub pubtator_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextKind {
    JatsXml,
    Html,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextSource {
    pub kind: ArticleFulltextKind,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArticleGetOptions {
    pub allow_pdf: bool,
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
pub struct ArticleSourceStatus {
    pub source: ArticleSource,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<SemanticScholarAuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArticleSourceAvailability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleSourceAvailability {
    Ok,
    Degraded,
    Unavailable,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct ArticleSearchPage {
    pub results: Vec<ArticleSearchResult>,
    pub total: Option<usize>,
    #[allow(dead_code)]
    pub next_page_token: Option<String>,
    pub source_status: Vec<ArticleSourceStatus>,
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
    pub first_index_date: Option<NaiveDate>,
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
    pub source_local_position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticlePubMedRescueKind {
    Unique,
    Led,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticleRankingMetadata {
    pub directness_tier: u8,
    pub anchor_count: u8,
    pub title_anchor_hits: u8,
    pub abstract_anchor_hits: u8,
    pub combined_anchor_hits: u8,
    pub all_anchors_in_title: bool,
    pub all_anchors_in_text: bool,
    pub study_or_review_cue: bool,
    #[serde(default)]
    pub pubmed_rescue: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubmed_rescue_kind: Option<ArticlePubMedRescueKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pubmed_source_position: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ArticleRankingMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lexical_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citation_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composite_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avg_source_rank: Option<f64>,
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
    PubMed,
    LitSense2,
}

impl ArticleSource {
    pub fn display_name(self) -> &'static str {
        match self {
            Self::PubTator => "PubTator3",
            Self::EuropePmc => "Europe PMC",
            Self::SemanticScholar => "Semantic Scholar",
            Self::PubMed => "PubMed",
            Self::LitSense2 => "LitSense2",
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
    PubMed,
    LitSense2,
}

impl ArticleSourceFilter {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "all" => Ok(Self::All),
            "pubtator" => Ok(Self::PubTator),
            "europepmc" | "europe-pmc" => Ok(Self::EuropePmc),
            "pubmed" => Ok(Self::PubMed),
            "litsense2" => Ok(Self::LitSense2),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected one of: all, pubtator, europepmc, pubmed, litsense2."
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::PubTator => "pubtator",
            Self::EuropePmc => "europepmc",
            Self::PubMed => "pubmed",
            Self::LitSense2 => "litsense2",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleRankingMode {
    Lexical,
    Semantic,
    Hybrid,
}

impl ArticleRankingMode {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "lexical" => Ok(Self::Lexical),
            "semantic" => Ok(Self::Semantic),
            "hybrid" => Ok(Self::Hybrid),
            _ => Err(BioMcpError::InvalidArgument(
                "Invalid article ranking mode. Expected one of: lexical, semantic, hybrid".into(),
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArticleRankingWeights {
    pub semantic: f64,
    pub lexical: f64,
    pub citations: f64,
    pub position: f64,
}

impl Default for ArticleRankingWeights {
    fn default() -> Self {
        Self {
            semantic: 0.4,
            lexical: 0.3,
            citations: 0.2,
            position: 0.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArticleRankingOptions {
    pub requested_mode: Option<ArticleRankingMode>,
    pub weights: ArticleRankingWeights,
    pub weights_overridden: bool,
}

impl ArticleRankingOptions {
    pub fn from_inputs(
        requested_mode: Option<&str>,
        weight_semantic: Option<f64>,
        weight_lexical: Option<f64>,
        weight_citations: Option<f64>,
        weight_position: Option<f64>,
    ) -> Result<Self, BioMcpError> {
        let defaults = ArticleRankingWeights::default();
        Ok(Self {
            requested_mode: requested_mode
                .map(ArticleRankingMode::from_flag)
                .transpose()?,
            weights: ArticleRankingWeights {
                semantic: weight_semantic.unwrap_or(defaults.semantic),
                lexical: weight_lexical.unwrap_or(defaults.lexical),
                citations: weight_citations.unwrap_or(defaults.citations),
                position: weight_position.unwrap_or(defaults.position),
            },
            weights_overridden: weight_semantic.is_some()
                || weight_lexical.is_some()
                || weight_citations.is_some()
                || weight_position.is_some(),
        })
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
    pub max_per_source: Option<usize>,
    pub sort: ArticleSort,
    pub ranking: ArticleRankingOptions,
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
const PUBMED_PAGE_SIZE: usize = 100;
const MAX_PAGE_FETCHES: usize = 50;
const WARN_PAGE_THRESHOLD: usize = 20;
const SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS: usize = 500;
const FEDERATED_PAGE_SIZE_CAP: usize = if EUROPE_PMC_PAGE_SIZE < PUBTATOR_PAGE_SIZE {
    EUROPE_PMC_PAGE_SIZE
} else {
    PUBTATOR_PAGE_SIZE
};
const MAX_FEDERATED_FETCH_RESULTS: usize = MAX_PAGE_FETCHES * FEDERATED_PAGE_SIZE_CAP;
const PUBMED_RESCUE_POSITION_MAX: usize = 0;
const INVALID_ARTICLE_ID_MSG: &str = "\
Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), \
PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., \
e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not \
indexed by PubMed or Europe PMC and cannot be resolved.";
pub const ARTICLE_RELEVANCE_RANKING_POLICY: &str = "calibrated PubMed rescue + lexical directness (top-ranked weak PubMed unique/led rows with at least one anchor hit > title coverage > title+abstract coverage > study/review cue > citation support > source-local position)";
pub const ARTICLE_SEMANTIC_RANKING_POLICY: &str =
    "semantic relevance (semantic score first, lexical directness fallback)";

#[cfg(test)]
mod tests {
    use super::test_support::{empty_filters, row_with};
    use super::*;

    #[test]
    fn article_sort_default_is_relevance() {
        let default: ArticleSort = Default::default();
        assert_eq!(default, ArticleSort::Relevance);
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
    fn article_ranking_mode_parses_supported_values() {
        assert_eq!(
            ArticleRankingMode::from_flag("lexical").expect("lexical should parse"),
            ArticleRankingMode::Lexical
        );
        assert_eq!(
            ArticleRankingMode::from_flag("semantic").expect("semantic should parse"),
            ArticleRankingMode::Semantic
        );
        assert_eq!(
            ArticleRankingMode::from_flag("hybrid").expect("hybrid should parse"),
            ArticleRankingMode::Hybrid
        );
        assert!(ArticleRankingMode::from_flag("auto").is_err());
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
        assert_eq!(
            ArticleSourceFilter::from_flag("pubmed").expect("pubmed should parse"),
            ArticleSourceFilter::PubMed
        );
        assert!(
            ArticleSourceFilter::from_flag("litsense2").is_ok(),
            "litsense2 should parse"
        );
    }

    #[test]
    fn article_source_pubmed_display_name() {
        assert_eq!(ArticleSource::PubMed.display_name(), "PubMed");
    }

    #[test]
    fn article_source_litsense2_display_name() {
        assert_eq!(ArticleSource::LitSense2.display_name(), "LitSense2");
    }

    #[tokio::test]
    async fn search_page_rejects_unknown_article_type_before_backend_planning() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.article_type = Some("invalid".into());

        let err = search_page(&filters, 1, 0, ArticleSourceFilter::PubTator)
            .await
            .expect_err("invalid article type should fail before planner-specific errors");

        assert_eq!(
            err.to_string(),
            "Invalid argument: --type must be one of: review, research, research-article, case-reports, meta-analysis"
        );
    }

    #[tokio::test]
    async fn search_page_rejects_max_per_source_above_limit_before_backend_planning() {
        let mut filters = empty_filters();
        filters.gene = Some("BRAF".into());
        filters.open_access = true;
        filters.max_per_source = Some(11);

        let err = search_page(&filters, 10, 0, ArticleSourceFilter::PubTator)
            .await
            .expect_err("invalid max-per-source should fail before planner-specific errors");

        assert_eq!(
            err.to_string(),
            "Invalid argument: --max-per-source must be <= --limit"
        );
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
}
