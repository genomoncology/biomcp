//! Shared article test helpers used by sidecar test modules.

#[allow(unused_imports)]
pub(super) use super::{
    ARTICLE_BATCH_MAX_IDS, AnnotationCount, Article, ArticleAnnotations, ArticleAuthorCompleteness,
    ArticleBatchEntitySummary, ArticleBatchItem, ArticlePubMedRescueKind, ArticleRankingMode,
    ArticleRankingOptions, ArticleSearchFilters, ArticleSearchResult, ArticleSemanticScholar,
    ArticleSemanticScholarPdf, ArticleSort, ArticleSource, ArticleSourceFilter,
};
#[allow(unused_imports)]
pub(super) use crate::entities::SearchPage;
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::europepmc::EuropePmcSort;

pub(super) fn empty_filters() -> ArticleSearchFilters {
    ArticleSearchFilters {
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
        no_preprints: false,
        exclude_retracted: false,
        max_per_source: None,
        sort: ArticleSort::Relevance,
        ranking: ArticleRankingOptions::default(),
    }
}

pub(super) fn row(pmid: &str, source: ArticleSource) -> ArticleSearchResult {
    row_with(pmid, source, Some("2025-01-01"), Some(1), Some(false))
}

pub(super) fn row_with(
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
        first_index_date: None,
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
        source_local_position: 0,
    }
}
