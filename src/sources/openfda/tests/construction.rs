//! Tier 2 - request construction. Pure: builds OpenFDA `RequestPlan`s and
//! asserts the method / path / query that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;

#[test]
fn escape_query_value_escapes_lucene_special_chars() {
    assert_eq!(
        OpenFdaClient::escape_query_value(r#"PD-1 "checkpoint"\test"#),
        r#"PD\-1 \"checkpoint\"\\test"#
    );
}

#[test]
fn faers_search_plan_sets_query_limit_skip_and_key() {
    let plan = OpenFdaClient::faers_search_plan(" patient.drug:X ", 3, 10, Some(" key "))
        .expect("faers plan");

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "drug/event.json");
    assert_eq!(plan.query_value("search"), Some("patient.drug:X"));
    assert_eq!(plan.query_value("limit"), Some("3"));
    assert_eq!(plan.query_value("skip"), Some("10"));
    assert_eq!(plan.query_value("api_key"), Some("key"));
}

#[test]
fn faers_count_plans_try_exact_fallback() {
    let plans = OpenFdaClient::faers_count_plans(
        "patient.drug:X",
        "patient.reaction.reactionmeddrapt",
        5,
        None,
    )
    .expect("count plans");

    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].0, "patient.reaction.reactionmeddrapt");
    assert_eq!(plans[0].1.path, "drug/event.json");
    assert_eq!(
        plans[0].1.query_value("count"),
        Some("patient.reaction.reactionmeddrapt")
    );
    assert_eq!(
        plans[1].1.query_value("count"),
        Some("patient.reaction.reactionmeddrapt.exact")
    );
}

#[test]
fn label_search_plan_escapes_drug_name_and_sorts() {
    let plan =
        OpenFdaClient::label_search_plan(r#"PD-1 "drug""#, Some("test-key")).expect("label plan");

    assert_eq!(plan.path, "drug/label.json");
    assert_eq!(plan.query_value("limit"), Some("5"));
    assert_eq!(plan.query_value("sort"), Some("effective_time:desc"));
    assert_eq!(plan.query_value("api_key"), Some("test-key"));
    assert!(
        plan.query_value("search")
            .unwrap()
            .contains(r#"openfda.generic_name:"PD\-1 \"drug\"""#)
    );
}

#[test]
fn label_generic_exact_search_plan_matches_the_plain_ingredient_field() {
    let plan = OpenFdaClient::label_generic_exact_search_plan("bevacizumab", Some("test-key"))
        .expect("exact generic label plan");

    // Ticket 2043: the plain-product escalation matches the exact field —
    // never a phrase — so a biosimilar-crowded first page cannot hide the
    // plain product's own record.
    assert_eq!(plan.path, "drug/label.json");
    assert_eq!(
        plan.query_value("search"),
        Some("openfda.generic_name.exact:\"bevacizumab\"")
    );
    assert_eq!(plan.query_value("limit"), Some("5"));
    assert_eq!(plan.query_value("sort"), Some("effective_time:desc"));
    assert_eq!(plan.query_value("api_key"), Some("test-key"));

    assert!(OpenFdaClient::label_generic_exact_search_plan(" ", None).is_err());
}

#[test]
fn label_elements_search_plan_scopes_the_identity_field() {
    let plan = OpenFdaClient::label_elements_search_plan("osimertinib", Some("test-key"))
        .expect("elements label plan");

    assert_eq!(plan.path, "drug/label.json");
    assert_eq!(
        plan.query_value("search"),
        Some("spl_product_data_elements:\"osimertinib\"")
    );
    // Ticket 1300 review: the unfielded full-text phrase answer runs past
    // the 8 MiB body read limit (a full-text "cisapride" answer is 16.4 MB),
    // so the fallback queries the identity field with a small limit.
    assert_eq!(plan.query_value("limit"), Some("10"));
    assert_eq!(plan.query_value("sort"), None);
    assert_eq!(plan.query_value("api_key"), Some("test-key"));

    let escaped = OpenFdaClient::label_elements_search_plan(r#"PD-1 "drug""#, None)
        .expect("escaped elements plan");
    assert_eq!(
        escaped.query_value("search"),
        Some("spl_product_data_elements:\"PD\\-1 \\\"drug\\\"\"")
    );

    assert!(OpenFdaClient::label_elements_search_plan(" ", None).is_err());
}

#[test]
fn drug_and_device_plans_set_expected_paths() {
    let drugs = OpenFdaClient::drugsfda_search_plan("openfda.brand_name:test", 3, 0, None)
        .expect("drugsfda plan");
    assert_eq!(drugs.path, "drug/drugsfda.json");
    assert_eq!(drugs.query_value("limit"), Some("3"));
    assert_eq!(drugs.query_value("skip"), Some("0"));

    let k510 = OpenFdaClient::device_510k_search_plan("device_name:\"FoundationOne CDx\"", 3, None)
        .expect("510k plan");
    assert_eq!(k510.path, "device/510k.json");
    assert_eq!(
        k510.query_value("search"),
        Some("device_name:\"FoundationOne CDx\"")
    );

    let pma = OpenFdaClient::device_pma_search_plan("trade_name:\"FoundationOne CDx\"", 4, None)
        .expect("pma plan");
    assert_eq!(pma.path, "device/pma.json");
    assert_eq!(pma.query_value("limit"), Some("4"));
}

#[test]
fn recall_shortage_and_device_event_plans_sort_latest_first() {
    let recall = OpenFdaClient::enforcement_search_plan("reason:contamination", 2, 7, None)
        .expect("recall plan");
    assert_eq!(recall.path, "drug/enforcement.json");
    assert_eq!(
        recall.query_value("sort"),
        Some("recall_initiation_date:desc")
    );
    assert_eq!(recall.query_value("skip"), Some("7"));

    let shortage = OpenFdaClient::shortage_search_plan("generic_name:carboplatin", 2, 8, None)
        .expect("shortage plan");
    assert_eq!(shortage.path, "drug/shortages.json");
    assert_eq!(shortage.query_value("sort"), Some("update_date:desc"));
    assert_eq!(shortage.query_value("skip"), Some("8"));

    let device = OpenFdaClient::device_event_search_plan("device:insulin", 2, 9, None)
        .expect("device event plan");
    assert_eq!(device.path, "device/event.json");
    assert_eq!(device.query_value("sort"), Some("date_received:desc"));
    assert_eq!(device.query_value("skip"), Some("9"));
}

#[test]
fn plans_validate_limits_and_required_values() {
    assert!(matches!(
        OpenFdaClient::faers_search_plan("drug:x", 0, 0, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        OpenFdaClient::drugsfda_search_plan("openfda.brand_name:test", 51, 0, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        OpenFdaClient::device_510k_search_plan(" ", 3, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        OpenFdaClient::label_search_plan(" ", None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
