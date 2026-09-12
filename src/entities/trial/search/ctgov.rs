//! CTGov trial search query, pagination, and count helpers.

use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use tracing::warn;

use crate::entities::drug::{TrialAlias, TrialAliasSource, resolve_trial_aliases_with_sources};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;

use super::super::ClinicalTrialSearchUnknownReason;
use super::super::{ClinicalTrialSearchTotal, TrialSearchFilters, TrialSearchHit, TrialSource};
use super::eligibility::{DetailFilterOutcome, ctgov_nct_id};
use super::{
    CTGOV_COUNT_CAP_REASON, CTGOV_COUNT_PAGE_SIZE, CtGovSearchContext, add_unique_ctgov_nct_ids,
    biodata_plan_error, claim_ctgov_candidate, completed_ctgov_union_count,
    ctgov_count_from_native_total, final_ctgov_union_count, prepare_ctgov_search_context,
    sort_trials_by_status_priority, validate_search_page_args, validate_trial_search,
    verify_age_eligibility, verify_detail_filters,
};

const CTGOV_MAX_PAGE_FETCHES: usize = 20;

fn build_ctgov_search_plan(
    filters: &TrialSearchFilters,
    _context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_query: Option<&str>,
    page_token: Option<String>,
    page_size: usize,
    count_total: bool,
) -> Result<biodata::ClinicalTrialsGovApiV2SearchPlan, BioMcpError> {
    let mut fields = super::biodata_filter_fields(filters, intervention_query);
    fields.condition = condition_query.map(str::to_owned);
    let filters = biodata::ClinicalTrialSearchFilters::new(fields, Default::default())
        .map_err(|error| biodata_plan_error("invalid trial search", error))?;
    biodata::ClinicalTrialsGovApiV2SearchPlan::new(
        &filters,
        page_size,
        page_token.as_deref(),
        count_total,
    )
    .map_err(|error| biodata_plan_error("invalid ClinicalTrials.gov trial search", error))
}

async fn apply_ctgov_post_filters(
    client: &ClinicalTrialsClient,
    context: &CtGovSearchContext,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) -> DetailFilterOutcome {
    let facility_geo = context
        .facility_geo_verification
        .as_ref()
        .map(|(facility, lat, lon, distance)| (facility.as_str(), *lat, *lon, *distance));
    let mut outcome =
        verify_detail_filters(client, studies, facility_geo, &context.eligibility_keywords).await;
    if let Some(age) = context.age_verification {
        outcome.studies = verify_age_eligibility(outcome.studies, age);
    }
    outcome
}

struct CtGovRawPage {
    provider_total: biodata::ClinicalTrialProviderTotal,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    provider_cursor: biodata::ClinicalTrialProviderCursor,
    raw_study_count: usize,
    verification_incomplete: bool,
}

impl std::fmt::Debug for CtGovRawPage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CtGovRawPage")
            .field("provider_total", &self.provider_total)
            .field("raw_study_count", &self.raw_study_count)
            .field("verification_incomplete", &self.verification_incomplete)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct CtGovWorkerState {
    condition_query: Option<String>,
    intervention_query: Option<String>,
    intervention_source: &'static str,
    matched_intervention_label: Option<String>,
    next_page_token: Option<String>,
    cursor_unusable: bool,
    exhausted: bool,
    pages_fetched: usize,
    provider_total_seen: bool,
}

impl std::fmt::Debug for CtGovWorkerState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CtGovWorkerState")
            .field("exhausted", &self.exhausted)
            .field("pages_fetched", &self.pages_fetched)
            .field("cursor_unusable", &self.cursor_unusable)
            .finish_non_exhaustive()
    }
}

struct CtGovSinglePageState {
    rows: Vec<TrialSearchHit>,
    provider_total: Option<biodata::ClinicalTrialProviderTotal>,
    retained_total: usize,
    exhausted: bool,
    page_token: Option<String>,
    stopped_inside_page: bool,
    unusable_cursor: bool,
    verification_incomplete: bool,
    traversal_capped: bool,
    remaining_skip: usize,
    started_with_cursor: bool,
}

impl CtGovSinglePageState {
    fn new(next_page: Option<String>, offset: usize, _requested_total: bool) -> Self {
        Self {
            rows: Vec::new(),
            provider_total: None,
            retained_total: 0,
            exhausted: false,
            page_token: next_page
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            remaining_skip: offset,
            stopped_inside_page: false,
            unusable_cursor: false,
            verification_incomplete: false,
            traversal_capped: false,
            started_with_cursor: next_page
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty()),
        }
    }
}

fn raw_condition_query(filters: &TrialSearchFilters) -> Option<&str> {
    filters
        .condition
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn raw_intervention_query(filters: &TrialSearchFilters) -> Option<&str> {
    filters
        .intervention
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests;

async fn resolve_ctgov_intervention_aliases(
    filters: &TrialSearchFilters,
) -> Result<Vec<TrialAlias>, BioMcpError> {
    if !matches!(filters.source, TrialSource::ClinicalTrialsGov) || filters.no_alias_expand {
        return Ok(raw_intervention_query(filters)
            .map(|value| {
                vec![TrialAlias {
                    label: value.to_string(),
                    source: TrialAliasSource::Requested,
                }]
            })
            .unwrap_or_default());
    }

    let Some(intervention_query) = raw_intervention_query(filters) else {
        return Ok(Vec::new());
    };

    resolve_trial_aliases_with_sources(intervention_query).await
}

fn fanout_next_page_error() -> BioMcpError {
    BioMcpError::InvalidArgument(
        "--next-page is not supported when CTGov intervention alias expansion uses multiple queries; use --offset or --no-alias-expand".into(),
    )
}

async fn fetch_ctgov_raw_page(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_query: Option<&str>,
    page_token: Option<String>,
    page_size: usize,
) -> Result<CtGovRawPage, BioMcpError> {
    let count_total = !filters.no_count_total;
    let resp = client
        .search(&build_ctgov_search_plan(
            filters,
            context,
            condition_query,
            intervention_query,
            page_token,
            page_size,
            count_total,
        )?)
        .await?;

    Ok(CtGovRawPage {
        provider_total: resp.provider_total().clone(),
        raw_study_count: resp.results().unwrap_or_default().len(),
        studies: resp.results().unwrap_or_default().to_vec(),
        provider_cursor: resp.provider_cursor().clone(),
        verification_incomplete: false,
    })
}

async fn fetch_ctgov_filtered_page(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_query: Option<&str>,
    page_token: Option<String>,
    page_size: usize,
) -> Result<CtGovRawPage, BioMcpError> {
    let mut page = fetch_ctgov_raw_page(
        client,
        filters,
        context,
        condition_query,
        intervention_query,
        page_token,
        page_size,
    )
    .await?;
    if page.raw_study_count > 0 {
        let outcome = apply_ctgov_post_filters(client, context, page.studies).await;
        page.studies = outcome.studies;
        page.verification_incomplete = outcome.incomplete;
    }
    Ok(page)
}

fn handle_ctgov_worker_outcome(
    worker_index: usize,
    worker: &CtGovWorkerState,
    result: Result<CtGovRawPage, BioMcpError>,
) -> Result<Option<CtGovRawPage>, BioMcpError> {
    match result {
        Ok(page) => Ok(Some(page)),
        Err(BioMcpError::WithSourceContext { context, source }) => {
            handle_ctgov_worker_outcome(worker_index, worker, Err(*source))
                .map_err(|error| error.with_source_context(context))
        }
        Err(BioMcpError::CtGovInterventionQueryRejected) if worker_index > 0 => {
            warn!(
                worker_index,
                source = worker.intervention_source,
                "Skipping expanded CTGov intervention alias rejected by the query parser"
            );
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

fn apply_ctgov_single_page(
    state: &mut CtGovSinglePageState,
    _context: &CtGovSearchContext,
    worker: &CtGovWorkerState,
    limit: usize,
    page: CtGovRawPage,
) {
    if state.provider_total.is_none() {
        state.provider_total = Some(page.provider_total.clone());
    }
    state.verification_incomplete |= page.verification_incomplete;

    if page.raw_study_count == 0 {
        apply_provider_cursor(state, page.provider_cursor);
        return;
    }

    let provider_cursor = page.provider_cursor;
    let mut studies = page.studies;
    state.retained_total = state.retained_total.saturating_add(studies.len());

    let page_started_with_skip = state.remaining_skip;
    let rows_before_page = state.rows.len();
    let page_study_count = studies.len();
    let mut page_consumed = 0;
    for study in studies.drain(..) {
        page_consumed += 1;
        if state.remaining_skip > 0 {
            state.remaining_skip -= 1;
            continue;
        }
        if state.rows.len() < limit {
            let Ok(mut row) = TrialSearchHit::from_biodata(study.into_projection()) else {
                continue;
            };
            row.matched_intervention_label = worker.matched_intervention_label.clone();
            state.rows.push(row);
        }
        if state.rows.len() >= limit {
            break;
        }
    }

    if state.rows.len() >= limit {
        if page_consumed >= page_study_count {
            apply_provider_cursor(state, provider_cursor);
        } else {
            state.page_token = None;
            state.stopped_inside_page = true;
        }
        return;
    }

    if page_started_with_skip > 0
        && state.remaining_skip == 0
        && state.rows.len() > rows_before_page
        && page_consumed >= page_study_count
    {
        apply_provider_cursor(state, provider_cursor);
        return;
    }

    apply_provider_cursor(state, provider_cursor);
}

fn apply_provider_cursor(
    state: &mut CtGovSinglePageState,
    cursor: biodata::ClinicalTrialProviderCursor,
) {
    match cursor {
        biodata::ClinicalTrialProviderCursor::Present(value) if !value.trim().is_empty() => {
            state.page_token = Some(value);
        }
        biodata::ClinicalTrialProviderCursor::Present(_) => {
            state.page_token = None;
            state.unusable_cursor = true;
        }
        biodata::ClinicalTrialProviderCursor::Absent
        | biodata::ClinicalTrialProviderCursor::Unavailable => {
            state.page_token = None;
            state.exhausted = true;
        }
    }
}

fn finish_ctgov_single_page(
    mut state: CtGovSinglePageState,
    context: &CtGovSearchContext,
    limit: usize,
    offset: usize,
) -> Result<super::TrialSearchPage, BioMcpError> {
    if !context.has_explicit_status {
        sort_trials_by_status_priority(&mut state.rows);
    }

    state.rows.truncate(limit);
    let has_local_filter =
        context.uses_expensive_post_filters || context.age_verification.is_some();
    let total = if state.verification_incomplete {
        ClinicalTrialSearchTotal::unknown(
            ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
        )
    } else if state.traversal_capped {
        ClinicalTrialSearchTotal::unknown(ClinicalTrialSearchUnknownReason::TraversalLimitReached)
    } else if state.exhausted && !state.started_with_cursor {
        super::exact_total(state.retained_total)?
    } else {
        match state
            .provider_total
            .unwrap_or(biodata::ClinicalTrialProviderTotal::Absent)
        {
            biodata::ClinicalTrialProviderTotal::NotRequested => ClinicalTrialSearchTotal::unknown(
                ClinicalTrialSearchUnknownReason::TotalNotRequested,
            ),
            biodata::ClinicalTrialProviderTotal::Absent => ClinicalTrialSearchTotal::unknown(
                ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
            ),
            biodata::ClinicalTrialProviderTotal::Present(value) if has_local_filter => {
                let value = usize::try_from(value).map_err(|_| BioMcpError::InternalProcessing)?;
                super::approximate_total(value)?
            }
            biodata::ClinicalTrialProviderTotal::Present(value) => {
                let value = usize::try_from(value).map_err(|_| BioMcpError::InternalProcessing)?;
                super::exact_total(value)?
            }
        }
    };
    let known_exhausted = total
        .value()
        .and_then(|value| usize::try_from(value).ok())
        .is_some_and(|value| offset.saturating_add(state.rows.len()) >= value);
    let continuation = if state.stopped_inside_page {
        super::offset_continuation(offset.saturating_add(state.rows.len()))?
    } else if state.exhausted || known_exhausted {
        biodata::ClinicalTrialSearchContinuation::terminal()
    } else if state.unusable_cursor {
        biodata::ClinicalTrialSearchContinuation::unavailable(
            biodata::ClinicalTrialSearchContinuationUnavailableReason::UnusableProviderCursor,
        )
    } else if let Some(cursor) = state.page_token {
        biodata::ClinicalTrialSearchContinuation::cursor(cursor)
            .map_err(|_| BioMcpError::InternalProcessing)?
    } else {
        biodata::ClinicalTrialSearchContinuation::terminal()
    };
    Ok(super::TrialSearchPage {
        results: state.rows,
        total,
        continuation,
    })
}

async fn search_page_with_single_ctgov_intervention(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    worker: &CtGovWorkerState,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<super::TrialSearchPage, BioMcpError> {
    let page_size = limit.clamp(1, 100);
    let request_total = !filters.no_count_total;
    let mut state = CtGovSinglePageState::new(next_page, offset, request_total);

    for _ in 0..CTGOV_MAX_PAGE_FETCHES {
        let page = fetch_ctgov_filtered_page(
            client,
            filters,
            context,
            worker.condition_query.as_deref(),
            worker.intervention_query.as_deref(),
            state.page_token.clone(),
            page_size,
        )
        .await?;

        apply_ctgov_single_page(&mut state, context, worker, limit, page);
        if state.exhausted || state.unusable_cursor || state.rows.len() >= limit {
            break;
        }
    }
    if !state.exhausted && state.rows.len() < limit && !state.unusable_cursor {
        state.traversal_capped = true;
    }
    finish_ctgov_single_page(state, context, limit, offset)
}

fn ctgov_workers(
    condition_query: Option<&str>,
    intervention_aliases: &[TrialAlias],
) -> Vec<CtGovWorkerState> {
    let has_intervention_fanout = intervention_aliases.len() > 1;
    if intervention_aliases.is_empty() {
        return vec![CtGovWorkerState {
            condition_query: condition_query.map(str::to_string),
            intervention_query: None,
            intervention_source: "none",
            matched_intervention_label: None,
            next_page_token: None,
            cursor_unusable: false,
            exhausted: false,
            pages_fetched: 0,
            provider_total_seen: false,
        }];
    }

    intervention_aliases
        .iter()
        .map(|alias| CtGovWorkerState {
            condition_query: condition_query.map(str::to_string),
            intervention_query: Some(alias.label.clone()),
            intervention_source: alias.source.as_str(),
            matched_intervention_label: has_intervention_fanout.then(|| alias.label.clone()),
            next_page_token: None,
            cursor_unusable: false,
            exhausted: false,
            pages_fetched: 0,
            provider_total_seen: false,
        })
        .collect()
}

fn apply_worker_cursor(
    worker: &mut CtGovWorkerState,
    cursor: biodata::ClinicalTrialProviderCursor,
) {
    match cursor {
        biodata::ClinicalTrialProviderCursor::Present(value) if !value.trim().is_empty() => {
            worker.next_page_token = Some(value);
        }
        biodata::ClinicalTrialProviderCursor::Present(_) => {
            worker.next_page_token = None;
            worker.exhausted = true;
            worker.cursor_unusable = true;
        }
        biodata::ClinicalTrialProviderCursor::Absent
        | biodata::ClinicalTrialProviderCursor::Unavailable => {
            worker.next_page_token = None;
            worker.exhausted = true;
        }
    }
}

fn cap_continuable_worker(worker: &mut CtGovWorkerState) -> bool {
    let capped = worker.pages_fetched >= CTGOV_MAX_PAGE_FETCHES && !worker.exhausted;
    if capped {
        worker.next_page_token = None;
        worker.exhausted = true;
    }
    capped
}

fn push_ctgov_union_rows(
    merged_rows: &mut Vec<TrialSearchHit>,
    merged_index: &mut HashMap<String, usize>,
    matched_labels: &HashMap<String, Option<String>>,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) {
    for study in studies {
        let matched_nct_id = ctgov_nct_id(&study).unwrap_or_default();
        let Ok(mut row) = TrialSearchHit::from_biodata(study.into_projection()) else {
            continue;
        };
        if merged_index.contains_key(row.nct_id()) {
            continue;
        }
        row.matched_intervention_label = matched_labels.get(&matched_nct_id).cloned().flatten();
        merged_index.insert(row.nct_id().to_owned(), merged_rows.len());
        merged_rows.push(row);
    }
}

fn ctgov_union_continuation(
    workers: &[CtGovWorkerState],
    has_buffered_rows: bool,
    degraded_coverage: bool,
    traversal_capped: bool,
    next_offset: usize,
) -> Result<biodata::ClinicalTrialSearchContinuation, BioMcpError> {
    if degraded_coverage {
        Ok(biodata::ClinicalTrialSearchContinuation::unavailable(
            biodata::ClinicalTrialSearchContinuationUnavailableReason::IncompleteSourceCoverage,
        ))
    } else if traversal_capped {
        Ok(biodata::ClinicalTrialSearchContinuation::unavailable(
            biodata::ClinicalTrialSearchContinuationUnavailableReason::TraversalLimitReached,
        ))
    } else if workers.iter().any(|worker| worker.cursor_unusable) {
        Ok(biodata::ClinicalTrialSearchContinuation::unavailable(
            biodata::ClinicalTrialSearchContinuationUnavailableReason::UnusableProviderCursor,
        ))
    } else if has_buffered_rows {
        super::offset_continuation(next_offset)
    } else {
        Ok(biodata::ClinicalTrialSearchContinuation::terminal())
    }
}

async fn search_page_with_ctgov_union(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_aliases: &[TrialAlias],
    limit: usize,
    offset: usize,
) -> Result<super::TrialSearchPage, BioMcpError> {
    const ALIAS_PROVIDER_PAGE_SIZE: usize = 100;
    let page_size = ALIAS_PROVIDER_PAGE_SIZE;
    let mut workers = ctgov_workers(condition_query, intervention_aliases);
    let mut merged_rows: Vec<TrialSearchHit> = Vec::new();
    let mut merged_index: HashMap<String, usize> = HashMap::new();
    let mut seen_nct_ids: HashSet<String> = HashSet::new();
    let mut matched_labels: HashMap<String, Option<String>> = HashMap::new();
    let mut traversal_capped = false;
    let mut degraded_coverage = false;
    let mut verification_incomplete = false;
    let mut provider_total_sum = 0usize;
    let mut provider_total_state = biodata::ClinicalTrialProviderTotal::Present(0);

    loop {
        let active_indices: Vec<usize> = workers
            .iter()
            .enumerate()
            .filter_map(|(index, worker)| (!worker.exhausted).then_some(index))
            .collect();
        if active_indices.is_empty() {
            break;
        }

        let pages = join_all(active_indices.iter().map(|index| {
            let worker = &workers[*index];
            fetch_ctgov_raw_page(
                client,
                filters,
                context,
                worker.condition_query.as_deref(),
                worker.intervention_query.as_deref(),
                worker.next_page_token.clone(),
                page_size,
            )
        }))
        .await;
        let mut round_studies = Vec::new();

        for (index, page_result) in active_indices.into_iter().zip(pages) {
            let Some(page) = handle_ctgov_worker_outcome(index, &workers[index], page_result)?
            else {
                workers[index].exhausted = true;
                degraded_coverage = true;
                continue;
            };
            let worker = &mut workers[index];
            worker.pages_fetched += 1;
            if !worker.provider_total_seen {
                worker.provider_total_seen = true;
                match page.provider_total {
                    biodata::ClinicalTrialProviderTotal::Present(value) => {
                        provider_total_sum = provider_total_sum.saturating_add(
                            usize::try_from(value).map_err(|_| BioMcpError::InternalProcessing)?,
                        );
                    }
                    biodata::ClinicalTrialProviderTotal::NotRequested => {
                        provider_total_state = biodata::ClinicalTrialProviderTotal::NotRequested;
                    }
                    biodata::ClinicalTrialProviderTotal::Absent
                        if !matches!(
                            provider_total_state,
                            biodata::ClinicalTrialProviderTotal::NotRequested
                        ) =>
                    {
                        provider_total_state = biodata::ClinicalTrialProviderTotal::Absent;
                    }
                    biodata::ClinicalTrialProviderTotal::Absent => {}
                }
            }

            if page.raw_study_count == 0 {
                apply_worker_cursor(worker, page.provider_cursor);
                traversal_capped |= cap_continuable_worker(worker);
                continue;
            }

            for study in page.studies {
                let Some(nct_id) = claim_ctgov_candidate(&mut seen_nct_ids, &study) else {
                    continue;
                };
                if nct_id.is_empty()
                    && context.age_verification.is_some_and(|age| {
                        verify_age_eligibility(vec![study.clone()], age).is_empty()
                    })
                {
                    continue;
                }
                matched_labels
                    .entry(nct_id)
                    .or_insert_with(|| worker.matched_intervention_label.clone());
                round_studies.push(study);
            }

            apply_worker_cursor(worker, page.provider_cursor);
            traversal_capped |= cap_continuable_worker(worker);
        }

        let outcome = apply_ctgov_post_filters(client, context, round_studies).await;
        verification_incomplete |= outcome.incomplete;
        push_ctgov_union_rows(
            &mut merged_rows,
            &mut merged_index,
            &matched_labels,
            outcome.studies,
        );
    }

    if !context.has_explicit_status {
        sort_trials_by_status_priority(&mut merged_rows);
    }

    let complete = workers
        .iter()
        .all(|worker| worker.exhausted && !worker.cursor_unusable);
    let total = if degraded_coverage {
        ClinicalTrialSearchTotal::unknown(
            ClinicalTrialSearchUnknownReason::IncompleteSourceCoverage,
        )
    } else if verification_incomplete {
        ClinicalTrialSearchTotal::unknown(
            ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
        )
    } else if traversal_capped {
        ClinicalTrialSearchTotal::unknown(ClinicalTrialSearchUnknownReason::TraversalLimitReached)
    } else if complete {
        super::exact_total(merged_rows.len())?
    } else {
        match provider_total_state {
            biodata::ClinicalTrialProviderTotal::Present(_) => {
                super::approximate_total(provider_total_sum)?
            }
            biodata::ClinicalTrialProviderTotal::Absent => ClinicalTrialSearchTotal::unknown(
                ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
            ),
            biodata::ClinicalTrialProviderTotal::NotRequested => ClinicalTrialSearchTotal::unknown(
                ClinicalTrialSearchUnknownReason::TotalNotRequested,
            ),
        }
    };
    let has_buffered_rows = merged_rows.len() > offset.saturating_add(limit);
    let rows: Vec<_> = merged_rows.into_iter().skip(offset).take(limit).collect();
    let continuation = ctgov_union_continuation(
        &workers,
        has_buffered_rows,
        degraded_coverage,
        traversal_capped,
        offset.saturating_add(rows.len()),
    )?;
    Ok(super::TrialSearchPage {
        results: rows,
        total,
        continuation,
    })
}

pub(super) async fn search_page_with_ctgov_client(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<super::TrialSearchPage, BioMcpError> {
    if !matches!(filters.source, TrialSource::ClinicalTrialsGov) {
        return Err(BioMcpError::InvalidArgument(
            "internal ctgov search helper requires --source ctgov".into(),
        ));
    }

    validate_search_page_args(limit, offset, next_page.as_deref())?;
    let normalized = validate_trial_search(filters)?;
    let context = prepare_ctgov_search_context(&normalized)?;
    let condition_query = raw_condition_query(filters);
    let aliases = resolve_ctgov_intervention_aliases(filters).await?;

    if aliases.len() > 1 {
        if next_page
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
        {
            return Err(fanout_next_page_error());
        }
        return search_page_with_ctgov_union(
            client,
            filters,
            &context,
            condition_query,
            &aliases,
            limit,
            offset,
        )
        .await;
    }

    let single_worker = ctgov_workers(condition_query, &aliases)
        .into_iter()
        .next()
        .expect("single CTGov worker should exist");
    search_page_with_single_ctgov_intervention(
        client,
        filters,
        &context,
        &single_worker,
        limit,
        offset,
        next_page,
    )
    .await
}

async fn count_all_with_ctgov_union(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_aliases: &[TrialAlias],
    traversal_page_cap: usize,
) -> Result<ClinicalTrialSearchTotal, BioMcpError> {
    let mut workers = ctgov_workers(condition_query, intervention_aliases);
    let mut seen_nct_ids: HashSet<String> = HashSet::new();
    let mut unique_nct_ids: HashSet<String> = HashSet::new();
    let mut fetched_pages = 0usize;
    let mut degraded_coverage = false;
    let mut verification_incomplete = false;

    loop {
        let active_indices: Vec<usize> = workers
            .iter()
            .enumerate()
            .filter_map(|(index, worker)| (!worker.exhausted).then_some(index))
            .collect();
        if active_indices.is_empty() {
            return completed_ctgov_union_count(
                degraded_coverage,
                verification_incomplete,
                unique_nct_ids.len(),
            );
        }

        if fetched_pages.saturating_add(active_indices.len()) > traversal_page_cap {
            return final_ctgov_union_count(
                degraded_coverage,
                verification_incomplete,
                true,
                unique_nct_ids.len(),
            );
        }

        let pages = join_all(active_indices.iter().map(|index| {
            let worker = &workers[*index];
            fetch_ctgov_raw_page(
                client,
                filters,
                context,
                worker.condition_query.as_deref(),
                worker.intervention_query.as_deref(),
                worker.next_page_token.clone(),
                CTGOV_COUNT_PAGE_SIZE,
            )
        }))
        .await;
        fetched_pages = fetched_pages.saturating_add(active_indices.len());
        let mut round_studies = Vec::new();

        for (index, page_result) in active_indices.into_iter().zip(pages) {
            let Some(page) = handle_ctgov_worker_outcome(index, &workers[index], page_result)?
            else {
                workers[index].exhausted = true;
                degraded_coverage = true;
                continue;
            };
            let worker = &mut workers[index];
            worker.pages_fetched += 1;

            if page.raw_study_count == 0 {
                apply_worker_cursor(worker, page.provider_cursor);
                continue;
            }

            for study in page.studies {
                if claim_ctgov_candidate(&mut seen_nct_ids, &study).is_some() {
                    round_studies.push(study);
                }
            }

            apply_worker_cursor(worker, page.provider_cursor);
        }

        let outcome = apply_ctgov_post_filters(client, context, round_studies).await;
        verification_incomplete |= outcome.incomplete;
        add_unique_ctgov_nct_ids(&mut unique_nct_ids, outcome.studies);
    }
}

pub(super) async fn count_all_with_ctgov_client(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    traversal_page_cap: usize,
) -> Result<ClinicalTrialSearchTotal, BioMcpError> {
    if !matches!(filters.source, TrialSource::ClinicalTrialsGov) {
        return Err(BioMcpError::InvalidArgument(
            "internal ctgov count helper requires --source ctgov".into(),
        ));
    }

    let normalized = validate_trial_search(filters)?;
    let context = prepare_ctgov_search_context(&normalized)?;
    let condition_query = raw_condition_query(filters);
    let aliases = resolve_ctgov_intervention_aliases(filters).await?;

    if aliases.len() > 1 {
        return count_all_with_ctgov_union(
            client,
            filters,
            &context,
            condition_query,
            &aliases,
            traversal_page_cap,
        )
        .await;
    }

    if !context.uses_expensive_post_filters {
        let resp = client
            .search(&build_ctgov_search_plan(
                filters,
                &context,
                raw_condition_query(filters),
                raw_intervention_query(filters),
                None,
                1,
                true,
            )?)
            .await?;
        let total = resp
            .provider_total()
            .value()
            .and_then(|total| usize::try_from(total).ok());
        return ctgov_count_from_native_total(total, context.age_verification.is_some());
    }

    let mut verified_total = 0usize;
    let mut page_token: Option<String> = None;
    let mut page_count = 0usize;
    let mut verification_incomplete = false;
    let mut fallback_total: Option<usize> = None;

    loop {
        if page_count >= traversal_page_cap {
            return Ok(if verification_incomplete {
                ClinicalTrialSearchTotal::unknown(
                    ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
                )
            } else {
                ClinicalTrialSearchTotal::unknown(CTGOV_COUNT_CAP_REASON)
            });
        }

        let resp = client
            .search(&build_ctgov_search_plan(
                filters,
                &context,
                raw_condition_query(filters),
                raw_intervention_query(filters),
                page_token.clone(),
                CTGOV_COUNT_PAGE_SIZE,
                true,
            )?)
            .await?;
        page_count += 1;

        fallback_total = fallback_total.or_else(|| {
            resp.provider_total()
                .value()
                .and_then(|value| usize::try_from(value).ok())
        });
        let cursor = resp.provider_cursor().clone();
        let outcome = apply_ctgov_post_filters(
            client,
            &context,
            resp.results().unwrap_or_default().to_vec(),
        )
        .await;
        verification_incomplete |= outcome.incomplete;
        verified_total = verified_total.saturating_add(outcome.studies.len());

        match cursor {
            biodata::ClinicalTrialProviderCursor::Present(value) if !value.trim().is_empty() => {
                page_token = Some(value);
            }
            biodata::ClinicalTrialProviderCursor::Absent => break,
            biodata::ClinicalTrialProviderCursor::Present(_)
            | biodata::ClinicalTrialProviderCursor::Unavailable => {
                return Ok(if verification_incomplete {
                    ClinicalTrialSearchTotal::unknown(
                        ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
                    )
                } else if let Some(total) = fallback_total {
                    super::approximate_total(total)?
                } else {
                    ClinicalTrialSearchTotal::unknown(
                        ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
                    )
                });
            }
        }
    }

    if verification_incomplete {
        Ok(ClinicalTrialSearchTotal::unknown(
            ClinicalTrialSearchUnknownReason::IncompleteLocalVerification,
        ))
    } else {
        super::exact_total(verified_total)
    }
}
