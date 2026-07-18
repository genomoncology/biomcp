//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json`
//! and response types. No network, no server.

use super::super::*;
use crate::sources::decode_json;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/cbioportal/",
            $name
        ))
    };
}

#[test]
fn parses_gene_resolution_fixture() {
    let genes: Vec<CBioGene> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
        StatusCode::OK,
        None,
        fixture!("genes_braf.json"),
        false,
    )
    .unwrap();

    assert_eq!(genes.len(), 1);
    assert_eq!(genes[0].entrez_gene_id, 673);
}

#[test]
fn parses_study_mutation_and_clinical_fixtures() {
    let study: CBioStudy = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
        StatusCode::OK,
        None,
        fixture!("study.json"),
        false,
    )
    .unwrap();
    assert_eq!(study.sequenced_sample_count, Some(100));

    let mutations: Vec<CBioMutation> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
        StatusCode::OK,
        None,
        fixture!("mutations.json"),
        false,
    )
    .unwrap();
    assert_eq!(mutations.len(), 3);
    assert_eq!(mutations[0].sample_id.as_deref(), Some("SAMPLE-1"));
    assert!(mutations[1].sample_id.as_deref().is_none_or(str::is_empty));

    let clinical: Vec<CBioClinicalData> = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
        StatusCode::OK,
        None,
        fixture!("clinical_data.json"),
        false,
    )
    .unwrap();
    assert_eq!(clinical.len(), 2);
    assert_eq!(clinical[0].sample_id.as_deref(), Some("SAMPLE-1"));
    assert_eq!(clinical[0].value.as_deref(), Some("Melanoma"));
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<Vec<CBioGene>>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        false,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert_eq!(err.code(), "api");
    assert!(msg.contains("cBioPortal"), "got: {msg}");
    assert!(!msg.contains("500"), "got: {msg}");
    assert!(!msg.contains("upstream failure"), "got: {msg}");
}
