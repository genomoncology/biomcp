use super::*;
use crate::entities::study::TopMutatedGeneRow as StudyTopMutatedGeneRow;

#[test]
fn study_top_mutated_markdown_renders_ranked_table() {
    let markdown = study_top_mutated_markdown(&StudyTopMutatedGenesResult {
        study_id: "msk_impact_2017".to_string(),
        total_samples: 3,
        rows: vec![
            StudyTopMutatedGeneRow {
                gene: "TP53".to_string(),
                mutated_samples: 2,
                mutation_events: 2,
                mutation_rate: 2.0 / 3.0,
            },
            StudyTopMutatedGeneRow {
                gene: "KRAS".to_string(),
                mutated_samples: 2,
                mutation_events: 2,
                mutation_rate: 2.0 / 3.0,
            },
        ],
        mutation_only_caveat: None,
    });

    assert!(markdown.contains("# Study Top Mutated Genes: msk_impact_2017"));
    assert!(
        markdown.contains(
            "| Gene | Mutated Samples | Mutation Events | Total Samples | Mutation Rate |"
        )
    );
    assert!(markdown.contains("| TP53 | 2 | 2 | 3 |"));
    assert!(markdown.contains("| KRAS | 2 | 2 | 3 |"));
}

use crate::entities::study::{
    CnaDistributionResult as StudyCnaDistributionResult, CoOccurrencePair as StudyCoOccurrencePair,
    CoOccurrenceResult as StudyCoOccurrenceResult, CohortResult as StudyCohortResult,
    ExpressionComparisonResult as StudyExpressionComparisonResult,
    ExpressionDistributionResult as StudyExpressionDistributionResult,
    ExpressionGroupStats as StudyExpressionGroupStats, FilterResult as StudyFilterResult,
    LocalCohortCoverageStatus as StudyLocalCohortCoverageStatus,
    MutationComparisonResult as StudyMutationComparisonResult,
    MutationFrequencyResult as StudyMutationFrequencyResult,
    MutationGroupStats as StudyMutationGroupStats,
    NotInLocalCohortsResult as StudyNotInLocalCohortsResult,
    SampleUniverseBasis as StudySampleUniverseBasis, StudyDownloadCatalog, StudyDownloadResult,
    StudyInfo, StudyQueryResult, SurvivalEndpoint as StudySurvivalEndpoint,
    SurvivalGroupResult as StudySurvivalGroupResult, SurvivalResult as StudySurvivalResult,
};

#[test]
fn study_list_markdown_renders_study_table() {
    let markdown = study_list_markdown(&[StudyInfo {
        study_id: "msk_impact_2017".to_string(),
        name: "MSK-IMPACT".to_string(),
        cancer_type: Some("mixed".to_string()),
        citation: Some("Zehir et al.".to_string()),
        sample_count: Some(10945),
        available_data: vec!["mutations".to_string(), "cna".to_string()],
    }]);

    assert!(markdown.contains("# Study Datasets"));
    assert!(markdown.contains("| Study ID | Name | Cancer Type | Samples | Available Data |"));
    assert!(markdown.contains("msk_impact_2017"));
    assert!(markdown.contains("mutations, cna"));
}

#[test]
fn study_query_markdown_renders_not_in_local_cohorts_signal() {
    let markdown = study_query_markdown(&StudyQueryResult::NotInLocalCohorts(
        StudyNotInLocalCohortsResult {
            study_id: "skcm_tcga_pan_can_atlas_2018".to_string(),
            gene: "BRAF".to_string(),
            coverage_status: StudyLocalCohortCoverageStatus::NotInLocalCohorts,
            message: "Study skcm_tcga_pan_can_atlas_2018 is not in the local cBioPortal cohort snapshot; no local cohort result was computed.".to_string(),
            suggestion: "Try `biomcp study download skcm_tcga_pan_can_atlas_2018` before querying this cohort.".to_string(),
            local_study_ids: vec!["msk_impact_2017".to_string()],
        },
    ));

    assert!(
        markdown.contains("# Study Not in Local cBioPortal Cohorts: skcm_tcga_pan_can_atlas_2018")
    );
    assert!(markdown.contains("not_in_local_cohorts"));
    assert!(markdown.contains("biomcp study download skcm_tcga_pan_can_atlas_2018"));
    assert!(markdown.contains("msk_impact_2017"));
}

#[test]
fn study_query_markdown_renders_mutation_shape() {
    let markdown = study_query_markdown(&StudyQueryResult::MutationFrequency(
        StudyMutationFrequencyResult {
            study_id: "msk_impact_2017".to_string(),
            gene: "TP53".to_string(),
            mutation_count: 10,
            unique_samples: 9,
            total_samples: 100,
            frequency: 0.09,
            top_variant_classes: vec![("Missense_Mutation".to_string(), 8)],
            top_protein_changes: vec![("p.R175H".to_string(), 3)],
            mutation_only_caveat: None,
        },
    ));

    assert!(markdown.contains("# Study Mutation Frequency: TP53 (msk_impact_2017)"));
    assert!(markdown.contains("| Mutation records | 10 |"));
    assert!(markdown.contains("## Top Variant Classes"));
    assert!(markdown.contains("## Top Protein Changes"));
}

#[test]
fn study_query_markdown_renders_cna_and_expression_shapes() {
    let cna = study_query_markdown(&StudyQueryResult::CnaDistribution(
        StudyCnaDistributionResult {
            study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
            gene: "ERBB2".to_string(),
            total_samples: 20,
            deep_deletion: 1,
            shallow_deletion: 2,
            diploid: 10,
            gain: 4,
            amplification: 3,
        },
    ));
    assert!(cna.contains("# Study CNA Distribution: ERBB2 (brca_tcga_pan_can_atlas_2018)"));
    assert!(cna.contains("| Amplification (2) | 3 |"));

    let expression = study_query_markdown(&StudyQueryResult::ExpressionDistribution(
        StudyExpressionDistributionResult {
            study_id: "paad_qcmg_uq_2016".to_string(),
            gene: "KRAS".to_string(),
            file: "data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt".to_string(),
            sample_count: 50,
            mean: 0.2,
            median: 0.1,
            min: -2.0,
            max: 2.5,
            q1: -0.4,
            q3: 0.5,
        },
    ));
    assert!(expression.contains("# Study Expression Distribution: KRAS (paad_qcmg_uq_2016)"));
    assert!(expression.contains("| Sample count | 50 |"));
}

#[test]
fn study_filter_markdown_renders_tables_and_samples() {
    let markdown = study_filter_markdown(&StudyFilterResult {
        study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
        criteria: vec![
            crate::entities::study::FilterCriterionSummary {
                description: "mutated TP53".to_string(),
                matched_count: 3,
            },
            crate::entities::study::FilterCriterionSummary {
                description: "amplified ERBB2".to_string(),
                matched_count: 2,
            },
        ],
        total_study_samples: Some(4),
        matched_count: 2,
        matched_sample_ids: vec!["S2".to_string(), "S3".to_string()],
    });

    assert!(markdown.contains("# Study Filter: brca_tcga_pan_can_atlas_2018"));
    assert!(markdown.contains("## Criteria"));
    assert!(markdown.contains("| Filter | Matching Samples |"));
    assert!(markdown.contains("| mutated TP53 | 3 |"));
    assert!(markdown.contains("## Result"));
    assert!(markdown.contains("| Study Total Samples | 4 |"));
    assert!(markdown.contains("| Intersection | 2 |"));
    assert!(markdown.contains("## Matched Samples"));
    assert!(markdown.contains("S2"));
    assert!(markdown.contains("S3"));
}

#[test]
fn study_filter_markdown_renders_empty_results_and_unknown_totals() {
    let markdown = study_filter_markdown(&StudyFilterResult {
        study_id: "demo_study".to_string(),
        criteria: vec![crate::entities::study::FilterCriterionSummary {
            description: "expression > 1.5 for MYC".to_string(),
            matched_count: 0,
        }],
        total_study_samples: None,
        matched_count: 0,
        matched_sample_ids: Vec::new(),
    });

    assert!(markdown.contains("| Study Total Samples | - |"));
    assert!(markdown.contains("| Intersection | 0 |"));
    assert!(markdown.contains("## Matched Samples"));
    assert!(markdown.contains("\nNone\n"));
}

#[test]
fn study_filter_markdown_truncates_long_sample_lists() {
    let markdown = study_filter_markdown(&StudyFilterResult {
        study_id: "long_study".to_string(),
        criteria: vec![crate::entities::study::FilterCriterionSummary {
            description: "mutated TP53".to_string(),
            matched_count: 55,
        }],
        total_study_samples: Some(100),
        matched_count: 55,
        matched_sample_ids: (1..=55).map(|idx| format!("S{idx}")).collect(),
    });

    assert!(markdown.contains("S1"));
    assert!(markdown.contains("S50"));
    assert!(!markdown.contains("S51\n"));
    assert!(markdown.contains("... and 5 more (use --json for full list)"));
}

#[test]
fn study_co_occurrence_markdown_renders_pair_table() {
    let markdown = study_co_occurrence_markdown(&StudyCoOccurrenceResult {
        study_id: "msk_impact_2017".to_string(),
        genes: vec!["TP53".to_string(), "KRAS".to_string()],
        total_samples: 100,
        sample_universe_basis: StudySampleUniverseBasis::ClinicalSampleFile,
        pairs: vec![StudyCoOccurrencePair {
            gene_a: "TP53".to_string(),
            gene_b: "KRAS".to_string(),
            both_mutated: 10,
            a_only: 20,
            b_only: 15,
            neither: 55,
            log_odds_ratio: Some(0.1234),
            p_value: Some(6.0e-22),
        }],
    });

    assert!(markdown.contains("# Study Co-occurrence: msk_impact_2017"));
    assert!(markdown.contains("Sample universe: clinical sample file"));
    assert!(markdown.contains(
        "| Gene A | Gene B | Both | A only | B only | Neither | Log Odds Ratio | p-value |"
    ));
    assert!(markdown.contains("| TP53 | KRAS | 10 | 20 | 15 | 55 | 0.123400 | 6.000e-22 |"));
}

#[test]
fn study_co_occurrence_markdown_marks_mutation_observed_fallback() {
    let markdown = study_co_occurrence_markdown(&StudyCoOccurrenceResult {
        study_id: "fallback_study".to_string(),
        genes: vec!["TP53".to_string(), "KRAS".to_string()],
        total_samples: 3,
        sample_universe_basis: StudySampleUniverseBasis::MutationObserved,
        pairs: vec![],
    });

    assert!(markdown.contains(
        "Sample universe: mutation-observed samples only (clinical sample file unavailable)"
    ));
}

#[test]
fn study_cohort_markdown_renders_group_counts() {
    let markdown = study_cohort_markdown(&StudyCohortResult {
        study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
        gene: "TP53".to_string(),
        stratification: "mutation".to_string(),
        mutant_samples: 348,
        wildtype_samples: 736,
        mutant_patients: 348,
        wildtype_patients: 736,
        total_samples: 1084,
        total_patients: 1084,
    });

    assert!(markdown.contains("# Study Cohort: TP53 (brca_tcga_pan_can_atlas_2018)"));
    assert!(markdown.contains("Stratification: mutation status"));
    assert!(markdown.contains("| Group | Samples | Patients |"));
    assert!(markdown.contains("| TP53-mutant | 348 | 348 |"));
    assert!(markdown.contains("| Total | 1084 | 1084 |"));
}

#[test]
fn study_survival_markdown_renders_group_table() {
    let markdown = study_survival_markdown(&StudySurvivalResult {
        study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
        gene: "TP53".to_string(),
        endpoint: StudySurvivalEndpoint::Os,
        groups: vec![
            StudySurvivalGroupResult {
                group_name: "TP53-mutant".to_string(),
                n_patients: 340,
                n_events: 48,
                n_censored: 292,
                km_median_months: Some(85.2),
                survival_1yr: Some(0.91),
                survival_3yr: Some(0.72),
                survival_5yr: None,
                event_rate: 0.141176,
                km_curve_points: Vec::new(),
            },
            StudySurvivalGroupResult {
                group_name: "TP53-wildtype".to_string(),
                n_patients: 720,
                n_events: 64,
                n_censored: 656,
                km_median_months: None,
                survival_1yr: Some(0.97),
                survival_3yr: Some(0.88),
                survival_5yr: Some(0.74),
                event_rate: 0.088889,
                km_curve_points: Vec::new(),
            },
        ],
        log_rank_p: Some(0.0042),
    });

    assert!(markdown.contains("# Study Survival: TP53 (brca_tcga_pan_can_atlas_2018)"));
    assert!(markdown.contains("Endpoint: Overall Survival (OS)"));
    assert!(
        markdown.contains(
            "| Group | N | Events | Censored | Event Rate | KM Median | 1yr | 3yr | 5yr |"
        )
    );
    assert!(
        markdown.contains("| TP53-mutant | 340 | 48 | 292 | 0.141176 | 85.2 | 0.910 | 0.720 | - |")
    );
    assert!(markdown.contains("Log-rank p-value: 0.004"));
}

#[test]
fn study_download_markdown_renders_result_table() {
    let markdown = study_download_markdown(&StudyDownloadResult {
        study_id: "msk_impact_2017".to_string(),
        path: "/tmp/studies/msk_impact_2017".to_string(),
        downloaded: true,
    });

    assert!(markdown.contains("# Study Download: msk_impact_2017"));
    assert!(markdown.contains("| Study ID | msk_impact_2017 |"));
    assert!(markdown.contains("| Downloaded | yes |"));
}

#[test]
fn study_download_catalog_markdown_renders_remote_ids() {
    let markdown = study_download_catalog_markdown(&StudyDownloadCatalog {
        study_ids: vec![
            "msk_impact_2017".to_string(),
            "brca_tcga_pan_can_atlas_2018".to_string(),
        ],
    });

    assert!(markdown.contains("# Downloadable cBioPortal Studies"));
    assert!(markdown.contains("| Study ID |"));
    assert!(markdown.contains("| msk_impact_2017 |"));
    assert!(markdown.contains("| brca_tcga_pan_can_atlas_2018 |"));
}

#[test]
fn study_compare_expression_markdown_renders_distribution_table() {
    let markdown = study_compare_expression_markdown(&StudyExpressionComparisonResult {
        study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
        stratify_gene: "TP53".to_string(),
        target_gene: "ERBB2".to_string(),
        groups: vec![
            StudyExpressionGroupStats {
                group_name: "TP53-mutant".to_string(),
                sample_count: 345,
                mean: Some(0.234),
                median: Some(0.112),
                min: Some(-2.1),
                max: Some(4.5),
                q1: Some(-0.45),
                q3: Some(0.78),
            },
            StudyExpressionGroupStats {
                group_name: "TP53-wildtype".to_string(),
                sample_count: 730,
                mean: Some(-0.089),
                median: Some(-0.156),
                min: Some(-3.2),
                max: Some(5.1),
                q1: Some(-0.67),
                q3: Some(0.34),
            },
        ],
        mann_whitney_u: Some(9821.0),
        mann_whitney_p: Some(0.003),
    });

    assert!(markdown.contains("# Study Group Comparison: Expression"));
    assert!(markdown.contains(
        "Stratify gene: TP53 | Target gene: ERBB2 | Study: brca_tcga_pan_can_atlas_2018"
    ));
    assert!(markdown.contains("| Group | N | Mean | Median | Q1 | Q3 | Min | Max |"));
    assert!(markdown.contains("Mann-Whitney U: 9821.000"));
    assert!(markdown.contains("Mann-Whitney p-value: 0.003"));
    assert!(
        markdown.contains(
            "| TP53-wildtype | 730 | -0.089 | -0.156 | -0.670 | 0.340 | -3.200 | 5.100 |"
        )
    );
}

#[test]
fn study_compare_mutations_markdown_renders_rate_table() {
    let markdown = study_compare_mutations_markdown(&StudyMutationComparisonResult {
        study_id: "brca_tcga_pan_can_atlas_2018".to_string(),
        stratify_gene: "TP53".to_string(),
        target_gene: "PIK3CA".to_string(),
        groups: vec![
            StudyMutationGroupStats {
                group_name: "TP53-mutant".to_string(),
                sample_count: 348,
                mutated_count: 120,
                mutation_rate: 0.344828,
            },
            StudyMutationGroupStats {
                group_name: "TP53-wildtype".to_string(),
                sample_count: 736,
                mutated_count: 220,
                mutation_rate: 0.298913,
            },
        ],
    });

    assert!(markdown.contains("# Study Group Comparison: Mutation Rate"));
    assert!(markdown.contains(
        "Stratify gene: TP53 | Target gene: PIK3CA | Study: brca_tcga_pan_can_atlas_2018"
    ));
    assert!(markdown.contains("| Group | N | Mutated | Mutation Rate |"));
    assert!(markdown.contains("| TP53-mutant | 348 | 120 | 0.344828 |"));
}
