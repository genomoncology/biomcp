//! Article search-result enrichment and visible-row fallback helpers.

use std::collections::HashMap;

use crate::entities::SearchPage;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{SemanticScholarClient, SemanticScholarPaper};

use super::candidates::finalize_article_candidates;
use super::detail::parse_pmid;
use super::detail::retained::{
    resolve_article_from_pmid_with_context, resolve_variant_article_from_pmid,
};
use super::{
    Article, ArticleSearchFilters, ArticleSearchResult, ArticleSearchTiming, ArticleSource,
    ArticleSourceAvailability, ArticleSourceStatus, SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS,
};

/// The overall article-search deadline active on this task, when the caller
/// is a plain search page. Variant-article work carries its deadline on the
/// execution context instead, so this seam only applies outside that path.
fn plain_article_search_deadline_elapsed(
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
) -> bool {
    execution.is_none()
        && crate::sources::current_variant_article_deadline()
            .is_some_and(|deadline| deadline.is_exhausted())
}

fn article_search_deadline_status(source: ArticleSource, stage: &str) -> ArticleSourceStatus {
    ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(ArticleSourceAvailability::Degraded),
        message: Some(format!("article search deadline elapsed during {stage}")),
    }
}

/// What an enrichment pass produced besides the finalized page: statuses for
/// sources that degraded mid-pass and the per-stage wall-clock timings.
pub(super) struct ArticleEnrichmentOutcome {
    pub(super) page: SearchPage<ArticleSearchResult>,
    pub(super) semantic_scholar_status: Option<ArticleSourceStatus>,
    pub(super) statuses: Vec<ArticleSourceStatus>,
    pub(super) timings: Vec<ArticleSearchTiming>,
}

fn article_search_semantic_scholar_lookup_id(row: &ArticleSearchResult) -> Option<String> {
    let pmid = row.pmid.trim();
    if !pmid.is_empty() {
        return Some(format!("PMID:{pmid}"));
    }
    row.doi
        .as_deref()
        .map(str::trim)
        .filter(|doi| !doi.is_empty())
        .map(|doi| format!("DOI:{doi}"))
}

fn article_search_row_needs_semantic_scholar_enrichment(row: &ArticleSearchResult) -> bool {
    row.source != ArticleSource::SemanticScholar
        && (row.citation_count.is_none()
            || row.influential_citation_count.is_none()
            || row
                .abstract_snippet
                .as_deref()
                .is_none_or(|snippet| snippet.trim().is_empty())
            || row.normalized_abstract.trim().is_empty())
}

fn merge_semantic_scholar_search_citation(target: &mut Option<u64>, incoming: Option<u64>) {
    match (*target, incoming) {
        (None, Some(value)) | (Some(0), Some(value)) => *target = Some(value),
        _ => {}
    }
}

fn merge_article_search_row_abstract_text(row: &mut ArticleSearchResult, abstract_text: &str) {
    let cleaned_abstract = crate::transform::article::clean_abstract(abstract_text);
    if cleaned_abstract.is_empty() {
        return;
    }

    if row
        .abstract_snippet
        .as_deref()
        .is_none_or(|snippet| snippet.trim().is_empty())
    {
        row.abstract_snippet =
            crate::transform::article::article_search_abstract_snippet(&cleaned_abstract);
    }
    if row.normalized_abstract.trim().is_empty() {
        row.normalized_abstract =
            crate::transform::article::normalize_article_search_text(&cleaned_abstract);
    }
}

fn merge_article_search_row_with_semantic_scholar(
    row: &mut ArticleSearchResult,
    paper: &SemanticScholarPaper,
) {
    merge_semantic_scholar_search_citation(&mut row.citation_count, paper.citation_count);
    merge_semantic_scholar_search_citation(
        &mut row.influential_citation_count,
        paper.influential_citation_count,
    );

    let Some(abstract_text) = paper.abstract_text.as_deref() else {
        return;
    };
    merge_article_search_row_abstract_text(row, abstract_text);
}

pub(super) async fn enrich_article_search_rows_with_semantic_scholar(
    rows: &mut [ArticleSearchResult],
) -> Option<ArticleSourceStatus> {
    enrich_article_search_rows_with_semantic_scholar_context(rows, None).await
}

pub(super) async fn enrich_article_search_rows_with_semantic_scholar_context(
    rows: &mut [ArticleSearchResult],
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
) -> Option<ArticleSourceStatus> {
    let mut lookup_ids = Vec::new();
    let mut lookup_positions: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, row) in rows.iter().enumerate() {
        if !article_search_row_needs_semantic_scholar_enrichment(row) {
            continue;
        }
        let Some(lookup_id) = article_search_semantic_scholar_lookup_id(row) else {
            continue;
        };
        match lookup_positions.get_mut(&lookup_id) {
            Some(positions) => positions.push(idx),
            None => {
                lookup_positions.insert(lookup_id.clone(), vec![idx]);
                lookup_ids.push(lookup_id);
            }
        }
    }

    if let Some(execution) = execution {
        execution.set_route_unit_plan(
            "enrichment",
            "semanticscholar",
            Some(
                lookup_ids
                    .len()
                    .div_ceil(SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS),
            ),
        );
    }
    if lookup_ids.is_empty() {
        return None;
    }

    let mut first_unit = match execution {
        Some(execution) => {
            execution
                .begin_provider_unit("enrichment", "semanticscholar")
                .await
        }
        None => None,
    };
    if execution.is_some() && first_unit.is_none() {
        return Some(ArticleSourceStatus {
            source: ArticleSource::SemanticScholar,
            enabled: true,
            auth_mode: None,
            status: Some(ArticleSourceAvailability::Degraded),
            message: Some("Variant article work budget exhausted".into()),
        });
    }
    let client = match match execution {
        Some(execution) => SemanticScholarClient::new_with_deadline(execution.deadline()).await,
        None => match crate::sources::current_variant_article_deadline() {
            Some(deadline) => SemanticScholarClient::new_with_deadline(&deadline).await,
            None => SemanticScholarClient::new(),
        },
    } {
        Ok(client) => client,
        Err(err) => {
            if let Some(unit) = first_unit.take() {
                unit.record_error(&err);
            }
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::SEMANTIC_SCHOLAR,
                "initialize article search enrichment",
            );
            if plain_article_search_deadline_elapsed(execution)
                && super::search::is_search_deadline_error(&err)
            {
                return Some(article_search_deadline_status(
                    ArticleSource::SemanticScholar,
                    "initialization",
                ));
            }
            return Some(ArticleSourceStatus {
                source: ArticleSource::SemanticScholar,
                enabled: true,
                auth_mode: None,
                status: Some(ArticleSourceAvailability::Unavailable),
                message: Some("Semantic Scholar enrichment unavailable".to_string()),
            });
        }
    };
    let auth_mode = client.auth_mode();
    let mut status = ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: Some(auth_mode),
        status: Some(ArticleSourceAvailability::Ok),
        message: None,
    };

    for chunk in lookup_ids.chunks(SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS) {
        if plain_article_search_deadline_elapsed(execution) {
            let deadline = article_search_deadline_status(
                ArticleSource::SemanticScholar,
                "Semantic Scholar enrichment",
            );
            status.status = deadline.status;
            status.message = deadline.message;
            break;
        }
        let unit = match first_unit.take() {
            Some(unit) => Some(unit),
            None => match execution {
                Some(execution) => {
                    execution
                        .begin_provider_unit("enrichment", "semanticscholar")
                        .await
                }
                None => None,
            },
        };
        if execution.is_some() && unit.is_none() {
            status.status = Some(ArticleSourceAvailability::Degraded);
            status.message = Some("Variant article work budget exhausted".into());
            break;
        }
        let result = client.paper_batch_search_enrichment(chunk).await;
        match result {
            Ok(papers) => {
                for (lookup_id, paper) in chunk.iter().zip(papers) {
                    let Some(paper) = paper else {
                        continue;
                    };
                    let Some(row_positions) = lookup_positions.get(lookup_id) else {
                        continue;
                    };
                    for row_idx in row_positions {
                        merge_article_search_row_with_semantic_scholar(&mut rows[*row_idx], &paper);
                    }
                }
                if let Some(unit) = unit {
                    unit.record("ok", 1);
                }
            }
            Err(err) => {
                if let Some(unit) = unit {
                    unit.record_error(&err);
                }
                crate::error::warn_external_failure(
                    &err,
                    crate::error::SourceProvider::SEMANTIC_SCHOLAR,
                    "batch article search enrichment",
                );
                if plain_article_search_deadline_elapsed(execution) {
                    let deadline = article_search_deadline_status(
                        ArticleSource::SemanticScholar,
                        "Semantic Scholar enrichment",
                    );
                    status.status = deadline.status;
                    status.message = deadline.message;
                } else {
                    status.status = Some(ArticleSourceAvailability::Unavailable);
                    status.message = Some("Semantic Scholar enrichment unavailable".to_string());
                }
                break;
            }
        }
    }
    Some(status)
}

fn article_search_row_needs_visible_article_fallback(row: &ArticleSearchResult) -> bool {
    (row.source == ArticleSource::PubMed || row.matched_sources.contains(&ArticleSource::PubMed))
        && parse_pmid(&row.pmid).is_some()
        && (row.citation_count.is_none()
            || matches!(row.citation_count, Some(0))
            || row
                .abstract_snippet
                .as_deref()
                .is_none_or(|snippet| snippet.trim().is_empty())
            || row.normalized_abstract.trim().is_empty())
}

fn merge_article_search_row_with_article_base(row: &mut ArticleSearchResult, article: &Article) {
    merge_semantic_scholar_search_citation(&mut row.citation_count, article.citation_count);
    if let Some(abstract_text) = article.abstract_text.as_deref() {
        merge_article_search_row_abstract_text(row, abstract_text);
    }
}

pub(super) async fn enrich_visible_article_search_rows_with_article_base(
    rows: &mut [ArticleSearchResult],
) -> Vec<ArticleSourceStatus> {
    enrich_visible_article_search_rows_with_article_base_context(rows, None).await
}

pub(super) async fn enrich_visible_article_search_rows_with_article_base_context(
    rows: &mut [ArticleSearchResult],
    execution: Option<&super::variant_search::VariantArticleExecutionContext>,
) -> Vec<ArticleSourceStatus> {
    let lookup_positions = rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| {
            article_search_row_needs_visible_article_fallback(row)
                .then(|| parse_pmid(&row.pmid).map(|pmid| (idx, pmid)))
                .flatten()
        })
        .collect::<Vec<_>>();
    if let Some(execution) = execution {
        execution.set_route_unit_plan("enrichment", "pubtator", Some(lookup_positions.len()));
        execution.set_route_unit_plan("enrichment", "europepmc", Some(0));
    }
    if lookup_positions.is_empty() {
        return Vec::new();
    }
    if let Some(execution) = execution {
        for (row_idx, pmid) in lookup_positions {
            let lookup_id = rows[row_idx].pmid.clone();
            match resolve_variant_article_from_pmid(pmid, &lookup_id, &lookup_id, None, execution)
                .await
            {
                Ok(article) => {
                    merge_article_search_row_with_article_base(&mut rows[row_idx], &article)
                }
                Err(err) => crate::error::warn_external_failure(
                    &err,
                    crate::error::SourceProvider::PUBTATOR3,
                    "visible article metadata fallback",
                ),
            }
        }
        return Vec::new();
    }

    let pubtator = match match crate::sources::current_variant_article_deadline() {
        Some(deadline) => PubTatorClient::new_with_deadline(&deadline).await,
        None => PubTatorClient::new(),
    } {
        Ok(client) => client,
        Err(err) => {
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::PUBTATOR3,
                "initialize visible article metadata fallback",
            );
            if plain_article_search_deadline_elapsed(execution)
                && super::search::is_search_deadline_error(&err)
            {
                return vec![article_search_deadline_status(
                    ArticleSource::PubTator,
                    "initialization",
                )];
            }
            return Vec::new();
        }
    };
    let europe = match match crate::sources::current_variant_article_deadline() {
        Some(deadline) => EuropePmcClient::new_with_deadline(&deadline).await,
        None => EuropePmcClient::new(),
    } {
        Ok(client) => client,
        Err(err) => {
            crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::EUROPE_PMC,
                "initialize visible article metadata fallback",
            );
            if plain_article_search_deadline_elapsed(execution)
                && super::search::is_search_deadline_error(&err)
            {
                return vec![article_search_deadline_status(
                    ArticleSource::EuropePmc,
                    "initialization",
                )];
            }
            return Vec::new();
        }
    };

    let mut statuses = Vec::new();
    for (row_idx, pmid) in lookup_positions {
        // The per-row PubTator/Europe PMC fallback chain is only as bounded as
        // the invocation deadline: once it elapses, name both consulted
        // sources and stop issuing further lookups.
        if plain_article_search_deadline_elapsed(execution) {
            for source in [ArticleSource::PubTator, ArticleSource::EuropePmc] {
                statuses.push(article_search_deadline_status(
                    source,
                    "article metadata fallback",
                ));
            }
            break;
        }
        let lookup_id = rows[row_idx].pmid.clone();
        let result = resolve_article_from_pmid_with_context(
            pmid, &lookup_id, &lookup_id, &pubtator, &europe, None, execution, false,
        )
        .await;
        match result {
            Ok(article) => merge_article_search_row_with_article_base(&mut rows[row_idx], &article),
            Err(err) => crate::error::warn_external_failure(
                &err,
                crate::error::SourceProvider::PUBTATOR3,
                "visible article metadata fallback",
            ),
        }
        // The final in-flight fallback may consume the deadline without another
        // loop iteration. Keep any materialized article and report the affected
        // PubTator/Europe PMC chain even when its detail helper returned success.
        if plain_article_search_deadline_elapsed(execution) {
            for source in [ArticleSource::PubTator, ArticleSource::EuropePmc] {
                statuses.push(article_search_deadline_status(
                    source,
                    "article metadata fallback",
                ));
            }
            break;
        }
    }
    statuses
}

pub(super) async fn enrich_and_finalize_article_candidates_with_semantic_scholar_status(
    mut rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
    enrichment_sources: &[ArticleSource],
) -> ArticleEnrichmentOutcome {
    let mut timings = Vec::new();
    let mut statuses = Vec::new();
    let source_status = if enrichment_sources.contains(&ArticleSource::SemanticScholar) {
        let started = std::time::Instant::now();
        let status = enrich_article_search_rows_with_semantic_scholar(&mut rows).await;
        timings.push(ArticleSearchTiming {
            source: Some(ArticleSource::SemanticScholar),
            stage: "enrichment",
            elapsed_ms: started.elapsed().as_millis() as u64,
        });
        status
    } else {
        None
    };
    let mut page = finalize_article_candidates(rows, limit, offset, total, filters);
    if enrichment_sources.contains(&ArticleSource::PubTator)
        || enrichment_sources.contains(&ArticleSource::EuropePmc)
    {
        let started = std::time::Instant::now();
        statuses
            .extend(enrich_visible_article_search_rows_with_article_base(&mut page.results).await);
        timings.push(ArticleSearchTiming {
            source: None,
            stage: "metadata_fallback",
            elapsed_ms: started.elapsed().as_millis() as u64,
        });
    }
    ArticleEnrichmentOutcome {
        page,
        semantic_scholar_status: source_status,
        statuses,
        timings,
    }
}

pub(super) async fn enrich_and_finalize_article_candidates(
    rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
    enrichment_sources: &[ArticleSource],
) -> ArticleEnrichmentOutcome {
    enrich_and_finalize_article_candidates_with_semantic_scholar_status(
        rows,
        limit,
        offset,
        total,
        filters,
        enrichment_sources,
    )
    .await
}

pub(super) async fn enrich_visible_article_search_page(
    mut page: SearchPage<ArticleSearchResult>,
    enrichment_sources: &[ArticleSource],
) -> ArticleEnrichmentOutcome {
    let mut timings = Vec::new();
    let mut statuses = Vec::new();
    let mut semantic_scholar_status = None;
    if enrichment_sources.contains(&ArticleSource::SemanticScholar) {
        let started = std::time::Instant::now();
        semantic_scholar_status =
            enrich_article_search_rows_with_semantic_scholar(&mut page.results).await;
        timings.push(ArticleSearchTiming {
            source: Some(ArticleSource::SemanticScholar),
            stage: "enrichment",
            elapsed_ms: started.elapsed().as_millis() as u64,
        });
    }
    if enrichment_sources.contains(&ArticleSource::PubTator)
        || enrichment_sources.contains(&ArticleSource::EuropePmc)
    {
        let started = std::time::Instant::now();
        statuses
            .extend(enrich_visible_article_search_rows_with_article_base(&mut page.results).await);
        timings.push(ArticleSearchTiming {
            source: None,
            stage: "metadata_fallback",
            elapsed_ms: started.elapsed().as_millis() as u64,
        });
    }
    ArticleEnrichmentOutcome {
        page,
        semantic_scholar_status,
        statuses,
        timings,
    }
}

#[cfg(test)]
mod tests;
