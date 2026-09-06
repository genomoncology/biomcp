//! Search-module tests split out from the legacy drug facade.

use super::super::test_support::*;
use super::*;

mod fallback;
mod mechanism;
mod who;

fn provider_hits(value: serde_json::Value) -> Vec<MyChemHit> {
    serde_json::from_value(value).expect("provider-shaped MyChem hits")
}

#[test]
fn ema_identity_admits_only_hits_with_exact_allowed_fields_and_preserves_field_order() {
    let hits = provider_hits(serde_json::json!([
        {
            "openfda": {
                "generic_name": ["other", " eflornithine. "],
                "brand_name": ["Vaniqa", "Second brand"]
            },
            "ndc": [
                {"nonproprietaryname": "not it"},
                {"nonproprietaryname": "Eflornithine"}
            ],
            "drugbank": {
                "name": "Eflornithine",
                "synonyms": ["2,5-diamino-2-(difluoromethyl)pentanoic acid", "acid"]
            },
            "chembl": {"pref_name": "DFMO"},
            "gtopdb": {"name": "excluded gtopdb"},
            "unii": {"display_name": "excluded unii"},
            "chebi": {"name": "excluded chebi"}
        },
        {
            "drugbank": {"name": "irrelevant", "synonyms": ["eflornithine"]},
            "openfda": {"brand_name": "must not leak"}
        }
    ]));

    let identity = ema_identity_from_mychem_hits("  EFLORNITHINE.. ", &hits)
        .expect("first hit should resolve through an allowed exact field");
    assert_eq!(
        identity.terms_for_test(),
        vec![
            ("EFLORNITHINE", "query"),
            ("other", "openfda.generic_name"),
            ("not it", "ndc.nonproprietaryname"),
            ("DFMO", "chembl.pref_name"),
            ("Vaniqa", "openfda.brand_name"),
            ("Second brand", "openfda.brand_name"),
        ]
    );
}

#[test]
fn ema_identity_has_no_all_hits_or_excluded_field_fallback() {
    for value in [
        serde_json::json!([]),
        serde_json::json!([{"drugbank": {"name": "unrelated", "synonyms": ["query"]}}]),
        serde_json::json!([{"gtopdb": {"name": "query"}}]),
        serde_json::json!([{"unii": {"display_name": "query"}}]),
        serde_json::json!([{"chebi": {"name": "query"}}]),
    ] {
        assert!(ema_identity_from_mychem_hits("query", &provider_hits(value)).is_none());
    }
}

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
