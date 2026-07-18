//! CTGov trial search query, pagination, and count helpers.

use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use tracing::warn;

use crate::entities::SearchPage;
use crate::entities::drug::{TrialAlias, TrialAliasSource, resolve_trial_aliases_with_sources};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::{ClinicalTrialsClient, CtGovSearchParams, CtGovStudy};
use crate::transform;
use crate::utils::date::validate_since;

use super::super::{TrialCount, TrialSearchFilters, TrialSearchResult, TrialSource};
use super::{
    CtGovSearchContext, build_essie_fragments, essie_escape, essie_escape_boolean_expression,
    normalize_sex, normalize_sponsor_type, prepare_ctgov_search_context, quote_essie_literal,
    sort_trials_by_status_priority, validate_search_page_args, validate_trial_search,
    verify_age_eligibility, verify_detail_filters,
};

pub(super) const CTGOV_COUNT_PAGE_SIZE: usize = 1000;
const CTGOV_MAX_PAGE_FETCHES: usize = 20;
const COUNT_TRAVERSAL_PAGE_CAP: usize = 50;

pub(super) fn ctgov_agg_filters(
    filters: &TrialSearchFilters,
) -> Result<Option<String>, BioMcpError> {
    let mut facets: Vec<String> = Vec::new();

    if let Some(sex) = filters
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        && let Some(code) = normalize_sex(sex)?
    {
        facets.push(format!("sex:{code}"));
    }

    if let Some(sponsor_type) = filters
        .sponsor_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        facets.push(format!(
            "funderType:{}",
            normalize_sponsor_type(sponsor_type)?
        ));
    }

    if facets.is_empty() {
        Ok(None)
    } else {
        Ok(Some(facets.join(",")))
    }
}

pub(super) fn validate_location(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    let has_lat = filters.lat.is_some();
    let has_lon = filters.lon.is_some();
    let has_distance = filters.distance.is_some();

    if has_distance && (!has_lat || !has_lon) {
        return Err(BioMcpError::InvalidArgument(
            "--distance requires both --lat and --lon".into(),
        ));
    }
    if (has_lat || has_lon) && !has_distance {
        return Err(BioMcpError::InvalidArgument(
            "--lat/--lon requires --distance".into(),
        ));
    }
    if has_lat != has_lon {
        return Err(BioMcpError::InvalidArgument(
            "--lat and --lon must be provided together".into(),
        ));
    }
    Ok(())
}

pub(super) fn ctgov_query_term(
    filters: &TrialSearchFilters,
    normalized_phase: Option<&[String]>,
) -> Result<Option<String>, BioMcpError> {
    let mut terms: Vec<String> = Vec::new();

    if let Some(phases) = normalized_phase {
        if phases.len() == 1 {
            terms.push(format!("AREA[Phase]{}", phases[0]));
        } else if !phases.is_empty() {
            let inner = phases
                .iter()
                .map(|phase| format!("AREA[Phase]{phase}"))
                .collect::<Vec<_>>()
                .join(" AND ");
            terms.push(format!("({inner})"));
        }
    }
    if let Some(sponsor) = filters
        .sponsor
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let sponsor = essie_escape(sponsor);
        terms.push(format!("AREA[LeadSponsorName]\"{sponsor}\""));
    }
    if let Some(mutation) = filters
        .mutation
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let mutation = essie_escape_boolean_expression(mutation);
        terms.push(format!(
            "(AREA[EligibilityCriteria]({mutation}) OR AREA[BriefTitle]({mutation}) \
             OR AREA[OfficialTitle]({mutation}) OR AREA[BriefSummary]({mutation}) \
             OR AREA[Keyword]({mutation}))"
        ));
    }
    if let Some(criteria) = filters
        .criteria
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let criteria = essie_escape_boolean_expression(criteria);
        terms.push(format!("AREA[EligibilityCriteria]({criteria})"));
    }
    if let Some(biomarker) = filters
        .biomarker
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let biomarker = essie_escape(biomarker);
        terms.push(format!(
            "(AREA[Keyword]\"{biomarker}\" OR AREA[InterventionName]\"{biomarker}\" OR AREA[Condition]\"{biomarker}\")"
        ));
    }
    if let Some(study_type) = filters
        .study_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let study_type = essie_escape(study_type);
        terms.push(format!("AREA[StudyType]\"{study_type}\""));
    }
    terms.extend(build_essie_fragments(filters)?);
    if let Some(date_from) = filters
        .date_from
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let date_from = validate_since(date_from)?;
        let date_to = filters
            .date_to
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(validate_since)
            .transpose()?;
        if let Some(date_to) = date_to.as_deref() {
            if date_from.as_str() > date_to {
                return Err(BioMcpError::InvalidArgument(
                    "--date-from must be <= --date-to".into(),
                ));
            }
            terms.push(format!(
                "AREA[LastUpdatePostDate]RANGE[{date_from},{date_to}]"
            ));
        } else {
            terms.push(format!("AREA[LastUpdatePostDate]RANGE[{date_from},MAX]"));
        }
    } else if let Some(date_to) = filters
        .date_to
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let date_to = validate_since(date_to)?;
        terms.push(format!("AREA[LastUpdatePostDate]RANGE[MIN,{date_to}]"));
    }
    if filters.results_available {
        terms.push("AREA[ResultsFirstPostDate]RANGE[MIN,MAX]".to_string());
    }
    if terms.is_empty() {
        Ok(None)
    } else {
        Ok(Some(terms.join(" AND ")))
    }
}

fn build_ctgov_search_params(
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_query: Option<&str>,
    page_token: Option<String>,
    page_size: usize,
    count_total: bool,
) -> CtGovSearchParams {
    CtGovSearchParams {
        condition: condition_query.map(str::to_string),
        intervention: intervention_query.map(str::trim).map(quote_essie_literal),
        facility: context.facility.clone(),
        status: context.normalized_status.clone(),
        agg_filters: context.agg_filters.clone(),
        query_term: context.query_term.clone(),
        fields_override: None,
        count_total,
        page_token,
        page_size,
        lat: filters.lat,
        lon: filters.lon,
        distance_miles: filters.distance,
    }
}

async fn apply_ctgov_post_filters(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    mut studies: Vec<CtGovStudy>,
) -> Vec<CtGovStudy> {
    let facility_geo = context
        .facility_geo_verification
        .as_ref()
        .map(|(facility, lat, lon, distance)| (facility.as_str(), *lat, *lon, *distance));
    studies =
        verify_detail_filters(client, studies, facility_geo, &context.eligibility_keywords).await;
    if let Some(age) = filters.age {
        studies = verify_age_eligibility(studies, age);
    }
    studies
}

#[derive(Debug)]
struct CtGovFilteredPage {
    total_count: Option<usize>,
    studies: Vec<CtGovStudy>,
    next_page_token: Option<String>,
    raw_study_count: usize,
}

#[derive(Debug, Clone)]
struct CtGovWorkerState {
    condition_query: Option<String>,
    intervention_query: Option<String>,
    intervention_source: &'static str,
    matched_intervention_label: Option<String>,
    next_page_token: Option<String>,
    exhausted: bool,
    pages_fetched: usize,
}

struct CtGovSinglePageState {
    rows: Vec<TrialSearchResult>,
    total: Option<usize>,
    verified_total: usize,
    exhausted: bool,
    page_token: Option<String>,
    remaining_skip: usize,
    requested_total: bool,
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

async fn fetch_ctgov_filtered_page(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
    context: &CtGovSearchContext,
    condition_query: Option<&str>,
    intervention_query: Option<&str>,
    page_token: Option<String>,
    page_size: usize,
) -> Result<CtGovFilteredPage, BioMcpError> {
    let count_total = !filters.no_count_total;
    let resp = client
        .search(&build_ctgov_search_params(
            filters,
            context,
            condition_query,
            intervention_query,
            page_token,
            page_size,
            count_total,
        ))
        .await?;

    let raw_study_count = resp.studies.len();
    let studies = if raw_study_count == 0 {
        Vec::new()
    } else {
        apply_ctgov_post_filters(client, filters, context, resp.studies).await
    };

    Ok(CtGovFilteredPage {
        total_count: resp.total_count.map(|value| value as usize),
        studies,
        next_page_token: resp.next_page_token,
        raw_study_count,
    })
}

fn handle_ctgov_worker_outcome(
    worker_index: usize,
    worker: &CtGovWorkerState,
    result: Result<CtGovFilteredPage, BioMcpError>,
) -> Result<Option<CtGovFilteredPage>, BioMcpError> {
    match result {
        Ok(page) => Ok(Some(page)),
        Err(BioMcpError::WithSourceContext { context, source }) => {
            handle_ctgov_worker_outcome(worker_index, worker, Err(*source))
                .map_err(|error| error.with_source_context(context))
        }
        Err(BioMcpError::CtGovInterventionQueryRejected { reason }) if worker_index > 0 => {
            warn!(
                alias = worker.intervention_query.as_deref().unwrap_or_default(),
                worker_index,
                source = worker.intervention_source,
                reason,
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
    page: CtGovFilteredPage,
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
            let mut row = transform::trial::from_ctgov_hit(&study);
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
            state
                .requested_total
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

fn push_ctgov_union_rows(
    merged_rows: &mut Vec<TrialSearchResult>,
    merged_index: &mut HashMap<String, usize>,
    worker: &CtGovWorkerState,
    studies: Vec<CtGovStudy>,
) {
    for study in studies {
        let mut row = transform::trial::from_ctgov_hit(&study);
        if merged_index.contains_key(&row.nct_id) {
            continue;
        }
        row.matched_intervention_label = worker.matched_intervention_label.clone();
        merged_index.insert(row.nct_id.clone(), merged_rows.len());
        merged_rows.push(row);
    }
}

fn add_unique_ctgov_nct_ids(unique_nct_ids: &mut HashSet<String>, studies: Vec<CtGovStudy>) {
    for study in studies {
        let row = transform::trial::from_ctgov_hit(&study);
        unique_nct_ids.insert(row.nct_id);
    }
}

fn ctgov_count_page_cap_would_be_exceeded(fetched_pages: usize, active_workers: usize) -> bool {
    fetched_pages.saturating_add(active_workers) > COUNT_TRAVERSAL_PAGE_CAP
}

fn ctgov_single_count_page_cap_reached(page_count: usize) -> bool {
    page_count >= COUNT_TRAVERSAL_PAGE_CAP
}

fn ctgov_count_from_native_total(total: usize, has_age_filter: bool) -> TrialCount {
    if has_age_filter {
        TrialCount::Approximate(total)
    } else {
        TrialCount::Exact(total)
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
        TrialCount::Unknown
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
            fetch_ctgov_filtered_page(
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

            push_ctgov_union_rows(&mut merged_rows, &mut merged_index, worker, page.studies);

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
    let context = prepare_ctgov_search_context(filters, &normalized)?;
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
) -> Result<TrialCount, BioMcpError> {
    let mut workers = ctgov_workers(condition_query, intervention_aliases);
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

        if ctgov_count_page_cap_would_be_exceeded(fetched_pages, active_indices.len()) {
            return Ok(TrialCount::Unknown);
        }

        let pages = join_all(active_indices.iter().map(|index| {
            let worker = &workers[*index];
            fetch_ctgov_filtered_page(
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

            add_unique_ctgov_nct_ids(&mut unique_nct_ids, page.studies);

            worker.next_page_token = page.next_page_token;
            if worker.next_page_token.is_none() {
                worker.exhausted = true;
            }
        }
    }
}

pub(super) async fn count_all_with_ctgov_client(
    client: &ClinicalTrialsClient,
    filters: &TrialSearchFilters,
) -> Result<TrialCount, BioMcpError> {
    if !matches!(filters.source, TrialSource::ClinicalTrialsGov) {
        return Err(BioMcpError::InvalidArgument(
            "internal ctgov count helper requires --source ctgov".into(),
        ));
    }

    let normalized = validate_trial_search(filters)?;
    let context = prepare_ctgov_search_context(filters, &normalized)?;
    let condition_query = raw_condition_query(filters);
    let aliases = resolve_ctgov_intervention_aliases(filters).await?;

    if aliases.len() > 1 {
        return count_all_with_ctgov_union(client, filters, &context, condition_query, &aliases)
            .await;
    }

    if !context.uses_expensive_post_filters {
        let resp = client
            .search(&build_ctgov_search_params(
                filters,
                &context,
                raw_condition_query(filters),
                raw_intervention_query(filters),
                None,
                1,
                true,
            ))
            .await?;
        let total = resp.total_count.unwrap_or(0) as usize;
        return Ok(ctgov_count_from_native_total(total, filters.age.is_some()));
    }

    let mut verified_total = 0usize;
    let mut page_token: Option<String> = None;
    let mut page_count = 0usize;

    loop {
        if ctgov_single_count_page_cap_reached(page_count) {
            return Ok(TrialCount::Unknown);
        }

        let resp = client
            .search(&build_ctgov_search_params(
                filters,
                &context,
                raw_condition_query(filters),
                raw_intervention_query(filters),
                page_token.clone(),
                CTGOV_COUNT_PAGE_SIZE,
                true,
            ))
            .await?;
        page_count += 1;

        let next_page_token = resp.next_page_token;
        let studies = apply_ctgov_post_filters(client, filters, &context, resp.studies).await;
        verified_total = verified_total.saturating_add(studies.len());

        if next_page_token.is_none() {
            break;
        }
        page_token = next_page_token;
    }

    Ok(TrialCount::Exact(verified_total))
}

#[cfg(test)]
mod tests;
