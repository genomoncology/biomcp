//! Sidecar tests for variant GWAS helpers.

use super::super::VariantGwasAssociation;
use super::*;

fn search_row(rsid: &str, p_value: f64) -> VariantGwasAssociation {
    VariantGwasAssociation {
        rsid: rsid.into(),
        trait_name: Some("trait".into()),
        p_value: super::super::GwasPValue::from_numeric(p_value),
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
    assert_eq!(
        intersection[0]
            .p_value
            .as_ref()
            .and_then(|value| value.numeric),
        Some(0.0001)
    );
}

#[test]
fn underflowed_exact_p_values_remain_truthful_and_filterable() {
    let exact = super::super::GwasPValue::from_provider_parts(Some(0.0), Some(3), Some(-1315))
        .expect("exact provider p-value");

    assert_eq!(exact.scientific, "3e-1315");
    assert_eq!(exact.mantissa, Some(3));
    assert_eq!(exact.exponent, Some(-1315));
    assert_eq!(exact.numeric, None);
    assert_eq!(
        serde_json::to_value(&exact).unwrap(),
        serde_json::json!({
            "scientific": "3e-1315",
            "mantissa": 3,
            "exponent": -1315,
            "numeric": null
        })
    );
    assert!(exact.is_at_most(5e-324));

    let larger = super::super::GwasPValue::from_provider_parts(Some(0.0), Some(5), Some(-420))
        .expect("second exact p-value");
    assert!(exact.total_cmp(&larger).is_lt());
}

#[test]
fn malformed_exact_p_value_parts_are_bounded_and_fall_back_truthfully() {
    for exponent in [i32::MIN, -1_000_001, 1, i32::MAX] {
        assert!(
            super::super::GwasPValue::from_provider_parts(None, Some(1), Some(exponent)).is_none(),
            "exponent {exponent} must be rejected"
        );
    }
    assert!(super::super::GwasPValue::from_provider_parts(None, Some(2), Some(0)).is_none());

    let fallback =
        super::super::GwasPValue::from_provider_parts(Some(5e-8), Some(1), Some(i32::MAX))
            .expect("valid numeric fallback");
    assert_eq!(fallback.numeric, Some(5e-8));
    assert_eq!(fallback.mantissa, None);
    assert_eq!(fallback.exponent, None);
    assert_eq!(fallback.scientific, "5e-8");
}

#[test]
fn scientific_comparison_is_safe_for_extreme_in_memory_exponents() {
    let smallest = super::super::GwasPValue {
        scientific: format!("1e{}", i32::MIN),
        mantissa: Some(1),
        exponent: Some(i32::MIN),
        numeric: None,
    };
    let largest = super::super::GwasPValue {
        scientific: format!("1e{}", i32::MAX),
        mantissa: Some(1),
        exponent: Some(i32::MAX),
        numeric: None,
    };
    assert!(smallest.total_cmp(&largest).is_lt());
    assert!(smallest.is_at_most(1.0));
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

/// "retired" travels as a typed error from the source reader
/// through `add_gwas_section` to the rendered card, and the rendered GWAS
/// section offers no retry because no retry reaches a retired endpoint.
/// Forcing the permanent flag off at the degrade site renders the
/// transient wording with a retry command and fails this test.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_retired_endpoint_degrades_the_rendered_gwas_section_with_no_retry() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let retired_body = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/gwas/associations_retired_410_body.txt"
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind retired GWAS fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let Ok(len) = stream.read(&mut request).await else {
                    return;
                };
                let _ = &request[..len];
                let response = format!(
                    "HTTP/1.1 410 Gone\r\nContent-Type: text/plain\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{retired_body}",
                    retired_body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    let mut env = crate::test_support::FixtureEnv::new();
    env.set("BIOMCP_GWAS_BASE", &base);

    let mut variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr1:g.100A>T",
        "gene": "TST",
        "rsid": "rs121913529"
    }))
    .expect("variant fixture");
    super::add_gwas_section(&mut variant, "rs121913529")
        .await
        .expect("the retired endpoint degrades the section, never fails the card");
    drop(env);
    server.abort();

    let rendered = crate::render::markdown::variant_markdown(&variant, &["all".to_string()])
        .expect("rendered variant card");
    assert!(
        rendered.contains("GWAS Catalog by-rsID endpoint retired upstream"),
        "the honest gap note reached the rendered card: {rendered}"
    );
    assert!(
        !rendered.contains("Retry:"),
        "a retired endpoint offers no retry: {rendered}"
    );
    let outcome = variant.section_outcomes.get("gwas").expect("gwas outcome");
    assert!(
        outcome.permanent(),
        "the typed retired error marks the degrade permanent"
    );
}

/// The retirement is a typed fact, not a sentence comparison: the error class carries it, and the reason text lives once.
#[test]
fn the_retired_error_class_is_the_one_fact_of_retirement() {
    let retired = BioMcpError::SourceEndpointRetired {
        source_name: "GWAS Catalog".into(),
        reason: crate::sources::gwas::RETIRED_ENDPOINT_REASON.into(),
        suggestion: String::new(),
    };
    assert!(crate::sources::gwas::is_retired_endpoint_error(&retired));
    let transient = BioMcpError::SourceUnavailable {
        source_name: "GWAS Catalog".into(),
        reason: crate::sources::gwas::RETIRED_ENDPOINT_REASON.into(),
        suggestion: String::new(),
    };
    assert!(
        !crate::sources::gwas::is_retired_endpoint_error(&transient),
        "the same sentence in a transient error is not retirement"
    );
}
