//! Trial search and count entry points exposed through the stable trial facade.

mod ctgov;
mod eligibility;
mod essie;
mod nci;
mod normalization;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::mydisease::MyDiseaseClient;
use crate::sources::nci_cts::NciCtsClient;

use self::ctgov::validate_location;
use self::ctgov::{
    count_all_with_ctgov_client, ctgov_agg_filters, ctgov_query_term, search_page_with_ctgov_client,
};
use self::eligibility::{
    collect_eligibility_keywords, verify_age_eligibility, verify_eligibility_criteria,
    verify_facility_geo,
};
use self::essie::has_essie_filters;
use self::essie::{
    build_essie_fragments, essie_escape, essie_escape_boolean_expression, has_boolean_operators,
};
use self::nci::search_page_with_nci_clients;
use self::normalization::{
    normalize_intervention_query, normalize_sex, normalize_sponsor_type,
    normalized_facility_filter, normalized_phase_filter, normalized_status_filter,
    sort_trials_by_status_priority,
};

use super::{TrialCount, TrialSearchFilters, TrialSearchResult, TrialSource};

pub(super) struct NormalizedTrialSearch {
    pub(super) normalized_status: Option<String>,
    pub(super) normalized_phase: Option<Vec<String>>,
}

pub(super) struct CtGovSearchContext {
    pub(super) normalized_status: Option<String>,
    pub(super) query_term: Option<String>,
    pub(super) facility: Option<String>,
    pub(super) agg_filters: Option<String>,
    pub(super) eligibility_keywords: Vec<String>,
    pub(super) facility_geo_verification: Option<(String, f64, f64, u32)>,
    pub(super) uses_expensive_post_filters: bool,
    pub(super) has_explicit_status: bool,
}

fn has_any_query(filters: &TrialSearchFilters) -> bool {
    filters
        .condition
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .intervention
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .facility
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .mutation
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .criteria
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .biomarker
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .prior_therapies
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .progression_on
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .line_of_therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .sponsor
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .status
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .phase
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .study_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.age.is_some()
        || filters
            .sex
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .sponsor_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .date_from
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .date_to
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.results_available
        || filters.distance.is_some()
}

pub(super) fn validate_search_page_args(
    limit: usize,
    offset: usize,
    next_page: Option<&str>,
) -> Result<(), BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    if next_page
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && offset > 0
    {
        return Err(BioMcpError::InvalidArgument(
            "--next-page cannot be used together with --offset".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_trial_search(
    filters: &TrialSearchFilters,
) -> Result<NormalizedTrialSearch, BioMcpError> {
    if !has_any_query(filters) {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search trial -c melanoma".into(),
        ));
    }
    let has_intervention = filters
        .intervention
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());
    if filters.no_alias_expand
        && (!has_intervention || !matches!(filters.source, TrialSource::ClinicalTrialsGov))
    {
        return Err(BioMcpError::InvalidArgument(
            "--no-alias-expand is only supported for CTGov intervention searches".into(),
        ));
    }

    let normalized_status = normalized_status_filter(filters)?;
    let normalized_phase = normalized_phase_filter(filters)?;
    validate_location(filters)?;

    if matches!(filters.source, TrialSource::NciCts)
        && normalized_status
            .as_deref()
            .is_some_and(|value| value.contains(','))
    {
        return Err(BioMcpError::InvalidArgument(
            "--status accepts one mapped status at a time for --source nci; comma-separated status lists are not supported".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts)
        && normalized_phase
            .as_ref()
            .is_some_and(|phases| phases.iter().any(|phase| phase == "EARLY_PHASE1"))
    {
        return Err(BioMcpError::InvalidArgument(
            "--phase early_phase1 is not supported for --source nci".into(),
        ));
    }

    if matches!(filters.source, TrialSource::NciCts) && has_essie_filters(filters) {
        return Err(BioMcpError::InvalidArgument(
            "--prior-therapies, --progression-on, and --line-of-therapy are only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts) && filters.results_available {
        return Err(BioMcpError::InvalidArgument(
            "--results-available is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts) && filters.age.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "--age is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts)
        && filters
            .sex
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
    {
        return Err(BioMcpError::InvalidArgument(
            "--sex is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts)
        && filters
            .sponsor_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
    {
        return Err(BioMcpError::InvalidArgument(
            "--sponsor-type is only supported for --source ctgov".into(),
        ));
    }

    Ok(NormalizedTrialSearch {
        normalized_status,
        normalized_phase,
    })
}

pub(super) fn prepare_ctgov_search_context(
    filters: &TrialSearchFilters,
    normalized: &NormalizedTrialSearch,
) -> Result<CtGovSearchContext, BioMcpError> {
    let query_term = ctgov_query_term(filters, normalized.normalized_phase.as_deref())?;
    let facility = normalized_facility_filter(filters);
    let eligibility_keywords = collect_eligibility_keywords(filters);
    let agg_filters = ctgov_agg_filters(filters)?;
    let has_explicit_status = filters
        .status
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty());
    let facility_geo_verification = facility
        .as_deref()
        .zip(filters.lat)
        .zip(filters.lon)
        .zip(filters.distance)
        .map(|(((facility_name, lat), lon), distance)| {
            (facility_name.to_string(), lat, lon, distance)
        });
    let uses_expensive_post_filters =
        facility_geo_verification.is_some() || !eligibility_keywords.is_empty();

    Ok(CtGovSearchContext {
        normalized_status: normalized.normalized_status.clone(),
        query_term,
        facility,
        agg_filters,
        eligibility_keywords,
        facility_geo_verification,
        uses_expensive_post_filters,
        has_explicit_status,
    })
}

pub async fn search(
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<(Vec<TrialSearchResult>, Option<u32>), BioMcpError> {
    let page = search_page(filters, limit, offset, None).await?;
    Ok((page.results, page.total.map(|v| v as u32)))
}

pub async fn count_all(filters: &TrialSearchFilters) -> Result<TrialCount, BioMcpError> {
    match filters.source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            count_all_with_ctgov_client(&client, filters).await
        }
        TrialSource::NciCts => {
            let page = search_page(filters, 1, 0, None).await?;
            Ok(TrialCount::Exact(page.total.unwrap_or(page.results.len())))
        }
    }
}

pub async fn search_page(
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
    match filters.source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            search_page_with_ctgov_client(&client, filters, limit, offset, next_page).await
        }
        TrialSource::NciCts => {
            validate_search_page_args(limit, offset, next_page.as_deref())?;
            let normalized = validate_trial_search(filters)?;

            if filters.date_from.is_some() || filters.date_to.is_some() {
                return Err(BioMcpError::InvalidArgument(
                    "--date-from/--date-to is only supported for --source ctgov".into(),
                ));
            }
            if next_page
                .as_deref()
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
            {
                return Err(BioMcpError::InvalidArgument(
                    "--next-page is only supported for --source ctgov".into(),
                ));
            }
            let client = NciCtsClient::new()?;
            let mydisease_client = MyDiseaseClient::new()?;
            search_page_with_nci_clients(
                &client,
                &mydisease_client,
                filters,
                &normalized,
                limit,
                offset,
            )
            .await
        }
    }
}
