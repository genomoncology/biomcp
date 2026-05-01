//! Article candidate identity, merge, source-cap, and finalize helpers.

use crate::entities::SearchPage;
use crate::error::BioMcpError;

use super::ranking::sort_article_rows;
use super::{ArticleSearchFilters, ArticleSearchResult, ArticleSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ArticleSourcePosition {
    pub(super) source: ArticleSource,
    pub(super) local_position: usize,
}

#[derive(Debug, Clone)]
pub(super) struct ArticleCandidate {
    pub(super) row: ArticleSearchResult,
    pub(super) source_positions: Vec<ArticleSourcePosition>,
    pub(super) semantic_signal: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArticleSourceCapMode {
    Default(usize),
    Explicit(usize),
    Disabled,
}

fn normalize_row_identifier(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

pub(super) fn article_source_priority(source: ArticleSource) -> u8 {
    match source {
        ArticleSource::PubTator => 0,
        ArticleSource::EuropePmc => 1,
        ArticleSource::PubMed => 2,
        ArticleSource::SemanticScholar => 3,
        ArticleSource::LitSense2 => 4,
    }
}

pub(super) fn stable_article_identifier(row: &ArticleSearchResult) -> String {
    normalize_row_identifier(Some(&row.pmid))
        .or_else(|| normalize_row_identifier(row.pmcid.as_deref()))
        .or_else(|| normalize_row_identifier(row.doi.as_deref()))
        .unwrap_or_else(|| row.title.to_ascii_lowercase())
}

pub(super) fn validate_article_source_cap(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> Result<(), BioMcpError> {
    if let Some(max_per_source) = filters.max_per_source
        && max_per_source > limit
    {
        return Err(BioMcpError::InvalidArgument(
            "--max-per-source must be <= --limit".into(),
        ));
    }
    Ok(())
}

fn resolve_article_source_cap(
    filters: &ArticleSearchFilters,
    limit: usize,
) -> ArticleSourceCapMode {
    match filters.max_per_source {
        None | Some(0) => ArticleSourceCapMode::Default((limit.saturating_mul(40) / 100).max(1)),
        Some(value) if value == limit => ArticleSourceCapMode::Disabled,
        Some(value) => ArticleSourceCapMode::Explicit(value),
    }
}

pub(super) fn ensure_matched_sources(row: &mut ArticleSearchResult) {
    if !row.matched_sources.contains(&row.source) {
        row.matched_sources.push(row.source);
    }
    row.matched_sources
        .sort_by_key(|source| article_source_priority(*source));
    row.matched_sources.dedup();
}

pub(super) fn article_candidate_from_row(mut row: ArticleSearchResult) -> ArticleCandidate {
    ensure_matched_sources(&mut row);
    ArticleCandidate {
        source_positions: vec![ArticleSourcePosition {
            source: row.source,
            local_position: row.source_local_position,
        }],
        semantic_signal: (row.source == ArticleSource::LitSense2)
            .then(|| row.score.unwrap_or(0.0).clamp(0.0, 1.0)),
        row,
    }
}

fn collapse_source_positions(source_positions: &mut Vec<ArticleSourcePosition>) {
    source_positions
        .sort_by_key(|entry| (article_source_priority(entry.source), entry.local_position));
    source_positions.dedup_by_key(|entry| entry.source);
}

fn min_source_local_position(source_positions: &[ArticleSourcePosition]) -> Option<usize> {
    source_positions
        .iter()
        .map(|entry| entry.local_position)
        .min()
}

fn article_rows_overlap(left: &ArticleSearchResult, right: &ArticleSearchResult) -> bool {
    let left_pmid = normalize_row_identifier(Some(&left.pmid));
    let right_pmid = normalize_row_identifier(Some(&right.pmid));
    let left_pmcid = normalize_row_identifier(left.pmcid.as_deref());
    let right_pmcid = normalize_row_identifier(right.pmcid.as_deref());
    let left_doi = normalize_row_identifier(left.doi.as_deref());
    let right_doi = normalize_row_identifier(right.doi.as_deref());

    left_pmid.is_some() && left_pmid == right_pmid
        || left_pmcid.is_some() && left_pmcid == right_pmcid
        || left_doi.is_some() && left_doi == right_doi
}

fn merge_missing_string(target: &mut Option<String>, incoming: Option<String>) {
    if target
        .as_deref()
        .map(str::trim)
        .is_none_or(|value| value.is_empty())
    {
        *target = incoming
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
    }
}

fn merge_missing_u64(target: &mut Option<u64>, incoming: Option<u64>) {
    if target.is_none() {
        *target = incoming;
    }
}

fn merge_article_candidate(target: &mut ArticleCandidate, incoming: ArticleCandidate) {
    let ArticleCandidate {
        row: incoming_row,
        mut source_positions,
        semantic_signal,
    } = incoming;
    let target_row = &mut target.row;
    merge_missing_string(&mut target_row.pmcid, incoming_row.pmcid);
    merge_missing_string(&mut target_row.doi, incoming_row.doi);
    if target_row.pmid.trim().is_empty() && !incoming_row.pmid.trim().is_empty() {
        target_row.pmid = incoming_row.pmid;
    }
    if target_row.title.trim().is_empty() && !incoming_row.title.trim().is_empty() {
        target_row.title = incoming_row.title;
    }
    merge_missing_string(&mut target_row.journal, incoming_row.journal);
    merge_missing_string(&mut target_row.date, incoming_row.date);
    merge_missing_u64(&mut target_row.citation_count, incoming_row.citation_count);
    merge_missing_u64(
        &mut target_row.influential_citation_count,
        incoming_row.influential_citation_count,
    );
    if target_row.score.is_none() {
        target_row.score = incoming_row.score;
    }
    if target_row.is_retracted.is_none() && incoming_row.is_retracted.is_some() {
        target_row.is_retracted = incoming_row.is_retracted;
    }
    merge_missing_string(
        &mut target_row.abstract_snippet,
        incoming_row.abstract_snippet,
    );
    if target_row.ranking.is_none() {
        target_row.ranking = incoming_row.ranking;
    }
    if target_row.normalized_title.is_empty() && !incoming_row.normalized_title.is_empty() {
        target_row.normalized_title = incoming_row.normalized_title;
    }
    if target_row.normalized_abstract.is_empty() && !incoming_row.normalized_abstract.is_empty() {
        target_row.normalized_abstract = incoming_row.normalized_abstract;
    }
    merge_missing_string(
        &mut target_row.publication_type,
        incoming_row.publication_type,
    );
    target_row
        .matched_sources
        .extend(incoming_row.matched_sources);
    ensure_matched_sources(target_row);
    target.source_positions.append(&mut source_positions);
    collapse_source_positions(&mut target.source_positions);
    if target.semantic_signal.is_none() {
        target.semantic_signal = semantic_signal;
    }
    if let Some(local_position) = min_source_local_position(&target.source_positions) {
        target_row.source_local_position = local_position;
    }
}

pub(super) fn merge_article_candidates(results: Vec<ArticleSearchResult>) -> Vec<ArticleCandidate> {
    let mut merged: Vec<ArticleCandidate> = Vec::with_capacity(results.len());

    for row in results {
        let row = article_candidate_from_row(row);
        let matches = merged
            .iter()
            .enumerate()
            .filter_map(|(idx, existing)| {
                article_rows_overlap(&existing.row, &row.row).then_some(idx)
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            merged.push(row);
            continue;
        }

        let keep_idx = matches[0];
        merge_article_candidate(&mut merged[keep_idx], row);
        for idx in matches.into_iter().skip(1).rev() {
            let duplicate = merged.remove(idx);
            merge_article_candidate(&mut merged[keep_idx], duplicate);
        }
    }

    merged
}

fn primary_source_native_position(candidate: &ArticleCandidate) -> usize {
    candidate
        .source_positions
        .iter()
        .find(|entry| entry.source == candidate.row.source)
        .map(|entry| entry.local_position)
        .unwrap_or(candidate.row.source_local_position)
}

fn distinct_primary_source_count(candidates: &[ArticleCandidate]) -> usize {
    let mut seen = Vec::new();
    for candidate in candidates {
        if !seen.contains(&candidate.row.source) {
            seen.push(candidate.row.source);
        }
    }
    seen.len()
}

fn cap_article_candidates_by_source(
    candidates: Vec<ArticleCandidate>,
    cap_mode: ArticleSourceCapMode,
) -> Vec<ArticleCandidate> {
    let source_count = distinct_primary_source_count(&candidates);
    let effective_cap = match cap_mode {
        ArticleSourceCapMode::Disabled => return candidates,
        ArticleSourceCapMode::Default(_) if source_count < 3 => return candidates,
        ArticleSourceCapMode::Default(_cap) | ArticleSourceCapMode::Explicit(_cap)
            if source_count < 2 =>
        {
            return candidates;
        }
        ArticleSourceCapMode::Default(cap) | ArticleSourceCapMode::Explicit(cap) => cap,
    };

    let mut buckets: Vec<(ArticleSource, Vec<ArticleCandidate>)> = Vec::new();
    for candidate in candidates {
        if let Some((_, rows)) = buckets
            .iter_mut()
            .find(|(source, _)| *source == candidate.row.source)
        {
            rows.push(candidate);
        } else {
            buckets.push((candidate.row.source, vec![candidate]));
        }
    }

    let mut retained = Vec::new();
    for (_, mut bucket) in buckets {
        bucket.sort_by(|left, right| {
            primary_source_native_position(left)
                .cmp(&primary_source_native_position(right))
                .then_with(|| {
                    stable_article_identifier(&left.row).cmp(&stable_article_identifier(&right.row))
                })
        });
        retained.extend(bucket.into_iter().take(effective_cap));
    }
    retained
}

pub(super) fn finalize_article_candidates(
    mut rows: Vec<ArticleSearchResult>,
    limit: usize,
    offset: usize,
    total: Option<usize>,
    filters: &ArticleSearchFilters,
) -> SearchPage<ArticleSearchResult> {
    for row in rows.iter_mut() {
        ensure_matched_sources(row);
    }

    let mut rows = merge_article_candidates(rows);
    rows.retain(|candidate| !candidate.row.pmid.trim().is_empty());
    rows = cap_article_candidates_by_source(rows, resolve_article_source_cap(filters, limit));
    sort_article_rows(&mut rows, filters.sort, filters);
    let mut rows = rows
        .into_iter()
        .map(|candidate| candidate.row)
        .collect::<Vec<_>>();
    rows.drain(0..offset.min(rows.len()));
    rows.truncate(limit);
    SearchPage::offset(rows, total)
}

#[cfg(test)]
mod tests;
