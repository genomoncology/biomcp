//! Shared disease test helpers used by sidecar test modules.

#[allow(unused_imports)]
pub(super) use std::collections::{HashMap, HashSet};

#[allow(unused_imports)]
pub(super) use super::{
    Disease, DiseaseAssociationScoreSummary, DiseaseGeneAssociation, DiseaseSearchResult,
    DiseaseTargetScore, DiseaseVariantAssociation,
};
#[allow(unused_imports)]
pub(super) use crate::entities::SearchPage;
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::mydisease::MyDiseaseHit;

pub(super) fn test_disease(id: &str, name: &str) -> Disease {
    Disease {
        id: id.to_string(),
        name: name.to_string(),
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
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        civic: None,
        disgenet: None,
        section_outcomes: crate::entities::disease::default_disease_section_outcomes(),
        xrefs: HashMap::new(),
    }
}

pub(super) fn test_disease_hit(
    id: &str,
    disease_name: &str,
    mondo_synonyms: &[&str],
    do_synonyms: &[&str],
) -> MyDiseaseHit {
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "mondo": {
            "name": disease_name,
            "synonym": mondo_synonyms,
        },
        "disease_ontology": {
            "name": disease_name,
            "synonyms": do_synonyms,
        }
    }))
    .expect("valid disease hit")
}

pub(super) fn test_discover_disease_concept(
    label: &str,
    primary_id: Option<&str>,
    synonyms: &[&str],
    xrefs: &[(&str, &str)],
    match_tier: crate::entities::discover::MatchTier,
    confidence: crate::entities::discover::DiscoverConfidence,
) -> crate::entities::discover::DiscoverConcept {
    crate::entities::discover::DiscoverConcept {
        label: label.to_string(),
        primary_id: primary_id.map(str::to_string),
        primary_type: crate::entities::discover::DiscoverType::Disease,
        synonyms: synonyms.iter().map(|value| (*value).to_string()).collect(),
        xrefs: xrefs
            .iter()
            .map(|(source, id)| crate::entities::discover::ConceptXref {
                source: (*source).to_string(),
                id: (*id).to_string(),
            })
            .collect(),
        sources: Vec::new(),
        match_tier,
        confidence,
    }
}
