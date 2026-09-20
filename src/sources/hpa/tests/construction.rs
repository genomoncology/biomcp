//! Tier 2 - request construction. Pure: builds `RequestPlan`s and asserts the
//! exact request shape. No network.

use super::super::*;
use crate::sources::HttpMethod;

#[test]
fn protein_data_plan_normalizes_ensembl_id_before_request() {
    let plan = HpaClient::protein_data_plan(" ensg00000157766.12 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "ENSG00000157766.xml");
    assert!(plan.query.is_empty());
}

#[test]
fn protein_data_plan_rejects_invalid_ensembl_id() {
    let err = HpaClient::protein_data_plan(" ").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));

    let err = HpaClient::protein_data_plan("BRAF").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

// Ticket 1213 acceptance 1: the cell line RNA plan targets the search download
// endpoint with the four documented parameters, and an unknown group fails
// before any request.

#[test]
fn cell_line_rna_plan_targets_the_search_download_endpoint() {
    let plan = HpaClient::cell_line_rna_plan(" ensg00000122025 ", "leukemia").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "api/search_download.php");
    assert_eq!(plan.query_value("search"), Some("ENSG00000122025"));
    assert_eq!(plan.query_value("format"), Some("json"));
    assert_eq!(plan.query_value("columns"), Some("g,eg,cell_RNA_leukemia"));
    assert_eq!(plan.query_value("compress"), Some("no"));
    assert_eq!(plan.query.len(), 4);
}

#[test]
fn cell_line_rna_plan_rejects_an_unknown_group_and_lists_the_names() {
    let err = HpaClient::cell_line_rna_plan("ENSG00000122025", "aml").unwrap_err();
    let BioMcpError::InvalidArgument(message) = err else {
        panic!("an unknown group is an invalid argument");
    };
    assert!(message.contains("aml"));
    assert!(message.contains("leukemia"));
    assert!(message.contains("uterine_cancer"));

    assert_eq!(HPA_CELL_LINE_GROUPS.len(), 30);
}
