//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::*;

fn association_response(value: serde_json::Value) -> MonarchAssociationResponse {
    serde_json::from_value(value).expect("association fixture should parse")
}

#[test]
fn map_gene_associations_maps_rows_and_relationships() {
    let resp = association_response(serde_json::json!({
        "total": 1,
        "items": [
            {
                "subject": "HGNC:4851",
                "subject_label": "HTT",
                "predicate": "biolink:gene_associated_with_condition",
                "primary_knowledge_source": "infores:orphanet",
                "object": "MONDO:0016621",
                "object_label": "juvenile Huntington disease"
            }
        ]
    }));

    let rows = MonarchClient::map_gene_associations(resp, 5);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].gene, "HTT");
    assert_eq!(
        rows[0].relationship.as_deref(),
        Some("gene associated with condition")
    );
    assert_eq!(rows[0].source.as_deref(), Some("infores:orphanet"));
}

#[test]
fn map_phenotype_associations_keeps_hpo_rows_and_qualifiers() {
    let resp = association_response(serde_json::json!({
        "items": [
            {
                "subject": "MONDO:0007739",
                "subject_label": "Disease",
                "object": "HP:0001250",
                "object_label": "Seizure",
                "predicate": "biolink:has_phenotype",
                "frequency_qualifier_label": "Frequent",
                "qualifiers_label": ["severe", "childhood"]
            },
            {"object": "MONDO:skip", "object_label": "Skip"}
        ]
    }));

    let rows = MonarchClient::map_phenotype_associations(resp, 5);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].hpo_id, "HP:0001250");
    assert_eq!(rows[0].frequency_qualifier.as_deref(), Some("Frequent"));
    assert_eq!(rows[0].qualifiers, vec!["severe", "childhood"]);
}

#[test]
fn map_model_associations_maps_genotype_rows() {
    let resp = association_response(serde_json::json!({
        "items": [
            {
                "subject": "MGI:3698752",
                "subject_label": "Htt tm1.1",
                "subject_taxon_label": "Mus musculus",
                "predicate": "biolink:model_of",
                "provided_by": "alliance_disease_edges"
            }
        ]
    }));

    let rows = MonarchClient::map_model_associations(resp, 5);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].organism.as_deref(), Some("Mus musculus"));
    assert_eq!(rows[0].source.as_deref(), Some("alliance_disease_edges"));
}

#[test]
fn map_phenotype_matches_maps_scores() {
    let rows: Vec<MonarchSemsimRow> = serde_json::from_value(serde_json::json!([
        {
            "subject": {
                "id": "MONDO:0010450",
                "name": "intellectual disability, X-linked 89"
            },
            "score": 13.302
        },
        {
            "subject": {"id": "HP:0001250", "name": "not a disease"},
            "score": 9.0
        }
    ]))
    .expect("semsim fixture should parse");

    let response = MonarchClient::map_phenotype_matches(rows);

    assert_eq!(response.raw_row_count, 2);
    assert!(!response.provider_window_exhausted);
    assert_eq!(response.matches.len(), 1);
    assert_eq!(response.matches[0].disease_id, "MONDO:0010450");
    assert!(response.matches[0].score > 0.0);
}

#[test]
fn map_phenotype_matches_counts_raw_rows_and_keeps_first_disease_occurrence() {
    let mut value = vec![serde_json::json!({
        "subject": {"id": "MONDO:0000002", "name": "second provider row"},
        "score": 7.0
    })];
    value.push(serde_json::json!({
        "subject": {"id": "MONDO:0000001", "name": "first duplicate"},
        "score": 6.0
    }));
    value.push(serde_json::json!({
        "subject": {"id": "MONDO:0000001", "name": "later duplicate"},
        "score": 99.0
    }));
    value.push(serde_json::json!({
        "subject": {"id": "HP:0001250", "name": "filtered phenotype"},
        "score": 5.0
    }));
    while value.len() < MONARCH_PHENOTYPE_WINDOW_LIMIT {
        let index = value.len();
        value.push(serde_json::json!({
            "subject": {"id": format!("MONDO:{index:07}"), "name": format!("disease {index}")},
            "score": 5.0
        }));
    }
    let rows: Vec<MonarchSemsimRow> =
        serde_json::from_value(serde_json::Value::Array(value)).unwrap();

    let response = MonarchClient::map_phenotype_matches(rows);

    assert_eq!(response.raw_row_count, MONARCH_PHENOTYPE_WINDOW_LIMIT);
    assert!(response.provider_window_exhausted);
    assert_eq!(response.matches[0].disease_id, "MONDO:0000002");
    assert_eq!(response.matches[1].disease_name, "first duplicate");
    assert_eq!(
        response
            .matches
            .iter()
            .filter(|row| row.disease_id == "MONDO:0000001")
            .count(),
        1
    );
}

#[test]
fn decode_json_response_maps_5xx_to_source_unavailable() {
    let content_type = HeaderValue::from_static("application/json");
    let err = MonarchClient::decode_json_response::<Vec<MonarchSemsimRow>>(
        StatusCode::BAD_GATEWAY,
        Some(&content_type),
        b"bad gateway",
    )
    .expect_err("5xx should be classified as source unavailable");

    match err {
        BioMcpError::SourceUnavailable {
            source_name,
            reason,
            suggestion,
        } => {
            assert_eq!(source_name, "Monarch Initiative");
            assert!(reason.contains("HTTP 502"));
            assert!(suggestion.contains("Retry later"));
        }
        other => panic!("expected SourceUnavailable, got {other}"),
    }
}

#[test]
fn phenotype_decode_failures_remain_typed_errors() {
    let json = HeaderValue::from_static("application/json");
    let html = HeaderValue::from_static("text/html");

    let status = MonarchClient::decode_json_response::<Vec<MonarchSemsimRow>>(
        StatusCode::NOT_FOUND,
        Some(&json),
        br#"{"error":"not found"}"#,
    )
    .expect_err("non-success must not become an empty phenotype page");
    assert!(matches!(status, BioMcpError::Api { .. }));

    let content_type = MonarchClient::decode_json_response::<Vec<MonarchSemsimRow>>(
        StatusCode::OK,
        Some(&html),
        b"<html>provider error</html>",
    )
    .expect_err("HTML must not become an empty phenotype page");
    assert!(matches!(
        content_type,
        BioMcpError::WithSourceContext { .. }
    ));

    let decode = MonarchClient::decode_json_response::<Vec<MonarchSemsimRow>>(
        StatusCode::OK,
        Some(&json),
        b"not-json",
    )
    .expect_err("invalid JSON must not become an empty phenotype page");
    assert!(matches!(decode, BioMcpError::ApiJson { .. }));

    let plain_json = MonarchClient::decode_json_response::<Vec<MonarchSemsimRow>>(
        StatusCode::OK,
        Some(&HeaderValue::from_static("text/plain")),
        b"[]",
    )
    .expect_err("JSON under a non-JSON content type must fail closed");
    assert!(matches!(plain_json, BioMcpError::WithSourceContext { .. }));
}

#[test]
fn direct_support_presence_and_completeness_fail_closed() {
    let subjects = vec!["MONDO:0000001".to_string(), "MONDO:0000002".to_string()];
    let objects = vec!["HP:0000256".to_string()];
    let positive = serde_json::json!({
        "subject": "MONDO:0000001",
        "object": "HP:0000256",
        "category": "biolink:DiseaseToPhenotypicFeatureAssociation",
        "predicate": "biolink:has_phenotype",
        "negated": false
    });

    let missing_total: MonarchDirectSupportResponse =
        serde_json::from_value(serde_json::json!({"items": [positive.clone()]})).unwrap();
    let support = MonarchClient::map_direct_support(missing_total, &subjects, &objects).unwrap();
    assert_eq!(
        support.status("MONDO:0000001", "HP:0000256"),
        PhenotypeDirectSupportStatus::Supported
    );
    assert_eq!(
        support.status("MONDO:0000002", "HP:0000256"),
        PhenotypeDirectSupportStatus::Indeterminate
    );

    let complete: MonarchDirectSupportResponse =
        serde_json::from_value(serde_json::json!({"total": 1, "items": [positive]})).unwrap();
    let support = MonarchClient::map_direct_support(complete, &subjects, &objects).unwrap();
    assert_eq!(
        support.status("MONDO:0000002", "HP:0000256"),
        PhenotypeDirectSupportStatus::NotSupported
    );

    for value in [
        serde_json::json!({"total": null, "items": []}),
        serde_json::json!({"total": "0", "items": []}),
        serde_json::json!({"total": 0, "items": null}),
        serde_json::json!({"total": 0, "items": {}}),
    ] {
        assert!(serde_json::from_value::<MonarchDirectSupportResponse>(value).is_err());
    }
}

#[test]
fn negated_or_filter_violating_rows_never_establish_support_or_absence() {
    let subjects = vec!["MONDO:0000001".to_string()];
    let objects = vec!["HP:0000256".to_string()];
    let response: MonarchDirectSupportResponse = serde_json::from_value(serde_json::json!({
        "total": 2,
        "items": [
            {"subject":"MONDO:0000001","object":"HP:0000256","category":"biolink:DiseaseToPhenotypicFeatureAssociation","predicate":"biolink:has_phenotype","negated":true},
            {"subject":"MONDO:outside","object":"HP:0000256","category":"biolink:DiseaseToPhenotypicFeatureAssociation","predicate":"biolink:has_phenotype"}
        ]
    })).unwrap();
    let support = MonarchClient::map_direct_support(response, &subjects, &objects).unwrap();
    assert_eq!(
        support.status("MONDO:0000001", "HP:0000256"),
        PhenotypeDirectSupportStatus::Indeterminate
    );
}
