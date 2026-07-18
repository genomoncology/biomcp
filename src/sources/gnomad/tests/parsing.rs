//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to the decoder
//! and GraphQL response mapper. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gnomad/",
            $name
        ))
    };
}

fn parse_fixture(name: &[u8]) -> Result<Option<GnomadConstraintData>, BioMcpError> {
    let content_type = HeaderValue::from_static("application/json");
    let response: GraphQlResponse<GeneConstraintResponse> =
        GnomadClient::decode_json_response(StatusCode::OK, Some(&content_type), name)?;
    GnomadClient::parse_gene_constraint_response(response)
}

#[test]
fn gene_constraint_maps_metrics_and_transcript() {
    let constraint = parse_fixture(fixture!("constraint_tp53.json"))
        .unwrap()
        .expect("gene result");

    assert_eq!(constraint.transcript.as_deref(), Some("ENST00000269305"));
    assert_eq!(constraint.pli, Some(0.9979));
    assert_eq!(constraint.loeuf, Some(0.449));
    assert_eq!(constraint.mis_z, Some(1.1539));
    assert_eq!(constraint.syn_z, Some(0.9583));
}

#[test]
fn gene_constraint_returns_some_with_transcript_when_constraint_is_null() {
    let constraint = parse_fixture(fixture!("constraint_ddx3x_null.json"))
        .unwrap()
        .expect("gene result");

    assert_eq!(constraint.transcript.as_deref(), Some("ENST00000644876"));
    assert_eq!(constraint.pli, None);
    assert_eq!(constraint.loeuf, None);
    assert_eq!(constraint.mis_z, None);
    assert_eq!(constraint.syn_z, None);
}

#[test]
fn gene_constraint_returns_none_for_gene_not_found() {
    let constraint =
        parse_fixture(fixture!("constraint_not_found.json")).expect("not found should degrade");

    assert!(constraint.is_none());
}

#[test]
fn gene_constraint_propagates_non_not_found_graphql_errors() {
    let err = parse_fixture(fixture!("constraint_graphql_error.json"))
        .expect_err("non-not-found graphql errors should surface");

    assert!(matches!(err, BioMcpError::Api { .. }));
    assert!(format!("{err:?}").contains("upstream exploded"));
}

#[test]
fn decode_json_response_maps_http_and_content_type_errors() {
    let content_type = HeaderValue::from_static("application/json");
    let err = GnomadClient::decode_json_response::<GraphQlResponse<GeneConstraintResponse>>(
        StatusCode::INTERNAL_SERVER_ERROR,
        Some(&content_type),
        b"upstream failed",
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("500"), "got: {msg}");
    assert!(msg.contains("upstream failed"), "got: {msg}");

    let html = HeaderValue::from_static("text/html");
    let err = GnomadClient::decode_json_response::<GraphQlResponse<GeneConstraintResponse>>(
        StatusCode::OK,
        Some(&html),
        b"<html></html>",
    )
    .unwrap_err();
    assert_eq!(err.code(), "api");
}
