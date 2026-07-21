//! Study markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn study_list_markdown(studies: &[StudyInfo]) -> String {
    let mut out = String::new();
    out.push_str("# Study Datasets\n\n");
    if studies.is_empty() {
        out.push_str("No local studies found.\n");
        return out;
    }

    out.push_str("| Study ID | Name | Cancer Type | Samples | Available Data |\n");
    out.push_str("|---|---|---|---|---|\n");
    for study in studies {
        let cancer_type = study.cancer_type.as_deref().unwrap_or("-");
        let sample_count = study
            .sample_count
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string());
        let available = if study.available_data.is_empty() {
            "-".to_string()
        } else {
            study.available_data.join(", ")
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            study.study_id, study.name, cancer_type, sample_count, available
        ));
    }
    out
}

fn format_optional_stat(value: Option<f64>, decimals: usize) -> String {
    value
        .map(|value| format!("{value:.prec$}", prec = decimals))
        .unwrap_or_else(|| "-".to_string())
}

fn format_optional_p_value(value: Option<f64>) -> String {
    value
        .map(|value| {
            if value == 0.0 {
                "0".to_string()
            } else if value < 0.001 {
                format!("{value:.2e}")
            } else if value < 0.01 {
                format!("{value:.4}")
            } else {
                format!("{value:.3}")
            }
        })
        .unwrap_or_else(|| "not available".to_string())
}

pub fn study_download_catalog_markdown(result: &StudyDownloadCatalog) -> String {
    let mut out = String::new();
    out.push_str("# Downloadable cBioPortal Studies\n\n");
    if result.study_ids.is_empty() {
        out.push_str("No remote study IDs found.\n");
        return out;
    }

    out.push_str("| Study ID |\n");
    out.push_str("|---|\n");
    for study_id in &result.study_ids {
        out.push_str(&format!("| {study_id} |\n"));
    }
    out
}

pub fn study_download_markdown(result: &StudyDownloadResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Study Download: {}\n\n", result.study_id));
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---|\n");
    out.push_str(&format!("| Study ID | {} |\n", result.study_id));
    out.push_str(&format!("| Path | {} |\n", result.path));
    out.push_str(&format!(
        "| Downloaded | {} |\n",
        if result.downloaded {
            "yes"
        } else {
            "already present"
        }
    ));
    out
}

pub fn study_query_markdown(result: &StudyQueryResult) -> String {
    match result {
        StudyQueryResult::NotInLocalCohorts(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Not in Local cBioPortal Cohorts: {}\n\n",
                result.study_id
            ));
            out.push_str("| Field | Value |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!("| Study ID | {} |\n", result.study_id));
            out.push_str(&format!("| Gene | {} |\n", result.gene));
            out.push_str("| Coverage Status | not_in_local_cohorts |\n");
            out.push_str(&format!("| Message | {} |\n", result.message));
            out.push_str(&format!("| Next Step | {} |\n", result.suggestion));
            out.push_str("\n## Local Cohorts in This Snapshot\n\n");
            if result.local_study_ids.is_empty() {
                out.push_str("No local studies found. Run `biomcp study download --list` to browse downloadable cBioPortal study IDs.\n");
            } else {
                for study_id in &result.local_study_ids {
                    out.push_str(&format!("- {study_id}\n"));
                }
            }
            out
        }
        StudyQueryResult::MutationFrequency(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Mutation Frequency: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Metric | Value |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!(
                "| Mutation records | {} |\n",
                result.mutation_count
            ));
            out.push_str(&format!("| Unique samples | {} |\n", result.unique_samples));
            out.push_str(&format!("| Total samples | {} |\n", result.total_samples));
            out.push_str(&format!("| Frequency | {:.6} |\n", result.frequency));
            out.push_str("\n## Top Variant Classes\n\n");
            out.push_str("| Class | Count |\n");
            out.push_str("|---|---|\n");
            if result.top_variant_classes.is_empty() {
                out.push_str("| - | 0 |\n");
            } else {
                for (class_name, count) in &result.top_variant_classes {
                    out.push_str(&format!("| {} | {} |\n", class_name, count));
                }
            }
            out.push_str("\n## Top Protein Changes\n\n");
            out.push_str("| Change | Count |\n");
            out.push_str("|---|---|\n");
            if result.top_protein_changes.is_empty() {
                out.push_str("| - | 0 |\n");
            } else {
                for (change, count) in &result.top_protein_changes {
                    out.push_str(&format!("| {} | {} |\n", change, count));
                }
            }
            if let Some(caveat) = &result.mutation_only_caveat {
                out.push_str(&format!("\nNote: {caveat}\n"));
            }
            out
        }
        StudyQueryResult::StructuralVariants(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Structural Variants: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Sample | Site 1 Gene | Site 2 Gene | Frame/Effect | Split Reads | Event Info |\n");
            out.push_str("|---|---|---|---|---|---|\n");
            if result.rows.is_empty() {
                out.push_str("| - | - | - | - | - | - |\n");
            } else {
                for row in &result.rows {
                    let split_reads = if row.tumor_split_read_count == "-"
                        && row.normal_split_read_count == "-"
                    {
                        "-".to_string()
                    } else {
                        format!(
                            "tumor {}/normal {}",
                            row.tumor_split_read_count, row.normal_split_read_count
                        )
                    };
                    out.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} |\n",
                        row.sample_id,
                        row.site1_gene,
                        row.site2_gene,
                        row.frame_effect,
                        split_reads,
                        row.event_info
                    ));
                }
            }
            out
        }
        StudyQueryResult::CnaDistribution(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study CNA Distribution: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Bucket | Count |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!(
                "| Deep deletion (-2) | {} |\n",
                result.deep_deletion
            ));
            out.push_str(&format!(
                "| Shallow deletion (-1) | {} |\n",
                result.shallow_deletion
            ));
            out.push_str(&format!("| Diploid (0) | {} |\n", result.diploid));
            out.push_str(&format!("| Gain (1) | {} |\n", result.gain));
            out.push_str(&format!(
                "| Amplification (2) | {} |\n",
                result.amplification
            ));
            out.push_str(&format!("| Total samples | {} |\n", result.total_samples));
            out
        }
        StudyQueryResult::ExpressionDistribution(result) => {
            let mut out = String::new();
            out.push_str(&format!(
                "# Study Expression Distribution: {} ({})\n\n",
                result.gene, result.study_id
            ));
            out.push_str("| Metric | Value |\n");
            out.push_str("|---|---|\n");
            out.push_str(&format!("| File | {} |\n", result.file));
            out.push_str(&format!("| Sample count | {} |\n", result.sample_count));
            out.push_str(&format!("| Mean | {:.6} |\n", result.mean));
            out.push_str(&format!("| Median | {:.6} |\n", result.median));
            out.push_str(&format!("| Min | {:.6} |\n", result.min));
            out.push_str(&format!("| Max | {:.6} |\n", result.max));
            out.push_str(&format!("| Q1 | {:.6} |\n", result.q1));
            out.push_str(&format!("| Q3 | {:.6} |\n", result.q3));
            out
        }
    }
}

pub fn study_top_mutated_markdown(result: &StudyTopMutatedGenesResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Study Top Mutated Genes: {}\n\n",
        result.study_id
    ));
    out.push_str("| Gene | Mutated Samples | Mutation Events | Total Samples | Mutation Rate |\n");
    out.push_str("|---|---|---|---|---|\n");
    if result.rows.is_empty() {
        out.push_str("| - | 0 | 0 | 0 | 0.000000 |\n");
        if let Some(caveat) = &result.mutation_only_caveat {
            out.push_str(&format!("\nNote: {caveat}\n"));
        }
        return out;
    }

    for row in &result.rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.6} |\n",
            row.gene,
            row.mutated_samples,
            row.mutation_events,
            result.total_samples,
            row.mutation_rate
        ));
    }
    if let Some(caveat) = &result.mutation_only_caveat {
        out.push_str(&format!("\nNote: {caveat}\n"));
    }
    out
}

pub fn study_filter_markdown(result: &StudyFilterResult) -> String {
    const SAMPLE_DISPLAY_LIMIT: usize = 50;

    let mut out = String::new();
    out.push_str(&format!("# Study Filter: {}\n\n", result.study_id));
    out.push_str("## Criteria\n\n");
    out.push_str("| Filter | Matching Samples |\n");
    out.push_str("|---|---|\n");
    if result.criteria.is_empty() {
        out.push_str("| - | 0 |\n");
    } else {
        for criterion in &result.criteria {
            out.push_str(&format!(
                "| {} | {} |\n",
                criterion.description, criterion.matched_count
            ));
        }
    }

    out.push_str("\n## Result\n\n");
    out.push_str("| Metric | Value |\n");
    out.push_str("|---|---|\n");
    let total = result
        .total_study_samples
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    out.push_str(&format!("| Study Total Samples | {total} |\n"));
    out.push_str(&format!("| Intersection | {} |\n", result.matched_count));

    out.push_str("\n## Matched Samples\n\n");
    if result.matched_sample_ids.is_empty() {
        out.push_str("None\n");
        return out;
    }

    for sample_id in result.matched_sample_ids.iter().take(SAMPLE_DISPLAY_LIMIT) {
        out.push_str(sample_id);
        out.push('\n');
    }
    let remaining = result
        .matched_sample_ids
        .len()
        .saturating_sub(SAMPLE_DISPLAY_LIMIT);
    if remaining > 0 {
        out.push_str(&format!(
            "... and {remaining} more (use --json for full list)\n"
        ));
    }
    out
}

pub fn study_cohort_markdown(result: &StudyCohortResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Study Cohort: {} ({})\n\n",
        result.gene, result.study_id
    ));
    let stratification = match result.stratification.as_str() {
        "mutation" => "mutation status",
        other => other,
    };
    out.push_str(&format!("Stratification: {stratification}\n\n"));
    out.push_str("| Group | Samples | Patients |\n");
    out.push_str("|---|---|---|\n");
    out.push_str(&format!(
        "| {}-mutant | {} | {} |\n",
        result.gene, result.mutant_samples, result.mutant_patients
    ));
    out.push_str(&format!(
        "| {}-wildtype | {} | {} |\n",
        result.gene, result.wildtype_samples, result.wildtype_patients
    ));
    out.push_str(&format!(
        "| Total | {} | {} |\n",
        result.total_samples, result.total_patients
    ));
    out
}

pub fn study_survival_markdown(result: &StudySurvivalResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Study Survival: {} ({})\n\n",
        result.gene, result.study_id
    ));
    out.push_str(&format!(
        "Endpoint: {} ({})\n\n",
        result.endpoint.label(),
        result.endpoint.code()
    ));
    out.push_str("| Group | N | Events | Censored | Event Rate | KM Median | 1yr | 3yr | 5yr |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {:.6} | {} | {} | {} | {} |\n",
            group.group_name,
            group.n_patients,
            group.n_events,
            group.n_censored,
            group.event_rate,
            format_optional_stat(group.km_median_months, 1),
            format_optional_stat(group.survival_1yr, 3),
            format_optional_stat(group.survival_3yr, 3),
            format_optional_stat(group.survival_5yr, 3)
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Log-rank p-value: {}\n",
        format_optional_p_value(result.log_rank_p)
    ));
    out
}

pub fn study_compare_expression_markdown(result: &StudyExpressionComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# Study Group Comparison: Expression\n\n");
    out.push_str(&format!(
        "Stratify gene: {} | Target gene: {} | Study: {}\n\n",
        result.stratify_gene, result.target_gene, result.study_id
    ));
    out.push_str("| Group | N | Mean | Median | Q1 | Q3 | Min | Max |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            group.group_name,
            group.sample_count,
            format_optional_stat(group.mean, 3),
            format_optional_stat(group.median, 3),
            format_optional_stat(group.q1, 3),
            format_optional_stat(group.q3, 3),
            format_optional_stat(group.min, 3),
            format_optional_stat(group.max, 3)
        ));
    }
    out.push('\n');
    out.push_str(&format!(
        "Mann-Whitney U: {}\n",
        format_optional_stat(result.mann_whitney_u, 3)
    ));
    out.push_str(&format!(
        "Mann-Whitney p-value: {}\n",
        format_optional_p_value(result.mann_whitney_p)
    ));
    out
}

pub fn study_compare_mutations_markdown(result: &StudyMutationComparisonResult) -> String {
    let mut out = String::new();
    out.push_str("# Study Group Comparison: Mutation Rate\n\n");
    out.push_str(&format!(
        "Stratify gene: {} | Target gene: {} | Study: {}\n\n",
        result.stratify_gene, result.target_gene, result.study_id
    ));
    out.push_str("| Group | N | Mutated | Mutation Rate |\n");
    out.push_str("|---|---|---|---|\n");
    for group in &result.groups {
        out.push_str(&format!(
            "| {} | {} | {} | {:.6} |\n",
            group.group_name, group.sample_count, group.mutated_count, group.mutation_rate
        ));
    }
    out
}

pub fn study_co_occurrence_markdown(result: &StudyCoOccurrenceResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Study Co-occurrence: {}\n\n", result.study_id));
    out.push_str(&format!("Genes: {}\n\n", result.genes.join(", ")));
    out.push_str(&format!("Total samples: {}\n\n", result.total_samples));
    out.push_str(&format!(
        "Sample universe: {}\n\n",
        match result.sample_universe_basis {
            StudySampleUniverseBasis::ClinicalSampleFile => "clinical sample file",
            StudySampleUniverseBasis::MutationObserved => {
                "mutation-observed samples only (clinical sample file unavailable)"
            }
        }
    ));
    out.push_str(
        "| Gene A | Gene B | Both | A only | B only | Neither | Log Odds Ratio | p-value |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    if result.pairs.is_empty() {
        out.push_str("| - | - | 0 | 0 | 0 | 0 | - | - |\n");
        return out;
    }
    for pair in &result.pairs {
        let lor = pair
            .log_odds_ratio
            .map(|v| format!("{v:.6}"))
            .unwrap_or_else(|| "-".to_string());
        let p_value = pair
            .p_value
            .map(|v| format!("{v:.3e}"))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            pair.gene_a,
            pair.gene_b,
            pair.both_mutated,
            pair.a_only,
            pair.b_only,
            pair.neither,
            lor,
            p_value
        ));
    }
    out
}
