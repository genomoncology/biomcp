//! Bounded response accounting for exact search; retained identity semantics live in the parent.
use super::*;

pub(super) struct ExactScan {
    pub retained: Vec<RetainedVariant>,
    pub saw_indeterminate: bool,
    pub exhaustive: bool,
    pub diagnostics: Vec<SearchDiagnostic>,
}

pub(super) async fn scan(
    client: &MyVariantClient,
    filters: &VariantSearchFilters,
    requested: &RequestedVariantIdentity,
    execution: Option<&crate::entities::article::variant_search::VariantArticleExecutionContext>,
) -> Result<ExactScan, BioMcpError> {
    const SOURCE_PAGE: usize = 50;
    const MAX_CANDIDATES: usize = 1_000;
    let mut provider_offset = 0;
    let mut retained = Vec::new();
    let mut seen = HashSet::new();
    let mut saw_indeterminate = false;
    let mut exhaustive = false;
    let mut diagnostics = Vec::new();
    let mut maximum_total = None;
    while provider_offset < MAX_CANDIDATES {
        let started = execution.and_then(|execution| execution.reserve("resolution"));
        if let Some(execution) = execution
            && started.is_none()
        {
            execution.record_not_attempted("resolution", "myvariant");
            break;
        }
        let result = client
            .search(&search_params(
                filters,
                filters.gene.clone(),
                SOURCE_PAGE,
                provider_offset,
            ))
            .await;
        if let (Some(execution), Some(started)) = (execution, started) {
            match &result {
                Ok(_) => execution.record("resolution", "myvariant", started, "ok", 1),
                Err(error) => execution.record_error("resolution", "myvariant", started, error),
            }
        }
        let initial = result?;
        let (resp, classified) = if provider_offset == 0 && initial.total == Some(0) {
            classify_provider_zero(&client, filters, initial, SOURCE_PAGE, provider_offset).await?
        } else {
            (initial, Vec::new())
        };
        diagnostics.extend(classified);
        if let Some(total) = resp.total {
            maximum_total =
                Some(maximum_total.map_or(total, |previous: usize| previous.max(total)));
        }
        let hit_count = resp.hits.len();
        let examined_count = hit_count.min(MAX_CANDIDATES - provider_offset);
        saw_indeterminate |= retain_compatible_hits(
            requested,
            resp.hits.into_iter().take(examined_count),
            &mut seen,
            &mut retained,
        );
        provider_offset += examined_count;
        if examined_count < hit_count {
            break;
        }
        exhaustive = maximum_total.map_or(hit_count == 0, |total| provider_offset >= total);
        if exhaustive || hit_count == 0 {
            break;
        }
    }
    Ok(ExactScan {
        retained,
        saw_indeterminate,
        exhaustive,
        diagnostics,
    })
}
