use super::*;
use crate::entities::drug::{Drug, DrugInteractionReport};
use crate::entities::variant::Variant;

#[test]
fn variant_json_next_commands_parse() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "rs113488022",
        "gene": "BRAF",
        "hgvs_p": "p.V600E",
        "rsid": "rs113488022"
    }))
    .expect("variant should deserialize");

    assert_entity_json_next_commands(
        "variant",
        &variant,
        crate::render::markdown::variant_evidence_urls(&variant),
        crate::render::markdown::related_variant(&variant),
        crate::render::provenance::variant_section_sources(&variant),
    );
}

#[test]
fn variant_json_next_commands_include_vus_literature_route() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr2:g.166848047C>G",
        "gene": "SCN1A",
        "hgvs_p": "p.T1174S",
        "legacy_name": "SCN1A T1174S",
        "significance": "Uncertain significance",
        "top_disease": {"condition": "Dravet syndrome", "reports": 7}
    }))
    .expect("variant should deserialize");

    let next_commands = crate::render::markdown::related_variant(&variant);
    let json = crate::render::json::to_entity_json(
        &variant,
        crate::render::markdown::variant_evidence_urls(&variant),
        next_commands,
        crate::render::provenance::variant_section_sources(&variant),
    )
    .expect("variant json");
    assert_json_next_commands_parse("variant-vus", &json);
    assert!(
        collect_next_commands(&json).contains(
            &"biomcp search article -g SCN1A -d \"Dravet syndrome\" -k \"T1174S\" --limit 5"
                .to_string()
        )
    );
}

#[test]
fn drug_json_next_commands_parse() {
    let drug = Drug {
        name: "osimertinib".to_string(),
        drugbank_id: Some("DB09330".to_string()),
        chembl_id: Some("CHEMBL3353410".to_string()),
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: vec!["EGFR".to_string()],
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    assert_entity_json_next_commands(
        "drug",
        &drug,
        crate::render::markdown::drug_evidence_urls(&drug),
        crate::render::markdown::related_drug(&drug),
        crate::render::provenance::drug_section_sources(&drug),
    );
}

#[test]
fn drug_interaction_report_json_next_commands_include_helper_follow_ups() {
    let report = DrugInteractionReport {
        name: "warfarin".to_string(),
        drugbank_id: Some("DB00682".to_string()),
        chembl_id: Some("CHEMBL88".to_string()),
        interactions: Vec::new(),
        class_summaries: Vec::new(),
        source_note: Some(
            "Structured rows come from the current DDInter download bundle.".to_string(),
        ),
        label_interaction_text: None,
    };

    let json = crate::render::json::to_entity_json(
        &report,
        crate::render::markdown::drug_interaction_report_evidence_urls(&report),
        crate::render::markdown::related_drug_interactions(&report.name),
        crate::render::provenance::drug_interaction_report_section_sources(&report),
    )
    .expect("drug interaction report json");

    assert_json_next_commands_parse("drug-interaction-report", &json);
    let next_commands = collect_next_commands(&json);
    assert_eq!(
        next_commands,
        vec![
            "biomcp get drug warfarin safety".to_string(),
            "biomcp search article --drug warfarin --limit 5".to_string(),
        ]
    );
}
