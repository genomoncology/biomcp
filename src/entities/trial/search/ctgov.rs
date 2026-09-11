//! CTGov trial search query, pagination, and count helpers.

use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use tracing::warn;

use crate::entities::SearchPage;
use crate::entities::drug::{TrialAlias, TrialAliasSource, resolve_trial_aliases_with_sources};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;

use super::super::TrialCountUnknownReason;
use super::super::{TrialCount, TrialSearchFilters, TrialSearchResult, TrialSource};
use super::eligibility::ctgov_nct_id;
use super::{
    CtGovSearchContext, biodata_plan_error, prepare_ctgov_search_context,
    sort_trials_by_status_priority, validate_search_page_args, validate_trial_search,
    verify_age_eligibility, verify_detail_filters,
};

pub(super) const CTGOV_COUNT_PAGE_SIZE: usize = 1000;
const CTGOV_MAX_PAGE_FETCHES: usize = 20;
pub(super) const COUNT_TRAVERSAL_PAGE_CAP: usize = 50;

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
    mut studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) -> Vec<biodata::ClinicalTrialsGovApiV2SearchResult> {
    let facility_geo = context
        .facility_geo_verification
        .as_ref()
        .map(|(facility, lat, lon, distance)| (facility.as_str(), *lat, *lon, *distance));
    studies =
        verify_detail_filters(client, studies, facility_geo, &context.eligibility_keywords).await;
    if let Some(age) = context.age_verification {
        studies = verify_age_eligibility(studies, age);
    }
    studies
}

struct CtGovRawPage {
    total_count: Option<usize>,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    next_page_token: Option<String>,
    raw_study_count: usize,
}

impl std::fmt::Debug for CtGovRawPage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CtGovRawPage")
            .field("total_count", &self.total_count)
            .field("raw_study_count", &self.raw_study_count)
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
    exhausted: bool,
    pages_fetched: usize,
}

impl std::fmt::Debug for CtGovWorkerState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CtGovWorkerState")
            .field("exhausted", &self.exhausted)
            .field("pages_fetched", &self.pages_fetched)
            .finish_non_exhaustive()
    }
}

struct CtGovSinglePageState {
    rows: Vec<TrialSearchResult>,
    total: Option<usize>,
    verified_total: usize,
    exhausted: bool,
    page_token: Option<String>,
    remaining_skip: usize,
    requested_total: bool,
    started_with_cursor: bool,
}

impl CtGovSinglePageState {
    fn new(next_page: Option<String>, offset: usize, requested_total: bool) -> Self {
        Self {
            rows: Vec::new(),
            total: None,
            verified_total: 0,
            exhausted: false,
            page_token: next_page
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            remaining_skip: offset,
            requested_total,
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
        total_count: resp
            .total_count()
            .and_then(|value| usize::try_from(value).ok()),
        raw_study_count: resp.results().unwrap_or_default().len(),
        studies: resp.results().unwrap_or_default().to_vec(),
        next_page_token: resp
            .next_page_token()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned),
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
        page.studies = apply_ctgov_post_filters(client, context, page.studies).await;
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
    context: &CtGovSearchContext,
    worker: &CtGovWorkerState,
    limit: usize,
    page: CtGovRawPage,
) {
    if state.total.is_none() {
        state.total = page.total_count;
    }

    if page.raw_study_count == 0 {
        state.exhausted = true;
        return;
    }

    let next_page_token = page.next_page_token;
    let mut studies = page.studies;
    if context.uses_expensive_post_filters {
        state.verified_total = state.verified_total.saturating_add(studies.len());
    }

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
            let Ok(mut row) = TrialSearchResult::from_biodata(study.projection().value()) else {
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
            state.page_token = next_page_token.clone();
        } else {
            state.page_token = None;
        }
        if next_page_token.is_none() {
            state.exhausted = true;
        }
        return;
    }

    if page_started_with_skip > 0
        && state.remaining_skip == 0
        && state.rows.len() > rows_before_page
        && page_consumed >= page_study_count
    {
        state.page_token = next_page_token;
        if state.page_token.is_none() {
            state.exhausted = true;
        }
        return;
    }

    state.page_token = next_page_token;
    if state.page_token.is_none() {
        state.exhausted = true;
    }
}

fn finish_ctgov_single_page(
    mut state: CtGovSinglePageState,
    context: &CtGovSearchContext,
    limit: usize,
    offset: usize,
) -> SearchPage<TrialSearchResult> {
    if !context.has_explicit_status {
        sort_trials_by_status_priority(&mut state.rows);
    }

    state.rows.truncate(limit);
    let returned_total = if context.uses_expensive_post_filters {
        state.exhausted.then_some(state.verified_total)
    } else {
        state.total.or_else(|| {
            (!state.started_with_cursor && state.requested_total)
                .then(|| offset.saturating_add(state.rows.len()))
        })
    };

    SearchPage::cursor(state.rows, returned_total, state.page_token)
}

async fn search_page_with_single_ctgov_intervention(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    worker: &CtGovWorkerState,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
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
        if state.exhausted || state.rows.len() >= limit {
            break;
        }
    }

    Ok(finish_ctgov_single_page(state, context, limit, offset))
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
            exhausted: false,
            pages_fetched: 0,
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
            exhausted: false,
            pages_fetched: 0,
        })
        .collect()
}

fn claim_ctgov_candidate(
    seen_nct_ids: &mut HashSet<String>,
    study: &biodata::ClinicalTrialsGovApiV2SearchResult,
) -> Option<String> {
    match ctgov_nct_id(study) {
        Some(nct_id) => seen_nct_ids.insert(nct_id.clone()).then_some(nct_id),
        None => Some(String::new()),
    }
}

fn push_ctgov_union_rows(
    merged_rows: &mut Vec<TrialSearchResult>,
    merged_index: &mut HashMap<String, usize>,
    matched_labels: &HashMap<String, Option<String>>,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) {
    for study in studies {
        let matched_nct_id = ctgov_nct_id(&study).unwrap_or_default();
        let Ok(mut row) = TrialSearchResult::from_biodata(study.projection().value()) else {
            continue;
        };
        if merged_index.contains_key(&row.nct_id) {
            continue;
        }
        row.matched_intervention_label = matched_labels.get(&matched_nct_id).cloned().flatten();
        merged_index.insert(row.nct_id.clone(), merged_rows.len());
        merged_rows.push(row);
    }
}

fn add_unique_ctgov_nct_ids(
    unique_nct_ids: &mut HashSet<String>,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
) {
    for study in studies {
        if let Ok(row) = TrialSearchResult::from_biodata(study.projection().value()) {
            unique_nct_ids.insert(row.nct_id);
        }
    }
}

const COUNT_CAP_REASON: TrialCountUnknownReason = TrialCountUnknownReason::TraversalLimitReached;

fn ctgov_count_from_native_total(total: Option<usize>, has_age_filter: bool) -> TrialCount {
    match (total, has_age_filter) {
        (Some(total), true) => TrialCount::Approximate(total),
        (Some(total), false) => TrialCount::Exact(total),
        (None, _) => TrialCount::Unknown(TrialCountUnknownReason::ProviderOmittedTotal),
    }
}

fn ctgov_union_total(
    degraded_coverage: bool,
    traversal_capped: bool,
    workers: &[CtGovWorkerState],
    merged_row_count: usize,
) -> Option<usize> {
    if degraded_coverage
        || traversal_capped
        || workers
            .iter()
            .any(|worker| !worker.exhausted || worker.next_page_token.is_some())
    {
        None
    } else {
        Some(merged_row_count)
    }
}

fn completed_ctgov_union_count(degraded_coverage: bool, unique_count: usize) -> TrialCount {
    if degraded_coverage {
        TrialCount::Unknown(TrialCountUnknownReason::IncompleteCoverage)
    } else {
        TrialCount::Exact(unique_count)
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
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
    let page_size = offset.saturating_add(limit).clamp(1, 100);
    let mut workers = ctgov_workers(condition_query, intervention_aliases);
    let mut merged_rows: Vec<TrialSearchResult> = Vec::new();
    let mut merged_index: HashMap<String, usize> = HashMap::new();
    let mut seen_nct_ids: HashSet<String> = HashSet::new();
    let mut matched_labels: HashMap<String, Option<String>> = HashMap::new();
    let mut traversal_capped = false;
    let mut degraded_coverage = false;

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

            if page.raw_study_count == 0 {
                worker.exhausted = true;
                worker.next_page_token = page.next_page_token;
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

            worker.next_page_token = page.next_page_token;
            if worker.next_page_token.is_none() {
                worker.exhausted = true;
                continue;
            }
            if worker.pages_fetched >= CTGOV_MAX_PAGE_FETCHES {
                worker.exhausted = true;
                traversal_capped = true;
            }
        }

        let verified_studies = apply_ctgov_post_filters(client, context, round_studies).await;
        push_ctgov_union_rows(
            &mut merged_rows,
            &mut merged_index,
            &matched_labels,
            verified_studies,
        );

        if merged_rows.len() >= offset.saturating_add(limit) {
            break;
        }
    }

    if !context.has_explicit_status {
        sort_trials_by_status_priority(&mut merged_rows);
    }

    let total = ctgov_union_total(
        degraded_coverage,
        traversal_capped,
        &workers,
        merged_rows.len(),
    );

    let rows = merged_rows.into_iter().skip(offset).take(limit).collect();
    Ok(SearchPage::cursor(rows, total, None))
}

pub(super) async fn search_page_with_ctgov_client(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
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
) -> Result<TrialCount, BioMcpError> {
    let mut workers = ctgov_workers(condition_query, intervention_aliases);
    let mut seen_nct_ids: HashSet<String> = HashSet::new();
    let mut unique_nct_ids: HashSet<String> = HashSet::new();
    let mut fetched_pages = 0usize;
    let mut degraded_coverage = false;

    loop {
        let active_indices: Vec<usize> = workers
            .iter()
            .enumerate()
            .filter_map(|(index, worker)| (!worker.exhausted).then_some(index))
            .collect();
        if active_indices.is_empty() {
            return Ok(completed_ctgov_union_count(
                degraded_coverage,
                unique_nct_ids.len(),
            ));
        }

        if fetched_pages.saturating_add(active_indices.len()) > traversal_page_cap {
            return Ok(TrialCount::Unknown(COUNT_CAP_REASON));
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
                worker.exhausted = true;
                worker.next_page_token = page.next_page_token;
                continue;
            }

            for study in page.studies {
                if claim_ctgov_candidate(&mut seen_nct_ids, &study).is_some() {
                    round_studies.push(study);
                }
            }

            worker.next_page_token = page.next_page_token;
            if worker.next_page_token.is_none() {
                worker.exhausted = true;
            }
        }

        let verified_studies = apply_ctgov_post_filters(client, context, round_studies).await;
        add_unique_ctgov_nct_ids(&mut unique_nct_ids, verified_studies);
    }
}

pub(super) async fn count_all_with_ctgov_client(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    traversal_page_cap: usize,
) -> Result<TrialCount, BioMcpError> {
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
            .total_count()
            .and_then(|total| usize::try_from(total).ok());
        return Ok(ctgov_count_from_native_total(
            total,
            context.age_verification.is_some(),
        ));
    }

    let mut verified_total = 0usize;
    let mut page_token: Option<String> = None;
    let mut page_count = 0usize;

    loop {
        if page_count >= traversal_page_cap {
            return Ok(TrialCount::Unknown(COUNT_CAP_REASON));
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

        let next_page_token = resp
            .next_page_token()
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned);
        let studies = apply_ctgov_post_filters(
            client,
            &context,
            resp.results().unwrap_or_default().to_vec(),
        )
        .await;
        verified_total = verified_total.saturating_add(studies.len());

        if next_page_token.is_none() {
            break;
        }
        page_token = next_page_token;
    }

    Ok(TrialCount::Exact(verified_total))
}
