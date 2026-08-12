//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

#[test]
fn associations_by_rsid_plan_sets_path_projection_and_limit() {
    let plan = GwasClient::associations_by_rsid_plan(" RS7903146 ", 500).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(
        plan.path,
        "singleNucleotidePolymorphisms/rs7903146/associations"
    );
    assert_eq!(plan.query_value("projection"), Some("associationByStudy"));
    assert_eq!(plan.query_value("page"), Some("0"));
    assert_eq!(plan.query_value("size"), Some("200"));
}

#[test]
fn search_plans_set_expected_paths_and_queries() {
    let gene = GwasClient::association_search_plan(Some(" tcf7l2 "), None, 5).unwrap();
    assert_eq!(gene.path, "v2/associations");
    assert_eq!(gene.query_value("mapped_gene"), Some("TCF7L2"));
    assert_eq!(gene.query_value("size"), Some("5"));
    assert_eq!(gene.query_value("sort"), Some("p_value"));
    assert_eq!(gene.query_value("direction"), Some("asc"));

    let trait_plan = GwasClient::association_search_plan(None, Some("type 2 diabetes"), 5).unwrap();
    assert_eq!(trait_plan.path, "v2/associations");
    assert_eq!(trait_plan.query_value("efo_trait"), Some("type 2 diabetes"));
}

#[test]
fn plans_reject_invalid_inputs() {
    assert!(matches!(
        GwasClient::associations_by_rsid_plan("7903146", 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        GwasClient::association_search_plan(Some(""), None, 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        GwasClient::association_search_plan(None, Some(""), 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        GwasClient::association_search_plan(None, None, 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        GwasClient::association_search_plan(Some("BRAF"), Some("melanoma"), 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
