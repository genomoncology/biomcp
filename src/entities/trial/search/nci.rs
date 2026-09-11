//! NCI CTS trial search orchestration. BioData owns request grammar.
use super::super::{TrialSearchFilters, TrialSearchResult};
use super::{NormalizedTrialSearch, biodata_plan_error};
use crate::entities::SearchPage;
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
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
    let disease = resolve_disease(mydisease, filters.condition.as_deref()).await?;
    let plan = biodata::NciCtsV2SearchPlan::new(&normalized.biodata, disease, limit, offset)
        .map_err(|error| biodata_plan_error("invalid NCI trial search", error))?;
    let response = client.search(&plan).await?;
    Ok(SearchPage::offset(
        response
            .results()
            .unwrap_or_default()
            .iter()
            .map(|r| TrialSearchResult::from_biodata(r.projection().value()))
            .collect::<Result<Vec<_>, _>>()?,
        response.total_count().and_then(|v| usize::try_from(v).ok()),
    ))
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
}
