//! Sidecar tests for variant GWAS helpers.

use super::super::VariantGwasAssociation;
use super::*;

fn search_row(rsid: &str, p_value: f64) -> VariantGwasAssociation {
    VariantGwasAssociation {
        rsid: rsid.into(),
        trait_name: Some("trait".into()),
        p_value: Some(p_value),
        effect_size: None,
        effect_type: None,
        confidence_interval: None,
        risk_allele_frequency: None,
        risk_allele: None,
        mapped_genes: Vec::new(),
        study_accession: None,
        pmid: None,
        author: None,
        sample_description: None,
    }
}

#[test]
fn combined_gene_and_trait_rows_are_an_intersection_before_p_value_filtering() {
    let gene = vec![search_row("rs1", 0.1), search_row("rs2", 0.001)];
    let trait_rows = vec![search_row("RS1", 0.0001), search_row("rs3", 0.00001)];

    let mut intersection = intersect_gwas_legs(gene, trait_rows, true);
    assert_eq!(
        intersection
            .iter()
            .map(|row| row.rsid.to_ascii_lowercase())
            .collect::<Vec<_>>(),
        vec!["rs1", "rs1"]
    );

    apply_p_value_filter(&mut intersection, Some(0.01));
    assert_eq!(intersection.len(), 1);
    assert_eq!(intersection[0].p_value, Some(0.0001));
}

#[test]
fn disjoint_gene_and_trait_rows_return_no_union_rows() {
    let rows = intersect_gwas_legs(
        vec![search_row("rs1", 0.1)],
        vec![search_row("rs2", 0.1)],
        true,
    );
    assert!(rows.is_empty());
}

#[tokio::test]
async fn search_gwas_page_rejects_invalid_probability_before_client_construction() {
    for (label, p_value) in [
        ("not a number", f64::NAN),
        ("positive infinity", f64::INFINITY),
        ("negative infinity", f64::NEG_INFINITY),
        ("overflow", f64::INFINITY),
        ("zero", 0.0),
        ("negative", -0.01),
        ("greater than one", 1.01),
    ] {
        let filters = super::super::GwasSearchFilters {
            p_value: Some(p_value),
            ..Default::default()
        };
        let err = search_gwas_page(&filters, 1, 0)
            .await
            .expect_err("invalid p-value should fail at the entity boundary");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)), "{label}");
        assert!(err.to_string().contains("--p-value"), "{label}");
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
