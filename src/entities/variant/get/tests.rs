//! Sidecar tests for variant detail and enrichment helpers.

use super::super::test_support::*;
use super::*;

fn braf_variant_stub() -> Variant {
    Variant {
        gene: "BRAF".into(),
        id: "chr7:g.140453136A>T".into(),
        hgvs_p: Some("p.X999Y".into()),
        legacy_name: None,
        hgvs_c: None,
        rsid: None,
        cosmic_id: None,
        significance: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        gnomad_af: None,
        allele_frequency_raw: None,
        allele_frequency_percent: None,
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population_breakdown: None,
        cosmic_context: None,
        cgi_associations: Vec::new(),
        civic: None,
        clinvar_conditions: Vec::new(),
        clinvar_condition_reports: None,
        top_disease: None,
        cancerhotspots: None,
        cancer_frequencies: Vec::new(),
        cancer_frequency_source: None,
        gwas: Vec::new(),
        gwas_unavailable_reason: None,
        supporting_pmids: None,
        prediction: None,
    }
}

#[test]
fn transcript_hgvs_get_and_normalize_share_normalized_genomic_identity() {
    let input = "NM_004333.6:c.1799T>A";
    assert!(matches!(
        classify_variant_input(input),
        VariantInputKind::TranscriptCodingHgvs(value) if value == input
    ));

    let response = VariantNormalizationResponse {
        input: input.to_string(),
        services: vec![
            crate::entities::variant::VariantNormalizationServiceResult {
                service: "mutalyzer".to_string(),
                status: VariantNormalizationStatus::Success,
                input_description: Some(input.to_string()),
                normalized_description: Some(input.to_string()),
                corrected_description: None,
                transcript_description: None,
                protein: Some(json!("NP_004324.2:p.(Val600Glu)")),
                genomic_descriptions: Vec::new(),
                warnings: Vec::new(),
                message: None,
            },
            crate::entities::variant::VariantNormalizationServiceResult {
                service: "variantvalidator".to_string(),
                status: VariantNormalizationStatus::Success,
                input_description: Some(input.to_string()),
                normalized_description: Some(input.to_string()),
                corrected_description: None,
                transcript_description: Some(input.to_string()),
                protein: Some(json!("NP_004324.2:p.(Val600Glu)")),
                genomic_descriptions: vec!["NC_000007.14:g.140753336A>T".to_string()],
                warnings: Vec::new(),
                message: None,
            },
        ],
    };

    assert_eq!(
        normalized_get_variant_id(&response).expect("normalization should yield gettable ID"),
        "chr7:g.140753336A>T"
    );
}

#[test]
fn transcript_hgvs_fallback_queries_clinvar_coding_identity() {
    assert_eq!(
        transcript_hgvs_clinvar_query("NM_004333.6:c.1799T>A"),
        "clinvar.hgvs.coding:\"NM_004333.6\\:c.1799T>A\""
    );
}

#[test]
fn transcript_hgvs_normalization_failure_suggests_variant_normalize() {
    let response = VariantNormalizationResponse {
        input: "NM_004333.6:c.1799T>A".to_string(),
        services: vec![
            crate::entities::variant::VariantNormalizationServiceResult {
                service: "mutalyzer".to_string(),
                status: VariantNormalizationStatus::InvalidInput,
                input_description: Some("NM_004333.6:c.1799T>A".to_string()),
                normalized_description: None,
                corrected_description: None,
                transcript_description: None,
                protein: None,
                genomic_descriptions: Vec::new(),
                warnings: Vec::new(),
                message: Some("Invalid transcript HGVS".to_string()),
            },
        ],
    };

    let message = normalized_get_variant_id(&response)
        .unwrap_err()
        .to_string();
    assert!(message.contains("Could not normalize transcript HGVS"));
    assert!(message.contains("Invalid transcript HGVS"));
    assert!(message.contains("biomcp variant normalize all NM_004333.6:c.1799T>A"));
    assert!(!message.contains("Unrecognized variant format"));
}

#[test]
fn variant_json_omits_legacy_name_when_absent() {
    let variant = gwas_only_variant_stub("rs7903146");
    let json = serde_json::to_value(&variant).expect("variant should serialize");
    assert!(json.get("legacy_name").is_none());
}

#[test]
fn parse_sections_supports_new_variant_sections() {
    let flags = parse_sections(&[
        "conservation".to_string(),
        "predictions".to_string(),
        "cosmic".to_string(),
        "cgi".to_string(),
        "civic".to_string(),
        "cbioportal".to_string(),
        "gwas".to_string(),
    ])
    .expect("sections should parse");

    assert!(flags.include_conservation);
    assert!(flags.include_expanded_predictions);
    assert!(flags.include_cosmic);
    assert!(flags.include_cgi);
    assert!(flags.include_civic);
    assert!(flags.include_cbioportal);
    assert!(flags.include_gwas);
}

#[test]
fn parse_sections_all_excludes_key_required_prediction() {
    let flags = parse_sections(&["all".to_string()]).expect("all should parse");

    assert!(!flags.include_prediction);
    assert!(flags.include_expanded_predictions);
    assert!(flags.include_clinvar);
    assert!(flags.include_population);
    assert!(flags.include_conservation);
    assert!(flags.include_cosmic);
    assert!(flags.include_cgi);
    assert!(flags.include_civic);
    assert!(flags.include_cbioportal);
    assert!(flags.include_cancerhotspots);
    assert!(flags.include_gwas);
}

#[test]
fn gwas_only_request_detection_matches_section_flags() {
    let gwas_only = parse_sections(&["gwas".to_string()]).expect("sections should parse");
    assert!(is_gwas_only_request(&gwas_only));

    let gwas_plus_clinvar = parse_sections(&["gwas".to_string(), "clinvar".to_string()])
        .expect("sections should parse");
    assert!(!is_gwas_only_request(&gwas_plus_clinvar));
}

#[test]
fn gwas_only_variant_stub_keeps_requested_rsid() {
    let variant = gwas_only_variant_stub("rs7903146");
    assert_eq!(variant.id, "rs7903146");
    assert_eq!(variant.rsid.as_deref(), Some("rs7903146"));
    assert!(variant.gwas.is_empty());
    assert_eq!(variant.gwas_unavailable_reason, None);
}

#[test]
fn workflow_signal_detects_clinvar_metadata_before_section_stripping() {
    let mut variant = gwas_only_variant_stub("rs397507459");
    assert!(!has_clinvar_workflow_signal(&variant));

    variant.significance = Some("Pathogenic".to_string());
    assert!(has_clinvar_workflow_signal(&variant));

    variant.significance = None;
    variant.conditions.push("Noonan syndrome".to_string());
    assert!(has_clinvar_workflow_signal(&variant));
}

#[test]
fn civic_molecular_profile_name_prefers_gene_and_hgvs_p() {
    let variant = Variant {
        gene: "BRAF".into(),
        id: "chr7:g.140453136A>T".into(),
        hgvs_p: Some("p.V600E".into()),
        legacy_name: None,
        hgvs_c: None,
        rsid: None,
        cosmic_id: None,
        significance: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        gnomad_af: None,
        allele_frequency_raw: None,
        allele_frequency_percent: None,
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population_breakdown: None,
        cosmic_context: None,
        cgi_associations: Vec::new(),
        civic: None,
        clinvar_conditions: Vec::new(),
        clinvar_condition_reports: None,
        top_disease: None,
        cancerhotspots: None,
        cancer_frequencies: Vec::new(),
        cancer_frequency_source: None,
        gwas: Vec::new(),
        gwas_unavailable_reason: None,
        supporting_pmids: None,
        prediction: None,
    };

    assert_eq!(
        civic_molecular_profile_name(&variant).as_deref(),
        Some("BRAF V600E")
    );
}

#[test]
fn gwas_only_request_returns_variant_when_gwas_is_unavailable() {
    let mut variant = gwas_only_variant_stub("rs7903146");
    mark_gwas_unavailable(&mut variant);

    assert_eq!(variant.id, "rs7903146");
    assert!(variant.gwas.is_empty());
    assert_eq!(
        variant.gwas_unavailable_reason.as_deref(),
        Some("GWAS association data temporarily unavailable.")
    );
    assert_eq!(variant.supporting_pmids, None);
}

#[test]
fn cancerhotspots_enrichment_uses_requested_change_not_resolved_hgvsp() {
    let rows: Vec<crate::sources::cancerhotspots::CancerHotspotRow> =
        serde_json::from_value(json!([
            {
                "hugoSymbol": "BRAF",
                "residue": "V600",
                "tumorCount": 897,
                "transcriptId": "ENST00000288602",
                "aminoAcidPosition": 600,
                "variantAminoAcid": {"E": 833}
            }
        ]))
        .expect("valid Cancer Hotspots rows");

    let recurrence = crate::sources::cancerhotspots::recurrence_for_change(&rows, "V600E");
    assert_eq!(recurrence.position_count, Some(897));
    assert_eq!(recurrence.same_aa_count, Some(833));
}

#[test]
fn cancerhotspots_upstream_failure_omits_recurrence_and_preserves_cbioportal() {
    let mut variant = braf_variant_stub();
    variant
        .cancer_frequencies
        .push(crate::sources::cbioportal::CancerFrequency {
            cancer_type: "Melanoma".into(),
            frequency: 0.5,
            sample_count: 10,
        });
    let err = BioMcpError::Api {
        api: "cancerhotspots.org".into(),
        message: "upstream failure".into(),
    };

    apply_cancerhotspots_result(&mut variant, Err(err))
        .expect_err("upstream failure should be returned");

    assert!(variant.cancerhotspots.is_none());
    assert_eq!(variant.cancer_frequencies.len(), 1);
}

#[test]
fn therapies_from_oncokb_truncation_shows_count() {
    let annotation: OncoKBAnnotation = serde_json::from_value(serde_json::json!({
        "treatments": [
            {"level": "LEVEL_1", "drugs": [{"drugName": "osimertinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_2", "drugs": [{"drugName": "afatinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_3A", "drugs": [{"drugName": "erlotinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_3B", "drugs": [{"drugName": "gefitinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_4", "drugs": [{"drugName": "dacomitinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_R1", "drugs": [{"drugName": "poziotinib"}], "cancerType": {"name": "Lung"}},
            {"level": "LEVEL_R2", "drugs": [{"drugName": "mobocertinib"}], "cancerType": {"name": "Lung"}}
        ]
    }))
    .expect("valid OncoKB annotation");

    let therapies = therapies_from_oncokb(&annotation);
    assert_eq!(therapies.len(), 6);
    assert!(
        therapies
            .last()
            .and_then(|row| row.note.as_deref())
            .is_some_and(|note| note.contains("(and 1 more)"))
    );
}
