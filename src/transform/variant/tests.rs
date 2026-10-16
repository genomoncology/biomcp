use super::*;

#[test]
fn pick_significance_handles_empty_and_partial_rcvs() {
    let empty: Vec<MyVariantClinVarRcv> = Vec::new();
    assert_eq!(pick_significance(&empty), None);

    let partial = vec![MyVariantClinVarRcv {
        clinical_significance: None,
        review_status: Some("criteria provided, single submitter".into()),
        conditions: None,
        preferred_name: None,
        accession: None,
        version: None,
        last_evaluated: None,
        number_submitters: None,
    }];
    assert_eq!(pick_significance(&partial), None);
}

#[test]
fn newest_rcv_evaluation_date_belongs_to_the_shown_classification() {
    // Recorded BRAF V600E shape (MyVariant 2026-08-06): the newest date
    // belongs to an Uncertain significance record while the shown
    // classification is Pathogenic.
    let rcvs = vec![
        test_rcv("Pathogenic", Some("2014-09-04")),
        test_rcv("Pathogenic", Some("2023-10-22")),
        test_rcv("Uncertain significance", Some("2025-01-23")),
        test_rcv("Pathogenic", None),
    ];
    let shown = pick_significance(&rcvs);
    assert_eq!(shown.as_deref(), Some("Pathogenic"));
    assert_eq!(
        newest_rcv_evaluation_date(&rcvs, shown.as_deref()),
        Some("2023-10-22".to_string())
    );
}

#[test]
fn newest_rcv_evaluation_date_skips_non_day_shaped_and_unshown_records() {
    let rcvs = vec![
        test_rcv("Pathogenic", Some("01 Apr 2019")),
        test_rcv("Pathogenic", Some("2015-06-15")),
        test_rcv("Likely pathogenic", Some("2024-12-31")),
    ];
    assert_eq!(
        newest_rcv_evaluation_date(&rcvs, Some("Pathogenic")),
        Some("2015-06-15".to_string())
    );
    // No day-shaped date on the shown classification omits the date
    // rather than trusting a provider spelling or another record's date.
    let undated = vec![test_rcv("Pathogenic", Some("01 Apr 2019"))];
    assert_eq!(
        newest_rcv_evaluation_date(&undated, Some("Pathogenic")),
        None
    );
    assert_eq!(newest_rcv_evaluation_date(&rcvs, None), None);
    assert_eq!(newest_rcv_evaluation_date(&rcvs, Some("")), None);
}

fn test_rcv(significance: &str, last_evaluated: Option<&str>) -> MyVariantClinVarRcv {
    MyVariantClinVarRcv {
        clinical_significance: Some(significance.to_string()),
        review_status: None,
        conditions: None,
        preferred_name: None,
        accession: None,
        version: None,
        last_evaluated: last_evaluated.map(str::to_string),
        number_submitters: None,
    }
}

#[test]
fn extracts_bayesdel_flavors_from_recorded_myvariant_payload() {
    let hit: MyVariantHit = serde_json::from_slice(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/myvariant/get_braf_v600e.json"
    )))
    .expect("recorded MyVariant payload should parse");

    let predictions = extract_expanded_predictions(&hit);
    let add_af = predictions
        .iter()
        .find(|entry| entry.tool == "BayesDel add-AF")
        .expect("add-AF prediction should be present");
    assert_eq!(add_af.score, Some(0.399079));
    assert_eq!(add_af.prediction.as_deref(), Some("D"));

    let no_af = predictions
        .iter()
        .find(|entry| entry.tool == "BayesDel no-AF")
        .expect("no-AF prediction should be present");
    assert_eq!(no_af.score, Some(0.335473));
    assert_eq!(no_af.prediction.as_deref(), Some("D"));
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
