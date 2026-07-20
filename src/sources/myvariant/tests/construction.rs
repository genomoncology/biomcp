//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query that would be sent, plus every validation error and the
//! filter-normalization helpers. Nothing is sent.

use crate::entities::variant::VariantProteinAlias;
use crate::error::BioMcpError;
use crate::sources::HttpMethod;
use crate::sources::myvariant::{
    MYVARIANT_FIELDS_GET, MYVARIANT_FIELDS_SEARCH, MyVariantClient, VariantSearchParams,
    civic_pubmed_ids, normalize_consequence_filter, normalize_impact_filter,
    normalize_population_filter, normalize_review_status_filter, normalize_significance_filter,
};

/// Empty-but-paged search params; tests override the fields they exercise.
fn params() -> VariantSearchParams {
    VariantSearchParams {
        gene: None,
        hgvsp: None,
        hgvsc: None,
        rsid: None,
        protein_alias: None,
        significance: None,
        max_frequency: None,
        min_cadd: None,
        consequence: None,
        review_status: None,
        population: None,
        revel_min: None,
        gerp_min: None,
        tumor_site: None,
        condition: None,
        impact: None,
        lof: false,
        has: None,
        missing: None,
        therapy: None,
        limit: 5,
        offset: 0,
    }
}

/// Read the single `q` query value out of a plan.
fn q(plan: &crate::sources::RequestPlan) -> &str {
    plan.query_value("q").expect("q present")
}

// ---- query_plan (free-form /query) ----

#[test]
fn query_plan_sets_path_and_core_query_params() {
    let plan =
        MyVariantClient::query_plan("dbnsfp.genename:BRAF", 3, 0, MYVARIANT_FIELDS_SEARCH).unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "query");
    assert_eq!(plan.query_value("q"), Some("dbnsfp.genename:BRAF"));
    assert_eq!(plan.query_value("size"), Some("3"));
    assert_eq!(plan.query_value("from"), Some("0"));
    assert_eq!(plan.query_value("fields"), Some(MYVARIANT_FIELDS_SEARCH));
}

#[test]
fn query_plan_trims_query_and_rejects_empty() {
    let err = MyVariantClient::query_plan("   ", 3, 0, "f").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("Query is required"));
}

#[test]
fn query_plan_rejects_offset_at_window() {
    let err = MyVariantClient::query_plan("q", 5, 10_000, "f").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--offset must be less than 10000"));
}

#[test]
fn query_plan_rejects_offset_plus_limit_overflow() {
    let err = MyVariantClient::query_plan("q", 25, 9_980, "f").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("--offset + --limit must be <= 10000")
    );
}

// ---- search_plan (filter-driven /query) ----

#[test]
fn search_plan_sets_path_size_from_and_fields() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "query");
    assert_eq!(plan.query_value("size"), Some("5"));
    assert_eq!(plan.query_value("from"), Some("0"));
    assert_eq!(plan.query_value("fields"), Some(MYVARIANT_FIELDS_SEARCH));
    assert_eq!(q(&plan), "dbnsfp.genename:BRAF");
}

#[test]
fn search_plan_uses_bounded_gerp_projection() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF".into()),
        ..params()
    })
    .unwrap();
    let fields = plan.query_value("fields").expect("fields present");
    assert!(fields.contains("dbnsfp.gerp*.rs"));
    assert!(!fields.contains("dbnsfp.gerp++.rs"));
    assert!(!fields.contains("dbnsfp.*"));
}

#[test]
fn search_plan_builds_gene_and_hgvsp_clauses_joined_with_and() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF".into()),
        hgvsp: Some("p.Val600Glu".into()),
        ..params()
    })
    .unwrap();
    let query = q(&plan);
    assert!(query.contains("dbnsfp.genename:BRAF"));
    assert!(query.contains("dbnsfp.hgvsp:\"p.Val600Glu\""));
    assert!(query.contains(" AND "));
}

#[test]
fn search_plan_prefixes_hgvsp_with_p_when_missing() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        hgvsp: Some("Val600Glu".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbnsfp.hgvsp:\"p.Val600Glu\"");
}

#[test]
fn search_plan_builds_exact_hgvsc_clause_and_prefixes_c() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        hgvsc: Some("1799T>A".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbnsfp.hgvsc:\"c.1799T>A\"");
}

#[test]
fn search_plan_keeps_already_prefixed_hgvsc() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        hgvsc: Some("c.1799T>A".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbnsfp.hgvsc:\"c.1799T>A\"");
}

#[test]
fn search_plan_lowercases_rsid_clause() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        rsid: Some("RS113488022".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbsnp.rsid:\"rs113488022\"");
}

#[test]
fn search_plan_builds_gene_residue_alias_clause() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("PTPN22".into()),
        protein_alias: Some(VariantProteinAlias {
            position: 620,
            residue: 'W',
        }),
        ..params()
    })
    .unwrap();
    assert_eq!(
        q(&plan),
        "dbnsfp.genename:PTPN22 AND (dbnsfp.hgvsp:*620W OR dbnsfp.hgvsp:*W620*)"
    );
}

#[test]
fn search_plan_protein_alias_requires_gene() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        gene: None,
        protein_alias: Some(VariantProteinAlias {
            position: 620,
            residue: 'W',
        }),
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("Residue alias search requires a gene symbol")
    );
}

#[test]
fn search_plan_rejects_invalid_gene_symbol_characters() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF:V600E".into()),
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("Gene symbol filter"));
}

#[test]
fn search_plan_significance_clause_uses_canonical_value() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        significance: Some("Likely Pathogenic".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(
        q(&plan),
        "clinvar.rcv.clinical_significance:likely_pathogenic"
    );
}

#[test]
fn search_plan_max_frequency_without_population_uses_global_af() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        max_frequency: Some(0.01),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "gnomad_exome.af.af:[* TO 0.01]");
}

#[test]
fn search_plan_max_frequency_with_population_scopes_to_population_af() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        max_frequency: Some(0.01),
        population: Some("AFR".into()),
        ..params()
    })
    .unwrap();
    let query = q(&plan);
    assert!(query.contains("gnomad_exome.af.af_afr:[* TO 0.01]"));
    // population also emits a bare existence clause
    assert!(query.contains("gnomad_exome.af.af_afr:*"));
}

#[test]
fn search_plan_rejects_out_of_range_max_frequency() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        max_frequency: Some(1.5),
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("--max-frequency must be between 0 and 1")
    );
}

#[test]
fn search_plan_min_cadd_clause() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        min_cadd: Some(20.0),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "cadd.phred:[20 TO *]");
}

#[test]
fn search_plan_rejects_negative_min_cadd() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        min_cadd: Some(-1.0),
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--min-cadd must be >= 0"));
}

#[test]
fn search_plan_rejects_non_finite_min_cadd() {
    for min_cadd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let err = MyVariantClient::search_plan(&VariantSearchParams {
            min_cadd: Some(min_cadd),
            ..params()
        })
        .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--min-cadd must be finite"));
    }
}

#[test]
fn search_plan_consequence_clause_uses_snpeff_effect_for_every_supported_term() {
    for (input, provider_term) in [
        ("missense_variant", "missense_variant"),
        ("synonymous_variant", "synonymous_variant"),
        ("frameshift_variant", "frameshift_variant"),
        ("nonsense_variant", "stop_gained"),
        ("stop_gained", "stop_gained"),
        ("stop_lost", "stop_lost"),
        ("start_lost", "start_lost"),
        ("splice_acceptor_variant", "splice_acceptor_variant"),
        ("splice_donor_variant", "splice_donor_variant"),
        ("inframe_insertion", "inframe_insertion"),
        ("inframe_deletion", "inframe_deletion"),
        ("intron_variant", "intron_variant"),
        ("upstream_gene_variant", "upstream_gene_variant"),
        ("downstream_gene_variant", "downstream_gene_variant"),
        (
            "non_coding_transcript_variant",
            "non_coding_transcript_variant",
        ),
    ] {
        let plan = MyVariantClient::search_plan(&VariantSearchParams {
            consequence: Some(input.into()),
            ..params()
        })
        .unwrap();
        assert_eq!(
            q(&plan),
            format!("snpeff.ann.effect:*{provider_term}*"),
            "input {input}"
        );
    }
}

#[test]
fn search_plan_rejects_invalid_consequence() {
    for input in ["protein_altering_variant", "missense_variant*", ""] {
        let err = MyVariantClient::search_plan(&VariantSearchParams {
            gene: Some("BRAF".into()),
            consequence: Some(input.into()),
            ..params()
        })
        .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--consequence"), "input {input}");
    }
}

#[test]
fn search_plan_review_status_clause_quotes_provider_phrase_for_every_alias() {
    for (input, provider_phrase) in [
        ("0", "no assertion criteria provided"),
        ("0_star", "no assertion criteria provided"),
        ("0_stars", "no assertion criteria provided"),
        ("none", "no assertion criteria provided"),
        ("1", "criteria provided, single submitter"),
        ("1_star", "criteria provided, single submitter"),
        ("1_stars", "criteria provided, single submitter"),
        ("2", "criteria provided, multiple submitters, no conflicts"),
        (
            "2_star",
            "criteria provided, multiple submitters, no conflicts",
        ),
        (
            "2_stars",
            "criteria provided, multiple submitters, no conflicts",
        ),
        ("3", "reviewed by expert panel"),
        ("3_star", "reviewed by expert panel"),
        ("3_stars", "reviewed by expert panel"),
        ("expert_panel", "reviewed by expert panel"),
        ("4", "practice guideline"),
        ("4_star", "practice guideline"),
        ("4_stars", "practice guideline"),
        ("criteria_provided", "criteria provided"),
    ] {
        let plan = MyVariantClient::search_plan(&VariantSearchParams {
            review_status: Some(input.into()),
            ..params()
        })
        .unwrap();
        assert_eq!(
            q(&plan),
            format!("clinvar.rcv.review_status:\"{provider_phrase}\""),
            "input {input}"
        );
    }
}

#[test]
fn search_plan_rejects_invalid_review_status() {
    for input in ["bogus", "2*", ""] {
        let err = MyVariantClient::search_plan(&VariantSearchParams {
            gene: Some("BRAF".into()),
            review_status: Some(input.into()),
            ..params()
        })
        .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--review-status"), "input {input}");
    }
}

#[test]
fn search_plan_revel_min_clause() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        revel_min: Some(0.5),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbnsfp.revel.score:[0.5 TO *]");
}

#[test]
fn search_plan_rejects_out_of_range_revel_min() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        revel_min: Some(2.0),
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("--revel-min must be between 0 and 1")
    );
}

#[test]
fn search_plan_gerp_min_clause() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        gerp_min: Some(4.0),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "dbnsfp.gerp++.rs:[4 TO *]");
}

#[test]
fn search_plan_rejects_non_finite_gerp_min() {
    for gerp_min in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let err = MyVariantClient::search_plan(&VariantSearchParams {
            gerp_min: Some(gerp_min),
            ..params()
        })
        .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--gerp-min must be finite"));
    }
}

#[test]
fn search_plan_tumor_site_and_condition_clauses() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        tumor_site: Some("skin".into()),
        condition: Some("Lung carcinoma".into()),
        ..params()
    })
    .unwrap();
    let query = q(&plan);
    assert!(query.contains("cosmic.tumor_site:\"skin\""));
    assert!(query.contains("clinvar.rcv.conditions.name:\"Lung carcinoma\""));
}

#[test]
fn search_plan_impact_clause_uppercases() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        impact: Some("high".into()),
        ..params()
    })
    .unwrap();
    assert_eq!(q(&plan), "snpeff.ann.putative_impact:HIGH");
}

#[test]
fn search_plan_lof_has_missing_and_therapy_clauses_compose() {
    let plan = MyVariantClient::search_plan(&VariantSearchParams {
        lof: true,
        has: Some("clinvar".into()),
        missing: Some("revel".into()),
        therapy: Some("vemurafenib".into()),
        ..params()
    })
    .unwrap();
    let query = q(&plan);
    assert!(query.contains("snpeff.lof.genename:*"));
    assert!(query.contains("_exists_:clinvar"));
    assert!(query.contains("NOT _exists_:dbnsfp.revel.score"));
    assert!(query.contains("civic.molecularProfiles.evidenceItems.therapies.name:\"vemurafenib\""));
}

#[test]
fn search_plan_maps_field_aliases_for_presence_and_absence() {
    let aliases = [
        ("cadd", "_exists_:cadd"),
        ("revel", "_exists_:dbnsfp.revel.score"),
        ("gerp", "_exists_:dbnsfp.gerp++.rs"),
        ("clinvar", "_exists_:clinvar"),
        (
            "gnomad",
            "(_exists_:gnomad_exome OR _exists_:gnomad.exomes OR _exists_:gnomad.genomes)",
        ),
        ("dbsnp", "_exists_:dbsnp.rsid"),
        ("snpeff", "_exists_:snpeff.ann.effect"),
        ("civic", "_exists_:civic"),
        ("cosmic", "_exists_:cosmic"),
    ];
    for &(alias, expression) in &aliases {
        let plan = MyVariantClient::search_plan(&VariantSearchParams {
            has: Some(alias.into()),
            ..params()
        })
        .unwrap();
        assert_eq!(q(&plan), expression, "presence alias {alias}");

        let plan = MyVariantClient::search_plan(&VariantSearchParams {
            missing: Some(alias.into()),
            ..params()
        })
        .unwrap();
        assert_eq!(
            q(&plan),
            format!("NOT {expression}"),
            "absence alias {alias}"
        );
    }
}

#[test]
fn search_plan_rejects_invalid_presence_and_absence_fields() {
    for (has, missing) in [
        (Some("not_a_real_field_zzz".into()), None),
        (None, Some("not_a_real_field_zzz".into())),
        (Some("revel:*".into()), None),
        (None, Some("revel:*".into())),
        (Some(String::new()), None),
        (None, Some(String::new())),
    ] {
        let expected_flag = if has.is_some() { "--has" } else { "--missing" };
        let err = MyVariantClient::search_plan(&VariantSearchParams {
            has,
            missing,
            ..params()
        })
        .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains(expected_flag));
    }
}

#[test]
fn search_plan_rejects_when_no_filters_present() {
    let err = MyVariantClient::search_plan(&params()).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("At least one filter is required"));
}

#[test]
fn search_plan_rejects_offset_at_window() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF".into()),
        offset: 10_000,
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("--offset must be less than 10000"));
}

#[test]
fn search_plan_rejects_offset_plus_limit_overflow() {
    let err = MyVariantClient::search_plan(&VariantSearchParams {
        gene: Some("BRAF".into()),
        limit: 25,
        offset: 9_980,
        ..params()
    })
    .unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(
        err.to_string()
            .contains("--offset + --limit must be <= 10000")
    );
}

// ---- get_plan (single-variant lookup) ----

#[test]
fn get_plan_builds_variant_path_with_get_fields() {
    let plan = MyVariantClient::get_plan("rs113488022").unwrap();
    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "variant/rs113488022");
    assert_eq!(plan.query_value("fields"), Some(MYVARIANT_FIELDS_GET));
    let fields = plan.query_value("fields").unwrap();
    assert!(fields.contains("dbnsfp.bayesdel.add_af.score"));
    assert!(fields.contains("dbnsfp.bayesdel.add_af.pred"));
    assert!(fields.contains("dbnsfp.bayesdel.no_af.score"));
    assert!(fields.contains("dbnsfp.bayesdel.no_af.pred"));
    assert!(!fields.contains("bayesdel_addaf"));
}

#[test]
fn get_plan_trims_and_rejects_empty_id() {
    let err = MyVariantClient::get_plan("   ").unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("Variant ID is required"));
}

#[test]
fn get_plan_rejects_overlong_id() {
    let err = MyVariantClient::get_plan(&"a".repeat(513)).unwrap_err();
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    assert!(err.to_string().contains("too long"));
}

// ---- filter normalizers ----

#[test]
fn significance_filter_accepts_aliases_and_rejects_unknown_and_empty() {
    assert_eq!(
        normalize_significance_filter("Likely Pathogenic").unwrap(),
        "likely_pathogenic"
    );
    assert_eq!(
        normalize_significance_filter("uncertain").unwrap(),
        "uncertain_significance"
    );
    assert_eq!(
        normalize_significance_filter("conflicting").unwrap(),
        "conflicting_interpretations_of_pathogenicity"
    );
    let unknown = normalize_significance_filter("bogus").unwrap_err();
    assert!(unknown.to_string().contains("--significance"));
    assert!(unknown.to_string().contains("Expected one of"));
    let empty = normalize_significance_filter("  ").unwrap_err();
    assert!(
        empty
            .to_string()
            .contains("--significance must not be empty")
    );
}

#[test]
fn consequence_filter_accepts_shorthand_and_aliases_and_rejects_unknown_and_empty() {
    assert_eq!(
        normalize_consequence_filter("missense").unwrap(),
        "missense_variant"
    );
    assert_eq!(
        normalize_consequence_filter("synonymous").unwrap(),
        "synonymous_variant"
    );
    assert_eq!(
        normalize_consequence_filter("non-synonymous").unwrap(),
        "missense_variant"
    );
    assert_eq!(
        normalize_consequence_filter("splice donor").unwrap(),
        "splice_donor_variant"
    );
    assert_eq!(
        normalize_consequence_filter("noncoding").unwrap(),
        "non_coding_transcript_variant"
    );
    let unknown = normalize_consequence_filter("bogus").unwrap_err();
    assert!(unknown.to_string().contains("--consequence"));
    assert!(unknown.to_string().contains("Expected one of"));
    let empty = normalize_consequence_filter("  ").unwrap_err();
    assert!(
        empty
            .to_string()
            .contains("--consequence must not be empty")
    );
}

#[test]
fn population_filter_lowercases_and_rejects_unknown_and_empty() {
    assert_eq!(normalize_population_filter("AFR").unwrap(), "afr");
    let unknown = normalize_population_filter("zzz").unwrap_err();
    assert!(unknown.to_string().contains("--population"));
    let empty = normalize_population_filter("  ").unwrap_err();
    assert!(empty.to_string().contains("--population must not be empty"));
}

#[test]
fn impact_filter_uppercases_and_rejects_unknown_and_empty() {
    assert_eq!(normalize_impact_filter("high").unwrap(), "HIGH");
    let unknown = normalize_impact_filter("severe").unwrap_err();
    assert!(unknown.to_string().contains("--impact"));
    let empty = normalize_impact_filter("  ").unwrap_err();
    assert!(empty.to_string().contains("--impact must not be empty"));
}

#[test]
fn review_status_filter_maps_stars_and_aliases_and_rejects_unknown_and_empty() {
    assert_eq!(
        normalize_review_status_filter("0").unwrap(),
        "no assertion criteria provided"
    );
    assert_eq!(
        normalize_review_status_filter("1_star").unwrap(),
        "criteria provided, single submitter"
    );
    assert_eq!(
        normalize_review_status_filter("4").unwrap(),
        "practice guideline"
    );
    assert_eq!(
        normalize_review_status_filter("expert_panel").unwrap(),
        "reviewed by expert panel"
    );
    assert_eq!(
        normalize_review_status_filter("criteria_provided").unwrap(),
        "criteria provided"
    );
    let unknown = normalize_review_status_filter("Some Custom Status").unwrap_err();
    assert!(unknown.to_string().contains("--review-status"));
    assert!(unknown.to_string().contains("Expected one of"));
    let empty = normalize_review_status_filter("  ").unwrap_err();
    assert!(
        empty
            .to_string()
            .contains("--review-status must not be empty")
    );
}

#[test]
fn civic_pubmed_ids_accepts_only_complete_canonical_pubmed_citations() {
    let hit = serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "civic": {"molecularProfiles": [{"evidenceItems": [
            {"source": {"citation": " PMID:6010004 ", "sourceType": "PUBMED"}},
            {"source": {"citation": 6010005, "sourceType": "pubmed"}},
            {"source": {"citation": "PMID:12; PMID:13", "sourceType": "PUBMED"}},
            {"source": {"citation": "https://pubmed.ncbi.nlm.nih.gov/14", "sourceType": "PUBMED"}},
            {"source": {"citation": "6010006", "sourceType": "ASCO"}},
            {"source": {"citation": -1, "sourceType": "PUBMED"}},
            {"source": {"citation": 18446744073709551615u64, "sourceType": "PUBMED"}}
        ]}]}
    }))
    .expect("fixture hit");

    assert_eq!(civic_pubmed_ids(&hit), vec!["6010004", "6010005"]);
}
