//! Select a unique source identity only after a complete bounded protein lookup.
use crate::entities::variant::resolution::protein_get;
use crate::entities::variant::{
    RequestedVariantIdentity, SourceVariantIdentity, VariantIdentityComparison,
    compare_variant_identity,
};
use crate::error::BioMcpError;
use crate::sources::myvariant::{MYVARIANT_FIELDS_GET, MyVariantClient, MyVariantHit};
use std::collections::HashSet;

const PAGE_SIZE: usize = 50;
const CANDIDATE_LIMIT: usize = 1_000;
const AMBIGUOUS: &str = "Protein interval lookup is ambiguous. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";
const EVIDENCE: &str = "Protein interval lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";
const INCOMPLETE: &str = "Protein interval lookup reached its candidate limit. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";

const POINT_AMBIGUOUS: &str = "Protein point lookup is ambiguous. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";
const POINT_EVIDENCE: &str = "Protein point lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";
const POINT_INCOMPLETE: &str = "Protein point lookup reached its candidate limit. Use an exact rsID or genomic HGVS ID, or search with the gene and protein change.";

pub(super) async fn lookup(
    client: &MyVariantClient,
    id: &str,
    gene: &str,
    change: &str,
    requested: &RequestedVariantIdentity,
) -> Result<MyVariantHit, BioMcpError> {
    if let Some(prepared) = protein_get::prepare(id)? {
        let terms = prepared
            .spellings
            .iter()
            .map(|spelling| {
                format!(
                    "dbnsfp.hgvsp:\"{}\"",
                    MyVariantClient::escape_query_value(spelling)
                )
            })
            .collect::<Vec<_>>();
        let query = format!(
            "dbnsfp.genename:{} AND ({})",
            prepared.gene,
            terms.join(" OR ")
        );
        return bounded_lookup(client, &query, id, gene, change, requested, false).await;
    }
    let q = format!(
        "dbnsfp.genename:{} AND dbnsfp.hgvsp:\"p.{}\"",
        gene,
        MyVariantClient::escape_query_value(change)
    );
    bounded_lookup(client, &q, id, gene, change, requested, true).await
}

fn not_found(id: &str, gene: &str, change: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "variant".into(),
        id: id.to_string(),
        suggestion: format!("Try searching: biomcp search variant -g {gene} --hgvsp {change}"),
    }
}

async fn bounded_lookup(
    client: &MyVariantClient,
    query: &str,
    id: &str,
    gene: &str,
    change: &str,
    requested: &RequestedVariantIdentity,
    point: bool,
) -> Result<MyVariantHit, BioMcpError> {
    let mut seen: HashSet<(String, Vec<String>, Vec<String>)> = HashSet::new();
    let mut selected = None;
    let mut point_candidates = Vec::new();
    let mut indeterminate = false;
    let mut examined = 0;
    let mut complete = false;
    while examined < CANDIDATE_LIMIT {
        let response = client
            .query_with_fields(query, PAGE_SIZE, examined, MYVARIANT_FIELDS_GET)
            .await?;
        let count = response.hits.len();
        let page_start = examined;
        for hit in response.hits.into_iter().take(CANDIDATE_LIMIT - examined) {
            examined += 1;
            let source = SourceVariantIdentity::from_myvariant_hit(&hit);
            match compare_variant_identity(requested, &source) {
                VariantIdentityComparison::Compatible { .. } => {
                    let mut assertions = source.protein_changes.clone();
                    assertions.sort();
                    let mut coding = if point {
                        source.coding_changes.clone()
                    } else {
                        Vec::new()
                    };
                    coding.sort();
                    if seen.insert((source.normalized_key(), assertions, coding)) {
                        if point {
                            point_candidates.push(hit.clone());
                        }
                        if selected.is_none() {
                            selected = Some(hit);
                        }
                    }
                }
                VariantIdentityComparison::Indeterminate { .. } => indeterminate = true,
                VariantIdentityComparison::Contradictory { .. } => {}
            }
        }
        if point && count > examined - page_start {
            break;
        }
        if crate::entities::variant::search::candidate_scan_exhaustive(
            response.total,
            examined,
            count,
        ) {
            complete = true;
            break;
        }
    }
    let [ambiguous, evidence, incomplete] = if point {
        [POINT_AMBIGUOUS, POINT_EVIDENCE, POINT_INCOMPLETE]
    } else {
        [AMBIGUOUS, EVIDENCE, INCOMPLETE]
    };
    if !complete {
        return Err(BioMcpError::InvalidArgument(incomplete.into()));
    }
    if point && !indeterminate {
        let keys = point_candidates
            .iter()
            .map(|hit| hit.id.trim().to_string())
            .collect::<HashSet<_>>();
        if keys.len() == point_candidates.len() {
            return super::resolve_protein_change_hit(id, gene, change, point_candidates);
        }
    }
    if seen.len() > 1 {
        return Err(BioMcpError::InvalidArgument(ambiguous.into()));
    }
    if indeterminate {
        return Err(BioMcpError::InvalidArgument(evidence.into()));
    }
    selected.ok_or_else(|| not_found(id, gene, change))
}

#[cfg(test)]
#[path = "protein_lookup_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "protein_lookup_transport_tests.rs"]
mod transport_tests;
