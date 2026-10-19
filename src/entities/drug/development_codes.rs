//! Resolve a sparse development-code query through an exact source synonym.

use crate::error::BioMcpError;
use crate::sources::chembl::ChemblClient;
use crate::sources::mychem::MyChemQueryResponse;
use crate::transform;

use super::{Drug, direct_drug_lookup};

/// Limit additional identity work to code-shaped requests with no identifiers.
/// Existing named cards and combination products keep their own identity rules.
fn needs_synonym_lookup(name: &str, drug: &Drug) -> bool {
    let Some(first_digit) = name.find(|c: char| c.is_ascii_digit()) else {
        return false;
    };
    let prefix = name[..first_digit].trim_end_matches(['-', ' ']);
    !prefix.is_empty()
        && prefix.chars().all(|c| c.is_ascii_alphabetic())
        && name[first_digit..].chars().all(|c| c.is_ascii_digit())
        && !transform::drug::card_names_combination(&drug.name)
        && drug.drugbank_id.is_none()
        && drug.chembl_id.is_none()
        && drug.unii.is_none()
}

pub(super) async fn resolve(
    name: &str,
    drug: &Drug,
) -> Result<Option<(String, MyChemQueryResponse)>, BioMcpError> {
    if !needs_synonym_lookup(name, drug) {
        return Ok(None);
    }
    let Some(identity) = ChemblClient::new()?.molecule_by_exact_synonym(name).await? else {
        return Ok(None);
    };
    let candidate = identity.pref_name.trim();
    let response = direct_drug_lookup(candidate).await?;
    let selected = transform::drug::select_hits_for_name(&response.hits, candidate);
    // Both the preferred name and the source molecule identifier must agree.
    // A candidate's incidental text hits cannot establish the join.
    let molecule_ids: Vec<_> = selected
        .iter()
        .filter_map(|hit| hit.chembl.as_ref()?.molecule_chembl_id.as_deref())
        .collect();
    let card = transform::drug::merge_mychem_hits(&selected, candidate);
    if !card.name.eq_ignore_ascii_case(candidate)
        || card.chembl_id.as_deref() != Some(identity.molecule_chembl_id.as_str())
        || molecule_ids.is_empty()
        || molecule_ids
            .iter()
            .any(|id| *id != identity.molecule_chembl_id)
    {
        return Err(BioMcpError::NotFound {
            entity: "drug".into(),
            id: name.into(),
            suggestion: "The ChEMBL synonym could not be confirmed on the preferred drug card. Try the preferred drug name.".into(),
        });
    }
    Ok(Some((candidate.to_string(), response)))
}
