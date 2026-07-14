//! Article search orchestration across planner, backends, enrichment, and finalization.

use std::future::Future;
use std::time::Duration;

use tokio::time::timeout;
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;

use super::backends::{
    search_europepmc_page, search_litsense2_candidates, search_pubmed_page, search_pubtator_page,
    search_semantic_scholar_candidates,
};
use super::candidates::validate_article_source_cap;
use super::enrichment::{
    enrich_and_finalize_article_candidates,
    enrich_and_finalize_article_candidates_with_semantic_scholar_status,
    enrich_visible_article_search_page,
};
use super::filters::{
    normalized_date_bounds, validate_required_search_filters, validate_search_filter_values,
};
use super::planner::{
    BackendPlan, litsense2_search_enabled, plan_backends, pubmed_filter_compatible,
};
use super::query::resolve_variant_entity_token;
use super::ranking::validate_article_ranking_options;
use super::{
    ArticleSearchFilters, ArticleSearchPage, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleSourceFilter, ArticleSourceStatus,
    MAX_FEDERATED_FETCH_RESULTS, MAX_SEARCH_LIMIT,
};

pub const VARIANT_ENTITY_RETRIEVAL_PATH: &str = "PubTator variant annotation recall";
pub const VARIANT_FALLBACK_RETRIEVAL_PATH: &str = "best-effort free-text fallback";

const FEDERATED_ARTICLE_SOURCE_TIMEOUT: Duration = Duration::from_secs(12);

pub struct VariantArticleSearchPage {
    pub page: ArticleSearchPage,
    pub retrieval_path: &'static str,
}

pub async fn search(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0, ArticleSourceFilter::All)
        .await?
        .results)
}

pub async fn search_variant_article_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<VariantArticleSearchPage, BioMcpError> {
    let Some(intent) = filters.variant.as_ref() else {
        return Ok(VariantArticleSearchPage {
            page: search_page(filters, limit, offset, ArticleSourceFilter::All).await?,
            retrieval_path: VARIANT_FALLBACK_RETRIEVAL_PATH,
        });
    };

    let pubtator = crate::sources::pubtator::PubTatorClient::new()?;
    if let Some(entity_id) = resolve_variant_entity_token(&pubtator, intent).await {
        let mut entity_filters = filters.clone();
        if let Some(entity_intent) = entity_filters.variant.as_mut() {
            entity_intent.entity_id = Some(entity_id);
        }
        let page = search_page(
            &entity_filters,
            limit,
            offset,
            ArticleSourceFilter::PubTator,
        )
        .await?;
        if !page.results.is_empty() {
            return Ok(VariantArticleSearchPage {
                page,
                retrieval_path: VARIANT_ENTITY_RETRIEVAL_PATH,
            });
        }
    }

    let mut fallback_filters = filters.clone();
    fallback_filters.gene = None;
    fallback_filters.gene_anchored = false;
    fallback_filters.keyword = Some(intent.original.clone());
    fallback_filters.variant = None;
    Ok(VariantArticleSearchPage {
        page: search_page(
            &fallback_filters,
            limit,
            offset,
            ArticleSourceFilter::PubTator,
        )
        .await?,
        retrieval_path: VARIANT_FALLBACK_RETRIEVAL_PATH,
    })
}

fn article_search_page(
    page: SearchPage<ArticleSearchResult>,
    source_status: Vec<ArticleSourceStatus>,
) -> ArticleSearchPage {
    ArticleSearchPage {
        results: page.results,
        total: page.total,
        next_page_token: page.next_page_token,
        source_status,
    }
}

#[derive(Default)]
struct SemanticScholarStatusTracker {
    auth_mode: Option<crate::sources::semantic_scholar::SemanticScholarAuthMode>,
    succeeded: bool,
    failed: bool,
    message: Option<String>,
}

impl SemanticScholarStatusTracker {
    fn record(&mut self, status: ArticleSourceStatus) {
        if self.auth_mode.is_none() {
            self.auth_mode = status.auth_mode;
        }
        match status.status {
            Some(ArticleSourceAvailability::Ok) => self.succeeded = true,
            Some(ArticleSourceAvailability::Degraded) => {
                self.succeeded = true;
                self.failed = true;
            }
            Some(ArticleSourceAvailability::Unavailable) => self.failed = true,
            Some(ArticleSourceAvailability::Skipped) | None => {}
        }
        if status.message.is_some() {
            self.message = status.message;
        }
    }

    fn finish(self) -> Vec<ArticleSourceStatus> {
        let status = if self.failed && self.succeeded {
            ArticleSourceAvailability::Degraded
        } else if self.failed {
            ArticleSourceAvailability::Unavailable
        } else {
            ArticleSourceAvailability::Ok
        };
        vec![ArticleSourceStatus {
            source: ArticleSource::SemanticScholar,
            enabled: true,
            auth_mode: self.auth_mode,
            status: Some(status),
            message: self.failed.then_some(
                self.message
                    .unwrap_or_else(|| "Semantic Scholar unavailable".to_string()),
            ),
        }]
    }
}

struct FederatedArticleRows {
    rows: Vec<ArticleSearchResult>,
    source_status: Vec<ArticleSourceStatus>,
    semantic_scholar_status: ArticleSourceStatus,
}

struct TypeCapableArticleRows {
    rows: Vec<ArticleSearchResult>,
    total: Option<usize>,
    source_status: Vec<ArticleSourceStatus>,
}

enum FederatedSourceOutcome<T> {
    Available(T),
    Unavailable {
        error: Option<BioMcpError>,
        status: ArticleSourceStatus,
    },
}

fn source_degraded_status(source: ArticleSource, message: String) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(message),
    }
}

fn timed_out_source_status(source: ArticleSource) -> ArticleSourceStatus {
    source_degraded_status(
        source,
        format!(
            "{} timed out after {}s",
            source.display_name(),
            FEDERATED_ARTICLE_SOURCE_TIMEOUT.as_secs()
        ),
    )
}

async fn with_federated_source_timeout<T, F>(
    source: ArticleSource,
    future: F,
) -> FederatedSourceOutcome<T>
where
    F: Future<Output = Result<T, BioMcpError>>,
{
    match timeout(FEDERATED_ARTICLE_SOURCE_TIMEOUT, future).await {
        Ok(Ok(value)) => FederatedSourceOutcome::Available(value),
        Ok(Err(err)) => {
            warn!(
                ?err,
                source = source.display_name(),
                "Federated article source failed"
            );
            FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(
                    source,
                    format!("{} search unavailable", source.display_name()),
                ),
            }
        }
        Err(_) => {
            warn!(
                source = source.display_name(),
                "Federated article source timed out"
            );
            FederatedSourceOutcome::Unavailable {
                error: None,
                status: timed_out_source_status(source),
            }
        }
    }
}

fn unavailable_source_error(source: ArticleSource) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: source.display_name().to_string(),
        reason: format!(
            "timed out after {}s during federated article search",
            FEDERATED_ARTICLE_SOURCE_TIMEOUT.as_secs()
        ),
        suggestion: format!(
            "Retry with --source all or use --source {}",
            source.display_name()
        ),
    }
}

async fn search_federated_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }
    let include_pubmed = pubmed_filter_compatible(filters);
    let include_litsense2 = litsense2_search_enabled(filters, ArticleSourceFilter::All);
    let (pubtator_leg, europe_leg, pubmed_leg, semantic_scholar_leg, litsense2_leg) = tokio::join!(
        with_federated_source_timeout(
            ArticleSource::PubTator,
            search_pubtator_page(filters, fetch_count, 0),
        ),
        with_federated_source_timeout(
            ArticleSource::EuropePmc,
            search_europepmc_page(filters, fetch_count, 0),
        ),
        async {
            if include_pubmed {
                Some(
                    with_federated_source_timeout(
                        ArticleSource::PubMed,
                        search_pubmed_page(filters, fetch_count, 0),
                    )
                    .await,
                )
            } else {
                None
            }
        },
        with_federated_source_timeout(
            ArticleSource::SemanticScholar,
            search_semantic_scholar_candidates(filters, fetch_count),
        ),
        async {
            if include_litsense2 {
                with_federated_source_timeout(
                    ArticleSource::LitSense2,
                    search_litsense2_candidates(filters, fetch_count),
                )
                .await
            } else {
                FederatedSourceOutcome::Available(Vec::new())
            }
        }
    );

    let federated = collect_federated_article_rows(
        pubtator_leg,
        europe_leg,
        pubmed_leg,
        semantic_scholar_leg,
        litsense2_leg,
    )?;
    let mut tracker = SemanticScholarStatusTracker::default();
    tracker.record(federated.semantic_scholar_status);
    let (page, enrichment_status) =
        enrich_and_finalize_article_candidates_with_semantic_scholar_status(
            federated.rows,
            limit,
            offset,
            None,
            filters,
        )
        .await;
    if let Some(status) = enrichment_status {
        tracker.record(status);
    }

    let mut source_status = federated.source_status;
    source_status.extend(tracker.finish());

    Ok(article_search_page(page, source_status))
}

#[allow(clippy::too_many_arguments)]
fn collect_federated_article_rows(
    pubtator_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
    europe_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
    pubmed_leg: Option<FederatedSourceOutcome<SearchPage<ArticleSearchResult>>>,
    semantic_scholar_leg: FederatedSourceOutcome<super::backends::SemanticScholarCandidateOutcome>,
    litsense2_leg: FederatedSourceOutcome<Vec<ArticleSearchResult>>,
) -> Result<FederatedArticleRows, BioMcpError> {
    let mut source_status = Vec::new();
    let (semantic_scholar_rows, semantic_scholar_status) = match semantic_scholar_leg {
        FederatedSourceOutcome::Available(outcome) => (outcome.rows, outcome.status),
        FederatedSourceOutcome::Unavailable { status, .. } => (Vec::new(), status),
    };
    let litsense2_rows = match litsense2_leg {
        FederatedSourceOutcome::Available(rows) => rows,
        FederatedSourceOutcome::Unavailable { status, .. } => {
            source_status.push(status);
            Vec::new()
        }
    };
    let pubmed_rows = match pubmed_leg {
        Some(FederatedSourceOutcome::Available(page)) => page.results,
        Some(FederatedSourceOutcome::Unavailable { status, .. }) => {
            source_status.push(status);
            Vec::new()
        }
        None => Vec::new(),
    };

    match (pubtator_leg, europe_leg) {
        (
            FederatedSourceOutcome::Available(pubtator_page),
            FederatedSourceOutcome::Available(europe_page),
        ) => {
            let mut merged = pubtator_page.results;
            merged.extend(europe_page.results);
            merged.extend(pubmed_rows);
            merged.extend(semantic_scholar_rows);
            merged.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows: merged,
                source_status,
                semantic_scholar_status,
            })
        }
        (
            FederatedSourceOutcome::Available(pubtator_page),
            FederatedSourceOutcome::Unavailable { status, .. },
        ) => {
            source_status.push(status);
            let mut rows = pubtator_page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows,
                source_status,
                semantic_scholar_status,
            })
        }
        (
            FederatedSourceOutcome::Unavailable { status, .. },
            FederatedSourceOutcome::Available(europe_page),
        ) => {
            source_status.push(status);
            let mut rows = europe_page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows,
                source_status,
                semantic_scholar_status,
            })
        }
        (
            FederatedSourceOutcome::Unavailable { error, status: _ },
            FederatedSourceOutcome::Unavailable { .. },
        ) => Err(error.unwrap_or_else(|| unavailable_source_error(ArticleSource::PubTator))),
    }
}

fn collect_type_capable_article_rows(
    europe_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
    pubmed_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
) -> Result<TypeCapableArticleRows, BioMcpError> {
    match (europe_leg, pubmed_leg) {
        (
            FederatedSourceOutcome::Available(europe_page),
            FederatedSourceOutcome::Available(pubmed_page),
        ) => {
            let mut rows = europe_page.results;
            rows.extend(pubmed_page.results);
            Ok(TypeCapableArticleRows {
                rows,
                total: None,
                source_status: Vec::new(),
            })
        }
        (
            FederatedSourceOutcome::Available(europe_page),
            FederatedSourceOutcome::Unavailable { status, .. },
        ) => Ok(TypeCapableArticleRows {
            rows: europe_page.results,
            total: europe_page.total,
            source_status: vec![status],
        }),
        (
            FederatedSourceOutcome::Unavailable { status, .. },
            FederatedSourceOutcome::Available(pubmed_page),
        ) => Ok(TypeCapableArticleRows {
            rows: pubmed_page.results,
            total: pubmed_page.total,
            source_status: vec![status],
        }),
        (
            FederatedSourceOutcome::Unavailable {
                error: europe_error,
                ..
            },
            FederatedSourceOutcome::Unavailable {
                error: pubmed_error,
                ..
            },
        ) => Err(europe_error
            .or(pubmed_error)
            .unwrap_or_else(|| unavailable_source_error(ArticleSource::EuropePmc))),
    }
}

async fn search_type_capable_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }
    let (europe_leg, pubmed_leg) = tokio::join!(
        with_federated_source_timeout(
            ArticleSource::EuropePmc,
            search_europepmc_page(filters, fetch_count, 0),
        ),
        with_federated_source_timeout(
            ArticleSource::PubMed,
            search_pubmed_page(filters, fetch_count, 0),
        ),
    );
    let capable = collect_type_capable_article_rows(europe_leg, pubmed_leg)?;
    let page =
        enrich_and_finalize_article_candidates(capable.rows, limit, offset, capable.total, filters)
            .await;
    Ok(article_search_page(page, capable.source_status))
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
            Ok(enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
            )
            .await)
        }
        BackendPlan::PubTatorOnly => {
            let page = search_pubtator_page(filters, fetch_count, 0).await?;
            Ok(enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
            )
            .await)
        }
        BackendPlan::PubMedOnly => {
            let page = search_pubmed_page(filters, fetch_count, 0).await?;
            Ok(enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
            )
            .await)
        }
        BackendPlan::SemanticScholarOnly => {
            let outcome = search_semantic_scholar_candidates(filters, fetch_count).await?;
            Ok(
                enrich_and_finalize_article_candidates(outcome.rows, limit, offset, None, filters)
                    .await,
            )
        }
        BackendPlan::LitSense2Only => {
            let rows = search_litsense2_candidates(filters, fetch_count).await?;
            Ok(enrich_and_finalize_article_candidates(rows, limit, offset, None, filters).await)
        }
        BackendPlan::TypeCapable => {
            unreachable!("type-capable search is handled by search_page")
        }
        BackendPlan::Both => unreachable!("federated relevance is handled by search_page"),
    }
}

async fn search_semantic_scholar_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for Semantic Scholar article search"
        )));
    }

    let outcome = search_semantic_scholar_candidates(filters, fetch_count).await?;
    let page =
        enrich_and_finalize_article_candidates(outcome.rows, limit, offset, None, filters).await;
    Ok(article_search_page(page, vec![outcome.status]))
}

pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    source: ArticleSourceFilter,
) -> Result<ArticleSearchPage, BioMcpError> {
    validate_search_page_request(filters, limit, source)?;
    let plan = plan_backends(filters, source)?;
    if plan == BackendPlan::TypeCapable {
        return search_type_capable_page(filters, limit, offset).await;
    }
    if filters.sort == ArticleSort::Relevance {
        if plan == BackendPlan::Both {
            return search_federated_page(filters, limit, offset).await;
        }
        if plan == BackendPlan::SemanticScholarOnly {
            return search_semantic_scholar_page(filters, limit, offset).await;
        }
        return Ok(article_search_page(
            search_relevance_page(filters, limit, offset, plan).await?,
            Vec::new(),
        ));
    }
    match plan {
        BackendPlan::EuropeOnly => {
            let page = search_europepmc_page(filters, limit, offset).await?;
            Ok(article_search_page(
                enrich_visible_article_search_page(page).await,
                Vec::new(),
            ))
        }
        BackendPlan::PubTatorOnly => {
            let page = search_pubtator_page(filters, limit, offset).await?;
            Ok(article_search_page(
                enrich_visible_article_search_page(page).await,
                Vec::new(),
            ))
        }
        BackendPlan::PubMedOnly | BackendPlan::LitSense2Only => Ok(article_search_page(
            search_relevance_page(filters, limit, offset, plan).await?,
            Vec::new(),
        )),
        BackendPlan::TypeCapable => {
            unreachable!("type-capable search returned before sort dispatch")
        }
        BackendPlan::SemanticScholarOnly => {
            search_semantic_scholar_page(filters, limit, offset).await
        }
        BackendPlan::Both => search_federated_page(filters, limit, offset).await,
    }
}

pub fn validate_search_page_request(
    filters: &ArticleSearchFilters,
    limit: usize,
    source: ArticleSourceFilter,
) -> Result<(), BioMcpError> {
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    validate_article_source_cap(filters, limit)?;
    validate_required_search_filters(filters)?;
    normalized_date_bounds(filters)?;
    validate_search_filter_values(filters)?;
    validate_article_ranking_options(filters)?;
    plan_backends(filters, source)?;
    Ok(())
}

#[cfg(test)]
mod tests;
