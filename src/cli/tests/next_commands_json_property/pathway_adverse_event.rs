use super::*;
use crate::entities::adverse_event::{AdverseEvent, AdverseEventReport, DeviceEvent};
use crate::entities::pathway::Pathway;
use crate::entities::protein::Protein;

#[test]
fn batch_protein_json_omits_requested_section_from_next_commands() {
    let protein = Protein {
        section_outcomes: Default::default(),
        accession: "P00533".to_string(),
        entry_id: Some("EGFR_HUMAN".to_string()),
        name: "Epidermal growth factor receptor".to_string(),
        gene_symbol: Some("EGFR".to_string()),
        organism: None,
        length: None,
        function: None,
        structures: Vec::new(),
        structure_count: None,
        domains: Vec::new(),
        interactions: Vec::new(),
        complexes: Vec::new(),
    };
    let requested_sections = ["complexes".to_string()];
    let json = crate::cli::render_batch_json(std::slice::from_ref(&protein), |item| {
        crate::render::json::to_entity_json_value(
            item,
            crate::render::markdown::protein_evidence_urls(item),
            crate::render::markdown::related_protein(item, &requested_sections),
            crate::render::provenance::protein_section_sources(item),
        )
    })
    .expect("batch json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let commands = value[0]["_meta"]["next_commands"]
        .as_array()
        .expect("next_commands array")
        .iter()
        .map(|cmd| cmd.as_str().expect("command string"))
        .collect::<Vec<_>>();

    assert!(
        !commands.contains(&"biomcp get protein P00533 complexes"),
        "requested section should not be suggested again: {value}"
    );
    assert!(
        commands.contains(&"biomcp get protein P00533 structures"),
        "expected structures follow-up: {value}"
    );
    assert!(
        commands.contains(&"biomcp get gene EGFR"),
        "expected linked gene follow-up: {value}"
    );
}

#[test]
fn pathway_json_next_commands_parse() {
    let pathway = Pathway {
        section_outcomes: Default::default(),
        source: "KEGG".to_string(),
        id: "hsa05200".to_string(),
        name: "Pathways in cancer".to_string(),
        species: None,
        summary: None,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let next_commands = crate::render::markdown::related_pathway(&pathway);
    assert_eq!(
        next_commands,
        vec!["biomcp pathway drugs hsa05200".to_string()]
    );
    assert!(
        next_commands
            .iter()
            .all(|cmd| !cmd.contains("get pathway hsa05200")),
        "pathway next_commands should not repeat the current flow"
    );
    assert!(
        next_commands
            .iter()
            .all(|cmd| !cmd.contains("events") && !cmd.contains("enrichment")),
        "pathway next_commands should not suggest unsupported sections"
    );

    assert_entity_json_next_commands(
        "pathway",
        &pathway,
        crate::render::markdown::pathway_evidence_urls(&pathway),
        next_commands,
        crate::render::provenance::pathway_section_sources(&pathway),
    );
}

#[test]
fn protein_json_next_commands_parse() {
    let protein = Protein {
        section_outcomes: Default::default(),
        accession: "P00533".to_string(),
        entry_id: Some("EGFR_HUMAN".to_string()),
        name: "Epidermal growth factor receptor".to_string(),
        gene_symbol: Some("EGFR".to_string()),
        organism: None,
        length: None,
        function: None,
        structures: Vec::new(),
        structure_count: None,
        domains: Vec::new(),
        interactions: Vec::new(),
        complexes: Vec::new(),
    };

    let base_next_commands = crate::render::markdown::related_protein(&protein, &[]);
    assert!(base_next_commands.contains(&"biomcp get protein P00533 structures".to_string()));
    assert!(base_next_commands.contains(&"biomcp get protein P00533 complexes".to_string()));

    let section_next_commands =
        crate::render::markdown::related_protein(&protein, &["complexes".to_string()]);
    assert!(!section_next_commands.contains(&"biomcp get protein P00533 complexes".to_string()));
    assert!(section_next_commands.contains(&"biomcp get protein P00533 structures".to_string()));
    assert!(section_next_commands.contains(&"biomcp get gene EGFR".to_string()));

    assert_entity_json_next_commands(
        "protein",
        &protein,
        crate::render::markdown::protein_evidence_urls(&protein),
        section_next_commands,
        crate::render::provenance::protein_section_sources(&protein),
    );
}

#[test]
fn batch_adverse_event_json_uses_variant_specific_meta() {
    let faers = AdverseEvent {
        report_id: "1001".to_string(),
        drug: "osimertinib".to_string(),
        reactions: Vec::new(),
        outcomes: Vec::new(),
        patient: None,
        concomitant_medications: Vec::new(),
        reporter_type: None,
        reporter_country: None,
        indication: None,
        serious: true,
        date: None,
    };
    let device = DeviceEvent {
        report_id: "MDR-123".to_string(),
        report_number: None,
        device: "HeartValve".to_string(),
        manufacturer: None,
        event_type: None,
        date: None,
        description: None,
    };
    let reports = vec![
        AdverseEventReport::Faers(faers),
        AdverseEventReport::Device(device),
    ];

    let json = crate::cli::render_batch_json(&reports, |item| match item {
        AdverseEventReport::Faers(report) => crate::render::json::to_entity_json_value(
            item,
            crate::render::markdown::adverse_event_evidence_urls(report),
            crate::render::markdown::related_adverse_event(report),
            crate::render::provenance::adverse_event_report_section_sources(item),
        ),
        AdverseEventReport::Device(report) => crate::render::json::to_entity_json_value(
            item,
            crate::render::markdown::device_event_evidence_urls(report),
            crate::render::markdown::related_device_event(report),
            crate::render::provenance::adverse_event_report_section_sources(item),
        ),
    })
    .expect("batch json");

    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    let items = value.as_array().expect("batch array");
    assert_eq!(items.len(), 2, "json={value}");
    assert_eq!(items[0]["_meta"]["evidence_urls"][0]["label"], "OpenFDA");
    assert_eq!(items[1]["_meta"]["evidence_urls"][0]["label"], "OpenFDA");
    assert!(
        items[0]["_meta"]["evidence_urls"][0]["url"]
            .as_str()
            .is_some_and(|url| url.contains("/drug/event.json")),
        "faers report should use drug event evidence url: {value}"
    );
    assert!(
        items[1]["_meta"]["evidence_urls"][0]["url"]
            .as_str()
            .is_some_and(|url| url.contains("/device/event.json")),
        "device report should use device event evidence url: {value}"
    );
    assert!(
        items.iter().all(|item| item["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|cmds| !cmds.is_empty())),
        "each report should retain next commands: {value}"
    );
}

#[test]
fn faers_json_next_commands_parse() {
    let faers = AdverseEvent {
        report_id: "1001".to_string(),
        drug: "osimertinib".to_string(),
        reactions: Vec::new(),
        outcomes: Vec::new(),
        patient: None,
        concomitant_medications: Vec::new(),
        reporter_type: None,
        reporter_country: None,
        indication: None,
        serious: true,
        date: None,
    };
    let report = AdverseEventReport::Faers(faers.clone());

    assert_entity_json_next_commands(
        "adverse-event-faers",
        &report,
        crate::render::markdown::adverse_event_evidence_urls(&faers),
        crate::render::markdown::related_adverse_event(&faers),
        crate::render::provenance::adverse_event_report_section_sources(&report),
    );
}

#[test]
fn faers_subset_json_filters_data_commands_and_provenance_together() {
    let faers = AdverseEvent {
        report_id: "1001".to_string(),
        drug: "drug with space".to_string(),
        reactions: vec!["Rash".into()],
        outcomes: vec!["Hospitalization".into()],
        patient: Some("Adult".into()),
        concomitant_medications: vec!["other drug".into()],
        reporter_type: Some("Physician".into()),
        reporter_country: Some("US".into()),
        indication: Some("lung cancer".into()),
        serious: true,
        date: Some("2025-01-01".into()),
    };
    let sections =
        crate::entities::adverse_event::parse_sections(&["guidance".into(), "concomitant".into()])
            .expect("valid sections");
    let commands = crate::render::markdown::adverse_event_guidance_commands(&faers);
    let value = crate::render::json::to_entity_json_value(
        &crate::entities::adverse_event::FaersSubsetReport::new(&faers, sections),
        crate::render::markdown::adverse_event_evidence_urls(&faers),
        commands.clone(),
        crate::render::provenance::adverse_event_subset_section_sources(&faers, sections),
    )
    .expect("subset JSON");

    assert_eq!(value["type"], "faers");
    assert_eq!(value["data"]["report_id"], "1001");
    assert_eq!(
        value["data"]["concomitant_medications"],
        serde_json::json!(["other drug"])
    );
    assert!(value["data"].get("reactions").is_none());
    assert!(value["data"].get("outcomes").is_none());
    assert!(value["data"].get("patient").is_none());
    assert_eq!(value["_meta"]["next_commands"], serde_json::json!(commands));
    assert_eq!(
        value["_meta"]["section_sources"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        value["_meta"]["section_sources"][0]["key"],
        "concomitant_drugs"
    );
    assert_eq!(value["_meta"]["evidence_urls"][0]["label"], "OpenFDA");
}

#[test]
fn device_event_json_next_commands_parse() {
    let device = DeviceEvent {
        report_id: "MDR-123".to_string(),
        report_number: None,
        device: "HeartValve".to_string(),
        manufacturer: None,
        event_type: None,
        date: None,
        description: None,
    };
    let report = AdverseEventReport::Device(device.clone());

    assert_entity_json_next_commands(
        "adverse-event-device",
        &report,
        crate::render::markdown::device_event_evidence_urls(&device),
        crate::render::markdown::related_device_event(&device),
        crate::render::provenance::adverse_event_report_section_sources(&report),
    );
}
