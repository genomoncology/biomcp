//! Article search backend fetchers for PubMed-family and semantic sources.

use std::collections::{HashMap, HashSet};

use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::litsense2::LitSense2Client;
use crate::sources::pubmed::{PubMedClient, PubMedESearchParams};
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::SemanticScholarClient;
use crate::transform;

use super::filters::{matches_result_filters, normalized_date_bounds};
use super::query::{
    build_free_text_article_query, build_pubmed_search_term, build_pubtator_query,
    build_search_query, pubtator_sort,
};
use super::{
    ArticleSearchFilters, ArticleSearchResult, ArticleSort, ArticleSource,
    ArticleSourceAvailability, ArticleSourceStatus, EUROPE_PMC_PAGE_SIZE,
    MAX_FEDERATED_FETCH_RESULTS, MAX_PAGE_FETCHES, PUBMED_PAGE_SIZE, PUBTATOR_PAGE_SIZE,
    WARN_PAGE_THRESHOLD,
};

fn normalized_bound_year(value: &str) -> Option<&str> {
    let year = value.get(..4)?;
    year.chars().all(|ch| ch.is_ascii_digit()).then_some(year)
}

pub(super) fn semantic_scholar_year_filter(
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Option<String> {
    let from_year = date_from.and_then(normalized_bound_year);
    let to_year = date_to.and_then(normalized_bound_year);
    match (from_year, to_year) {
        (Some(from), Some(to)) => Some(format!("{from}-{to}")),
        (Some(from), None) => Some(format!("{from}-")),
        (None, Some(to)) => Some(format!("-{to}")),
        (None, None) => None,
    }
}

pub(super) struct SemanticScholarCandidateOutcome {
    pub rows: Vec<ArticleSearchResult>,
    pub status: ArticleSourceStatus,
}

pub(super) async fn search_pubmed_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    if limit == 0 || limit > MAX_FEDERATED_FETCH_RESULTS {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_FEDERATED_FETCH_RESULTS}"
        )));
    }
    if filters.open_access {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --open-access filtering".into(),
        ));
    }
    if filters.no_preprints {
        return Err(BioMcpError::InvalidArgument(
            "PubMed ESearch does not support --no-preprints filtering".into(),
        ));
    }

    let term = build_pubmed_search_term(filters)?;
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let client = PubMedClient::new()?;

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut batch_start = 0usize;
    let mut visible_skipped = 0usize;
    let mut source_position = 0usize;
    let mut fetched_pages = 0usize;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages == WARN_PAGE_THRESHOLD + 1 {
            tracing::warn!(
                "article search is deep (>{WARN_PAGE_THRESHOLD} page fetches); continuing up to {MAX_PAGE_FETCHES} — consider narrowing your query"
            );
        }

        let response = client
            .esearch(&PubMedESearchParams {
                term: term.clone(),
                retstart: batch_start,
                retmax: PUBMED_PAGE_SIZE,
                date_from: normalized_date_from.clone(),
                date_to: normalized_date_to.clone(),
            })
            .await?;
        if total.is_none() {
            total = Some(response.count as usize);
            if total.is_some_and(|value| offset >= value) {
                return Ok(SearchPage::offset(Vec::new(), total));
            }
        }
        if response.idlist.is_empty() {
            break;
        }

        let batch_len = response.idlist.len();
        let entries = client.esummary(&response.idlist).await?;
        for entry in entries {
            let mut row =
                transform::article::from_pubmed_esummary_entry(&entry).ok_or_else(|| {
                    BioMcpError::Api {
                        api: "pubmed-eutils".to_string(),
                        message: format!(
                            "ESummary entry for PMID {} has blank title after cleaning",
                            entry.uid
                        ),
                    }
                })?;
            if !matches_result_filters(
                &row,
                filters,
                normalized_date_from.as_deref(),
                normalized_date_to.as_deref(),
            ) {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            row.source_local_position = source_position;
            source_position = source_position.saturating_add(1);
            if visible_skipped < offset {
                visible_skipped = visible_skipped.saturating_add(1);
                continue;
            }
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        batch_start = batch_start.saturating_add(batch_len);
    }

    Ok(SearchPage::offset(out, total))
}

pub(super) async fn search_europepmc_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let europe = EuropePmcClient::new()?;
    let query = build_search_query(filters)?;
    let europepmc_sort = filters.sort.as_europepmc_sort();
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut page: usize = (offset / EUROPE_PMC_PAGE_SIZE) + 1;
    let mut local_skip = offset % EUROPE_PMC_PAGE_SIZE;
    let mut source_position = 0usize;
    let mut fetched_pages = 0usize;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        if fetched_pages == WARN_PAGE_THRESHOLD + 1 {
            tracing::warn!(
                "article search is deep (>{WARN_PAGE_THRESHOLD} page fetches); continuing up to {MAX_PAGE_FETCHES} — consider narrowing your query"
            );
        }
        let resp = europe
            .search_query_with_sort(&query, page, EUROPE_PMC_PAGE_SIZE, europepmc_sort)
            .await?;
        if total.is_none() {
            total = resp.hit_count.map(|v| v as usize);
            if total.is_some_and(|value| offset >= value) {
                return Ok(SearchPage::offset(Vec::new(), total));
            }
        }
        let Some(results) = resp.result_list.map(|v| v.result) else {
            break;
        };
        if results.is_empty() {
            break;
        }

        for hit in results {
            if local_skip > 0 {
                local_skip -= 1;
                continue;
            }

            let Some(mut row) = transform::article::from_europepmc_search_result(&hit) else {
                continue;
            };
            if !matches_result_filters(
                &row,
                filters,
                normalized_date_from.as_deref(),
                normalized_date_to.as_deref(),
            ) {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            row.source_local_position = source_position;
            source_position = source_position.saturating_add(1);
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        page += 1;
    }

    // Safety-first default: when date-sorted results contain no visible retraction marker,
    // try adding one matched retracted publication if available.
    if !filters.exclude_retracted
        && filters.sort == ArticleSort::Date
        && !out.iter().any(|row| row.is_retracted == Some(true))
    {
        let retracted_query = format!("({query}) AND PUB_TYPE:\"retracted publication\"");
        if let Ok(resp) = europe
            .search_query_with_sort(&retracted_query, 1, 10, europepmc_sort)
            .await
        {
            let replacement = resp
                .result_list
                .map(|v| v.result)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|hit| transform::article::from_europepmc_search_result(&hit))
                .find(|row| {
                    row.is_retracted == Some(true)
                        && !seen_pmids.contains(&row.pmid)
                        && matches_result_filters(
                            row,
                            filters,
                            normalized_date_from.as_deref(),
                            normalized_date_to.as_deref(),
                        )
                });
            if let Some(mut row) = replacement {
                if out.len() >= limit && !out.is_empty() {
                    out.pop();
                }
                if out.len() < limit {
                    row.source_local_position = out.len();
                    seen_pmids.insert(row.pmid.clone());
                    out.push(row);
                }
            }
        }
    }

    Ok(SearchPage::offset(out, total))
}

pub(super) async fn search_pubtator_page(
    filters: &ArticleSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<ArticleSearchResult>, BioMcpError> {
    let pubtator = PubTatorClient::new()?;
    let query = build_pubtator_query(filters, &pubtator).await?;
    let sort = pubtator_sort(filters.sort);
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;

    let mut out: Vec<ArticleSearchResult> = Vec::with_capacity(limit.min(10));
    let mut seen_pmids: HashSet<String> = HashSet::with_capacity(limit.min(10));
    let mut total: Option<usize> = None;
    let mut page: usize = (offset / PUBTATOR_PAGE_SIZE) + 1;
    let mut local_skip = offset % PUBTATOR_PAGE_SIZE;
    let mut source_position = 0usize;
    let mut fetched_pages = 0usize;
    while out.len() < limit && fetched_pages < MAX_PAGE_FETCHES {
        fetched_pages = fetched_pages.saturating_add(1);
        let resp = pubtator
            .search(&query, page, PUBTATOR_PAGE_SIZE, sort)
            .await?;
        if total.is_none() {
            total = resp.count.map(|v| v as usize);
            if total.is_some_and(|value| offset >= value) {
                return Ok(SearchPage::offset(Vec::new(), total));
            }
        }

        if resp.results.is_empty() {
            break;
        }

        for hit in resp.results {
            if local_skip > 0 {
                local_skip -= 1;
                continue;
            }
            let Some(mut row) = transform::article::from_pubtator_search_result(&hit) else {
                continue;
            };
            if !matches_result_filters(
                &row,
                filters,
                normalized_date_from.as_deref(),
                normalized_date_to.as_deref(),
            ) {
                continue;
            }
            if !seen_pmids.insert(row.pmid.clone()) {
                continue;
            }
            row.source_local_position = source_position;
            source_position = source_position.saturating_add(1);
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }
        page += 1;
    }

    Ok(SearchPage::offset(out, total))
}

pub(super) async fn search_semantic_scholar_candidates(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<SemanticScholarCandidateOutcome, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    let auth_mode = client.auth_mode();
    let mut status = ArticleSourceStatus {
        source: ArticleSource::SemanticScholar,
        enabled: true,
        auth_mode: Some(auth_mode),
        status: Some(ArticleSourceAvailability::Ok),
        message: None,
    };

    let query = build_free_text_article_query(filters);
    if query.trim().is_empty() {
        return Ok(SemanticScholarCandidateOutcome {
            rows: Vec::new(),
            status,
        });
    }
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let year_filter = semantic_scholar_year_filter(
        normalized_date_from.as_deref(),
        normalized_date_to.as_deref(),
    );

    let response = match client
        .paper_search(&query, limit, year_filter.as_deref())
        .await
    {
        Ok(response) => response,
        Err(err) => {
            warn!(?err, query, "Semantic Scholar article search leg failed");
            status.status = Some(ArticleSourceAvailability::Unavailable);
            status.message = Some("Semantic Scholar search unavailable".to_string());
            return Ok(SemanticScholarCandidateOutcome {
                rows: Vec::new(),
                status,
            });
        }
    };

    let mut rows = Vec::with_capacity(response.data.len());
    let mut source_position = 0usize;
    for paper in response.data {
        let external_ids = paper.external_ids.as_ref();
        let title = paper
            .title
            .as_deref()
            .map(transform::article::clean_title)
            .unwrap_or_default();
        let abstract_text = paper
            .abstract_text
            .as_deref()
            .map(transform::article::clean_abstract);
        let mut row = ArticleSearchResult {
            pmid: external_ids
                .and_then(|ids| ids.pubmed.clone())
                .unwrap_or_default()
                .trim()
                .to_string(),
            pmcid: external_ids
                .and_then(|ids| ids.pmcid.clone())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            doi: external_ids
                .and_then(|ids| ids.doi.clone())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            title,
            journal: paper
                .venue
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            date: paper.year.map(|year| year.to_string()),
            first_index_date: None,
            citation_count: paper.citation_count,
            influential_citation_count: paper.influential_citation_count,
            source: ArticleSource::SemanticScholar,
            matched_sources: vec![ArticleSource::SemanticScholar],
            score: None,
            is_retracted: None,
            abstract_snippet: abstract_text
                .as_deref()
                .and_then(transform::article::article_search_abstract_snippet),
            ranking: None,
            normalized_title: paper
                .title
                .as_deref()
                .map(transform::article::normalize_article_search_text)
                .unwrap_or_default(),
            normalized_abstract: abstract_text
                .as_deref()
                .map(transform::article::normalize_article_search_text)
                .unwrap_or_default(),
            publication_type: None,
            source_local_position: 0,
        };
        if matches_result_filters(
            &row,
            filters,
            normalized_date_from.as_deref(),
            normalized_date_to.as_deref(),
        ) {
            row.source_local_position = source_position;
            source_position = source_position.saturating_add(1);
            rows.push(row);
        }
    }

    Ok(SemanticScholarCandidateOutcome { rows, status })
}

pub(super) async fn search_litsense2_candidates(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<Vec<ArticleSearchResult>, BioMcpError> {
    let query = build_free_text_article_query(filters);
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let (normalized_date_from, normalized_date_to) = normalized_date_bounds(filters)?;
    let response = LitSense2Client::new()?.sentence_search(&query).await?;

    let mut deduped: HashMap<u64, (crate::sources::litsense2::LitSense2SearchHit, usize)> =
        HashMap::new();
    for (index, hit) in response.into_iter().enumerate() {
        match deduped.get_mut(&hit.pmid) {
            Some((best, _)) if hit.score > best.score => *best = hit,
            Some(_) => {}
            None => {
                deduped.insert(hit.pmid, (hit, index));
            }
        }
    }

    let mut deduped = deduped.into_values().collect::<Vec<_>>();
    deduped.sort_by(
        |(left_hit, left_first_seen), (right_hit, right_first_seen)| {
            right_hit
                .score
                .total_cmp(&left_hit.score)
                .then_with(|| left_first_seen.cmp(right_first_seen))
        },
    );

    let pmids = deduped
        .iter()
        .map(|(hit, _)| hit.pmid.to_string())
        .collect::<Vec<_>>();
    let mut hydrated = PubMedClient::new()?
        .esummary(&pmids)
        .await?
        .into_iter()
        .filter_map(|entry| {
            transform::article::from_pubmed_esummary_entry(&entry)
                .map(|row| (row.pmid.clone(), row))
        })
        .collect::<HashMap<_, _>>();

    let mut rows = Vec::with_capacity(deduped.len());
    let mut source_position = 0usize;
    for (hit, _) in deduped {
        let pmid = hit.pmid.to_string();
        let cleaned_text = transform::article::clean_abstract(&hit.text);
        let fallback_title = if cleaned_text.is_empty() {
            format!("PMID {pmid}")
        } else {
            transform::article::article_search_fallback_title(&cleaned_text)
        };
        let mut row = hydrated
            .remove(&pmid)
            .unwrap_or_else(|| ArticleSearchResult {
                pmid: pmid.clone(),
                pmcid: hit.pmcid.clone(),
                doi: None,
                title: fallback_title.clone(),
                journal: None,
                date: None,
                first_index_date: None,
                citation_count: None,
                influential_citation_count: None,
                source: ArticleSource::LitSense2,
                matched_sources: vec![ArticleSource::LitSense2],
                score: Some(hit.score),
                is_retracted: None,
                abstract_snippet: transform::article::article_search_abstract_snippet(
                    &cleaned_text,
                ),
                ranking: None,
                normalized_title: transform::article::normalize_article_search_text(
                    &fallback_title,
                ),
                normalized_abstract: transform::article::normalize_article_search_text(
                    &cleaned_text,
                ),
                publication_type: None,
                source_local_position: 0,
            });
        row.source = ArticleSource::LitSense2;
        row.matched_sources = vec![ArticleSource::LitSense2];
        row.score = Some(hit.score);
        if row.pmcid.is_none() {
            row.pmcid = hit.pmcid.clone();
        }
        if row.title.trim().is_empty() {
            row.title = fallback_title.clone();
        }
        row.abstract_snippet = transform::article::article_search_abstract_snippet(&cleaned_text);
        row.normalized_title = transform::article::normalize_article_search_text(&row.title);
        row.normalized_abstract = transform::article::normalize_article_search_text(&cleaned_text);
        row.is_retracted = None;
        row.publication_type = None;
        if !matches_result_filters(
            &row,
            filters,
            normalized_date_from.as_deref(),
            normalized_date_to.as_deref(),
        ) {
            continue;
        }
        row.source_local_position = source_position;
        source_position = source_position.saturating_add(1);
        rows.push(row);
        if rows.len() >= limit {
            break;
        }
    }

    Ok(rows)
}

#[cfg(test)]
mod tests;
