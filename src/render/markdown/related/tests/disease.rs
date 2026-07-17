#[test]
fn related_disease_suggests_review_when_phenotypes_are_sparse() {
    let disease = Disease {
        id: "MONDO:0007947".to_string(),
        name: "Marfan syndrome".to_string(),
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
        phenotypes: vec![
            crate::entities::disease::DiseasePhenotype {
                hpo_id: "HP:0001166".to_string(),
                name: Some("Arachnodactyly".to_string()),
                evidence: None,
                frequency: None,
                frequency_qualifier: None,
                onset_qualifier: None,
                sex_qualifier: None,
                stage_qualifier: None,
                qualifiers: Vec::new(),
                source: None,
            },
            crate::entities::disease::DiseasePhenotype {
                hpo_id: "HP:0002616".to_string(),
                name: Some("Aortic root dilatation".to_string()),
                evidence: None,
                frequency: None,
                frequency_qualifier: None,
                onset_qualifier: None,
                sex_qualifier: None,
                stage_qualifier: None,
                qualifiers: Vec::new(),
                source: None,
            },
        ],
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert_eq!(
        related[0],
        "biomcp search article -d \"Marfan syndrome\" --type review --limit 5"
    );
    assert!(related.contains(&"biomcp search trial -c \"Marfan syndrome\"".to_string()));
    assert!(related.contains(&"biomcp search drug --indication \"Marfan syndrome\"".to_string()));
    assert!(
        related.contains(&"biomcp search diagnostic --disease \"Marfan syndrome\"".to_string())
    );
}

#[test]
fn related_disease_promotes_top_gene_context_before_generic_pivots() {
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert_eq!(related[0], "biomcp get gene SCN1A clingen constraint");
    assert_eq!(related[1], "biomcp search trial -c \"Dravet syndrome\"");
    assert!(related.contains(&"biomcp search diagnostic --disease \"Dravet syndrome\"".to_string()));
    assert_eq!(
        related_command_description(&related[0]),
        Some("review ClinGen validity and constraint evidence for the top disease gene")
    );
}

#[test]
fn related_disease_falls_back_to_unscored_top_gene_context() {
    let disease = Disease {
        id: "MONDO:0100135".to_string(),
        name: "Dravet syndrome".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: vec!["SCN1A".to_string()],
        gene_associations: Vec::new(),
        top_genes: vec!["SCN1A".to_string()],
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert_eq!(related[0], "biomcp get gene SCN1A clingen constraint");
}

#[test]
fn related_disease_uses_synonym_when_name_is_raw_id() {
    let disease = Disease {
        id: "MONDO:0100605".to_string(),
        name: "MONDO:0100605".to_string(),
        definition: None,
        synonyms: vec!["4H leukodystrophy".to_string()],
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0001252".to_string(),
            name: Some("Hypomyelination".to_string()),
            evidence: None,
            frequency: None,
            frequency_qualifier: None,
            onset_qualifier: None,
            sex_qualifier: None,
            stage_qualifier: None,
            qualifiers: Vec::new(),
            source: None,
        }],
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert_eq!(
        related[0],
        "biomcp search article -d \"4H leukodystrophy\" --type review --limit 5"
    );
}

#[test]
fn related_disease_non_oncology_skips_study_hints() {
    let disease = Disease {
        id: "MONDO:0007947".to_string(),
        name: "Marfan syndrome".to_string(),
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert!(!related.iter().any(|cmd| cmd.contains("biomcp study ")));
}

#[test]
fn related_disease_quotes_single_word_indication_search() {
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease(&disease);
    assert!(related.contains(&"biomcp search drug --indication \"melanoma\"".to_string()));
}

#[test]
fn related_disease_oncology_without_local_match_falls_back_to_download_list() {
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
        civic: Some(crate::sources::civic::CivicContext::default()),
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let related = related_disease_with_oncology_study_id(&disease, None);

    assert!(related.contains(&"biomcp study download --list".to_string()));
}

#[test]
fn related_disease_oncology_with_local_match_prefers_top_mutated() {
    let disease = Disease {
        id: "MONDO:0007254".to_string(),
        name: "breast cancer".to_string(),
        definition: None,
        synonyms: vec!["mammary carcinoma".to_string()],
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: vec![crate::entities::disease::DiseaseTargetScore {
            symbol: "TP53".to_string(),
            summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                overall_score: 0.8,
                gwas_score: None,
                rare_variant_score: None,
                somatic_mutation_score: Some(0.4),
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let study_id = best_oncology_study_id(
        &disease,
        &[crate::sources::cbioportal_study::StudyLookupRow {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            has_mutations: true,
            terms: vec![
                "BRCA TCGA PanCan Atlas 2018".to_string(),
                "Breast Cancer".to_string(),
                "Breast Invasive Carcinoma".to_string(),
                "BRCA".to_string(),
            ],
        }],
    );
    let related = related_disease_with_oncology_study_id(&disease, study_id);

    assert!(
        related
            .contains(&"biomcp study top-mutated --study brca_tcga_pan_can_atlas_2018".to_string())
    );
    assert!(!related.contains(&"biomcp study download --list".to_string()));
}

#[test]
fn related_disease_oncology_matches_noncontiguous_carcinoma_study_labels() {
    let disease = Disease {
        id: "MONDO:0004989".to_string(),
        name: "breast carcinoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: vec![crate::entities::disease::DiseaseTargetScore {
            symbol: "TP53".to_string(),
            summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                overall_score: 0.8,
                gwas_score: None,
                rare_variant_score: None,
                somatic_mutation_score: Some(0.4),
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
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: std::collections::HashMap::new(),
    };

    let study_id = best_oncology_study_id(
        &disease,
        &[crate::sources::cbioportal_study::StudyLookupRow {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            has_mutations: true,
            terms: vec!["Breast Invasive Carcinoma".to_string()],
        }],
    );
    let related = related_disease_with_oncology_study_id(&disease, study_id);

    assert!(
        related
            .contains(&"biomcp study top-mutated --study brca_tcga_pan_can_atlas_2018".to_string())
    );
    assert!(!related.contains(&"biomcp study download --list".to_string()));
}
