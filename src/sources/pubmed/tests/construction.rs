//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

fn client_with_api_key(api_key: Option<&str>) -> PubMedClient {
    PubMedClient {
        client: crate::sources::shared_client().expect("shared client"),
        base: std::borrow::Cow::Borrowed("http://127.0.0.1"),
        api_key: api_key.map(str::to_string),
    }
}

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
fn citation_diagnostic_plan_redacts_key_and_reports_cache_mode() {
    let keyed = client_with_api_key(Some("super-secret-ncbi"));
    let plan = keyed
        .citation_request_plan("22663011")
        .expect("citation plan");
    assert_eq!(plan.path, "/efetch.fcgi");
    assert_eq!(plan.content_type_expectation, "xml");
    assert_eq!(plan.cache_mode, "auth");
    assert_eq!(plan.auth_mode, "authenticated");
    assert!(
        !plan
            .query_params
            .iter()
            .any(|(_, value)| value.contains("super-secret"))
    );

    let keyless = client_with_api_key(None)
        .citation_request_plan("22663011")
        .expect("keyless citation plan");
    assert_eq!(keyless.cache_mode, "default");
    assert_eq!(keyless.auth_mode, "keyless");
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

#[test]
fn legacy_request_plans_keep_article_contract_shape() {
    let client = client_with_api_key(Some("super-secret-ncbi"));

    let esearch: PubMedESearchRequestPlan = client
        .esearch_request_plan(&PubMedESearchParams {
            term: " BRAF melanoma ".into(),
            retstart: 0,
            retmax: 20,
            date_from: Some("2020-01-01".into()),
            date_to: None,
        })
        .expect("PubMedESearchRequestPlan");
    assert_eq!(esearch.method, "GET");
    assert_eq!(esearch.path, "/esearch.fcgi");
    assert!(
        esearch
            .query_params
            .contains(&("term", "BRAF melanoma".to_string()))
    );
    assert!(esearch.query_params.contains(&("retmax", "20".to_string())));
    assert_eq!(esearch.auth_mode, "authenticated");
    assert!(
        !esearch
            .query_params
            .iter()
            .any(|(_, value)| value.contains("super-secret"))
    );

    let ids = vec!["123".to_string(), "456".to_string()];
    let esummary: PubMedESummaryRequestPlan = client
        .esummary_request_plan(&ids)
        .expect("plan")
        .expect("PubMedESummaryRequestPlan");
    assert_eq!(esummary.method, "GET");
    assert_eq!(esummary.path, "/esummary.fcgi");
    assert!(
        esummary
            .query_params
            .contains(&("id", "123,456".to_string()))
    );
    assert_eq!(esummary.content_type_expectation, "json");

    let keyless = client_with_api_key(None);
    let keyless_plan = keyless
        .esummary_request_plan(&ids)
        .expect("keyless plan")
        .expect("summary plan");
    assert_eq!(keyless_plan.cache_mode, "default");
    assert_eq!(keyless_plan.auth_mode, "keyless");
}
