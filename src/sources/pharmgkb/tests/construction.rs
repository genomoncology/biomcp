//! Tier 2 - request construction. Pure: builds PharmGKB annotation plans and
//! asserts the path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

#[test]
fn clinpgx_default_and_annotation_queries_follow_the_live_contract() {
    assert_eq!(PHARMGKB_BASE, "https://api.clinpgx.org/v1");

    let plans = PharmGkbClient::drug_annotation_page_plans("warfarin", 10, 4).expect("drug plans");
    assert!(plans.iter().all(|plan| {
        plan.request.query_value("view") == Some("min")
            && plan.request.query_value("limit").is_none()
            && plan.request.query_value("offset").is_none()
            && plan.limit == 10
            && plan.offset == 4
    }));
}

#[test]
fn drug_annotation_plans_cover_three_annotation_kinds() {
    let plans = PharmGkbClient::drug_annotation_plans(" warfarin ", 250).expect("drug plans");

    assert_eq!(plans.len(), 3);
    assert!(plans.iter().all(|plan| plan.limit == 100));
    assert_eq!(plans[0].request.method, HttpMethod::Get);
    assert_eq!(plans[0].request.path, "data/clinicalAnnotation");
    assert_eq!(
        plans[0].request.query_value("relatedChemicals.name"),
        Some("warfarin")
    );
    assert_eq!(plans[0].request.query_value("view"), Some("min"));
    assert_eq!(plans[1].request.path, "data/guidelineAnnotation");
    assert_eq!(plans[2].request.path, "data/labelAnnotation");
}

#[test]
fn gene_annotation_plans_normalize_gene_and_properties() {
    let plans = PharmGkbClient::gene_annotation_plans(" cyp2d6 ", 5).expect("gene plans");

    assert_eq!(plans.len(), 3);
    assert_eq!(
        plans[0].request.query_value("location.genes.symbol"),
        Some("CYP2D6")
    );
    assert_eq!(
        plans[1].request.query_value("relatedGenes.symbol"),
        Some("CYP2D6")
    );
    assert_eq!(
        plans[2].request.query_value("relatedGenes.symbol"),
        Some("CYP2D6")
    );
}

#[test]
fn annotation_plans_validate_inputs() {
    assert!(matches!(
        PharmGkbClient::drug_annotation_plans(" ", 10),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        PharmGkbClient::gene_annotation_plans("CYP2D6!", 10),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
