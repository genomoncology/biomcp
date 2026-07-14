use super::super::*;
use crate::error::BioMcpError;
use reqwest::header::{CONTENT_TYPE, HeaderValue, RETRY_AFTER};

const RECORD: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/sources/orcid/record.json"
));
const WORKS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/sources/orcid/works.json"
));
const ORCID: &str = "0000-0002-7433-2740";

fn headers(content_type: Option<&'static str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(content_type) = content_type {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    }
    headers
}

fn url(orcid: &str, operation: &str) -> String {
    format!("https://pub.orcid.org/v3.0/{orcid}/{operation}")
}

#[test]
fn maps_public_record_professional_fields_provenance_and_partial_dates() {
    let outcome = decode_record(
        ORCID,
        &url(ORCID, "record"),
        StatusCode::OK,
        &headers(Some("Application/Vnd.Orcid+Json; charset=UTF-8")),
        RECORD,
    )
    .unwrap();
    let OrcidFetchOutcome::Available {
        requested_orcid,
        canonical_orcid,
        data,
    } = outcome
    else {
        panic!("expected available record")
    };
    assert_eq!(requested_orcid, ORCID);
    assert_eq!(canonical_orcid, ORCID);
    assert_eq!(data.modified_date, Some(1_720_915_200_000));
    assert_eq!(data.names.len(), 1);
    let name = &data.names[0];
    assert_eq!(name.given_names.as_deref(), Some("Atul Janardhan"));
    assert_eq!(name.family_name.as_deref(), Some("Butte"));
    assert_eq!(name.visibility, "PUBLIC");
    let source = name.source.as_ref().unwrap();
    assert_eq!(source.source_orcid.as_deref(), Some(ORCID));
    assert_eq!(source.source_name.as_deref(), Some("Atul J. Butte"));
    assert_eq!(
        source.assertion_origin_orcid.as_deref(),
        Some("0000-0001-5109-3700")
    );
    assert_eq!(source.assertion_origin_name.as_deref(), Some("UCSF"));

    assert_eq!(data.employments.len(), 1);
    let employment = &data.employments[0];
    assert_eq!(employment.put_code, Some(1_576_165));
    assert_eq!(employment.role_title.as_deref(), Some("Professor"));
    assert_eq!(
        employment.start_date.as_ref().unwrap().year.as_deref(),
        Some("2015")
    );
    assert_eq!(employment.start_date.as_ref().unwrap().day, None);
    assert_eq!(employment.end_date, None);
    assert_eq!(
        employment.organization.name,
        "University of California San Francisco"
    );
    assert_eq!(
        employment.organization.city.as_deref(),
        Some("San Francisco")
    );
    assert_eq!(
        employment.organization.disambiguation_source.as_deref(),
        Some("ROR")
    );
    assert_eq!(employment.visibility, "PUBLIC");
    let employment_source = employment.source.as_ref().unwrap();
    assert_eq!(employment_source.source_orcid.as_deref(), Some(ORCID));
    assert_eq!(
        employment_source.assertion_origin_orcid.as_deref(),
        Some("0000-0001-5109-3700")
    );
    assert_eq!(
        employment_source.assertion_origin_name.as_deref(),
        Some("UCSF")
    );
    assert!(employment.created_date.is_some());
    assert!(employment.modified_date.is_some());
}

#[test]
fn mapped_record_serialization_excludes_private_profile_and_demographic_fields() {
    let OrcidFetchOutcome::Available { data, .. } = decode_record(
        ORCID,
        &url(ORCID, "record"),
        StatusCode::OK,
        &headers(Some(ORCID_MEDIA_TYPE)),
        RECORD,
    )
    .unwrap() else {
        panic!("expected available")
    };
    let json = serde_json::to_string(&data).unwrap().to_ascii_lowercase();
    for excluded in [
        "email",
        "researcher",
        "homepage",
        "biography",
        "keyword",
        "addresses",
        "gender",
        "ethnicity",
        "demographic",
        "inferred",
        "excluded@example.org",
    ] {
        assert!(
            !json.contains(excluded),
            "mapped record leaked {excluded}: {json}"
        );
    }
    assert!(!json.contains("private employer"));
}

#[test]
fn non_public_name_is_filtered_instead_of_assumed_public() {
    let mut value: serde_json::Value = serde_json::from_slice(RECORD).unwrap();
    value["person"]["name"]["visibility"] = serde_json::json!("LIMITED");
    let body = serde_json::to_vec(&value).unwrap();
    let OrcidFetchOutcome::Available { data, .. } = decode_record(
        ORCID,
        &url(ORCID, "record"),
        StatusCode::OK,
        &headers(Some(ORCID_MEDIA_TYPE)),
        &body,
    )
    .unwrap() else {
        panic!("expected available")
    };
    assert!(data.names.is_empty());
    assert_eq!(data.employments.len(), 1);
}

#[test]
fn works_preserve_group_ids_multiple_public_assertions_and_no_continuation() {
    let OrcidFetchOutcome::Available { data, .. } = decode_works(
        ORCID,
        &url(ORCID, "works"),
        StatusCode::OK,
        &headers(Some("application/vnd.orcid+json;charset=UTF-8")),
        WORKS,
    )
    .unwrap() else {
        panic!("expected available works")
    };
    assert_eq!(data.groups.len(), 1);
    let group = &data.groups[0];
    assert_eq!(group.external_ids.len(), 1);
    assert_eq!(
        group.external_ids[0].external_id_value.as_deref(),
        Some("10.1000/example")
    );
    assert_eq!(
        group.external_ids[0].external_id_type.as_deref(),
        Some("doi")
    );
    assert_eq!(
        group.external_ids[0].external_id_relationship.as_deref(),
        Some("SELF")
    );
    assert_eq!(
        group.external_ids[0].normalized_value.as_deref(),
        Some("10.1000/example")
    );
    assert_eq!(
        group.external_ids[0].normalized_url.as_deref(),
        Some("https://doi.org/10.1000/example")
    );
    assert_eq!(group.summaries.len(), 2);
    assert_eq!(group.summaries[0].put_code, Some(11));
    assert_eq!(group.summaries[0].visibility, "PUBLIC");
    assert_eq!(group.summaries[0].created_date, Some(1_600_000_000_000));
    assert_eq!(group.summaries[0].modified_date, Some(1_700_000_000_000));
    assert_eq!(group.summaries[1].put_code, Some(12));
    assert_eq!(
        group.summaries[0].external_ids[0]
            .external_id_type
            .as_deref(),
        Some("pmid")
    );
    assert_eq!(
        group.summaries[0]
            .source
            .as_ref()
            .unwrap()
            .assertion_origin_name
            .as_deref(),
        Some("Crossref")
    );
    assert_eq!(data.continuation, None);
    let serialized = serde_json::to_string(&data).unwrap().to_ascii_lowercase();
    assert!(!serialized.contains("continuation"));
    assert!(!serialized.contains("private assertion"));
    assert!(!serialized.contains("private-only assertion"));
    assert!(!serialized.contains("private-group-id"));
}

#[test]
fn statuses_are_classified_before_success_media_validation() {
    let empty = HeaderMap::new();
    assert!(matches!(
        decode_record(
            ORCID,
            &url(ORCID, "record"),
            StatusCode::NOT_FOUND,
            &empty,
            b"not json"
        )
        .unwrap(),
        OrcidFetchOutcome::NotFound { .. }
    ));
    assert!(matches!(
        decode_works(
            ORCID,
            &url(ORCID, "works"),
            StatusCode::SERVICE_UNAVAILABLE,
            &empty,
            b"not json"
        )
        .unwrap(),
        OrcidFetchOutcome::Unavailable {
            reason: OrcidUnavailableReason::ServerStatus { status: 503 },
            ..
        }
    ));
    assert!(matches!(
        decode_record(
            ORCID,
            &url(ORCID, "record"),
            StatusCode::INTERNAL_SERVER_ERROR,
            &empty,
            b"not json"
        )
        .unwrap(),
        OrcidFetchOutcome::Unavailable {
            reason: OrcidUnavailableReason::ServerStatus { status: 500 },
            ..
        }
    ));

    let mut rate_headers = HeaderMap::new();
    rate_headers.insert(
        RETRY_AFTER,
        HeaderValue::from_str(&"x".repeat(200)).unwrap(),
    );
    let OrcidFetchOutcome::RateLimited { retry_after, .. } = decode_works(
        ORCID,
        &url(ORCID, "works"),
        StatusCode::TOO_MANY_REQUESTS,
        &rate_headers,
        b"body must not leak",
    )
    .unwrap() else {
        panic!("expected rate limited")
    };
    assert_eq!(retry_after.unwrap().len(), RETRY_AFTER_MAX_CHARS);
}

#[test]
fn malformed_wrong_media_and_inconsistent_identity_are_distinct_errors() {
    for content_type in [None, Some("application/json"), Some("text/plain")] {
        let error = decode_record(
            ORCID,
            &url(ORCID, "record"),
            StatusCode::OK,
            &headers(content_type),
            RECORD,
        )
        .unwrap_err();
        assert!(matches!(error, BioMcpError::Api { .. }));
    }

    let forbidden = decode_works(
        ORCID,
        &url(ORCID, "works"),
        StatusCode::FORBIDDEN,
        &headers(Some(ORCID_MEDIA_TYPE)),
        b"{}",
    )
    .unwrap_err();
    assert!(matches!(forbidden, BioMcpError::Api { .. }));
    assert!(forbidden.to_string().contains("403"));

    let malformed_json = decode_works(
        ORCID,
        &url(ORCID, "works"),
        StatusCode::OK,
        &headers(Some(ORCID_MEDIA_TYPE)),
        b"{not-json",
    )
    .unwrap_err();
    assert!(matches!(malformed_json, BioMcpError::ApiJson { .. }));

    let mismatch = decode_record(
        "0000-0002-1825-0097",
        &url("0000-0002-1825-0097", "record"),
        StatusCode::OK,
        &headers(Some(ORCID_MEDIA_TYPE)),
        RECORD,
    )
    .unwrap_err();
    assert!(mismatch.to_string().contains("disagree"));

    let wrong_base_path = decode_record_with_base(
        ORCID,
        &format!("https://pub.orcid.org/other/{ORCID}/record"),
        ORCID_BASE,
        StatusCode::OK,
        &headers(Some(ORCID_MEDIA_TYPE)),
        RECORD,
    )
    .unwrap_err();
    assert!(wrong_base_path.to_string().contains("base and operation"));
}

#[tokio::test]
async fn client_maps_typed_body_overflow_to_unavailable() {
    let response: reqwest::Response = http::Response::builder()
        .status(StatusCode::OK)
        .body(reqwest::Body::from(vec![
            0_u8;
            crate::sources::DEFAULT_MAX_BODY_BYTES
                + 1
        ]))
        .unwrap()
        .into();
    let client = OrcidClient {
        client: crate::sources::test_client().unwrap(),
        base: Cow::Borrowed(ORCID_BASE),
    };

    let outcome = client.decode_works_response(ORCID, response).await.unwrap();
    assert!(matches!(
        outcome,
        OrcidFetchOutcome::Unavailable {
            reason: OrcidUnavailableReason::BodyLimit {
                max_bytes: crate::sources::DEFAULT_MAX_BODY_BYTES
            },
            ..
        }
    ));
}
