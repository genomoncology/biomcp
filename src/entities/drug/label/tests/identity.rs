use super::*;

fn amivantamab_row() -> serde_json::Value {
    serde_json::json!({
        "set_id": "1466c070-9f97-4fa4-a955-6a6b59981fb8",
        "openfda": {
            "brand_name": ["Rybrevant"],
            "generic_name": ["AMIVANTAMAB-VMJW"],
            "substance_name": ["AMIVANTAMAB"]
        },
        "spl_product_data_elements": [
            "Rybrevant amivantamab-vmjw AMIVANTAMAB AMIVANTAMAB EDETATE DISODIUM HISTIDINE"
        ]
    })
}

fn tagrisso_row() -> serde_json::Value {
    serde_json::json!({
        "set_id": "5e81b4a7-b971-45e1-9c31-29cea8c87ce7",
        "spl_product_data_elements": [
            "TAGRISSO osimertinib OSIMERTINIB OSIMERTINIB MANNITOL MICROCRYSTALLINE CELLULOSE"
        ],
        "indications_and_usage": ["1 INDICATIONS AND USAGE 1.1 TAGRISSO is indicated for..."],
        "dosage_and_administration": ["2 DOSAGE AND ADMINISTRATION 2.1 Selection of dose..."]
    })
}

fn active_ingredient_row() -> serde_json::Value {
    serde_json::json!({
        "set_id": "active-ingredient-only",
        "openfda": {},
        "active_ingredients": [{"name": "Osimertinib Mesylate", "strength": "80mg"}]
    })
}

#[test]
fn identity_guard_accepts_a_sparse_spl_product_data_elements_record() {
    assert!(label_result_matches_identity(
        &tagrisso_row(),
        "osimertinib"
    ));
    assert!(label_result_matches_identity(&tagrisso_row(), "TAGRISSO"));
}

#[test]
fn identity_guard_rejects_labels_that_only_mention_the_drug() {
    // A full-text "osimertinib" search returns amivantamab's label first
    // because its sections mention osimertinib. The record's own identity is
    // Rybrevant / amivantamab-vmjw, so it must not count as osimertinib.
    let amivantamab = amivantamab_row();
    assert!(!label_result_matches_identity(&amivantamab, "osimertinib"));
    assert!(label_result_matches_identity(&amivantamab, "amivantamab"));
    assert!(label_result_matches_identity(
        &amivantamab,
        "Amivantamab-vmjw"
    ));
}

#[test]
fn identity_guard_matches_active_ingredients_as_word_sequences() {
    let row = active_ingredient_row();
    assert!(label_result_matches_identity(&row, "osimertinib mesylate"));
    assert!(label_result_matches_identity(&row, "osimertinib"));
    // The name must appear as its own word sequence, not as a substring of
    // another identity word.
    assert!(!label_result_matches_identity(
        &row,
        "osimertinib mesylate tablets"
    ));
    assert!(!label_result_matches_identity(&row, ""));
    assert!(!label_result_matches_identity(&row, "   "));
}

#[test]
fn filter_keeps_only_identity_matching_results_and_rewrites_the_total() {
    let response = serde_json::json!({
        "meta": {"results": {"skip": 0, "limit": 100, "total": 3}},
        "results": [
            amivantamab_row(),
            tagrisso_row(),
            {"set_id": "lazertinib", "openfda": {"generic_name": ["LAZERTINIB"]}}
        ]
    });

    let filtered = filter_label_response_to_identity(&response, "osimertinib")
        .expect("one identity-matching result");
    let results = filtered["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["set_id"], "5e81b4a7-b971-45e1-9c31-29cea8c87ce7");
    assert_eq!(filtered["meta"]["results"]["total"], 1);

    assert!(filter_label_response_to_identity(&response, "amivantamab").is_some());
    // A drug whose own record is absent from the full-text results keeps no
    // label at all rather than returning another drug's label.
    assert!(filter_label_response_to_identity(&response, "gefitinib").is_none());
}
