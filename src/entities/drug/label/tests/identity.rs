use super::*;

/// The recorded openFDA full-text search for osimertinib (capture
/// `label_osimertinib_fulltext_20261007.json`): five records whose sections
/// mention osimertinib while only the sparse TAGRISSO record is the drug's
/// own. The TAGRISSO element lists every strength in one string, so each
/// inactive excipient repeats across strengths; the second review caught the
/// old token-repetition guard keeping those repeats as identity.
const OSIMERTINIB_FULLTEXT_CAPTURE: &str = include_str!(
    "../../../../../testdata/sources/openfda/label_osimertinib_fulltext_20261007.json"
);

/// The recorded openFDA full-text search for mobocertinib (capture
/// `label_mobocertinib_fulltext_20261007.json`): fourteen other records rank
/// ahead of the withdrawn EXKIVITY record, whose element carries no
/// `openfda` identity fields.
const MOBOCERTINIB_FULLTEXT_CAPTURE: &str = include_str!(
    "../../../../../testdata/sources/openfda/label_mobocertinib_fulltext_20261007.json"
);

fn capture_results(capture: &str) -> Vec<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(capture)
        .expect("capture parses")
        .get("results")
        .and_then(|v| v.as_array())
        .expect("capture carries a results array")
        .clone()
}

fn result_by_set_id(results: &[serde_json::Value], set_id: &str) -> serde_json::Value {
    results
        .iter()
        .find(|result| result.get("set_id").and_then(|v| v.as_str()) == Some(set_id))
        .unwrap_or_else(|| panic!("capture carries the {set_id} row"))
        .clone()
}

fn osimertinib_capture_row(set_id: &str) -> serde_json::Value {
    result_by_set_id(&capture_results(OSIMERTINIB_FULLTEXT_CAPTURE), set_id)
}

fn tagrisso_full_row() -> serde_json::Value {
    // The sparse TAGRISSO record: no populated openfda identity fields, and
    // an element that concatenates four per-strength entries.
    osimertinib_capture_row("5e81b4a7-b971-45e1-9c31-29cea8c87ce7")
}

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

fn active_ingredient_row() -> serde_json::Value {
    serde_json::json!({
        "set_id": "active-ingredient-only",
        "openfda": {},
        "active_ingredients": [{"name": "Osimertinib Mesylate", "strength": "80mg"}]
    })
}

fn propulsid_row() -> serde_json::Value {
    // openFDA's only cisapride record: sparse metadata, three per-strength
    // entries with differing inactive excipient lists, captured 2026-10-07.
    serde_json::json!({
        "set_id": "fdd8f491-28d6-49ae-9935-0224b8815e84",
        "openfda": {},
        "spl_product_data_elements": [PROPULSID_ELEMENTS]
    })
}

#[test]
fn identity_guard_accepts_the_sparse_multistrength_tagrisso_record() {
    let tagrisso = tagrisso_full_row();
    assert!(label_result_matches_identity(&tagrisso, "osimertinib"));
    assert!(label_result_matches_identity(&tagrisso, "TAGRISSO"));
    assert!(label_result_matches_identity(
        &tagrisso,
        "TAGRISSO osimertinib"
    ));
}

#[test]
fn identity_guard_rejects_inactive_ingredients_that_repeat_across_strengths() {
    // Ticket 1300 second review: the Tagrisso element lists the 40 mg and
    // 80 mg strengths in one string, so every inactive excipient repeats
    // across strengths. None of them may identify the record.
    let tagrisso = tagrisso_full_row();
    for name in [
        "mannitol",
        "microcrystalline cellulose",
        "titanium dioxide",
        "talc",
        "ferric oxide",
        "polyethylene glycol",
        "sodium stearyl fumarate",
        "ferrosoferric oxide",
        "low-substituted hydroxypropyl cellulose",
        "beige biconvex",
    ] {
        assert!(
            !label_result_matches_identity(&tagrisso, name),
            "{name} is an inactive excipient of the Tagrisso element"
        );
    }

    // The Rybrevant Faspro element repeats its whole entry, inactives
    // included, once per strength.
    let faspro = osimertinib_capture_row("9e58b045-e352-4f62-99bf-77fae1bebc69");
    for name in [
        "edetate disodium",
        "acetic acid",
        "methionine",
        "polysorbate 80",
        "sodium acetate",
        "sucrose",
        "water",
    ] {
        assert!(
            !label_result_matches_identity(&faspro, name),
            "{name} is an inactive excipient of the Rybrevant Faspro element"
        );
    }
    assert!(label_result_matches_identity(&faspro, "amivantamab"));
    assert!(label_result_matches_identity(&faspro, "Rybrevant"));

    // The Lazcluze entries differ per strength, and the ferric oxide
    // coloring agents differ with them; neither strength's excipients count.
    let lazcluze = osimertinib_capture_row("c417f9ee-2027-4ed5-92ad-3c19266de16c");
    for name in [
        "ferric oxide",
        "silicon dioxide",
        "croscarmellose sodium",
        "microcrystalline cellulose",
        "mannitol",
        "magnesium stearate",
        "titanium dioxide",
        "talc",
        "glyceryl monocaprylocaprate",
        "ferrosoferric oxide",
    ] {
        assert!(
            !label_result_matches_identity(&lazcluze, name),
            "{name} is an inactive excipient of the Lazcluze element"
        );
    }
    assert!(label_result_matches_identity(&lazcluze, "lazertinib"));
    assert!(label_result_matches_identity(&lazcluze, "LAZCLUZE"));

    // Datopotamab's single entry lists histidine and histidine hydrochloride
    // as two adjacent excipients, so histidine repeats without being a name.
    let datroway = osimertinib_capture_row("2950227c-6230-4ca4-a135-46e44d9424a0");
    for name in ["histidine", "sucrose", "polysorbate 80", "water"] {
        assert!(
            !label_result_matches_identity(&datroway, name),
            "{name} is an inactive excipient of the Datroway element"
        );
    }
    assert!(label_result_matches_identity(&datroway, "datopotamab"));
    assert!(label_result_matches_identity(&datroway, "Deruxtecan"));
    assert!(label_result_matches_identity(&datroway, "DATROWAY"));
}

#[test]
fn identity_guard_accepts_the_sparse_propulsid_record() {
    // Cisapride's only openFDA record: the identity-field fallback query
    // returns it, so the guard must confirm it rather than settle empty.
    let propulsid = propulsid_row();
    assert!(label_result_matches_identity(&propulsid, "cisapride"));
    assert!(label_result_matches_identity(&propulsid, "Propulsid"));
    for name in [
        "silicon dioxide",
        "lactose monohydrate",
        "magnesium stearate",
        "povidone",
        "methylparaben",
        "sodium chloride",
        "sorbitol",
        "water",
        "cherry",
    ] {
        assert!(
            !label_result_matches_identity(&propulsid, name),
            "{name} is an inactive excipient of the Propulsid element"
        );
    }
}

#[test]
fn identity_guard_rejects_token_sequences_joined_across_fields() {
    // Ticket 1300 review: a name that spans the end of one identity field and
    // the start of another (substance_name "AMIVANTAMAB" followed by the
    // element's "Rybrevant") is not the record's own name sequence.
    let amivantamab = amivantamab_row();
    assert!(!label_result_matches_identity(
        &amivantamab,
        "amivantamab rybrevant"
    ));
}

#[test]
fn identity_guard_reads_the_name_runs_of_each_element_entry() {
    // A sparse record whose only identity is the element line still matches
    // through its leading names, including a brand that appears once.
    let sporanx = serde_json::json!({
        "set_id": "sporanx",
        "spl_product_data_elements": [
            "SPORANOX ITRACONAZOLE ITRACONAZOLE ITRACONAZOLE GELATIN, UNSPECIFIED SUCROSE"
        ]
    });
    assert!(label_result_matches_identity(&sporanx, "itraconazole"));
    assert!(label_result_matches_identity(&sporanx, "Sporanox"));
    assert!(!label_result_matches_identity(&sporanx, "gelatin"));
    assert!(!label_result_matches_identity(&sporanx, "sucrose"));
}

#[test]
fn identity_guard_rejects_labels_that_only_mention_the_drug() {
    // A broad search for osimertinib returns amivantamab's label first
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
    let response: serde_json::Value =
        serde_json::from_str(OSIMERTINIB_FULLTEXT_CAPTURE).expect("capture parses");

    let filtered = filter_label_response_to_identity(&response, "osimertinib")
        .expect("one identity-matching result");
    let results = filtered["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["set_id"], "5e81b4a7-b971-45e1-9c31-29cea8c87ce7");
    assert_eq!(filtered["meta"]["results"]["total"], 1);

    assert!(filter_label_response_to_identity(&response, "amivantamab").is_some());
    // A drug whose own record is absent from the broad-search results keeps
    // no label at all rather than returning another drug's label.
    assert!(filter_label_response_to_identity(&response, "gefitinib").is_none());
}

#[test]
fn filter_keeps_the_withdrawn_exkivity_record_from_the_mobocertinib_capture() {
    let response: serde_json::Value =
        serde_json::from_str(MOBOCERTINIB_FULLTEXT_CAPTURE).expect("capture parses");
    let filtered = filter_label_response_to_identity(&response, "mobocertinib")
        .expect("one identity-matching result");
    let results = filtered["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["set_id"], "f1a91500-a944-4cb8-b4a8-ae278bcf728d");
    assert!(filter_label_response_to_identity(&response, "itraconazole").is_some());
}
