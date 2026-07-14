//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::europepmc::{EuropePmcClient, EuropePmcSearchRequestPlan, EuropePmcSort};

#[test]
fn search_query_plan_sets_keyword_shape_and_date_sort() {
    let plan =
        EuropePmcClient::search_query_plan(" alternative microexon ", 2, 25, EuropePmcSort::Date)
            .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "search");
    assert_eq!(plan.query_value("query"), Some("alternative microexon"));
    assert_eq!(plan.query_value("format"), Some("json"));
    assert_eq!(plan.query_value("page"), Some("2"));
    assert_eq!(plan.query_value("pageSize"), Some("25"));
    assert_eq!(plan.query_value("sort"), Some("P_PDATE_D desc"));
}

#[test]
fn search_query_plan_sets_citation_sort() {
    let plan = EuropePmcClient::search_query_plan("BRAF", 1, 5, EuropePmcSort::Citations).unwrap();
    assert_eq!(plan.query_value("sort"), Some("CITED desc"));
}

#[test]
fn search_query_plan_validates_query_and_paging() {
    assert!(matches!(
        EuropePmcClient::search_query_plan("   ", 1, 5, EuropePmcSort::Relevance),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        EuropePmcClient::search_query_plan("BRAF", 0, 5, EuropePmcSort::Relevance),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        EuropePmcClient::search_query_plan("BRAF", 1, 101, EuropePmcSort::Relevance),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn legacy_request_plan_keeps_article_contract_shape() {
    let client = EuropePmcClient::new().unwrap();
    let plan: EuropePmcSearchRequestPlan = client
        .search_query_request_plan("alternative microexon", 2, 25, EuropePmcSort::Date)
        .expect("EuropePmcSearchRequestPlan");

    assert_eq!(plan.method, "GET");
    assert_eq!(plan.path, "/search");
    assert!(
        plan.query_params
            .contains(&("query", "alternative microexon".to_string()))
    );
    assert!(plan.query_params.contains(&("page", "2".to_string())));
    assert!(plan.query_params.contains(&("pageSize", "25".to_string())));
    assert!(
        plan.query_params
            .contains(&("sort", "P_PDATE_D desc".to_string()))
    );
    assert_eq!(plan.content_type_expectation, "json");
}

#[test]
fn full_text_xml_plan_builds_id_endpoint_and_normalizes_pmc() {
    let plan = EuropePmcClient::full_text_xml_plan("MED", "22663011")
        .unwrap()
        .expect("plan");
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "22663011/fullTextXML");

    let plan = EuropePmcClient::full_text_xml_plan("PMC", "123")
        .unwrap()
        .expect("plan");
    assert_eq!(plan.path, "PMC123/fullTextXML");
}

#[test]
fn supplementary_files_plan_validates_and_normalizes_pmcid() {
    let plan = EuropePmcClient::supplementary_files_plan(" pmc11143360 ").unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "PMC11143360/supplementaryFiles");
    assert!(plan.query.is_empty());

    assert!(matches!(
        EuropePmcClient::supplementary_files_plan("PMC../unsafe"),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn full_text_xml_plan_empty_source_or_id_returns_none() {
    assert!(
        EuropePmcClient::full_text_xml_plan("", "22663011")
            .unwrap()
            .is_none()
    );
    assert!(
        EuropePmcClient::full_text_xml_plan("MED", "")
            .unwrap()
            .is_none()
    );
}
