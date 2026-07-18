//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
//! the response types, plus the pure post-processing helper `select_get_hit_value` and
//! the `de_vec_or_single` / `FloatOrVec` shapes. No network, no server.

use crate::error::BioMcpError;
use crate::sources::decode_json;
use crate::sources::myvariant::{
    FloatOrVec, MyVariantClient, MyVariantClinVar, MyVariantHit, MyVariantSearchResponse,
};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde_json::json;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/myvariant/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_search_response_total_and_hits_from_real_fixture() {
    let resp: MyVariantSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("search_braf.json"),
        true,
    )
    .unwrap();
    assert_eq!(resp.total, Some(1));
    assert!(!resp.hits.is_empty());
    assert!(resp.hits[0].id.starts_with("chr7"));
    assert_eq!(
        resp.hits[0]
            .dbnsfp
            .as_ref()
            .and_then(|d| d.genename.first()),
        Some("BRAF")
    );
}

#[test]
fn parses_get_hit_nested_fields_from_real_fixture() {
    let hit: MyVariantHit = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("get_braf_v600e.json"),
        true,
    )
    .unwrap();

    assert_eq!(hit.id, "chr7:g.140453136A>T");
    assert_eq!(hit.cadd.as_ref().and_then(|c| c.phred), Some(32.0));
    assert_eq!(
        hit.dbsnp.as_ref().and_then(|d| d.rsid.clone()),
        Some("rs113488022".into())
    );
    assert_eq!(
        hit.dbnsfp
            .as_ref()
            .and_then(|d| d.revel.as_ref())
            .and_then(|r| r.score.as_ref())
            .and_then(FloatOrVec::first),
        Some(0.931)
    );
    assert_eq!(
        hit.dbnsfp.as_ref().and_then(|d| d.genename.first()),
        Some("BRAF")
    );
    let bayesdel = hit.dbnsfp.as_ref().and_then(|d| d.bayesdel.as_ref());
    assert_eq!(
        bayesdel
            .and_then(|b| b.add_af.as_ref())
            .and_then(|v| v.score.as_ref())
            .and_then(FloatOrVec::first),
        Some(0.399079)
    );
    assert_eq!(
        bayesdel
            .and_then(|b| b.no_af.as_ref())
            .and_then(|v| v.score.as_ref())
            .and_then(FloatOrVec::first),
        Some(0.335473)
    );
    assert_eq!(hit.cosmic.as_ref().and_then(|c| c.mut_freq), Some(2.83));
    assert!(hit.exac.as_ref().and_then(|e| e.af).is_some());
    assert_eq!(hit.clinvar.as_ref().and_then(|c| c.variant_id), Some(13961));
    assert!(
        hit.clinvar
            .as_ref()
            .map(|c| !c.rcv.is_empty())
            .unwrap_or(false)
    );
    assert!(
        hit.gnomad_exome
            .as_ref()
            .and_then(|g| g.af.as_ref())
            .and_then(|a| a.af)
            .is_some()
    );
    assert!(hit.civic.is_some());
    assert!(hit.cgi.is_some());
}

#[test]
fn select_get_hit_value_passes_object_through() {
    let value = json!({"_id": "chr1:g.1A>T"});
    let out = MyVariantClient::select_get_hit_value(value.clone(), "chr1:g.1A>T").unwrap();
    assert_eq!(out, value);
}

#[test]
fn select_get_hit_value_takes_first_array_element() {
    let value = json!([{"_id": "first"}, {"_id": "second"}]);
    let out = MyVariantClient::select_get_hit_value(value, "x").unwrap();
    assert_eq!(out.get("_id").and_then(|v| v.as_str()), Some("first"));
}

#[test]
fn select_get_hit_value_empty_array_is_not_found() {
    let err = MyVariantClient::select_get_hit_value(json!([]), "rs999").unwrap_err();
    match err {
        BioMcpError::NotFound { entity, id, .. } => {
            assert_eq!(entity, "variant");
            assert_eq!(id, "rs999");
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn select_get_hit_value_scalar_is_api_error() {
    let err = MyVariantClient::select_get_hit_value(json!("nope"), "x").unwrap_err();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("Unexpected response type"));
}

#[test]
fn clinvar_rcv_deserializes_single_object() {
    let clinvar: MyVariantClinVar = serde_json::from_value(json!({
        "variant_id": 123,
        "rcv": {
            "clinical_significance": "Pathogenic",
            "review_status": "criteria provided",
            "conditions": "Lung carcinoma"
        }
    }))
    .expect("single-object RCV should deserialize");
    assert_eq!(clinvar.variant_id, Some(123));
    assert_eq!(clinvar.rcv.len(), 1);
    assert_eq!(
        clinvar.rcv[0].clinical_significance.as_deref(),
        Some("Pathogenic")
    );
}

#[test]
fn clinvar_rcv_deserializes_array() {
    let clinvar: MyVariantClinVar = serde_json::from_value(json!({
        "variant_id": 456,
        "rcv": [
            { "clinical_significance": "Pathogenic" },
            { "clinical_significance": "Likely pathogenic" }
        ]
    }))
    .expect("array RCV should deserialize");
    assert_eq!(clinvar.variant_id, Some(456));
    assert_eq!(clinvar.rcv.len(), 2);
}

#[test]
fn clinvar_rcv_defaults_to_empty_when_missing() {
    let clinvar: MyVariantClinVar =
        serde_json::from_value(json!({ "variant_id": 789 })).expect("missing rcv ok");
    assert_eq!(clinvar.variant_id, Some(789));
    assert!(clinvar.rcv.is_empty());
}

#[test]
fn float_or_vec_first_returns_single_or_head_of_list() {
    let single: FloatOrVec = serde_json::from_value(json!(0.5)).unwrap();
    assert_eq!(single.first(), Some(0.5));
    let multi: FloatOrVec = serde_json::from_value(json!([1.0, 2.0])).unwrap();
    assert_eq!(multi.first(), Some(1.0));
    let empty: FloatOrVec = serde_json::from_value(json!([])).unwrap();
    assert_eq!(empty.first(), None);
}

#[test]
fn gnomad_nested_fields_deserialize() {
    let hit: MyVariantHit = serde_json::from_value(json!({
        "_id": "chr1:g.1A>T",
        "dbnsfp": {"genename": "TP53"},
        "gnomad": {
            "exomes": { "af": { "af": 0.001 } },
            "genomes": { "af": { "af": 0.002 } }
        }
    }))
    .expect("gnomad nested object should deserialize");
    assert_eq!(
        hit.gnomad
            .as_ref()
            .and_then(|g| g.exomes.as_ref())
            .and_then(|e| e.af.as_ref())
            .and_then(|a| a.af),
        Some(0.001)
    );
    assert_eq!(
        hit.gnomad
            .as_ref()
            .and_then(|g| g.genomes.as_ref())
            .and_then(|e| e.af.as_ref())
            .and_then(|a| a.af),
        Some(0.002)
    );
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<MyVariantSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("MyVariant.info"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_json_rejects_html_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<MyVariantSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::MYVARIANT),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("MyVariant.info"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
