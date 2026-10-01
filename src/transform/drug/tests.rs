
use super::*;

#[test]
fn approval_date_rejects_multibyte_input_with_matching_byte_length() {
    assert_eq!(normalize_approval_date("β23456789"), None);
}

#[test]
fn search_mechanism_ranking_prefers_kinase_moa_and_rejects_metabolism_only() {
    let fixtures = [
        serde_json::json!({
            "_id": "dabrafenib",
            "drugbank": {"name": "Dabrafenib"},
            "ndc": {"pharm_classes": [
                "Cytochrome P450 2C9 Inducers [MoA]",
                "Protein Kinase Inhibitors [MoA]"
            ]}
        }),
        serde_json::json!({
            "_id": "vemurafenib",
            "drugbank": {"name": "Vemurafenib"},
            "ndc": {"pharm_classes": [
                "Inhibitor of Serine/threonine-protein kinase B-raf [MoA]"
            ]}
        }),
        serde_json::json!({
            "_id": "metabolism-only",
            "drugbank": {"name": "Example drug"},
            "ndc": {"pharm_classes": [
                "Cytochrome P450 2C9 Inducers [MoA]"
            ]}
        }),
    ];

    let mechanisms = fixtures
        .into_iter()
        .map(|fixture| {
            let hit: MyChemHit = serde_json::from_value(fixture).expect("valid search hit");
            from_mychem_search_hit(&hit).and_then(|row| row.mechanism)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        mechanisms,
        vec![
            Some("Protein Kinase Inhibitors".to_string()),
            Some("Inhibitor of Serine/threonine-protein kinase B-raf".to_string()),
            None,
        ]
    );
}

#[test]
fn search_mechanism_prefers_chembl_over_ranked_moa_fallback() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "dabrafenib-with-chembl",
        "drugbank": {"name": "Dabrafenib"},
        "chembl": {"drug_mechanisms": [{
            "mechanism_of_action": "BRAF inhibitor"
        }]},
        "ndc": {"pharm_classes": ["Protein Kinase Inhibitors [MoA]"]}
    }))
    .expect("valid search hit");

    let row = from_mychem_search_hit(&hit).expect("named hit should render");
    assert_eq!(row.mechanism.as_deref(), Some("BRAF inhibitor"));
}

#[test]
fn merge_mychem_hits_collects_deduped_mechanisms() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "1",
        "_score": 1.0,
        "chembl": {
            "molecule_chembl_id": "CHEMBL1",
            "molecule_type": "Small molecule",
            "pref_name": "test",
            "drug_mechanisms": [
                {"action_type": "INHIBITOR", "target_name": "BRAF"},
                {"action_type": "INHIBITOR", "target_name": "BRAF"},
                {"action_type": "AGONIST", "target_name": "TP53"},
                {"action_type": "ANTAGONIST", "target_name": "EGFR"},
                {"action_type": "BLOCKER", "target_name": "ALK"}
            ]
        }
    }))
    .expect("valid JSON");

    let drug = merge_mychem_hits(&[&hit], "test");
    assert_eq!(drug.mechanisms.len(), 3, "mechanisms should be limited");
    assert_eq!(drug.mechanisms[0], "Inhibitor of BRAF");
    assert_eq!(drug.mechanisms[1], "Agonist of TP53");
    assert_eq!(drug.mechanisms[2], "Antagonist of EGFR");
    assert_eq!(drug.mechanism.as_deref(), Some("Inhibitor of BRAF"));
}

#[test]
fn merge_mychem_hits_feeds_anchor_synonyms_past_the_brand_cap() {
    // More than three DrugBank synonyms: the fourth still reaches
    // ddinter_synonyms even though brand_names caps at three, and
    // the interactions seam passes it on (ticket 1254).
    let anchor: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "1",
        "_score": 10.0,
        "drugbank": {
            "name": "Aspirin",
            "id": "DB00945",
            "synonyms": [
                "Bayer",
                "ECM",
                "2-Acetoxybenzoic acid",
                "acetylsalicylic acid",
                "Acenterine"
            ]
        }
    }))
    .expect("valid anchor hit");

    let drug = merge_mychem_hits(&[&anchor], "aspirin");
    assert!(
        drug.ddinter_synonyms
            .iter()
            .any(|synonym| synonym.eq_ignore_ascii_case("acetylsalicylic acid"))
    );
    assert!(drug.brand_names.len() <= 3);

    let identity =
        crate::entities::drug::interactions::ddinter_identity_for_anchor("aspirin", &drug);
    assert!(identity.terms().contains(
        &crate::sources::ddinter::normalize_name_key("acetylsalicylic acid").expect("key")
    ));
}

#[test]
fn merge_mychem_hits_does_not_pool_synonyms_across_hits() {
    // A combination product's synonym must not widen the anchor's
    // DDInter identity: pooled, "dipyridamole" would name a real
    // interaction partner and the aggregation would skip that row
    // because both sides match the anchor (ticket 1254).
    let anchor: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "1",
        "_score": 10.0,
        "drugbank": {
            "name": "Aspirin",
            "synonyms": ["Bayer", "acetylsalicylic acid"]
        }
    }))
    .expect("valid anchor hit");
    let combination: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "2",
        "_score": 5.0,
        "drugbank": {
            "name": "Aspirin/dipyridamole",
            "synonyms": ["dipyridamole", "Aggrenox"]
        }
    }))
    .expect("valid combination-product hit");

    let drug = merge_mychem_hits(&[&anchor, &combination], "aspirin");
    assert!(
        drug.ddinter_synonyms
            .iter()
            .any(|synonym| synonym.eq_ignore_ascii_case("acetylsalicylic acid"))
    );
    assert!(
        drug.ddinter_synonyms
            .iter()
            .all(|synonym| !synonym.eq_ignore_ascii_case("dipyridamole"))
    );
    assert!(
        drug.brand_names
            .iter()
            .all(|synonym| !synonym.eq_ignore_ascii_case("dipyridamole"))
    );

    let identity =
        crate::entities::drug::interactions::ddinter_identity_for_anchor("aspirin", &drug);
    assert!(
        !identity
            .terms()
            .contains(&crate::sources::ddinter::normalize_name_key("dipyridamole").expect("key"))
    );
}

#[test]
fn select_hits_for_name_matches_salt_forms() {
    let base: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "1",
        "_score": 1.0,
        "drugbank": {"name": "Dabrafenib"}
    }))
    .expect("valid base hit");

    let salt: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "2",
        "_score": 1.0,
        "chembl": {
            "pref_name": "DABRAFENIB MESYLATE",
            "molecule_chembl_id": "CHEMBL2105729",
            "drug_mechanisms": [
                {"action_type": "INHIBITOR", "target_name": "BRAF"}
            ]
        }
    }))
    .expect("valid salt hit");

    let hits = [base, salt];
    let selected = select_hits_for_name(&hits, "dabrafenib");
    assert_eq!(selected.len(), 2);
}

#[test]
fn merge_mychem_hits_collects_drug_interactions() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "1",
        "_score": 1.0,
        "drugbank": {
            "id": "DB0001",
            "name": "warfarin",
            "drug_interactions": [
                {"name": "Aspirin", "description": "May increase bleeding risk."},
                {"name": "Clopidogrel", "description": "Monitor for bleeding."}
            ]
        }
    }))
    .expect("valid JSON");

    assert_eq!(
        hit.drugbank
            .as_ref()
            .map(|d| d.drug_interactions.len())
            .unwrap_or_default(),
        2
    );
    let drug = merge_mychem_hits(&[&hit], "warfarin");
    assert_eq!(drug.interactions.len(), 2);
    assert_eq!(drug.interactions[0].drug, "Aspirin");
}

#[test]
fn drug_sections_maps_osimertinib() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "DB09330",
        "_score": 1.0,
        "drugbank": {"id": "DB09330", "name": "osimertinib"},
        "chembl": {
            "molecule_chembl_id": "CHEMBL3353410",
            "molecule_type": "Small molecule",
            "pref_name": "OSIMERTINIB",
            "drug_mechanisms": [
                {"action_type": "INHIBITOR", "target_name": "EGFR"}
            ]
        },
        "gtopdb": {
            "interaction_targets": [{"symbol": "EGFR"}]
        },
        "drugcentral": {
            "approval": [{"agency": "FDA", "date": "20151113"}],
            "drug_use": {"indication": [{"concept_name": "Non-small cell lung cancer"}]}
        }
    }))
    .expect("valid osimertinib hit");

    let drug = merge_mychem_hits(&[&hit], "osimertinib");
    assert_eq!(drug.name, "osimertinib");
    assert_eq!(drug.targets.first().map(String::as_str), Some("EGFR"));
    assert_eq!(drug.drug_type.as_deref(), Some("small-molecule"));
    assert_eq!(drug.approval_date.as_deref(), Some("2015-11-13"));
    assert_eq!(drug.approval_date_raw.as_deref(), Some("2015-11-13"));
    assert_eq!(
        drug.approval_date_display.as_deref(),
        Some("November 13, 2015")
    );
    assert_eq!(
        drug.approval_summary.as_deref(),
        Some("FDA approved on November 13, 2015")
    );
}

#[test]
fn drug_sections_maps_imatinib() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "DB00619",
        "_score": 1.0,
        "drugbank": {"id": "DB00619", "name": "imatinib"},
        "chembl": {
            "molecule_chembl_id": "CHEMBL941",
            "molecule_type": "Small molecule",
            "pref_name": "IMATINIB",
            "drug_mechanisms": [
                {"action_type": "INHIBITOR", "target_name": "ABL1"}
            ]
        },
        "gtopdb": {
            "interaction_targets": [{"symbol": "ABL1"}]
        }
    }))
    .expect("valid imatinib hit");

    let drug = merge_mychem_hits(&[&hit], "imatinib");
    assert_eq!(drug.name, "imatinib");
    assert_eq!(drug.targets.first().map(String::as_str), Some("ABL1"));
    assert!(
        drug.mechanism
            .as_deref()
            .is_some_and(|v| v.to_ascii_lowercase().contains("inhibitor"))
    );
}

#[test]
fn from_mychem_search_hit_uses_openfda_names_when_other_sources_are_missing() {
    let hit: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "openfda-only",
        "_score": 42.0,
        "openfda": {
            "brand_name": "Keytruda",
            "generic_name": "pembrolizumab"
        }
    }))
    .expect("valid openfda-only hit");

    let row = from_mychem_search_hit(&hit).expect("openfda names should produce a row");
    assert_eq!(row.name, "pembrolizumab");
}

#[test]
fn approval_date_display_formats_month_name() {
    assert_eq!(
        approval_date_display("2014-09-04").as_deref(),
        Some("September 4, 2014")
    );
    assert_eq!(
        approval_date_display("20140904").as_deref(),
        Some("September 4, 2014")
    );
}

#[test]
fn select_hits_for_name_matches_openfda_brand_name() {
    let keytruda: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "brand-hit",
        "_score": 10.0,
        "openfda": {
            "brand_name": "Keytruda",
            "generic_name": "pembrolizumab"
        }
    }))
    .expect("valid brand hit");

    let unrelated: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "other-hit",
        "_score": 1.0,
        "drugbank": {"name": "nivolumab"}
    }))
    .expect("valid unrelated hit");

    let hits = [keytruda, unrelated];
    let selected = select_hits_for_name(&hits, "keytruda");
    assert_eq!(selected.len(), 1);
}

#[test]
fn merge_mychem_hits_prefers_canonical_name_from_brand_hit() {
    let keytruda: MyChemHit = serde_json::from_value(serde_json::json!({
        "_id": "brand-hit",
        "_score": 10.0,
        "openfda": {
            "brand_name": "Keytruda",
            "generic_name": "pembrolizumab"
        }
    }))
    .expect("valid brand hit");

    let drug = merge_mychem_hits(&[&keytruda], "keytruda");
    assert_eq!(drug.name, "pembrolizumab");
}
