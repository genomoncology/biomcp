use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;

#[test]
fn exact_search_no_records_404_requires_the_provider_shape() {
    assert!(is_no_records_404(
        br#"{"status":{"code":404,"msg":"No records were found for given query"}}"#
    ));
    assert!(!is_no_records_404(
        br#"{"status":{"code":404,"msg":"not found"}}"#
    ));
    assert!(!is_no_records_404(
        br#"{"status":{"code":200,"msg":"No records were found for given query"}}"#
    ));
}

#[test]
fn envelope_requires_the_contract_keys_and_success_code() {
    let valid = br#"{"status":{"code":200},"metadata":{},"data":[]}"#;
    assert!(decode_envelope(StatusCode::OK, valid).is_ok());

    let missing_data = br#"{"status":{"code":200},"metadata":{}}"#;
    assert!(matches!(
        decode_envelope(StatusCode::OK, missing_data),
        Err(BioMcpError::Api { .. })
    ));
    assert!(matches!(
        decode_envelope(StatusCode::NOT_FOUND, valid),
        Err(BioMcpError::Api { .. })
    ));
}
