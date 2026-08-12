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

fn parse_population_fixture(bytes: &[u8]) -> Result<Option<GnomadVariantPopulation>, BioMcpError> {
    let content_type = HeaderValue::from_static("application/json");
    let response: GraphQlResponse<VariantPopulationResponse> =
        GnomadClient::decode_json_response(StatusCode::OK, Some(&content_type), bytes)?;
    GnomadClient::parse_variant_population_response(response)
}

#[test]
fn variant_population_keeps_exome_genome_faf_flags_and_numeric_frequencies_separate() {
    let population = parse_population_fixture(
        br#"{
      "data":{"variant":{"variant_id":"10-112998590-C-T",
        "exome":{"ac":2,"an":10,"homozygote_count":1,"hemizygote_count":0,
          "filters":["AC0"],"faf95":{"popmax":0.25,"popmax_population":"nfe"},
          "populations":[{"id":"nfe","ac":1,"an":4,"homozygote_count":0,"hemizygote_count":0}]},
        "genome":{"ac":3,"an":20,"homozygote_count":0,"hemizygote_count":1,
          "filters":["RF"],"faf95":{"popmax":0.2,"popmax_population":"sas"},
          "populations":[{"id":"sas","ac":3,"an":12,"homozygote_count":0,"hemizygote_count":1}]}
      }}}
    "#,
    )
    .unwrap()
    .expect("population result");

    let exome = population.exome.expect("exome");
    let genome = population.genome.expect("genome");
    assert_eq!(exome.allele_frequency, Some(0.2));
    assert_eq!(exome.populations[0].allele_frequency, Some(0.25));
    assert_eq!(exome.filters, ["AC0"]);
    assert_eq!(genome.allele_frequency, Some(0.15));
    assert_eq!(genome.populations[0].allele_frequency, Some(0.25));
    assert_eq!(genome.filters, ["RF"]);
    assert_eq!(
        genome.faf95.unwrap().popmax_population.as_deref(),
        Some("sas")
    );
}

#[test]
fn variant_population_distinguishes_absence_and_provider_failure() {
    assert!(
        parse_population_fixture(br#"{"data":{"variant":null}}"#)
            .unwrap()
            .is_none()
    );
    assert!(
        parse_population_fixture(
            br#"{"errors":[{"message":"Variant not found"}],"data":{"variant":null}}"#,
        )
        .unwrap()
        .is_none()
    );
    let error = parse_population_fixture(
        br#"{"errors":[{"message":"upstream exploded"}],"data":{"variant":null}}"#,
    )
    .unwrap_err();
    assert!(matches!(error, BioMcpError::Api { .. }));
}

#[test]
fn recorded_v4_population_has_grpmax_and_discordant_exome_genome_filters() {
    let population = parse_population_fixture(fixture!("variant_7_140753125_t_c_v4_20260812.json"))
        .unwrap()
        .expect("recorded population result");
    let exome = population.exome.expect("exome");
    let genome = population.genome.expect("genome");

    assert_eq!(exome.filters, ["AC0"]);
    assert!(genome.filters.is_empty());
    assert_eq!(genome.allele_frequency, Some(4.0 / 152_302.0));
    assert_eq!(genome.populations[0].allele_frequency, Some(4.0 / 41_580.0));
    let faf95 = genome.faf95.expect("genome FAF95");
    assert_eq!(faf95.popmax, Some(0.00003242));
    assert_eq!(faf95.popmax_population.as_deref(), Some("afr"));
}
