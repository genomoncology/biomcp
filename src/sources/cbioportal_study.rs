use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::{Path, PathBuf};

use crate::error::BioMcpError;

const SOURCE_NAME: &str = "cbioportal-study";
const META_STUDY_FILE: &str = "meta_study.txt";
const MUTATIONS_FILE: &str = "data_mutations.txt";
const CLINICAL_SAMPLE_FILE: &str = "data_clinical_sample.txt";
const CLINICAL_PATIENT_FILE: &str = "data_clinical_patient.txt";
const CNA_FILE: &str = "data_cna.txt";
const EXPRESSION_FILES: &[&str] = &[
    "data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt",
    "data_mrna_seq_v2_rsem.txt",
];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StudyMeta {
    pub study_id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub description: Option<String>,
    pub cancer_type: Option<String>,
    pub citation: Option<String>,
    pub pmid: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StudyDataset {
    pub study_id: String,
    pub path: PathBuf,
    pub meta: StudyMeta,
    pub has_mutations: bool,
    pub has_cna: bool,
    pub has_expression: bool,
    pub has_clinical_sample: bool,
}

#[derive(Debug, Clone)]
pub struct MutationFrequencyResult {
    pub study_id: String,
    pub gene: String,
    pub mutation_count: usize,
    pub unique_samples: usize,
    pub total_samples: usize,
    pub frequency: f64,
    pub top_variant_classes: Vec<(String, usize)>,
    pub top_protein_changes: Vec<(String, usize)>,
}

#[derive(Debug, Clone)]
pub struct CnaDistributionResult {
    pub study_id: String,
    pub gene: String,
    pub total_samples: usize,
    pub deep_deletion: usize,
    pub shallow_deletion: usize,
    pub diploid: usize,
    pub gain: usize,
    pub amplification: usize,
}

#[derive(Debug, Clone)]
pub struct ExpressionDistributionResult {
    pub study_id: String,
    pub gene: String,
    pub file: String,
    pub sample_count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub q1: f64,
    pub q3: f64,
}

#[derive(Debug, Clone)]
pub struct CoOccurrencePair {
    pub gene_a: String,
    pub gene_b: String,
    pub both_mutated: usize,
    pub a_only: usize,
    pub b_only: usize,
    pub neither: usize,
    pub log_odds_ratio: Option<f64>,
    pub p_value: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleUniverseBasis {
    ClinicalSampleFile,
    MutationObserved,
}

#[derive(Debug, Clone)]
pub struct CoOccurrenceResult {
    pub study_id: String,
    pub genes: Vec<String>,
    pub total_samples: usize,
    pub sample_universe_basis: SampleUniverseBasis,
    pub pairs: Vec<CoOccurrencePair>,
}

#[derive(Debug, Clone)]
pub struct CohortSplit {
    pub study_id: String,
    pub gene: String,
    pub mutant_samples: HashSet<String>,
    pub wildtype_samples: HashSet<String>,
    pub mutant_patients: HashSet<String>,
    pub wildtype_patients: HashSet<String>,
    pub total_samples: usize,
    pub total_patients: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurvivalStatus {
    Event,
    Censored,
}

#[derive(Debug, Clone)]
pub struct PatientSurvivalRecord {
    #[allow(dead_code)]
    pub patient_id: String,
    pub status: SurvivalStatus,
    pub months: f64,
}

#[derive(Debug, Clone)]
pub struct KmEstimate {
    pub km_median_months: Option<f64>,
    pub survival_1yr: Option<f64>,
    pub survival_3yr: Option<f64>,
    pub survival_5yr: Option<f64>,
    pub curve_points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct SurvivalGroupStats {
    pub group_name: String,
    pub n_patients: usize,
    pub n_events: usize,
    pub n_censored: usize,
    pub km_median_months: Option<f64>,
    pub survival_1yr: Option<f64>,
    pub survival_3yr: Option<f64>,
    pub survival_5yr: Option<f64>,
    pub event_rate: f64,
    pub km_curve_points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone)]
pub struct SurvivalByMutationResult {
    pub study_id: String,
    pub gene: String,
    pub endpoint: String,
    pub groups: Vec<SurvivalGroupStats>,
    pub log_rank_p: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ExpressionGroupStats {
    pub group_name: String,
    pub sample_count: usize,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub q1: f64,
    pub q3: f64,
}

#[derive(Debug, Clone)]
pub struct ExpressionComparisonByMutationResult {
    pub study_id: String,
    pub stratify_gene: String,
    pub target_gene: String,
    pub groups: Vec<ExpressionGroupStats>,
    pub mann_whitney_u: Option<f64>,
    pub mann_whitney_p: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct MannWhitneyResult {
    pub u_statistic: f64,
    pub p_value: f64,
}

#[derive(Debug, Clone)]
pub struct MutationGroupStats {
    pub group_name: String,
    pub sample_count: usize,
    pub mutated_count: usize,
    pub mutation_rate: f64,
}

#[derive(Debug, Clone)]
pub struct MutationComparisonByMutationResult {
    pub study_id: String,
    pub stratify_gene: String,
    pub target_gene: String,
    pub groups: Vec<MutationGroupStats>,
}

#[derive(Debug, Clone)]
pub(crate) enum SourceFilterCriterion {
    Mutated(String),
    Amplified(String),
    Deleted(String),
    ExpressionAbove(String, f64),
    ExpressionBelow(String, f64),
    CancerType(String),
}

#[derive(Debug, Clone)]
pub(crate) struct SourceFilterCriterionSummary {
    pub description: String,
    pub matched_count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SourceFilterResult {
    pub study_id: String,
    pub criteria: Vec<SourceFilterCriterionSummary>,
    pub total_study_samples: Option<usize>,
    pub matched_count: usize,
    pub matched_sample_ids: Vec<String>,
}

pub fn resolve_study_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_STUDY_DIR")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("studies"),
        None => std::env::temp_dir().join("biomcp").join("studies"),
    }
}

pub fn list_studies(root: &Path) -> Result<Vec<StudyDataset>, BioMcpError> {
    if !root.exists() {
        return Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Study root does not exist: {}", root.display()),
            suggestion: "Set BIOMCP_STUDY_DIR to a directory containing study folders.".to_string(),
        });
    }

    if !root.is_dir() {
        return Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Study root is not a directory: {}", root.display()),
            suggestion: "Set BIOMCP_STUDY_DIR to a directory containing study folders.".to_string(),
        });
    }

    let mut studies = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_study_id = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("unknown")
            .to_string();

        let meta_path = path.join(META_STUDY_FILE);
        let meta = if meta_path.exists() {
            parse_meta_study(&meta_path, &dir_study_id)?
        } else {
            StudyMeta {
                study_id: dir_study_id.clone(),
                name: dir_study_id.clone(),
                short_name: None,
                description: None,
                cancer_type: None,
                citation: None,
                pmid: None,
            }
        };

        studies.push(StudyDataset {
            study_id: meta.study_id.clone(),
            path: path.clone(),
            meta,
            has_mutations: path.join(MUTATIONS_FILE).exists(),
            has_cna: path.join(CNA_FILE).exists(),
            has_expression: find_expression_file(&path).is_some(),
            has_clinical_sample: path.join(CLINICAL_SAMPLE_FILE).exists(),
        });
    }

    studies.sort_by(|a, b| a.study_id.cmp(&b.study_id));
    Ok(studies)
}

pub fn mutation_frequency(
    study_dir: &Path,
    gene: &str,
) -> Result<MutationFrequencyResult, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);

    let path = study_dir.join(MUTATIONS_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);

    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_idx = column_index(&header, &["TUMOR_SAMPLE_BARCODE", "SAMPLE_ID"]);
    let variant_class_idx = column_index(&header, &["VARIANT_CLASSIFICATION"]);
    let protein_change_idx = column_index(&header, &["HGVSP_SHORT", "AMINO_ACID_CHANGE"]);

    let mut mutation_count = 0usize;
    let mut unique_samples = HashSet::new();
    let mut variant_counts: HashMap<String, usize> = HashMap::new();
    let mut protein_counts: HashMap<String, usize> = HashMap::new();

    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }

        mutation_count += 1;

        if let Some(idx) = sample_idx {
            let sample = row_field(&row, idx);
            if !sample.is_empty() {
                unique_samples.insert(sample.to_string());
            }
        }

        if let Some(idx) = variant_class_idx {
            let value = row_field(&row, idx);
            let class_name = if value.is_empty() {
                "unknown".to_string()
            } else {
                value.to_string()
            };
            *variant_counts.entry(class_name).or_insert(0) += 1;
        }

        if let Some(idx) = protein_change_idx {
            let value = row_field(&row, idx);
            if !value.is_empty() {
                *protein_counts.entry(value.to_string()).or_insert(0) += 1;
            }
        }
    }

    let total_samples = clinical_sample_ids(study_dir)?
        .map(|ids| ids.len())
        .filter(|count| *count > 0)
        .unwrap_or(unique_samples.len());
    let frequency = if total_samples > 0 {
        unique_samples.len() as f64 / total_samples as f64
    } else {
        0.0
    };

    Ok(MutationFrequencyResult {
        study_id,
        gene,
        mutation_count,
        unique_samples: unique_samples.len(),
        total_samples,
        frequency,
        top_variant_classes: sorted_counts(variant_counts),
        top_protein_changes: sorted_counts(protein_counts),
    })
}

pub fn cna_distribution(
    study_dir: &Path,
    gene: &str,
) -> Result<CnaDistributionResult, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);

    let path = study_dir.join(CNA_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_start = matrix_sample_start(&header, &reader.headers, &path)?;

    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }

        let mut deep_deletion = 0usize;
        let mut shallow_deletion = 0usize;
        let mut diploid = 0usize;
        let mut gain = 0usize;
        let mut amplification = 0usize;

        for value in row.iter().skip(sample_start) {
            let Some(v) = parse_i32(value) else { continue };
            match v {
                -2 => deep_deletion += 1,
                -1 => shallow_deletion += 1,
                0 => diploid += 1,
                1 => gain += 1,
                2 => amplification += 1,
                _ => {}
            }
        }

        let total_samples = deep_deletion + shallow_deletion + diploid + gain + amplification;
        return Ok(CnaDistributionResult {
            study_id,
            gene,
            total_samples,
            deep_deletion,
            shallow_deletion,
            diploid,
            gain,
            amplification,
        });
    }

    Err(BioMcpError::NotFound {
        entity: "gene".to_string(),
        id: gene,
        suggestion: format!("Try a different gene symbol in study '{study_id}'."),
    })
}

pub fn expression_distribution(
    study_dir: &Path,
    gene: &str,
) -> Result<ExpressionDistributionResult, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);

    let path = find_expression_file(study_dir).ok_or_else(|| BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "No supported expression matrix found under {}",
            study_dir.display()
        ),
        suggestion: "Use a study with expression data or query type mutations/cna.".to_string(),
    })?;

    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_start = matrix_sample_start(&header, &reader.headers, &path)?;

    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }

        let mut values = row
            .iter()
            .skip(sample_start)
            .filter_map(|v| parse_f64(v))
            .collect::<Vec<_>>();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if values.is_empty() {
            return Ok(ExpressionDistributionResult {
                study_id,
                gene,
                file: path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string(),
                sample_count: 0,
                mean: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                q1: 0.0,
                q3: 0.0,
            });
        }

        let sample_count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / sample_count as f64;
        let median = quantile_inclusive(&values, 0.5);
        let q1 = quantile_inclusive(&values, 0.25);
        let q3 = quantile_inclusive(&values, 0.75);

        return Ok(ExpressionDistributionResult {
            study_id,
            gene,
            file: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string(),
            sample_count,
            mean,
            median,
            min: *values.first().unwrap_or(&0.0),
            max: *values.last().unwrap_or(&0.0),
            q1,
            q3,
        });
    }

    Err(BioMcpError::NotFound {
        entity: "gene".to_string(),
        id: gene,
        suggestion: format!("Try a different gene symbol in study '{study_id}'."),
    })
}

pub fn co_occurrence(
    study_dir: &Path,
    genes: &[String],
) -> Result<CoOccurrenceResult, BioMcpError> {
    let genes = normalize_gene_list(genes)?;
    if genes.len() < 2 {
        return Err(BioMcpError::InvalidArgument(
            "Co-occurrence requires at least 2 genes.".to_string(),
        ));
    }

    let study_id = study_id_from_dir(study_dir);
    let path = study_dir.join(MUTATIONS_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);

    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_idx =
        column_index(&header, &["TUMOR_SAMPLE_BARCODE", "SAMPLE_ID"]).ok_or_else(|| {
            BioMcpError::SourceUnavailable {
                source_name: SOURCE_NAME.to_string(),
                reason: format!(
                    "Missing SAMPLE_ID/Tumor_Sample_Barcode column in {}",
                    path.display()
                ),
                suggestion: "Use a valid cBioPortal mutation file.".to_string(),
            }
        })?;

    let requested: HashSet<String> = genes.iter().cloned().collect();
    let mut mutated_by_gene: HashMap<String, HashSet<String>> =
        genes.iter().map(|g| (g.clone(), HashSet::new())).collect();
    let mut observed_samples = HashSet::new();

    while let Some(row) = reader.next_row()? {
        let sample = row_field(&row, sample_idx);
        if sample.is_empty() {
            continue;
        }
        observed_samples.insert(sample.to_string());

        let row_gene = row_field(&row, gene_idx).to_ascii_uppercase();
        if !requested.contains(&row_gene) {
            continue;
        }
        if let Some(samples) = mutated_by_gene.get_mut(&row_gene) {
            samples.insert(sample.to_string());
        }
    }

    let (sample_universe, sample_universe_basis) = match clinical_sample_ids(study_dir)? {
        Some(ids) if !ids.is_empty() => (ids, SampleUniverseBasis::ClinicalSampleFile),
        _ => (observed_samples, SampleUniverseBasis::MutationObserved),
    };
    let total_samples = sample_universe.len();
    let log_fact = build_log_factorial(total_samples);

    let mut pairs = Vec::new();
    for i in 0..genes.len() {
        for j in (i + 1)..genes.len() {
            let gene_a = &genes[i];
            let gene_b = &genes[j];
            let set_a = mutated_by_gene
                .get(gene_a)
                .expect("gene set initialized for co-occurrence");
            let set_b = mutated_by_gene
                .get(gene_b)
                .expect("gene set initialized for co-occurrence");

            let both_mutated = set_a
                .iter()
                .filter(|sample| sample_universe.contains(*sample) && set_b.contains(*sample))
                .count();
            let a_only = set_a
                .iter()
                .filter(|sample| sample_universe.contains(*sample) && !set_b.contains(*sample))
                .count();
            let b_only = set_b
                .iter()
                .filter(|sample| sample_universe.contains(*sample) && !set_a.contains(*sample))
                .count();
            let neither = total_samples.saturating_sub(both_mutated + a_only + b_only);

            pairs.push(CoOccurrencePair {
                gene_a: gene_a.clone(),
                gene_b: gene_b.clone(),
                both_mutated,
                a_only,
                b_only,
                neither,
                log_odds_ratio: log_odds_ratio(both_mutated, a_only, b_only, neither),
                p_value: Some(fisher_exact_two_tailed(
                    both_mutated,
                    a_only,
                    b_only,
                    neither,
                    &log_fact,
                )),
            });
        }
    }

    Ok(CoOccurrenceResult {
        study_id,
        genes,
        total_samples,
        sample_universe_basis,
        pairs,
    })
}

pub fn mutated_sample_ids(study_dir: &Path, gene: &str) -> Result<HashSet<String>, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let path = study_dir.join(MUTATIONS_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_idx =
        column_index(&header, &["TUMOR_SAMPLE_BARCODE", "SAMPLE_ID"]).ok_or_else(|| {
            BioMcpError::SourceUnavailable {
                source_name: SOURCE_NAME.to_string(),
                reason: format!(
                    "Missing SAMPLE_ID/Tumor_Sample_Barcode column in {}",
                    path.display()
                ),
                suggestion: "Use a valid cBioPortal mutation file.".to_string(),
            }
        })?;

    let mut samples = HashSet::new();
    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }
        let sample = row_field(&row, sample_idx);
        if !sample.is_empty() {
            samples.insert(sample.to_string());
        }
    }

    Ok(samples)
}

pub fn cna_values_by_sample(
    study_dir: &Path,
    gene: &str,
) -> Result<HashMap<String, i32>, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);
    let path = study_dir.join(CNA_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_start = matrix_sample_start(&header, &reader.headers, &path)?;

    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }

        let mut values = HashMap::new();
        for (offset, sample_id) in reader.headers.iter().skip(sample_start).enumerate() {
            let sample_id = sample_id.trim();
            if sample_id.is_empty() {
                continue;
            }
            let value = row
                .get(sample_start + offset)
                .map(String::as_str)
                .unwrap_or("");
            if let Some(value) = parse_i32(value) {
                values.insert(sample_id.to_string(), value);
            }
        }
        return Ok(values);
    }

    Err(BioMcpError::NotFound {
        entity: "gene".to_string(),
        id: gene,
        suggestion: format!("Try a different gene symbol in study '{study_id}'."),
    })
}

pub fn clinical_column_values(
    study_dir: &Path,
    column: &str,
) -> Result<HashMap<String, String>, BioMcpError> {
    let path = study_dir.join(CLINICAL_SAMPLE_FILE);
    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let sample_idx = require_column(&header, "SAMPLE_ID", &path)?;
    let column_idx = require_column(&header, column, &path)?;

    let mut values = HashMap::new();
    while let Some(row) = reader.next_row()? {
        let sample = row_field(&row, sample_idx);
        if sample.is_empty() {
            continue;
        }
        let value = row_field(&row, column_idx);
        values.insert(sample.to_string(), value.to_string());
    }

    Ok(values)
}

pub(crate) fn filter_samples(
    study_dir: &Path,
    criteria: &[SourceFilterCriterion],
) -> Result<SourceFilterResult, BioMcpError> {
    if criteria.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter criterion is required. Use one or more of --mutated, --amplified, --deleted, --expression-above, --expression-below, --cancer-type.".to_string(),
        ));
    }

    let mut summaries = Vec::with_capacity(criteria.len());
    let mut intersection = None::<HashSet<String>>;

    for criterion in criteria {
        let sample_ids = criterion_sample_ids(study_dir, criterion)?;
        summaries.push(SourceFilterCriterionSummary {
            description: criterion_description(criterion),
            matched_count: sample_ids.len(),
        });

        intersection = Some(match intersection.take() {
            Some(current) => current.intersection(&sample_ids).cloned().collect(),
            None => sample_ids,
        });
    }

    let mut matched_sample_ids = intersection
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    matched_sample_ids.sort();

    Ok(SourceFilterResult {
        study_id: study_id_from_dir(study_dir),
        criteria: summaries,
        total_study_samples: clinical_sample_ids(study_dir)?.map(|ids| ids.len()),
        matched_count: matched_sample_ids.len(),
        matched_sample_ids,
    })
}

pub fn sample_to_patient_map(study_dir: &Path) -> Result<HashMap<String, String>, BioMcpError> {
    let path = study_dir.join(CLINICAL_SAMPLE_FILE);
    if !path.exists() {
        return Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Missing required file: {}", path.display()),
            suggestion:
                "Use a study with data_clinical_sample.txt for cohort, survival, and compare queries."
                    .to_string(),
        });
    }

    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let sample_idx = require_column(&header, "SAMPLE_ID", &path)?;
    let patient_idx = require_column(&header, "PATIENT_ID", &path)?;

    let mut sample_to_patient = HashMap::new();
    while let Some(row) = reader.next_row()? {
        let sample = row_field(&row, sample_idx);
        let patient = row_field(&row, patient_idx);
        if sample.is_empty() || patient.is_empty() {
            continue;
        }
        sample_to_patient.insert(sample.to_string(), patient.to_string());
    }

    Ok(sample_to_patient)
}

pub fn build_cohort_split(study_dir: &Path, gene: &str) -> Result<CohortSplit, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);
    let direct_mutant_samples = mutated_sample_ids(study_dir, &gene)?;
    let sample_to_patient = sample_to_patient_map(study_dir)?;

    let all_patients = sample_to_patient
        .values()
        .cloned()
        .collect::<HashSet<String>>();
    let mutant_patients = direct_mutant_samples
        .iter()
        .filter_map(|sample| sample_to_patient.get(sample))
        .cloned()
        .collect::<HashSet<String>>();

    let mut mutant_samples = HashSet::new();
    let mut wildtype_samples = HashSet::new();
    for (sample, patient) in &sample_to_patient {
        if mutant_patients.contains(patient) {
            mutant_samples.insert(sample.clone());
        } else {
            wildtype_samples.insert(sample.clone());
        }
    }

    let wildtype_patients = all_patients
        .difference(&mutant_patients)
        .cloned()
        .collect::<HashSet<_>>();

    Ok(CohortSplit {
        study_id,
        gene,
        mutant_samples,
        wildtype_samples,
        mutant_patients,
        wildtype_patients,
        total_samples: sample_to_patient.len(),
        total_patients: all_patients.len(),
    })
}

pub fn cohort_by_mutation(study_dir: &Path, gene: &str) -> Result<CohortSplit, BioMcpError> {
    build_cohort_split(study_dir, gene)
}

pub fn patient_survival_data(
    study_dir: &Path,
    endpoint: &str,
) -> Result<HashMap<String, PatientSurvivalRecord>, BioMcpError> {
    let endpoint = normalize_survival_endpoint(endpoint)?;
    let path = study_dir.join(CLINICAL_PATIENT_FILE);
    if !path.exists() {
        return Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Missing required file: {}", path.display()),
            suggestion:
                "Use a study with data_clinical_patient.txt and canonical survival columns."
                    .to_string(),
        });
    }

    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let patient_idx = require_column(&header, "PATIENT_ID", &path)?;
    let (status_col, months_col) = survival_columns(&endpoint);
    let status_idx = require_column(&header, status_col, &path)?;
    let months_idx = require_column(&header, months_col, &path)?;

    let mut records = HashMap::new();
    while let Some(row) = reader.next_row()? {
        let patient_id = row_field(&row, patient_idx);
        if patient_id.is_empty() {
            continue;
        }
        let Some(status) = parse_survival_status(row_field(&row, status_idx)) else {
            continue;
        };
        let Some(months) = parse_f64(row_field(&row, months_idx)) else {
            continue;
        };

        records.insert(
            patient_id.to_string(),
            PatientSurvivalRecord {
                patient_id: patient_id.to_string(),
                status,
                months,
            },
        );
    }

    Ok(records)
}

pub fn expression_values_by_sample(
    study_dir: &Path,
    gene: &str,
) -> Result<HashMap<String, f64>, BioMcpError> {
    let gene = normalize_gene(gene)?;
    let study_id = study_id_from_dir(study_dir);
    let path = find_expression_file(study_dir).ok_or_else(|| BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "No supported expression matrix found under {}",
            study_dir.display()
        ),
        suggestion: "Use a study with expression data or query type mutations/cna.".to_string(),
    })?;

    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let gene_idx = require_column(&header, "HUGO_SYMBOL", &path)?;
    let sample_start = matrix_sample_start(&header, &reader.headers, &path)?;

    while let Some(row) = reader.next_row()? {
        if !row_field(&row, gene_idx).eq_ignore_ascii_case(&gene) {
            continue;
        }

        let mut values = HashMap::new();
        for (offset, sample_id) in reader.headers.iter().skip(sample_start).enumerate() {
            let sample_id = sample_id.trim();
            if sample_id.is_empty() {
                continue;
            }
            let value = row
                .get(sample_start + offset)
                .map(String::as_str)
                .unwrap_or("");
            if let Some(value) = parse_f64(value) {
                values.insert(sample_id.to_string(), value);
            }
        }
        return Ok(values);
    }

    Err(BioMcpError::NotFound {
        entity: "gene".to_string(),
        id: gene,
        suggestion: format!("Try a different gene symbol in study '{study_id}'."),
    })
}

pub fn mutation_count_in_samples(
    study_dir: &Path,
    gene: &str,
    sample_set: &HashSet<String>,
) -> Result<usize, BioMcpError> {
    let mutated_samples = mutated_sample_ids(study_dir, gene)?;
    Ok(mutated_samples.intersection(sample_set).count())
}

pub fn survival_by_mutation(
    study_dir: &Path,
    gene: &str,
    endpoint: &str,
) -> Result<SurvivalByMutationResult, BioMcpError> {
    let cohort = build_cohort_split(study_dir, gene)?;
    let endpoint = normalize_survival_endpoint(endpoint)?;
    let survival = patient_survival_data(study_dir, &endpoint)?;
    let mutant_records = survival_records_for_patients(&cohort.mutant_patients, &survival);
    let wildtype_records = survival_records_for_patients(&cohort.wildtype_patients, &survival);

    Ok(SurvivalByMutationResult {
        study_id: cohort.study_id.clone(),
        gene: cohort.gene.clone(),
        endpoint,
        groups: vec![
            survival_group_stats(format!("{}-mutant", cohort.gene), &mutant_records),
            survival_group_stats(format!("{}-wildtype", cohort.gene), &wildtype_records),
        ],
        log_rank_p: log_rank_two_group(&mutant_records, &wildtype_records),
    })
}

pub fn compare_expression_by_mutation(
    study_dir: &Path,
    stratify_gene: &str,
    target_gene: &str,
) -> Result<ExpressionComparisonByMutationResult, BioMcpError> {
    let cohort = build_cohort_split(study_dir, stratify_gene)?;
    let target_gene = normalize_gene(target_gene)?;
    let values_by_sample = expression_values_by_sample(study_dir, &target_gene)?;
    let mutant_values = expression_values_for_samples(&cohort.mutant_samples, &values_by_sample);
    let wildtype_values =
        expression_values_for_samples(&cohort.wildtype_samples, &values_by_sample);
    let mann_whitney = mann_whitney_u_test(&mutant_values, &wildtype_values);

    Ok(ExpressionComparisonByMutationResult {
        study_id: cohort.study_id.clone(),
        stratify_gene: cohort.gene.clone(),
        target_gene,
        groups: vec![
            expression_group_stats(format!("{}-mutant", cohort.gene), &mutant_values),
            expression_group_stats(format!("{}-wildtype", cohort.gene), &wildtype_values),
        ],
        mann_whitney_u: mann_whitney.as_ref().map(|result| result.u_statistic),
        mann_whitney_p: mann_whitney.map(|result| result.p_value),
    })
}

pub fn compare_mutations_by_mutation(
    study_dir: &Path,
    stratify_gene: &str,
    target_gene: &str,
) -> Result<MutationComparisonByMutationResult, BioMcpError> {
    let cohort = build_cohort_split(study_dir, stratify_gene)?;
    let target_gene = normalize_gene(target_gene)?;
    let mutant_count = mutation_count_in_samples(study_dir, &target_gene, &cohort.mutant_samples)?;
    let wildtype_count =
        mutation_count_in_samples(study_dir, &target_gene, &cohort.wildtype_samples)?;

    Ok(MutationComparisonByMutationResult {
        study_id: cohort.study_id.clone(),
        stratify_gene: cohort.gene.clone(),
        target_gene,
        groups: vec![
            mutation_group_stats(
                format!("{}-mutant", cohort.gene),
                cohort.mutant_samples.len(),
                mutant_count,
            ),
            mutation_group_stats(
                format!("{}-wildtype", cohort.gene),
                cohort.wildtype_samples.len(),
                wildtype_count,
            ),
        ],
    })
}

fn criterion_sample_ids(
    study_dir: &Path,
    criterion: &SourceFilterCriterion,
) -> Result<HashSet<String>, BioMcpError> {
    match criterion {
        SourceFilterCriterion::Mutated(gene) => mutated_sample_ids(study_dir, gene),
        SourceFilterCriterion::Amplified(gene) => Ok(missing_gene_row_as_empty(
            cna_values_by_sample(study_dir, gene),
        )?
        .into_iter()
        .filter_map(|(sample_id, value)| (value == 2).then_some(sample_id))
        .collect()),
        SourceFilterCriterion::Deleted(gene) => Ok(missing_gene_row_as_empty(
            cna_values_by_sample(study_dir, gene),
        )?
        .into_iter()
        .filter_map(|(sample_id, value)| (value == -2).then_some(sample_id))
        .collect()),
        SourceFilterCriterion::ExpressionAbove(gene, threshold) => Ok(missing_gene_row_as_empty(
            expression_values_by_sample(study_dir, gene),
        )?
        .into_iter()
        .filter_map(|(sample_id, value)| (value > *threshold).then_some(sample_id))
        .collect()),
        SourceFilterCriterion::ExpressionBelow(gene, threshold) => Ok(missing_gene_row_as_empty(
            expression_values_by_sample(study_dir, gene),
        )?
        .into_iter()
        .filter_map(|(sample_id, value)| (value < *threshold).then_some(sample_id))
        .collect()),
        SourceFilterCriterion::CancerType(cancer_type) => {
            let expected = cancer_type.trim();
            Ok(clinical_column_values(study_dir, "CANCER_TYPE")?
                .into_iter()
                .filter_map(|(sample_id, value)| {
                    value
                        .trim()
                        .eq_ignore_ascii_case(expected)
                        .then_some(sample_id)
                })
                .collect())
        }
    }
}

fn criterion_description(criterion: &SourceFilterCriterion) -> String {
    match criterion {
        SourceFilterCriterion::Mutated(gene) => format!("mutated {gene}"),
        SourceFilterCriterion::Amplified(gene) => format!("amplified {gene}"),
        SourceFilterCriterion::Deleted(gene) => format!("deleted {gene}"),
        SourceFilterCriterion::ExpressionAbove(gene, threshold) => {
            format!("expression > {} for {gene}", format_threshold(*threshold))
        }
        SourceFilterCriterion::ExpressionBelow(gene, threshold) => {
            format!("expression < {} for {gene}", format_threshold(*threshold))
        }
        SourceFilterCriterion::CancerType(cancer_type) => {
            format!("cancer type = {}", cancer_type.trim())
        }
    }
}

fn missing_gene_row_as_empty<T>(
    result: Result<HashMap<String, T>, BioMcpError>,
) -> Result<HashMap<String, T>, BioMcpError> {
    match result {
        Ok(values) => Ok(values),
        Err(BioMcpError::NotFound { entity, .. }) if entity == "gene" => Ok(HashMap::new()),
        Err(err) => Err(err),
    }
}

fn parse_meta_study(path: &Path, fallback_study_id: &str) -> Result<StudyMeta, BioMcpError> {
    let file = File::open(path).map_err(|_| BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!("Missing study metadata file: {}", path.display()),
        suggestion: "Ensure each study folder contains meta_study.txt.".to_string(),
    })?;

    let mut study_id = fallback_study_id.to_string();
    let mut name = fallback_study_id.to_string();
    let mut short_name = None;
    let mut description = None;
    let mut cancer_type = None;
    let mut citation = None;
    let mut pmid = None;

    for line in BufReader::new(file).lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = raw_key.trim().to_ascii_lowercase();
        let value = raw_value.trim();

        match key.as_str() {
            "cancer_study_identifier" => {
                if !value.is_empty() {
                    study_id = value.to_string();
                }
            }
            "name" => {
                if !value.is_empty() {
                    name = value.to_string();
                }
            }
            "short_name" => {
                short_name = non_empty(value);
            }
            "description" => {
                description = non_empty(value);
            }
            "type_of_cancer" => {
                cancer_type = non_empty(value);
            }
            "citation" => {
                citation = non_empty(value);
            }
            "pmid" => {
                pmid = non_empty(value);
            }
            _ => {}
        }
    }

    Ok(StudyMeta {
        study_id,
        name,
        short_name,
        description,
        cancer_type,
        citation,
        pmid,
    })
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn find_expression_file(study_dir: &Path) -> Option<PathBuf> {
    for name in EXPRESSION_FILES {
        let path = study_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn normalize_gene(gene: &str) -> Result<String, BioMcpError> {
    let gene = gene.trim();
    if gene.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene is required for study query.".to_string(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(gene) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {gene}"
        )));
    }
    Ok(gene.to_ascii_uppercase())
}

fn normalize_gene_list(genes: &[String]) -> Result<Vec<String>, BioMcpError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for raw in genes {
        let gene = normalize_gene(raw)?;
        if seen.insert(gene.clone()) {
            out.push(gene);
        }
    }
    Ok(out)
}

fn study_id_from_dir(study_dir: &Path) -> String {
    study_dir
        .file_name()
        .and_then(|s| s.to_str())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn parse_f64(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() || matches!(value.to_ascii_uppercase().as_str(), "NA" | "NAN" | "NULL") {
        return None;
    }
    value.parse::<f64>().ok()
}

fn parse_i32(value: &str) -> Option<i32> {
    let value = value.trim();
    if value.is_empty() || matches!(value.to_ascii_uppercase().as_str(), "NA" | "NAN" | "NULL") {
        return None;
    }
    value.parse::<i32>().ok()
}

fn format_threshold(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn sorted_counts(map: HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut out = map.into_iter().collect::<Vec<_>>();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

fn quantile_inclusive(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    let position = p.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        return sorted[lower];
    }
    let weight = position - lower as f64;
    sorted[lower] * (1.0 - weight) + sorted[upper] * weight
}

fn normalize_survival_endpoint(endpoint: &str) -> Result<String, BioMcpError> {
    let endpoint = endpoint.trim().to_ascii_uppercase();
    match endpoint.as_str() {
        "OS" | "DFS" | "PFS" | "DSS" => Ok(endpoint),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown survival endpoint '{other}'. Expected: os, dfs, pfs, dss."
        ))),
    }
}

fn survival_columns(endpoint: &str) -> (&'static str, &'static str) {
    match endpoint {
        "OS" => ("OS_STATUS", "OS_MONTHS"),
        "DFS" => ("DFS_STATUS", "DFS_MONTHS"),
        "PFS" => ("PFS_STATUS", "PFS_MONTHS"),
        "DSS" => ("DSS_STATUS", "DSS_MONTHS"),
        _ => unreachable!("endpoint validated before column lookup"),
    }
}

fn parse_survival_status(value: &str) -> Option<SurvivalStatus> {
    let prefix = value
        .trim()
        .split_once(':')
        .map(|(head, _)| head)
        .unwrap_or(value);
    match prefix.trim() {
        "1" => Some(SurvivalStatus::Event),
        "0" => Some(SurvivalStatus::Censored),
        _ => None,
    }
}

fn erfc_approx(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let poly = -z * z - 1.265_512_23
        + t * (1.000_023_68
            + t * (0.374_091_96
                + t * (0.096_784_18
                    + t * (-0.186_288_06
                        + t * (0.278_868_07
                            + t * (-1.135_203_98
                                + t * (1.488_515_87 + t * (-0.822_152_23 + t * 0.170_872_77))))))));
    let approx = t * poly.exp();
    if x >= 0.0 { approx } else { 2.0 - approx }
}

fn two_sided_normal_tail(z: f64) -> f64 {
    erfc_approx(z.abs() / std::f64::consts::SQRT_2).clamp(0.0, 1.0)
}

fn chi_square_1df_tail(chi2: f64) -> f64 {
    if !chi2.is_finite() || chi2 < 0.0 {
        return 1.0;
    }
    erfc_approx((chi2 / 2.0).sqrt()).clamp(0.0, 1.0)
}

fn survival_records_for_patients<'a>(
    patients: &HashSet<String>,
    survival: &'a HashMap<String, PatientSurvivalRecord>,
) -> Vec<&'a PatientSurvivalRecord> {
    patients
        .iter()
        .filter_map(|patient| survival.get(patient))
        .collect::<Vec<_>>()
}

fn event_times(records: &[&PatientSurvivalRecord]) -> Vec<f64> {
    let mut times = records
        .iter()
        .filter(|record| matches!(record.status, SurvivalStatus::Event))
        .map(|record| record.months)
        .collect::<Vec<_>>();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    times.dedup_by(|a, b| a == b);
    times
}

fn kaplan_meier(records: &[&PatientSurvivalRecord]) -> KmEstimate {
    if records.is_empty() {
        return KmEstimate {
            km_median_months: None,
            survival_1yr: None,
            survival_3yr: None,
            survival_5yr: None,
            curve_points: Vec::new(),
        };
    }

    let max_follow_up = records
        .iter()
        .map(|record| record.months)
        .fold(0.0_f64, f64::max);
    let times = event_times(records);
    if times.is_empty() {
        return KmEstimate {
            km_median_months: None,
            survival_1yr: Some(1.0),
            survival_3yr: Some(1.0),
            survival_5yr: Some(1.0),
            curve_points: vec![(0.0, 1.0), (max_follow_up, 1.0)],
        };
    }

    let mut survival = 1.0;
    let mut km_median_months = None;
    let mut survival_1yr = Some(1.0);
    let mut survival_3yr = Some(1.0);
    let mut survival_5yr = Some(1.0);
    let mut curve_points = vec![(0.0, 1.0)];

    for time in times.iter().copied() {
        let at_risk = records
            .iter()
            .filter(|record| record.months >= time)
            .count();
        if at_risk == 0 {
            continue;
        }
        let event_count = records
            .iter()
            .filter(|record| {
                matches!(record.status, SurvivalStatus::Event) && record.months == time
            })
            .count();
        if event_count == 0 {
            continue;
        }
        survival *= 1.0 - event_count as f64 / at_risk as f64;
        if km_median_months.is_none() && survival <= 0.5 {
            km_median_months = Some(time);
        }
        if time <= 12.0 {
            survival_1yr = Some(survival);
        }
        if time <= 36.0 {
            survival_3yr = Some(survival);
        }
        if time <= 60.0 {
            survival_5yr = Some(survival);
        }
        curve_points.push((time, survival));
    }

    if let Some(last_event_time) = times.last().copied()
        && max_follow_up > last_event_time
    {
        curve_points.push((max_follow_up, survival));
    }

    KmEstimate {
        km_median_months,
        survival_1yr,
        survival_3yr,
        survival_5yr,
        curve_points,
    }
}

fn log_rank_two_group(
    group_a: &[&PatientSurvivalRecord],
    group_b: &[&PatientSurvivalRecord],
) -> Option<f64> {
    if group_a.is_empty() || group_b.is_empty() {
        return None;
    }

    let mut observed_minus_expected = 0.0;
    let mut variance = 0.0;
    let mut total_events = 0_usize;

    let mut pooled = Vec::with_capacity(group_a.len() + group_b.len());
    pooled.extend_from_slice(group_a);
    pooled.extend_from_slice(group_b);

    for time in event_times(&pooled) {
        let n_a = group_a
            .iter()
            .filter(|record| record.months >= time)
            .count();
        let n_b = group_b
            .iter()
            .filter(|record| record.months >= time)
            .count();
        let n = n_a + n_b;
        if n == 0 {
            continue;
        }

        let d_a = group_a
            .iter()
            .filter(|record| {
                matches!(record.status, SurvivalStatus::Event) && record.months == time
            })
            .count();
        let d_b = group_b
            .iter()
            .filter(|record| {
                matches!(record.status, SurvivalStatus::Event) && record.months == time
            })
            .count();
        let d = d_a + d_b;
        if d == 0 {
            continue;
        }

        total_events += d;
        let n_a_fraction = n_a as f64 / n as f64;
        observed_minus_expected += d_a as f64 - d as f64 * n_a_fraction;
        if n > 1 {
            variance +=
                d as f64 * n_a_fraction * (1.0 - n_a_fraction) * ((n - d) as f64 / (n - 1) as f64);
        }
    }

    if total_events == 0 || variance <= 0.0 {
        return None;
    }

    Some(chi_square_1df_tail(
        observed_minus_expected * observed_minus_expected / variance,
    ))
}

fn survival_group_stats(
    group_name: String,
    records: &[&PatientSurvivalRecord],
) -> SurvivalGroupStats {
    let mut months = records
        .iter()
        .map(|record| record.months)
        .collect::<Vec<_>>();
    months.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n_patients = months.len();
    let n_events = records
        .iter()
        .filter(|record| matches!(record.status, SurvivalStatus::Event))
        .count();
    let n_censored = n_patients.saturating_sub(n_events);
    let event_rate = if n_patients > 0 {
        n_events as f64 / n_patients as f64
    } else {
        0.0
    };
    let km = kaplan_meier(records);

    SurvivalGroupStats {
        group_name,
        n_patients,
        n_events,
        n_censored,
        km_median_months: km.km_median_months,
        survival_1yr: km.survival_1yr,
        survival_3yr: km.survival_3yr,
        survival_5yr: km.survival_5yr,
        event_rate,
        km_curve_points: km.curve_points,
    }
}

fn expression_values_for_samples(
    sample_set: &HashSet<String>,
    values_by_sample: &HashMap<String, f64>,
) -> Vec<f64> {
    sample_set
        .iter()
        .filter_map(|sample| values_by_sample.get(sample).copied())
        .collect::<Vec<_>>()
}

fn mann_whitney_u_test(group_a: &[f64], group_b: &[f64]) -> Option<MannWhitneyResult> {
    if group_a.is_empty() || group_b.is_empty() {
        return None;
    }

    let mut pooled = group_a
        .iter()
        .map(|value| (*value, 0_u8))
        .chain(group_b.iter().map(|value| (*value, 1_u8)))
        .collect::<Vec<_>>();
    pooled.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut rank_sum_a = 0.0;
    let mut tie_correction_sum = 0.0;
    let mut idx = 0_usize;
    while idx < pooled.len() {
        let mut end = idx + 1;
        while end < pooled.len() && pooled[end].0 == pooled[idx].0 {
            end += 1;
        }
        let rank_start = idx as f64 + 1.0;
        let rank_end = end as f64;
        let midrank = (rank_start + rank_end) / 2.0;
        let count_a = pooled[idx..end]
            .iter()
            .filter(|(_, group)| *group == 0)
            .count();
        rank_sum_a += midrank * count_a as f64;
        let tie_size = (end - idx) as f64;
        tie_correction_sum += tie_size.powi(3) - tie_size;
        idx = end;
    }

    let n1 = group_a.len() as f64;
    let n2 = group_b.len() as f64;
    let n = n1 + n2;
    let u_a = rank_sum_a - n1 * (n1 + 1.0) / 2.0;
    let u_b = n1 * n2 - u_a;
    let u_statistic = u_a.min(u_b);
    let variance = n1 * n2 / 12.0 * ((n + 1.0) - tie_correction_sum / (n * (n - 1.0)));
    if variance <= 0.0 {
        return None;
    }

    let mean = n1 * n2 / 2.0;
    let z = ((mean - u_statistic).abs() - 0.5).max(0.0) / variance.sqrt();
    Some(MannWhitneyResult {
        u_statistic,
        p_value: two_sided_normal_tail(z),
    })
}

fn expression_group_stats(group_name: String, values: &[f64]) -> ExpressionGroupStats {
    let mut values = values.to_vec();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if values.is_empty() {
        return ExpressionGroupStats {
            group_name,
            sample_count: 0,
            mean: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            q1: 0.0,
            q3: 0.0,
        };
    }

    let sample_count = values.len();
    let sum: f64 = values.iter().sum();
    ExpressionGroupStats {
        group_name,
        sample_count,
        mean: sum / sample_count as f64,
        median: quantile_inclusive(&values, 0.5),
        min: *values.first().unwrap_or(&0.0),
        max: *values.last().unwrap_or(&0.0),
        q1: quantile_inclusive(&values, 0.25),
        q3: quantile_inclusive(&values, 0.75),
    }
}

fn mutation_group_stats(
    group_name: String,
    sample_count: usize,
    mutated_count: usize,
) -> MutationGroupStats {
    let mutation_rate = if sample_count > 0 {
        mutated_count as f64 / sample_count as f64
    } else {
        0.0
    };
    MutationGroupStats {
        group_name,
        sample_count,
        mutated_count,
        mutation_rate,
    }
}

fn build_log_factorial(n: usize) -> Vec<f64> {
    let mut log_fact = vec![0.0_f64; n + 1];
    for i in 1..=n {
        log_fact[i] = log_fact[i - 1] + (i as f64).ln();
    }
    log_fact
}

fn fisher_exact_two_tailed(a: usize, b: usize, c: usize, d: usize, log_fact: &[f64]) -> f64 {
    let n = a + b + c + d;
    if n == 0 {
        return 1.0;
    }

    debug_assert!(log_fact.len() > n);

    let r1 = a + b;
    let r2 = c + d;
    let c1 = a + c;
    let k_min = c1.saturating_sub(r2);
    let k_max = r1.min(c1);

    let log_p = |k: usize| -> f64 {
        log_fact[r1] - log_fact[k] - log_fact[r1 - k] + log_fact[r2]
            - log_fact[c1 - k]
            - log_fact[r2 - (c1 - k)]
            - log_fact[n]
            + log_fact[c1]
            + log_fact[n - c1]
    };

    let log_p_observed = log_p(a);
    let log_cutoff = log_p_observed + 1e-7_f64.ln_1p();
    let mut p_value = 0.0;

    for k in k_min..=k_max {
        let log_p_k = log_p(k);
        if log_p_k <= log_cutoff {
            p_value += log_p_k.exp();
        }
    }

    p_value.min(1.0)
}

fn log_odds_ratio(both: usize, a_only: usize, b_only: usize, neither: usize) -> Option<f64> {
    let total = both + a_only + b_only + neither;
    if total == 0 {
        return None;
    }

    let mut a = both as f64;
    let mut b = a_only as f64;
    let mut c = b_only as f64;
    let mut d = neither as f64;
    if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 {
        a += 0.5;
        b += 0.5;
        c += 0.5;
        d += 0.5;
    }

    Some(((a * d) / (b * c)).ln())
}

fn clinical_sample_ids(study_dir: &Path) -> Result<Option<HashSet<String>>, BioMcpError> {
    let path = study_dir.join(CLINICAL_SAMPLE_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let mut reader = TsvReader::open(&path)?;
    let header = header_map(&reader.headers);
    let Some(sample_idx) = column_index(&header, &["SAMPLE_ID"]) else {
        return Ok(None);
    };

    let mut samples = HashSet::new();
    while let Some(row) = reader.next_row()? {
        let sample = row_field(&row, sample_idx);
        if !sample.is_empty() {
            samples.insert(sample.to_string());
        }
    }
    Ok(Some(samples))
}

fn matrix_sample_start(
    header: &HashMap<String, usize>,
    headers: &[String],
    path: &Path,
) -> Result<usize, BioMcpError> {
    let hugo_idx = require_column(header, "HUGO_SYMBOL", path)?;
    let sample_start = match header.get("ENTREZ_GENE_ID") {
        Some(entrez_idx) if *entrez_idx > hugo_idx => entrez_idx + 1,
        _ => hugo_idx + 1,
    };
    if sample_start >= headers.len() {
        return Err(BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("No sample columns found in {}", path.display()),
            suggestion: "Use a valid cBioPortal matrix file with sample columns.".to_string(),
        });
    }
    Ok(sample_start)
}

fn row_field(row: &[String], idx: usize) -> &str {
    row.get(idx).map(String::as_str).unwrap_or("").trim()
}

fn normalize_header(value: &str) -> String {
    value.trim_matches('\u{feff}').trim().to_ascii_uppercase()
}

fn header_map(headers: &[String]) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(idx, col)| (normalize_header(col), idx))
        .collect()
}

fn column_index(header: &HashMap<String, usize>, names: &[&str]) -> Option<usize> {
    names
        .iter()
        .find_map(|name| header.get(&normalize_header(name)).copied())
}

fn require_column(
    header: &HashMap<String, usize>,
    column: &str,
    path: &Path,
) -> Result<usize, BioMcpError> {
    header
        .get(&normalize_header(column))
        .copied()
        .ok_or_else(|| BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Missing required column '{column}' in {}", path.display()),
            suggestion: "Use a valid cBioPortal data file for this study.".to_string(),
        })
}

struct TsvReader {
    headers: Vec<String>,
    lines: Lines<BufReader<File>>,
}

impl TsvReader {
    fn open(path: &Path) -> Result<Self, BioMcpError> {
        let file = File::open(path).map_err(|_| BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("Missing required file: {}", path.display()),
            suggestion: "Check BIOMCP_STUDY_DIR and dataset contents.".to_string(),
        })?;

        let mut lines = BufReader::new(file).lines();
        let mut header = None;
        for next in lines.by_ref() {
            let line = next?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            header = Some(
                line.split('\t')
                    .map(|v| v.trim_end_matches('\r').to_string())
                    .collect::<Vec<_>>(),
            );
            break;
        }

        let headers = header.ok_or_else(|| BioMcpError::SourceUnavailable {
            source_name: SOURCE_NAME.to_string(),
            reason: format!("No header found in {}", path.display()),
            suggestion: "Use a valid non-empty TSV file.".to_string(),
        })?;

        Ok(Self { headers, lines })
    }

    fn next_row(&mut self) -> Result<Option<Vec<String>>, BioMcpError> {
        for next in self.lines.by_ref() {
            let line = next?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let mut row = line
                .split('\t')
                .map(|v| v.trim_end_matches('\r').to_string())
                .collect::<Vec<_>>();
            if row.len() < self.headers.len() {
                row.resize(self.headers.len(), String::new());
            }
            return Ok(Some(row));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestStudyDir {
        root: PathBuf,
    }

    impl TestStudyDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "biomcp-study-source-test-{name}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create root");
            Self { root }
        }

        fn study_path(&self, study_id: &str) -> PathBuf {
            let path = self.root.join(study_id);
            fs::create_dir_all(&path).expect("create study dir");
            path
        }

        fn write_file(&self, path: &Path, content: &str) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, content).expect("write test file");
        }
    }

    impl Drop for TestStudyDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_minimal_clinical_samples(study_dir: &Path, rows: &[&str]) {
        let mut content = String::from(
            "# comment 1\n# comment 2\nPATIENT_ID\tSAMPLE_ID\tCANCER_TYPE\tCANCER_TYPE_DETAILED\tONCOTREE_CODE\n",
        );
        for row in rows {
            content.push_str(row);
            content.push('\n');
        }
        fs::write(study_dir.join("data_clinical_sample.txt"), content)
            .expect("write clinical sample");
    }

    fn write_clinical_patients(study_dir: &Path, rows: &[&str]) {
        let mut content = String::from(
            "# comment 1\n# comment 2\nPATIENT_ID\tOS_STATUS\tOS_MONTHS\tDFS_STATUS\tDFS_MONTHS\tPFS_STATUS\tPFS_MONTHS\tDSS_STATUS\tDSS_MONTHS\n",
        );
        for row in rows {
            content.push_str(row);
            content.push('\n');
        }
        fs::write(study_dir.join("data_clinical_patient.txt"), content)
            .expect("write clinical patient");
    }

    fn write_filter_fixture(study_dir: &Path) {
        fs::write(
            study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\n\
TP53\tS1\tMissense_Mutation\tp.R175H\n\
TP53\tS2\tMissense_Mutation\tp.R248Q\n\
TP53\tS3\tNonsense_Mutation\tp.R213*\n\
KRAS\tS4\tMissense_Mutation\tp.G12D\n",
        )
        .expect("write mutations");
        write_minimal_clinical_samples(
            study_dir,
            &[
                "P1\tS1\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P2\tS2\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P3\tS3\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P4\tS4\tPancreatic Cancer\tPancreatic Ductal Adenocarcinoma\tPAAD",
            ],
        );
        fs::write(
            study_dir.join("data_cna.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\tS4\n\
ERBB2\t2064\t0\t2\t2\t2\n\
PTEN\t5728\t-2\t0\t0\tNA\n",
        )
        .expect("write cna");
        fs::write(
            study_dir.join("data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\tS4\n\
MYC\t4609\t1.5\t0.2\t2.0\t0.5\n\
ERBB2\t2064\t2.1\t1.0\t3.4\tbad\n",
        )
        .expect("write expression");
    }

    #[test]
    fn list_studies_reads_meta_and_data_flags() {
        let fixture = TestStudyDir::new("list-studies");
        let study_dir = fixture.study_path("demo_study");
        fixture.write_file(
            &study_dir.join("meta_study.txt"),
            "cancer_study_identifier: demo_study\nname: Demo Study\nshort_name: Demo\ntype_of_cancer: mixed\ncitation: Demo et al.\npmid: 12345\n",
        );
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        );
        fixture.write_file(
            &study_dir.join("data_cna.txt"),
            "Hugo_Symbol\tS1\nTP53\t1\n",
        );
        fixture.write_file(
            &study_dir.join("data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\nTP53\t7157\t0.3\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &["P1\tS1\tLung Cancer\tLung Adenocarcinoma\tLUAD"],
        );

        let studies = list_studies(&fixture.root).expect("list studies");
        assert_eq!(studies.len(), 1);
        let study = &studies[0];
        assert_eq!(study.study_id, "demo_study");
        assert_eq!(study.meta.name, "Demo Study");
        assert_eq!(study.meta.short_name.as_deref(), Some("Demo"));
        assert_eq!(study.meta.cancer_type.as_deref(), Some("mixed"));
        assert_eq!(study.meta.citation.as_deref(), Some("Demo et al."));
        assert_eq!(study.meta.pmid.as_deref(), Some("12345"));
        assert!(study.has_mutations);
        assert!(study.has_cna);
        assert!(study.has_expression);
        assert!(study.has_clinical_sample);
    }

    #[test]
    fn mutation_frequency_counts_records_samples_and_top_buckets() {
        let fixture = TestStudyDir::new("mutation-frequency");
        let study_dir = fixture.study_path("mf_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "#metadata line\nHugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nTP53\tS1\tNonsense_Mutation\tp.R196*\nTP53\tS2\tMissense_Mutation\tp.R248Q\nEGFR\tS3\tMissense_Mutation\tp.L858R\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P2\tS2\tLung Cancer\tLung Squamous Cell Carcinoma\tLUSC",
                "P3\tS3\tLung Cancer\tLung Adenocarcinoma\tLUAD",
            ],
        );

        let result = mutation_frequency(&study_dir, "tp53").expect("mutation frequency");
        assert_eq!(result.study_id, "mf_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.mutation_count, 3);
        assert_eq!(result.unique_samples, 2);
        assert_eq!(result.total_samples, 3);
        assert!((result.frequency - (2.0 / 3.0)).abs() < 1e-9);
        assert_eq!(
            result.top_variant_classes,
            vec![
                ("Missense_Mutation".to_string(), 2),
                ("Nonsense_Mutation".to_string(), 1)
            ]
        );
        assert_eq!(
            result.top_protein_changes,
            vec![
                ("p.R175H".to_string(), 1),
                ("p.R196*".to_string(), 1),
                ("p.R248Q".to_string(), 1)
            ]
        );
    }

    #[test]
    fn cna_distribution_supports_header_with_entrez_column() {
        let fixture = TestStudyDir::new("cna-dist");
        let study_dir = fixture.study_path("cna_study");
        fixture.write_file(
            &study_dir.join("data_cna.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\tS4\tS5\nTP53\t7157\t-2\t-1\t0\t1\t2\n",
        );

        let result = cna_distribution(&study_dir, "TP53").expect("cna distribution");
        assert_eq!(result.study_id, "cna_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.total_samples, 5);
        assert_eq!(result.deep_deletion, 1);
        assert_eq!(result.shallow_deletion, 1);
        assert_eq!(result.diploid, 1);
        assert_eq!(result.gain, 1);
        assert_eq!(result.amplification, 1);
    }

    #[test]
    fn expression_distribution_ignores_na_and_empty_values() {
        let fixture = TestStudyDir::new("expr-dist");
        let study_dir = fixture.study_path("expr_study");
        fixture.write_file(
            &study_dir.join("data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\tS4\tS5\nESR1\t2099\t1.0\tNA\t\t2.0\t3.0\n",
        );

        let result = expression_distribution(&study_dir, "ESR1").expect("expression distribution");
        assert_eq!(result.study_id, "expr_study");
        assert_eq!(result.gene, "ESR1");
        assert_eq!(
            result.file,
            "data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"
        );
        assert_eq!(result.sample_count, 3);
        assert!((result.mean - 2.0).abs() < 1e-9);
        assert!((result.median - 2.0).abs() < 1e-9);
        assert!((result.min - 1.0).abs() < 1e-9);
        assert!((result.max - 3.0).abs() < 1e-9);
        assert!((result.q1 - 1.5).abs() < 1e-9);
        assert!((result.q3 - 2.5).abs() < 1e-9);
    }

    #[test]
    fn cna_values_by_sample_reads_matrix_rows_and_optional_entrez_column() {
        let fixture = TestStudyDir::new("filter-cna-map");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let values = cna_values_by_sample(&study_dir, "erbb2").expect("cna values");
        assert_eq!(values.len(), 4);
        assert_eq!(values.get("S1"), Some(&0));
        assert_eq!(values.get("S2"), Some(&2));
        assert_eq!(values.get("S3"), Some(&2));
        assert_eq!(values.get("S4"), Some(&2));
    }

    #[test]
    fn clinical_column_values_reads_trimmed_clinical_values() {
        let fixture = TestStudyDir::new("filter-clinical-map");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let values = clinical_column_values(&study_dir, "CANCER_TYPE").expect("clinical values");
        assert_eq!(values.len(), 4);
        assert_eq!(values.get("S1").map(String::as_str), Some("Breast Cancer"));
        assert_eq!(values.get("S2").map(String::as_str), Some("Lung Cancer"));
        assert_eq!(
            values.get("S4").map(String::as_str),
            Some("Pancreatic Cancer")
        );
    }

    #[test]
    fn filter_samples_intersects_mutation_and_cna_criteria() {
        let fixture = TestStudyDir::new("filter-mut-cna");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let result = filter_samples(
            &study_dir,
            &[
                SourceFilterCriterion::Mutated("TP53".to_string()),
                SourceFilterCriterion::Amplified("ERBB2".to_string()),
            ],
        )
        .expect("filter result");

        assert_eq!(result.study_id, "filter_study");
        assert_eq!(result.criteria.len(), 2);
        assert_eq!(result.criteria[0].description, "mutated TP53");
        assert_eq!(result.criteria[0].matched_count, 3);
        assert_eq!(result.criteria[1].description, "amplified ERBB2");
        assert_eq!(result.criteria[1].matched_count, 3);
        assert_eq!(result.total_study_samples, Some(4));
        assert_eq!(result.matched_count, 2);
        assert_eq!(result.matched_sample_ids, vec!["S2", "S3"]);
    }

    #[test]
    fn filter_samples_intersects_mutation_and_clinical_criteria() {
        let fixture = TestStudyDir::new("filter-mut-clinical");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let result = filter_samples(
            &study_dir,
            &[
                SourceFilterCriterion::Mutated("TP53".to_string()),
                SourceFilterCriterion::CancerType(" breast cancer ".to_string()),
            ],
        )
        .expect("filter result");

        assert_eq!(result.matched_count, 2);
        assert_eq!(result.matched_sample_ids, vec!["S1", "S3"]);
    }

    #[test]
    fn filter_samples_intersects_three_way_criteria() {
        let fixture = TestStudyDir::new("filter-three-way");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let result = filter_samples(
            &study_dir,
            &[
                SourceFilterCriterion::Mutated("TP53".to_string()),
                SourceFilterCriterion::Amplified("ERBB2".to_string()),
                SourceFilterCriterion::ExpressionAbove("MYC".to_string(), 1.0),
            ],
        )
        .expect("filter result");

        assert_eq!(result.criteria[2].description, "expression > 1 for MYC");
        assert_eq!(result.criteria[2].matched_count, 2);
        assert_eq!(result.matched_count, 1);
        assert_eq!(result.matched_sample_ids, vec!["S3"]);
    }

    #[test]
    fn filter_samples_reports_empty_intersection_and_single_criterion_results() {
        let fixture = TestStudyDir::new("filter-empty");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let single = filter_samples(
            &study_dir,
            &[SourceFilterCriterion::Deleted("PTEN".to_string())],
        )
        .expect("single criterion");
        assert_eq!(single.matched_count, 1);
        assert_eq!(single.matched_sample_ids, vec!["S1"]);

        let no_matches = filter_samples(
            &study_dir,
            &[
                SourceFilterCriterion::Deleted("PTEN".to_string()),
                SourceFilterCriterion::CancerType("Lung Cancer".to_string()),
            ],
        )
        .expect("no matches");
        assert_eq!(no_matches.matched_count, 0);
        assert!(no_matches.matched_sample_ids.is_empty());
    }

    #[test]
    fn filter_samples_treats_missing_gene_rows_as_empty_sets() {
        let fixture = TestStudyDir::new("filter-missing-gene");
        let study_dir = fixture.study_path("filter_study");
        write_filter_fixture(&study_dir);

        let result = filter_samples(
            &study_dir,
            &[
                SourceFilterCriterion::Amplified("ALK".to_string()),
                SourceFilterCriterion::ExpressionAbove("FGFR1".to_string(), 0.5),
            ],
        )
        .expect("missing gene rows should become empty sets");

        assert_eq!(result.criteria.len(), 2);
        assert_eq!(result.criteria[0].matched_count, 0);
        assert_eq!(result.criteria[1].matched_count, 0);
        assert_eq!(result.matched_count, 0);
        assert!(result.matched_sample_ids.is_empty());
    }

    #[test]
    fn filter_samples_propagates_missing_required_files_and_columns() {
        let fixture = TestStudyDir::new("filter-missing-inputs");
        let missing_cna = fixture.study_path("missing_cna");
        fs::write(
            missing_cna.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        )
        .expect("write mutations");

        let err = filter_samples(
            &missing_cna,
            &[SourceFilterCriterion::Amplified("ERBB2".to_string())],
        )
        .expect_err("missing cna file should fail");
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));

        let missing_column = fixture.study_path("missing_column");
        fs::write(
            missing_column.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        )
        .expect("write mutations");
        fs::write(
            missing_column.join("data_clinical_sample.txt"),
            "PATIENT_ID\tSAMPLE_ID\nP1\tS1\n",
        )
        .expect("write clinical");

        let err = filter_samples(
            &missing_column,
            &[SourceFilterCriterion::CancerType(
                "Breast Cancer".to_string(),
            )],
        )
        .expect_err("missing CANCER_TYPE should fail");
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    }

    #[test]
    fn fisher_exact_two_tailed_matches_reference_tables() {
        let cases = [
            ((1, 1, 1, 2), 1.0),
            ((1, 9, 11, 3), 0.002759456185220083),
            ((0, 5, 5, 0), 0.007936507936507936),
            ((10, 0, 0, 10), 1.082508822446903e-05),
            ((1, 1, 1, 0), 1.0),
        ];

        for ((a, b, c, d), expected) in cases {
            let log_fact = build_log_factorial(a + b + c + d);
            let actual = fisher_exact_two_tailed(a, b, c, d, &log_fact);
            assert!(
                (actual - expected).abs() < 1e-6,
                "expected fisher_exact_two_tailed([{a}, {b}, {c}, {d}]) ~= {expected}, got {actual}"
            );
        }
    }

    #[test]
    fn fisher_exact_two_tailed_returns_one_for_zero_total_table() {
        let log_fact = build_log_factorial(0);
        let actual = fisher_exact_two_tailed(0, 0, 0, 0, &log_fact);
        assert_eq!(actual, 1.0);
    }

    #[test]
    fn co_occurrence_computes_pair_counts() {
        let fixture = TestStudyDir::new("co-occur");
        let study_dir = fixture.study_path("co_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nKRAS\tS1\tMissense_Mutation\tp.G12D\nTP53\tS2\tMissense_Mutation\tp.R248Q\nKRAS\tS3\tMissense_Mutation\tp.G12V\nEGFR\tS4\tMissense_Mutation\tp.L858R\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P2\tS2\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P3\tS3\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P4\tS4\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P5\tS5\tLung Cancer\tLung Adenocarcinoma\tLUAD",
            ],
        );

        let result = co_occurrence(&study_dir, &["TP53".into(), "KRAS".into()])
            .expect("co-occurrence result");
        assert_eq!(result.study_id, "co_study");
        assert_eq!(result.total_samples, 5);
        assert_eq!(
            result.sample_universe_basis,
            SampleUniverseBasis::ClinicalSampleFile
        );
        assert_eq!(result.pairs.len(), 1);
        let pair = &result.pairs[0];
        assert_eq!(pair.gene_a, "TP53");
        assert_eq!(pair.gene_b, "KRAS");
        assert_eq!(pair.both_mutated, 1);
        assert_eq!(pair.a_only, 1);
        assert_eq!(pair.b_only, 1);
        assert_eq!(pair.neither, 2);
        assert!(pair.log_odds_ratio.is_some());
        assert!(pair.p_value.is_some());
        assert!((pair.p_value.expect("p-value") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn co_occurrence_falls_back_to_mutation_observed_samples_without_clinical_file() {
        let fixture = TestStudyDir::new("co-occur-no-clinical");
        let study_dir = fixture.study_path("co_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nTP53\tS2\tMissense_Mutation\tp.R248Q\nKRAS\tS2\tMissense_Mutation\tp.G12D\nKRAS\tS3\tMissense_Mutation\tp.G12V\n",
        );

        let result = co_occurrence(&study_dir, &["TP53".into(), "KRAS".into()])
            .expect("co-occurrence result");
        assert_eq!(result.total_samples, 3);
        assert_eq!(
            result.sample_universe_basis,
            SampleUniverseBasis::MutationObserved
        );
        let pair = &result.pairs[0];
        assert_eq!(pair.both_mutated, 1);
        assert_eq!(pair.a_only, 1);
        assert_eq!(pair.b_only, 1);
        assert_eq!(pair.neither, 0);
        assert!(pair.p_value.is_some());
        assert!((pair.p_value.expect("p-value") - 1.0).abs() < 1e-9);
    }

    #[test]
    fn missing_required_file_returns_source_unavailable() {
        let fixture = TestStudyDir::new("missing-file");
        let study_dir = fixture.study_path("missing_study");
        let err = mutation_frequency(&study_dir, "TP53").expect_err("missing file should fail");
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    }

    #[test]
    fn cohort_by_mutation_classifies_patients_and_samples() {
        let fixture = TestStudyDir::new("cohort-split");
        let study_dir = fixture.study_path("cohort_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nEGFR\tS3\tMissense_Mutation\tp.L858R\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P1\tS2\tLung Cancer\tLung Adenocarcinoma\tLUAD",
                "P2\tS3\tLung Cancer\tLung Adenocarcinoma\tLUAD",
            ],
        );

        let result = cohort_by_mutation(&study_dir, "tp53").expect("cohort split");
        assert_eq!(result.study_id, "cohort_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.total_samples, 3);
        assert_eq!(result.total_patients, 2);
        assert_eq!(result.mutant_patients.len(), 1);
        assert!(result.mutant_patients.contains("P1"));
        assert_eq!(result.wildtype_patients.len(), 1);
        assert!(result.wildtype_patients.contains("P2"));
        assert_eq!(result.mutant_samples.len(), 2);
        assert!(result.mutant_samples.contains("S1"));
        assert!(result.mutant_samples.contains("S2"));
        assert_eq!(result.wildtype_samples.len(), 1);
        assert!(result.wildtype_samples.contains("S3"));
    }

    #[test]
    fn patient_survival_data_requires_canonical_columns_and_filters_invalid_rows() {
        let fixture = TestStudyDir::new("patient-survival");
        let study_dir = fixture.study_path("survival_study");
        write_clinical_patients(
            &study_dir,
            &[
                "P1\t1:DECEASED\t10\t1:Recurred\t8\t1:Progressed\t7\t1:Died of disease\t10",
                "P2\t0:LIVING\t24\t0:DiseaseFree\t22\t0:No progression\t20\t0:Alive\t24",
                "P3\t0:LIVING\tNA\t0:DiseaseFree\t14\t0:No progression\t12\t0:Alive\t18",
                "P4\tUNKNOWN\t12\t0:DiseaseFree\t16\t0:No progression\t15\t0:Alive\t12",
            ],
        );

        let result = patient_survival_data(&study_dir, "os").expect("survival records");
        assert_eq!(result.len(), 2);
        assert!(matches!(
            result.get("P1").map(|row| row.status),
            Some(SurvivalStatus::Event)
        ));
        assert_eq!(result.get("P1").map(|row| row.months), Some(10.0));
        assert!(matches!(
            result.get("P2").map(|row| row.status),
            Some(SurvivalStatus::Censored)
        ));
        assert!(!result.contains_key("P3"));
        assert!(!result.contains_key("P4"));
    }

    #[test]
    fn kaplan_meier_estimate_uses_event_times_and_landmarks() {
        let records = [
            PatientSurvivalRecord {
                patient_id: "P1".to_string(),
                status: SurvivalStatus::Event,
                months: 10.0,
            },
            PatientSurvivalRecord {
                patient_id: "P2".to_string(),
                status: SurvivalStatus::Censored,
                months: 12.0,
            },
            PatientSurvivalRecord {
                patient_id: "P3".to_string(),
                status: SurvivalStatus::Event,
                months: 20.0,
            },
            PatientSurvivalRecord {
                patient_id: "P4".to_string(),
                status: SurvivalStatus::Censored,
                months: 30.0,
            },
        ];
        let record_refs = records.iter().collect::<Vec<_>>();

        let estimate = kaplan_meier(&record_refs);
        assert_eq!(estimate.km_median_months, Some(20.0));
        assert_eq!(estimate.survival_1yr, Some(0.75));
        assert_eq!(estimate.survival_3yr, Some(0.375));
        assert_eq!(estimate.survival_5yr, Some(0.375));
        assert_eq!(
            estimate.curve_points,
            vec![(0.0, 1.0), (10.0, 0.75), (20.0, 0.375), (30.0, 0.375)]
        );
    }

    #[test]
    fn kaplan_meier_estimate_without_events_returns_flat_curve() {
        let records = [
            PatientSurvivalRecord {
                patient_id: "P1".to_string(),
                status: SurvivalStatus::Censored,
                months: 12.0,
            },
            PatientSurvivalRecord {
                patient_id: "P2".to_string(),
                status: SurvivalStatus::Censored,
                months: 24.0,
            },
        ];
        let record_refs = records.iter().collect::<Vec<_>>();

        let estimate = kaplan_meier(&record_refs);
        assert_eq!(estimate.km_median_months, None);
        assert_eq!(estimate.survival_1yr, Some(1.0));
        assert_eq!(estimate.survival_3yr, Some(1.0));
        assert_eq!(estimate.survival_5yr, Some(1.0));
        assert_eq!(estimate.curve_points, vec![(0.0, 1.0), (24.0, 1.0)]);
    }

    #[test]
    fn log_rank_two_group_is_defined_when_only_one_group_has_events() {
        let group_a = [
            PatientSurvivalRecord {
                patient_id: "A1".to_string(),
                status: SurvivalStatus::Event,
                months: 5.0,
            },
            PatientSurvivalRecord {
                patient_id: "A2".to_string(),
                status: SurvivalStatus::Event,
                months: 10.0,
            },
            PatientSurvivalRecord {
                patient_id: "A3".to_string(),
                status: SurvivalStatus::Event,
                months: 15.0,
            },
        ];
        let group_b = [
            PatientSurvivalRecord {
                patient_id: "B1".to_string(),
                status: SurvivalStatus::Censored,
                months: 5.0,
            },
            PatientSurvivalRecord {
                patient_id: "B2".to_string(),
                status: SurvivalStatus::Censored,
                months: 10.0,
            },
            PatientSurvivalRecord {
                patient_id: "B3".to_string(),
                status: SurvivalStatus::Censored,
                months: 15.0,
            },
        ];
        let group_a_refs = group_a.iter().collect::<Vec<_>>();
        let group_b_refs = group_b.iter().collect::<Vec<_>>();

        let p_value = log_rank_two_group(&group_a_refs, &group_b_refs).expect("log-rank p-value");
        assert!((p_value - 0.08326451666355043).abs() < 1e-6);
    }

    #[test]
    fn mann_whitney_u_test_handles_ties_and_smaller_u_statistic() {
        let result =
            mann_whitney_u_test(&[1.0, 2.0, 2.0, 5.0], &[2.0, 3.0, 4.0]).expect("mw result");
        assert!((result.u_statistic - 4.0).abs() < 1e-9);
        assert!((result.p_value - 0.5820796519295022).abs() < 1e-6);
    }

    #[test]
    fn mann_whitney_u_test_returns_none_when_all_values_are_identical() {
        assert!(mann_whitney_u_test(&[1.0, 1.0, 1.0], &[1.0, 1.0]).is_none());
    }

    #[test]
    fn survival_by_mutation_returns_group_aggregates_for_analyzable_patients() {
        let fixture = TestStudyDir::new("survival-by-mutation");
        let study_dir = fixture.study_path("survival_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P1\tS2\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P2\tS3\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P3\tS4\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
            ],
        );
        write_clinical_patients(
            &study_dir,
            &[
                "P1\t1:DECEASED\t12\t1:Recurred\t8\t1:Progressed\t7\t1:Died of disease\t12",
                "P2\t0:LIVING\t20\t0:DiseaseFree\t18\t0:No progression\t15\t0:Alive\t20",
                "P3\t1:DECEASED\tNA\t1:Recurred\t10\t1:Progressed\t8\t1:Died of disease\t11",
            ],
        );

        let result = survival_by_mutation(&study_dir, "TP53", "OS").expect("survival summary");
        assert_eq!(result.study_id, "survival_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.endpoint, "OS");
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].n_patients, 1);
        assert_eq!(result.groups[0].n_events, 1);
        assert_eq!(result.groups[0].n_censored, 0);
        assert_eq!(result.groups[0].km_median_months, Some(12.0));
        assert_eq!(result.groups[0].survival_1yr, Some(0.0));
        assert_eq!(result.groups[0].survival_3yr, Some(0.0));
        assert_eq!(result.groups[0].survival_5yr, Some(0.0));
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].n_patients, 1);
        assert_eq!(result.groups[1].n_events, 0);
        assert_eq!(result.groups[1].n_censored, 1);
        assert_eq!(result.groups[1].km_median_months, None);
        assert_eq!(result.groups[1].survival_1yr, Some(1.0));
        assert_eq!(result.groups[1].survival_3yr, Some(1.0));
        assert_eq!(result.groups[1].survival_5yr, Some(1.0));
        assert!((result.log_rank_p.expect("log-rank p-value") - 0.31731050786291415).abs() < 1e-6);
    }

    #[test]
    fn compare_expression_by_mutation_summarizes_group_distributions() {
        let fixture = TestStudyDir::new("compare-expression");
        let study_dir = fixture.study_path("compare_expr_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P1\tS2\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P2\tS3\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
            ],
        );
        fixture.write_file(
            &study_dir.join("data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\nERBB2\t2064\t2.0\t4.0\t1.0\n",
        );

        let result = compare_expression_by_mutation(&study_dir, "TP53", "ERBB2")
            .expect("expression compare");
        assert_eq!(result.study_id, "compare_expr_study");
        assert_eq!(result.stratify_gene, "TP53");
        assert_eq!(result.target_gene, "ERBB2");
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].sample_count, 2);
        assert!((result.groups[0].mean - 3.0).abs() < 1e-9);
        assert!((result.groups[0].median - 3.0).abs() < 1e-9);
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].sample_count, 1);
        assert!((result.groups[1].mean - 1.0).abs() < 1e-9);
        assert_eq!(result.mann_whitney_u, Some(0.0));
        assert!(
            (result.mann_whitney_p.expect("mann-whitney p-value") - 0.5402913746074199).abs()
                < 1e-6
        );
    }

    #[test]
    fn compare_mutations_by_mutation_counts_unique_samples_in_each_group() {
        let fixture = TestStudyDir::new("compare-mutations");
        let study_dir = fixture.study_path("compare_mut_study");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nPIK3CA\tS2\tMissense_Mutation\tp.H1047R\nPIK3CA\tS3\tMissense_Mutation\tp.E545K\n",
        );
        write_minimal_clinical_samples(
            &study_dir,
            &[
                "P1\tS1\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P1\tS2\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
                "P2\tS3\tBreast Cancer\tBreast Invasive Carcinoma\tBRCA",
            ],
        );

        let result =
            compare_mutations_by_mutation(&study_dir, "TP53", "PIK3CA").expect("mutation compare");
        assert_eq!(result.study_id, "compare_mut_study");
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].sample_count, 2);
        assert_eq!(result.groups[0].mutated_count, 1);
        assert!((result.groups[0].mutation_rate - 0.5).abs() < 1e-9);
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].sample_count, 1);
        assert_eq!(result.groups[1].mutated_count, 1);
        assert!((result.groups[1].mutation_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cohort_commands_require_clinical_sample_file_instead_of_falling_back() {
        let fixture = TestStudyDir::new("cohort-requires-clinical-sample");
        let study_dir = fixture.study_path("missing_clinical_sample");
        fixture.write_file(
            &study_dir.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\n",
        );

        let err = cohort_by_mutation(&study_dir, "TP53").expect_err("missing clinical file");
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
        assert!(err.to_string().contains("data_clinical_sample.txt"));
    }
}
