//! Variant-specific article route union, provenance, ranking, and pagination.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use clap::ValueEnum;
use serde::Serialize;

use crate::entities::variant::{
    NormalizedVariantAliases, RequestedVariantIdentity, VariantArticleResolutionContext,
    VariantResolutionStatus, VariantSearchResolution,
};
use crate::error::BioMcpError;

use super::backends::search_pubtator_page;
use super::candidates::{
    ArticleCandidate, article_candidate_from_row, merge_article_candidate_pool,
    stable_article_identifier,
};
use super::enrichment::{
    enrich_article_search_rows_with_semantic_scholar,
    enrich_visible_article_search_rows_with_article_base,
};
use super::query::resolve_variant_entity_tokens;
use super::search::{
    VARIANT_ENTITY_RETRIEVAL_PATH, VARIANT_FALLBACK_RETRIEVAL_PATH, search_federated_page,
};
use super::{
    ArticleRankingOptions, ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleVariantIntent, MAX_FEDERATED_FETCH_RESULTS,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VariantArticleStrategy {
    #[default]
    Union,
    Annotation,
    Lexical,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct VariantArticleProvenance {
    pub route: String,
    pub source: String,
    pub matched_alias: Option<String>,
    pub native_position: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleSourceStatus {
    pub route: String,
    pub source: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticlePagination {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: Option<usize>,
    pub has_more: bool,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleRow {
    #[serde(flatten)]
    pub article: ArticleSearchResult,
    pub requested_variant: RequestedVariantIdentity,
    pub matched_aliases: Vec<String>,
    pub retrieval_routes: Vec<String>,
    pub sources: Vec<String>,
    pub rank: usize,
    pub provenance: Vec<VariantArticleProvenance>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantArticleResponse {
    pub requested_variant: RequestedVariantIdentity,
    pub resolution: VariantSearchResolution,
    pub strategy: VariantArticleStrategy,
    pub complete: bool,
    pub truncated: bool,
    pub pagination: VariantArticlePagination,
    pub source_status: Vec<VariantArticleSourceStatus>,
    pub retrieval_path: &'static str,
    pub results: Vec<VariantArticleRow>,
}

#[derive(Debug)]
pub struct VariantArticleOutcome {
    pub response: VariantArticleResponse,
    pub hard_error: bool,
}

fn article_filters() -> ArticleSearchFilters {
    ArticleSearchFilters {
        gene: None,
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: None,
        date_from: None,
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: true,
        exclude_retracted: true,
        max_per_source: None,
        sort: ArticleSort::Relevance,
        ranking: ArticleRankingOptions::default(),
    }
}

fn source_name(source: ArticleSource) -> &'static str {
    match source {
        ArticleSource::PubTator => "pubtator",
        ArticleSource::EuropePmc => "europepmc",
        ArticleSource::SemanticScholar => "semanticscholar",
        ArticleSource::PubMed => "pubmed",
        ArticleSource::LitSense2 => "litsense2",
    }
}

fn status(route: &str, source: &str, state: &str) -> VariantArticleSourceStatus {
    VariantArticleSourceStatus {
        route: route.to_string(),
        source: source.to_string(),
        status: state.to_string(),
        detail: None,
    }
}

fn candidate_with_provenance(
    row: ArticleSearchResult,
    route: &str,
    source: &str,
    matched_alias: Option<String>,
) -> ArticleCandidate {
    let native_position = row.source_local_position.saturating_add(1);
    let mut candidate = article_candidate_from_row(row);
    candidate.variant_provenance.push(VariantArticleProvenance {
        route: route.to_string(),
        source: source.to_string(),
        matched_alias,
        native_position,
    });
    candidate
}

fn exact_aliases(context: &VariantArticleResolutionContext) -> Vec<String> {
    let mut aliases = BTreeSet::new();
    let gene = context
        .source_identity
        .as_ref()
        .and_then(|source| source.genes.first())
        .or(context.requested.gene.as_ref());
    let mut insert_change = |change: &str| {
        let change = change.trim();
        if change.is_empty() {
            return;
        }
        if let Some(gene) = gene {
            aliases.insert(format!("{gene} {change}"));
        } else {
            aliases.insert(change.to_string());
        }
    };
    if let Some(change) = context.requested.protein_change.as_deref() {
        insert_change(change);
    }
    if let Some(change) = context.requested.coding_change.as_deref() {
        insert_change(change);
    }
    if let Some(source) = context.source_identity.as_ref() {
        for change in source.protein_changes.iter().chain(&source.coding_changes) {
            insert_change(change);
        }
        if !source.genomic_id.trim().is_empty() {
            aliases.insert(source.genomic_id.trim().to_string());
        }
        aliases.extend(source.rsids.iter().map(|value| value.trim().to_string()));
    }
    aliases.retain(|value| !value.is_empty());
    aliases.into_iter().collect()
}

fn combined_normalized_aliases(
    context: &VariantArticleResolutionContext,
) -> NormalizedVariantAliases {
    let mut aliases = context.requested.normalized_aliases();
    if let Some(source) = context.source_identity.as_ref() {
        aliases.protein_changes.extend(
            source
                .protein_changes
                .iter()
                .filter_map(|value| crate::entities::variant::normalize_protein_change(value)),
        );
        aliases.coding_changes.extend(source.coding_changes.clone());
        if !source.genomic_id.trim().is_empty() {
            aliases.genomic_ids.push(source.genomic_id.clone());
        }
        aliases.rsids.extend(source.rsids.clone());
    }
    aliases.protein_changes.sort();
    aliases.protein_changes.dedup();
    aliases.coding_changes.sort();
    aliases.coding_changes.dedup();
    aliases.genomic_ids.sort();
    aliases.genomic_ids.dedup();
    aliases.rsids.sort();
    aliases.rsids.dedup();
    aliases
}

fn primary_exact_alias(context: &VariantArticleResolutionContext) -> Option<String> {
    let gene = context.requested.gene.as_deref();
    if let Some(change) = context.requested.protein_change.as_deref() {
        return Some(
            gene.map(|gene| format!("{gene} {change}"))
                .unwrap_or_else(|| change.to_string()),
        );
    }
    if let Some(change) = context.requested.coding_change.as_deref() {
        return Some(
            gene.map(|gene| format!("{gene} {change}"))
                .unwrap_or_else(|| change.to_string()),
        );
    }
    context
        .source_identity
        .as_ref()
        .map(|source| source.genomic_id.clone())
        .filter(|value| !value.is_empty())
        .or_else(|| context.requested.rsid.clone())
}

async fn annotation_candidates(
    input: &str,
    context: &VariantArticleResolutionContext,
) -> Result<(Vec<ArticleCandidate>, bool), BioMcpError> {
    let pubtator = crate::sources::pubtator::PubTatorClient::new()?;
    let tokens = resolve_variant_entity_tokens(&pubtator, input, &context.requested).await?;
    let mut candidates = Vec::new();
    let mut incomplete = false;
    for token in tokens {
        let mut filters = article_filters();
        filters.variant = Some(ArticleVariantIntent {
            original: input.to_string(),
            gene: context.requested.gene.clone(),
            change: context.requested.protein_change.clone(),
            entity_id: Some(token.entity_id),
        });
        let page = search_pubtator_page(&filters, MAX_FEDERATED_FETCH_RESULTS, 0).await?;
        incomplete |= page.total.is_some_and(|total| total > page.results.len());
        for row in page.results {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
                return Ok((candidates, true));
            }
            candidates.push(candidate_with_provenance(
                row,
                "pubtator_variant",
                "pubtator",
                Some(token.matched_alias.clone()),
            ));
        }
    }
    Ok((candidates, incomplete))
}

async fn lexical_candidates(
    context: &VariantArticleResolutionContext,
) -> Result<(Vec<ArticleCandidate>, bool), BioMcpError> {
    let searches = exact_aliases(context).into_iter().map(|alias| async move {
        let mut filters = article_filters();
        filters.keyword = Some(alias.clone());
        search_federated_page(&filters, 100, 0)
            .await
            .map(|page| (alias, page))
    });
    let mut candidates = Vec::new();
    let mut incomplete = false;
    for result in futures::future::join_all(searches).await {
        let (alias, page) = result?;
        incomplete |= page.source_status.iter().any(|status| {
            !status
                .message
                .as_deref()
                .is_some_and(|message| message.contains("enrichment"))
                && matches!(
                    status.status,
                    Some(
                        ArticleSourceAvailability::Degraded
                            | ArticleSourceAvailability::Unavailable
                    )
                )
        });
        for row in page.results {
            if candidates.len() >= MAX_FEDERATED_FETCH_RESULTS {
                return Ok((candidates, true));
            }
            let sources = if row.matched_sources.is_empty() {
                vec![row.source]
            } else {
                row.matched_sources.clone()
            };
            let mut candidate = article_candidate_from_row(row);
            for source in sources {
                candidate.variant_provenance.push(VariantArticleProvenance {
                    route: "exact_lexical".to_string(),
                    source: source_name(source).to_string(),
                    matched_alias: Some(alias.clone()),
                    native_position: candidate.row.source_local_position.saturating_add(1),
                });
            }
            candidates.push(candidate);
        }
    }
    Ok((candidates, incomplete))
}

fn pmid_seed(pmid: String) -> ArticleSearchResult {
    ArticleSearchResult {
        pmid,
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: String::new(),
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![ArticleSource::PubMed],
        score: None,
        is_retracted: None,
        abstract_snippet: None,
        ranking: None,
        normalized_title: String::new(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }
}

async fn citation_candidates(
    context: &VariantArticleResolutionContext,
) -> Result<Vec<ArticleCandidate>, BioMcpError> {
    let Some(source_id) = context.source_id.as_deref() else {
        return Ok(Vec::new());
    };
    let hit = crate::sources::myvariant::MyVariantClient::new()?
        .get(source_id)
        .await?;
    let matched_alias = primary_exact_alias(context);
    Ok(crate::sources::myvariant::civic_pubmed_ids(&hit)
        .into_iter()
        .enumerate()
        .map(|(position, pmid)| {
            let mut row = pmid_seed(pmid);
            row.source_local_position = position;
            candidate_with_provenance(row, "source_citation", "civic", matched_alias.clone())
        })
        .collect())
}

async fn fallback_candidates(input: &str) -> Result<(Vec<ArticleCandidate>, bool), BioMcpError> {
    let mut filters = article_filters();
    filters.keyword = Some(input.to_string());
    let page = search_pubtator_page(&filters, MAX_FEDERATED_FETCH_RESULTS, 0).await?;
    let incomplete = page.total.is_some_and(|total| total > page.results.len());
    Ok((
        page.results
            .into_iter()
            .map(|row| candidate_with_provenance(row, "best_effort_free_text", "pubtator", None))
            .collect(),
        incomplete,
    ))
}

fn route_score(candidate: &ArticleCandidate) -> f64 {
    let best = candidate.variant_provenance.iter().fold(
        BTreeMap::<(&str, &str), usize>::new(),
        |mut best, fact| {
            best.entry((&fact.route, &fact.source))
                .and_modify(|position| *position = (*position).min(fact.native_position))
                .or_insert(fact.native_position);
            best
        },
    );
    best.values()
        .map(|position| 1.0 / (60.0 + *position as f64))
        .sum()
}

fn rank_candidates(candidates: &mut [ArticleCandidate]) {
    candidates.sort_by(|left, right| {
        let left_exact = left
            .variant_provenance
            .iter()
            .any(|fact| fact.route != "best_effort_free_text");
        let right_exact = right
            .variant_provenance
            .iter()
            .any(|fact| fact.route != "best_effort_free_text");
        right_exact
            .cmp(&left_exact)
            .then_with(|| {
                route_score(right)
                    .partial_cmp(&route_score(left))
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| {
                left.variant_provenance
                    .iter()
                    .map(|fact| fact.native_position)
                    .min()
                    .cmp(
                        &right
                            .variant_provenance
                            .iter()
                            .map(|fact| fact.native_position)
                            .min(),
                    )
            })
            .then_with(|| {
                stable_article_identifier(&left.row).cmp(&stable_article_identifier(&right.row))
            })
    });
}

async fn enrich_candidates(candidates: &mut [ArticleCandidate]) {
    let mut rows = candidates
        .iter()
        .map(|candidate| candidate.row.clone())
        .collect::<Vec<_>>();
    let _ = enrich_article_search_rows_with_semantic_scholar(&mut rows).await;
    enrich_visible_article_search_rows_with_article_base(&mut rows).await;
    for (candidate, row) in candidates.iter_mut().zip(rows) {
        candidate.row = row;
    }
}

pub async fn search_variant_articles(
    input: &str,
    strategy: VariantArticleStrategy,
    limit: usize,
    offset: usize,
) -> Result<VariantArticleOutcome, BioMcpError> {
    if limit == 0 || limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".to_string(),
        ));
    }
    let mut context = crate::entities::variant::resolve_article_variant(input).await?;
    context.resolution.normalized_aliases = combined_normalized_aliases(&context);
    if !context.available {
        return Ok(VariantArticleOutcome {
            response: VariantArticleResponse {
                requested_variant: context.requested,
                resolution: context.resolution,
                strategy,
                complete: false,
                truncated: true,
                pagination: VariantArticlePagination {
                    offset,
                    limit,
                    returned: 0,
                    total: None,
                    has_more: false,
                    next_page_token: None,
                },
                source_status: vec![status("resolution", "myvariant", "unavailable")],
                retrieval_path: "variant resolution unavailable",
                results: Vec::new(),
            },
            hard_error: true,
        });
    }
    let resolved = matches!(context.resolution.status, VariantResolutionStatus::Resolved);
    let mut candidates = Vec::new();
    let mut statuses = Vec::new();
    let mut succeeded_routes = 0usize;
    let mut failed_routes = 0usize;

    if !resolved {
        match strategy {
            VariantArticleStrategy::Union => match fallback_candidates(input).await {
                Ok((rows, incomplete)) => {
                    candidates.extend(rows);
                    statuses.push(status(
                        "best_effort_free_text",
                        "pubtator",
                        if incomplete { "degraded" } else { "ok" },
                    ));
                    succeeded_routes += 1;
                    failed_routes += usize::from(incomplete);
                }
                Err(_) => {
                    statuses.push(status("best_effort_free_text", "pubtator", "unavailable"));
                    failed_routes += 1;
                }
            },
            VariantArticleStrategy::Annotation => {
                statuses.push(status("pubtator_variant", "pubtator", "ok"));
                succeeded_routes += 1;
            }
            VariantArticleStrategy::Lexical => {
                statuses.push(status("exact_lexical", "federated", "ok"));
                succeeded_routes += 1;
            }
        }
    } else {
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Annotation
        ) {
            match annotation_candidates(input, &context).await {
                Ok((rows, incomplete)) => {
                    candidates.extend(rows);
                    statuses.push(status(
                        "pubtator_variant",
                        "pubtator",
                        if incomplete { "degraded" } else { "ok" },
                    ));
                    succeeded_routes += 1;
                    failed_routes += usize::from(incomplete);
                }
                Err(_) => {
                    statuses.push(status("pubtator_variant", "pubtator", "unavailable"));
                    failed_routes += 1;
                }
            }
        }
        if matches!(
            strategy,
            VariantArticleStrategy::Union | VariantArticleStrategy::Lexical
        ) {
            match lexical_candidates(&context).await {
                Ok((rows, incomplete)) => {
                    candidates.extend(rows);
                    statuses.push(status(
                        "exact_lexical",
                        "federated",
                        if incomplete { "degraded" } else { "ok" },
                    ));
                    succeeded_routes += 1;
                    failed_routes += usize::from(incomplete);
                }
                Err(_) => {
                    statuses.push(status("exact_lexical", "federated", "unavailable"));
                    failed_routes += 1;
                }
            }
        }
        if strategy == VariantArticleStrategy::Union {
            match citation_candidates(&context).await {
                Ok(rows) => {
                    candidates.extend(rows);
                    statuses.push(status("source_citation", "myvariant", "ok"));
                    succeeded_routes += 1;
                }
                Err(_) => {
                    statuses.push(status("source_citation", "myvariant", "unavailable"));
                    failed_routes += 1;
                }
            }
        }
    }

    let mut candidates = merge_article_candidate_pool(candidates);
    rank_candidates(&mut candidates);
    enrich_candidates(&mut candidates).await;
    let total_candidates = candidates.len();
    let complete = failed_routes == 0;
    let hard_error = succeeded_routes == 0 && failed_routes > 0;
    let has_more = offset.saturating_add(limit) < total_candidates;
    let truncated = !complete || offset > 0 || has_more;
    let rows = candidates
        .into_iter()
        .enumerate()
        .skip(offset)
        .take(limit)
        .map(|(index, candidate)| {
            let matched_aliases = candidate
                .variant_provenance
                .iter()
                .filter_map(|fact| fact.matched_alias.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let retrieval_routes = candidate
                .variant_provenance
                .iter()
                .map(|fact| fact.route.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let sources = candidate
                .variant_provenance
                .iter()
                .map(|fact| fact.source.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            VariantArticleRow {
                article: candidate.row,
                requested_variant: context.requested.clone(),
                matched_aliases,
                retrieval_routes,
                sources,
                rank: index + 1,
                provenance: candidate.variant_provenance,
            }
        })
        .collect::<Vec<_>>();
    statuses.sort_by(|left, right| (&left.route, &left.source).cmp(&(&right.route, &right.source)));
    let retrieval_path = if !resolved {
        VARIANT_FALLBACK_RETRIEVAL_PATH
    } else if strategy == VariantArticleStrategy::Annotation {
        VARIANT_ENTITY_RETRIEVAL_PATH
    } else if strategy == VariantArticleStrategy::Lexical {
        "resolved exact-alias article search"
    } else {
        "resolved exact-variant route union"
    };
    Ok(VariantArticleOutcome {
        response: VariantArticleResponse {
            requested_variant: context.requested,
            resolution: context.resolution,
            strategy,
            complete,
            truncated,
            pagination: VariantArticlePagination {
                offset,
                limit,
                returned: rows.len(),
                total: complete.then_some(total_candidates),
                has_more,
                next_page_token: None,
            },
            source_status: statuses,
            retrieval_path,
            results: rows,
        },
        hard_error,
    })
}
