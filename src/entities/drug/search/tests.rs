//! Search-module tests split out from the legacy drug facade.

use super::super::test_support::*;
use super::*;

mod fallback;
mod mechanism;
mod who;

#[test]
fn complete_candidate_ranking_moves_a_later_exact_match_before_broad_rows() {
    let mut rows = vec![
        ("broad-page-one", DrugSearchMatchKind::BroadText),
        ("broad-page-two", DrugSearchMatchKind::BroadText),
        ("exact-page-two", DrugSearchMatchKind::ProductName),
        ("active-page-two", DrugSearchMatchKind::ActiveSubstance),
    ];
    rank_drug_candidates(&mut rows);
    assert_eq!(
        rows.into_iter().map(|(row, _)| row).collect::<Vec<_>>(),
        vec![
            "exact-page-two",
            "active-page-two",
            "broad-page-one",
            "broad-page-two"
        ]
    );
}
