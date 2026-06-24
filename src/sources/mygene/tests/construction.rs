//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query / body that would be sent. Nothing is sent.

use crate::error::BioMcpError;
use crate::sources::mygene::MyGeneClient;
use crate::sources::{HttpMethod, RequestBody};

#[test]
fn search_plan_sets_path_and_core_query_params() {
    let plan = MyGeneClient::search_plan("symbol:EGFR", 5, 0, None).unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "query");
    assert_eq!(plan.query_value("q"), Some("symbol:EGFR"));
    assert_eq!(plan.query_value("species"), Some("human"));
    assert_eq!(plan.query_value("size"), Some("5"));
    assert_eq!(plan.query_value("from"), Some("0"));
    assert!(!plan.has_query("chr"));
    let fields = plan.query_value("fields").expect("fields present");
    assert!(fields.contains("alias"));
    assert!(fields.contains("genomic_pos.chr"));
}

#[test]
fn search_plan_adds_chr_filter_only_when_non_empty() {
    let plan = MyGeneClient::search_plan("symbol:EGFR", 5, 0, Some("7")).unwrap();
    assert_eq!(plan.query_value("chr"), Some("7"));

    let blank = MyGeneClient::search_plan("symbol:EGFR", 5, 0, Some("  ")).unwrap();
    assert!(!blank.has_query("chr"));
}

#[test]
fn search_plan_rejects_offset_at_or_above_window() {
    let err = MyGeneClient::search_plan("symbol:EGFR", 5, 10_000, None).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--offset"));
}

#[test]
fn search_plan_rejects_offset_plus_limit_overflow() {
    let err = MyGeneClient::search_plan("symbol:EGFR", 2, 9_999, None).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--limit"));
}

#[test]
fn get_plan_default_uses_minimal_fields_quoted_symbol_and_size_one() {
    let plan = MyGeneClient::get_plan("BRAF", false).unwrap();
    assert_eq!(plan.path, "query");
    assert_eq!(plan.query_value("q"), Some("symbol:\"BRAF\""));
    assert_eq!(plan.query_value("species"), Some("human"));
    assert_eq!(plan.query_value("size"), Some("1"));
    let fields = plan.query_value("fields").expect("fields present");
    assert!(fields.contains("ensembl.gene"));
    assert!(!fields.contains("ensembl.transcript"));
}

#[test]
fn get_plan_with_transcripts_requests_transcript_and_protein_fields() {
    let plan = MyGeneClient::get_plan("BRAF", true).unwrap();
    let fields = plan.query_value("fields").expect("fields present");
    assert!(fields.contains("ensembl.transcript"));
    assert!(fields.contains("ensembl.protein"));
}

#[test]
fn get_plan_rejects_empty_symbol() {
    let err = MyGeneClient::get_plan("   ", false).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("required"));
}

#[test]
fn get_plan_rejects_overlong_symbol() {
    let err = MyGeneClient::get_plan(&"A".repeat(129), false).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("too long"));
}

#[test]
fn get_plan_rejects_invalid_symbol_characters() {
    let err = MyGeneClient::get_plan("BRAF:V600E", false).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("letters, numbers"));
}

#[test]
fn batch_symbols_plan_builds_post_form_preserving_input_order() {
    let (plan, ids) = MyGeneClient::batch_symbols_plan(&[
        " 1956 ".to_string(),
        "7157".to_string(),
        String::new(),
        "673".to_string(),
    ])
    .unwrap();
    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "gene");
    assert_eq!(ids, vec!["1956", "7157", "673"]);
    match &plan.body {
        RequestBody::Form(form) => {
            assert!(form.iter().any(|(k, v)| k == "ids" && v == "1956,7157,673"));
            assert!(form.iter().any(|(k, v)| k == "fields" && v == "symbol"));
            assert!(form.iter().any(|(k, v)| k == "species" && v == "human"));
        }
        other => panic!("expected form body, got {other:?}"),
    }
}

#[test]
fn batch_symbols_plan_rejects_empty_input() {
    let err = MyGeneClient::batch_symbols_plan(&[]).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("at least one ID"));
}

#[test]
fn batch_symbols_plan_rejects_oversized_batch() {
    let ids: Vec<String> = (1..=201).map(|n| n.to_string()).collect();
    let err = MyGeneClient::batch_symbols_plan(&ids).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("200"));
}
