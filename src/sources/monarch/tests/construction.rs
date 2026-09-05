//! Tier 2 - request construction. Pure: builds `RequestPlan`s and asserts the
//! exact request shape. No network.

use super::super::*;
use crate::sources::HttpMethod;

#[test]
fn disease_gene_associations_plan_sets_object_gene_category_and_limit() {
    let plan = MonarchClient::disease_gene_associations_plan("MONDO:0007739", 500).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "v3/api/association");
    assert_eq!(plan.query_value("object"), Some("MONDO:0007739"));
    assert_eq!(plan.query_value("subject_category"), Some("biolink:Gene"));
    assert_eq!(plan.query_value("limit"), Some("200"));
}

#[test]
fn disease_phenotypes_plan_sets_subject_phenotype_category() {
    let plan = MonarchClient::disease_phenotypes_plan("DOID:14330", 5).unwrap();

    assert_eq!(plan.path, "v3/api/association");
    assert_eq!(plan.query_value("subject"), Some("DOID:14330"));
    assert_eq!(
        plan.query_value("object_category"),
        Some("biolink:PhenotypicFeature")
    );
    assert_eq!(plan.query_value("limit"), Some("5"));
}

#[test]
fn disease_models_plan_sets_object_genotype_category() {
    let plan = MonarchClient::disease_models_plan("MONDO:0007739", 5).unwrap();

    assert_eq!(plan.path, "v3/api/association");
    assert_eq!(plan.query_value("object"), Some("MONDO:0007739"));
    assert_eq!(
        plan.query_value("subject_category"),
        Some("biolink:Genotype")
    );
}

#[test]
fn phenotype_similarity_search_plan_normalizes_terms_and_sets_limit() {
    let plan = MonarchClient::phenotype_similarity_search_plan(&[
        "hp_0001263".into(),
        "HP:0001250".into(),
        "HP:0001250".into(),
    ])
    .unwrap();

    assert_eq!(
        plan.path,
        "v3/api/semsim/search/HP:0001263,HP:0001250/Human%20Diseases"
    );
    assert_eq!(plan.query_value("limit"), Some("50"));
}

#[test]
fn plans_reject_invalid_disease_ids_and_empty_hpo_terms() {
    let err = MonarchClient::disease_gene_associations_plan("OMIM:1", 5).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));

    let err = MonarchClient::phenotype_similarity_search_plan(&[]).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}
