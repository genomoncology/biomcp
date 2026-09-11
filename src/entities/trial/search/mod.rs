//! Trial search and count entry points exposed through the stable trial facade.

mod ctgov;
mod eligibility;
mod nci;
mod normalization;
#[cfg(test)]
mod plan_tests;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::mydisease::MyDiseaseClient;
use crate::sources::nci_cts::NciCtsClient;

use self::ctgov::{count_all_with_ctgov_client, search_page_with_ctgov_client};
use self::eligibility::{
    collect_eligibility_keywords, verify_age_eligibility, verify_detail_filters,
};
use self::nci::search_page_with_nci_clients;
use self::normalization::sort_trials_by_status_priority;

use super::{TrialCount, TrialSearchFilters, TrialSearchResult, TrialSource};

pub(super) struct NormalizedTrialSearch {
    pub(super) biodata: biodata::ClinicalTrialSearchFilters,
}

pub(super) struct CtGovSearchContext {
    pub(super) eligibility_keywords: Vec<String>,
    pub(super) facility_geo_verification: Option<(String, f64, f64, u32)>,
    pub(super) age_verification: Option<f64>,
    pub(super) uses_expensive_post_filters: bool,
    pub(super) has_explicit_status: bool,
}

pub(super) fn biodata_filter_fields(
    filters: &TrialSearchFilters,
    intervention: Option<&str>,
) -> biodata::ClinicalTrialSearchFilterFields {
    biodata::ClinicalTrialSearchFilterFields {
        condition: filters.condition.clone(),
        intervention: intervention.map(str::to_owned),
        facility: filters.facility.clone(),
        status: filters.status.clone(),
        phase: filters.phase.clone(),
        study_type: filters.study_type.clone(),
        age: filters.age,
        sex: filters.sex.clone(),
        sponsor: filters.sponsor.clone(),
        sponsor_type: filters.sponsor_type.clone(),
        date_from: filters.date_from.clone(),
        date_to: filters.date_to.clone(),
        mutation: filters.mutation.clone(),
        criteria: filters.criteria.clone(),
        biomarker: filters.biomarker.clone(),
        prior_therapies: filters.prior_therapies.clone(),
        progression_on: filters.progression_on.clone(),
        line_of_therapy: filters.line_of_therapy.clone(),
        results_available: filters.results_available,
        latitude: filters.lat,
        longitude: filters.lon,
        distance_miles: filters.distance,
    }
}

pub(super) fn biodata_filters(
    filters: &TrialSearchFilters,
    intervention: Option<&str>,
) -> Result<biodata::ClinicalTrialSearchFilters, BioMcpError> {
    biodata::ClinicalTrialSearchFilters::new(
        biodata_filter_fields(filters, intervention),
        Default::default(),
    )
    .map_err(|error| biodata_plan_error("invalid trial search", error))
}

pub(super) fn biodata_plan_error(
    context: &str,
    error: biodata::ClinicalTrialSearchPlanError,
) -> BioMcpError {
    let correction = error
        .correction()
        .map(|value| format!("; {value}"))
        .unwrap_or_default();
    BioMcpError::InvalidArgument(format!(
        "{context} {}: {}{correction}",
        error.field(),
        error.code()
    ))
}

// This exceeds practical single-condition result sets while bounding abusive provider requests.
const MAX_SEARCH_OFFSET: usize = 100_000;

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
    if offset > MAX_SEARCH_OFFSET {
        return Err(BioMcpError::InvalidArgument(format!(
            "--offset must be at most {MAX_SEARCH_OFFSET} for trial search"
        )));
    }
    Ok(())
}

pub(super) fn validate_trial_search(
    filters: &TrialSearchFilters,
) -> Result<NormalizedTrialSearch, BioMcpError> {
    let biodata = biodata_filters(filters, filters.intervention.as_deref())?;
    if biodata.is_empty() {
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

    if matches!(filters.source, TrialSource::NciCts) {
        biodata::NciCtsV2SearchPlan::new(&biodata, None, 1, 0)
            .map_err(|error| biodata_plan_error("invalid NCI trial search", error))?;
    }
    Ok(NormalizedTrialSearch { biodata })
}

pub(super) fn prepare_ctgov_search_context(
    normalized: &NormalizedTrialSearch,
) -> Result<CtGovSearchContext, BioMcpError> {
    let facility = normalized.biodata.facility().map(str::to_owned);
    let eligibility_keywords = collect_eligibility_keywords(&normalized.biodata);
    let has_explicit_status = normalized.biodata.has_status();
    let facility_geo_verification = facility.as_deref().zip(normalized.biodata.geography()).map(
        |(facility_name, (lat, lon, distance))| (facility_name.to_string(), lat, lon, distance),
    );
    let uses_expensive_post_filters =
        facility_geo_verification.is_some() || !eligibility_keywords.is_empty();

    Ok(CtGovSearchContext {
        eligibility_keywords,
        facility_geo_verification,
        age_verification: normalized.biodata.age(),
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
    validate_trial_search(filters)?;
    match filters.source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            count_all_with_ctgov_client(&client, filters, ctgov::COUNT_TRAVERSAL_PAGE_CAP).await
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
    validate_trial_search(filters)?;
    validate_search_page_args(limit, offset, next_page.as_deref())?;
    match filters.source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            search_page_with_ctgov_client(&client, filters, limit, offset, next_page).await
        }
        TrialSource::NciCts => {
            let normalized = validate_trial_search(filters)?;

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
