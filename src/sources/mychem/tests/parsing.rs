//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and the response types. No network, no server.

use crate::error::BioMcpError;
use crate::sources::decode_json;
use crate::sources::mychem::{
    MyChemChebiField, MyChemNdcField, MyChemPharmClass, MyChemQueryResponse,
};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

const MYCHEM_API: &str = "mychem.info";

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/mychem/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

#[test]
fn parses_query_response_from_real_fixture() {
    let resp: MyChemQueryResponse = decode_json(
        MYCHEM_API,
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("query_imatinib.json"),
        true,
    )
    .unwrap();

    assert!(resp.total >= 1);
    let hit = resp.hits.first().expect("hit");
    let ndc = hit.ndc.as_ref().expect("ndc field");
    let MyChemNdcField::One(ndc) = ndc else {
        panic!("expected one ndc entry");
    };
    assert_eq!(ndc.nonproprietaryname.as_deref(), Some("Imatinib Mesylate"));
    assert!(ndc.pharm_classes.iter().any(|class| {
        class
            .as_str()
            .map(|value| value.contains("Kinase Inhibitor"))
            .unwrap_or(false)
    }));
}

#[test]
fn pharm_class_supports_string_and_map() {
    let input = r#"
    {
      "total": 1,
      "hits": [
        {
          "_id": "X",
          "_score": 1.0,
          "ndc": {
            "pharm_classes": [
              "Kinase inhibitor [MoA]",
              { "classname": "Alkylating agent [MoA]" }
            ]
          }
        }
      ]
    }
    "#;
    let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
    let hit = parsed.hits.first().expect("hit");
    let ndc = hit.ndc.as_ref().expect("ndc");
    let MyChemNdcField::One(ndc) = ndc else {
        panic!("expected one ndc entry");
    };
    let classes = ndc
        .pharm_classes
        .iter()
        .filter_map(MyChemPharmClass::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        classes,
        vec!["Kinase inhibitor [MoA]", "Alkylating agent [MoA]"]
    );
}

#[test]
fn chebi_name_round_trips() {
    let input = r#"
    {
      "total": 2,
      "hits": [
        { "_id": "CHEBI:1", "_score": 1.0, "chebi": { "name": "Example inhibitor" } },
        { "_id": "CHEBI:2", "_score": 1.0, "chebi": [{ "name": "Example inhibitor 2" }] }
      ]
    }
    "#;
    let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
    let hit = parsed.hits.first().expect("hit");
    assert_eq!(
        hit.chebi.as_ref().and_then(MyChemChebiField::name),
        Some("Example inhibitor")
    );

    let hit = parsed.hits.get(1).expect("hit");
    assert_eq!(
        hit.chebi.as_ref().and_then(MyChemChebiField::name),
        Some("Example inhibitor 2")
    );
}

#[test]
fn unii_supports_object_and_list() {
    let input = r#"
    {
      "total": 2,
      "hits": [
        { "_id": "X", "_score": 1.0, "unii": { "unii": "ABC", "display_name": "Example" } },
        { "_id": "Y", "_score": 1.0, "unii": [{ "unii": "DEF", "display_name": "Example 2" }] }
      ]
    }
    "#;

    let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
    let hit = parsed.hits.first().expect("hit");
    let unii = hit.unii.as_ref().expect("unii");
    assert_eq!(unii.unii(), Some("ABC"));
    assert_eq!(unii.display_name(), Some("Example"));

    let hit = parsed.hits.get(1).expect("hit");
    let unii = hit.unii.as_ref().expect("unii");
    assert_eq!(unii.unii(), Some("DEF"));
    assert_eq!(unii.display_name(), Some("Example 2"));
}

#[test]
fn atc_classifications_support_string_and_list() {
    let input = r#"
    {
      "total": 2,
      "hits": [
        { "_id": "X", "_score": 1.0, "chembl": { "atc_classifications": "L01XX08" } },
        { "_id": "Y", "_score": 1.0, "chembl": { "atc_classifications": ["L01BB04", "L04AA40"] } }
      ]
    }
    "#;

    let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
    let first = parsed.hits.first().expect("first");
    assert_eq!(
        first
            .chembl
            .as_ref()
            .expect("chembl")
            .atc_classifications
            .clone()
            .into_vec(),
        vec!["L01XX08"]
    );

    let second = parsed.hits.get(1).expect("second");
    assert_eq!(
        second
            .chembl
            .as_ref()
            .expect("chembl")
            .atc_classifications
            .clone()
            .into_vec(),
        vec!["L01BB04", "L04AA40"]
    );
}

#[test]
fn drugcentral_approval_supports_object_and_list() {
    let input = r#"
    {
      "total": 2,
      "hits": [
        { "_id": "A", "_score": 1.0, "drugcentral": { "approval": { "agency": "FDA", "date": "2011-08-17" } } },
        { "_id": "B", "_score": 1.0, "drugcentral": { "approval": [{ "agency": "FDA" }, { "agency": "EMA" }] } }
      ]
    }
    "#;

    let parsed: MyChemQueryResponse = serde_json::from_str(input).expect("parse");
    let first = parsed
        .hits
        .first()
        .and_then(|hit| hit.drugcentral.as_ref())
        .map(|dc| dc.approval.len());
    assert_eq!(first, Some(1));

    let second = parsed
        .hits
        .get(1)
        .and_then(|hit| hit.drugcentral.as_ref())
        .map(|dc| dc.approval.len());
    assert_eq!(second, Some(2));
}

#[test]
fn drugbank_interactions_support_object_and_list() {
    let parsed: MyChemQueryResponse = serde_json::from_value(serde_json::json!({
        "total": 2,
        "hits": [
            {"_id": "A", "_score": 1.0, "drugbank": {"id": "DB1", "drug_interactions": {"name": "Aspirin"}}},
            {"_id": "B", "_score": 1.0, "drugbank": {"id": "DB2", "drug_interactions": [{"name": "Clopidogrel"}]}}
        ]
    }))
    .expect("parse");

    let first = parsed
        .hits
        .first()
        .and_then(|h| h.drugbank.as_ref())
        .map(|d| d.drug_interactions.len());
    let second = parsed
        .hits
        .get(1)
        .and_then(|h| h.drugbank.as_ref())
        .map(|d| d.drug_interactions.len());
    assert_eq!(first, Some(1));
    assert_eq!(second, Some(1));
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<MyChemQueryResponse>(
        MYCHEM_API,
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(msg.contains("mychem.info"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<MyChemQueryResponse>(
        MYCHEM_API,
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("mychem.info"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
