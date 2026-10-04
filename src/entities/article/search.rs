//! Article search orchestration across planner, backends, enrichment, and finalization.

use std::future::Future;
use std::time::Duration;

use tokio::time::timeout;
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;

use super::backends::{
    search_europepmc_page, search_europepmc_page_with_context, search_litsense2_candidates,
    search_pubmed_page, search_pubmed_page_with_context, search_pubtator_page,
    search_pubtator_page_with_context, search_semantic_scholar_candidates,
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
use super::ranking::validate_article_ranking_options;
use super::{
    ArticleSearchDiagnostics, ArticleSearchFilters, ArticleSearchPage, ArticleSearchResult,
    ArticleSearchTiming, ArticleSort, ArticleSource, ArticleSourceAvailability,
    ArticleSourceFilter, ArticleSourceStatus, MAX_FEDERATED_FETCH_RESULTS, MAX_SEARCH_LIMIT,
};

pub const VARIANT_ENTITY_RETRIEVAL_PATH: &str = "PubTator variant annotation recall";
pub const VARIANT_FALLBACK_RETRIEVAL_PATH: &str = "best-effort free-text fallback";

const FEDERATED_ARTICLE_SOURCE_TIMEOUT: Duration = Duration::from_secs(12);

mod deadline;
pub(super) use deadline::{article_search_deadline_error, is_search_deadline_error};
use deadline::{
    article_search_deadline_budget, timed_source_call, timed_source_leg,
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
    timings: Vec<ArticleSearchTiming>,
) -> ArticleSearchPage {
    ArticleSearchPage {
        results: page.results,
        total: page.total,
        next_page_token: page.next_page_token,
        source_status,
        diagnostics: ArticleSearchDiagnostics {
            deadline_ms: crate::sources::current_variant_article_deadline()
                .map(|deadline| deadline.limit().as_millis() as u64)
                .unwrap_or_else(|| article_search_deadline_budget().as_millis() as u64),
            source_timings: timings,
        },
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

pub(super) struct FederatedArticleRows {
    pub(super) rows: Vec<ArticleSearchResult>,
    pub(super) source_status: Vec<ArticleSourceStatus>,
    pub(super) semantic_scholar_status: ArticleSourceStatus,
    pub(super) truncated_sources: Vec<ArticleSource>,
    pub(super) primary_error: Option<BioMcpError>,
    pub(super) timings: Vec<ArticleSearchTiming>,
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

fn source_provider(source: ArticleSource) -> crate::error::SourceProvider {
    match source {
        ArticleSource::PubTator => crate::error::SourceProvider::PUBTATOR3,
        ArticleSource::EuropePmc => crate::error::SourceProvider::EUROPE_PMC,
        ArticleSource::PubMed => crate::error::SourceProvider::PUBMED,
        ArticleSource::SemanticScholar => crate::error::SourceProvider::SEMANTIC_SCHOLAR,
        ArticleSource::LitSense2 => crate::error::SourceProvider::LITSENSE2,
    }
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
            crate::error::warn_external_failure(
                &err,
                source_provider(source),
                "federated article search",
            );
            // When the overall search deadline cancelled the send, say so
            // instead of reporting an ordinary provider failure.
            let message = if crate::sources::current_variant_article_deadline()
                .is_some_and(|deadline| deadline.is_exhausted())
            {
                format!(
                    "{} did not answer before the article search deadline",
                    source.display_name()
                )
            } else {
                format!("{} search unavailable", source.display_name())
            };
            FederatedSourceOutcome::Unavailable {
                error: Some(err),
                status: source_degraded_status(source, message),
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

fn variant_budget_source_name(source: ArticleSource) -> &'static str {
    match source {
        ArticleSource::PubTator => "pubtator",
        ArticleSource::EuropePmc => "europepmc",
        ArticleSource::SemanticScholar => "semanticscholar",
        ArticleSource::PubMed => "pubmed",
        ArticleSource::LitSense2 => "litsense2",
    }
}

async fn with_variant_article_budget<T, F>(
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
    source: ArticleSource,
    future: F,
) -> FederatedSourceOutcome<T>
where
    F: Future<Output = Result<T, BioMcpError>>,
{
    let Some(execution) = execution else {
        return with_federated_source_timeout(source, future).await;
    };
    let Some(started) = execution.reserve(route) else {
        execution.record_not_attempted(route, variant_budget_source_name(source));
        return FederatedSourceOutcome::Unavailable {
            error: None,
            status: source_degraded_status(source, "variant article work budget exhausted".into()),
        };
    };
    let outcome = with_federated_source_timeout(source, future).await;
    match &outcome {
        FederatedSourceOutcome::Available(_) => {
            execution.record(route, variant_budget_source_name(source), started, "ok", 1)
        }
        FederatedSourceOutcome::Unavailable {
            error: Some(error), ..
        } => execution.record_error(route, variant_budget_source_name(source), started, error),
        FederatedSourceOutcome::Unavailable { error: None, .. } => execution.record(
            route,
            variant_budget_source_name(source),
            started,
            "unavailable",
            0,
        ),
    }
    outcome
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

fn page_outcome_truncated<T>(
    outcome: &FederatedSourceOutcome<SearchPage<T>>,
    fetch_count: usize,
) -> bool {
    matches!(
        outcome,
        FederatedSourceOutcome::Available(page)
            if page.total.is_some_and(|total| total > page.results.len())
                || (page.total.is_none() && page.results.len() >= fetch_count)
    )
}

pub(super) async fn acquire_federated_article_rows(
    filters: &ArticleSearchFilters,
    fetch_count: usize,
) -> Result<FederatedArticleRows, BioMcpError> {
    acquire_federated_article_rows_with_context(filters, fetch_count, None, "federated").await
}

pub(super) async fn acquire_federated_article_rows_with_context(
    filters: &ArticleSearchFilters,
    fetch_count: usize,
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
) -> Result<FederatedArticleRows, BioMcpError> {
    if fetch_count == 0 || fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "federated article acquisition size must be between 1 and {MAX_FEDERATED_FETCH_RESULTS}"
        )));
    }
    let include_pubmed = pubmed_filter_compatible(filters);
    let include_litsense2 = litsense2_search_enabled(filters, ArticleSourceFilter::All);
    // Each leg is boxed so callers keep a small state machine: the joined
    // futures carry every provider's request stack inline and previously
    // overflowed the test-thread stack once timing fields joined them.
    let (pubtator_leg, europe_leg, pubmed_leg, semantic_scholar_leg, litsense2_leg) = tokio::join!(
        Box::pin(timed_source_leg(
            ArticleSource::PubTator,
            "search",
            search_pubtator_page_with_context(filters, fetch_count, 0, execution, route, None),
        )),
        Box::pin(timed_source_leg(
            ArticleSource::EuropePmc,
            "search",
            search_europepmc_page_with_context(filters, fetch_count, 0, execution, route, None),
        )),
        Box::pin(async {
            if include_pubmed {
                Some(
                    timed_source_leg(
                        ArticleSource::PubMed,
                        "search",
                        search_pubmed_page_with_context(
                            filters,
                            fetch_count,
                            0,
                            execution,
                            route,
                            None,
                        ),
                    )
                    .await,
                )
            } else {
                None
            }
        }),
        Box::pin(timed_source_leg(
            ArticleSource::SemanticScholar,
            "search",
            search_semantic_scholar_candidates(filters, fetch_count, execution, route, None),
        )),
        Box::pin(async {
            if include_litsense2 {
                let started = std::time::Instant::now();
                let outcome = with_variant_article_budget(
                    execution,
                    route,
                    ArticleSource::LitSense2,
                    search_litsense2_candidates(filters, fetch_count),
                )
                .await;
                Some((
                    outcome,
                    ArticleSearchTiming {
                        source: Some(ArticleSource::LitSense2),
                        stage: "search",
                        elapsed_ms: started.elapsed().as_millis() as u64,
                    },
                ))
            } else {
                None
            }
        })
    );

    let (pubtator_leg, pubtator_timing) = pubtator_leg;
    let (europe_leg, europe_timing) = europe_leg;
    let (semantic_scholar_leg, semantic_scholar_timing) = semantic_scholar_leg;
    let mut timings = vec![pubtator_timing, europe_timing, semantic_scholar_timing];
    let pubmed_leg = pubmed_leg.map(|(outcome, timing)| {
        timings.push(timing);
        outcome
    });
    let litsense2_leg = litsense2_leg.map(|(outcome, timing)| {
        timings.push(timing);
        outcome
    });

    let mut truncated_sources = Vec::new();
    if page_outcome_truncated(&pubtator_leg, fetch_count) {
        truncated_sources.push(ArticleSource::PubTator);
    }
    if page_outcome_truncated(&europe_leg, fetch_count) {
        truncated_sources.push(ArticleSource::EuropePmc);
    }
    if pubmed_leg
        .as_ref()
        .is_some_and(|outcome| page_outcome_truncated(outcome, fetch_count))
    {
        truncated_sources.push(ArticleSource::PubMed);
    }
    if matches!(
        &semantic_scholar_leg,
        FederatedSourceOutcome::Available(outcome) if outcome.rows.len() >= fetch_count
    ) {
        truncated_sources.push(ArticleSource::SemanticScholar);
    }
    if litsense2_leg
        .as_ref()
        .is_some_and(|outcome| matches!(outcome, FederatedSourceOutcome::Available(rows) if rows.len() >= fetch_count))
    {
        truncated_sources.push(ArticleSource::LitSense2);
    }
    let mut federated = collect_federated_article_rows(
        pubtator_leg,
        europe_leg,
        pubmed_leg,
        semantic_scholar_leg,
        litsense2_leg.unwrap_or_else(|| FederatedSourceOutcome::Available(Vec::new())),
    )?;
    federated.truncated_sources = truncated_sources;
    federated.timings = timings;
    Ok(federated)
}

pub(super) async fn search_federated_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    enrichment_sources: &[ArticleSource],
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }
    let federated = acquire_federated_article_rows(filters, fetch_count).await?;
    // Both primaries failing is fatal only when nothing else answered: any
    // PubMed/Semantic Scholar/LitSense2 rows that did arrive are returned as
    // a partial page (ticket 1293).
    if federated.rows.is_empty()
        && let Some(error) = federated.primary_error
    {
        return Err(error);
    }
    let mut timings = federated.timings;
    let mut tracker = SemanticScholarStatusTracker::default();
    tracker.record(federated.semantic_scholar_status);
    let enrichment = enrich_and_finalize_article_candidates_with_semantic_scholar_status(
        federated.rows,
        limit,
        offset,
        None,
        filters,
        enrichment_sources,
    )
    .await;
    timings.extend(enrichment.timings);
    if let Some(status) = enrichment.semantic_scholar_status {
        tracker.record(status);
    }

    let mut source_status = federated.source_status;
    source_status.extend(tracker.finish());
    source_status.extend(enrichment.statuses);

    Ok(article_search_page(enrichment.page, source_status, timings))
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
    for (source, leg) in [
        (ArticleSource::PubTator, Some(&pubtator_leg)),
        (ArticleSource::EuropePmc, Some(&europe_leg)),
        (ArticleSource::PubMed, pubmed_leg.as_ref()),
    ] {
        if let Some(FederatedSourceOutcome::Available(page)) = leg
            && let Some(note) = &page.partial_note
        {
            source_status.push(source_degraded_status(source, note.clone()));
        }
    }
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
                truncated_sources: Vec::new(),
                primary_error: None,
                timings: Vec::new(),
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
                truncated_sources: Vec::new(),
                primary_error: None,
                timings: Vec::new(),
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
                truncated_sources: Vec::new(),
                primary_error: None,
                timings: Vec::new(),
            })
        }
        (
            FederatedSourceOutcome::Unavailable {
                error,
                status: pubtator_status,
            },
            FederatedSourceOutcome::Unavailable {
                status: europe_status,
                ..
            },
        ) => {
            source_status.extend([pubtator_status, europe_status]);
            let mut rows = pubmed_rows;
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            Ok(FederatedArticleRows {
                rows,
                source_status,
                semantic_scholar_status,
                truncated_sources: Vec::new(),
                timings: Vec::new(),
                primary_error: Some(
                    error.unwrap_or_else(|| unavailable_source_error(ArticleSource::PubTator)),
                ),
            })
        }
    }
}

fn collect_type_capable_article_rows(
    europe_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
    pubmed_leg: FederatedSourceOutcome<SearchPage<ArticleSearchResult>>,
) -> Result<TypeCapableArticleRows, BioMcpError> {
    let mut partial_status = Vec::new();
    for (source, leg) in [
        (ArticleSource::EuropePmc, &europe_leg),
        (ArticleSource::PubMed, &pubmed_leg),
    ] {
        if let FederatedSourceOutcome::Available(page) = leg
            && let Some(note) = &page.partial_note
        {
            partial_status.push(source_degraded_status(source, note.clone()));
        }
    }
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
                source_status: partial_status,
            })
        }
        (
            FederatedSourceOutcome::Available(europe_page),
            FederatedSourceOutcome::Unavailable { status, .. },
        ) => Ok(TypeCapableArticleRows {
            rows: europe_page.results,
            total: europe_page.total,
            source_status: {
                partial_status.push(status);
                partial_status
            },
        }),
        (
            FederatedSourceOutcome::Unavailable { status, .. },
            FederatedSourceOutcome::Available(pubmed_page),
        ) => Ok(TypeCapableArticleRows {
            rows: pubmed_page.results,
            total: pubmed_page.total,
            source_status: {
                partial_status.push(status);
                partial_status
            },
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
    enrichment_sources: &[ArticleSource],
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }
    let (europe_leg, pubmed_leg) = tokio::join!(
        Box::pin(timed_source_leg(
            ArticleSource::EuropePmc,
            "search",
            search_europepmc_page(filters, fetch_count, 0),
        )),
        Box::pin(timed_source_leg(
            ArticleSource::PubMed,
            "search",
            search_pubmed_page(filters, fetch_count, 0),
        )),
    );
    let (europe_leg, europe_timing) = europe_leg;
    let (pubmed_leg, pubmed_timing) = pubmed_leg;
    let mut timings = vec![europe_timing, pubmed_timing];
    let capable = collect_type_capable_article_rows(europe_leg, pubmed_leg)?;
    let enrichment = enrich_and_finalize_article_candidates(
        capable.rows,
        limit,
        offset,
        capable.total,
        filters,
        enrichment_sources,
    )
    .await;
    timings.extend(enrichment.timings);
    let mut source_status = capable.source_status;
    if let Some(status) = enrichment.semantic_scholar_status {
        source_status.push(status);
    }
    source_status.extend(enrichment.statuses);
    Ok(article_search_page(enrichment.page, source_status, timings))
}

async fn search_relevance_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    plan: BackendPlan,
    enrichment_sources: &[ArticleSource],
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for federated article search"
        )));
    }

    // Single-backend plans carry no per-source timeout, so the invocation
    // deadline is the only wall-clock bound on these legs; each is still
    // timed for --full diagnostics.
    match plan {
        BackendPlan::EuropeOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::EuropePmc,
                "search",
                search_europepmc_page(filters, fetch_count, 0),
            )
            .await;
            let page = result?;
            let status = page.partial_note.as_ref()
                .map(|note| source_degraded_status(ArticleSource::EuropePmc, note.clone()));
            let enrichment = enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(enrichment, timing, status))
        }
        BackendPlan::PubTatorOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubTator,
                "search",
                search_pubtator_page(filters, fetch_count, 0),
            )
            .await;
            let page = result?;
            let status = page.partial_note.as_ref()
                .map(|note| source_degraded_status(ArticleSource::PubTator, note.clone()));
            let enrichment = enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(enrichment, timing, status))
        }
        BackendPlan::PubMedOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubMed,
                "search",
                search_pubmed_page(filters, fetch_count, 0),
            )
            .await;
            let page = result?;
            let status = page.partial_note.as_ref()
                .map(|note| source_degraded_status(ArticleSource::PubMed, note.clone()));
            let enrichment = enrich_and_finalize_article_candidates(
                page.results,
                limit,
                offset,
                page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(enrichment, timing, status))
        }
        BackendPlan::SemanticScholarOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::SemanticScholar,
                "search",
                search_semantic_scholar_candidates(filters, fetch_count, None, "federated", None),
            )
            .await;
            let outcome = result?;
            let enrichment = enrich_and_finalize_article_candidates(
                outcome.rows,
                limit,
                offset,
                None,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(enrichment, timing, None))
        }
        BackendPlan::LitSense2Only => {
            let (result, timing) = timed_source_call(
                ArticleSource::LitSense2,
                "search",
                search_litsense2_candidates(filters, fetch_count),
            )
            .await;
            let rows = result?;
            let enrichment = enrich_and_finalize_article_candidates(
                rows,
                limit,
                offset,
                None,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(enrichment, timing, None))
        }
        BackendPlan::TypeCapable => {
            unreachable!("type-capable search is handled by search_page")
        }
        BackendPlan::Both => unreachable!("federated relevance is handled by search_page"),
    }
}

fn finish_single_backend_page(
    enrichment: super::enrichment::ArticleEnrichmentOutcome,
    search_timing: ArticleSearchTiming,
    status: Option<ArticleSourceStatus>,
) -> ArticleSearchPage {
    let mut timings = vec![search_timing];
    timings.extend(enrichment.timings);
    let mut source_status = enrichment.statuses;
    source_status.extend(status);
    if let Some(status) = enrichment.semantic_scholar_status {
        source_status.push(status);
    }
    article_search_page(enrichment.page, source_status, timings)
}

async fn search_semantic_scholar_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    enrichment_sources: &[ArticleSource],
) -> Result<ArticleSearchPage, BioMcpError> {
    let fetch_count = limit.saturating_add(offset);
    if fetch_count > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for Semantic Scholar article search"
        )));
    }

    let (result, timing) = timed_source_call(
        ArticleSource::SemanticScholar,
        "search",
        search_semantic_scholar_candidates(filters, fetch_count, None, "federated", None),
    )
    .await;
    let outcome = result?;
    let status = outcome.status;
    let enrichment = enrich_and_finalize_article_candidates(
        outcome.rows,
        limit,
        offset,
        None,
        filters,
        enrichment_sources,
    )
    .await;
    let mut timings = vec![timing];
    timings.extend(enrichment.timings);
    let mut source_status = vec![status];
    if let Some(status) = enrichment.semantic_scholar_status {
        source_status.push(status);
    }
    source_status.extend(enrichment.statuses);
    Ok(article_search_page(enrichment.page, source_status, timings))
}

pub async fn search_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    source: ArticleSourceFilter,
) -> Result<ArticleSearchPage, BioMcpError> {
    validate_search_page_request(filters, limit, source)?;
    let plan = plan_backends(filters, source)?;
    let source_plan = super::planner::article_source_plan(filters, source)?;
    let enrichment_sources = source_plan.enrichment_sources;
    // Boxed for the same reason the federated legs are: the dispatch state
    // machine carries every provider request stack inline and exceeds the
    // test-thread stack when polled from `#[tokio::test]`.
    let dispatch = Box::pin(search_page_dispatch(
        filters,
        limit,
        offset,
        plan,
        &enrichment_sources,
    ));
    // One deadline per invocation. A caller that already scopes a deadline
    // (the variant-article path) keeps it; every other article search gets
    // the article-search budget so provider sends and enrichment loops stay
    // bounded together.
    if crate::sources::current_variant_article_deadline().is_some() {
        return dispatch.await;
    }
    let deadline =
        crate::sources::VariantArticleDeadline::from_now(article_search_deadline_budget());
    let result = crate::sources::with_variant_article_deadline(deadline.clone(), dispatch).await;
    match result {
        Err(error) if deadline.is_exhausted() && is_search_deadline_error(&error) => {
            Err(article_search_deadline_error(&deadline))
        }
        other => other,
    }
}

async fn search_page_dispatch(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    plan: BackendPlan,
    enrichment_sources: &[ArticleSource],
) -> Result<ArticleSearchPage, BioMcpError> {
    if plan == BackendPlan::TypeCapable {
        return search_type_capable_page(filters, limit, offset, enrichment_sources).await;
    }
    if filters.sort == ArticleSort::Relevance {
        if plan == BackendPlan::Both {
            return search_federated_page(filters, limit, offset, enrichment_sources).await;
        }
        if plan == BackendPlan::SemanticScholarOnly {
            return search_semantic_scholar_page(filters, limit, offset, enrichment_sources).await;
        }
        return search_relevance_page(filters, limit, offset, plan, enrichment_sources).await;
    }
    match plan {
        BackendPlan::EuropeOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::EuropePmc,
                "search",
                search_europepmc_page(filters, limit, offset),
            )
            .await;
            let page = result?;
            let status = page.partial_note.as_ref()
                .map(|note| source_degraded_status(ArticleSource::EuropePmc, note.clone()));
            let enrichment = enrich_visible_article_search_page(page, enrichment_sources).await;
            Ok(finish_single_backend_page(enrichment, timing, status))
        }
        BackendPlan::PubTatorOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubTator,
                "search",
                search_pubtator_page(filters, limit, offset),
            )
            .await;
            let page = result?;
            let status = page.partial_note.as_ref()
                .map(|note| source_degraded_status(ArticleSource::PubTator, note.clone()));
            let enrichment = enrich_visible_article_search_page(page, enrichment_sources).await;
            Ok(finish_single_backend_page(enrichment, timing, status))
        }
        BackendPlan::PubMedOnly | BackendPlan::LitSense2Only => {
            search_relevance_page(filters, limit, offset, plan, enrichment_sources).await
        }
        BackendPlan::TypeCapable => {
            unreachable!("type-capable search returned before sort dispatch")
        }
        BackendPlan::SemanticScholarOnly => {
            search_semantic_scholar_page(filters, limit, offset, enrichment_sources).await
        }
        BackendPlan::Both => {
            search_federated_page(filters, limit, offset, enrichment_sources).await
        }
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
