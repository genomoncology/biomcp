//! Response parsing and local result shaping tests. Pure: feed bytes and
//! headers into decode helpers. No network.

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};

use super::*;

fn json_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

#[test]
fn decode_summary_response_maps_association_rows() {
    let resp: DisgenetResponse<DisgenetGdaSummaryRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        summary_response_bytes(),
    )
    .unwrap();
    let rows = DisgenetClient::associations_from_response(resp, 10).unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].disease_name, "Breast Carcinoma");
    assert_eq!(rows[0].publication_count, Some(1234));
    assert_eq!(rows[0].clinical_trial_count, Some(4));
    assert_eq!(rows[0].evidence_level.as_deref(), Some("Definitive"));
}

#[test]
fn associations_from_response_applies_limit() {
    let resp: DisgenetResponse<DisgenetGdaSummaryRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        summary_response_bytes(),
    )
    .unwrap();
    let rows = DisgenetClient::associations_from_response(resp, 1).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].gene_symbol, "TP53");
}

#[test]
fn empty_payload_returns_empty_vec() {
    let resp: DisgenetResponse<DisgenetGdaSummaryRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        empty_summary_response_bytes(),
    )
    .unwrap();
    let rows = DisgenetClient::associations_from_response(resp, 10).unwrap();

    assert!(rows.is_empty());
}

#[test]
fn disease_resolution_uses_synonym_match() {
    let resp: DisgenetResponse<DisgenetDiseaseRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        disease_response_bytes(),
    )
    .unwrap();
    let disease_id = DisgenetClient::disease_id_from_response("breast cancer", resp).unwrap();

    assert_eq!(disease_id, "UMLS_C0678222");
}

#[test]
fn disease_resolution_returns_source_unavailable_when_resolution_fails() {
    let resp: DisgenetResponse<DisgenetDiseaseRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        empty_disease_response_bytes(),
    )
    .unwrap();
    let err =
        DisgenetClient::disease_id_from_response("completely unknown disease", resp).unwrap_err();

    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
}

fn assert_rejected_credential_status(status: StatusCode) {
    const PROVIDER_BODY: &[u8] = b"<html><body>rejected-response-must-not-be-echoed</body></html>";

    let err = DisgenetClient::decode_json_response::<serde_json::Value>(
        status,
        &HeaderMap::new(),
        PROVIDER_BODY,
    )
    .unwrap_err();
    let json = crate::render::json::to_error_json(&err).expect("JSON error");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON error");
    let message = value["error"]["message"].as_str().expect("error message");

    assert_eq!(value["error"]["code"], "api_key_rejected");
    assert_eq!(value["_meta"]["not_found"], false);
    assert!(message.contains("DISGENET_API_KEY"));
    assert!(message.contains("rejected"));
    assert!(message.contains("access"));
    assert!(!message.contains("rejected-response-must-not-be-echoed"));
}

#[test]
fn rejected_credential_recovery_points_to_configuration() {
    let error = BioMcpError::ApiKeyRejected {
        api: "disgenet".into(),
        env_var: "DISGENET_API_KEY".into(),
        docs_url: "https://www.disgenet.com/".into(),
    };
    let context = DisgenetClient::error_context(&error);
    let projection = error.with_source_context(context).public_projection();

    assert_eq!(projection.source, Some("DisGeNET"));
    assert_eq!(
        projection.recovery,
        Some(crate::error::RecoveryAction::ReviewSourceConfiguration.message())
    );
}

#[test]
fn unauthorized_response_reports_rejected_credential() {
    assert_rejected_credential_status(StatusCode::UNAUTHORIZED);
}

#[test]
fn forbidden_response_reports_rejected_credential() {
    assert_rejected_credential_status(StatusCode::FORBIDDEN);
}

#[test]
fn rate_limit_error_includes_retry_after_seconds() {
    let mut headers = json_headers();
    headers.insert(
        HeaderName::from_static("x-rate-limit-retry-after-seconds"),
        HeaderValue::from_static("85564"),
    );

    let err = DisgenetClient::decode_json_response::<serde_json::Value>(
        StatusCode::TOO_MANY_REQUESTS,
        &headers,
        br#"{"message": "Too many requests"}"#,
    )
    .unwrap_err();
    let message = format!("{err:?}");

    assert!(message.contains("85564"));
    assert!(message.contains("Too many requests"));
}

#[test]
fn http_500_returns_api_error() {
    let err = DisgenetClient::decode_json_response::<serde_json::Value>(
        StatusCode::INTERNAL_SERVER_ERROR,
        &json_headers(),
        br#"{"message": "upstream failure"}"#,
    )
    .unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
}

#[test]
fn non_ok_response_status_returns_api_error() {
    let resp: DisgenetResponse<DisgenetGdaSummaryRow> = DisgenetClient::decode_json_response(
        StatusCode::OK,
        &json_headers(),
        br#"{"status":"ERROR","httpStatus":200,"payload":[]}"#,
    )
    .unwrap();
    let err = DisgenetClient::associations_from_response(resp, 10).unwrap_err();

    assert!(matches!(err, BioMcpError::Api { .. }));
}
