//! Article search-result enrichment and visible-row fallback helpers.

use std::collections::HashMap;

use tracing::warn;

use crate::entities::SearchPage;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{SemanticScholarClient, SemanticScholarPaper};

use super::candidates::finalize_article_candidates;
use super::detail::{parse_pmid, resolve_article_from_pmid};
use super::{
    Article, ArticleSearchFilters, ArticleSearchResult, ArticleSource,
    ArticleSourceAvailability, ArticleSourceStatus, SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS,
};

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

    if lookup_ids.is_empty() {
        return None;
    }

    let client = match SemanticScholarClient::new() {
        Ok(client) => client,
        Err(err) => {
            warn!(?err, "Semantic Scholar search-row enrichment unavailable");
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

    for (chunk_idx, chunk) in lookup_ids
        .chunks(SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS)
        .enumerate()
    {
        let chunk_start = chunk_idx * SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS;
        let chunk_end = chunk_start + chunk.len();
        match client.paper_batch_search_enrichment(chunk).await {
            Ok(papers) => {
                for (lookup_id, paper) in chunk.iter().zip(papers.into_iter()) {
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
            }
            Err(err) => {
                warn!(
                    ?err,
                    chunk_start,
                    chunk_end,
                    "Semantic Scholar article-search batch enrichment failed",
                );
                status.status = Some(ArticleSourceAvailability::Unavailable);
                status.message = Some("Semantic Scholar enrichment unavailable".to_string());
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

async fn enrich_visible_article_search_rows_with_article_base(rows: &mut [ArticleSearchResult]) {
    let lookup_positions = rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| {
            article_search_row_needs_visible_article_fallback(row)
                .then(|| parse_pmid(&row.pmid).map(|pmid| (idx, pmid)))
                .flatten()
        })
        .collect::<Vec<_>>();
    if lookup_positions.is_empty() {
        return;
    }

    let pubtator = match PubTatorClient::new() {
        Ok(client) => client,
        Err(err) => {
            warn!(?err, "PubTator visible-row metadata fallback unavailable");
            return;
        }
    };
    let europe = match EuropePmcClient::new() {
        Ok(client) => client,
        Err(err) => {
            warn!(?err, "Europe PMC visible-row metadata fallback unavailable");
            return;
        }
    };

    for (row_idx, pmid) in lookup_positions {
        let lookup_id = rows[row_idx].pmid.clone();
        match resolve_article_from_pmid(pmid, &lookup_id, &lookup_id, &pubtator, &europe, None)
            .await
        {
            Ok(article) => merge_article_search_row_with_article_base(&mut rows[row_idx], &article),
            Err(err) => warn!(
                ?err,
                pmid = lookup_id,
                "Visible article-search metadata fallback failed",
            ),
        }
    }
}

pub(super) async fn enrich_and_finalize_article_candidates_with_semantic_scholar_status(
    mut rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
) -> (SearchPage<ArticleSearchResult>, Option<ArticleSourceStatus>) {
    let source_status = enrich_article_search_rows_with_semantic_scholar(&mut rows).await;
    let mut page = finalize_article_candidates(rows, limit, offset, total, filters);
    enrich_visible_article_search_rows_with_article_base(&mut page.results).await;
    (page, source_status)
}

pub(super) async fn enrich_and_finalize_article_candidates(
    rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
) -> SearchPage<ArticleSearchResult> {
    enrich_and_finalize_article_candidates_with_semantic_scholar_status(
        rows, limit, offset, total, filters,
    )
    .await
    .0
}

pub(super) async fn enrich_visible_article_search_page(
    mut page: SearchPage<ArticleSearchResult>,
) -> SearchPage<ArticleSearchResult> {
    let _ = enrich_article_search_rows_with_semantic_scholar(&mut page.results).await;
    enrich_visible_article_search_rows_with_article_base(&mut page.results).await;
    page
}

#[cfg(test)]
mod tests;
