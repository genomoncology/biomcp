//! Variant search against MyVariant.info with quality scoring and result shaping.

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::myvariant::{MyVariantClient, MyVariantHit, VariantSearchParams};
use crate::transform;
use std::collections::HashSet;

use super::{
    RequestedVariantIdentity, SourceVariantIdentity, VariantArticleResolutionContext,
    VariantIdentityComparison, VariantResolutionStatus, VariantSearchFilters,
    VariantSearchResolution, VariantSearchResult, compare_variant_identity,
};

#[derive(Debug, Clone)]
pub(crate) struct VariantSearchPage {
    pub results: Vec<VariantSearchResult>,
    pub total: Option<usize>,
    pub requested_variant: Option<RequestedVariantIdentity>,
    pub resolution: Option<VariantSearchResolution>,
    pub has_more: Option<bool>,
}

fn search_result_quality_score(row: &VariantSearchResult) -> i32 {
    let mut score = 0;
    if row
        .significance
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 4;
    }
    if row.gnomad_af.is_some() {
        score += 4;
    }
    if row.clinvar_stars.is_some() {
        score += 3;
    }
    if row.revel.is_some() {
        score += 2;
    }
    if row.gerp.is_some() {
        score += 2;
    }
    if row
        .hgvs_p
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
    {
        score += 2;
    }
    if !row.gene.trim().is_empty() {
        score += 1;
    }
    score
}

pub fn search_query_summary(filters: &VariantSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={v}"));
    }
    if let Some(alias) = filters.protein_alias.as_ref() {
        parts.push(format!("residue_alias={}", alias.label()));
    }
    if let Some(v) = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("hgvsp={v}"));
    }
    if let Some(v) = filters
        .hgvsc
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("hgvsc={v}"));
    }
    if let Some(v) = filters
        .rsid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("rsid={v}"));
    }
    if let Some(v) = filters
        .significance
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("significance={v}"));
    }
    if let Some(v) = filters.max_frequency {
        parts.push(format!("max_frequency={v}"));
    }
    if let Some(v) = filters.min_cadd {
        parts.push(format!("min_cadd={v}"));
    }
    if let Some(v) = filters
        .consequence
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("consequence={v}"));
    }
    if let Some(v) = filters
        .review_status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("review_status={v}"));
    }
    if let Some(v) = filters
        .population
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("population={v}"));
    }
    if let Some(v) = filters.revel_min {
        parts.push(format!("revel_min={v}"));
    }
    if let Some(v) = filters.gerp_min {
        parts.push(format!("gerp_min={v}"));
    }
    if let Some(v) = filters
        .tumor_site
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("tumor_site={v}"));
    }
    if let Some(v) = filters
        .condition
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("condition={v}"));
    }
    if let Some(v) = filters
        .impact
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("impact={v}"));
    }
    if filters.lof {
        parts.push("lof=true".to_string());
    }
    if let Some(v) = filters
        .has
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("has={v}"));
    }
    if let Some(v) = filters
        .missing
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("missing={v}"));
    }
    if let Some(v) = filters
        .therapy
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("therapy={v}"));
    }

    parts.join(", ")
}

// dead-code reason: mod::search is exercised by native tests or binary dispatch
#[allow(dead_code)]
pub async fn search(
    filters: &VariantSearchFilters,
    limit: usize,
) -> Result<Vec<VariantSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub(crate) async fn resolve_article_variant(
    input: &str,
) -> Result<VariantArticleResolutionContext, BioMcpError> {
    let requested = RequestedVariantIdentity::from_variant_input(input)?;
    if requested.genomic_accession.is_some() {
        let result = match MyVariantClient::new() {
            Ok(client) => client.get(input).await,
            Err(err) => Err(err),
        };
        return Ok(match result {
            Ok(hit) => {
                let source = SourceVariantIdentity::from_myvariant_hit(&hit);
                let status = match compare_variant_identity(&requested, &source) {
                    VariantIdentityComparison::Compatible { .. } => {
                        VariantResolutionStatus::Resolved
                    }
                    VariantIdentityComparison::Indeterminate { .. } => {
                        VariantResolutionStatus::Ambiguous
                    }
                    VariantIdentityComparison::Contradictory { .. } => {
                        VariantResolutionStatus::Unresolved
                    }
                };
                let resolved = matches!(status, VariantResolutionStatus::Resolved);
                VariantArticleResolutionContext {
                    requested: requested.clone(),
                    resolution: VariantSearchResolution {
                        status,
                        normalized_aliases: requested.normalized_aliases(),
                        exhaustive: true,
                    },
                    source_id: resolved.then(|| hit.id.clone()),
                    source_identity: resolved.then_some(source),
                    available: true,
                }
            }
            Err(BioMcpError::NotFound { .. }) => VariantArticleResolutionContext {
                requested: requested.clone(),
                resolution: VariantSearchResolution {
                    status: VariantResolutionStatus::Unresolved,
                    normalized_aliases: requested.normalized_aliases(),
                    exhaustive: true,
                },
                source_id: None,
                source_identity: None,
                available: true,
            },
            Err(_) => VariantArticleResolutionContext {
                requested: requested.clone(),
                resolution: VariantSearchResolution {
                    status: VariantResolutionStatus::Ambiguous,
                    normalized_aliases: requested.normalized_aliases(),
                    exhaustive: false,
                },
                source_id: None,
                source_identity: None,
                available: false,
            },
        });
    }
    let filters = VariantSearchFilters {
        gene: requested.gene.clone(),
        hgvsp: requested.protein_change.clone(),
        hgvsc: requested.coding_change.clone(),
        rsid: requested.rsid.clone(),
        requested_identity: Some(requested.clone()),
        ..Default::default()
    };
    let page = match search_page(&filters, 2, 0).await {
        Ok(page) => page,
        Err(_) => {
            return Ok(VariantArticleResolutionContext {
                requested: requested.clone(),
                resolution: VariantSearchResolution {
                    status: VariantResolutionStatus::Ambiguous,
                    normalized_aliases: requested.normalized_aliases(),
                    exhaustive: false,
                },
                source_id: None,
                source_identity: None,
                available: false,
            });
        }
    };
    let resolution = page.resolution.unwrap_or(VariantSearchResolution {
        status: VariantResolutionStatus::Unresolved,
        normalized_aliases: requested.normalized_aliases(),
        exhaustive: false,
    });
    let resolved = matches!(resolution.status, VariantResolutionStatus::Resolved)
        .then(|| page.results.into_iter().next())
        .flatten();
    Ok(VariantArticleResolutionContext {
        requested,
        resolution,
        source_id: resolved.as_ref().map(|row| row.id.clone()),
        source_identity: resolved.and_then(|row| row.source_identity),
        available: true,
    })
}

pub async fn search_page(
    filters: &VariantSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<VariantSearchPage, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let has_precision_filter = filters
        .hgvsp
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .hgvsc
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .rsid
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.protein_alias.is_some()
        || filters
            .significance
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.max_frequency.is_some()
        || filters.min_cadd.is_some()
        || filters
            .review_status
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .population
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.revel_min.is_some()
        || filters.gerp_min.is_some()
        || filters
            .tumor_site
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .condition
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .impact
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.lof
        || filters
            .has
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .missing
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .consequence
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty());
    let fetch_limit = if has_precision_filter {
        limit
    } else {
        (limit.saturating_mul(40)).clamp(limit, 200)
    };

    let params_at = |page_limit, page_offset| VariantSearchParams {
        gene: filters.gene.clone(),
        hgvsp: filters.hgvsp.clone(),
        hgvsc: filters.hgvsc.clone(),
        rsid: filters.rsid.clone(),
        protein_alias: filters.protein_alias.clone(),
        significance: filters.significance.clone(),
        max_frequency: filters.max_frequency,
        min_cadd: filters.min_cadd,
        consequence: filters.consequence.clone(),
        review_status: filters.review_status.clone(),
        population: filters.population.clone(),
        revel_min: filters.revel_min,
        gerp_min: filters.gerp_min,
        tumor_site: filters.tumor_site.clone(),
        condition: filters.condition.clone(),
        impact: filters.impact.clone(),
        lof: filters.lof,
        has: filters.has.clone(),
        missing: filters.missing.clone(),
        therapy: filters.therapy.clone(),
        limit: page_limit,
        offset: page_offset,
    };

    let client = MyVariantClient::new()?;
    let Some(requested) = filters.requested_identity.as_ref() else {
        let resp = client.search(&params_at(fetch_limit, offset)).await?;
        let mut out = resp
            .hits
            .iter()
            .map(transform::variant::from_myvariant_search_hit)
            .collect::<Vec<_>>();
        sort_results(&mut out);
        out.truncate(limit);
        let page = SearchPage::offset(out, resp.total);
        return Ok(VariantSearchPage {
            results: page.results,
            total: page.total,
            requested_variant: None,
            resolution: None,
            has_more: None,
        });
    };

    const SOURCE_PAGE: usize = 50;
    const MAX_CANDIDATES: usize = 1_000;
    let mut provider_offset = 0;
    let mut retained = Vec::new();
    let mut seen = HashSet::new();
    let mut saw_indeterminate = false;
    let mut exhaustive = false;
    while provider_offset < MAX_CANDIDATES {
        let resp = client
            .search(&params_at(SOURCE_PAGE, provider_offset))
            .await?;
        let provider_total = resp.total;
        let hit_count = resp.hits.len();
        let examined_count = hit_count.min(MAX_CANDIDATES - provider_offset);
        saw_indeterminate |= retain_compatible_hits(
            requested,
            resp.hits.into_iter().take(examined_count),
            &mut seen,
            &mut retained,
        );
        provider_offset += examined_count;
        if candidate_scan_exhaustive(provider_total, provider_offset, hit_count) {
            exhaustive = true;
            break;
        }
    }
    Ok(finalize_exact_page(
        requested,
        retained,
        offset,
        limit,
        saw_indeterminate,
        exhaustive,
    ))
}

fn candidate_scan_exhaustive(
    provider_total: Option<usize>,
    examined_offset: usize,
    returned_count: usize,
) -> bool {
    returned_count == 0 || provider_total.is_some_and(|total| examined_offset >= total)
}

fn retain_compatible_hits(
    requested: &RequestedVariantIdentity,
    hits: impl IntoIterator<Item = MyVariantHit>,
    seen: &mut HashSet<String>,
    retained: &mut Vec<VariantSearchResult>,
) -> bool {
    let mut saw_indeterminate = false;
    for hit in hits {
        let source = SourceVariantIdentity::from_myvariant_hit(&hit);
        match compare_variant_identity(requested, &source) {
            VariantIdentityComparison::Compatible { matched_alias } => {
                if seen.insert(source.normalized_key()) {
                    let mut row = transform::variant::from_myvariant_search_hit(&hit);
                    row.source_identity = Some(source);
                    row.matched_alias = Some(matched_alias);
                    retained.push(row);
                }
            }
            VariantIdentityComparison::Indeterminate { .. } => saw_indeterminate = true,
            VariantIdentityComparison::Contradictory { .. } => {}
        }
    }
    saw_indeterminate
}

fn finalize_exact_page(
    requested: &RequestedVariantIdentity,
    mut retained: Vec<VariantSearchResult>,
    offset: usize,
    limit: usize,
    saw_indeterminate: bool,
    exhaustive: bool,
) -> VariantSearchPage {
    sort_results(&mut retained);
    let compatible_count = retained.len();
    let status = resolution_status(compatible_count, saw_indeterminate, exhaustive);
    let total = exhaustive.then_some(compatible_count);
    let has_more = offset.saturating_add(limit) < compatible_count || !exhaustive;
    let results = retained.into_iter().skip(offset).take(limit).collect();
    VariantSearchPage {
        results,
        total,
        requested_variant: Some(requested.clone()),
        resolution: Some(VariantSearchResolution {
            status,
            normalized_aliases: requested.normalized_aliases(),
            exhaustive,
        }),
        has_more: Some(has_more),
    }
}

fn resolution_status(
    compatible_identity_count: usize,
    saw_indeterminate: bool,
    exhaustive: bool,
) -> VariantResolutionStatus {
    if saw_indeterminate || !exhaustive || compatible_identity_count > 1 {
        VariantResolutionStatus::Ambiguous
    } else if compatible_identity_count == 1 {
        VariantResolutionStatus::Resolved
    } else {
        VariantResolutionStatus::Unresolved
    }
}

fn sort_results(out: &mut [VariantSearchResult]) {
    out.sort_by(|a, b| {
        search_result_quality_score(b)
            .cmp(&search_result_quality_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
}

#[cfg(test)]
mod tests;
