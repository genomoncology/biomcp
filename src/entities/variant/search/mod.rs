//! Variant search against MyVariant.info with quality scoring and result shaping.

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::myvariant::{MyVariantClient, VariantSearchParams};
use crate::transform;

use super::{VariantSearchFilters, VariantSearchResult};

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

pub async fn search_page(
    filters: &VariantSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<VariantSearchResult>, BioMcpError> {
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

    let params = VariantSearchParams {
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
        limit: fetch_limit,
        offset,
    };

    let client = MyVariantClient::new()?;
    let resp = client.search(&params).await?;
    let mut out = resp
        .hits
        .iter()
        .map(transform::variant::from_myvariant_search_hit)
        .collect::<Vec<_>>();
    out.sort_by(|a, b| {
        search_result_quality_score(b)
            .cmp(&search_result_quality_score(a))
            .then_with(|| a.id.cmp(&b.id))
    });
    out.truncate(limit);
    Ok(SearchPage::offset(out, resp.total))
}

#[cfg(test)]
mod tests;
