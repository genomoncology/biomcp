//! Tier 3 — response parsing. Pure: feeds committed fixture bytes and status values
//! to the VariantValidator decoder. No network, no server.

use super::super::*;
use crate::entities::variant::VariantNormalizationStatus;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/variantvalidator/",
            $name
        ))
    };
}

#[test]
fn normalize_response_extracts_warnings_and_grch38_genomic_description() {
    let content_type = HeaderValue::from_static("application/json");
    let result = decode_normalize_response(
        StatusCode::OK,
        Some(&content_type),
        fixture!("normalize_erbb2.json"),
    );

    assert_eq!(result.status, VariantNormalizationStatus::Success);
    assert_eq!(
        result.transcript_description.as_deref(),
        Some("NM_004448.2:c.829G>T")
    );
    assert!(result.warnings[0].contains("TranscriptVersionWarning"));
    assert!(
        result
            .genomic_descriptions
            .iter()
            .any(|value| value.coordinate == "NC_000017.11:g.39710409G>T")
    );
    assert!(
        result
            .genomic_descriptions
            .iter()
            .all(|value| !value.coordinate.contains("NC_000017.10")),
        "GRCh37 genomic descriptions must not be labeled through the GRCh38 markdown surface"
    );
}

#[test]
fn result_from_value_maps_warning_without_transcript_to_invalid_input() {
    let value = serde_json::json!({
        "flag": "warning",
        "validation_warning_1": {
            "submitted_variant": "NM_000248.3:c.",
            "validation_warnings": ["LovdSyntaxcheckInvalid"]
        }
    });

    let result = result_from_value(&value);
    assert_eq!(result.status, VariantNormalizationStatus::InvalidInput);
    assert_eq!(result.input_description.as_deref(), Some("NM_000248.3:c."));
}

#[test]
fn result_from_value_maps_missing_transcript_to_service_error() {
    let value = serde_json::json!({
        "flag": "empty",
        "result": {
            "submitted_variant": "NM_000248.3:c.135del"
        }
    });

    let result = result_from_value(&value);
    assert_eq!(result.status, VariantNormalizationStatus::ServiceError);
    assert!(result.normalized_description.is_none());
}

#[test]
fn normalize_response_maps_not_found_and_http_errors() {
    let result = decode_normalize_response(StatusCode::NOT_FOUND, None, b"not found");
    assert_eq!(result.status, VariantNormalizationStatus::NotFound);
    assert!(
        result
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("HTTP 404")
    );

    let result =
        decode_normalize_response(StatusCode::INTERNAL_SERVER_ERROR, None, b"upstream failed");
    assert_eq!(result.status, VariantNormalizationStatus::ServiceError);
    assert!(
        result
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("HTTP 500")
    );
}

#[test]
fn normalize_response_maps_html_response_to_service_error() {
    let html = HeaderValue::from_static("text/html");
    let result =
        decode_normalize_response(StatusCode::OK, Some(&html), b"<html>maintenance</html>");

    assert_eq!(result.status, VariantNormalizationStatus::ServiceError);
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| !message.is_empty())
    );
}
