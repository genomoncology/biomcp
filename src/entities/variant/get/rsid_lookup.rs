//! Resolve one rsID identity only after a complete bounded source scan.
use crate::entities::variant::{
    RequestedVariantIdentity, SourceVariantIdentity, VariantIdentityComparison,
    compare_variant_identity,
};
use crate::error::BioMcpError;
use crate::sources::myvariant::{MYVARIANT_FIELDS_GET, MyVariantClient, MyVariantHit};
use std::collections::HashSet;

const PAGE_SIZE: usize = 50;
const CANDIDATE_LIMIT: usize = 1_000;
const AMBIGUOUS: &str =
    "RsID lookup is ambiguous. Use an exact genomic HGVS ID, or search with the rsID.";
const EVIDENCE: &str = "RsID lookup lacks complete identity evidence. Use an exact genomic HGVS ID, or search with the rsID.";
const INCOMPLETE: &str = "RsID lookup did not complete its candidate scan. Use an exact genomic HGVS ID, or search with the rsID.";

pub(super) async fn lookup(
    client: &MyVariantClient,
    id: &str,
    rsid: &str,
    requested: &RequestedVariantIdentity,
) -> Result<MyVariantHit, BioMcpError> {
    let query = format!("dbsnp.rsid:{rsid}");
    let mut seen: HashSet<(String, String, Vec<String>, Vec<String>)> = HashSet::new();
    let mut selected = None;
    let mut indeterminate = false;
    let mut examined = 0;
    let mut known_total: Option<usize> = None;
    let mut complete = false;
    while examined < CANDIDATE_LIMIT {
        let response = client
            .query_with_fields(&query, PAGE_SIZE, examined, MYVARIANT_FIELDS_GET)
            .await?;
        if let Some(total) = response.total {
            known_total = Some(known_total.map_or(total, |prior| prior.max(total)));
        }
        let count = response.hits.len();
        let page_start = examined;
        for hit in response.hits.into_iter().take(CANDIDATE_LIMIT - examined) {
            examined += 1;
            let source = SourceVariantIdentity::from_myvariant_hit(&hit);
            if hit.id.trim().is_empty()
                || source.rsids.is_empty()
                || source.rsids.iter().any(|value| value.trim().is_empty())
            {
                indeterminate = true;
                continue;
            }
            match compare_variant_identity(requested, &source) {
                VariantIdentityComparison::Compatible { .. } => {
                    let mut protein = source.protein_changes.clone();
                    let mut coding = source.coding_changes.clone();
                    protein.sort();
                    coding.sort();
                    if seen.insert((hit.id.clone(), source.normalized_key(), protein, coding))
                        && selected.is_none()
                    {
                        selected = Some(hit);
                    }
                }
                VariantIdentityComparison::Indeterminate { .. } => indeterminate = true,
                VariantIdentityComparison::Contradictory { .. } => {}
            }
        }
        // Received rows beyond the allowance cannot establish exhaustive selection.
        if count > examined - page_start {
            break;
        }
        complete = known_total.map_or(count == 0, |total| examined >= total);
        if complete || count == 0 {
            break;
        }
    }
    if !complete {
        return Err(BioMcpError::InvalidArgument(INCOMPLETE.into()));
    }
    if seen.len() > 1 {
        return Err(BioMcpError::InvalidArgument(AMBIGUOUS.into()));
    }
    if indeterminate {
        return Err(BioMcpError::InvalidArgument(EVIDENCE.into()));
    }
    selected.ok_or_else(|| BioMcpError::NotFound {
        entity: "variant".into(),
        id: rsid.into(),
        suggestion: format!("Try searching: biomcp search variant -g \"{id}\""),
    })
}

#[cfg(test)]
#[path = "rsid_lookup_tests.rs"]
mod rsid_lookup_tests;
#[cfg(test)]
#[path = "rsid_lookup_transport_tests.rs"]
mod transport_tests;
