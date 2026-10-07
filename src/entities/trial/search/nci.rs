//! NCI CTS trial search helpers.

use crate::entities::SearchPage;
use crate::entities::disease::resolve_disease_hit_by_name;
use crate::error::BioMcpError;
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit};
use crate::sources::nci_cts::{
    NciCtsClient, NciDiseaseFilter, NciGeoFilter, NciSearchParams, NciStatusFilter,
};
use crate::transform;
use tracing::warn;

use super::super::{TrialSearchFilters, TrialSearchResult};
use super::{NormalizedTrialSearch, nci_biomarker_value, normalized_facility_filter};

/// The NCI condition filter plus the note to surface when grounding
/// degraded from an NCI concept ID to a plain keyword search. The note
/// reaches the response (`meta.notes` and the markdown footer), not only a
/// log line — the ticket 2021 pattern for visible degrades.
struct NciDiseaseGrounding {
    filter: NciDiseaseFilter,
    degrade_note: Option<String>,
}

fn nci_keyword_degrade_note(condition: &str, reason: &str) -> String {
    format!(
        "NCI trial search used a plain keyword search for '{condition}' because {reason}; \
results may be broader than the requested condition."
    )
}

async fn resolve_nci_disease_filter_with_client(
    client: &MyDiseaseClient,
    condition: Option<&str>,
) -> Result<Option<NciDiseaseGrounding>, BioMcpError> {
    let Some(condition) = condition.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    match resolve_disease_hit_by_name(client, condition).await {
        Ok(hit) => Ok(Some(nci_disease_grounding_from_hit(condition, hit))),
        Err(BioMcpError::NotFound { .. }) => Ok(Some(NciDiseaseGrounding {
            filter: NciDiseaseFilter::Keyword(condition.to_string()),
            degrade_note: Some(nci_keyword_degrade_note(
                condition,
                "it does not ground to a disease concept in MyDisease",
            )),
        })),
        Err(err) => {
            warn!(
                condition,
                error = %err,
                "NCI disease grounding failed, falling back to keyword"
            );
            let reason_text = err.to_string();
            let reason = reason_text
                .lines()
                .next()
                .unwrap_or("disease grounding failed");
            Ok(Some(NciDiseaseGrounding {
                filter: NciDiseaseFilter::Keyword(condition.to_string()),
                degrade_note: Some(nci_keyword_degrade_note(
                    condition,
                    &format!("disease grounding failed ({reason})"),
                )),
            }))
        }
    }
}

fn nci_disease_grounding_from_hit(condition: &str, hit: MyDiseaseHit) -> NciDiseaseGrounding {
    let mut disease = transform::disease::from_mydisease_hit(hit);
    if let Some(nci_id) = disease
        .xrefs
        .remove("NCI")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        NciDiseaseGrounding {
            filter: NciDiseaseFilter::ConceptId(nci_id),
            degrade_note: None,
        }
    } else {
        NciDiseaseGrounding {
            filter: NciDiseaseFilter::Keyword(condition.to_string()),
            degrade_note: Some(nci_keyword_degrade_note(
                condition,
                "the resolved disease carries no NCI concept ID",
            )),
        }
    }
}

pub(super) async fn search_page_with_nci_clients(
    client: &NciCtsClient,
    mydisease_client: &MyDiseaseClient,
    filters: &TrialSearchFilters,
    normalized: &NormalizedTrialSearch,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
    let grounding =
        resolve_nci_disease_filter_with_client(mydisease_client, filters.condition.as_deref())
            .await?;
    let degrade_note = grounding.as_ref().and_then(|g| g.degrade_note.clone());
    let params = NciSearchParams {
        disease: grounding.map(|g| g.filter),
        interventions: filters.intervention.clone(),
        sites_org_name: normalized_facility_filter(filters),
        status: nci_status_filter(normalized.normalized_status.as_deref())?,
        phases: nci_phase_filters(normalized.normalized_phase.as_deref())?,
        geo: nci_geo_filter(filters),
        biomarkers: nci_biomarker_value(filters)?,
        size: limit,
        from: offset,
    };

    let resp = client.search(&params).await?;
    let mut page = SearchPage::offset(
        resp.hits()
            .iter()
            .map(transform::trial::from_nci_hit)
            .collect::<Result<Vec<_>, _>>()?,
        resp.total,
    );
    page.partial_note = degrade_note;
    Ok(page)
}

fn nci_status_filter(value: Option<&str>) -> Result<Option<NciStatusFilter>, BioMcpError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if value.contains(',') {
        return Err(BioMcpError::InvalidArgument(
            "--status accepts one mapped status at a time for --source nci; comma-separated status lists are not supported".into(),
        ));
    }

    let filter = match value {
        "RECRUITING" => NciStatusFilter::SiteRecruitmentStatus("ACTIVE".into()),
        "NOT_YET_RECRUITING" => NciStatusFilter::CurrentTrialStatus("Approved".into()),
        "ENROLLING_BY_INVITATION" => {
            NciStatusFilter::CurrentTrialStatus("Enrolling by Invitation".into())
        }
        "ACTIVE_NOT_RECRUITING" => {
            NciStatusFilter::SiteRecruitmentStatus("CLOSED_TO_ACCRUAL".into())
        }
        "COMPLETED" => NciStatusFilter::CurrentTrialStatus("Complete".into()),
        "SUSPENDED" => NciStatusFilter::CurrentTrialStatus("Temporarily Closed to Accrual".into()),
        "TERMINATED" => NciStatusFilter::CurrentTrialStatus("Administratively Complete".into()),
        "WITHDRAWN" => NciStatusFilter::CurrentTrialStatus("Withdrawn".into()),
        other => {
            return Err(BioMcpError::InvalidArgument(format!(
                "--status {other} is not supported for --source nci"
            )));
        }
    };

    Ok(Some(filter))
}

fn nci_phase_filters(value: Option<&[String]>) -> Result<Vec<String>, BioMcpError> {
    let Some(phases) = value else {
        return Ok(Vec::new());
    };
    if phases == ["PHASE1", "PHASE2"] {
        return Ok(vec!["I_II".to_string()]);
    }
    if phases == ["PHASE2", "PHASE3"] {
        return Ok(vec!["II_III".to_string()]);
    }

    phases
        .iter()
        .map(|phase| match phase.as_str() {
            "PHASE1" => Ok("I".to_string()),
            "PHASE2" => Ok("II".to_string()),
            "PHASE3" => Ok("III".to_string()),
            "PHASE4" => Ok("IV".to_string()),
            "NA" => Ok("NA".to_string()),
            "EARLY_PHASE1" => Err(BioMcpError::InvalidArgument(
                "--phase early_phase1 is not supported for --source nci".into(),
            )),
            other => Err(BioMcpError::InvalidArgument(format!(
                "--phase {other} is not supported for --source nci"
            ))),
        })
        .collect()
}

fn nci_geo_filter(filters: &TrialSearchFilters) -> Option<NciGeoFilter> {
    let (Some(lat), Some(lon), Some(distance_miles)) = (filters.lat, filters.lon, filters.distance)
    else {
        return None;
    };
    Some(NciGeoFilter {
        lat,
        lon,
        distance_miles,
    })
}

#[cfg(test)]
mod tests;
