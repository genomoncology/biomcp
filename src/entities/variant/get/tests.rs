//! Sidecar tests for variant detail and enrichment helpers.

use super::super::test_support::*;
use super::*;
use crate::sources::civic::{CivicContext, CivicEvidenceItem};

fn identity_hit(
    gene: &str,
    protein_change: Option<&str>,
) -> crate::sources::myvariant::MyVariantHit {
    let dbnsfp =
        protein_change.map(|change| serde_json::json!({"genename": gene, "hgvsp": change}));
    serde_json::from_value(serde_json::json!({
        "_id": "chr7:g.140453136A>T",
        "dbnsfp": dbnsfp
    }))
    .expect("valid MyVariant hit")
}

#[test]
fn exact_helper_candidate_selection_rejects_conflicts_and_missing_evidence() {
    let requested = super::super::RequestedVariantIdentity::for_search(
        Some("BRAF".into()),
        Some("p.Val600Glu".into()),
        None,
        None,
    );
    assert!(candidate_matches_requested_identity(
        &requested,
        &identity_hit("BRAF", Some("p.V600E")),
    ));
    assert!(!candidate_matches_requested_identity(
        &requested,
        &identity_hit("BRAF", Some("p.V601E")),
    ));
    assert!(!candidate_matches_requested_identity(
        &requested,
        &identity_hit("BRAF", None),
    ));
}

fn braf_variant_stub() -> Variant {
    Variant {
        section_outcomes: super::super::default_variant_section_outcomes(),
        gene: "BRAF".into(),
        id: "chr7:g.140453136A>T".into(),
        genome_build: None,
        genome_build_provenance: None,
        build_ambiguous: None,
        build_candidates: Vec::new(),
        hgvs_p: Some("p.X999Y".into()),
        legacy_name: None,
        hgvs_c: None,
        transcript: None,
        rsid: None,
        cosmic_id: None,
        significance: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population: None,
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
fn variant_detail_coordinate_serializes_with_its_answering_build() {
    let mut variant = braf_variant_stub();
    variant.genome_build = Some(GenomeBuild::Grch37);
    variant.genome_build_provenance = Some("MyVariant.info provider default".into());
    let value = serde_json::to_value(variant).expect("serialize variant detail");
    assert_eq!(value["id"], "chr7:g.140453136A>T");
    assert_eq!(value["genome_build"], "GRCh37");
    assert_eq!(
        value["genome_build_provenance"],
        "MyVariant.info provider default"
    );
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
            crate::entities::variant::VariantNormalizationAggregate::Legacy(
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
            ),
            crate::entities::variant::VariantNormalizationAggregate::Legacy(
                crate::entities::variant::VariantNormalizationServiceResult {
                    service: "variantvalidator".to_string(),
                    status: VariantNormalizationStatus::Success,
                    input_description: Some(input.to_string()),
                    normalized_description: Some(input.to_string()),
                    corrected_description: None,
                    transcript_description: Some(input.to_string()),
                    protein: Some(json!("NP_004324.2:p.(Val600Glu)")),
                    genomic_descriptions: vec![crate::entities::GenomicCoordinate {
                        coordinate: "NC_000007.14:g.140753336A>T".into(),
                        genome_build: "GRCh38".into(),
                        source: "test".into(),
                        provenance: None,
                    }],
                    warnings: Vec::new(),
                    message: None,
                },
            ),
        ],
    };

    assert_eq!(
        normalized_get_variant_id(&response).expect("normalization should yield gettable ID"),
        "chr7:g.140753336A>T"
    );
}

#[test]
fn known_build_not_found_names_the_build_and_upstream_status() {
    let message = build_aware_not_found(
        "chr1:g.1A>T",
        GenomeBuild::Grch38,
        BioMcpError::NotFound {
            entity: "variant".into(),
            id: "chr1:g.1A>T".into(),
            suggestion: "Try searching".into(),
        },
    )
    .to_string();

    assert!(message.contains("GRCh38"));
    assert!(message.contains("upstream HTTP 404"));
    assert!(!message.contains("Retry the remote source"));
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
            crate::entities::variant::VariantNormalizationAggregate::Legacy(
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
            ),
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
fn default_section_stripping_preserves_cached_civic_but_removes_graphql_context() {
    let mut variant = braf_variant_stub();
    variant.civic = Some(VariantCivicSection {
        cached_evidence: vec![CivicEvidenceItem {
            id: 1,
            name: "cached predictive evidence".into(),
            molecular_profile: "BRAF X999Y".into(),
            evidence_type: "Predictive".into(),
            evidence_level: "B".into(),
            significance: "Sensitivity".into(),
            disease: None,
            therapies: Vec::new(),
            status: "accepted".into(),
            citation: None,
            source_type: None,
            publication_year: None,
        }],
        graphql: Some(CivicContext {
            evidence_total_count: 10,
            assertion_total_count: 3,
            evidence_items: Vec::new(),
            assertions: Vec::new(),
        }),
    });

    strip_civic_live_details(&mut variant);

    let civic = variant.civic.expect("cached CIViC should remain");
    assert_eq!(civic.cached_evidence.len(), 1);
    assert!(civic.graphql.is_none());
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
        section_outcomes: super::super::default_variant_section_outcomes(),
        gene: "BRAF".into(),
        id: "chr7:g.140453136A>T".into(),
        genome_build: None,
        genome_build_provenance: None,
        build_ambiguous: None,
        build_candidates: Vec::new(),
        hgvs_p: Some("p.V600E".into()),
        legacy_name: None,
        hgvs_c: None,
        transcript: None,
        rsid: None,
        cosmic_id: None,
        significance: None,
        clinvar_id: None,
        clinvar_review_status: None,
        clinvar_review_stars: None,
        conditions: Vec::new(),
        consequence: None,
        cadd_score: None,
        sift_pred: None,
        polyphen_pred: None,
        conservation: None,
        expanded_predictions: Vec::new(),
        population: None,
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
fn population_request_requires_a_grch38_genomic_coordinate() {
    let mut variant = braf_variant_stub();
    assert_eq!(population_variant_id(&variant), None);

    variant.genome_build = Some(GenomeBuild::Grch37);
    assert_eq!(population_variant_id(&variant), None);

    variant.genome_build = Some(GenomeBuild::Grch38);
    assert_eq!(
        population_variant_id(&variant).as_deref(),
        Some("7-140453136-A-T")
    );
}

#[test]
fn population_result_names_the_pinned_dataset_and_keeps_sources_separate() {
    let data = GnomadVariantPopulation {
        variant_id: "7-140453136-A-T".into(),
        exome: Some(crate::sources::gnomad::GnomadSequencingPopulation {
            allele_frequency: Some(0.1),
            ac: 1,
            an: 10,
            homozygote_count: 0,
            hemizygote_count: 0,
            filters: vec!["AC0".into()],
            faf95: None,
            populations: Vec::new(),
        }),
        genome: None,
    };
    let result = population_result(GnomadPopulationStatus::Data, None, Some(data));

    assert_eq!(result.status, GnomadPopulationStatus::Data);
    assert_eq!(result.dataset, "gnomad_r4");
    assert_eq!(result.release, "gnomAD v4");
    assert!(result.exome.is_some());
    assert!(result.genome.is_none());
    assert!(result.faf_caveat.contains("bottlenecked"));
}

#[test]
fn population_status_json_keeps_explicit_null_exome_and_genome_results() {
    for (status, message) in [
        (GnomadPopulationStatus::Missing, GNOMAD_GRCH38_REQUIRED),
        (
            GnomadPopulationStatus::Absent,
            "This variant is absent from gnomAD v4.",
        ),
        (
            GnomadPopulationStatus::ProviderFailure,
            GNOMAD_PROVIDER_FAILURE,
        ),
    ] {
        let value = serde_json::to_value(population_result(status, Some(message), None)).unwrap();
        assert_eq!(value["status"], serde_json::to_value(status).unwrap());
        assert!(value["exome"].is_null());
        assert!(value["genome"].is_null());
        assert_eq!(value["message"], message);
    }
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
fn cancerhotspots_checked_absence_is_empty_not_data() {
    let recurrence = crate::sources::cancerhotspots::CancerHotspotRecurrence {
        source: "cancerhotspots.org".to_string(),
        position_count: None,
        same_aa_count: None,
        matched_transcript: None,
    };

    assert_eq!(
        cancerhotspots_outcome(&recurrence).outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Empty
    );
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

#[tokio::test]
async fn ticket_589_variant_preflights_are_inapplicable_without_provider_credit() {
    let mut prediction = braf_variant_stub();
    prediction.id = "rs589000".into();
    add_prediction(&mut prediction)
        .await
        .expect("inapplicable prediction should remain a successful card");

    let mut hotspots = braf_variant_stub();
    add_cancerhotspots(&mut hotspots, &VariantIdFormat::RsId("rs589000".into())).await;

    let mut cbioportal = braf_variant_stub();
    cbioportal.gene.clear();
    add_cbioportal(&mut cbioportal).await;

    let mut civic = braf_variant_stub();
    civic.gene.clear();
    civic.hgvs_p = None;
    add_civic(&mut civic).await;

    let mut gwas = braf_variant_stub();
    gwas.rsid = None;
    add_gwas_section(&mut gwas, "chr7:g.140453136A>T")
        .await
        .expect("inapplicable GWAS should remain a successful card");

    #[cfg(feature = "alphagenome")]
    let prediction_outcome = "inapplicable";
    #[cfg(not(feature = "alphagenome"))]
    let prediction_outcome = "unavailable";

    for (variant, key, provider) in [
        (&prediction, "predict", "AlphaGenome"),
        (&hotspots, "cancerhotspots", "cancerhotspots.org"),
        (&cbioportal, "cbioportal", "cBioPortal"),
        (&civic, "civic", "CIViC"),
        (&gwas, "gwas", "GWAS Catalog"),
    ] {
        let outcome = serde_json::to_value(
            variant
                .section_outcomes
                .get(key)
                .expect("requested outcome must be completed"),
        )
        .expect("outcome should serialize");
        let expected_outcome = if key == "predict" {
            prediction_outcome
        } else {
            "inapplicable"
        };
        assert_eq!(outcome["outcome"], expected_outcome, "key={key}");
        assert_eq!(outcome["sources"], serde_json::json!([]), "key={key}");
        assert!(
            outcome["message"]
                .as_str()
                .is_some_and(|message| !message.trim().is_empty()),
            "local outcome needs a safe explanation: key={key}, outcome={outcome}"
        );
        if key != "predict" || prediction_outcome == "inapplicable" {
            assert!(
                !outcome.to_string().contains(provider),
                "uncontacted provider was credited: key={key}, outcome={outcome}"
            );
        }

        let projection = crate::render::provenance::variant_section_sources(variant);
        let projected = projection
            .iter()
            .find(|section| section.key == key)
            .unwrap_or_else(|| panic!("inapplicable outcome missing from provenance: key={key}"));
        let expected_state = if key == "predict" && prediction_outcome == "unavailable" {
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        } else {
            crate::entities::section_outcome::SectionOutcomeState::Inapplicable
        };
        assert_eq!(projected.outcome, expected_state, "key={key}");
        assert!(projected.sources.is_empty(), "key={key}");
        assert!(
            projection
                .iter()
                .all(|section| section.sources.iter().all(|source| source != provider)),
            "uncontacted provider appeared in provenance: key={key}"
        );
    }
}

#[cfg(feature = "alphagenome")]
#[tokio::test]
async fn coordinate_less_prediction_is_inapplicable_without_alphagenome_credit() {
    let mut variant = braf_variant_stub();
    variant.id = "rs589000".into();

    add_prediction(&mut variant)
        .await
        .expect("coordinate preflight should remain a successful card");

    let outcome = serde_json::to_value(
        variant
            .section_outcomes
            .get("predict")
            .expect("prediction outcome must be completed"),
    )
    .expect("outcome should serialize");
    assert_eq!(outcome["outcome"], "inapplicable");
    assert_eq!(
        outcome["message"],
        "Genomic coordinates are required for prediction."
    );
    assert_eq!(outcome["sources"], serde_json::json!([]));
    assert!(
        crate::render::provenance::variant_section_sources(&variant)
            .iter()
            .all(|section| !section.sources.iter().any(|source| source == "AlphaGenome"))
    );
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
