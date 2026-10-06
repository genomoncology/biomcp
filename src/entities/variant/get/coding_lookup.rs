//! Select one source record after a complete bounded written coding lookup.
use crate::entities::variant::{
    RequestedVariantIdentity, SourceVariantIdentity, VariantIdentityComparison,
    compare_variant_identity,
};
use crate::error::BioMcpError;
use crate::sources::myvariant::{MYVARIANT_FIELDS_GET, MyVariantClient, MyVariantHit};
use std::collections::HashSet;

const PAGE_SIZE: usize = 50;
const CANDIDATE_LIMIT: usize = 1_000;
const AMBIGUOUS: &str = "Gene coding lookup is ambiguous. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";
const EVIDENCE: &str = "Gene coding lookup lacks complete identity evidence. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";
const INCOMPLETE: &str = "Gene coding lookup did not complete its candidate scan. Use an exact rsID or genomic HGVS ID, or search with the gene and coding change.";

pub(super) async fn lookup(
    client: &MyVariantClient,
    id: &str,
    gene: &str,
    change: &str,
    requested: &RequestedVariantIdentity,
) -> Result<MyVariantHit, BioMcpError> {
    let query = format!(
        "dbnsfp.genename:{gene} AND dbnsfp.hgvsc:\"{}\"",
        MyVariantClient::escape_query_value(change)
    );
    let mut seen = HashSet::new();
    let mut selected = None;
    let mut indeterminate = false;
    let mut examined = 0;
    let mut known_total: Option<usize> = None;
    let mut complete = false;
    while examined < CANDIDATE_LIMIT {
        let response = client
            .query_with_fields(&query, PAGE_SIZE, examined, MYVARIANT_FIELDS_GET)
            .await
            .map_err(source_error)?;
        if let Some(total) = response.total {
            known_total = Some(known_total.map_or(total, |prior| prior.max(total)));
        }
        let count = response.hits.len();
        let page_start = examined;
        for hit in response.hits.into_iter().take(CANDIDATE_LIMIT - examined) {
            examined += 1;
            let source = SourceVariantIdentity::from_myvariant_hit(&hit);
            if hit.id.trim().is_empty() {
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
        id: id.into(),
        suggestion: format!("Try searching: biomcp search variant \"{gene} {change}\""),
    })
}

pub(super) fn candidate_matches_requested_identity(
    requested: &RequestedVariantIdentity,
    hit: &crate::sources::myvariant::MyVariantHit,
) -> bool {
    matches!(
        compare_variant_identity(requested, &SourceVariantIdentity::from_myvariant_hit(hit)),
        VariantIdentityComparison::Compatible { .. }
    )
}

// Keep ordinary error classes while discarding provider bodies and request URLs.
fn source_error(error: BioMcpError) -> BioMcpError {
    match error {
        BioMcpError::WithSourceContext { context, source } => {
            source_error(*source).with_source_context(context)
        }
        BioMcpError::Http(error) => BioMcpError::Http(error.without_url()),
        BioMcpError::HttpMiddleware(error) => BioMcpError::HttpMiddleware(match error {
            reqwest_middleware::Error::Reqwest(error) => {
                reqwest_middleware::Error::Reqwest(error.without_url())
            }
            reqwest_middleware::Error::Middleware(_) => {
                reqwest_middleware::Error::Middleware(anyhow::anyhow!("Source request failed."))
            }
        }),
        BioMcpError::ApiJson { api, .. } => BioMcpError::ApiJson {
            api,
            source: <serde_json::Error as serde::de::Error>::custom(
                "Source response could not be decoded.",
            ),
        },
        error @ (BioMcpError::BodyLimit { .. } | BioMcpError::ProviderResponseLimit { .. }) => {
            error
        }
        _ => BioMcpError::Api {
            api: "MyVariant.info".into(),
            message: "Source request failed.".into(),
        },
    }
}

#[cfg(test)]
#[path = "coding_lookup_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "coding_lookup_transport_tests.rs"]
mod transport_tests;
