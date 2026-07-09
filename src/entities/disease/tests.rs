//! Top-level disease proof-hook facade preserved for src/lib.rs tests.

use super::test_support::test_disease;
use super::{Disease, DiseasePhenotype};

pub(crate) async fn proof_augment_genes_with_opentargets_merges_sources_without_duplicates() {
    super::associations::proof_augment_genes_with_opentargets_merges_sources_without_duplicates()
        .await;
}

pub(crate) async fn proof_augment_genes_with_opentargets_respects_twenty_gene_cap() {
    super::associations::proof_augment_genes_with_opentargets_respects_twenty_gene_cap().await;
}

pub(crate) async fn proof_enrich_sparse_disease_identity_prefers_exact_ols4_match() {
    super::enrichment::proof_enrich_sparse_disease_identity_prefers_exact_ols4_match().await;
}

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

#[test]
fn disease_clinical_features_empty_serializes_as_absent() {
    let disease = test_disease("MONDO:0005105", "melanoma");

    let value = serde_json::to_value(&disease).expect("disease should serialize");

    assert!(value.get("clinical_features").is_none());
}

#[test]
fn disease_clinical_features_missing_json_deserializes_empty() {
    let disease: Disease = serde_json::from_str(r#"{"id":"MONDO:0005105","name":"melanoma"}"#)
        .expect("missing clinical_features should deserialize");

    assert!(disease.clinical_features.is_empty());
}

#[test]
fn disease_clinical_features_nonempty_serializes_rows() {
    let mut disease = test_disease("MONDO:0005105", "melanoma");
    disease.clinical_features.push(clinical_feature_row());

    let value = serde_json::to_value(&disease).expect("disease should serialize");
    let rows = value["clinical_features"]
        .as_array()
        .expect("clinical_features should serialize as rows");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["hpo_id"], "HP:0000132");
    assert_eq!(rows[0]["name"], "Menorrhagia");
    assert_eq!(rows[0]["source"], "infores:hpo-annotations");
}
