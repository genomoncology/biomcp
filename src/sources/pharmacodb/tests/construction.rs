//! Tier 2 — request construction. Pure: builds PharmacoDB request plans and
//! reads their bodies back. No network, no server.

use super::super::*;

fn body_json(plan: &RequestPlan) -> serde_json::Value {
    match &plan.body {
        crate::sources::RequestBody::Json(value) => value.clone(),
        other => panic!("PharmacoDB plans post JSON, found {other:?}"),
    }
}

fn query_of(plan: &RequestPlan) -> String {
    body_json(plan)["query"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

// Ticket 1205 acceptance 1: every experiments request sends `all: true` with an
// id argument and never a name argument, and the counts request asks for the row
// id and its dataset name only.

#[test]
fn every_plan_posts_to_the_one_graphql_path() {
    let plans = [
        PharmacoDbClient::cell_line_by_uid_plan("MOLM13_950_2019").unwrap(),
        PharmacoDbClient::cell_line_by_name_plan("MOLM-13").unwrap(),
        PharmacoDbClient::compound_by_name_plan("venetoclax").unwrap(),
        PharmacoDbClient::experiment_counts_plan(PharmacoFilter::CellLine(1248)),
        PharmacoDbClient::experiments_plan(PharmacoFilter::Compound(53572)),
    ];
    for plan in plans {
        assert_eq!(plan.method, crate::sources::HttpMethod::Post);
        assert_eq!(plan.path, "graphql");
        assert!(plan.query.is_empty(), "PharmacoDB carries no query string");
    }
}

#[test]
fn experiments_plans_send_all_true_with_an_id_argument_and_no_name_argument() {
    for filter in [
        PharmacoFilter::CellLine(1248),
        PharmacoFilter::Compound(53572),
        PharmacoFilter::Pair {
            cell_line_id: 1248,
            compound_id: 53572,
        },
    ] {
        for plan in [
            PharmacoDbClient::experiments_plan(filter),
            PharmacoDbClient::experiment_counts_plan(filter),
        ] {
            let query = query_of(&plan);
            assert!(query.contains("all: true"), "{query}");
            assert!(query.contains("cellLineId: $cellLineId"), "{query}");
            assert!(query.contains("compoundId: $compoundId"), "{query}");
            assert!(!query.contains("cellLineName"), "{query}");
            assert!(!query.contains("compoundName"), "{query}");
            assert!(!query.contains("per_page"), "{query}");
        }
    }
}

#[test]
fn each_filter_sends_only_the_ids_it_names() {
    let cell_line = body_json(&PharmacoDbClient::experiments_plan(
        PharmacoFilter::CellLine(1248),
    ));
    assert_eq!(
        cell_line["variables"],
        serde_json::json!({"cellLineId": 1248})
    );

    let compound = body_json(&PharmacoDbClient::experiments_plan(
        PharmacoFilter::Compound(53572),
    ));
    assert_eq!(
        compound["variables"],
        serde_json::json!({"compoundId": 53572})
    );

    let pair = body_json(&PharmacoDbClient::experiments_plan(PharmacoFilter::Pair {
        cell_line_id: 1248,
        compound_id: 53572,
    }));
    assert_eq!(
        pair["variables"],
        serde_json::json!({"cellLineId": 1248, "compoundId": 53572})
    );
}

#[test]
fn the_counts_projection_asks_for_the_id_and_the_dataset_name_only() {
    let query = query_of(&PharmacoDbClient::experiment_counts_plan(
        PharmacoFilter::CellLine(1248),
    ));
    for field in ["AAC", "IC50", "EC50", "Einf", "HS", "DSS1", "tissue", "uid"] {
        assert!(!query.contains(field), "counts asked for {field}: {query}");
    }
    assert!(query.contains("dataset {"), "{query}");
}

#[test]
fn the_row_projection_leaves_out_dss2_and_dss3() {
    let query = query_of(&PharmacoDbClient::experiments_plan(
        PharmacoFilter::CellLine(1248),
    ));
    for field in ["AAC", "IC50", "EC50", "Einf", "HS", "DSS1"] {
        assert!(
            query.contains(field),
            "row projection wants {field}: {query}"
        );
    }
    assert!(!query.contains("DSS2"), "{query}");
    assert!(!query.contains("DSS3"), "{query}");
}

#[test]
fn lookup_plans_carry_their_one_variable() {
    let uid = body_json(&PharmacoDbClient::cell_line_by_uid_plan("MOLM13_950_2019").unwrap());
    assert_eq!(
        uid["variables"],
        serde_json::json!({"cellUID": "MOLM13_950_2019"})
    );
    let name = body_json(&PharmacoDbClient::cell_line_by_name_plan(" HL-60(TB) ").unwrap());
    assert_eq!(
        name["variables"],
        serde_json::json!({"cellName": "HL-60(TB)"})
    );
    let compound = body_json(&PharmacoDbClient::compound_by_name_plan("venetoclax").unwrap());
    assert_eq!(
        compound["variables"],
        serde_json::json!({"compoundName": "venetoclax"})
    );
}

#[test]
fn a_blank_lookup_value_fails_before_any_request() {
    for error in [
        PharmacoDbClient::cell_line_by_uid_plan("  ").unwrap_err(),
        PharmacoDbClient::cell_line_by_name_plan("").unwrap_err(),
        PharmacoDbClient::compound_by_name_plan("\t").unwrap_err(),
    ] {
        assert!(matches!(error, BioMcpError::InvalidArgument(_)), "{error}");
    }
}

#[test]
fn the_recorded_dataset_list_holds_the_ten_pharmacodb_datasets() {
    assert_eq!(PHARMACODB_DATASET_NAMES.len(), 10);
    assert!(PHARMACODB_DATASET_NAMES.contains(&"GDSC1"));
    assert!(PHARMACODB_DATASET_NAMES.contains(&"gCSI"));
}
