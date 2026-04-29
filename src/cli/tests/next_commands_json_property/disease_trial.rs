use super::*;
use crate::entities::disease::Disease;
use crate::entities::pgx::Pgx;
use crate::entities::trial::Trial;

#[test]
fn disease_json_next_commands_parse() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let next_commands = crate::render::markdown::disease_next_commands(&disease, &[]);
    assert_eq!(
        next_commands
            .iter()
            .take(6)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![
            "biomcp get disease MONDO:0005105 genes",
            "biomcp get disease MONDO:0005105 pathways",
            "biomcp get disease MONDO:0005105 phenotypes",
            "biomcp get disease MONDO:0005105 diagnostics",
            "biomcp get disease MONDO:0005105 clinical_features",
            "biomcp get disease MONDO:0005105 survival",
        ]
    );
    assert!(
        next_commands.contains(&"biomcp search trial -c \"melanoma\"".to_string()),
        "expected disease cross-entity helper after section follow-ups: {next_commands:?}"
    );
    assert!(next_commands.contains(&"biomcp search diagnostic --disease \"melanoma\"".to_string()));

    assert_entity_json_next_commands(
        "disease",
        &disease,
        crate::render::markdown::disease_evidence_urls(&disease),
        next_commands,
        crate::render::provenance::disease_section_sources(&disease),
    );
}

#[test]
fn disease_json_next_commands_omit_requested_section_follow_up() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    for requested_section in ["survival", "funding"] {
        let requested_sections = [requested_section.to_string()];
        let next_commands =
            crate::render::markdown::disease_next_commands(&disease, &requested_sections);
        let json = crate::render::json::to_entity_json(
            &disease,
            crate::render::markdown::disease_evidence_urls(&disease),
            next_commands,
            crate::render::provenance::disease_section_sources(&disease),
        )
        .expect("disease json");
        let commands = collect_next_commands(&json);

        assert!(
            !commands.contains(&format!(
                "biomcp get disease MONDO:0005105 {requested_section}"
            )),
            "requested section should not be suggested again: {commands:?}"
        );
    }
}

#[test]
fn disease_json_suggestions_match_see_also_without_more_hints() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let json = crate::render::json::to_entity_json_with_suggestions(
        &disease,
        crate::render::markdown::disease_evidence_urls(&disease),
        crate::render::markdown::disease_next_commands(&disease, &[]),
        crate::render::markdown::related_disease(&disease),
        crate::render::provenance::disease_section_sources(&disease),
    )
    .expect("disease json");
    let suggestions = collect_suggestions(&json);

    assert!(suggestions.contains(&"biomcp search trial -c \"melanoma\"".to_string()));
    assert!(!suggestions.contains(&"biomcp get disease MONDO:0005105 genes".to_string()));
}

#[test]
fn disease_json_next_commands_include_top_gene_context() {
    let disease = Disease {
        id: "MONDO:0100135".to_string(),
        name: "Dravet syndrome".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: vec!["SCN1A".to_string()],
        gene_associations: Vec::new(),
        top_genes: vec!["SCN1A".to_string()],
        top_gene_scores: vec![crate::entities::disease::DiseaseTargetScore {
            symbol: "SCN1A".to_string(),
            summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                overall_score: 0.872,
                gwas_score: None,
                rare_variant_score: Some(0.997),
                somatic_mutation_score: None,
            },
        }],
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let next_commands = crate::render::markdown::disease_next_commands(&disease, &[]);
    let json = crate::render::json::to_entity_json(
        &disease,
        crate::render::markdown::disease_evidence_urls(&disease),
        next_commands,
        crate::render::provenance::disease_section_sources(&disease),
    )
    .expect("disease json");
    assert_json_next_commands_parse("disease-top-gene", &json);
    assert!(
        collect_next_commands(&json)
            .contains(&"biomcp get gene SCN1A clingen constraint".to_string())
    );
}

#[test]
fn pgx_json_next_commands_parse() {
    let pgx = Pgx {
        query: "CYP2D6".to_string(),
        gene: Some("CYP2D6".to_string()),
        drug: Some("warfarin sodium".to_string()),
        interactions: Vec::new(),
        recommendations: Vec::new(),
        frequencies: Vec::new(),
        guidelines: Vec::new(),
        annotations: Vec::new(),
        annotations_note: None,
    };

    assert_entity_json_next_commands(
        "pgx",
        &pgx,
        crate::render::markdown::pgx_evidence_urls(&pgx),
        crate::render::markdown::related_pgx(&pgx),
        crate::render::provenance::pgx_section_sources(&pgx),
    );
}

#[test]
fn trial_json_next_commands_parse() {
    let trial = Trial {
        nct_id: "NCT01234567".to_string(),
        source: None,
        title: "Example trial".to_string(),
        status: "Completed".to_string(),
        phase: None,
        study_type: None,
        age_range: None,
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["dabrafenib".to_string()],
        intervention_details: Vec::new(),
        sponsor: None,
        enrollment: None,
        summary: None,
        start_date: None,
        completion_date: None,
        eligibility_text: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    };
    let next_commands = crate::render::markdown::related_trial(&trial);
    assert!(next_commands.iter().any(|cmd| {
        cmd == "biomcp search article --drug dabrafenib -q \"NCT01234567 Example trial\" --limit 5"
    }));

    assert_entity_json_next_commands(
        "trial",
        &trial,
        crate::render::markdown::trial_evidence_urls(&trial),
        next_commands,
        crate::render::provenance::trial_section_sources(&trial),
    );
}
