//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response helpers. No network, no server.

use crate::error::BioMcpError;
use crate::sources::decode_json;
use crate::sources::mydisease::{MyDiseaseClient, MyDiseaseHit, MyDiseaseQueryResponse};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/mydisease/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_query_response_from_real_fixture() {
    let resp: MyDiseaseQueryResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYDISEASE),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("query_melanoma.json"),
        false,
    )
    .unwrap();

    assert!(!resp.hits.is_empty());
    assert!(resp.hits[0].id.starts_with("MONDO:") || resp.hits[0].id.starts_with("DOID:"));
}

#[test]
fn parses_get_response_from_real_fixture() {
    let hit = MyDiseaseClient::decode_get_hit(
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_mondo_0005105.json"),
        "MONDO:0005105",
    )
    .unwrap();

    assert_eq!(hit.id, "MONDO:0005105");
    assert!(hit.mondo.is_some());
    assert!(hit.disease_ontology.is_some());
}

#[test]
fn decode_get_hit_maps_not_found_status() {
    let err = MyDiseaseClient::decode_get_hit(
        StatusCode::NOT_FOUND,
        Some(&json_ct()),
        b"{\"error\":\"missing\"}",
        "MONDO:missing",
    )
    .unwrap_err();
    match err {
        BioMcpError::NotFound { entity, id, .. } => {
            assert_eq!(entity, "disease");
            assert_eq!(id, "MONDO:missing");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn hpo_fields_deserialize_from_hit() {
    let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
        "_id": "MONDO:0017309",
        "hpo": {
            "phenotype_related_to_disease": [
                {"hpo_id": "HP:0001653", "evidence": "TAS", "hp_freq": "HP:0040280"}
            ],
            "inheritance": {"hpo_id": "HP:0000006"}
        }
    }))
    .expect("hpo payload should deserialize");

    let hpo = hit.hpo.expect("hpo field should exist");
    assert_eq!(hpo.phenotype_related_to_disease.len(), 1);
    assert_eq!(
        hpo.phenotype_related_to_disease[0].hpo_id.as_deref(),
        Some("HP:0001653")
    );
    assert_eq!(hpo.inheritance.len(), 1);
    assert_eq!(hpo.inheritance[0].hpo_id.as_deref(), Some("HP:0000006"));
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<MyDiseaseQueryResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYDISEASE),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        false,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("MyDisease.info"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}
