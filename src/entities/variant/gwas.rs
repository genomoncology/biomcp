//! GWAS Catalog search and GWAS enrichment for variant detail retrieval.

use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::gwas::{GwasAssociation, GwasAssociationSummary, GwasClient};

use super::resolution::parse_variant_id;
use super::{GwasSearchFilters, Variant, VariantGwasAssociation, VariantIdFormat};

pub(crate) fn validate_p_value(p_value: Option<f64>) -> Result<(), BioMcpError> {
    if p_value.is_some_and(|value| !value.is_finite() || value <= 0.0 || value > 1.0) {
        return Err(BioMcpError::InvalidArgument(
            "--p-value must be finite and greater than 0 and at most 1".into(),
        ));
    }
    Ok(())
}

// dead-code reason: gwas::search_gwas is exercised by native tests or binary dispatch
#[allow(dead_code)]
pub async fn search_gwas(
    filters: &GwasSearchFilters,
    limit: usize,
) -> Result<Vec<VariantGwasAssociation>, BioMcpError> {
    Ok(search_gwas_page(filters, limit, 0).await?.results)
}

#[derive(Debug, Clone)]
pub struct GwasSearchPage {
    pub results: Vec<VariantGwasAssociation>,
    pub pagination: GwasPagination,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GwasPagination {
    pub limit: usize,
    pub offset: usize,
    pub returned: usize,
    pub has_more: bool,
    pub next_offset: Option<usize>,
    pub truncated_by_provider_budget: bool,
}

pub(crate) fn validate_gwas_window(limit: usize, offset: usize) -> Result<usize, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    let end = offset.checked_add(limit).ok_or_else(|| {
        BioMcpError::InvalidArgument("--offset + --limit must be <= 50 for GWAS search".into())
    })?;
    if end > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(
            "--offset + --limit must be <= 50 for GWAS search".into(),
        ));
    }
    Ok(end)
}

pub async fn search_gwas_page(
    filters: &GwasSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<GwasSearchPage, BioMcpError> {
    let window_end = validate_gwas_window(limit, offset)?;
    validate_p_value(filters.p_value)?;
    let page_probe_limit = window_end.checked_add(1).unwrap_or(50).min(50);

    let gene = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let trait_query = filters
        .trait_query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let p_value_threshold = filters.p_value;

    if gene.is_none() && trait_query.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "Provide -g <gene> or --trait <text>. Example: biomcp search gwas -g TCF7L2".into(),
        ));
    }

    let client = GwasClient::new()?;
    let combined = gene.is_some() && trait_query.is_some();
    let fetch_limit = if combined { 50 } else { page_probe_limit };
    let mut gene_rows = Vec::new();
    let mut trait_rows = Vec::new();
    let mut provider_truncated = false;

    if let Some(gene) = gene.as_deref() {
        let page = client
            .search_associations(Some(gene), None, fetch_limit)
            .await?;
        provider_truncated |= page.total > fetch_limit;
        gene_rows.extend(
            page.associations
                .iter()
                .take(50)
                .filter_map(map_gwas_summary),
        );
    }

    if let Some(trait_query) = trait_query.as_deref() {
        let page = client
            .search_associations(None, Some(trait_query), fetch_limit)
            .await?;
        provider_truncated |= page.total > fetch_limit;
        trait_rows.extend(
            page.associations
                .iter()
                .take(50)
                .filter_map(map_gwas_summary),
        );
    }

    let mut rows = dedupe_gwas_rows(intersect_gwas_legs(gene_rows, trait_rows, combined), 50)?;
    apply_p_value_filter(&mut rows, p_value_threshold);
    let has_more = rows.len() > window_end;
    let truncated_by_provider_budget = !has_more && fetch_limit == 50 && provider_truncated;
    let results = rows.drain(..).skip(offset).take(limit).collect::<Vec<_>>();
    let returned = results.len();
    let next_offset = has_more.then(|| offset + returned);
    Ok(GwasSearchPage {
        results,
        pagination: GwasPagination {
            limit,
            offset,
            returned,
            has_more,
            next_offset,
            truncated_by_provider_budget,
        },
    })
}

fn apply_p_value_filter(rows: &mut Vec<VariantGwasAssociation>, threshold: Option<f64>) {
    if let Some(threshold) = threshold {
        rows.retain(|row| row.p_value.is_some_and(|value| value <= threshold));
    }
}

fn intersect_gwas_legs(
    mut gene_rows: Vec<VariantGwasAssociation>,
    mut trait_rows: Vec<VariantGwasAssociation>,
    combined: bool,
) -> Vec<VariantGwasAssociation> {
    if !combined {
        gene_rows.append(&mut trait_rows);
        return gene_rows;
    }
    let gene_ids = gene_rows
        .iter()
        .take(50)
        .map(|row| row.rsid.trim().to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    let trait_ids = trait_rows
        .iter()
        .take(50)
        .map(|row| row.rsid.trim().to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    gene_rows.retain(|row| trait_ids.contains(&row.rsid.trim().to_ascii_lowercase()));
    trait_rows.retain(|row| gene_ids.contains(&row.rsid.trim().to_ascii_lowercase()));
    gene_rows.append(&mut trait_rows);
    gene_rows
}

fn map_gwas_summary(association: &GwasAssociationSummary) -> Option<VariantGwasAssociation> {
    let allele = association.snp_allele.first();
    let rsid = allele
        .and_then(|row| row.rs_id.as_deref())
        .map(str::to_string)
        .or_else(|| {
            association
                .snp_effect_allele
                .iter()
                .find_map(|row| rsid_from_risk_allele(row))
        })?;
    let effect_size = association.or_per_copy_num.or(association.beta_num);
    let effect_type = association
        .or_per_copy_num
        .map(|_| "OR".to_string())
        .or_else(|| association.beta_num.map(|_| "beta".to_string()));
    let trait_name = association
        .efo_traits
        .iter()
        .filter_map(|row| row.efo_trait.as_deref())
        .map(str::trim)
        .find(|value| !value.is_empty())
        .or_else(|| {
            association
                .reported_trait
                .iter()
                .map(String::as_str)
                .find(|v| !v.trim().is_empty())
        })
        .map(str::to_string);
    let risk_allele = allele
        .and_then(|row| row.effect_allele.as_deref())
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| association.snp_effect_allele.first().cloned());
    Some(VariantGwasAssociation {
        rsid: rsid.to_ascii_lowercase(),
        trait_name,
        p_value: association.p_value,
        effect_size,
        effect_type,
        confidence_interval: association.range.clone(),
        risk_allele_frequency: association.risk_frequency,
        risk_allele,
        mapped_genes: association.mapped_genes.clone(),
        study_accession: association.accession_id.clone(),
        pmid: association.pubmed_id.clone(),
        author: association.first_author.clone(),
        sample_description: None,
    })
}

pub fn gwas_search_query_summary(filters: &GwasSearchFilters) -> String {
    let mut parts = Vec::new();
    if let Some(gene) = filters
        .gene
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("gene={gene}"));
    }
    if let Some(trait_query) = filters
        .trait_query
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("trait={trait_query}"));
    }
    if let Some(p_value) = filters.p_value {
        parts.push(format!("p_value={p_value}"));
    }
    parts.join(", ")
}

fn dedupe_gwas_rows(
    mut rows: Vec<VariantGwasAssociation>,
    limit: usize,
) -> Result<Vec<VariantGwasAssociation>, BioMcpError> {
    let mut seen = std::collections::HashSet::new();
    rows.retain(|row| {
        let key = format!(
            "{}|{}|{}",
            row.rsid.to_ascii_lowercase(),
            row.trait_name
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase(),
            row.study_accession
                .as_deref()
                .unwrap_or_default()
                .to_ascii_uppercase()
        );
        seen.insert(key)
    });

    rows.sort_by(|a, b| {
        a.p_value
            .unwrap_or(f64::INFINITY)
            .total_cmp(&b.p_value.unwrap_or(f64::INFINITY))
            .then_with(|| a.rsid.cmp(&b.rsid))
    });
    rows.truncate(limit);
    Ok(rows)
}

fn rsid_from_risk_allele(value: &str) -> Option<String> {
    let token = value.trim();
    if token.is_empty() {
        return None;
    }
    let prefix = token.split('-').next().unwrap_or(token).trim();
    if prefix.len() < 3 || !prefix.to_ascii_lowercase().starts_with("rs") {
        return None;
    }
    Some(prefix.to_ascii_lowercase())
}

fn association_rsid(association: &GwasAssociation, fallback: Option<&str>) -> Option<String> {
    if let Some(rsid) = association
        .snps
        .iter()
        .filter_map(|snp| snp.rs_id.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
    {
        return Some(rsid);
    }

    if let Some(rsid) = association
        .loci
        .iter()
        .flat_map(|locus| locus.strongest_risk_alleles.iter())
        .filter_map(|allele| allele.risk_allele_name.as_deref())
        .find_map(rsid_from_risk_allele)
    {
        return Some(rsid);
    }

    fallback
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
}

fn association_trait_name(association: &GwasAssociation) -> Option<String> {
    association
        .efo_traits
        .iter()
        .filter_map(|row| row.trait_field.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
        .or_else(|| {
            association
                .study
                .as_ref()
                .and_then(|study| study.disease_trait.as_ref())
                .and_then(|trait_row| trait_row.trait_field.as_deref())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        })
}

fn association_risk_allele(association: &GwasAssociation) -> Option<String> {
    association
        .loci
        .iter()
        .flat_map(|locus| locus.strongest_risk_alleles.iter())
        .filter_map(|allele| allele.risk_allele_name.as_deref())
        .map(str::trim)
        .find(|v| !v.is_empty())
        .map(str::to_string)
}

fn association_genes(association: &GwasAssociation) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for gene in association
        .loci
        .iter()
        .flat_map(|locus| locus.author_reported_genes.iter())
        .filter_map(|gene| gene.gene_name.as_deref())
    {
        let symbol = gene.trim();
        if symbol.is_empty() {
            continue;
        }
        let key = symbol.to_ascii_uppercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(symbol.to_string());
    }
    out
}

fn association_sample_description(association: &GwasAssociation) -> Option<String> {
    let study = association.study.as_ref()?;
    let mut parts = Vec::new();
    if let Some(v) = study
        .initial_sample_size
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("initial: {v}"));
    }
    if let Some(v) = study
        .replication_sample_size
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("na"))
    {
        parts.push(format!("replication: {v}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

fn map_gwas_association(
    association: &GwasAssociation,
    fallback_rsid: Option<&str>,
) -> Option<VariantGwasAssociation> {
    let rsid = association_rsid(association, fallback_rsid)?;
    let (effect_size, effect_type) = if let Some(v) = association.or_per_copy_num {
        (Some(v), Some("OR".to_string()))
    } else if let Some(v) = association.beta_num {
        (Some(v), Some("beta".to_string()))
    } else {
        (None, None)
    };

    let study_accession = association
        .study
        .as_ref()
        .and_then(|study| study.accession_id.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let pmid = association
        .study
        .as_ref()
        .and_then(|study| study.publication_info.as_ref())
        .and_then(|pubinfo| pubinfo.pubmed_id.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);
    let author = association
        .study
        .as_ref()
        .and_then(|study| study.publication_info.as_ref())
        .and_then(|pubinfo| pubinfo.author.as_ref())
        .and_then(|author| author.fullname.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    Some(VariantGwasAssociation {
        rsid,
        trait_name: association_trait_name(association),
        p_value: association.pvalue,
        effect_size,
        effect_type,
        confidence_interval: association
            .range
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string),
        risk_allele_frequency: association.risk_frequency,
        risk_allele: association_risk_allele(association),
        mapped_genes: association_genes(association),
        study_accession,
        pmid,
        author,
        sample_description: association_sample_description(association),
    })
}

pub(in crate::entities::variant) async fn add_gwas_section(
    variant: &mut Variant,
    query_id: &str,
) -> Result<(), BioMcpError> {
    variant.gwas.clear();
    variant.gwas_unavailable_reason = None;
    variant.supporting_pmids = Some(Vec::new());

    let fallback_rsid = parse_variant_id(query_id)
        .ok()
        .and_then(|parsed| match parsed {
            VariantIdFormat::RsId(rsid) => Some(rsid),
            _ => None,
        });

    let rsid = variant
        .rsid
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_ascii_lowercase)
        .or(fallback_rsid);

    let Some(rsid) = rsid else {
        variant.section_outcomes.complete(
            "gwas",
            SectionOutcome::inapplicable("An rsID is required for GWAS associations."),
        );
        return Ok(());
    };

    let client = match GwasClient::new() {
        Ok(client) => client,
        Err(err) if err.code() == "source_unavailable" => {
            mark_gwas_unavailable(variant);
            return Ok(());
        }
        Err(err) => return Err(err),
    };
    let associations = match client.associations_by_rsid(&rsid, 20).await {
        Ok(associations) => associations,
        Err(err) if err.code() == "source_unavailable" => {
            mark_gwas_unavailable(variant);
            return Ok(());
        }
        Err(err) => return Err(err),
    };
    let mut rows: Vec<VariantGwasAssociation> = associations
        .iter()
        .filter_map(|assoc| map_gwas_association(assoc, Some(&rsid)))
        .collect();
    rows = dedupe_gwas_rows(rows, 10)?;
    let supporting_pmids = collect_supporting_pmids(&rows);
    variant.gwas = rows;
    variant.supporting_pmids = Some(supporting_pmids);
    let outcome = if variant.gwas.is_empty() {
        SectionOutcome::empty("GWAS Catalog")
    } else {
        SectionOutcome::data("GWAS Catalog")
    };
    variant.section_outcomes.complete("gwas", outcome);
    Ok(())
}

pub(in crate::entities::variant) fn mark_gwas_unavailable(variant: &mut Variant) {
    variant.supporting_pmids = None;
    variant.gwas_unavailable_reason = Some("GWAS association data temporarily unavailable.".into());
    variant.section_outcomes.complete(
        "gwas",
        SectionOutcome::unavailable("GWAS association data is temporarily unavailable."),
    );
}

fn collect_supporting_pmids(rows: &[VariantGwasAssociation]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for pmid in rows.iter().filter_map(|row| row.pmid.as_deref()) {
        let pmid = pmid.trim();
        if pmid.is_empty() {
            continue;
        }
        let key = pmid.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(pmid.to_string());
        }
    }

    out
}

#[cfg(test)]
mod tests;
