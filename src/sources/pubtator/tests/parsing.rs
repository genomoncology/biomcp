//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response types. No network, no server.

use super::super::*;
use crate::sources::decode_json;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/pubtator/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_real_export_capture_and_retains_disease_normalized_id() {
    let resp: PubTatorExportResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("export_22663011.json"),
        true,
    )
    .unwrap();

    let document = resp.documents.first().expect("captured document");
    assert_eq!(document.pmid, Some(22663011));
    for kind in ["Gene", "Disease", "Chemical", "Species", "Variant"] {
        assert!(
            document
                .passages
                .iter()
                .flat_map(|passage| &passage.annotations)
                .any(|annotation| {
                    annotation
                        .infons
                        .as_ref()
                        .and_then(|infons| infons.kind.as_deref())
                        == Some(kind)
                }),
            "capture is missing {kind} annotations"
        );
    }

    let disease = document
        .passages
        .iter()
        .flat_map(|passage| &passage.annotations)
        .find(|annotation| {
            annotation.text.as_deref() == Some("melanoma")
                && annotation
                    .infons
                    .as_ref()
                    .and_then(|infons| infons.kind.as_deref())
                    == Some("Disease")
        })
        .expect("captured Disease annotation");
    let disease_infons = serde_json::to_value(disease.infons.as_ref().expect("Disease infons"))
        .expect("serialize Disease infons");
    assert_eq!(
        disease_infons
            .get("normalized_id")
            .and_then(serde_json::Value::as_str),
        Some("D008545")
    );

    let gene = document
        .passages
        .iter()
        .flat_map(|passage| &passage.annotations)
        .find(|annotation| {
            annotation.text.as_deref() == Some("BRAF")
                && annotation
                    .infons
                    .as_ref()
                    .and_then(|infons| infons.kind.as_deref())
                    == Some("Gene")
        })
        .expect("captured Gene annotation");
    let gene_infons = serde_json::to_value(gene.infons.as_ref().expect("Gene infons"))
        .expect("serialize Gene infons");
    assert_eq!(
        gene_infons
            .get("normalized_id")
            .and_then(serde_json::Value::as_u64),
        Some(673)
    );
}

#[test]
fn parses_autocomplete_response_fixture() {
    let resp: Vec<PubTatorAutocompleteResult> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("autocomplete_braf.json"),
        true,
    )
    .unwrap();

    assert_eq!(resp.len(), 1);
    assert_eq!(resp[0].id.as_deref(), Some("@GENE_BRAF"));
    assert_eq!(resp[0].biotype.as_deref(), Some("gene"));
    assert_eq!(resp[0].db_id.as_deref(), Some("673"));
    assert_eq!(resp[0].name.as_deref(), Some("BRAF"));
}

#[test]
fn parses_search_response_fixture_and_stringifies_numeric_pmid() {
    let resp: PubTatorSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf.json"),
        true,
    )
    .unwrap();

    assert_eq!(resp.count, Some(1));
    assert_eq!(resp.results.len(), 1);
    assert_eq!(resp.results[0].id.as_deref(), Some("123"));
    assert_eq!(resp.results[0].pmid.as_deref(), Some("123"));
    assert_eq!(
        resp.results[0].title.as_deref(),
        Some("BRAF alterations in melanoma")
    );
}

#[test]
fn search_result_trims_empty_string_pmids_to_none() {
    let result: PubTatorSearchResult = serde_json::from_value(serde_json::json!({
        "_id": "empty-pmid",
        "pmid": "   ",
        "title": "No PMID"
    }))
    .unwrap();

    assert_eq!(result.pmid, None);
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<PubTatorSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("PubTator 3"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failure"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<PubTatorSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::PUBTATOR3),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("PubTator 3"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}

fn admitted_record(id: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "passages": [{
            "infons": {"type": "title"},
            "offset": 0,
            "text": title,
            "sentences": [],
            "annotations": [],
            "relations": []
        }]
    })
}

#[test]
fn pubtator_detail_selects_the_later_admitted_match_and_keeps_original_bytes() {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "PubTator3": [admitted_record("8", "other"), admitted_record("7", "selected")]
    }))
    .unwrap();

    let detail = parse_publication_detail(7, &bytes).expect("bound detail");
    let PubTatorDetail::Adopted(response) = detail.expect("nonempty detail") else {
        panic!("expected adopted response");
    };
    assert_eq!(response.selected_index(), 1);
    assert_eq!(response.response_bytes(), bytes);
    let capture = biodata::Capture::from_bytes("pubtator3", "7", &bytes).unwrap();
    assert_eq!(response.capture().digest(), capture.digest());
    assert_eq!(response.value().title().raw(), Some("selected"));
}

#[test]
fn pubtator_detail_rejects_admitted_identity_and_shape_neighbors_safely() {
    let cases = [
        (
            serde_json::json!({"PubTator3": [admitted_record("7", "one"), admitted_record("7", "two")]}),
            "ambiguous_identity",
        ),
        (
            serde_json::json!({"PubTator3": [{"id": "7", "pmid": 8, "passages": []}]}),
            "conflicting_identity",
        ),
        (
            serde_json::json!({"PubTator3": [admitted_record("8", "wrong")]}),
            "identity_mismatch",
        ),
        (
            serde_json::json!({"PubTator3": [{"id": "0", "passages": []}]}),
            "unusable_identity",
        ),
        (
            serde_json::json!({"PubTator3": [{"id": "8", "passages": [], "neighbor": true}]}),
            "legacy_identity_mismatch",
        ),
        (
            serde_json::json!({"PubTator3": [{
                "id": "7", "pmid": 8, "passages": [], "neighbor": true
            }]}),
            "legacy_conflicting_identity",
        ),
        (
            serde_json::json!({"PubTator3": [
                {"id": "7", "passages": [], "neighbor": true},
                {"id": "7", "passages": [], "neighbor": true}
            ]}),
            "legacy_ambiguous_identity",
        ),
    ];

    for (payload, code) in cases {
        let error = parse_publication_detail(7, &serde_json::to_vec(&payload).unwrap())
            .expect_err("neighbor must reject");
        let debug = format!("{error:?}");
        assert!(debug.contains(code), "{debug}");
        assert!(!debug.contains("wrong"), "{debug}");
        assert!(!debug.contains("neighbor"), "{debug}");
    }
}

#[test]
fn pubtator_detail_uses_legacy_only_for_one_bound_unsupported_shape() {
    let bytes = br#"{"PubTator3":[{"id":"7","pmid":7,"pmcid":"PMC7","passages":[{"infons":{"type":"title","wider":"kept by legacy"},"text":"legacy title"}]}]}"#;

    let detail = parse_publication_detail(7, bytes).expect("legacy detail");
    let PubTatorDetail::Legacy {
        requested_pmid,
        document,
    } = detail.expect("nonempty detail")
    else {
        panic!("expected legacy response");
    };
    assert_eq!(requested_pmid.as_str(), "7");
    assert_eq!(document.id.as_deref(), Some("7"));
    assert_eq!(document.pmid, Some(7));
}

#[test]
fn pubtator_detail_transport_checks_status_and_content_type_before_adapter() {
    let json = HeaderValue::from_static("application/json");
    let html = HeaderValue::from_static("text/html");
    let status_error = validate_detail_transport(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&json),
        br#"{"private":"body"}"#,
    )
    .expect_err("status must reject");
    assert!(format!("{status_error:?}").contains("HTTP 500"));
    assert!(!format!("{status_error:?}").contains("private"));

    let content_error = validate_detail_transport(StatusCode::OK, Some(&html), b"<html>")
        .expect_err("content type must reject");
    assert_eq!(content_error.code(), "api");
}

#[test]
fn pubtator_detail_empty_is_distinct_and_adapter_errors_are_terminal() {
    assert!(
        parse_publication_detail(7, br#"{"PubTator3":[]}"#)
            .unwrap()
            .is_none()
    );
    for (bytes, code) in [
        (
            br#"{"PubTator3":[{"id":"7","id":"7","passages":[]}]}"#.as_slice(),
            "duplicate_member",
        ),
        (
            br#"{"PubTator3":[{"id":"7","passages":"many"}]}"#.as_slice(),
            "legacy_decode",
        ),
    ] {
        let error = parse_publication_detail(7, bytes).expect_err("invalid detail");
        assert!(format!("{error:?}").contains(code));
    }

    let oversized = vec![b' '; biodata::PubTator3Limits::default().max_input_bytes() + 1];
    let error = parse_publication_detail(7, &oversized).expect_err("resource neighbor");
    assert!(format!("{error:?}").contains("response_too_large"));
}
