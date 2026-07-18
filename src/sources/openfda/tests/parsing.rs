//! Tier 3 - response parsing. Pure: feeds committed fixture bytes to OpenFDA
//! decoders and typed result structs. No network, no server.

use super::super::*;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/openfda/",
            $name
        ))
    };
}

#[test]
fn drugsfda_response_decodes_application_rows() {
    let resp: OpenFdaResponse<DrugsFdaResult> =
        OpenFdaClient::decode_json_optional(StatusCode::OK, fixture!("drugsfda_imatinib.json"))
            .expect("decode")
            .expect("some response");

    assert_eq!(resp.meta.results.limit, 1);
    assert_eq!(
        resp.results[0].application_number.as_deref(),
        Some("NDA021588")
    );
    assert_eq!(resp.results[0].sponsor_name.as_deref(), Some("NOVARTIS"));
    assert_eq!(
        resp.results[0].products[0].brand_name.as_deref(),
        Some("GLEEVEC")
    );
}

#[test]
fn device_responses_decode_510k_and_pma_rows() {
    let k510: OpenFdaResponse<Fda510kResult> =
        OpenFdaClient::decode_json_optional(StatusCode::OK, fixture!("device_510k.json"))
            .expect("decode")
            .expect("some response");
    assert_eq!(k510.results[0].k_number.as_deref(), Some("K123456"));
    assert_eq!(
        k510.results[0].device_name.as_deref(),
        Some("FoundationOne CDx")
    );

    let pma: OpenFdaResponse<FdaPmaResult> =
        OpenFdaClient::decode_json_optional(StatusCode::OK, fixture!("device_pma.json"))
            .expect("decode")
            .expect("some response");
    assert_eq!(pma.results[0].pma_number.as_deref(), Some("P000019"));
    assert_eq!(
        pma.results[0].trade_name.as_deref(),
        Some("FoundationOne CDx")
    );
}

#[test]
fn faers_and_count_responses_decode() {
    let faers: OpenFdaResponse<FaersEventResult> =
        OpenFdaClient::decode_json_optional(StatusCode::OK, fixture!("faers_event.json"))
            .expect("decode")
            .expect("some response");
    assert_eq!(faers.results[0].safetyreportid, "10000001");
    assert_eq!(
        faers.results[0].patient.as_ref().unwrap().reaction[0]
            .reactionmeddrapt
            .as_deref(),
        Some("Nausea")
    );

    let count: OpenFdaCountResponse =
        OpenFdaClient::decode_json_optional(StatusCode::OK, fixture!("faers_count.json"))
            .expect("decode")
            .expect("some response");
    assert_eq!(count.results[0].term, "Nausea");
    assert_eq!(count.results[0].count, 12);
}

#[test]
fn decode_json_optional_maps_404_http_and_json_errors() {
    let none: Option<OpenFdaResponse<DrugsFdaResult>> =
        OpenFdaClient::decode_json_optional(StatusCode::NOT_FOUND, b"not found").unwrap();
    assert!(none.is_none());

    let err = OpenFdaClient::decode_json_optional::<OpenFdaResponse<DrugsFdaResult>>(
        StatusCode::INTERNAL_SERVER_ERROR,
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");

    let err = OpenFdaClient::decode_json_optional::<OpenFdaResponse<DrugsFdaResult>>(
        StatusCode::OK,
        b"not json",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api_json");
}

#[test]
fn count_value_detects_keyword_field_retry() {
    let value = serde_json::json!({
        "error": {
            "code": "SERVER_ERROR",
            "details": "Field is not a keyword field"
        }
    });

    assert!(OpenFdaClient::count_value_requests_exact_retry(
        &value,
        "patient.reaction.reactionmeddrapt"
    ));
    assert!(!OpenFdaClient::count_value_requests_exact_retry(
        &value,
        "patient.reaction.reactionmeddrapt.exact"
    ));
}
