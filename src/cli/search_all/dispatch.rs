//! Search-all section execution, timeout handling, and backend result shaping.

use std::collections::HashSet;
use std::time::Duration;

use serde_json::{Value, json};

use crate::error::BioMcpError;

use super::format::{
    dedupe_gwas_rows, refine_drug_results, to_json_array, trait_matches_disease_query,
    variant_significance_rank,
};
use super::links::{build_links, canonical_search_command};
use super::plan::{PreparedInput, article_filters};
use super::{MAX_SEARCH_ALL_LIMIT, SearchAllLink, SearchAllSection, SectionKind};

const SECTION_TIMEOUT: Duration = Duration::from_secs(12);
// Article fan-out can legitimately run longer than the other sections when the
// shared Semantic Scholar pool throttles unauthenticated enrichment.
const ARTICLE_SECTION_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub(super) struct SectionResult {
    rows: Vec<Value>,
    total: Option<usize>,
    note: Option<String>,
}

impl SectionResult {
    fn new(rows: Vec<Value>, total: Option<usize>) -> Self {
        Self {
            rows,
            total,
            note: None,
        }
    }
}

pub(super) fn section_timeout(kind: SectionKind) -> Duration {
    match kind {
        SectionKind::Article => ARTICLE_SECTION_TIMEOUT,
        _ => SECTION_TIMEOUT,
    }
}

pub(super) async fn dispatch_section(kind: SectionKind, input: &PreparedInput) -> SearchAllSection {
    let search_self = canonical_search_command(kind, input, input.limit);
    let timeout = section_timeout(kind);
    let effective_limit = section_fetch_limit(kind, input);
    let section_result =
        tokio::time::timeout(timeout, run_section(kind, input, effective_limit)).await;

    match section_result {
        Ok(Ok(section_result)) => {
            let links = build_links(kind, input, &section_result.rows, &search_self);
            SearchAllSection {
                entity: kind.entity().to_string(),
                label: kind.label().to_string(),
                count: section_result.rows.len(),
                total: section_result.total,
                error: None,
                note: section_result.note,
                results: section_result.rows,
                links,
            }
        }
        Ok(Err(err)) => SearchAllSection {
            entity: kind.entity().to_string(),
            label: kind.label().to_string(),
            count: 0,
            total: None,
            error: Some(err.to_string()),
            note: None,
            results: Vec::new(),
            links: vec![SearchAllLink {
                rel: "search.retry".to_string(),
                title: format!("Retry {} search", kind.entity()),
                command: search_self,
            }],
        },
        Err(_) => SearchAllSection {
            entity: kind.entity().to_string(),
            label: kind.label().to_string(),
            count: 0,
            total: None,
            error: Some(format!(
                "{} search timed out after {}s",
                kind.entity(),
                timeout.as_secs()
            )),
            note: None,
            results: Vec::new(),
            links: vec![SearchAllLink {
                rel: "search.retry".to_string(),
                title: format!("Retry {} search", kind.entity()),
                command: search_self,
            }],
        },
    }
}

async fn run_section(
    kind: SectionKind,
    input: &PreparedInput,
    limit: usize,
) -> Result<SectionResult, BioMcpError> {
    match kind {
        SectionKind::Gene => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for gene search.".into())
            })?;
            let filters = crate::entities::gene::GeneSearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let page = crate::entities::gene::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Variant => {
            if let Some(variant_id) = input
                .variant_context
                .as_ref()
                .and_then(|ctx| (ctx.parsed_gene.is_none()).then_some(ctx.raw.as_str()))
            {
                let row = crate::entities::variant::get(variant_id, &[]).await?;
                return Ok(SectionResult::new(to_json_array(vec![row])?, Some(1)));
            }

            let filters = crate::entities::variant::VariantSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                hgvsp: input
                    .variant_context
                    .as_ref()
                    .and_then(|ctx| ctx.parsed_change.clone()),
                condition: input.disease.clone(),
                therapy: input.drug.clone(),
                ..Default::default()
            };
            let has_filter = filters
                .gene
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .hgvsp
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .condition
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .therapy
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
            if !has_filter {
                return Err(BioMcpError::InvalidArgument(
                    "No filters available for variant search.".into(),
                ));
            }
            let page = crate::entities::variant::search_page(&filters, limit, 0).await?;
            let mut rows = page.results;
            let mut total = page.total;
            let mut note = None;

            if rows.is_empty() && input.gene_anchor().is_some() && input.disease.is_some() {
                let fallback_filters = crate::entities::variant::VariantSearchFilters {
                    gene: filters.gene.clone(),
                    hgvsp: filters.hgvsp.clone(),
                    therapy: filters.therapy.clone(),
                    ..Default::default()
                };
                let fallback_page =
                    crate::entities::variant::search_page(&fallback_filters, limit, 0).await?;
                rows = fallback_page.results;
                total = fallback_page.total;
                if !rows.is_empty() {
                    note = Some(
                        "No disease-filtered variants found; showing top gene variants."
                            .to_string(),
                    );
                }
            }

            if input.gene_anchor().is_some() && input.disease.is_some() {
                rows.sort_by(|a, b| {
                    variant_significance_rank(a.significance.as_deref())
                        .cmp(&variant_significance_rank(b.significance.as_deref()))
                });
            }
            Ok(SectionResult {
                rows: to_json_array(rows)?,
                total,
                note,
            })
        }
        SectionKind::Disease => {
            let query = input.disease.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "No disease anchor available for disease search.".into(),
                )
            })?;
            let filters = crate::entities::disease::DiseaseSearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let page = crate::entities::disease::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Drug => {
            let filters = crate::entities::drug::DrugSearchFilters {
                query: input.drug_query().map(str::to_string),
                target: input.gene_anchor().map(str::to_string),
                indication: input.disease.clone(),
                ..Default::default()
            };
            let has_filter = filters
                .query
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .target
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || filters
                    .indication
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty());
            if !has_filter {
                return Err(BioMcpError::InvalidArgument(
                    "No filters available for drug search.".into(),
                ));
            }
            let page = crate::entities::drug::search_page(&filters, limit, 0).await?;
            let rows = refine_drug_results(page.results, input.drug.as_deref(), limit);
            let total = if input.drug.is_some() {
                Some(rows.len())
            } else {
                page.total
            };
            Ok(SectionResult::new(to_json_array(rows)?, total))
        }
        SectionKind::Trial => {
            let base_filters = crate::entities::trial::TrialSearchFilters {
                condition: input.trial_condition_query().map(str::to_string),
                intervention: input.drug.clone(),
                biomarker: input.gene_anchor().map(str::to_string),
                mutation: input.variant_trial_query(),
                date_from: input.since.clone(),
                source: crate::entities::trial::TrialSource::ClinicalTrialsGov,
                ..Default::default()
            };
            let preferred_filters = crate::entities::trial::TrialSearchFilters {
                status: Some(
                    "RECRUITING,ACTIVE_NOT_RECRUITING,ENROLLING_BY_INVITATION,NOT_YET_RECRUITING"
                        .to_string(),
                ),
                ..base_filters.clone()
            };

            let mut rows = Vec::new();
            let mut total = None;
            let mut preferred_error: Option<BioMcpError> = None;

            match crate::entities::trial::search_page(&preferred_filters, limit, 0, None).await {
                Ok(page) => {
                    total = page.total;
                    rows = page.results;
                }
                Err(err) => {
                    preferred_error = Some(err);
                }
            }

            if rows.len() < limit {
                let backfill_fetch = if input.counts_only {
                    limit
                } else {
                    input.limit.saturating_mul(3).min(MAX_SEARCH_ALL_LIMIT)
                };
                match crate::entities::trial::search_page(&base_filters, backfill_fetch, 0, None)
                    .await
                {
                    Ok(page) => {
                        total = total.or(page.total);
                        rows = merge_trial_backfill_rows(rows, page.results, limit);
                    }
                    Err(err) if rows.is_empty() => {
                        return Err(preferred_error.unwrap_or(err));
                    }
                    Err(_) => {}
                }
            }

            if rows.is_empty()
                && let Some(err) = preferred_error
            {
                return Err(err);
            }

            rows.truncate(limit);
            Ok(SectionResult::new(to_json_array(rows)?, total))
        }
        SectionKind::Article => {
            let filters = article_filters(input);
            let page = crate::entities::article::search_page(
                &filters,
                limit,
                0,
                crate::entities::article::ArticleSourceFilter::All,
            )
            .await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Pathway => {
            let query = input.gene_anchor().ok_or_else(|| {
                BioMcpError::InvalidArgument("No gene anchor available for pathway search.".into())
            })?;
            let filters = crate::entities::pathway::PathwaySearchFilters {
                query: Some(query.to_string()),
                ..Default::default()
            };
            let pathway_limit = limit.min(25);
            let (results, total) =
                crate::entities::pathway::search_with_filters(&filters, pathway_limit).await?;
            Ok(SectionResult::new(to_json_array(results)?, total))
        }
        SectionKind::Pgx => {
            let filters = crate::entities::pgx::PgxSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                drug: input.drug.clone(),
                ..Default::default()
            };
            let page = crate::entities::pgx::search_page(&filters, limit, 0).await?;
            Ok(SectionResult::new(to_json_array(page.results)?, page.total))
        }
        SectionKind::Gwas => {
            let trait_query = input.disease.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument("No disease anchor available for GWAS search.".into())
            })?;
            let filters = crate::entities::variant::GwasSearchFilters {
                gene: input.gene_anchor().map(str::to_string),
                trait_query: Some(trait_query.to_string()),
                region: None,
                p_value: None,
            };
            let page = crate::entities::variant::search_gwas_page(&filters, limit, 0).await?;
            let mut rows = page.results;
            if let Some(disease) = input.disease.as_deref() {
                rows.retain(|row| trait_matches_disease_query(row, disease));
            }
            let mut rows = dedupe_gwas_rows(rows);
            rows.truncate(limit);
            let total = rows.len();
            Ok(SectionResult::new(to_json_array(rows)?, Some(total)))
        }
        SectionKind::AdverseEvent => {
            let drug = input.drug.as_deref().ok_or_else(|| {
                BioMcpError::InvalidArgument(
                    "No drug anchor available for adverse-event search.".into(),
                )
            })?;
            let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                drug: Some(drug.to_string()),
                since: input.since.clone(),
                ..Default::default()
            };
            let grouped = crate::entities::adverse_event::search_count(
                &filters,
                "patient.reaction.reactionmeddrapt",
                limit,
            )
            .await?;
            let total = grouped.buckets.len();
            let rows = grouped
                .buckets
                .into_iter()
                .map(|bucket| {
                    json!({
                        "reaction": bucket.value,
                        "count": bucket.count
                    })
                })
                .collect::<Vec<_>>();
            Ok(SectionResult::new(rows, Some(total)))
        }
    }
}

pub(super) fn section_fetch_limit(kind: SectionKind, input: &PreparedInput) -> usize {
    if !input.counts_only {
        return input.limit;
    }

    // Counts-only does not skip backend fetches entirely because some sections derive
    // counts locally and markdown still needs stable follow-up commands, but it can
    // safely collapse sections backed by stable totals to a single fetched row.
    match kind {
        SectionKind::Gene | SectionKind::Disease | SectionKind::Trial | SectionKind::Pgx => 1,
        SectionKind::Article if !input.debug_plan => 1,
        SectionKind::Variant
        | SectionKind::Drug
        | SectionKind::Pathway
        | SectionKind::Gwas
        | SectionKind::AdverseEvent
        | SectionKind::Article => input.limit,
    }
}

pub(super) fn merge_trial_backfill_rows(
    mut preferred: Vec<crate::entities::trial::TrialSearchResult>,
    backfill: Vec<crate::entities::trial::TrialSearchResult>,
    limit: usize,
) -> Vec<crate::entities::trial::TrialSearchResult> {
    preferred.truncate(limit);
    if preferred.len() >= limit {
        return preferred;
    }

    let mut seen = preferred
        .iter()
        .map(|row| row.nct_id.clone())
        .collect::<HashSet<_>>();
    for row in backfill {
        if preferred.len() >= limit {
            break;
        }
        if seen.insert(row.nct_id.clone()) {
            preferred.push(row);
        }
    }
    preferred
}
