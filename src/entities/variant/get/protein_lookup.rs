//! Execute gene plus protein detail queries without changing point selection policy.
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
        return interval_lookup(client, &query, id, gene, change, requested).await;
    }
    let q = format!(
        "dbnsfp.genename:{} AND dbnsfp.hgvsp:\"p.{}\"",
        gene,
        MyVariantClient::escape_query_value(change)
    );
    let resp = client
        .query_with_fields(&q, 5, 0, MYVARIANT_FIELDS_GET)
        .await?;
    resp.hits
        .into_iter()
        .find(|hit| super::candidate_matches_requested_identity(requested, hit))
        .ok_or_else(|| not_found(id, gene, change))
}

fn not_found(id: &str, gene: &str, change: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "variant".into(),
        id: id.to_string(),
        suggestion: format!("Try searching: biomcp search variant -g {gene} --hgvsp {change}"),
    }
}

async fn interval_lookup(
    client: &MyVariantClient,
    query: &str,
    id: &str,
    gene: &str,
    change: &str,
    requested: &RequestedVariantIdentity,
) -> Result<MyVariantHit, BioMcpError> {
    let mut seen: HashSet<(String, Vec<String>)> = HashSet::new();
    let mut selected = None;
    let mut indeterminate = false;
    let mut examined = 0;
    let mut complete = false;
    while examined < CANDIDATE_LIMIT {
        let response = client
            .query_with_fields(query, PAGE_SIZE, examined, MYVARIANT_FIELDS_GET)
            .await?;
        let count = response.hits.len();
        for hit in response.hits.into_iter().take(CANDIDATE_LIMIT - examined) {
            examined += 1;
            let source = SourceVariantIdentity::from_myvariant_hit(&hit);
            match compare_variant_identity(requested, &source) {
                VariantIdentityComparison::Compatible { .. } => {
                    let mut assertions = source.protein_changes.clone();
                    assertions.sort();
                    if seen.insert((source.normalized_key(), assertions)) && selected.is_none() {
                        selected = Some(hit);
                    }
                }
                VariantIdentityComparison::Indeterminate { .. } => indeterminate = true,
                VariantIdentityComparison::Contradictory { .. } => {}
            }
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
    if !complete {
        return Err(BioMcpError::InvalidArgument(INCOMPLETE.into()));
    }
    if seen.len() > 1 {
        return Err(BioMcpError::InvalidArgument(AMBIGUOUS.into()));
    }
    if indeterminate {
        return Err(BioMcpError::InvalidArgument(EVIDENCE.into()));
    }
    selected.ok_or_else(|| not_found(id, gene, change))
}

#[cfg(test)]
#[path = "protein_lookup_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "protein_lookup_transport_tests.rs"]
mod transport_tests;
