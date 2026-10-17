#[allow(unused_imports)]
use super::super::candidates::article_candidate_from_row;
#[allow(unused_imports)]
use super::super::test_support::*;
#[allow(unused_imports)]
use super::super::{ARTICLE_RELEVANCE_RANKING_POLICY, ARTICLE_SEMANTIC_RANKING_POLICY};
use super::*;

fn rank_result_rows_by_directness(
    rows: &mut [ArticleSearchResult],
    filters: &ArticleSearchFilters,
) {
    let mut candidates = rows
        .iter()
        .cloned()
        .map(article_candidate_from_row)
        .collect::<Vec<_>>();
    rank_articles_by_directness(&mut candidates, filters);
    for (slot, candidate) in rows.iter_mut().zip(candidates) {
        *slot = candidate.row;
    }
}

mod calibration;
mod directness;
mod keyword;
mod policy;
