//! Retained variant transform behavior tests.

use super::*;

fn rcv(value: serde_json::Value) -> MyVariantClinVarRcv {
    let hit: MyVariantHit =
        serde_json::from_value(serde_json::json!({"_id":"test", "clinvar":{"rcv":value}})).unwrap();
    hit.clinvar.unwrap().rcv()[0].clone()
}

#[test]
fn significance_rank_prefers_pathogenic_over_benign() {
    assert!(significance_rank("Pathogenic") > significance_rank("Benign"));
    assert!(significance_rank("Likely pathogenic") > significance_rank("Uncertain significance"));
}

#[test]
fn normalize_gene_uppercases() {
    assert_eq!(normalize_gene("egfr").as_deref(), Some("EGFR"));
    assert_eq!(normalize_gene("  tP53 ").as_deref(), Some("TP53"));
}

#[test]
fn normalize_polyphen_codes() {
    assert_eq!(normalize_polyphen("D"), "Probably damaging");
    assert_eq!(normalize_polyphen("P"), "Possibly damaging");
    assert_eq!(normalize_polyphen("B"), "Benign");
}

#[test]
fn clinvar_review_stars_known_statuses() {
    assert_eq!(
        clinvar_review_stars("no assertion criteria provided"),
        Some(0)
    );
    assert_eq!(
        clinvar_review_stars("criteria provided, single submitter"),
        Some(1)
    );
    assert_eq!(
        clinvar_review_stars("criteria provided, multiple submitters, no conflicts"),
        Some(2)
    );
    assert_eq!(clinvar_review_stars("reviewed by expert panel"), Some(3));
    assert_eq!(clinvar_review_stars("practice guideline"), Some(4));
}

#[test]
fn pick_review_status_prefers_highest_star_rating() {
    let rcvs = vec![
        rcv(serde_json::json!({
            "clinical_significance": null,
            "review_status": "criteria provided, single submitter",
            "conditions": null,
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
        rcv(serde_json::json!({
            "clinical_significance": null,
            "review_status": "reviewed by expert panel",
            "conditions": null,
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
    ];

    let (status, stars) = pick_review_status(&rcvs);
    assert_eq!(stars, Some(3));
    assert_eq!(status.as_deref(), Some("reviewed by expert panel"));
}

#[test]
fn pick_significance_handles_empty_and_partial_rcvs() {
    let empty: Vec<MyVariantClinVarRcv> = Vec::new();
    assert_eq!(pick_significance(&empty), None);

    let partial = vec![rcv(serde_json::json!({
        "clinical_significance": null,
        "review_status": "criteria provided, single submitter",
        "conditions": null,
        "preferred_name": null,
        "accession": null,
        "version": null,
        "last_evaluated": null,
        "number_submitters": null,
    }))];
    assert_eq!(pick_significance(&partial), None);
}

#[test]
fn pick_significance_with_brca1_rcvs() {
    let rcvs = vec![
        rcv(serde_json::json!({
            "clinical_significance": "Likely benign",
            "review_status": "criteria provided, single submitter",
            "conditions": serde_json::json!({"name": "Breast-ovarian cancer"}),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
        rcv(serde_json::json!({
            "clinical_significance": "Pathogenic",
            "review_status": "reviewed by expert panel",
            "conditions": serde_json::json!({"name": "Hereditary breast cancer"}),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
    ];

    assert_eq!(pick_significance(&rcvs).as_deref(), Some("Pathogenic"));
}

#[test]
fn pick_significance_with_kras_rcvs() {
    let rcvs = vec![
        rcv(serde_json::json!({
            "clinical_significance": "Uncertain significance",
            "review_status": null,
            "conditions": serde_json::json!({"name": "Colorectal carcinoma"}),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
        rcv(serde_json::json!({
            "clinical_significance": "Likely pathogenic",
            "review_status": "criteria provided, single submitter",
            "conditions": serde_json::json!({"name": "Lung adenocarcinoma"}),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
    ];

    assert_eq!(
        pick_significance(&rcvs).as_deref(),
        Some("Likely pathogenic")
    );
}

#[test]
fn aggregate_clinvar_conditions_counts_reports() {
    let rcvs = vec![
        rcv(serde_json::json!({
            "clinical_significance": null,
            "review_status": null,
            "conditions": serde_json::json!([
                {"name": "Melanoma"},
                {"name": "Lung cancer"}
            ]),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
        rcv(serde_json::json!({
            "clinical_significance": null,
            "review_status": null,
            "conditions": serde_json::json!({"name": "Melanoma"}),
            "preferred_name": null,
            "accession": null,
            "version": null,
            "last_evaluated": null,
            "number_submitters": null,
        })),
    ];

    let (names, rows, reports) = aggregate_clinvar_conditions(&rcvs);
    assert_eq!(reports, Some(3));
    assert_eq!(names.first().map(String::as_str), Some("Melanoma"));
    assert_eq!(rows.first().map(|r| r.reports), Some(2));
}

#[test]
fn extracts_expanded_variant_sections() {
    let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "dbnsfp": {
            "genename": "BRAF",
            "hgvsp": "p.V600E",
            "sift": {"pred": "D", "score": 0.01},
            "revel": {"score": 0.94},
            "alphamissense": {"score": 0.99, "pred": "P"},
            "phylop": {"100way_vertebrate": {"rankscore": 0.92}},
            "phastcons": {"100way_vertebrate": {"rankscore": 0.88}},
            "gerp++": {"rs": 5.6}
        },
        "gnomad_exome": {"af": {"af": 0.0001, "af_afr": 0.0002, "af_eas_jpn": 0.0003}},
        "exac": {"af": 0.0004},
        "exac_nontcga": {"af": 0.0005},
        "cosmic": {"cosmic_id": "COSM476", "mut_freq": 2.8, "tumor_site": "skin"},
        "cgi": [{"drug": "vemurafenib", "association": "Responsive", "evidence_level": "FDA"}],
        "civic": {
            "molecularProfiles": [{
                "name": "BRAF V600E",
                "evidenceItems": [{
                    "id": 1,
                    "name": "EID1",
                    "evidenceType": "PREDICTIVE",
                    "evidenceLevel": "A",
                    "significance": "SENSITIVITYRESPONSE",
                    "status": "ACCEPTED",
                    "disease": {"displayName": "Melanoma"},
                    "therapies": [{"name": "Vemurafenib"}]
                }]
            }]
        }
    }))
    .expect("variant payload should parse");

    let variant = from_myvariant_hit(&hit);
    assert!(variant.conservation.is_some());
    assert!(!variant.expanded_predictions.is_empty());
    assert!(variant.population.is_none());
    assert_eq!(variant.cgi_associations.len(), 1);
    assert_eq!(
        variant
            .civic
            .as_ref()
            .map(|v| v.cached_evidence.len())
            .unwrap_or_default(),
        1
    );
}

#[test]
fn from_myvariant_hit_sets_top_disease_from_sorted_clinvar_rows() {
    let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "dbnsfp": {
            "genename": "BRAF",
            "hgvsp": "p.V600E"
        },
        "clinvar": {
            "rcv": [
                {"conditions": [{"name": "Melanoma"}, {"name": "Lung cancer"}]},
                {"conditions": {"name": "Melanoma"}}
            ]
        }
    }))
    .expect("variant payload should parse");

    let variant = from_myvariant_hit(&hit);
    assert_eq!(
        variant
            .top_disease
            .as_ref()
            .map(|row| row.condition.as_str()),
        Some("Melanoma")
    );
    assert_eq!(variant.top_disease.as_ref().map(|row| row.reports), Some(2));
    assert_eq!(
        variant
            .clinvar_conditions
            .first()
            .map(|row| row.condition.as_str()),
        Some("Melanoma")
    );
}

#[test]
fn derive_legacy_name_normalizes_stop_alias_variants() {
    for alias in ["p.L39X", "p.Leu39Ter", "p.Leu39*"] {
        let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
                "_id": "chr6:g.118880200T>G",
                "dbnsfp": {
                    "genename": "PLN",
                    "hgvsp": [alias]
                },
                "clinvar": {
                    "variant_id": 4472
                },
                "snpeff": {"ann": {"feature_id": "NM_002667.5", "genename": "PLN", "hgvs_c": "c.116T>G", "hgvs_p": alias}}
            }))
            .expect("variant payload should parse");

        let variant = from_myvariant_hit(&hit);
        assert_eq!(variant.legacy_name.as_deref(), Some("PLN L39stop"));
    }
}

#[test]
fn derive_legacy_name_normalizes_missense_alias_variants() {
    for alias in ["p.R25C", "p.Arg25Cys"] {
        let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
                "_id": "chr6:g.118880157C>T",
                "dbnsfp": {
                    "genename": "PLN",
                    "hgvsp": [alias]
                },
                "clinvar": {
                    "rcv": [{"clinical_significance": "Likely pathogenic"}]
                },
                "snpeff": {"ann": {"feature_id": "NM_002667.5", "genename": "PLN", "hgvs_c": "c.73C>T", "hgvs_p": alias}}
            }))
            .expect("variant payload should parse");

        let variant = from_myvariant_search_hit(&hit);
        assert_eq!(variant.legacy_name.as_deref(), Some("PLN R25C"));
    }
}

#[test]
fn from_myvariant_hit_leaves_legacy_name_empty_without_clinvar() {
    let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
        "_id": "chr6:g.118880200T>G",
        "dbnsfp": {
            "genename": "PLN",
            "hgvsp": ["p.L39X", "p.Leu39Ter", "p.Leu39*"]
        }
    }))
    .expect("variant payload should parse");

    let variant = from_myvariant_hit(&hit);
    assert_eq!(variant.legacy_name, None);
}

#[test]
fn transcript_annotation_never_zips_independent_dbnsfp_arrays() {
    let hit: MyVariantHit = serde_json::from_value(serde_json::json!({
            "_id": "chr10:g.89720808T>G",
            "clinvar": {
                "variant_id": 1,
                "gene": {"symbol": "PTEN"},
                "rcv": [{
                    "preferred_name": "NM_000314.8(PTEN):c.959T>G (p.Leu320Ter)",
                    "clinical_significance": "Pathogenic"
                }]
            },
            "dbnsfp": {
                "genename": ["PTEN", "PTEN"],
                "hgvsc": ["c.386T>G", "c.959T>G"],
                "hgvsp": ["p.Leu129Ter", "p.Leu320Ter"]
            },
            "snpeff": {"ann": [
                {"feature_id":"NM_001304717.5", "genename":"PTEN", "hgvs_c":"c.1478T>G", "hgvs_p":"p.Leu493Ter"},
                {"feature_id":"NM_000314.8", "genename":"PTEN", "hgvs_c":"c.959T>G", "hgvs_p":"p.Leu320Ter"}
            ]}
        }))
        .expect("adversarial MyVariant fixture");

    let variant = from_myvariant_hit(&hit);
    assert_eq!(variant.hgvs_c.as_deref(), Some("c.959T>G"));
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Leu320Ter"));
    assert_eq!(variant.legacy_name.as_deref(), Some("PTEN L320stop"));
}

#[test]
fn real_braf_receipt_prefers_the_transcript_associated_v600_annotation() {
    let hit: MyVariantHit = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/myvariant/get_braf_v600e_grch38_20260806.json"
    )))
    .expect("real BRAF receipt");

    let variant = from_myvariant_hit(&hit);
    assert_eq!(variant.hgvs_c.as_deref(), Some("c.1799T>A"));
    assert_eq!(variant.hgvs_p.as_deref(), Some("p.Val600Glu"));
    assert_eq!(variant.legacy_name.as_deref(), Some("BRAF V600E"));
}
