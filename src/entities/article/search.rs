//! Article search orchestration across planner, backends, enrichment, and finalization.

use std::future::Future;
use std::time::Duration;

use tokio::time::timeout;
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;

use super::backends::{
    PartialSearchPage, search_europepmc_page, search_europepmc_page_with_context,
    search_litsense2_candidates, search_pubmed_page, search_pubmed_page_with_context,
    search_pubtator_page, search_pubtator_page_with_context, search_semantic_scholar_candidates,
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
pub(crate) use deadline::article_search_deadline_budget;
pub(in crate::entities::article) use deadline::is_search_deadline_error;
pub(crate) use deadline::{
    ARTICLE_SEARCH_DEADLINE_REASON_PREFIX, ARTICLE_SEARCH_DEADLINE_SUGGESTION,
};
use deadline::{article_search_deadline_error, timed_source_call, timed_source_leg};
mod status;
use status::{
    SemanticScholarStatusTracker, source_degraded_status, source_provider, timed_out_source_status,
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

fn page_outcome_truncated(
    outcome: &FederatedSourceOutcome<PartialSearchPage>,
    fetch_count: usize,
) -> bool {
    // A mid-flight truncation means more results exist than were returned.
    matches!(outcome, FederatedSourceOutcome::Available(partial) if partial.degradation.is_some())
        || matches!(
            outcome,
            FederatedSourceOutcome::Available(partial)
                if partial.page.total.is_some_and(|total| total > partial.page.results.len())
                    || (partial.page.total.is_none()
                        && partial.page.results.len() >= fetch_count)
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
    pubtator_leg: FederatedSourceOutcome<PartialSearchPage>,
    europe_leg: FederatedSourceOutcome<PartialSearchPage>,
    pubmed_leg: Option<FederatedSourceOutcome<PartialSearchPage>>,
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
    let (pubmed_rows, pubmed_degradation) = match pubmed_leg {
        Some(FederatedSourceOutcome::Available(partial)) => {
            (partial.page.results, partial.degradation)
        }
        Some(FederatedSourceOutcome::Unavailable { status, .. }) => {
            source_status.push(status);
            (Vec::new(), None)
        }
        None => (Vec::new(), None),
    };

    match (pubtator_leg, europe_leg) {
        (
            FederatedSourceOutcome::Available(pubtator_partial),
            FederatedSourceOutcome::Available(europe_partial),
        ) => {
            let mut merged = pubtator_partial.page.results;
            merged.extend(europe_partial.page.results);
            merged.extend(pubmed_rows);
            merged.extend(semantic_scholar_rows);
            merged.extend(litsense2_rows);
            for degradation in [pubtator_partial.degradation, europe_partial.degradation]
                .into_iter()
                .flatten()
            {
                source_status.push(degradation);
            }
            if let Some(degradation) = pubmed_degradation {
                source_status.push(degradation);
            }
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
            FederatedSourceOutcome::Available(pubtator_partial),
            FederatedSourceOutcome::Unavailable { status, .. },
        ) => {
            source_status.push(status);
            let mut rows = pubtator_partial.page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            if let Some(degradation) = pubtator_partial.degradation {
                source_status.push(degradation);
            }
            if let Some(degradation) = pubmed_degradation {
                source_status.push(degradation);
            }
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
            FederatedSourceOutcome::Available(europe_partial),
        ) => {
            source_status.push(status);
            let mut rows = europe_partial.page.results;
            rows.extend(pubmed_rows);
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            if let Some(degradation) = europe_partial.degradation {
                source_status.push(degradation);
            }
            if let Some(degradation) = pubmed_degradation {
                source_status.push(degradation);
            }
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
                error: europe_error,
                status: europe_status,
            },
        ) => {
            source_status.extend([pubtator_status, europe_status]);
            let mut rows = pubmed_rows;
            rows.extend(semantic_scholar_rows);
            rows.extend(litsense2_rows);
            if let Some(degradation) = pubmed_degradation {
                source_status.push(degradation);
            }
            // A fast non-retryable failure must not hide the deadline: when
            // either primary died on the invocation deadline, that error
            // explains the invocation and carries the deadline translation
            // through `search_page` (ticket 2023).
            let primary_error = match (error, europe_error) {
                (Some(err), _) if is_search_deadline_error(&err) => Some(err),
                (_, Some(err)) if is_search_deadline_error(&err) => Some(err),
                (error, _) => error,
            }
            .unwrap_or_else(|| unavailable_source_error(ArticleSource::PubTator));
            Ok(FederatedArticleRows {
                rows,
                source_status,
                semantic_scholar_status,
                truncated_sources: Vec::new(),
                timings: Vec::new(),
                primary_error: Some(primary_error),
            })
        }
    }
}

fn collect_type_capable_article_rows(
    europe_leg: FederatedSourceOutcome<PartialSearchPage>,
    pubmed_leg: FederatedSourceOutcome<PartialSearchPage>,
) -> Result<TypeCapableArticleRows, BioMcpError> {
    match (europe_leg, pubmed_leg) {
        (
            FederatedSourceOutcome::Available(europe_partial),
            FederatedSourceOutcome::Available(pubmed_partial),
        ) => {
            let mut rows = europe_partial.page.results;
            rows.extend(pubmed_partial.page.results);
            let mut source_status = Vec::new();
            for degradation in [europe_partial.degradation, pubmed_partial.degradation]
                .into_iter()
                .flatten()
            {
                source_status.push(degradation);
            }
            Ok(TypeCapableArticleRows {
                rows,
                total: None,
                source_status,
            })
        }
        (
            FederatedSourceOutcome::Available(europe_partial),
            FederatedSourceOutcome::Unavailable { status, .. },
        ) => {
            let mut source_status = vec![status];
            if let Some(degradation) = europe_partial.degradation {
                source_status.push(degradation);
            }
            Ok(TypeCapableArticleRows {
                rows: europe_partial.page.results,
                total: europe_partial.page.total,
                source_status,
            })
        }
        (
            FederatedSourceOutcome::Unavailable { status, .. },
            FederatedSourceOutcome::Available(pubmed_partial),
        ) => {
            let mut source_status = vec![status];
            if let Some(degradation) = pubmed_partial.degradation {
                source_status.push(degradation);
            }
            Ok(TypeCapableArticleRows {
                rows: pubmed_partial.page.results,
                total: pubmed_partial.page.total,
                source_status,
            })
        }
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
            let partial = result?;
            let enrichment = enrich_and_finalize_article_candidates(
                partial.page.results,
                limit,
                offset,
                partial.page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(
                enrichment,
                timing,
                partial.degradation,
            ))
        }
        BackendPlan::PubTatorOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubTator,
                "search",
                search_pubtator_page(filters, fetch_count, 0),
            )
            .await;
            let partial = result?;
            let enrichment = enrich_and_finalize_article_candidates(
                partial.page.results,
                limit,
                offset,
                partial.page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(
                enrichment,
                timing,
                partial.degradation,
            ))
        }
        BackendPlan::PubMedOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubMed,
                "search",
                search_pubmed_page(filters, fetch_count, 0),
            )
            .await;
            let partial = result?;
            let enrichment = enrich_and_finalize_article_candidates(
                partial.page.results,
                limit,
                offset,
                partial.page.total,
                filters,
                enrichment_sources,
            )
            .await;
            Ok(finish_single_backend_page(
                enrichment,
                timing,
                partial.degradation,
            ))
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
    degradation: Option<ArticleSourceStatus>,
) -> ArticleSearchPage {
    let mut timings = vec![search_timing];
    timings.extend(enrichment.timings);
    let mut source_status = Vec::new();
    if let Some(degradation) = degradation {
        source_status.push(degradation);
    }
    source_status.extend(enrichment.statuses);
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
    // Every sort and plan shares one window: the date and citations arms
    // walk provider pages to serve a deep offset, so an unbounded request
    // burns up to MAX_PAGE_FETCHES pages per source and still reports
    // `has_more` (ticket 2023).
    if limit.saturating_add(offset) > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset + --limit must be <= {MAX_FEDERATED_FETCH_RESULTS} for article search"
        )));
    }
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
            let partial = result?;
            let enrichment =
                enrich_visible_article_search_page(partial.page, enrichment_sources).await;
            Ok(finish_single_backend_page(
                enrichment,
                timing,
                partial.degradation,
            ))
        }
        BackendPlan::PubTatorOnly => {
            let (result, timing) = timed_source_call(
                ArticleSource::PubTator,
                "search",
                search_pubtator_page(filters, limit, offset),
            )
            .await;
            let partial = result?;
            let enrichment =
                enrich_visible_article_search_page(partial.page, enrichment_sources).await;
            Ok(finish_single_backend_page(
                enrichment,
                timing,
                partial.degradation,
            ))
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
