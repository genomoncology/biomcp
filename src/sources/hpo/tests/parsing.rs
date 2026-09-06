//! Tier 3 - response parsing and local result shaping. Pure: feeds JSON bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::super::*;

#[test]
fn decode_json_response_maps_term_response() {
    let term: HpoTerm = HpoClient::decode_json_response(
        StatusCode::OK,
        Some(&HeaderValue::from_static("application/json")),
        br#"{"id":"HP:0001653","name":"Aortic root aneurysm"}"#,
        true,
    )
    .unwrap();

    assert_eq!(term.id, "HP:0001653");
    assert_eq!(term.name, "Aortic root aneurysm");
}

#[test]
fn decode_json_response_maps_not_found_and_http_errors() {
    let err = HpoClient::decode_json_response::<HpoTerm>(StatusCode::NOT_FOUND, None, b"", true)
        .unwrap_err();
    assert!(matches!(err, BioMcpError::NotFound { .. }));

    let err = HpoClient::decode_json_response::<HpoTerm>(
        StatusCode::BAD_GATEWAY,
        Some(&HeaderValue::from_static("application/json")),
        br#"{"error":"upstream"}"#,
        true,
    )
    .unwrap_err();
    assert!(format!("{err:?}").contains("HTTP 502 Bad Gateway"));
}

#[test]
fn decode_json_response_rejects_non_json_content_type() {
    let legacy: HpoTerm = HpoClient::decode_json_response(
        StatusCode::OK,
        None,
        br#"{"id":"HP:0001653","name":"legacy sniffed JSON"}"#,
        false,
    )
    .expect("non-phenotype HPO callers retain the prior JSON-sniffing behavior");
    assert_eq!(legacy.name, "legacy sniffed JSON");
    let err = HpoClient::decode_json_response::<HpoTerm>(
        StatusCode::OK,
        Some(&HeaderValue::from_static("text/plain")),
        br#"{"id":"HP:0001653","name":"Aortic root aneurysm"}"#,
        true,
    )
    .expect_err("HPO JSON under a non-JSON content type must fail closed");
    assert!(matches!(err, BioMcpError::WithSourceContext { .. }));
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

#[test]
fn search_envelope_requires_an_array_terms_field() {
    for value in [
        serde_json::json!({}),
        serde_json::json!({"terms": null}),
        serde_json::json!({"terms": {"id": "HP:0001250"}}),
    ] {
        assert!(serde_json::from_value::<HpoSearchResponse>(value).is_err());
    }

    let empty: HpoSearchResponse =
        serde_json::from_value(serde_json::json!({"terms": []})).unwrap();
    assert!(empty.terms.is_empty());
}

#[test]
fn decoded_search_rows_preserve_hpo_labels_and_provider_order() {
    let response: HpoSearchResponse = serde_json::from_value(serde_json::json!({
        "terms": [
            {"id": "HP:0000256", "name": "Macrocephaly"},
            {"id": "hp_0001250", "name": "Seizure"}
        ]
    }))
    .unwrap();

    let rows = HpoClient::decode_search_terms(response);
    assert_eq!(rows[0].id, "HP:0000256");
    assert_eq!(rows[0].label, "Macrocephaly");
    assert_eq!(rows[1].id, "HP:0001250");
}

#[tokio::test(start_paused = true)]
async fn phenotype_hpo_operation_uses_only_the_shared_four_physical_attempts() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let attempts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&attempts);
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            observed.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                let mut request = [0_u8; 2048];
                let _ = stream.read(&mut request).await;
                stream
                    .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
                    .await
                    .unwrap();
            });
        }
    });
    let client = HpoClient::new_for_test(base).unwrap();
    client
        .phenotype_term("HP:0001250")
        .await
        .expect_err("four retry-policy attempts must still return the provider error");
    assert_eq!(attempts.load(Ordering::SeqCst), 4);
    server.abort();
}
