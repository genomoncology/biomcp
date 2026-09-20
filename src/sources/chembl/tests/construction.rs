//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

#[test]
fn drug_targets_plan_requests_mechanism_endpoint() {
    let plan = ChemblClient::drug_targets_plan(" CHEMBL25 ", 99).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "mechanism.json");
    assert_eq!(plan.query_value("molecule_chembl_id"), Some("CHEMBL25"));
    assert_eq!(plan.query_value("limit"), Some("25"));
}

#[test]
fn target_summary_plan_sets_target_path() {
    let plan = ChemblClient::target_summary_plan(" CHEMBL3390820 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "target/CHEMBL3390820.json");
    assert!(plan.query.is_empty());
}

#[test]
fn plans_reject_empty_identifiers() {
    assert!(matches!(
        ChemblClient::drug_targets_plan(" ", 5),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        ChemblClient::target_summary_plan(" ",),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        ChemblClient::cell_line_by_cellosaurus_plan(" "),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        ChemblClient::cell_line_assay_count_plan(" "),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

// Ticket 1214 acceptance 1: the cell line plans target the documented endpoints.

#[test]
fn cell_line_plan_joins_on_the_cellosaurus_accession() {
    let plan = ChemblClient::cell_line_by_cellosaurus_plan(" CVCL_2119 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "cell_line.json");
    assert_eq!(plan.query_value("cellosaurus_id"), Some("CVCL_2119"));
    // The join never goes through a name filter.
    assert!(!plan.has_query("cell_name"));
    assert!(!plan.has_query("cell_name__iexact"));
}

#[test]
fn assay_count_plan_asks_for_one_row_of_the_cell_line_assays() {
    let plan = ChemblClient::cell_line_assay_count_plan(" CHEMBL3706573 ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "assay.json");
    assert_eq!(plan.query_value("cell_chembl_id"), Some("CHEMBL3706573"));
    assert_eq!(plan.query_value("limit"), Some("1"));
}

#[test]
fn status_plan_reads_the_release_endpoint() {
    let plan = ChemblClient::status_plan();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "status.json");
    assert!(plan.query.is_empty());
}
