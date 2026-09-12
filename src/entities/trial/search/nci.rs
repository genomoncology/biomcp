//! NCI CTS trial search orchestration. BioData owns request grammar.
use super::super::{TrialSearchFilters, TrialSearchHit};
use super::{NormalizedTrialSearch, biodata_plan_error};
use crate::entities::disease::resolve_disease_hit_by_name;
use crate::error::BioMcpError;
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit};
use crate::sources::nci_cts::NciCtsClient;
use crate::transform;
use tracing::warn;

async fn resolve_disease(
    client: &MyDiseaseClient,
    condition: Option<&str>,
) -> Result<Option<biodata::NciCtsV2DiseaseSelection>, BioMcpError> {
    let Some(condition) = condition.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    match resolve_disease_hit_by_name(client, condition).await {
        Ok(hit) => Ok(Some(from_hit(condition, hit))),
        Err(BioMcpError::NotFound { .. }) => Ok(Some(biodata::NciCtsV2DiseaseSelection::Keyword(
            condition.to_owned(),
        ))),
        Err(error) => {
            warn!(error=%error,"NCI disease grounding failed; using the keyword fallback");
            Ok(Some(biodata::NciCtsV2DiseaseSelection::Keyword(
                condition.to_owned(),
            )))
        }
    }
}
fn from_hit(condition: &str, hit: MyDiseaseHit) -> biodata::NciCtsV2DiseaseSelection {
    let mut disease = transform::disease::from_mydisease_hit(hit);
    disease
        .xrefs
        .remove("NCI")
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .map_or_else(
            || biodata::NciCtsV2DiseaseSelection::Keyword(condition.to_owned()),
            biodata::NciCtsV2DiseaseSelection::ConceptId,
        )
}

pub(super) async fn search_page_with_nci_clients(
    client: &NciCtsClient,
    mydisease: &MyDiseaseClient,
    filters: &TrialSearchFilters,
    normalized: &NormalizedTrialSearch,
    limit: usize,
    offset: usize,
) -> Result<super::TrialSearchPage, BioMcpError> {
    let disease = resolve_disease(mydisease, filters.condition.as_deref()).await?;
    let plan = biodata::NciCtsV2SearchPlan::new(&normalized.biodata, disease, limit, offset)
        .map_err(|error| biodata_plan_error("invalid NCI trial search", error))?;
    let response = client.search(&plan).await?;
    let results = response
        .results()
        .unwrap_or_default()
        .iter()
        .cloned()
        .map(|result| TrialSearchHit::from_biodata(result.into_projection()))
        .collect::<Result<Vec<_>, _>>()?;
    let (total, continuation) =
        nci_search_state(response.provider_total(), results.len(), limit, offset)?;
    Ok(super::TrialSearchPage {
        results,
        total,
        continuation,
    })
}

fn nci_search_state(
    provider_total: &biodata::ClinicalTrialProviderTotal,
    returned: usize,
    limit: usize,
    offset: usize,
) -> Result<
    (
        biodata::ClinicalTrialSearchTotal,
        biodata::ClinicalTrialSearchContinuation,
    ),
    BioMcpError,
> {
    let total = match provider_total {
        biodata::ClinicalTrialProviderTotal::Present(value) => {
            let value = usize::try_from(*value).map_err(|_| BioMcpError::InternalProcessing)?;
            super::exact_total(value)?
        }
        biodata::ClinicalTrialProviderTotal::Absent => biodata::ClinicalTrialSearchTotal::unknown(
            biodata::ClinicalTrialSearchUnknownReason::ProviderOmittedTotal,
        ),
        biodata::ClinicalTrialProviderTotal::NotRequested => {
            biodata::ClinicalTrialSearchTotal::unknown(
                biodata::ClinicalTrialSearchUnknownReason::TotalNotRequested,
            )
        }
    };
    let known_exhausted = total
        .value()
        .and_then(|value| usize::try_from(value).ok())
        .is_some_and(|value| offset.saturating_add(returned) >= value);
    let continuation = if returned < limit || known_exhausted {
        biodata::ClinicalTrialSearchContinuation::terminal()
    } else {
        super::offset_continuation(offset.saturating_add(returned))?
    };
    Ok((total, continuation))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(value: serde_json::Value) -> MyDiseaseHit {
        serde_json::from_value(value).expect("valid disease hit")
    }

    #[test]
    fn disease_grounding_prefers_nci_identity_and_otherwise_keeps_keyword() {
        let grounded = from_hit(
            "melanoma",
            hit(
                serde_json::json!({"_id":"MONDO:1","mondo":{"name":"Melanoma","xrefs":{"ncit":["C3224"]}}}),
            ),
        );
        assert_eq!(
            grounded,
            biodata::NciCtsV2DiseaseSelection::ConceptId("C3224".into())
        );
        let fallback = from_hit(
            "melanoma",
            hit(serde_json::json!({"_id":"MONDO:1","mondo":{"name":"Melanoma"}})),
        );
        assert_eq!(
            fallback,
            biodata::NciCtsV2DiseaseSelection::Keyword("melanoma".into())
        );
    }

    #[test]
    fn unsupported_nci_filter_fails_before_client_construction() {
        let filters = TrialSearchFilters {
            age: Some(67.0),
            source: super::super::super::TrialSource::NciCts,
            ..Default::default()
        };
        let error = super::super::validate_trial_search(&filters)
            .err()
            .expect("age must fail");
        assert!(matches!(error, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn nci_missing_total_never_uses_returned_rows_as_a_total() {
        let (total, continuation) =
            nci_search_state(&biodata::ClinicalTrialProviderTotal::Absent, 5, 5, 10).unwrap();
        assert_eq!(total.value(), None);
        assert_eq!(total.reason(), Some("provider_omitted_total"));
        assert_eq!(continuation.status(), "offset");
        assert_eq!(continuation.offset_value(), Some(15));

        let (total, continuation) =
            nci_search_state(&biodata::ClinicalTrialProviderTotal::Present(15), 5, 5, 10).unwrap();
        assert_eq!((total.value(), total.precision()), (Some(15), "exact"));
        assert_eq!(continuation.status(), "terminal");
    }
}
