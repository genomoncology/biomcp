//! OpenFDA fallback and label-row search coverage.

use super::*;

#[test]
fn search_results_from_openfda_label_response_prefers_exact_brand_match() {
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["KEYTRUDA QLEX"],
                    "generic_name": ["Pembrolizumab and berahyaluronidase alfa-pmph"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["Keytruda"],
                    "generic_name": ["Pembrolizumab"]
                }
            }
        ]
    });

    let rows = search_results_from_openfda_label_response(&response, " Keytruda ", 5);
    let names = rows.into_iter().map(|row| row.name).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "pembrolizumab".to_string(),
            "pembrolizumab and berahyaluronidase alfa-pmph".to_string()
        ]
    );
}

#[test]
fn search_results_from_openfda_label_response_returns_remaining_unique_generics() {
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["Keytruda"],
                    "generic_name": ["Pembrolizumab"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["KEYTRUDA QLEX"],
                    "generic_name": ["Pembrolizumab and berahyaluronidase alfa-pmph"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["Keytruda refill"],
                    "generic_name": ["Pembrolizumab"]
                }
            }
        ]
    });

    let rows = search_results_from_openfda_label_response(&response, "Keytruda", 5);
    let names = rows.into_iter().map(|row| row.name).collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "pembrolizumab".to_string(),
            "pembrolizumab and berahyaluronidase alfa-pmph".to_string()
        ]
    );
}

#[test]
fn search_results_from_openfda_label_response_respects_limit() {
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["Keytruda"],
                    "generic_name": ["Pembrolizumab"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["KEYTRUDA QLEX"],
                    "generic_name": ["Pembrolizumab and berahyaluronidase alfa-pmph"]
                }
            }
        ]
    });

    let rows = search_results_from_openfda_label_response(&response, "Keytruda", 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "pembrolizumab");
}

#[test]
fn openfda_label_fallback_is_first_page_only() {
    let name_filters = DrugSearchFilters {
        query: Some("Keytruda".into()),
        ..Default::default()
    };
    let structured_filters = DrugSearchFilters {
        target: Some("EGFR".into()),
        ..Default::default()
    };
    let dummy = DrugSearchResult {
        name: "pembrolizumab".into(),
        drugbank_id: None,
        drug_type: None,
        mechanism: None,
        target: None,
    };

    assert!(should_attempt_openfda_fallback(&[], 0, &name_filters));
    assert!(!should_attempt_openfda_fallback(&[], 10, &name_filters));
    assert!(!should_attempt_openfda_fallback(
        &[],
        0,
        &structured_filters
    ));
    assert!(!should_attempt_openfda_fallback(&[dummy], 0, &name_filters));
}

#[test]
fn openfda_exact_brand_generic_names_take_only_rows_that_own_the_brand() {
    // Ticket 2039: the recorded "Pain Relief" label page — every row merely
    // mentions the shelf name inside a longer brand, so no row's record owns
    // the name and no generic name is offered.
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["4% Lidocaine Pain Relief Patch"],
                    "generic_name": ["LIDOCAINE 4%"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["Pain Relief Extra Strength"],
                    "generic_name": ["ACETAMINOPHEN"]
                }
            }
        ]
    });

    let names = openfda_exact_brand_generic_names(&response, "Pain Relief");
    assert!(names.is_empty());
}

#[test]
fn openfda_exact_brand_generic_names_keep_disagreeing_ingredients_apart() {
    // Ticket 2039: the recorded "Sleep Aid" label page — two labels carry
    // the shelf name as their own brand and pair it with different
    // ingredients, so both generic names survive and the caller can refuse.
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["Nighttime Sleep Aid Berry Flavor"],
                    "generic_name": ["DIPHENHYDRAMINE HYDROCHLORIDE"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["SLEEP AID"],
                    "generic_name": ["DOXYLAMINE SUCCINATE"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["Sleep Aid"],
                    "generic_name": ["DIPHENHYDRAMINE HCL"]
                }
            }
        ]
    });

    let names = openfda_exact_brand_generic_names(&response, "sleep aid");
    assert_eq!(
        names,
        vec![
            "doxylamine succinate".to_string(),
            "diphenhydramine hcl".to_string()
        ]
    );
}

#[test]
fn openfda_exact_brand_generic_names_deduplicate_one_ingredients_rows() {
    // Ticket 2039: one brand whose every own-brand row pairs the same
    // ingredient offers exactly that ingredient, the Keytruda shape.
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["Keytruda"],
                    "generic_name": ["Pembrolizumab"]
                }
            },
            {
                "openfda": {
                    "brand_name": ["KEYTRUDA"],
                    "generic_name": ["PEMBROLIZUMAB"]
                }
            }
        ]
    });

    let names = openfda_exact_brand_generic_names(&response, " Keytruda ");
    assert_eq!(names, vec!["pembrolizumab".to_string()]);
}
