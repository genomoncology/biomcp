//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;

use super::super::*;

#[test]
fn decode_json_response_maps_term_response() {
    let term: HpoTerm = HpoClient::decode_json_response(
        StatusCode::OK,
        br#"{"id":"HP:0001653","name":"Aortic root aneurysm"}"#,
    )
    .unwrap();

    assert_eq!(term.id, "HP:0001653");
    assert_eq!(term.name, "Aortic root aneurysm");
}

#[test]
fn decode_json_response_maps_not_found_and_http_errors() {
    let err = HpoClient::decode_json_response::<HpoTerm>(StatusCode::NOT_FOUND, b"").unwrap_err();
    assert!(matches!(err, BioMcpError::NotFound { .. }));

    let err = HpoClient::decode_json_response::<HpoTerm>(
        StatusCode::BAD_GATEWAY,
        br#"{"error":"upstream"}"#,
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));
}

#[test]
fn decode_search_term_ids_maps_search_results() {
    let response: HpoSearchResponse = serde_json::from_value(serde_json::json!({
        "terms": [
            {"id": "HP:0001250", "name": "Seizure"},
            {"id": "hp_0001263", "name": "Developmental delay"},
            {"id": "NOT_AN_HPO", "name": "Ignore me"},
            {"id": "HP:0001250", "name": "Seizure duplicate"}
        ]
    }))
    .unwrap();

    let ids = HpoClient::decode_search_term_ids(response, 5);

    assert_eq!(
        ids,
        vec!["HP:0001250".to_string(), "HP:0001263".to_string()]
    );
}
