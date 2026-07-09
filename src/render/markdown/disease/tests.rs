use super::*;
use crate::entities::disease::{DiseasePhenotype, DiseaseVariantAssociation};

fn clinical_feature_row() -> DiseasePhenotype {
    DiseasePhenotype {
        hpo_id: "HP:0000132".to_string(),
        name: Some("Menorrhagia".to_string()),
        evidence: Some("IEA".to_string()),
        frequency: None,
        frequency_qualifier: Some("Frequent".to_string()),
        onset_qualifier: None,
        sex_qualifier: Some("Female".to_string()),
        stage_qualifier: None,
        qualifiers: Vec::new(),
        source: Some("infores:hpo-annotations".to_string()),
    }
}

fn disease_with_clinical_features() -> Disease {
    Disease {
        id: "MONDO:0004277".to_string(),
        name: "uterine leiomyoma".to_string(),
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
        clinical_features: vec![clinical_feature_row()],
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
    }
}

fn disease_without_clinical_features() -> Disease {
    let mut disease = disease_with_clinical_features();
    disease.id = "MONDO:0005105".to_string();
    disease.name = "melanoma".to_string();
    disease.clinical_features.clear();
    disease
}

#[test]
fn ticket_377_disease_renderer_envelope_contracts() {
    let mut disease = disease_without_clinical_features();
    disease.associated_genes = vec!["BRAF".to_string()];
    disease.gene_associations = vec![crate::entities::disease::DiseaseGeneAssociation {
        gene: "BRAF".to_string(),
        relationship: Some("associated".to_string()),
        source: Some("OpenTargets".to_string()),
        opentargets_score: Some(crate::entities::disease::DiseaseAssociationScoreSummary {
            overall_score: 0.912,
            gwas_score: None,
            rare_variant_score: None,
            somatic_mutation_score: Some(0.876),
        }),
    }];
    disease.top_genes = vec!["BRAF".to_string()];

    let next_commands = crate::render::markdown::disease_next_commands(&disease, &[]);
    let section_sources = crate::render::provenance::disease_section_sources(&disease);
    let json = crate::render::json::to_entity_json_with_suggestions(
        &disease,
        vec![(
            "MONDO",
            "https://monarchinitiative.org/MONDO:0005105".to_string(),
        )],
        next_commands,
        vec!["biomcp search article -d melanoma".to_string()],
        section_sources,
    )
    .expect("disease JSON renderer envelope");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid disease JSON");

    assert_eq!(value["_meta"]["evidence_urls"][0]["label"], "MONDO");
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| {
                commands.iter().any(|command| {
                    command.as_str() == Some("biomcp get disease MONDO:0005105 genes")
                })
            })
    );
    assert_eq!(
        value["_meta"]["suggestions"][0],
        "biomcp search article -d melanoma"
    );
    assert!(
        value["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|sources| {
                sources.iter().any(|source| {
                    source["key"] == "associated_genes"
                        && source["sources"].as_array().is_some_and(|labels| {
                            labels.iter().any(|label| label == "Open Targets")
                        })
                })
            })
    );

    let markdown = disease_markdown(&disease, &["genes".to_string()]).expect("disease_markdown");
    assert!(markdown.contains("| Gene | Relationship | Source | OpenTargets |"));
    assert!(markdown.contains("| BRAF | associated | OpenTargets |"));
}

include!("tests/rendering.rs");
include!("tests/extended.rs");
