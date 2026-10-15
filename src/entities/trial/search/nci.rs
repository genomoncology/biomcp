//! NCI CTS trial search orchestration. BioData owns request grammar.
use super::super::{TrialSearchFilters, TrialSearchHit};
use super::{NormalizedTrialSearch, biodata_plan_error};
use crate::entities::disease::resolve_disease_hit_by_name;
use crate::error::BioMcpError;
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit};
use crate::sources::nci_cts::NciCtsClient;
use crate::transform;
use tracing::warn;

/// The NCI condition selection plus the note to surface when grounding
/// degraded from an NCI concept ID to a plain keyword search. The note
/// reaches the response (`meta.notes` and the markdown footer), not only a
/// log line — the visible-degrade pattern.
struct NciDiseaseGrounding {
    selection: biodata::NciCtsV2DiseaseSelection,
    degrade_note: Option<String>,
}

/// MyDisease's query endpoint refuses inputs over 512 bytes, and that
/// refusal used to fail the whole NCI search. The condition now truncates to the
/// longest prefix the lookup accepts, with a note keeping the
/// truncation visible.
const NCI_CONDITION_GROUNDING_LIMIT_BYTES: usize = 512;

fn truncate_condition_for_grounding(condition: &str) -> (&str, bool) {
    if condition.len() <= NCI_CONDITION_GROUNDING_LIMIT_BYTES {
        return (condition, false);
    }
    let mut end = NCI_CONDITION_GROUNDING_LIMIT_BYTES;
    while end > 0 && !condition.is_char_boundary(end) {
        end -= 1;
    }
    (&condition[..end], true)
}

fn nci_truncation_note() -> String {
    format!(
        "NCI trial search used at most the first {NCI_CONDITION_GROUNDING_LIMIT_BYTES} bytes of the condition \
(the disease lookup accepts no more); results reflect the truncated condition."
    )
}

fn nci_keyword_degrade_note(condition: &str, reason: &str) -> String {
    format!(
        "NCI trial search used a plain keyword search for '{condition}' because {reason}; \
results may be broader than the requested condition."
    )
}

async fn resolve_nci_disease_grounding(
    client: &MyDiseaseClient,
    condition: Option<&str>,
) -> Result<Option<NciDiseaseGrounding>, BioMcpError> {
    let Some(requested) = condition.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let (condition, truncated) = truncate_condition_for_grounding(requested);

    let mut grounding = match resolve_disease_hit_by_name(client, condition).await {
        Ok(hit) => Some(nci_disease_grounding_from_hit(condition, hit)),
        // The exact-resolution refusal rejects the condition itself; a keyword search
        // for the raw token would mix the same holders' trials behind a
        // note, so trial search returns the refusal and its choices
        // instead of degrading.
        Err(err @ BioMcpError::InvalidArgument(_)) => return Err(err),
        Err(BioMcpError::NotFound { .. }) => Some(NciDiseaseGrounding {
            selection: biodata::NciCtsV2DiseaseSelection::Keyword(condition.to_string()),
            degrade_note: Some(nci_keyword_degrade_note(
                condition,
                "it does not ground to a disease concept in MyDisease",
            )),
        }),
        Err(error) if crate::sources::mydisease::optional_failure(&error) => {
            warn!(error=%error, "NCI disease grounding failed, falling back to keyword");
            let reason_text = error.to_string();
            let reason = reason_text
                .lines()
                .next()
                .unwrap_or("disease grounding failed");
            Some(NciDiseaseGrounding {
                selection: biodata::NciCtsV2DiseaseSelection::Keyword(condition.to_string()),
                degrade_note: Some(nci_keyword_degrade_note(
                    condition,
                    &format!("disease grounding failed ({reason})"),
                )),
            })
        }
        Err(error) => return Err(error),
    };
    // The truncation note applies whether grounding succeeded on the
    // truncated text or degraded to a keyword search: the search ran on
    // fewer bytes than the caller sent.
    if truncated && let Some(grounding) = grounding.as_mut() {
        let truncation = nci_truncation_note();
        grounding.degrade_note = Some(match grounding.degrade_note.take() {
            Some(existing) => format!("{truncation} {existing}"),
            None => truncation,
        });
    }
    Ok(grounding)
}

fn nci_disease_grounding_from_hit(condition: &str, hit: MyDiseaseHit) -> NciDiseaseGrounding {
    let mut disease = transform::disease::from_mydisease_hit(hit);
    if let Some(nci_id) = disease
        .xrefs
        .remove("NCI")
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
    {
        NciDiseaseGrounding {
            selection: biodata::NciCtsV2DiseaseSelection::ConceptId(nci_id),
            degrade_note: None,
        }
    } else {
        NciDiseaseGrounding {
            selection: biodata::NciCtsV2DiseaseSelection::Keyword(condition.to_string()),
            degrade_note: Some(nci_keyword_degrade_note(
                condition,
                "the resolved disease carries no NCI concept ID",
            )),
        }
    }
}

pub(super) async fn search_page_with_nci_clients(
    client: &NciCtsClient,
    mydisease: &MyDiseaseClient,
    filters: &TrialSearchFilters,
    normalized: &NormalizedTrialSearch,
    limit: usize,
    offset: usize,
) -> Result<super::TrialSearchPage, BioMcpError> {
    let grounding =
        resolve_nci_disease_grounding(mydisease, filters.condition.as_deref()).await?;
    let degrade_note = grounding.as_ref().and_then(|g| g.degrade_note.clone());
    let plan = biodata::NciCtsV2SearchPlan::new(
        &normalized.biodata,
        grounding.map(|g| g.selection),
        limit,
        offset,
    )
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
        eligibility_verification_upstream_total: None,
        partial_note: degrade_note,
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
        let grounded = nci_disease_grounding_from_hit(
            "melanoma",
            hit(
                serde_json::json!({"_id":"MONDO:1","mondo":{"name":"Melanoma","xrefs":{"ncit":["C3224"]}}}),
            ),
        );
        assert_eq!(
            grounded.selection,
            biodata::NciCtsV2DiseaseSelection::ConceptId("C3224".into())
        );
        assert_eq!(grounded.degrade_note, None);
        let fallback = nci_disease_grounding_from_hit(
            "melanoma",
            hit(serde_json::json!({"_id":"MONDO:1","mondo":{"name":"Melanoma"}})),
        );
        assert_eq!(
            fallback.selection,
            biodata::NciCtsV2DiseaseSelection::Keyword("melanoma".into())
        );
        assert!(fallback.degrade_note.is_some());
    }

    #[test]
    fn overlong_conditions_truncate_on_a_char_boundary() {
        let (condition, truncated) =
            truncate_condition_for_grounding(&"a".repeat(NCI_CONDITION_GROUNDING_LIMIT_BYTES));
        assert_eq!(condition.len(), NCI_CONDITION_GROUNDING_LIMIT_BYTES);
        assert!(!truncated);
        let long = format!("{}{}", "ä".repeat(NCI_CONDITION_GROUNDING_LIMIT_BYTES), "z");
        let (condition, truncated) = truncate_condition_for_grounding(&long);
        assert!(truncated);
        assert!(condition.len() < NCI_CONDITION_GROUNDING_LIMIT_BYTES);
        assert!(condition.is_char_boundary(NCI_CONDITION_GROUNDING_LIMIT_BYTES - 1));
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
