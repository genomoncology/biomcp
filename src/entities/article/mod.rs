//! Article entity models and workflows exposed through the stable article facade.

mod assets;
mod backends;
mod batch;
mod candidates;
mod detail;
mod enrichment;
mod filters;
mod fulltext;
mod graph;
mod identity_verification;
mod planner;
mod query;
mod ranking;
mod search;
#[cfg(test)]
mod test_support;
pub(crate) mod variant_search;

pub use self::assets::{article_asset_bytes, article_assets_manifest};
pub use self::batch::get_batch_compact;
pub use self::detail::get;
pub use self::graph::{citations, recommendations, references};
pub(crate) use self::identity_verification::VariantArticleVerificationOptions;
#[allow(unused_imports)]
pub(crate) use self::planner::{
    ArticleSearchDebugSummary, BackendPlan, article_source_plan, article_type_limitation_note,
    litsense2_search_enabled, plan_backends, semantic_scholar_search_enabled, summarize_debug_plan,
};
#[allow(unused_imports)]
pub(crate) use self::ranking::{article_effective_ranking_mode, article_relevance_ranking_policy};
pub use self::search::{search, search_page, validate_search_page_request};
#[allow(unused_imports)]
pub use self::variant_search::{VariantArticleStrategy, search_variant_articles};
#[allow(unused_imports)]
pub(crate) use self::variant_search::{
    parse_variant_article_batch, search_variant_article_batch,
    search_variant_article_batch_with_options, search_variant_articles_with_options,
    search_variant_articles_with_plan,
};

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::entities::section_outcome::SectionOutcomes;
use crate::entities::source_state_registry::outcome_keys;
use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcSort;
use crate::sources::semantic_scholar::SemanticScholarAuthMode;

fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) const ARTICLE_OUTCOME_KEYS: &[&str] = &["fulltext", "indexing", "tldr"];

fn default_article_section_outcomes() -> SectionOutcomes {
    SectionOutcomes::with_keys(&outcome_keys("article"))
}

fn deserialize_article_section_outcomes<'de, D>(
    deserializer: D,
) -> Result<SectionOutcomes, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let outcomes = SectionOutcomes::deserialize(deserializer)?;
    outcomes
        .validate_keys(&outcome_keys("article"))
        .map_err(serde::de::Error::custom)?;
    if outcomes.get("fulltext").is_none() {
        return Err(serde::de::Error::custom(
            "missing section outcome key: fulltext",
        ));
    }
    Ok(outcomes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    #[serde(
        default = "default_article_section_outcomes",
        deserialize_with = "deserialize_article_section_outcomes"
    )]
    pub section_outcomes: SectionOutcomes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    pub author_count: usize,
    pub author_completeness: ArticleAuthorCompleteness,
    pub author_source: ArticleSource,
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
    pub full_text_manifest: Option<ArticleFulltextManifest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text_coverage: Option<ArticleFulltextCoverage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_included: Option<ArticleNotIncluded>,
    #[serde(skip)]
    pub europepmc_license: Option<String>,
    #[serde(skip)]
    pub europepmc_retracted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ArticleAnnotations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexing: Option<ArticleIndexing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_scholar: Option<ArticleSemanticScholar>,
    #[serde(default)]
    pub pubtator_fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleIndexing {
    pub status: ArticleIndexingStatus,
    pub source: ArticleSource,
    pub authors: Vec<ArticleIndexingAuthor>,
    pub mesh_headings: Vec<ArticleMeshHeading>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ArticleIndexingFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleIndexingStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleIndexingFailureCode {
    MissingPmid,
    ClientError,
    NetworkError,
    HttpError,
    RateLimited,
    InvalidResponse,
    ResponseTooLarge,
    ParseError,
    NotFound,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleIndexingFailure {
    pub code: ArticleIndexingFailureCode,
    pub message: String,
}

impl ArticleIndexingFailure {
    pub(super) fn from_code(code: ArticleIndexingFailureCode) -> Self {
        let message = match code {
            ArticleIndexingFailureCode::MissingPmid => {
                "This article has no PMID for PubMed indexing."
            }
            ArticleIndexingFailureCode::ClientError => {
                "PubMed indexing could not initialize its client."
            }
            ArticleIndexingFailureCode::NetworkError => "PubMed indexing could not reach PubMed.",
            ArticleIndexingFailureCode::HttpError => {
                "PubMed returned an unsuccessful response for indexing."
            }
            ArticleIndexingFailureCode::RateLimited => "PubMed indexing was rate limited.",
            ArticleIndexingFailureCode::InvalidResponse => {
                "PubMed returned an invalid indexing response."
            }
            ArticleIndexingFailureCode::ResponseTooLarge => {
                "PubMed indexing response exceeded the size limit."
            }
            ArticleIndexingFailureCode::ParseError => {
                "PubMed indexing response could not be parsed."
            }
            ArticleIndexingFailureCode::NotFound => {
                "PubMed indexing metadata was not found for this article."
            }
            ArticleIndexingFailureCode::Timeout => "PubMed indexing timed out.",
        };
        Self {
            code,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleIndexingAuthor {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
    pub affiliations: Vec<ArticleAffiliation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAffiliation {
    pub text: String,
    pub identifiers: Vec<ArticleAffiliationIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAffiliationIdentifier {
    pub source: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleMeshHeading {
    pub descriptor: ArticleMeshTerm,
    pub qualifiers: Vec<ArticleMeshTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleMeshTerm {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<String>,
    pub major_topic: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextManifestKind {
    JatsXml,
    PmcHtml,
    Pdf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextManifest {
    pub source_kind: ArticleFulltextManifestKind,
    pub provider: ArticleFulltextProvider,
    pub source_identifier: String,
    pub quality: ArticleFulltextQuality,
    pub reuse: ArticleFulltextReuse,
    pub provenance: ArticleFulltextProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArticleFulltextProvider {
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextCoverageKind {
    FullText,
    AbstractOnly,
    MetadataOnly,
    None,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextCoverage {
    pub coverage: ArticleFulltextCoverageKind,
    pub attempts: Vec<ArticleFulltextAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextAttempt {
    pub provider: ArticleFulltextProvider,
    pub source_kind: ArticleFulltextAttemptSourceKind,
    pub coverage: ArticleFulltextAttemptCoverage,
    pub outcome: ArticleFulltextAttemptOutcome,
    pub cache_state: ArticleFulltextCacheState,
    pub reason: ArticleFulltextAttemptReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextAttemptSourceKind {
    JatsXml,
    PmcHtml,
    Pdf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextAttemptCoverage {
    FullText,
    AbstractOnly,
    MetadataOnly,
    None,
    Unusable,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextAttemptOutcome {
    Data,
    Empty,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextCacheState {
    Hit,
    Miss,
    Bypass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleFulltextAttemptReason {
    BodyDetected,
    AbstractWithoutBody,
    MetadataWithoutBody,
    NoContent,
    UnusableContent,
    SourceUnavailable,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextQuality {
    pub has_sections: bool,
    pub has_tables: bool,
    pub has_references: bool,
    pub has_fulltext_signal: bool,
    pub has_entity_annotations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextReuse {
    pub license_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_source: Option<ArticleFulltextProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reuse_warning: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleFulltextProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retracted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_url: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pdf_fallback_used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAssetsManifest {
    pub article_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    pub provider: ArticleFulltextProvider,
    pub provenance: ArticleFulltextProvenance,
    pub assets: Vec<ArticleAssetEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage: Vec<ArticleAssetNamedCoverage>,
    #[serde(skip)]
    pub not_included: Option<ArticleNotIncluded>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAssetEntry {
    pub filename: String,
    pub asset_key: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    pub size_bytes: usize,
    pub sha256: String,
    pub provider: ArticleFulltextProvider,
    pub reuse: ArticleFulltextReuse,
    pub provenance: ArticleFulltextProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jats: Option<ArticleAssetJats>,
    pub discovery_routes: Vec<ArticleAssetDiscoveryRoute>,
    pub handle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ArticleAssetDiscoveryRoute {
    pub provider: ArticleFulltextProvider,
    pub source_document: ArticleAssetSourceDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleAssetSourceDocument {
    PmcOaArchive,
    EuropePmcZip,
    Figshare,
    JatsXml,
    PmcHtml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleAssetNamedOutcome {
    Retrievable,
    HealthyAbsent,
    AccessOrLicenceDenied,
    PmcProofOfWork,
    UnsupportedOrigin,
    SourceUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAssetNamedCoverage {
    pub filename: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    pub provider: ArticleFulltextProvider,
    pub source_document: ArticleAssetSourceDocument,
    pub outcome: ArticleAssetNamedOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    pub discovery_routes: Vec<ArticleAssetDiscoveryRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAssetJats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleNotIncluded {
    pub figure_images: ArticleAssetCoverage,
    pub supplementary_files: ArticleAssetCoverage,
    pub complex_tables: ArticleOmittedCoverage,
    #[serde(skip)]
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleAssetCoverage {
    pub count: usize,
    pub retrieve_with: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleOmittedCoverage {
    pub count: usize,
    pub retrieve_with: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArticleGetOptions {
    pub allow_pdf: bool,
    pub include_asset_summary: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSourcePlan {
    pub candidate_sources: Vec<ArticleSource>,
    pub enrichment_sources: Vec<ArticleSource>,
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
    // dead-code reason: mod::next_page_token is exercised by native tests or binary dispatch
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleAuthorCompleteness {
    Complete,
    SourceLimited,
    Unavailable,
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
    #[serde(default)]
    pub authors: Vec<String>,
    pub author_count: usize,
    pub author_completeness: ArticleAuthorCompleteness,
    pub author_source: ArticleSource,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_scholar_id: Option<String>,
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
    #[serde(default)]
    pub edges: Vec<ArticleGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleRecommendationsResult {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub positive_seeds: Vec<ArticleRelatedPaper>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub negative_seeds: Vec<ArticleRelatedPaper>,
    #[serde(default)]
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
    SemanticScholar,
    LitSense2,
}

impl ArticleSourceFilter {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "all" => Ok(Self::All),
            "pubtator" => Ok(Self::PubTator),
            "europepmc" | "europe-pmc" => Ok(Self::EuropePmc),
            "pubmed" => Ok(Self::PubMed),
            "semanticscholar" => Ok(Self::SemanticScholar),
            "litsense2" => Ok(Self::LitSense2),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected one of: all, pubtator, europepmc, pubmed, semanticscholar, litsense2."
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::PubTator => "pubtator",
            Self::EuropePmc => "europepmc",
            Self::PubMed => "pubmed",
            Self::SemanticScholar => "semanticscholar",
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
pub struct ArticleVariantIntent {
    pub original: String,
    pub gene: Option<String>,
    pub change: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ArticleSearchFilters {
    pub gene: Option<String>,
    pub gene_anchored: bool,
    pub disease: Option<String>,
    pub drug: Option<String>,
    pub variant: Option<ArticleVariantIntent>,
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
const ARTICLE_SECTION_INDEXING: &str = "indexing";
const ARTICLE_SECTION_ASSETS: &str = "assets";
const ARTICLE_SECTION_ASSET: &str = "asset";
const ARTICLE_SECTION_ALL: &str = "all";

pub const ARTICLE_SECTION_NAMES: &[&str] = &[
    ARTICLE_SECTION_ANNOTATIONS,
    ARTICLE_SECTION_FULLTEXT,
    ARTICLE_SECTION_TLDR,
    ARTICLE_SECTION_INDEXING,
    ARTICLE_SECTION_ASSETS,
    ARTICLE_SECTION_ASSET,
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

    fn related_paper(title: &str) -> ArticleRelatedPaper {
        ArticleRelatedPaper {
            paper_id: Some("paper-1".to_string()),
            pmid: Some("1".to_string()),
            doi: None,
            arxiv_id: None,
            title: title.to_string(),
            journal: None,
            year: Some(2026),
        }
    }

    #[test]
    fn graph_and_recommendation_primary_collections_serialize_when_empty() {
        let graph = ArticleGraphResult {
            article: related_paper("Anchor"),
            edges: Vec::new(),
        };
        let recommendations = ArticleRecommendationsResult {
            positive_seeds: Vec::new(),
            negative_seeds: Vec::new(),
            recommendations: Vec::new(),
        };

        let graph_json = serde_json::to_value(graph).expect("graph JSON");
        let recommendations_json =
            serde_json::to_value(recommendations).expect("recommendations JSON");
        assert_eq!(graph_json["edges"], serde_json::json!([]));
        assert_eq!(
            recommendations_json["recommendations"],
            serde_json::json!([])
        );
        assert!(recommendations_json.get("positive_seeds").is_none());
        assert!(recommendations_json.get("negative_seeds").is_none());
    }

    #[test]
    fn graph_and_recommendation_nonempty_shapes_remain_compatible() {
        let paper = related_paper("Related");
        let graph = ArticleGraphResult {
            article: related_paper("Anchor"),
            edges: vec![ArticleGraphEdge {
                paper: paper.clone(),
                intents: vec!["background".to_string()],
                contexts: Vec::new(),
                is_influential: false,
            }],
        };
        let recommendations = ArticleRecommendationsResult {
            positive_seeds: vec![related_paper("Anchor")],
            negative_seeds: Vec::new(),
            recommendations: vec![paper],
        };

        let graph_json = serde_json::to_value(graph).expect("graph JSON");
        let recommendations_json =
            serde_json::to_value(recommendations).expect("recommendations JSON");
        assert_eq!(graph_json["edges"][0]["paper"]["title"], "Related");
        assert_eq!(
            recommendations_json["recommendations"][0]["title"],
            "Related"
        );
        assert_eq!(recommendations_json["positive_seeds"][0]["title"], "Anchor");
    }

    #[test]
    fn requested_fulltext_coverage_is_additive_and_uses_closed_values() {
        let base = serde_json::json!({
            "title": "Fixture",
            "author_count": 0,
            "author_completeness": "unavailable",
            "author_source": "pubtator"
        });
        let mut article: Article = serde_json::from_value(base).expect("compatible article");
        assert!(
            serde_json::to_value(&article)
                .expect("base JSON")
                .get("full_text_coverage")
                .is_none()
        );

        article.full_text_coverage = Some(ArticleFulltextCoverage {
            coverage: ArticleFulltextCoverageKind::AbstractOnly,
            attempts: vec![ArticleFulltextAttempt {
                provider: ArticleFulltextProvider {
                    label: "Europe PMC XML".into(),
                    source: "Europe PMC".into(),
                },
                source_kind: ArticleFulltextAttemptSourceKind::JatsXml,
                coverage: ArticleFulltextAttemptCoverage::AbstractOnly,
                outcome: ArticleFulltextAttemptOutcome::Empty,
                cache_state: ArticleFulltextCacheState::Bypass,
                reason: ArticleFulltextAttemptReason::AbstractWithoutBody,
            }],
        });
        let value = serde_json::to_value(article).expect("requested JSON");
        assert_eq!(value["full_text_coverage"]["coverage"], "abstract_only");
        assert_eq!(
            value["full_text_coverage"]["attempts"][0],
            serde_json::json!({
                "provider": {"label": "Europe PMC XML", "source": "Europe PMC"},
                "source_kind": "jats_xml",
                "coverage": "abstract_only",
                "outcome": "empty",
                "cache_state": "bypass",
                "reason": "abstract_without_body"
            })
        );
    }

    #[test]
    fn article_section_outcomes_default_to_keyed_not_requested_and_reject_foreign_keys() {
        let base = serde_json::json!({
            "title": "Fixture",
            "author_count": 0,
            "author_completeness": "unavailable",
            "author_source": "pubtator"
        });
        let article: Article = serde_json::from_value(base.clone()).expect("compatible article");
        assert_eq!(
            article
                .section_outcomes
                .get("fulltext")
                .expect("fulltext key")
                .outcome(),
            crate::entities::section_outcome::SectionOutcomeState::NotRequested
        );

        let mut foreign = base.clone();
        foreign["section_outcomes"] = serde_json::json!({
            "foreign": {"outcome": "empty", "sources": ["Provider"]}
        });
        assert!(serde_json::from_value::<Article>(foreign).is_err());

        let mut empty = base;
        empty["section_outcomes"] = serde_json::json!({});
        assert!(serde_json::from_value::<Article>(empty).is_err());
    }

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
        assert_eq!(
            ArticleSourceFilter::from_flag("semanticscholar")
                .expect("semanticscholar should parse"),
            ArticleSourceFilter::SemanticScholar
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
