//! Article search orchestration across planner, backends, enrichment, and finalization.

use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;

use super::backends::{
    search_europepmc_page, search_litsense2_candidates, search_pubmed_page, search_pubtator_page,
    search_semantic_scholar_candidates,
};
use super::candidates::{finalize_article_candidates, validate_article_source_cap};
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
use super::ranking::validate_article_ranking_options;
use super::{
    ArticleSearchFilters, ArticleSearchPage, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleSourceFilter, ArticleSourceStatus,
    MAX_FEDERATED_FETCH_RESULTS, MAX_SEARCH_LIMIT,
};

pub async fn search(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0, ArticleSourceFilter::All)
        .await?
        .results)
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
    semantic_scholar_status: ArticleSourceStatus,
}

fn semantic_scholar_unavailable_status(message: &str) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Unavailable),
        message: Some(message.to_string()),
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
        search_pubtator_page(filters, fetch_count, 0),
        search_europepmc_page(filters, fetch_count, 0),
        async {
            if include_pubmed {
                Some(search_pubmed_page(filters, fetch_count, 0).await)
            } else {
                None
            }
        },
        search_semantic_scholar_candidates(filters, fetch_count),
        async {
            if include_litsense2 {
                search_litsense2_candidates(filters, fetch_count).await
            } else {
                Ok(Vec::new())
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

    Ok(article_search_page(page, tracker.finish()))
}

#[allow(clippy::too_many_arguments)]
fn collect_federated_article_rows(
    pubtator_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    europe_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    pubmed_leg: Option<Result<SearchPage<ArticleSearchResult>, BioMcpError>>,
    semantic_scholar_leg: Result<super::backends::SemanticScholarCandidateOutcome, BioMcpError>,
    litsense2_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
) -> Result<FederatedArticleRows, BioMcpError> {
    let (semantic_scholar_rows, semantic_scholar_status) = match semantic_scholar_leg {
        Ok(outcome) => (outcome.rows, outcome.status),
        Err(err) => {
            warn!(
                ?err,
                "Semantic Scholar search leg failed; continuing without it"
            );
            (
                Vec::new(),
                semantic_scholar_unavailable_status("Semantic Scholar search unavailable"),
            )
        }
    };
    let litsense2_rows = match litsense2_leg {
        Ok(rows) => rows,
        Err(err) => {
            warn!(?err, "LitSense2 search leg failed; continuing without it");
            Vec::new()
        }
    };
    let pubmed_rows = match pubmed_leg {
        Some(Ok(page)) => page.results,
        Some(Err(err)) => {
            warn!(?err, "PubMed search leg failed; continuing without it");
            Vec::new()
        }
        None => Vec::new(),
    };

    match (pubtator_leg, europe_leg) {
        (Ok(pubtator_page), Ok(europe_page)) => {
            let mut merged = pubtator_page.results;
            merged.extend(europe_page.results);
            merged.extend(pubmed_rows);
            merged.extend(semantic_scholar_rows);
            merged.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows: merged,
                semantic_scholar_status,
            })
        }
        (Ok(pubtator_page), Err(err)) => {
            warn!(
                ?err,
                "Europe PMC search leg failed; returning PubTator-only results"
            );
            let mut rows = pubtator_page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows,
                semantic_scholar_status,
            })
        }
        (Err(err), Ok(europe_page)) => {
            warn!(
                ?err,
                "PubTator search leg failed; returning Europe PMC-only results"
            );
            let mut rows = europe_page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows,
                semantic_scholar_status,
            })
        }
        (Err(pubtator_err), Err(europe_err)) => {
            warn!(?europe_err, "Europe PMC leg also failed");
            Err(pubtator_err)
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[allow(clippy::too_many_arguments)]
pub(super) fn merge_federated_pages(
    pubtator_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    europe_leg: Result<SearchPage<ArticleSearchResult>, BioMcpError>,
    pubmed_leg: Option<Result<SearchPage<ArticleSearchResult>, BioMcpError>>,
    semantic_scholar_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
    litsense2_leg: Result<Vec<ArticleSearchResult>, BioMcpError>,
    limit: usize,
    offset: usize,
    filters: &ArticleSearchFilters,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let semantic_scholar_leg = semantic_scholar_leg.map(|rows| {
        super::backends::SemanticScholarCandidateOutcome {
            rows,
            status: ArticleSourceStatus {
                source: ArticleSource::SemanticScholar,
                enabled: true,
                auth_mode: None,
                status: Some(ArticleSourceAvailability::Ok),
                message: None,
            },
        }
    });
    let rows = collect_federated_article_rows(
        pubtator_leg,
        europe_leg,
        pubmed_leg,
        semantic_scholar_leg,
        litsense2_leg,
    )?
    .rows;
    Ok(finalize_article_candidates(rows, limit, offset, None, filters))
}

async fn search_type_capable_page(
    filters: &ArticleSearchFilters,
    fetch_count: usize,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let (europe_leg, pubmed_leg) = tokio::join!(
        search_europepmc_page(filters, fetch_count, 0),
        search_pubmed_page(filters, fetch_count, 0),
    );

    match (europe_leg, pubmed_leg) {
        (Ok(europe_page), Ok(pubmed_page)) => {
            let mut rows = europe_page.results;
            rows.extend(pubmed_page.results);
            Ok(enrich_and_finalize_article_candidates(rows, limit, offset, None, filters).await)
        }
        (Ok(europe_page), Err(err)) => {
            warn!(
                ?err,
                "PubMed type-capable leg failed; returning Europe PMC-only results"
            );
            Ok(enrich_and_finalize_article_candidates(
                europe_page.results,
                limit,
                offset,
                europe_page.total,
                filters,
            )
            .await)
        }
        (Err(err), Ok(pubmed_page)) => {
            warn!(
                ?err,
                "Europe PMC type-capable leg failed; returning PubMed-only results"
            );
            Ok(enrich_and_finalize_article_candidates(
                pubmed_page.results,
                limit,
                offset,
                pubmed_page.total,
                filters,
            )
            .await)
        }
        (Err(europe_err), Err(pubmed_err)) => {
            warn!(?pubmed_err, "PubMed type-capable leg also failed");
            Err(europe_err)
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
        BackendPlan::LitSense2Only => {
            let rows = search_litsense2_candidates(filters, fetch_count).await?;
            Ok(enrich_and_finalize_article_candidates(rows, limit, offset, None, filters).await)
        }
        BackendPlan::TypeCapable => {
            search_type_capable_page(filters, fetch_count, limit, offset).await
        }
        BackendPlan::Both => unreachable!("federated relevance is handled by search_page"),
    }
}

pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    source: ArticleSourceFilter,
) -> Result<ArticleSearchPage, BioMcpError> {
    validate_search_page_request(filters, limit, source)?;
    let plan = plan_backends(filters, source)?;
    if filters.sort == ArticleSort::Relevance {
        if plan == BackendPlan::Both {
            return search_federated_page(filters, limit, offset).await;
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
        BackendPlan::PubMedOnly | BackendPlan::LitSense2Only | BackendPlan::TypeCapable => Ok(
            article_search_page(
                search_relevance_page(filters, limit, offset, plan).await?,
                Vec::new(),
            ),
        ),
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
