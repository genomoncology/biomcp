//! Bounded, complete candidate retrieval for exact identity resolution.

use super::{BioMcpError, MyDiseaseClient, MyDiseaseHit, source_resolution_error};

const PAGE_SIZE: usize = 50;
const MAX_PAGES: usize = 4;

pub(super) async fn complete_exact_candidates(
    client: &MyDiseaseClient,
    requested: &str,
) -> Result<Vec<MyDiseaseHit>, BioMcpError> {
    let mut hits = Vec::new();
    let mut expected_total = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..MAX_PAGES {
        let response = client
            .query(requested, PAGE_SIZE, hits.len(), None, None, None, None)
            .await
            .map_err(|_| source_resolution_error())?;
        let total = *expected_total.get_or_insert(response.total);
        if total > PAGE_SIZE * MAX_PAGES
            || response.total != total
            || response.hits.len() != total.saturating_sub(hits.len()).min(PAGE_SIZE)
        {
            return Err(source_resolution_error());
        }
        // Repeated records cannot establish coverage of the advertised total.
        // Never select the first page's exact name before checking later holders.
        for hit in response.hits {
            if !seen.insert(hit.id.clone()) {
                return Err(source_resolution_error());
            }
            hits.push(hit);
        }
        if hits.len() == total {
            return Ok(hits);
        }
    }
    Err(source_resolution_error())
}
