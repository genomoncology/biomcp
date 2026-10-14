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

/// Decode one v2 association fixture reply the way the client does: the
/// envelope's embedded association rows and its total element count.
fn fixture_search_page(body: &str) -> crate::sources::gwas::GwasAssociationSearchPage {
    let value: serde_json::Value = serde_json::from_str(body).expect("fixture JSON");
    crate::sources::gwas::GwasAssociationSearchPage {
        associations: serde_json::from_value(value["_embedded"]["associations"].clone())
            .expect("fixture associations"),
        total: value["page"]["totalElements"].as_u64().unwrap_or(0) as usize,
    }
}

/// The rows the pre-change sequential path returned on the same replies:
/// both legs fetched one after another, then the same mapping,
/// intersection, dedupe, p-value filter, and window the page runs.
fn sequential_combined_rows(
    gene_body: &str,
    trait_body: &str,
    limit: usize,
    offset: usize,
) -> Vec<VariantGwasAssociation> {
    let gene_page = fixture_search_page(gene_body);
    let trait_page = fixture_search_page(trait_body);
    let gene_rows: Vec<_> = gene_page
        .associations
        .iter()
        .take(50)
        .filter_map(map_gwas_summary)
        .collect();
    let trait_rows: Vec<_> = trait_page
        .associations
        .iter()
        .take(50)
        .filter_map(map_gwas_summary)
        .collect();
    let mut rows = dedupe_gwas_rows(intersect_gwas_legs(gene_rows, trait_rows, true), 50)
        .expect("dedupe sequential rows");
    apply_p_value_filter(&mut rows, None);
    rows.into_iter().skip(offset).take(limit).collect()
}

/// A combined gene and trait search sends its two GWAS Catalog queries at
/// the same time: a fixture server that holds every association reply for
/// 1.5 s must let the page finish inside 2.5 s — the sequential legs this
/// change replaced needed both holds, about 3 s — while its rows and
/// pagination stay the sequential path's on the same two replies. The
/// gene reply shares one rsID with the trait reply so the intersection
/// and its row order are exercised, not just the clock.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn combined_gene_and_trait_queries_run_at_once_and_keep_the_sequential_rows() {
    use std::time::{Duration, Instant};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    const GENE_REPLY: &str = r#"{
      "_embedded": {
        "associations": [
          {
            "snp_allele": [{ "rs_id": "rs121913529", "effect_allele": "T" }],
            "efo_traits": [{ "efo_trait": "cutaneous melanoma" }],
            "mapped_genes": ["BRAF"],
            "p_value": 0.0042,
            "or_per_copy_num": 1.21,
            "range": "[1.05-1.40]",
            "accession_id": "GCST90000001",
            "pubmed_id": "11111111",
            "first_author": "Gene Author"
          },
          {
            "snp_allele": [{ "rs_id": "rs121913530", "effect_allele": "C" }],
            "efo_traits": [{ "efo_trait": "cutaneous melanoma" }],
            "mapped_genes": ["BRAF"],
            "p_value": 0.031,
            "accession_id": "GCST90000002",
            "pubmed_id": "11111112",
            "first_author": "Gene Author"
          }
        ]
      },
      "page": { "size": 2, "totalElements": 2, "totalPages": 1, "number": 0 }
    }"#;
    const TRAIT_REPLY: &str = r#"{
      "_embedded": {
        "associations": [
          {
            "snp_allele": [{ "rs_id": "rs121913529", "effect_allele": "T" }],
            "efo_traits": [{ "efo_trait": "cutaneous melanoma" }],
            "mapped_genes": ["BRAF"],
            "p_value": 0.00021,
            "or_per_copy_num": 1.52,
            "range": "[1.31-1.77]",
            "accession_id": "GCST90000003",
            "pubmed_id": "22222221",
            "first_author": "Trait Author"
          },
          {
            "snp_allele": [{ "rs_id": "rs11233445", "effect_allele": "A" }],
            "efo_traits": [{ "efo_trait": "cutaneous melanoma" }],
            "mapped_genes": ["BRAF"],
            "p_value": 0.0000000031,
            "accession_id": "GCST90000004",
            "pubmed_id": "22222222",
            "first_author": "Trait Author"
          }
        ]
      },
      "page": { "size": 2, "totalElements": 2, "totalPages": 1, "number": 0 }
    }"#;
    const HOLD: Duration = Duration::from_millis(1500);
    const PAGE_DEADLINE: Duration = Duration::from_millis(2500);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind parallel GWAS fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let Ok(len) = stream.read(&mut request).await else {
                    return;
                };
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                let body = if request.contains("mapped_gene=") {
                    GENE_REPLY
                } else if request.contains("efo_trait=") {
                    TRAIT_REPLY
                } else {
                    let refused = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(refused.as_bytes()).await;
                    return;
                };
                tokio::time::sleep(HOLD).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    let mut env = crate::test_support::FixtureEnv::new();
    env.set("BIOMCP_GWAS_BASE", &base);

    let filters = super::super::GwasSearchFilters {
        gene: Some("BRAF".into()),
        trait_query: Some("melanoma".into()),
        p_value: None,
    };
    let started = Instant::now();
    let page = search_gwas_page(&filters, 10, 0)
        .await
        .expect("combined gene and trait page");
    let elapsed = started.elapsed();
    drop(env);
    server.abort();

    assert!(
        elapsed < PAGE_DEADLINE,
        "two replies held 1.5 s each must answer together, not one after the other: {elapsed:?}"
    );

    let expected = sequential_combined_rows(GENE_REPLY, TRAIT_REPLY, 10, 0);
    assert_eq!(
        serde_json::to_value(&page.results).expect("serialize page rows"),
        serde_json::to_value(&expected).expect("serialize sequential rows"),
        "the concurrent legs must return the sequential path's rows unchanged"
    );
    assert_eq!(
        serde_json::to_value(&page.pagination).expect("serialize pagination"),
        serde_json::json!({
            "limit": 10,
            "offset": 0,
            "returned": 2,
            "has_more": false,
            "next_offset": null,
            "truncated_by_provider_budget": false
        })
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
