//! Article ranking policy, lexical metadata, and sort-order helpers.

use std::cmp::Ordering;
use std::collections::HashSet;

use crate::error::BioMcpError;
use crate::transform;

use super::candidates::{
    ArticleCandidate, ArticleSourcePosition, ensure_matched_sources, stable_article_identifier,
};
use super::filters::{has_keyword_query, parse_row_date};
use super::{
    ARTICLE_RELEVANCE_RANKING_POLICY, ARTICLE_SEMANTIC_RANKING_POLICY, ArticlePubMedRescueKind,
    ArticleRankingMetadata, ArticleRankingMode, ArticleRankingWeights, ArticleSearchFilters,
    ArticleSearchResult, ArticleSort, ArticleSource,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct ResolvedArticleRanking {
    pub(super) mode: ArticleRankingMode,
    weights: ArticleRankingWeights,
}

pub(super) fn resolve_article_ranking(filters: &ArticleSearchFilters) -> ResolvedArticleRanking {
    ResolvedArticleRanking {
        mode: filters.ranking.requested_mode.unwrap_or_else(|| {
            if has_keyword_query(filters) {
                ArticleRankingMode::Hybrid
            } else {
                ArticleRankingMode::Lexical
            }
        }),
        weights: filters.ranking.weights,
    }
}

pub(crate) fn article_effective_ranking_mode(
    filters: &ArticleSearchFilters,
) -> Option<ArticleRankingMode> {
    (filters.sort == ArticleSort::Relevance).then(|| resolve_article_ranking(filters).mode)
}

fn format_article_ranking_number(value: f64) -> String {
    let mut out = format!("{value:.3}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" { "0".to_string() } else { out }
}

pub(crate) fn article_relevance_ranking_policy(filters: &ArticleSearchFilters) -> Option<String> {
    if filters.sort != ArticleSort::Relevance {
        return None;
    }

    let ranking = resolve_article_ranking(filters);
    Some(match ranking.mode {
        ArticleRankingMode::Lexical => ARTICLE_RELEVANCE_RANKING_POLICY.to_string(),
        ArticleRankingMode::Semantic => ARTICLE_SEMANTIC_RANKING_POLICY.to_string(),
        ArticleRankingMode::Hybrid => format!(
            "hybrid relevance (score = {}*semantic + {}*lexical + {}*citations + {}*position)",
            format_article_ranking_number(ranking.weights.semantic),
            format_article_ranking_number(ranking.weights.lexical),
            format_article_ranking_number(ranking.weights.citations),
            format_article_ranking_number(ranking.weights.position),
        ),
    })
}

pub(super) fn validate_article_ranking_options(
    filters: &ArticleSearchFilters,
) -> Result<(), BioMcpError> {
    let requested_mode = filters.ranking.requested_mode;
    let weights_overridden = filters.ranking.weights_overridden;
    if filters.sort != ArticleSort::Relevance {
        if requested_mode.is_some() || weights_overridden {
            return Err(BioMcpError::InvalidArgument(
                "--ranking-mode and --weight-* require --sort relevance".into(),
            ));
        }
        return Ok(());
    }

    for (flag, value) in [
        ("--weight-semantic", filters.ranking.weights.semantic),
        ("--weight-lexical", filters.ranking.weights.lexical),
        ("--weight-citations", filters.ranking.weights.citations),
        ("--weight-position", filters.ranking.weights.position),
    ] {
        if !value.is_finite() {
            return Err(BioMcpError::InvalidArgument(format!(
                "{flag} must be finite"
            )));
        }
        if value < 0.0 {
            return Err(BioMcpError::InvalidArgument(format!("{flag} must be >= 0")));
        }
    }

    let resolved = resolve_article_ranking(filters);
    if weights_overridden && resolved.mode != ArticleRankingMode::Hybrid {
        return Err(BioMcpError::InvalidArgument(
            "--weight-* flags require --ranking-mode hybrid or no explicit ranking mode".into(),
        ));
    }

    if resolved.mode == ArticleRankingMode::Hybrid
        && filters.ranking.weights.semantic == 0.0
        && filters.ranking.weights.lexical == 0.0
        && filters.ranking.weights.citations == 0.0
        && filters.ranking.weights.position == 0.0
    {
        return Err(BioMcpError::InvalidArgument(
            "At least one hybrid ranking weight must be > 0".into(),
        ));
    }

    Ok(())
}

fn compare_optional_dates_desc(
    left: Option<&ArticleSearchResult>,
    right: Option<&ArticleSearchResult>,
) -> Ordering {
    match (
        left.and_then(|row| parse_row_date(row.date.as_deref())),
        right.and_then(|row| parse_row_date(row.date.as_deref())),
    ) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn compare_optional_citations_desc(
    left: Option<&ArticleSearchResult>,
    right: Option<&ArticleSearchResult>,
) -> Ordering {
    match (
        left.and_then(|row| row.citation_count),
        right.and_then(|row| row.citation_count),
    ) {
        (Some(left), Some(right)) => right.cmp(&left),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(super) fn build_anchor_set(filters: &ArticleSearchFilters) -> Vec<String> {
    let mut anchors = Vec::new();
    let mut seen = HashSet::new();
    for value in [
        filters.gene.as_deref(),
        filters.disease.as_deref(),
        filters.drug.as_deref(),
    ] {
        let Some(anchor) = value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(transform::article::normalize_article_search_text)
        else {
            continue;
        };
        if seen.insert(anchor.clone()) {
            anchors.push(anchor);
        }
    }

    if let Some(keyword) = filters.keyword.as_deref() {
        for token in keyword.split_whitespace() {
            let anchor = transform::article::normalize_article_search_text(token);
            if !anchor.is_empty() && seen.insert(anchor.clone()) {
                anchors.push(anchor);
            }
        }
    }

    anchors
}

fn anchor_matches_text(text: &str, anchor: &str) -> bool {
    if anchor.is_empty() || text.is_empty() {
        return false;
    }
    if anchor.chars().any(|ch| ch.is_whitespace()) {
        return text.contains(anchor);
    }

    for (idx, _) in text.match_indices(anchor) {
        let start_ok = text[..idx]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric());
        let end_idx = idx + anchor.len();
        let end_ok = text[end_idx..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric());
        if start_ok && end_ok {
            return true;
        }
    }
    false
}

fn has_study_or_review_cue(row: &ArticleSearchResult) -> bool {
    let title = row.normalized_title.as_str();
    let publication_type = row
        .publication_type
        .as_deref()
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    [
        "review",
        "meta-analysis",
        "meta analysis",
        "systematic review",
        "clinical trial",
    ]
    .into_iter()
    .any(|cue| title.contains(cue) || publication_type.contains(cue))
}

fn pubmed_rescue_metadata(
    row: &ArticleSearchResult,
    source_positions: &[ArticleSourcePosition],
    directness_tier: u8,
    combined_hits: usize,
) -> (bool, Option<ArticlePubMedRescueKind>, Option<usize>) {
    let Some(pubmed_position) = source_positions
        .iter()
        .find(|entry| entry.source == ArticleSource::PubMed)
        .map(|entry| entry.local_position)
    else {
        return (false, None, None);
    };

    if pubmed_position > super::PUBMED_RESCUE_POSITION_MAX
        || directness_tier > 1
        || combined_hits == 0
    {
        return (false, None, None);
    }

    let pubmed_unique = matches!(row.matched_sources.as_slice(), [ArticleSource::PubMed]);
    let has_non_pubmed_source = source_positions
        .iter()
        .any(|entry| entry.source != ArticleSource::PubMed);
    let pubmed_led = row.matched_sources.contains(&ArticleSource::PubMed)
        && has_non_pubmed_source
        && source_positions
            .iter()
            .filter(|entry| entry.source != ArticleSource::PubMed)
            .all(|entry| pubmed_position < entry.local_position);

    let pubmed_rescue_kind = if pubmed_unique {
        Some(ArticlePubMedRescueKind::Unique)
    } else if pubmed_led {
        Some(ArticlePubMedRescueKind::Led)
    } else {
        None
    };

    (
        pubmed_rescue_kind.is_some(),
        pubmed_rescue_kind,
        pubmed_rescue_kind.map(|_| pubmed_position),
    )
}

fn lexical_ranking_metadata(
    row: &ArticleSearchResult,
    source_positions: &[ArticleSourcePosition],
    anchors: &[String],
) -> ArticleRankingMetadata {
    let (title_hits, abstract_hits, combined_hits) = lexical_anchor_hits(row, anchors);
    let anchor_count = anchors.len();
    let all_anchors_in_title = anchor_count > 0 && title_hits == anchor_count;
    let all_anchors_in_text = anchor_count > 0 && combined_hits == anchor_count;
    let directness_tier = if all_anchors_in_title {
        3
    } else if all_anchors_in_text {
        2
    } else if combined_hits > 0 {
        1
    } else {
        0
    };
    let study_or_review_cue = has_study_or_review_cue(row);
    let (pubmed_rescue, pubmed_rescue_kind, pubmed_source_position) =
        pubmed_rescue_metadata(row, source_positions, directness_tier, combined_hits);

    ArticleRankingMetadata {
        directness_tier,
        anchor_count: anchor_count.min(u8::MAX as usize) as u8,
        title_anchor_hits: title_hits.min(u8::MAX as usize) as u8,
        abstract_anchor_hits: abstract_hits.min(u8::MAX as usize) as u8,
        combined_anchor_hits: combined_hits.min(u8::MAX as usize) as u8,
        all_anchors_in_title,
        all_anchors_in_text,
        study_or_review_cue,
        pubmed_rescue,
        pubmed_rescue_kind,
        pubmed_source_position,
        mode: None,
        semantic_score: None,
        lexical_score: None,
        citation_score: None,
        position_score: None,
        composite_score: None,
        avg_source_rank: None,
    }
}

fn lexical_anchor_hits(row: &ArticleSearchResult, anchors: &[String]) -> (usize, usize, usize) {
    let title_hits = anchors
        .iter()
        .filter(|anchor| anchor_matches_text(&row.normalized_title, anchor))
        .count();
    let abstract_hits = anchors
        .iter()
        .filter(|anchor| anchor_matches_text(&row.normalized_abstract, anchor))
        .count();
    let combined_hits = anchors
        .iter()
        .filter(|anchor| {
            anchor_matches_text(&row.normalized_title, anchor)
                || anchor_matches_text(&row.normalized_abstract, anchor)
        })
        .count();
    (title_hits, abstract_hits, combined_hits)
}

fn populate_lexical_ranking_metadata(
    rows: &mut [ArticleCandidate],
    filters: &ArticleSearchFilters,
) {
    let anchors = build_anchor_set(filters);

    for row in rows.iter_mut() {
        ensure_matched_sources(&mut row.row);
        row.row.ranking = Some(lexical_ranking_metadata(
            &row.row,
            &row.source_positions,
            &anchors,
        ));
    }
}

fn compare_article_candidates_lexical(
    left: &ArticleCandidate,
    right: &ArticleCandidate,
) -> Ordering {
    let left_ranking = left.row.ranking.as_ref();
    let right_ranking = right.row.ranking.as_ref();
    right_ranking
        .map(|ranking| ranking.pubmed_rescue)
        .cmp(&left_ranking.map(|ranking| ranking.pubmed_rescue))
        .then_with(|| {
            right_ranking
                .map(|ranking| ranking.directness_tier)
                .cmp(&left_ranking.map(|ranking| ranking.directness_tier))
        })
        .then_with(|| {
            right_ranking
                .map(|ranking| ranking.title_anchor_hits)
                .cmp(&left_ranking.map(|ranking| ranking.title_anchor_hits))
        })
        .then_with(|| {
            right_ranking
                .map(|ranking| ranking.combined_anchor_hits)
                .cmp(&left_ranking.map(|ranking| ranking.combined_anchor_hits))
        })
        .then_with(|| {
            right_ranking
                .map(|ranking| ranking.study_or_review_cue)
                .cmp(&left_ranking.map(|ranking| ranking.study_or_review_cue))
        })
        .then_with(|| compare_optional_citations_desc(Some(&left.row), Some(&right.row)))
        .then_with(|| {
            right
                .row
                .influential_citation_count
                .cmp(&left.row.influential_citation_count)
        })
        .then_with(|| {
            left.row
                .source_local_position
                .cmp(&right.row.source_local_position)
        })
        .then_with(|| {
            stable_article_identifier(&left.row).cmp(&stable_article_identifier(&right.row))
        })
}

fn semantic_signal(candidate: &ArticleCandidate) -> f64 {
    candidate.semantic_signal.unwrap_or(0.0)
}

fn avg_source_rank(source_positions: &[ArticleSourcePosition], fallback_position: usize) -> f64 {
    if source_positions.is_empty() {
        return fallback_position as f64 + 1.0;
    }
    source_positions
        .iter()
        .map(|entry| entry.local_position as f64 + 1.0)
        .sum::<f64>()
        / source_positions.len() as f64
}

fn normalized_citation_score(citation_count: Option<u64>, max_citation_count: u64) -> f64 {
    if max_citation_count == 0 {
        return 0.0;
    }
    let citations = citation_count.unwrap_or(0) as f64;
    (1.0 + citations).ln() / (1.0 + max_citation_count as f64).ln()
}

fn normalized_position_score(avg_source_rank: f64, max_avg_source_rank: f64) -> f64 {
    if max_avg_source_rank <= 1.0 {
        0.0
    } else {
        1.0 - (avg_source_rank / max_avg_source_rank)
    }
}

pub(super) fn rank_articles_by_directness(
    rows: &mut [ArticleCandidate],
    filters: &ArticleSearchFilters,
) {
    populate_lexical_ranking_metadata(rows, filters);
    for row in rows.iter_mut() {
        if let Some(ranking) = row.row.ranking.as_mut() {
            ranking.mode = Some(ArticleRankingMode::Lexical);
        }
    }
    rows.sort_by(compare_article_candidates_lexical);
}

fn rank_articles_by_semantic(rows: &mut [ArticleCandidate], filters: &ArticleSearchFilters) {
    populate_lexical_ranking_metadata(rows, filters);
    for row in rows.iter_mut() {
        let semantic_score = semantic_signal(row);
        if let Some(ranking) = row.row.ranking.as_mut() {
            ranking.mode = Some(ArticleRankingMode::Semantic);
            ranking.semantic_score = Some(semantic_score);
        }
    }
    rows.sort_by(|left, right| {
        semantic_signal(right)
            .total_cmp(&semantic_signal(left))
            .then_with(|| compare_article_candidates_lexical(left, right))
    });
}

fn rank_articles_hybrid(rows: &mut [ArticleCandidate], filters: &ArticleSearchFilters) {
    populate_lexical_ranking_metadata(rows, filters);
    let ranking = resolve_article_ranking(filters);
    let anchors = build_anchor_set(filters);
    let max_citation_count = rows
        .iter()
        .filter_map(|candidate| candidate.row.citation_count)
        .max()
        .unwrap_or(0);
    let max_avg_source_rank = rows
        .iter()
        .map(|candidate| {
            avg_source_rank(
                &candidate.source_positions,
                candidate.row.source_local_position,
            )
        })
        .fold(0.0, f64::max);

    for row in rows.iter_mut() {
        let semantic_score = semantic_signal(row);
        let lexical_score = if anchors.is_empty() {
            0.0
        } else {
            let (_, _, combined_hits) = lexical_anchor_hits(&row.row, &anchors);
            combined_hits as f64 / anchors.len() as f64
        };
        let citation_score = normalized_citation_score(row.row.citation_count, max_citation_count);
        let avg_source_rank = avg_source_rank(&row.source_positions, row.row.source_local_position);
        let position_score = normalized_position_score(avg_source_rank, max_avg_source_rank);
        let composite_score = ranking.weights.semantic * semantic_score
            + ranking.weights.lexical * lexical_score
            + ranking.weights.citations * citation_score
            + ranking.weights.position * position_score;

        if let Some(metadata) = row.row.ranking.as_mut() {
            metadata.mode = Some(ArticleRankingMode::Hybrid);
            metadata.semantic_score = Some(semantic_score);
            metadata.lexical_score = Some(lexical_score);
            metadata.citation_score = Some(citation_score);
            metadata.position_score = Some(position_score);
            metadata.composite_score = Some(composite_score);
            metadata.avg_source_rank = Some(avg_source_rank);
        }
    }

    rows.sort_by(|left, right| {
        let left_score = left
            .row
            .ranking
            .as_ref()
            .and_then(|ranking| ranking.composite_score)
            .unwrap_or(0.0);
        let right_score = right
            .row
            .ranking
            .as_ref()
            .and_then(|ranking| ranking.composite_score)
            .unwrap_or(0.0);
        right_score.total_cmp(&left_score).then_with(|| {
            stable_article_identifier(&left.row).cmp(&stable_article_identifier(&right.row))
        })
    });
}

pub(super) fn sort_article_rows(
    rows: &mut [ArticleCandidate],
    sort: ArticleSort,
    filters: &ArticleSearchFilters,
) {
    match sort {
        ArticleSort::Relevance => match resolve_article_ranking(filters).mode {
            ArticleRankingMode::Lexical => rank_articles_by_directness(rows, filters),
            ArticleRankingMode::Semantic => rank_articles_by_semantic(rows, filters),
            ArticleRankingMode::Hybrid => rank_articles_hybrid(rows, filters),
        },
        ArticleSort::Citations => rows.sort_by(|left, right| {
            compare_optional_citations_desc(Some(&left.row), Some(&right.row))
                .then_with(|| compare_optional_dates_desc(Some(&left.row), Some(&right.row)))
                .then_with(|| left.row.pmid.cmp(&right.row.pmid))
        }),
        ArticleSort::Date => rows.sort_by(|left, right| {
            compare_optional_dates_desc(Some(&left.row), Some(&right.row))
                .then_with(|| compare_optional_citations_desc(Some(&left.row), Some(&right.row)))
                .then_with(|| left.row.pmid.cmp(&right.row.pmid))
        }),
    }
}

#[cfg(test)]
mod tests;
