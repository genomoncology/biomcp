//! Merge acquired membership and provenance without provider work.

use super::ArticleCandidate;
use crate::entities::article::candidates::merge_article_candidate_pool;

pub(super) fn merge_candidates(mut candidates: Vec<ArticleCandidate>) -> Vec<ArticleCandidate> {
    for candidate in &mut candidates {
        for trace in &mut candidate.candidate_trace {
            trace.after_union = true;
        }
    }
    let mut candidates = merge_article_candidate_pool(candidates);
    for candidate in &mut candidates {
        for trace in &mut candidate.candidate_trace {
            trace.after_dedup = true;
        }
    }
    candidates
}
