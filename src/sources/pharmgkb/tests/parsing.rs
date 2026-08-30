//! Tier 3 - response parsing. Pure: feeds committed fixture bytes to PharmGKB
//! decoders and mappers. No network, no server.

use super::super::*;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pharmgkb/",
            $name
        ))
    };
}

#[test]
fn annotation_response_maps_ids_titles_levels_and_urls() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: PharmGkbDataResponse = PharmGkbClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("warfarin_annotations.json"),
    )
    .expect("decode")
    .expect("some response");
    let rows = PharmGkbClient::annotations_from_response(resp, "Clinical Annotation", 0, 10);

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].id, "981239556");
    assert_eq!(rows[0].title, "PA166134613");
    assert_eq!(rows[0].kind, "Clinical Annotation");
    assert_eq!(rows[0].level.as_deref(), Some("3"));
    assert_eq!(rows[1].kind, "Guideline Annotation");
    assert!(rows[1].url.as_deref().unwrap().starts_with("https://"));
    assert_eq!(rows[2].kind, "Label Annotation");
}

#[test]
fn annotation_response_applies_the_local_page_after_mapping() {
    let content_type = HeaderValue::from_static("application/json");
    let resp: PharmGkbDataResponse = PharmGkbClient::decode_json_optional(
        StatusCode::OK,
        Some(&content_type),
        fixture!("warfarin_annotations.json"),
    )
    .expect("decode")
    .expect("some response");

    let rows = PharmGkbClient::annotations_from_response(resp, "Clinical Annotation", 1, 1);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, "PA166104949");
    assert_eq!(rows[0].kind, "Guideline Annotation");
}

#[test]
fn dedupe_and_limit_keeps_unique_annotation_rows() {
    let rows = vec![
        PharmGkbAnnotation {
            source: "PharmGKB".to_string(),
            kind: "Clinical Annotation".to_string(),
            id: "1".to_string(),
            title: "Same".to_string(),
            level: None,
            url: None,
        },
        PharmGkbAnnotation {
            source: "PharmGKB".to_string(),
            kind: "Clinical Annotation".to_string(),
            id: "1".to_string(),
            title: "same".to_string(),
            level: None,
            url: None,
        },
        PharmGkbAnnotation {
            source: "PharmGKB".to_string(),
            kind: "Label Annotation".to_string(),
            id: "2".to_string(),
            title: "Other".to_string(),
            level: None,
            url: None,
        },
    ];

    let out = dedupe_and_limit(rows, 5);
    assert_eq!(out.len(), 2);
}

#[test]
fn decode_json_optional_maps_404_http_content_type_and_json_errors() {
    let none: Option<PharmGkbDataResponse> =
        PharmGkbClient::decode_json_optional(StatusCode::NOT_FOUND, None, b"not found").unwrap();
    assert!(none.is_none());

    let content_type = HeaderValue::from_static("application/json");
    let err = PharmGkbClient::decode_json_optional::<PharmGkbDataResponse>(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&content_type),
        b"upstream failed",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
    assert!(format!("{err:?}").contains("500"));

    let html = HeaderValue::from_static("text/html");
    let err = PharmGkbClient::decode_json_optional::<PharmGkbDataResponse>(
        StatusCode::OK,
        Some(&html),
        b"<html></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");

    let err = PharmGkbClient::decode_json_optional::<PharmGkbDataResponse>(
        StatusCode::OK,
        Some(&content_type),
        b"not json",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api_json");
}
