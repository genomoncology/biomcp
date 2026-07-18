//! Tier 3 - response parsing and local result shaping. Pure: feeds XML bytes
//! into decode helpers and validates output. No network.

use reqwest::StatusCode;
use reqwest::header::HeaderValue;

use super::super::*;

#[test]
fn parse_gene_hpa_uses_only_top_level_canonical_blocks() {
    let parsed = parse_gene_hpa(super::HPA_XML).expect("parsed");

    assert_eq!(
        parsed,
        GeneHpa {
            tissues: vec![
                HpaTissueExpression {
                    tissue: "Adipose tissue".to_string(),
                    level: "Low".to_string(),
                },
                HpaTissueExpression {
                    tissue: "Liver".to_string(),
                    level: "High".to_string(),
                },
            ],
            subcellular_main_location: vec!["cytosol".to_string(), "vesicles".to_string()],
            subcellular_additional_location: vec!["plasma membrane".to_string()],
            reliability: Some("Supported".to_string()),
            protein_summary: Some("Ubiquitous cytoplasmic expression.".to_string()),
            rna_summary: Some("Low tissue specificity; Detected in all".to_string()),
        }
    );
}

#[test]
fn parse_gene_hpa_handles_protein_atlas_wrapper_element() {
    let wrapped = format!("<proteinAtlas>{}</proteinAtlas>", super::HPA_XML);
    let parsed = parse_gene_hpa(&wrapped).expect("parsed with wrapper");
    assert_eq!(parsed.tissues.len(), 2);
    assert_eq!(parsed.reliability.as_deref(), Some("Supported"));
}

#[test]
fn decode_protein_data_xml_returns_none_for_not_found() {
    let xml = HpaClient::decode_protein_data_xml(StatusCode::NOT_FOUND, None, Vec::new())
        .expect("not found should decode");

    assert_eq!(xml, None);
}

#[test]
fn decode_protein_data_xml_accepts_xml_and_rejects_html() {
    let content_type = HeaderValue::from_static("text/xml");
    let xml = HpaClient::decode_protein_data_xml(
        StatusCode::OK,
        Some(&content_type),
        super::HPA_XML.as_bytes().to_vec(),
    )
    .expect("XML should decode")
    .expect("XML should be present");
    let parsed = parse_gene_hpa(&xml).expect("decoded XML should parse");
    assert_eq!(parsed.tissues.len(), 2);

    let content_type = HeaderValue::from_static("text/html");
    let err = HpaClient::decode_protein_data_xml(
        StatusCode::OK,
        Some(&content_type),
        b"<html>not xml</html>".to_vec(),
    )
    .expect_err("HTML should fail");
    assert!(format!("{err:?}").contains("Unexpected HTML response"));
}
