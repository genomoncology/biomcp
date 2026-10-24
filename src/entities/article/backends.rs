//! Article search backend fetchers for PubMed-family and semantic sources.

use std::collections::{HashMap, HashSet};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::litsense2::{LitSense2Client, LitSense2SearchHit};
use crate::sources::pubmed::{ESummaryEntry, PubMedClient, PubMedESearchParams};
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{
    SemanticScholarAuthMode, SemanticScholarClient, SemanticScholarSearchResponse,
};
use crate::transform;

use super::filters::{
    filter_semantic_disease_rows, matches_result_filters as matches_filters,
    normalized_date_bounds, resolve_semantic_disease_terms, semantic_disease_unavailable_status,
};
use super::query::{
    build_free_text_article_query, build_pubmed_search_term, build_pubtator_query,
    build_search_query, pubtator_sort,
};
use super::search::is_search_deadline_error;
use super::variant_search::sources::{retained_provider_progress, save_provider_progress};
use super::{
    ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleSourceStatus, EUROPE_PMC_PAGE_SIZE,
    MAX_FEDERATED_FETCH_RESULTS, MAX_PAGE_FETCHES, PUBMED_PAGE_SIZE, PUBTATOR_PAGE_SIZE,
    WARN_PAGE_THRESHOLD,
};

// Date-sorted candidates receive publication-date correction before local day
// bounds. Native provider windows still bound which records are discovered.
fn matches_result_filters(
    row: &ArticleSearchResult,
    filters: &ArticleSearchFilters,
    from: Option<&str>,
    to: Option<&str>,
) -> bool {
    if filters.sort == ArticleSort::Date {
        matches_filters(row, filters, None, None)
    } else {
        matches_filters(row, filters, from, to)
    }
}

fn normalized_bound_year(value: &str) -> Option<&str> {
    let year = value.get(..4)?;
    year.chars().all(|ch| ch.is_ascii_digit()).then_some(year)
}

pub(super) fn semantic_scholar_year_filter(
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Option<String> {
    let from_year = date_from.and_then(normalized_bound_year);
    let to_year = date_to.and_then(normalized_bound_year);
    match (from_year, to_year) {
        (Some(from), Some(to)) => Some(format!("{from}-{to}")),
        (Some(from), None) => Some(format!("{from}-")),
        (None, Some(to)) => Some(format!("-{to}")),
        (None, None) => None,
    }
}

pub(super) struct SemanticScholarCandidateOutcome {
    pub rows: Vec<ArticleSearchResult>,
    pub status: ArticleSourceStatus,
}

fn semantic_scholar_status(auth_mode: SemanticScholarAuthMode) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: Some(auth_mode),
        status: Some(ArticleSourceAvailability::Ok),
        message: None,
    }
}

fn semantic_scholar_unavailable_outcome(
    auth_mode: SemanticScholarAuthMode,
) -> SemanticScholarCandidateOutcome {
    let mut status = semantic_scholar_status(auth_mode);
    status.status = Some(ArticleSourceAvailability::Unavailable);
    status.message = Some("Semantic Scholar search unavailable".to_string());
    SemanticScholarCandidateOutcome {
        rows: Vec::new(),
        status,
    }
}

/// One backend's search page plus the failure, if any, that truncated it.
/// A paginating backend that already delivered rows must not discard them
/// when a later page fails mid-flight: the rows stay and the
/// held source is named degraded, as the federated merge already does.
pub(super) struct PartialSearchPage {
    pub page: SearchPage<ArticleSearchResult>,
    pub degradation: Option<ArticleSourceStatus>,
}

impl PartialSearchPage {
    pub(super) fn complete(page: SearchPage<ArticleSearchResult>) -> Self {
        Self {
            page,
            degradation: None,
        }
    }
}

/// The status a mid-flight truncation reports: the deadline says so by
/// name, any other failure says what failed.
fn truncated_source_status(source: ArticleSource, error: &BioMcpError) -> ArticleSourceStatus {
    let mut message = if is_search_deadline_error(error) {
        let mut message = format!(
            "{} did not answer before the article search deadline",
            source.display_name()
        );
        if let Some(failure) = crate::sources::literature_retry::failure(error) {
            message.push_str(&format!(": {failure}"));
        }
        message
    } else if let Some(failure) = crate::sources::literature_retry::failure(error) {
        format!("{} search degraded: {failure}", source.display_name())
    } else {
        format!("{} search degraded: {error}", source.display_name())
    };
    if source == ArticleSource::PubTator {
        crate::sources::literature_retry::current_observation().append_sent_request(&mut message);
    }
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(message),
    }
}

/// Construct a provider client honoring the invocation deadline when one
/// is active, so cache construction never blocks past the deadline on the
/// epoch lock. Without a deadline this is the plain path.
async fn pubmed_client_for_current_deadline() -> Result<PubMedClient, BioMcpError> {
    match crate::sources::current_variant_article_deadline() {
        Some(deadline) => PubMedClient::new_with_deadline(&deadline).await,
        None => PubMedClient::new(),
    }
}

async fn europepmc_client_for_current_deadline() -> Result<EuropePmcClient, BioMcpError> {
    match crate::sources::current_variant_article_deadline() {
        Some(deadline) => EuropePmcClient::new_with_deadline(&deadline).await,
        None => EuropePmcClient::new(),
    }
}

async fn pubtator_client_for_current_deadline() -> Result<PubTatorClient, BioMcpError> {
    match crate::sources::current_variant_article_deadline() {
        Some(deadline) => PubTatorClient::new_with_deadline(&deadline).await,
        None => PubTatorClient::new(),
    }
}

async fn semantic_scholar_client_for_current_deadline() -> Result<SemanticScholarClient, BioMcpError>
{
    match crate::sources::current_variant_article_deadline() {
        Some(deadline) => SemanticScholarClient::new_with_deadline(&deadline).await,
        None => SemanticScholarClient::new(),
    }
}

async fn litsense2_client_for_current_deadline() -> Result<LitSense2Client, BioMcpError> {
    match crate::sources::current_variant_article_deadline() {
        Some(deadline) => LitSense2Client::new_with_deadline(&deadline).await,
        None => LitSense2Client::new(),
    }
}

pub(super) async fn search_pubmed_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<PartialSearchPage, BioMcpError> {
    search_pubmed_page_with_context(filters, limit, offset, None, "federated", None).await
}

pub(super) async fn search_pubmed_page_with_context(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
    strict_query: Option<&str>,
) -> Result<PartialSearchPage, BioMcpError> {
    if limit == 0 || limit > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_FEDERATED_FETCH_RESULTS}"
        )));
    }
    if filters.open_access {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --open-access filtering".into(),
        ));
    }
    if filters.no_preprints {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --no-preprints filtering".into(),
        ));
    }

    let term = match strict_query {
        Some(query) => query.to_string(),
        None => build_pubmed_search_term(filters)?,
    };
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let (progress_key, retained) = retained_provider_progress(execution, route, "pubmed", &term);
    if retained.complete {
        return Ok(PartialSearchPage::complete(SearchPage::offset(
            retained.restored_rows(),
            retained.total,
        )));
    }
    let client_result = variant_article_client(execution, route, "pubmed", async {
        match execution {
            Some(execution) => PubMedClient::new_with_deadline(execution.deadline()).await,
            None => pubmed_client_for_current_deadline().await,
        }
    })
    .await;
    let (client, mut first_unit) = match client_result {
        Ok(client) => client,
        Err(error) if retained.rows.is_empty() => return Err(error),
        Err(error) => {
            return Ok(PartialSearchPage {
                page: SearchPage::offset(retained.restored_rows(), retained.total),
                degradation: Some(truncated_source_status(ArticleSource::PubMed, &error)),
            });
        }
    };
    let mut out = retained.restored_rows();
    let mut seen_pmids: HashSet<String> = out.iter().map(|row| row.pmid.clone()).collect();
    let mut total = retained.total;
    let mut batch_start = retained.offset;
    let mut visible_skipped = 0usize;
    let mut source_position = out.len();
    let mut fetched_pages = 0usize;
    let mut degradation = None;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages > 1
            && let Some(execution) = execution
        {
            execution.add_route_unit(route, "pubmed");
        }
        if fetched_pages == WARN_PAGE_THRESHOLD + 1 {
            tracing::warn!(
                "article search is deep (>{WARN_PAGE_THRESHOLD} page fetches); continuing up to {MAX_PAGE_FETCHES} — consider narrowing your query"
            );
        }

        let response = match variant_article_request(
            execution,
            route,
            "pubmed",
            &mut first_unit,
            client.esearch(&PubMedESearchParams {
                term: term.clone(),
                sort: None,
                retstart: batch_start,
                retmax: PUBMED_PAGE_SIZE,
                date_from: normalized_date_from.clone(),
                date_to: normalized_date_to.clone(),
            }),
            |response| {
                if total.is_none() {
                    total = Some(response.count as usize);
                }
                Ok(response)
            },
        )
        .await
        {
            Ok(response) => response,
            // Rows already accumulated survive a mid-flight failure; the
            // held source is named degraded.
            Err(error) if !out.is_empty() => {
                degradation = Some(truncated_source_status(ArticleSource::PubMed, &error));
                break;
            }
            Err(error) => return Err(error),
        };
        let Some(response) = response else {
            break;
        };
        if total.is_some_and(|value| offset >= value) {
            return Ok(PartialSearchPage::complete(SearchPage::offset(
                Vec::new(),
                total,
            )));
        }
        if response.idlist.is_empty() {
            save_provider_progress(
                execution,
                &progress_key,
                &out,
                batch_start,
                None,
                total,
                total.is_some_and(|value| batch_start >= value),
            );
            break;
        }

        let batch_len = response.idlist.len();
        if let Some(execution) = execution {
            execution.add_route_unit(route, "pubmed");
        }
        let outcome = match variant_article_request(
            execution,
            route,
            "pubmed",
            &mut first_unit,
            client.esummary(&response.idlist),
            |entries| {
                append_pubmed_entries(
                    entries,
                    filters,
                    normalized_date_from.as_deref(),
                    normalized_date_to.as_deref(),
                    limit,
                    offset,
                    PubMedAppendState {
                        out: &mut out,
                        seen_pmids: &mut seen_pmids,
                        visible_skipped: &mut visible_skipped,
                        source_position: &mut source_position,
                    },
                )
            },
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(error) if !out.is_empty() => {
                degradation = Some(truncated_source_status(ArticleSource::PubMed, &error));
                break;
            }
            Err(error) => return Err(error),
        };
        let Some(()) = outcome else {
            break;
        };
        // Once a strict page has made page-eligible PMIDs visible, keep their
        // verification capacity before fetching another discovery page.
        if let Some(execution) = execution {
            execution.reserve_identity_verification_through(offset.saturating_add(out.len()));
        }

        batch_start = batch_start.saturating_add(batch_len);
        save_provider_progress(
            execution,
            &progress_key,
            &out,
            batch_start,
            None,
            total,
            total.is_some_and(|value| batch_start >= value),
        );
        if total.is_some_and(|value| batch_start >= value) {
            break;
        }
    }

    Ok(PartialSearchPage {
        page: SearchPage::offset(out, total),
        degradation,
    })
}

struct PubMedAppendState<'a> {
    out: &'a mut Vec<ArticleSearchResult>,
    seen_pmids: &'a mut HashSet<String>,
    visible_skipped: &'a mut usize,
    source_position: &'a mut usize,
}

fn append_pubmed_entries(
    entries: Vec<ESummaryEntry>,
    filters: &ArticleSearchFilters,
    normalized_date_from: Option<&str>,
    normalized_date_to: Option<&str>,
    limit: usize,
    offset: usize,
    state: PubMedAppendState<'_>,
) -> Result<(), BioMcpError> {
    for entry in entries {
        let mut row = transform::article::from_pubmed_esummary_entry(&entry).ok_or_else(|| {
            BioMcpError::Api {
                api: "pubmed-eutils".to_string(),
                message: format!(
                    "ESummary entry for PMID {} has blank title after cleaning",
                    entry.uid
                ),
            }
        })?;
        if !matches_result_filters(&row, filters, normalized_date_from, normalized_date_to) {
            continue;
        }
        if !state.seen_pmids.insert(row.pmid.clone()) {
            continue;
        }
        row.source_local_position = *state.source_position;
        *state.source_position = state.source_position.saturating_add(1);
        if *state.visible_skipped < offset {
            *state.visible_skipped = state.visible_skipped.saturating_add(1);
            continue;
        }
        state.out.push(row);
        if state.out.len() >= limit && filters.sort != ArticleSort::Date {
            break;
        }
    }
    Ok(())
}

pub(super) async fn search_europepmc_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<PartialSearchPage, BioMcpError> {
    search_europepmc_page_with_context(filters, limit, offset, None, "federated", None).await
}

pub(super) async fn search_europepmc_page_with_context(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
    strict_query: Option<&str>,
) -> Result<PartialSearchPage, BioMcpError> {
    let query = match strict_query {
        Some(query) => query.to_string(),
        None => build_search_query(filters)?,
    };
    let europepmc_sort = filters.sort.as_europepmc_sort();
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let (progress_key, retained) =
        retained_provider_progress(execution, route, "europepmc", &query);
    if retained.complete {
        return Ok(PartialSearchPage::complete(SearchPage::offset(
            retained.restored_rows(),
            retained.total,
        )));
    }
    let client_result = variant_article_client(execution, route, "europepmc", async {
        match execution {
            Some(execution) => EuropePmcClient::new_with_deadline(execution.deadline()).await,
            None => europepmc_client_for_current_deadline().await,
        }
    })
    .await;
    let (europe, mut first_unit) = match client_result {
        Ok(client) => client,
        Err(error) if retained.rows.is_empty() => return Err(error),
        Err(error) => {
            return Ok(PartialSearchPage {
                page: SearchPage::offset(retained.restored_rows(), retained.total),
                degradation: Some(truncated_source_status(ArticleSource::EuropePmc, &error)),
            });
        }
    };
    let europe = europe.with_abstracts(filters.include_abstracts);
    let mut out = retained.restored_rows();
    let mut seen_pmids: HashSet<String> = out.iter().map(|row| row.pmid.clone()).collect();
    let mut total = retained.total;
    // Europe PMC ignores the `page` parameter, so paging is cursorMark-based:
    // start at `*`, follow `nextCursorMark`, and serve `offset` by walking and
    // discarding rows inside the fetched pages (bounded by MAX_PAGE_FETCHES).
    let mut cursor = retained.cursor.unwrap_or_else(|| "*".to_string());
    let mut next_cursor: Option<String> = None;
    let mut local_skip = offset;
    let mut fetched_rows = retained.offset;
    let mut source_position = out.len();
    let mut fetched_pages = 0usize;
    let mut degradation = None;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages > 1
            && let Some(execution) = execution
        {
            execution.add_route_unit(route, "europepmc");
        }
        if fetched_pages == WARN_PAGE_THRESHOLD + 1 {
            tracing::warn!(
                "article search is deep (>{WARN_PAGE_THRESHOLD} page fetches); continuing up to {MAX_PAGE_FETCHES} — consider narrowing your query"
            );
        }
        let outcome = match variant_article_request(
            execution,
            route,
            "europepmc",
            &mut first_unit,
            europe.search_query_with_sort(&query, &cursor, EUROPE_PMC_PAGE_SIZE, europepmc_sort),
            |resp| {
                if total.is_none() {
                    total = resp.hit_count.map(|v| v as usize);
                }
                next_cursor = resp.next_cursor_mark.clone();
                if total.is_some_and(|value| offset >= value) {
                    return Ok((true, false));
                }
                let Some(results) = resp.result_list.map(|v| v.result) else {
                    return Ok((false, true));
                };
                fetched_rows = fetched_rows.saturating_add(results.len());
                let empty = results.is_empty();
                for hit in results {
                    if local_skip > 0 {
                        local_skip -= 1;
                        continue;
                    }
                    let Some(mut row) = transform::article::from_europepmc_search_result(&hit)
                    else {
                        continue;
                    };
                    if !matches_result_filters(
                        &row,
                        filters,
                        normalized_date_from.as_deref(),
                        normalized_date_to.as_deref(),
                    ) || !seen_pmids.insert(row.pmid.clone())
                    {
                        continue;
                    }
                    row.source_local_position = source_position;
                    source_position = source_position.saturating_add(1);
                    out.push(row);
                    if out.len() >= limit && filters.sort != ArticleSort::Date {
                        break;
                    }
                }
                Ok((false, empty))
            },
        )
        .await
        {
            Ok(outcome) => outcome,
            // Fetched rows survive a mid-flight page failure; the held
            // source is named degraded.
            Err(error) if !out.is_empty() => {
                degradation = Some(truncated_source_status(ArticleSource::EuropePmc, &error));
                break;
            }
            Err(error) => return Err(error),
        };
        let Some((offset_beyond_total, empty)) = outcome else {
            break;
        };
        let continuation = next_cursor.clone().filter(|next| *next != cursor);
        let source_complete = empty
            || offset_beyond_total
            || continuation.is_none()
            || total.is_some_and(|value| fetched_rows >= value);
        save_provider_progress(
            execution,
            &progress_key,
            &out,
            fetched_rows,
            continuation,
            total,
            source_complete,
        );
        if offset_beyond_total {
            return Ok(PartialSearchPage::complete(SearchPage::offset(
                Vec::new(),
                total,
            )));
        }
        if empty {
            break;
        }

        // Cursor exhaustion: stop when the service omits `nextCursorMark`,
        // repeats the cursor just sent, or every hit has been fetched.
        let Some(next) = next_cursor.take() else {
            break;
        };
        if next == cursor {
            break;
        }
        if total.is_some_and(|value| fetched_rows >= value) {
            break;
        }
        cursor = next;
    }

    // Safety-first default: when date-sorted results contain no visible retraction marker,
    // try adding one matched retracted publication if available.
    if strict_query.is_none()
        && !filters.exclude_retracted
        && filters.sort == ArticleSort::Date
        && !out.iter().any(|row| row.is_retracted == Some(true))
    {
        if let Some(execution) = execution {
            execution.add_route_unit(route, "europepmc");
        }
        let retracted_query = format!("({query}) AND PUB_TYPE:\"retracted publication\"");
        let _ = variant_article_request(
            execution,
            route,
            "europepmc",
            &mut first_unit,
            europe.search_query_with_sort(&retracted_query, "*", 10, europepmc_sort),
            |resp| {
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
                if let Some(mut row) = replacement {
                    if filters.sort != ArticleSort::Date && out.len() >= limit && !out.is_empty() {
                        out.pop();
                    }
                    if filters.sort == ArticleSort::Date || out.len() < limit {
                        row.source_local_position = out.len();
                        seen_pmids.insert(row.pmid.clone());
                        out.push(row);
                    }
                }
                Ok(())
            },
        )
        .await;
    }

    Ok(PartialSearchPage {
        page: SearchPage::offset(out, total),
        degradation,
    })
}

pub(super) async fn search_pubtator_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<PartialSearchPage, BioMcpError> {
    search_pubtator_page_with_context(filters, limit, offset, None, "federated", None).await
}

pub(super) async fn search_pubtator_page_with_context(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
    strict_query: Option<&str>,
) -> Result<PartialSearchPage, BioMcpError> {
    let query_identity = format!("{filters:?}:{strict_query:?}");
    let (progress_key, retained) =
        retained_provider_progress(execution, route, "pubtator", &query_identity);
    if retained.complete {
        return Ok(PartialSearchPage::complete(SearchPage::offset(
            retained.restored_rows(),
            retained.total,
        )));
    }
    let client_result = variant_article_client(execution, route, "pubtator", async {
        match execution {
            Some(execution) => PubTatorClient::new_with_deadline(execution.deadline()).await,
            None => pubtator_client_for_current_deadline().await,
        }
    })
    .await;
    let (pubtator, mut first_unit) = match client_result {
        Ok(client) => client,
        Err(error) if retained.rows.is_empty() => return Err(error),
        Err(error) => {
            return Ok(PartialSearchPage {
                page: SearchPage::offset(retained.restored_rows(), retained.total),
                degradation: Some(truncated_source_status(ArticleSource::PubTator, &error)),
            });
        }
    };
    let query = match strict_query {
        Some(query) => query.to_string(),
        None => build_pubtator_query(filters, &pubtator).await?,
    };
    let sort = pubtator_sort(filters.sort);
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let mut out = retained.restored_rows();
    let mut seen_pmids: HashSet<String> = out.iter().map(|row| row.pmid.clone()).collect();
    let mut total = retained.total;
    let mut page = retained.offset.max((offset / PUBTATOR_PAGE_SIZE) + 1);
    let mut local_skip = offset % PUBTATOR_PAGE_SIZE;
    let mut source_position = out.len();
    let mut fetched_pages = 0usize;
    let mut degradation = None;
    let mut year_filter: Option<Vec<String>> = None;
    let mut date_facets_checked = offset != 0;
    let mut probe_rows = Vec::new();
    let mut source_exhausted = false;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages > 1
            && let Some(execution) = execution
        {
            execution.add_route_unit(route, "pubtator");
        }
        let mut restart_with_years = None;
        let outcome = match variant_article_request(
            execution,
            route,
            "pubtator",
            &mut first_unit,
            pubtator.search(
                &query,
                page,
                PUBTATOR_PAGE_SIZE,
                sort,
                year_filter.as_deref(),
            ),
            |resp| {
                if year_filter.is_some() {
                    probe_rows.clear();
                }
                if !date_facets_checked {
                    date_facets_checked = true;
                    restart_with_years = resp.date_filter_years(
                        normalized_date_from
                            .as_deref()
                            .and_then(normalized_bound_year),
                        normalized_date_to
                            .as_deref()
                            .and_then(normalized_bound_year),
                    );
                }
                if total.is_none() {
                    total = resp.count.map(|v| v as usize);
                }
                if total.is_some_and(|value| offset >= value) {
                    return Ok((true, false));
                }
                let empty = resp.results.is_empty();
                for hit in resp.results {
                    if local_skip > 0 {
                        local_skip -= 1;
                        continue;
                    }
                    let Some(mut row) = transform::article::from_pubtator_search_result(&hit)
                    else {
                        continue;
                    };
                    if !matches_result_filters(
                        &row,
                        filters,
                        normalized_date_from.as_deref(),
                        normalized_date_to.as_deref(),
                    ) || !seen_pmids.insert(row.pmid.clone())
                    {
                        continue;
                    }
                    row.source_local_position = source_position;
                    source_position = source_position.saturating_add(1);
                    out.push(row);
                    if out.len() >= limit && filters.sort != ArticleSort::Date {
                        break;
                    }
                }
                Ok((false, empty))
            },
        )
        .await
        {
            Ok(outcome) => outcome,
            // Fetched rows survive a mid-flight page failure; the held
            // source is named degraded.
            Err(error) if !out.is_empty() || !probe_rows.is_empty() => {
                if out.is_empty() {
                    out = std::mem::take(&mut probe_rows);
                }
                degradation = Some(truncated_source_status(ArticleSource::PubTator, &error));
                break;
            }
            Err(error) => return Err(error),
        };
        let Some((offset_beyond_total, empty)) = outcome else {
            break;
        };
        if let Some(years) = restart_with_years {
            if years.is_empty() && out.is_empty() {
                return Ok(PartialSearchPage::complete(SearchPage::offset(
                    Vec::new(),
                    Some(0),
                )));
            }
            // Keep the same relevance order and exact day filtering. The native
            // year facet removes historical pages, including for open bounds.
            // Keep first-page rows only as a failure backup. A successful native
            // query owns its own order, scores and duplicate selection.
            if !years.is_empty() && (filters.sort == ArticleSort::Date || out.len() < limit) {
                probe_rows = std::mem::take(&mut out);
                seen_pmids.clear();
                source_position = 0;
                year_filter = Some(years);
                total = None;
                continue;
            }
        }
        let page_complete = empty
            || offset_beyond_total
            || total.is_some_and(|value| page.saturating_mul(PUBTATOR_PAGE_SIZE) >= value);
        save_provider_progress(
            execution,
            &progress_key,
            &out,
            page + 1,
            None,
            total,
            page_complete,
        );
        if offset_beyond_total {
            return Ok(PartialSearchPage::complete(SearchPage::offset(
                Vec::new(),
                total,
            )));
        }
        if empty {
            source_exhausted = true;
            break;
        }
        if total.is_some_and(|value| page.saturating_mul(PUBTATOR_PAGE_SIZE) >= value) {
            source_exhausted = true;
            break;
        }
        page += 1;
    }

    if year_filter.is_some() {
        // The provider counts whole years; only an exhausted successful stream
        // gives the total after exact day filtering.
        total =
            (source_exhausted && degradation.is_none() && out.len() < limit).then_some(out.len());
    }
    Ok(PartialSearchPage {
        page: SearchPage::offset(out, total),
        degradation,
    })
}

pub(super) async fn search_semantic_scholar_candidates(
    filters: &ArticleSearchFilters,
    limit: usize,
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
    route: &str,
    strict_query: Option<&str>,
) -> Result<SemanticScholarCandidateOutcome, BioMcpError> {
    let page_limit = limit.min(crate::sources::semantic_scholar::SEMANTIC_SCHOLAR_SEARCH_PAGE_MAX);
    let query = strict_query
        .map(str::to_string)
        .unwrap_or_else(|| build_free_text_article_query(filters));
    let query_identity = format!("{filters:?}:{strict_query:?}");
    let (progress_key, retained) =
        retained_provider_progress(execution, route, "semanticscholar", &query_identity);
    if retained.complete {
        return Ok(SemanticScholarCandidateOutcome {
            rows: retained.restored_rows(),
            status: ArticleSourceStatus {
                source: ArticleSource::SemanticScholar,
                enabled: true,
                auth_mode: None,
                status: Some(ArticleSourceAvailability::Ok),
                message: Some("reused retained provider acquisition".into()),
            },
        });
    }
    let client_result = variant_article_client(execution, route, "semanticscholar", async {
        match execution {
            Some(execution) => SemanticScholarClient::new_with_deadline(execution.deadline()).await,
            None => semantic_scholar_client_for_current_deadline().await,
        }
    })
    .await;
    let (client, mut first_unit) = match client_result {
        Ok(client) => client,
        Err(error) if retained.rows.is_empty() => return Err(error),
        Err(error) => {
            return Ok(SemanticScholarCandidateOutcome {
                rows: retained.restored_rows(),
                status: ArticleSourceStatus {
                    source: ArticleSource::SemanticScholar,
                    enabled: true,
                    auth_mode: None,
                    status: Some(ArticleSourceAvailability::Degraded),
                    message: Some(error.to_string()),
                },
            });
        }
    };
    let auth_mode = client.auth_mode();
    let status = semantic_scholar_status(auth_mode);

    let disease_terms =
        match resolve_semantic_disease_terms(filters, ArticleSource::SemanticScholar).await {
            Ok(terms) => terms,
            Err(error) => {
                let mut outcome = semantic_scholar_unavailable_outcome(auth_mode);
                outcome.status.message =
                    semantic_disease_unavailable_status(ArticleSource::SemanticScholar, error)
                        .message;
                return Ok(outcome);
            }
        };

    if query.trim().is_empty() {
        return Ok(SemanticScholarCandidateOutcome {
            rows: Vec::new(),
            status,
        });
    }
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let year_filter = semantic_scholar_year_filter(
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
    );

    let mut rows = retained.restored_rows();
    let mut native_offset = retained.offset;
    let mut cursor = retained.cursor;
    let mut total = retained.total;
    let mut complete = false;
    let mut status = status;
    let mut seen = rows
        .iter()
        .map(super::candidates::stable_article_identifier)
        .collect::<HashSet<_>>();
    for _ in 0..MAX_PAGE_FETCHES {
        if rows.len() >= limit {
            break;
        }
        if strict_query.is_none()
            && native_offset
                >= crate::sources::semantic_scholar::SEMANTIC_SCHOLAR_RELEVANCE_MAX_RESULTS
        {
            complete = false;
            status.status = Some(ArticleSourceAvailability::Degraded);
            status.message = Some(
                "Semantic Scholar relevance retrieval reached its 1000-result provider limit"
                    .into(),
            );
            break;
        }
        let response = if strict_query.is_some() {
            variant_article_request(
                execution,
                route,
                "semanticscholar",
                &mut first_unit,
                client.paper_search_bulk_page(&query, page_limit, cursor.as_deref()),
                Ok,
            )
            .await
        } else {
            variant_article_request(
                execution,
                route,
                "semanticscholar",
                &mut first_unit,
                client.paper_search_page(&query, page_limit, year_filter.as_deref(), native_offset),
                Ok,
            )
            .await
        };
        let response = match response {
            Ok(Some(response)) => response,
            Ok(None) => {
                status.status = Some(ArticleSourceAvailability::Degraded);
                break;
            }
            Err(error) => {
                crate::error::warn_external_failure(
                    &error,
                    crate::error::SourceProvider::SEMANTIC_SCHOLAR,
                    "article search",
                );
                if rows.is_empty() {
                    return Ok(semantic_scholar_unavailable_outcome(auth_mode));
                }
                status.status = Some(ArticleSourceAvailability::Degraded);
                status.message = Some(format!("Semantic Scholar acquisition unfinished: {error}"));
                break;
            }
        };
        let received = response.data.len();
        total = response.total.map(|total| total as usize);
        let next_offset = response.next;
        let next_cursor = response.token.clone();
        complete = received == 0
            || if strict_query.is_some() {
                next_cursor.is_none()
            } else {
                next_offset.is_none()
            };
        let source_position = native_offset;
        native_offset = next_offset.unwrap_or(native_offset.saturating_add(received));
        let mut page = semantic_scholar_rows_from_response(
            filters,
            normalized_date_from.as_deref(),
            normalized_date_to.as_deref(),
            response,
        );
        filter_semantic_disease_rows(&mut page, disease_terms.as_ref());
        for mut row in page {
            row.source_local_position += source_position;
            if seen.insert(super::candidates::stable_article_identifier(&row)) {
                rows.push(row);
            }
        }
        if !complete && (next_cursor == cursor && strict_query.is_some()) {
            break;
        }
        cursor = next_cursor;
        save_provider_progress(
            execution,
            &progress_key,
            &rows,
            native_offset,
            cursor.clone(),
            total,
            complete,
        );
        if complete || execution.is_none() {
            break;
        }
        if let Some(execution) = execution {
            execution.add_route_unit(route, "semanticscholar");
        }
    }
    if !complete && execution.is_some() {
        status.status = Some(ArticleSourceAvailability::Degraded);
        status
            .message
            .get_or_insert_with(|| "Semantic Scholar acquisition is unfinished".into());
    }
    save_provider_progress(
        execution,
        &progress_key,
        &rows,
        native_offset,
        cursor,
        total,
        complete,
    );
    Ok(SemanticScholarCandidateOutcome { rows, status })
}

fn semantic_scholar_rows_from_response(
    filters: &ArticleSearchFilters,
    normalized_date_from: Option<&str>,
    normalized_date_to: Option<&str>,
    response: SemanticScholarSearchResponse,
) -> Vec<ArticleSearchResult> {
    let mut rows = Vec::with_capacity(response.data.len());
    let mut source_position = 0usize;
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
        let mut row = ArticleSearchResult {
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
            arxiv_id: external_ids
                .and_then(|ids| ids.arxiv.clone())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            semantic_scholar_id: paper
                .paper_id
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            title,
            journal: paper
                .venue
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            date: paper.year.map(|year| year.to_string()),
            first_index_date: None,
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
            source_local_position: 0,
        };
        if matches_result_filters(&row, filters, normalized_date_from, normalized_date_to) {
            row.source_local_position = source_position;
            source_position = source_position.saturating_add(1);
            rows.push(row);
        }
    }

    rows
}

pub(super) async fn search_litsense2_candidates(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<PartialSearchPage, BioMcpError> {
    let query = build_free_text_article_query(filters);
    if query.trim().is_empty() {
        return Ok(PartialSearchPage::complete(SearchPage::offset(
            Vec::new(),
            None,
        )));
    }
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let litsense2 = litsense2_client_for_current_deadline().await?;
    let disease_terms =
        match resolve_semantic_disease_terms(filters, ArticleSource::LitSense2).await {
            Ok(terms) => terms,
            Err(error) => {
                return Ok(PartialSearchPage {
                    page: SearchPage::offset(Vec::new(), None),
                    degradation: Some(semantic_disease_unavailable_status(
                        ArticleSource::LitSense2,
                        error,
                    )),
                });
            }
        };
    let hits = litsense2.sentence_search(&query).await?;
    let deduped = dedupe_litsense2_hits(hits);

    let pmids = deduped
        .iter()
        .map(|(hit, _)| hit.pmid.to_string())
        .collect::<Vec<_>>();
    let hydrated = hydrate_pubmed_entries(
        pubmed_client_for_current_deadline()
            .await?
            .esummary(&pmids)
            .await?,
    );

    let mut rows = litsense2_rows_from_hits(
        filters,
        usize::MAX,
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
        deduped,
        hydrated,
    );
    filter_semantic_disease_rows(&mut rows, disease_terms.as_ref());
    rows.truncate(limit);
    Ok(PartialSearchPage::complete(SearchPage::offset(rows, None)))
}

fn dedupe_litsense2_hits(hits: Vec<LitSense2SearchHit>) -> Vec<(LitSense2SearchHit, usize)> {
    let mut deduped: HashMap<u64, (LitSense2SearchHit, usize)> = HashMap::new();
    for (index, hit) in hits.into_iter().enumerate() {
        match deduped.get_mut(&hit.pmid) {
            Some((best, _)) if hit.score > best.score => *best = hit,
            Some(_) => {}
            None => {
                deduped.insert(hit.pmid, (hit, index));
            }
        }
    }

    let mut deduped = deduped.into_values().collect::<Vec<_>>();
    deduped.sort_by(
        |(left_hit, left_first_seen), (right_hit, right_first_seen)| {
            right_hit
                .score
                .total_cmp(&left_hit.score)
                .then_with(|| left_first_seen.cmp(right_first_seen))
        },
    );
    deduped
}

fn litsense2_rows_from_hits(
    filters: &ArticleSearchFilters,
    limit: usize,
    normalized_date_from: Option<&str>,
    normalized_date_to: Option<&str>,
    deduped: Vec<(LitSense2SearchHit, usize)>,
    mut hydrated: HashMap<String, ArticleSearchResult>,
) -> Vec<ArticleSearchResult> {
    let mut rows = Vec::with_capacity(deduped.len());
    let mut source_position = 0usize;
    for (hit, _) in deduped {
        let pmid = hit.pmid.to_string();
        let cleaned_text = transform::article::clean_abstract(&hit.text);
        let fallback_title = if cleaned_text.is_empty() {
            format!("PMID {pmid}")
        } else {
            transform::article::article_search_fallback_title(&cleaned_text)
        };
        let mut row = hydrated
            .remove(&pmid)
            .unwrap_or_else(|| ArticleSearchResult {
                pmid: pmid.clone(),
                pmcid: hit.pmcid.clone(),
                doi: None,
                arxiv_id: None,
                semantic_scholar_id: None,
                title: fallback_title.clone(),
                journal: None,
                date: None,
                first_index_date: None,
                citation_count: None,
                influential_citation_count: None,
                source: ArticleSource::LitSense2,
                matched_sources: vec![ArticleSource::LitSense2],
                score: Some(hit.score),
                is_retracted: None,
                abstract_snippet: transform::article::article_search_abstract_snippet(
                    &cleaned_text,
                ),
                ranking: None,
                normalized_title: transform::article::normalize_article_search_text(
                    &fallback_title,
                ),
                normalized_abstract: transform::article::normalize_article_search_text(
                    &cleaned_text,
                ),
                publication_type: None,
                source_local_position: 0,
            });
        row.source = ArticleSource::LitSense2;
        row.matched_sources = vec![ArticleSource::LitSense2];
        row.score = Some(hit.score);
        if row.pmcid.is_none() {
            row.pmcid = hit.pmcid.clone();
        }
        if row.title.trim().is_empty() {
            row.title = fallback_title.clone();
        }
        row.abstract_snippet = transform::article::article_search_abstract_snippet(&cleaned_text);
        row.normalized_title = transform::article::normalize_article_search_text(&row.title);
        row.normalized_abstract = transform::article::normalize_article_search_text(&cleaned_text);
        row.is_retracted = None;
        row.publication_type = None;
        if !matches_result_filters(&row, filters, normalized_date_from, normalized_date_to) {
            continue;
        }
        row.source_local_position = source_position;
        source_position = source_position.saturating_add(1);
        rows.push(row);
        if rows.len() >= limit {
            break;
        }
    }

    rows
}

fn hydrate_pubmed_entries(entries: Vec<ESummaryEntry>) -> HashMap<String, ArticleSearchResult> {
    entries
        .into_iter()
        .filter_map(|entry| {
            transform::article::from_pubmed_esummary_entry(&entry)
                .map(|row| (row.pmid.clone(), row))
        })
        .collect()
}

pub(crate) mod provider_unit;
#[cfg(test)]
mod tests;

pub(crate) use self::provider_unit::{
    VariantArticleProviderUnit, variant_article_client, variant_article_request,
};
