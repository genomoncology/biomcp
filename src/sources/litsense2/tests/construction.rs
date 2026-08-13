//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::litsense2::{LitSense2Client, LitSense2SearchRequestPlan};

#[test]
fn search_plan_sets_sentence_path_and_query() {
    let plan = LitSense2Client::search_plan("sentences/", " BRAF ").unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "sentences/");
    assert_eq!(plan.query_value("query"), Some("BRAF"));
    assert_eq!(plan.query_value("rerank"), Some("true"));
}

#[test]
fn search_plan_sets_passage_path() {
    let plan = LitSense2Client::search_plan("passages/", "BRAF").unwrap();
    assert_eq!(plan.path, "passages/");
}

#[test]
fn search_plan_rejects_empty_query() {
    let err = LitSense2Client::search_plan("sentences/", "   ").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("query is required"));
}

#[test]
fn search_plan_rejects_bad_path() {
    let err = LitSense2Client::search_plan("bad/", "BRAF").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("sentences"));
}

#[test]
fn legacy_request_plan_keeps_article_contract_shape() {
    let client = LitSense2Client::new().expect("client");
    let sentence: LitSense2SearchRequestPlan = client
        .search_request_plan("sentences/", "BRAF")
        .expect("LitSense2SearchRequestPlan");
    assert_eq!(sentence.method, "GET");
    assert_eq!(sentence.path, "/sentences/");
    assert!(
        sentence
            .query_params
            .contains(&("query", "BRAF".to_string()))
    );
    assert!(
        sentence
            .query_params
            .contains(&("rerank", "true".to_string()))
    );

    let passage = client
        .search_request_plan("passages/", "BRAF")
        .expect("passage plan");
    assert_eq!(passage.path, "/passages/");
}

#[test]
fn pubmed_hydration_contract_still_builds_esummary_plan() {
    let ids = vec!["22663011".to_string()];
    let hydration = crate::sources::pubmed::PubMedClient::esummary_plan(&ids, None)
        .expect("hydration plan")
        .expect("nonempty PubMed RequestPlan");
    assert_eq!(hydration.path, "esummary.fcgi");
    assert_eq!(hydration.query_value("id"), Some("22663011"));
}
