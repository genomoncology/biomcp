//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

fn params(term: &str) -> PubMedESearchParams {
    PubMedESearchParams {
        term: term.into(),
        retstart: 0,
        retmax: 10,
        date_from: None,
        date_to: None,
    }
}

#[test]
fn citation_plan_sets_required_query_params_and_api_key() {
    let plan = PubMedClient::citation_plan(" 22663011 ", Some(" test-key ")).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "efetch.fcgi");
    assert_eq!(plan.query_value("db"), Some("pubmed"));
    assert_eq!(plan.query_value("retmode"), Some("xml"));
    assert_eq!(plan.query_value("id"), Some("22663011"));
    assert_eq!(plan.query_value("api_key"), Some("test-key"));
}

#[test]
fn citation_plan_rejects_non_numeric_pmid() {
    assert!(matches!(
        PubMedClient::citation_plan("PMC123", None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn esearch_plan_sets_required_query_params_and_api_key() {
    let mut request = params(" BRAF melanoma ");
    request.retstart = 5;
    request.retmax = 20;
    let plan = PubMedClient::esearch_plan(&request, Some(" test-key ")).unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "esearch.fcgi");
    assert_eq!(plan.query_value("db"), Some("pubmed"));
    assert_eq!(plan.query_value("retmode"), Some("json"));
    assert_eq!(plan.query_value("term"), Some("BRAF melanoma"));
    assert_eq!(plan.query_value("retstart"), Some("5"));
    assert_eq!(plan.query_value("retmax"), Some("20"));
    assert_eq!(plan.query_value("api_key"), Some("test-key"));
}

#[test]
fn esearch_plan_applies_date_range_params() {
    let mut request = params("BRAF");
    request.date_from = Some("2020-01-01".into());
    request.date_to = Some("2024-12-31".into());
    let plan = PubMedClient::esearch_plan(&request, None).unwrap();

    assert_eq!(plan.query_value("datetype"), Some("pdat"));
    assert_eq!(plan.query_value("mindate"), Some("2020/01/01"));
    assert_eq!(plan.query_value("maxdate"), Some("2024/12/31"));
}

#[test]
fn esearch_plan_validates_term_and_retmax() {
    assert!(matches!(
        PubMedClient::esearch_plan(&params("   "), None),
        Err(BioMcpError::InvalidArgument(_))
    ));

    let mut zero = params("BRAF");
    zero.retmax = 0;
    assert!(matches!(
        PubMedClient::esearch_plan(&zero, None),
        Err(BioMcpError::InvalidArgument(_))
    ));

    let mut too_many = params("BRAF");
    too_many.retmax = 10_001;
    assert!(matches!(
        PubMedClient::esearch_plan(&too_many, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn esummary_plan_sets_ids_and_api_key() {
    let ids = vec![" 123 ".to_string(), "456".to_string()];
    let plan = PubMedClient::esummary_plan(&ids, Some("test-key"))
        .unwrap()
        .expect("summary plan");

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "esummary.fcgi");
    assert_eq!(plan.query_value("db"), Some("pubmed"));
    assert_eq!(plan.query_value("retmode"), Some("json"));
    assert_eq!(plan.query_value("id"), Some("123,456"));
    assert_eq!(plan.query_value("api_key"), Some("test-key"));
}

#[test]
fn esummary_plan_handles_empty_and_blank_ids() {
    assert!(PubMedClient::esummary_plan(&[], None).unwrap().is_none());

    let ids = vec!["123".to_string(), "   ".to_string()];
    assert!(matches!(
        PubMedClient::esummary_plan(&ids, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
