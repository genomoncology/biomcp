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
fn openfda_label_generic_names_read_a_disagreeing_page_with_no_owner() {
    // Ticket 2039: the recorded "Pain Relief" label page — no row carries
    // the shelf name as its own brand, and the rows name unrelated
    // ingredients, so every generic name survives and the caller refuses.
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

    let names = openfda_label_generic_names(&response, "Pain Relief");
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"lidocaine 4%".to_string()));
    assert!(names.contains(&"acetaminophen".to_string()));
}

#[test]
fn openfda_label_generic_names_prefer_the_rows_that_own_the_brand() {
    // Ticket 2039: the Keytruda shape — the rows whose own brand equals the
    // query own the name, so the QLEX combination row never contributes.
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
            },
            {
                "openfda": {
                    "brand_name": ["KEYTRUDA"],
                    "generic_name": ["PEMBROLIZUMAB"]
                }
            }
        ]
    });

    let names = openfda_label_generic_names(&response, " Keytruda ");
    assert_eq!(names, vec!["pembrolizumab".to_string()]);
}

#[test]
fn openfda_label_generic_names_keep_disagreeing_owners_apart() {
    // Ticket 2039: the recorded "Sleep Aid" label page — two labels carry
    // the shelf name as their own brand and pair it with different
    // ingredients, so both generic names survive and the caller refuses.
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

    let names = openfda_label_generic_names(&response, "sleep aid");
    assert_eq!(
        names,
        vec![
            "doxylamine succinate".to_string(),
            "diphenhydramine hcl".to_string()
        ]
    );
}

#[test]
fn openfda_label_generic_names_take_a_lone_pairing_label() {
    // Ticket 2043's Darzalex shape, kept honest by ticket 2039: no row
    // carries the plain brand itself, and the page holds one label whose
    // qualified brand pairs the family with one plain ingredient, so that
    // lone generic name resolves the name.
    let response = serde_json::json!({
        "results": [
            {
                "openfda": {
                    "brand_name": ["Darzalex Faspro"],
                    "generic_name": ["DARATUMUMAB AND HYALURONIDASE-FIHJ (HUMAN RECOMBINANT)"]
                }
            }
        ]
    });

    let names = openfda_label_generic_names(&response, "Darzalex");
    assert_eq!(
        names,
        vec!["daratumumab and hyaluronidase-fihj (human recombinant)".to_string()]
    );
}
