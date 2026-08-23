#[test]
fn disease_markdown_renders_opentargets_scores_in_summary_and_genes_table() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: vec!["BRAF".to_string(), "NRAS".to_string()],
        gene_associations: vec![
            crate::entities::disease::DiseaseGeneAssociation {
                gene: "BRAF".to_string(),
                relationship: Some("causal".to_string()),
                source: Some("Monarch".to_string()),
                opentargets_score: Some(crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.912,
                    gwas_score: Some(0.321),
                    rare_variant_score: Some(0.654),
                    somatic_mutation_score: Some(0.876),
                }),
            },
            crate::entities::disease::DiseaseGeneAssociation {
                gene: "NRAS".to_string(),
                relationship: Some("associated".to_string()),
                source: Some("CIViC".to_string()),
                opentargets_score: None,
            },
        ],
        top_genes: vec!["BRAF".to_string(), "NRAS".to_string()],
        top_gene_scores: vec![
            crate::entities::disease::DiseaseTargetScore {
                symbol: "BRAF".to_string(),
                summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.912,
                    gwas_score: Some(0.321),
                    rare_variant_score: Some(0.654),
                    somatic_mutation_score: Some(0.876),
                },
            },
            crate::entities::disease::DiseaseTargetScore {
                symbol: "NRAS".to_string(),
                summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.701,
                    gwas_score: None,
                    rare_variant_score: None,
                    somatic_mutation_score: Some(0.443),
                },
            },
        ],
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

    let summary = disease_markdown(&disease, &[]).expect("rendered markdown");
    assert!(summary.contains("Genes (Open Targets): BRAF (OT 0.912), NRAS (OT 0.701)"));

    let genes = disease_markdown(&disease, &["genes".to_string()]).expect("rendered markdown");
    assert!(genes.contains("| Gene | Relationship | Source | OpenTargets |"));
    assert!(genes.contains("overall 0.912; GWAS 0.321; rare 0.654; somatic 0.876"));
    assert!(genes.contains("| NRAS | associated | CIViC | - |"));
}

#[test]
fn disease_markdown_renders_diagnostics_note_then_shell_safe_search_command() {
    let mut disease = disease_without_clinical_features();
    disease.id = "MONDO:9999999".to_string();
    disease.name = "rare disease; subtype".to_string();
    disease.diagnostics = Some(vec![crate::entities::diagnostic::DiagnosticSearchResult {
        source: crate::entities::diagnostic::DIAGNOSTIC_SOURCE_GTR.to_string(),
        accession: "GTR000000777.1".to_string(),
        name: "Rare Disease Molecular Panel".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer_or_lab: Some("Rare Diagnostics Lab".to_string()),
        genes: vec!["GENE1".to_string()],
        conditions: vec!["Rare disease; subtype".to_string()],
    }]);
    disease.diagnostics_note = Some(
        "Showing first 10 diagnostic matches in this disease card. Use diagnostic search with --limit and --offset for the larger result set."
            .to_string(),
    );

    let markdown = disease_markdown(&disease, &["diagnostics".to_string()]).expect("markdown");
    let row_pos = markdown
        .find("| GTR000000777.1 | Rare Disease Molecular Panel |")
        .expect("diagnostic row");
    let note_pos = markdown
        .find("Showing first 10 diagnostic matches in this disease card.")
        .expect("diagnostics note");
    let command = "See also: `biomcp search diagnostic --disease \"rare disease; subtype\" --source all --limit 50`";
    let command_pos = markdown.find(command).expect("diagnostic search command");

    assert!(row_pos < note_pos);
    assert!(note_pos < command_pos);
    assert!(markdown.contains("--source all --limit 50"));
}

pub(crate) fn proof_disease_markdown_renders_ot_only_gene_association_table() {
    let disease = Disease {
        id: "MONDO:0003864".to_string(),
        name: "chronic lymphocytic leukemia".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: vec!["TP53".to_string(), "ATM".to_string()],
        gene_associations: vec![
            crate::entities::disease::DiseaseGeneAssociation {
                gene: "TP53".to_string(),
                relationship: Some("associated with disease".to_string()),
                source: Some("OpenTargets".to_string()),
                opentargets_score: Some(crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.991,
                    gwas_score: None,
                    rare_variant_score: None,
                    somatic_mutation_score: Some(0.881),
                }),
            },
            crate::entities::disease::DiseaseGeneAssociation {
                gene: "ATM".to_string(),
                relationship: Some("associated with disease".to_string()),
                source: Some("OpenTargets".to_string()),
                opentargets_score: Some(crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.942,
                    gwas_score: None,
                    rare_variant_score: None,
                    somatic_mutation_score: Some(0.731),
                }),
            },
        ],
        top_genes: vec!["TP53".to_string(), "ATM".to_string()],
        top_gene_scores: vec![
            crate::entities::disease::DiseaseTargetScore {
                symbol: "TP53".to_string(),
                summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.991,
                    gwas_score: None,
                    rare_variant_score: None,
                    somatic_mutation_score: Some(0.881),
                },
            },
            crate::entities::disease::DiseaseTargetScore {
                symbol: "ATM".to_string(),
                summary: crate::entities::disease::DiseaseAssociationScoreSummary {
                    overall_score: 0.942,
                    gwas_score: None,
                    rare_variant_score: None,
                    somatic_mutation_score: Some(0.731),
                },
            },
        ],
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

    let genes = disease_markdown(&disease, &["genes".to_string()]).expect("rendered markdown");
    assert!(genes.contains("| Gene | Relationship | Source | OpenTargets |"));
    assert!(genes.contains(
        "| TP53 | associated with disease | OpenTargets | overall 0.991; somatic 0.881 |"
    ));
    assert!(!genes.contains("TP53, ATM"));
}

#[test]
fn disease_markdown_renders_ot_only_gene_association_table() {
    proof_disease_markdown_renders_ot_only_gene_association_table();
}

#[test]
fn disease_markdown_links_source_cells_and_footer_evidence_urls() {
    let disease = Disease {
        id: "MONDO:0009061".to_string(),
        name: "cystic fibrosis".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: vec![crate::entities::disease::DiseaseGeneAssociation {
            gene: "CFTR".to_string(),
            relationship: Some("gene associated with condition".to_string()),
            source: Some("infores:orphanet".to_string()),
            opentargets_score: None,
        }],
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0001945".to_string(),
            name: Some("Dehydration".to_string()),
            evidence: None,
            frequency: None,
            frequency_qualifier: None,
            onset_qualifier: None,
            sex_qualifier: None,
            stage_qualifier: None,
            qualifiers: Vec::new(),
            source: Some("infores:omim".to_string()),
        }],
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: vec![crate::entities::disease::DiseaseModelAssociation {
            model: "Cftr tm1Unc".to_string(),
            model_id: Some("MGI:3698752".to_string()),
            organism: Some("Mus musculus".to_string()),
            relationship: Some("model of".to_string()),
            source: Some("infores:mgi".to_string()),
            evidence_count: Some(3),
        }],
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
        xrefs: std::collections::HashMap::from([
            ("Orphanet".to_string(), "586".to_string()),
            ("OMIM".to_string(), "219700".to_string()),
        ]),
    };

    let markdown = disease_markdown(&disease, &["all".to_string()]).expect("markdown");
    let genes = markdown
        .split_once("## Associated Genes\n")
        .expect("associated genes section")
        .1
        .split("\n## ")
        .next()
        .expect("associated genes section body");
    let phenotypes = markdown
        .split_once("## Phenotypes\n")
        .expect("phenotypes section")
        .1
        .split("\n## ")
        .next()
        .expect("phenotypes section body");
    let models = markdown
        .split_once("## Models\n")
        .expect("models section")
        .1
        .split("\n## ")
        .next()
        .expect("models section body");

    assert!(genes.contains(
        "| CFTR | gene associated with condition | [infores:orphanet](https://www.orpha.net/en/disease/detail/586) | - |"
    ));
    assert!(phenotypes.contains(
        "| HP:0001945 | Dehydration | - | - | - | - | - | [infores:omim](https://www.omim.org/entry/219700) |"
    ));
    assert!(models.contains(
        "| Cftr tm1Unc | Mus musculus | model of | [infores:mgi](https://www.informatics.jax.org/accession/MGI:3698752) | 3 |"
    ));
    assert!(markdown.contains("[Orphanet](https://www.orpha.net/en/disease/detail/586)"));
    assert!(markdown.contains("[OMIM](https://www.omim.org/entry/219700)"));
    assert!(markdown.contains("[MGI](https://www.informatics.jax.org/accession/MGI:"));
}

#[test]
fn disease_markdown_renders_clinical_features_section() {
    let disease = disease_with_clinical_features();

    let markdown =
        disease_markdown(&disease, &["clinical_features".to_string()]).expect("markdown");

    assert!(markdown.contains(
        "## Clinical Features (Monarch / HPO)\n\n| HPO ID | Name | Evidence | Frequency | Onset | Sex | Stage | Source |\n|---|---|---|---|---|---|---|---|\n| HP:0000132 | Menorrhagia | IEA | Frequent | - | Female | - | infores:hpo-annotations |"
    ));
}

#[test]
fn disease_markdown_clinical_features_empty_state_is_truthful() {
    let disease = disease_without_clinical_features();

    let markdown =
        disease_markdown(&disease, &["clinical_features".to_string()]).expect("markdown");

    assert!(markdown.contains("## Clinical Features (Monarch / HPO)"));
    assert!(markdown.contains("No Monarch/HPO phenotype annotations found for this disease."));
    assert!(!markdown.contains("| HPO ID | Name | Evidence | Frequency | Onset | Sex | Stage | Source |"));
}

#[test]
fn disease_markdown_preserves_full_definition_text() {
    let full_definition = concat!(
        "A rare hypomyelinating leukodystrophy disorder in which the cause of the disease is ",
        "a variation in any of the POLR genes, including POLR1C, POLR3A or POLR3B. ",
        "It is characterized by the association of hypomyelination, hypodontia, ",
        "hypogonadotropic hypogonadism, and neurodevelopmental delay or regression."
    );
    let disease = Disease {
        id: "MONDO:0100605".to_string(),
        name: "4H leukodystrophy".to_string(),
        definition: Some(full_definition.to_string()),
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

    let markdown = disease_markdown(&disease, &[]).expect("rendered markdown");
    assert!(markdown.contains(full_definition));
    assert!(markdown.contains("hypogonadotropic hypogonadism"));
    assert!(markdown.contains("neurodevelopmental delay or regression"));
    assert!(!markdown.contains("It is characterized by the association of…"));
}

#[test]
fn disease_markdown_phenotypes_section_renders_key_features() {
    let disease = Disease {
        id: "MONDO:0008222".to_string(),
        name: "Andersen-Tawil syndrome".to_string(),
        definition: Some(
            "A potassium channel disorder characterized by periodic paralysis, prolonged QT interval, and ventricular arrhythmias."
                .to_string(),
        ),
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0000001".to_string(),
            name: Some("Periodic paralysis".to_string()),
            evidence: None,
            frequency: None,
            frequency_qualifier: Some("Very frequent (80-99%)".to_string()),
            onset_qualifier: None,
            sex_qualifier: None,
            stage_qualifier: None,
            qualifiers: Vec::new(),
            source: None,
        }],
        clinical_features: Vec::new(),
        key_features: vec![
            "periodic paralysis".to_string(),
            "prolonged QT interval".to_string(),
            "ventricular arrhythmias".to_string(),
        ],
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

    let markdown = disease_markdown(&disease, &["phenotypes".to_string()]).expect("markdown");
    assert!(markdown.contains("### Key Features"));
    assert!(markdown.contains("- periodic paralysis"));
    assert!(markdown.contains("These summarize the classic presentation; the table below is the comprehensive HPO annotation list."));
    assert!(markdown.contains("source-backed"));
}

#[test]
fn disease_markdown_phenotypes_section_renders_definition_hint_when_key_features_missing() {
    let disease = Disease {
        id: "MONDO:0001111".to_string(),
        name: "Example syndrome".to_string(),
        definition: Some("Example syndrome is a rare inherited condition.".to_string()),
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0001250".to_string(),
            name: Some("Seizure".to_string()),
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

    let markdown = disease_markdown(&disease, &["phenotypes".to_string()]).expect("markdown");
    assert!(markdown.contains(
        "Classic features are best summarized in the disease definition. Run `biomcp get disease MONDO:0001111` to review the definition; the table below is the comprehensive HPO annotation list."
    ));
    assert!(markdown.contains("source-backed"));
    assert!(!markdown.contains("### Key Features"));
}

#[test]
fn disease_markdown_phenotypes_section_without_definition_only_shows_completeness_note() {
    let disease = Disease {
        id: "MONDO:0002222".to_string(),
        name: "Undocumented syndrome".to_string(),
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
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0001250".to_string(),
            name: Some("Seizure".to_string()),
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

    let markdown = disease_markdown(&disease, &["phenotypes".to_string()]).expect("markdown");
    assert!(markdown.contains("source-backed"));
    assert!(!markdown.contains("Classic features are best summarized"));
    assert!(!markdown.contains("### Key Features"));
}

#[test]
fn disease_markdown_renders_top_variant_summary() {
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
        variants: vec![DiseaseVariantAssociation {
            variant: "BRAF V600E".to_string(),
            relationship: Some("associated with disease".to_string()),
            source: Some("CIViC".to_string()),
            evidence_count: Some(3),
        }],
        top_variant: Some(DiseaseVariantAssociation {
            variant: "BRAF V600E".to_string(),
            relationship: Some("associated with disease".to_string()),
            source: Some("CIViC".to_string()),
            evidence_count: Some(3),
        }),
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

    let markdown = disease_markdown(&disease, &["variants".to_string()]).expect("markdown");
    assert!(
        markdown.contains(
            "Top Variant: BRAF V600E - associated with disease (CIViC, 3 evidence items)"
        )
    );
}

#[test]
fn disease_markdown_renders_survival_summary_and_note() {
    let disease = Disease {
        id: "MONDO:0001234".to_string(),
        name: "chronic myeloid leukemia".to_string(),
        definition: None,
        synonyms: vec!["CML".to_string()],
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
        survival: Some(crate::entities::disease::DiseaseSurvival {
            site_code: 97,
            site_label: "Chronic Myeloid Leukemia (CML)".to_string(),
            series: vec![
                crate::entities::disease::DiseaseSurvivalSeries {
                    sex: "Both Sexes".to_string(),
                    latest_observed: Some(crate::entities::disease::DiseaseSurvivalPoint {
                        year: 2017,
                        relative_survival_rate: Some(69.4),
                        standard_error: Some(1.1),
                        lower_ci: Some(67.2),
                        upper_ci: Some(71.3),
                        modeled_relative_survival_rate: Some(70.4),
                        case_count: Some(471),
                    }),
                    latest_modeled: Some(crate::entities::disease::DiseaseSurvivalPoint {
                        year: 2018,
                        relative_survival_rate: None,
                        standard_error: None,
                        lower_ci: None,
                        upper_ci: None,
                        modeled_relative_survival_rate: Some(70.0),
                        case_count: None,
                    }),
                    points: vec![
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2016,
                            relative_survival_rate: Some(67.1),
                            standard_error: Some(1.2),
                            lower_ci: Some(64.8),
                            upper_ci: Some(69.5),
                            modeled_relative_survival_rate: Some(67.1),
                            case_count: Some(450),
                        },
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2017,
                            relative_survival_rate: Some(69.4),
                            standard_error: Some(1.1),
                            lower_ci: Some(67.2),
                            upper_ci: Some(71.3),
                            modeled_relative_survival_rate: Some(70.4),
                            case_count: Some(471),
                        },
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2018,
                            relative_survival_rate: None,
                            standard_error: None,
                            lower_ci: None,
                            upper_ci: None,
                            modeled_relative_survival_rate: Some(70.0),
                            case_count: None,
                        },
                    ],
                },
                crate::entities::disease::DiseaseSurvivalSeries {
                    sex: "Male".to_string(),
                    latest_observed: Some(crate::entities::disease::DiseaseSurvivalPoint {
                        year: 2017,
                        relative_survival_rate: Some(63.9),
                        standard_error: Some(1.6),
                        lower_ci: Some(60.8),
                        upper_ci: Some(66.9),
                        modeled_relative_survival_rate: Some(64.8),
                        case_count: Some(284),
                    }),
                    latest_modeled: Some(crate::entities::disease::DiseaseSurvivalPoint {
                        year: 2018,
                        relative_survival_rate: None,
                        standard_error: None,
                        lower_ci: None,
                        upper_ci: None,
                        modeled_relative_survival_rate: Some(64.2),
                        case_count: None,
                    }),
                    points: vec![
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2016,
                            relative_survival_rate: Some(61.3),
                            standard_error: Some(1.7),
                            lower_ci: Some(58.1),
                            upper_ci: Some(64.4),
                            modeled_relative_survival_rate: Some(61.3),
                            case_count: Some(273),
                        },
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2017,
                            relative_survival_rate: Some(63.9),
                            standard_error: Some(1.6),
                            lower_ci: Some(60.8),
                            upper_ci: Some(66.9),
                            modeled_relative_survival_rate: Some(64.8),
                            case_count: Some(284),
                        },
                        crate::entities::disease::DiseaseSurvivalPoint {
                            year: 2018,
                            relative_survival_rate: None,
                            standard_error: None,
                            lower_ci: None,
                            upper_ci: None,
                            modeled_relative_survival_rate: Some(64.2),
                            case_count: None,
                        },
                    ],
                },
            ],
        }),
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

    let markdown = disease_markdown(&disease, &["survival".to_string()]).expect("markdown");
    assert!(markdown.contains("## Survival (SEER Explorer)"));
    assert!(markdown.contains("site code 97"));
    assert!(markdown.contains("All Ages"));
    assert!(markdown.contains("All Races / Ethnicities"));
    assert!(markdown.contains(
        "| Sex | Latest observed year | 5-year relative survival | 95% CI | Cases | Latest modeled |",
    ));
    assert!(markdown.contains(
        "| Both Sexes | 2017 | 69.4% | 67.2%-71.3% | 471 | 2018: 70.0% |",
    ));
    assert!(markdown.contains("### Recent History"));
    assert!(markdown.contains("| Male | 2017 | 63.9% | 60.8%-66.9% | 284 |"));

    let mut note_disease = disease.clone();
    note_disease.survival = None;
    note_disease.survival_note =
        Some("SEER survival data not available for this condition.".to_string());

    let note_markdown =
        disease_markdown(&note_disease, &["survival".to_string()]).expect("note markdown");
    assert!(note_markdown.contains("## Survival (SEER Explorer)"));
    assert!(note_markdown.contains("SEER survival data not available for this condition."));
    assert!(!note_markdown.contains("| Sex | Latest observed year |"));
}
