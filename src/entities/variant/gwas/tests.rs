//! Sidecar tests for variant GWAS helpers.

use super::super::VariantGwasAssociation;
use super::*;

#[tokio::test]
async fn search_gwas_page_rejects_invalid_probability_before_client_construction() {
    for p_value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -0.01, 1.01] {
        let filters = super::super::GwasSearchFilters {
            p_value: Some(p_value),
            ..Default::default()
        };
        let err = search_gwas_page(&filters, 1, 0)
            .await
            .expect_err("invalid p-value should fail at the entity boundary");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--p-value"));
    }
}

#[test]
fn collect_supporting_pmids_dedupes_case_insensitively() {
    let rows = vec![
        VariantGwasAssociation {
            rsid: "rs1".to_string(),
            trait_name: None,
            p_value: None,
            effect_size: None,
            effect_type: None,
            confidence_interval: None,
            risk_allele_frequency: None,
            risk_allele: None,
            mapped_genes: Vec::new(),
            study_accession: None,
            pmid: Some("12345".to_string()),
            author: None,
            sample_description: None,
        },
        VariantGwasAssociation {
            rsid: "rs1".to_string(),
            trait_name: None,
            p_value: None,
            effect_size: None,
            effect_type: None,
            confidence_interval: None,
            risk_allele_frequency: None,
            risk_allele: None,
            mapped_genes: Vec::new(),
            study_accession: None,
            pmid: Some("12345".to_string()),
            author: None,
            sample_description: None,
        },
        VariantGwasAssociation {
            rsid: "rs1".to_string(),
            trait_name: None,
            p_value: None,
            effect_size: None,
            effect_type: None,
            confidence_interval: None,
            risk_allele_frequency: None,
            risk_allele: None,
            mapped_genes: Vec::new(),
            study_accession: None,
            pmid: Some("PMID-ABC".to_string()),
            author: None,
            sample_description: None,
        },
        VariantGwasAssociation {
            rsid: "rs1".to_string(),
            trait_name: None,
            p_value: None,
            effect_size: None,
            effect_type: None,
            confidence_interval: None,
            risk_allele_frequency: None,
            risk_allele: None,
            mapped_genes: Vec::new(),
            study_accession: None,
            pmid: Some("pmid-abc".to_string()),
            author: None,
            sample_description: None,
        },
    ];

    assert_eq!(
        collect_supporting_pmids(&rows),
        vec!["12345".to_string(), "PMID-ABC".to_string()]
    );
}
