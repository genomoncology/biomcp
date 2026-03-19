use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

use crate::error::BioMcpError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudyInfo {
    pub study_id: String,
    pub name: String,
    pub cancer_type: Option<String>,
    pub citation: Option<String>,
    pub sample_count: Option<usize>,
    pub available_data: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudyDownloadCatalog {
    pub study_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudyDownloadResult {
    pub study_id: String,
    pub path: String,
    pub downloaded: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoOccurrenceResult {
    pub study_id: String,
    pub genes: Vec<String>,
    pub total_samples: usize,
    pub sample_universe_basis: SampleUniverseBasis,
    pub pairs: Vec<CoOccurrencePair>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SampleUniverseBasis {
    ClinicalSampleFile,
    MutationObserved,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "query_type", content = "result", rename_all = "snake_case")]
pub enum StudyQueryResult {
    MutationFrequency(MutationFrequencyResult),
    CnaDistribution(CnaDistributionResult),
    ExpressionDistribution(ExpressionDistributionResult),
}

#[derive(Debug, Clone, Copy)]
pub enum StudyQueryType {
    Mutations,
    Cna,
    Expression,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CohortResult {
    pub study_id: String,
    pub gene: String,
    pub stratification: String,
    pub mutant_samples: usize,
    pub wildtype_samples: usize,
    pub mutant_patients: usize,
    pub wildtype_patients: usize,
    pub total_samples: usize,
    pub total_patients: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalEndpoint {
    Os,
    Dfs,
    Pfs,
    Dss,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SurvivalGroupResult {
    pub group_name: String,
    pub n_patients: usize,
    pub n_events: usize,
    pub n_censored: usize,
    pub km_median_months: Option<f64>,
    pub survival_1yr: Option<f64>,
    pub survival_3yr: Option<f64>,
    pub survival_5yr: Option<f64>,
    pub event_rate: f64,
    #[serde(skip, default)]
    pub km_curve_points: Vec<(f64, f64)>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SurvivalResult {
    pub study_id: String,
    pub gene: String,
    pub endpoint: SurvivalEndpoint,
    pub groups: Vec<SurvivalGroupResult>,
    pub log_rank_p: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExpressionComparisonResult {
    pub study_id: String,
    pub stratify_gene: String,
    pub target_gene: String,
    pub groups: Vec<ExpressionGroupStats>,
    pub mann_whitney_u: Option<f64>,
    pub mann_whitney_p: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutationGroupStats {
    pub group_name: String,
    pub sample_count: usize,
    pub mutated_count: usize,
    pub mutation_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutationComparisonResult {
    pub study_id: String,
    pub stratify_gene: String,
    pub target_gene: String,
    pub groups: Vec<MutationGroupStats>,
}

#[derive(Debug, Clone)]
pub enum FilterCriterion {
    Mutated(String),
    Amplified(String),
    Deleted(String),
    ExpressionAbove(String, f64),
    ExpressionBelow(String, f64),
    CancerType(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilterCriterionSummary {
    pub description: String,
    pub matched_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilterResult {
    pub study_id: String,
    pub criteria: Vec<FilterCriterionSummary>,
    pub total_study_samples: Option<usize>,
    pub matched_count: usize,
    pub matched_sample_ids: Vec<String>,
}

impl StudyQueryType {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "mutations" | "mutation" => Ok(Self::Mutations),
            "cna" | "copy_number" | "copy-number" => Ok(Self::Cna),
            "expression" | "expr" => Ok(Self::Expression),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown study query type '{other}'. Expected: mutations, cna, expression."
            ))),
        }
    }
}

impl SurvivalEndpoint {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "os" | "overall" | "overall_survival" => Ok(Self::Os),
            "dfs" | "disease_free" => Ok(Self::Dfs),
            "pfs" | "progression_free" => Ok(Self::Pfs),
            "dss" | "disease_specific" => Ok(Self::Dss),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown survival endpoint '{other}'. Expected: os, dfs, pfs, dss."
            ))),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Os => "OS",
            Self::Dfs => "DFS",
            Self::Pfs => "PFS",
            Self::Dss => "DSS",
        }
    }

    pub fn status_column(&self) -> &'static str {
        match self {
            Self::Os => "OS_STATUS",
            Self::Dfs => "DFS_STATUS",
            Self::Pfs => "PFS_STATUS",
            Self::Dss => "DSS_STATUS",
        }
    }

    pub fn months_column(&self) -> &'static str {
        match self {
            Self::Os => "OS_MONTHS",
            Self::Dfs => "DFS_MONTHS",
            Self::Pfs => "PFS_MONTHS",
            Self::Dss => "DSS_MONTHS",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Os => "Overall Survival",
            Self::Dfs => "Disease-Free Survival",
            Self::Pfs => "Progression-Free Survival",
            Self::Dss => "Disease-Specific Survival",
        }
    }
}

pub async fn list_studies() -> Result<Vec<StudyInfo>, BioMcpError> {
    let root = crate::sources::cbioportal_study::resolve_study_root();
    run_blocking(move || {
        let studies = crate::sources::cbioportal_study::list_studies(&root)?;
        let mut out = Vec::with_capacity(studies.len());
        for study in studies {
            let mut available_data = Vec::new();
            if study.has_mutations {
                available_data.push("mutations".to_string());
            }
            if study.has_cna {
                available_data.push("cna".to_string());
            }
            if study.has_expression {
                available_data.push("expression".to_string());
            }
            if study.has_clinical_sample {
                available_data.push("clinical".to_string());
            }

            out.push(StudyInfo {
                study_id: study.study_id,
                name: study.meta.name,
                cancer_type: study.meta.cancer_type,
                citation: study.meta.citation,
                sample_count: clinical_sample_count(&study.path),
                available_data,
            });
        }
        Ok(out)
    })
    .await
}

pub async fn list_downloadable_studies() -> Result<StudyDownloadCatalog, BioMcpError> {
    let client = crate::sources::cbioportal_download::CBioPortalDownloadClient::new()?;
    let study_ids = client.list_study_ids().await?;
    Ok(StudyDownloadCatalog { study_ids })
}

pub async fn download_study(study_id: &str) -> Result<StudyDownloadResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let client = crate::sources::cbioportal_download::CBioPortalDownloadClient::new()?;
    Ok(client
        .download_study(
            &study_id,
            &crate::sources::cbioportal_study::resolve_study_root(),
        )
        .await?
        .into())
}

pub async fn query_study(
    study_id: &str,
    gene: &str,
    query_type: StudyQueryType,
) -> Result<StudyQueryResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let gene = normalize_gene(gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        match query_type {
            StudyQueryType::Mutations => Ok(StudyQueryResult::MutationFrequency(
                crate::sources::cbioportal_study::mutation_frequency(&study_dir, &gene)?.into(),
            )),
            StudyQueryType::Cna => Ok(StudyQueryResult::CnaDistribution(
                crate::sources::cbioportal_study::cna_distribution(&study_dir, &gene)?.into(),
            )),
            StudyQueryType::Expression => Ok(StudyQueryResult::ExpressionDistribution(
                crate::sources::cbioportal_study::expression_distribution(&study_dir, &gene)?
                    .into(),
            )),
        }
    })
    .await
}

pub async fn expression_values(study_id: &str, gene: &str) -> Result<Vec<f64>, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let gene = normalize_gene(gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        let mut values =
            crate::sources::cbioportal_study::expression_values_by_sample(&study_dir, &gene)?
                .into_values()
                .collect::<Vec<_>>();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Ok(values)
    })
    .await
}

pub async fn compare_expression_values(
    study_id: &str,
    stratify_gene: &str,
    target_gene: &str,
) -> Result<Vec<(String, Vec<f64>)>, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let stratify_gene = normalize_gene(stratify_gene)?;
    let target_gene = normalize_gene(target_gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        let cohort =
            crate::sources::cbioportal_study::build_cohort_split(&study_dir, &stratify_gene)?;
        let values_by_sample = crate::sources::cbioportal_study::expression_values_by_sample(
            &study_dir,
            &target_gene,
        )?;

        let mut mutant_values = cohort
            .mutant_samples
            .iter()
            .filter_map(|sample| values_by_sample.get(sample).copied())
            .collect::<Vec<_>>();
        let mut wildtype_values = cohort
            .wildtype_samples
            .iter()
            .filter_map(|sample| values_by_sample.get(sample).copied())
            .collect::<Vec<_>>();

        mutant_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        wildtype_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        Ok(vec![
            (format!("{stratify_gene}-mutant"), mutant_values),
            (format!("{stratify_gene}-wildtype"), wildtype_values),
        ])
    })
    .await
}

pub async fn co_occurrence(
    study_id: &str,
    genes: &[String],
) -> Result<CoOccurrenceResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let genes = normalize_gene_list(genes)?;
    if genes.len() < 2 || genes.len() > 10 {
        return Err(BioMcpError::InvalidArgument(
            "Study co-occurrence requires 2 to 10 unique genes.".to_string(),
        ));
    }

    let root = crate::sources::cbioportal_study::resolve_study_root();
    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        let result = crate::sources::cbioportal_study::co_occurrence(&study_dir, &genes)?;
        Ok(result.into())
    })
    .await
}

pub async fn filter(
    study_id: &str,
    criteria: Vec<FilterCriterion>,
) -> Result<FilterResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    if criteria.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            filter_required_message().to_string(),
        ));
    }

    let source_criteria = normalize_filter_criteria(criteria)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        let result =
            crate::sources::cbioportal_study::filter_samples(&study_dir, &source_criteria)?;
        Ok(result.into())
    })
    .await
}

pub async fn cohort(study_id: &str, gene: &str) -> Result<CohortResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let gene = normalize_gene(gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        Ok(crate::sources::cbioportal_study::cohort_by_mutation(&study_dir, &gene)?.into())
    })
    .await
}

pub async fn survival(
    study_id: &str,
    gene: &str,
    endpoint: SurvivalEndpoint,
) -> Result<SurvivalResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let gene = normalize_gene(gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();
    let _required_columns = (endpoint.status_column(), endpoint.months_column());

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        Ok(crate::sources::cbioportal_study::survival_by_mutation(
            &study_dir,
            &gene,
            endpoint.code(),
        )?
        .into())
    })
    .await
}

pub async fn compare_expression(
    study_id: &str,
    stratify_gene: &str,
    target_gene: &str,
) -> Result<ExpressionComparisonResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let stratify_gene = normalize_gene(stratify_gene)?;
    let target_gene = normalize_gene(target_gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        Ok(
            crate::sources::cbioportal_study::compare_expression_by_mutation(
                &study_dir,
                &stratify_gene,
                &target_gene,
            )?
            .into(),
        )
    })
    .await
}

pub async fn compare_mutations(
    study_id: &str,
    stratify_gene: &str,
    target_gene: &str,
) -> Result<MutationComparisonResult, BioMcpError> {
    let study_id = normalize_study_id(study_id)?;
    let stratify_gene = normalize_gene(stratify_gene)?;
    let target_gene = normalize_gene(target_gene)?;
    let root = crate::sources::cbioportal_study::resolve_study_root();

    run_blocking(move || {
        let study_dir = resolve_study_dir(&root, &study_id)?;
        Ok(
            crate::sources::cbioportal_study::compare_mutations_by_mutation(
                &study_dir,
                &stratify_gene,
                &target_gene,
            )?
            .into(),
        )
    })
    .await
}

impl From<crate::sources::cbioportal_study::MutationFrequencyResult> for MutationFrequencyResult {
    fn from(value: crate::sources::cbioportal_study::MutationFrequencyResult) -> Self {
        Self {
            study_id: value.study_id,
            gene: value.gene,
            mutation_count: value.mutation_count,
            unique_samples: value.unique_samples,
            total_samples: value.total_samples,
            frequency: value.frequency,
            top_variant_classes: value.top_variant_classes,
            top_protein_changes: value.top_protein_changes,
        }
    }
}

impl From<crate::sources::cbioportal_study::CnaDistributionResult> for CnaDistributionResult {
    fn from(value: crate::sources::cbioportal_study::CnaDistributionResult) -> Self {
        Self {
            study_id: value.study_id,
            gene: value.gene,
            total_samples: value.total_samples,
            deep_deletion: value.deep_deletion,
            shallow_deletion: value.shallow_deletion,
            diploid: value.diploid,
            gain: value.gain,
            amplification: value.amplification,
        }
    }
}

impl From<crate::sources::cbioportal_study::ExpressionDistributionResult>
    for ExpressionDistributionResult
{
    fn from(value: crate::sources::cbioportal_study::ExpressionDistributionResult) -> Self {
        Self {
            study_id: value.study_id,
            gene: value.gene,
            file: value.file,
            sample_count: value.sample_count,
            mean: value.mean,
            median: value.median,
            min: value.min,
            max: value.max,
            q1: value.q1,
            q3: value.q3,
        }
    }
}

impl From<crate::sources::cbioportal_study::CoOccurrencePair> for CoOccurrencePair {
    fn from(value: crate::sources::cbioportal_study::CoOccurrencePair) -> Self {
        Self {
            gene_a: value.gene_a,
            gene_b: value.gene_b,
            both_mutated: value.both_mutated,
            a_only: value.a_only,
            b_only: value.b_only,
            neither: value.neither,
            log_odds_ratio: value.log_odds_ratio,
            p_value: value.p_value,
        }
    }
}

impl From<crate::sources::cbioportal_study::CoOccurrenceResult> for CoOccurrenceResult {
    fn from(value: crate::sources::cbioportal_study::CoOccurrenceResult) -> Self {
        Self {
            study_id: value.study_id,
            genes: value.genes,
            total_samples: value.total_samples,
            sample_universe_basis: value.sample_universe_basis.into(),
            pairs: value.pairs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::sources::cbioportal_study::CohortSplit> for CohortResult {
    fn from(value: crate::sources::cbioportal_study::CohortSplit) -> Self {
        Self {
            study_id: value.study_id,
            gene: value.gene,
            stratification: "mutation".to_string(),
            mutant_samples: value.mutant_samples.len(),
            wildtype_samples: value.wildtype_samples.len(),
            mutant_patients: value.mutant_patients.len(),
            wildtype_patients: value.wildtype_patients.len(),
            total_samples: value.total_samples,
            total_patients: value.total_patients,
        }
    }
}

impl From<crate::sources::cbioportal_study::SampleUniverseBasis> for SampleUniverseBasis {
    fn from(value: crate::sources::cbioportal_study::SampleUniverseBasis) -> Self {
        match value {
            crate::sources::cbioportal_study::SampleUniverseBasis::ClinicalSampleFile => {
                Self::ClinicalSampleFile
            }
            crate::sources::cbioportal_study::SampleUniverseBasis::MutationObserved => {
                Self::MutationObserved
            }
        }
    }
}

impl From<crate::sources::cbioportal_study::SurvivalGroupStats> for SurvivalGroupResult {
    fn from(value: crate::sources::cbioportal_study::SurvivalGroupStats) -> Self {
        Self {
            group_name: value.group_name,
            n_patients: value.n_patients,
            n_events: value.n_events,
            n_censored: value.n_censored,
            km_median_months: value.km_median_months,
            survival_1yr: value.survival_1yr,
            survival_3yr: value.survival_3yr,
            survival_5yr: value.survival_5yr,
            event_rate: value.event_rate,
            km_curve_points: value.km_curve_points,
        }
    }
}

impl From<crate::sources::cbioportal_study::SurvivalByMutationResult> for SurvivalResult {
    fn from(value: crate::sources::cbioportal_study::SurvivalByMutationResult) -> Self {
        Self {
            study_id: value.study_id,
            gene: value.gene,
            endpoint: SurvivalEndpoint::from_flag(&value.endpoint)
                .expect("source survival endpoint should be valid"),
            groups: value.groups.into_iter().map(Into::into).collect(),
            log_rank_p: value.log_rank_p,
        }
    }
}

impl From<crate::sources::cbioportal_study::ExpressionGroupStats> for ExpressionGroupStats {
    fn from(value: crate::sources::cbioportal_study::ExpressionGroupStats) -> Self {
        Self {
            group_name: value.group_name,
            sample_count: value.sample_count,
            mean: value.mean,
            median: value.median,
            min: value.min,
            max: value.max,
            q1: value.q1,
            q3: value.q3,
        }
    }
}

impl From<crate::sources::cbioportal_study::ExpressionComparisonByMutationResult>
    for ExpressionComparisonResult
{
    fn from(value: crate::sources::cbioportal_study::ExpressionComparisonByMutationResult) -> Self {
        Self {
            study_id: value.study_id,
            stratify_gene: value.stratify_gene,
            target_gene: value.target_gene,
            groups: value.groups.into_iter().map(Into::into).collect(),
            mann_whitney_u: value.mann_whitney_u,
            mann_whitney_p: value.mann_whitney_p,
        }
    }
}

impl From<crate::sources::cbioportal_download::StudyInstallResult> for StudyDownloadResult {
    fn from(value: crate::sources::cbioportal_download::StudyInstallResult) -> Self {
        Self {
            study_id: value.study_id,
            path: value.path.display().to_string(),
            downloaded: value.downloaded,
        }
    }
}

impl From<crate::sources::cbioportal_study::MutationGroupStats> for MutationGroupStats {
    fn from(value: crate::sources::cbioportal_study::MutationGroupStats) -> Self {
        Self {
            group_name: value.group_name,
            sample_count: value.sample_count,
            mutated_count: value.mutated_count,
            mutation_rate: value.mutation_rate,
        }
    }
}

impl From<crate::sources::cbioportal_study::MutationComparisonByMutationResult>
    for MutationComparisonResult
{
    fn from(value: crate::sources::cbioportal_study::MutationComparisonByMutationResult) -> Self {
        Self {
            study_id: value.study_id,
            stratify_gene: value.stratify_gene,
            target_gene: value.target_gene,
            groups: value.groups.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::sources::cbioportal_study::SourceFilterCriterionSummary>
    for FilterCriterionSummary
{
    fn from(value: crate::sources::cbioportal_study::SourceFilterCriterionSummary) -> Self {
        Self {
            description: value.description,
            matched_count: value.matched_count,
        }
    }
}

impl From<crate::sources::cbioportal_study::SourceFilterResult> for FilterResult {
    fn from(value: crate::sources::cbioportal_study::SourceFilterResult) -> Self {
        Self {
            study_id: value.study_id,
            criteria: value.criteria.into_iter().map(Into::into).collect(),
            total_study_samples: value.total_study_samples,
            matched_count: value.matched_count,
            matched_sample_ids: value.matched_sample_ids,
        }
    }
}

fn normalize_filter_criteria(
    criteria: Vec<FilterCriterion>,
) -> Result<Vec<crate::sources::cbioportal_study::SourceFilterCriterion>, BioMcpError> {
    criteria
        .into_iter()
        .map(|criterion| match criterion {
            FilterCriterion::Mutated(gene) => Ok(
                crate::sources::cbioportal_study::SourceFilterCriterion::Mutated(normalize_gene(
                    &gene,
                )?),
            ),
            FilterCriterion::Amplified(gene) => Ok(
                crate::sources::cbioportal_study::SourceFilterCriterion::Amplified(normalize_gene(
                    &gene,
                )?),
            ),
            FilterCriterion::Deleted(gene) => Ok(
                crate::sources::cbioportal_study::SourceFilterCriterion::Deleted(normalize_gene(
                    &gene,
                )?),
            ),
            FilterCriterion::ExpressionAbove(gene, threshold) => Ok(
                crate::sources::cbioportal_study::SourceFilterCriterion::ExpressionAbove(
                    normalize_gene(&gene)?,
                    threshold,
                ),
            ),
            FilterCriterion::ExpressionBelow(gene, threshold) => Ok(
                crate::sources::cbioportal_study::SourceFilterCriterion::ExpressionBelow(
                    normalize_gene(&gene)?,
                    threshold,
                ),
            ),
            FilterCriterion::CancerType(value) => {
                let cancer_type = value.trim();
                if cancer_type.is_empty() {
                    return Err(BioMcpError::InvalidArgument(
                        "Cancer type is required.".to_string(),
                    ));
                }
                Ok(
                    crate::sources::cbioportal_study::SourceFilterCriterion::CancerType(
                        cancer_type.to_string(),
                    ),
                )
            }
        })
        .collect()
}

pub(crate) fn filter_required_message() -> &'static str {
    "At least one filter criterion is required. Use one or more of --mutated, --amplified, --deleted, --expression-above, --expression-below, --cancer-type."
}

fn normalize_study_id(study_id: &str) -> Result<String, BioMcpError> {
    let study_id = study_id.trim();
    if study_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Study ID is required.".to_string(),
        ));
    }
    let mut components = Path::new(study_id).components();
    let is_single_segment = matches!(
        (components.next(), components.next()),
        (Some(std::path::Component::Normal(_)), None)
    );
    if !is_single_segment
        || study_id.contains('\\')
        || study_id
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid study ID '{study_id}'. Expected a single identifier such as 'msk_impact_2017'."
        )));
    }
    Ok(study_id.to_string())
}

fn normalize_gene(gene: &str) -> Result<String, BioMcpError> {
    let gene = gene.trim();
    if gene.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene is required.".to_string(),
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
    for gene in genes {
        let normalized = normalize_gene(gene)?;
        if seen.insert(normalized.clone()) {
            out.push(normalized);
        }
    }
    Ok(out)
}

fn resolve_study_dir(root: &Path, study_id: &str) -> Result<std::path::PathBuf, BioMcpError> {
    let studies = crate::sources::cbioportal_study::list_studies(root)?;
    studies
        .into_iter()
        .find(|study| study.study_id.eq_ignore_ascii_case(study_id))
        .map(|study| study.path)
        .ok_or_else(|| BioMcpError::NotFound {
            entity: "study".to_string(),
            id: study_id.to_string(),
            suggestion: "Try listing available studies: biomcp study list".to_string(),
        })
}

fn clinical_sample_count(study_dir: &Path) -> Option<usize> {
    let path = study_dir.join("data_clinical_sample.txt");
    let file = File::open(path).ok()?;
    let mut lines = std::io::BufReader::new(file).lines();

    let mut sample_idx = None;
    for next in lines.by_ref() {
        let line = next.ok()?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let header = line
            .split('\t')
            .map(|value| value.trim().to_ascii_uppercase())
            .collect::<Vec<_>>();
        sample_idx = header.iter().position(|col| col == "SAMPLE_ID");
        break;
    }

    let sample_idx = sample_idx?;
    let mut samples = HashSet::new();
    for next in lines {
        let line = next.ok()?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        let sample = fields.get(sample_idx).copied().unwrap_or("").trim();
        if !sample.is_empty() {
            samples.insert(sample.to_string());
        }
    }
    Some(samples.len())
}

async fn run_blocking<T, F>(work: F) -> Result<T, BioMcpError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, BioMcpError> + Send + 'static,
{
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|err| BioMcpError::Api {
            api: "study".to_string(),
            message: format!("Study query worker failed: {err}"),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::sync::Mutex;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

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
                "biomcp-study-entity-test-{name}-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create root");
            Self { root }
        }
    }

    impl Drop for TestStudyDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, content).expect("write file");
    }

    fn minimal_study_fixture(root: &Path, study_id: &str) {
        let study = root.join(study_id);
        fs::create_dir_all(&study).expect("create study");
        write_file(
            &study.join("meta_study.txt"),
            &format!(
                "cancer_study_identifier: {study_id}\nname: {study_id} name\ntype_of_cancer: mixed\ncitation: demo citation\n"
            ),
        );
        write_file(
            &study.join("data_mutations.txt"),
            "Hugo_Symbol\tTumor_Sample_Barcode\tVariant_Classification\tHGVSp_Short\nTP53\tS1\tMissense_Mutation\tp.R175H\nTP53\tS2\tMissense_Mutation\tp.R248Q\nKRAS\tS2\tMissense_Mutation\tp.G12D\n",
        );
        write_file(
            &study.join("data_clinical_sample.txt"),
            "# comment\nPATIENT_ID\tSAMPLE_ID\tCANCER_TYPE\tCANCER_TYPE_DETAILED\tONCOTREE_CODE\nP1\tS1\tLung Cancer\tLung Adenocarcinoma\tLUAD\nP2\tS2\tLung Cancer\tLung Adenocarcinoma\tLUAD\nP3\tS3\tLung Cancer\tLung Adenocarcinoma\tLUAD\n",
        );
        write_file(
            &study.join("data_cna.txt"),
            "Hugo_Symbol\tS1\tS2\tS3\nTP53\t-2\t0\t2\n",
        );
        write_file(
            &study.join("data_mrna_seq_v2_rsem_zscores_ref_all_samples.txt"),
            "Hugo_Symbol\tEntrez_Gene_Id\tS1\tS2\tS3\nTP53\t7157\t1.0\t2.0\t3.0\nERBB2\t2064\t2.0\t4.0\t1.0\n",
        );
        write_file(
            &study.join("data_clinical_patient.txt"),
            "# comment\nPATIENT_ID\tOS_STATUS\tOS_MONTHS\tDFS_STATUS\tDFS_MONTHS\tPFS_STATUS\tPFS_MONTHS\tDSS_STATUS\tDSS_MONTHS\nP1\t1:DECEASED\t12\t1:Recurred\t8\t1:Progressed\t7\t1:Died of disease\t12\nP2\t0:LIVING\t24\t0:DiseaseFree\t20\t0:No progression\t18\t0:Alive\t24\nP3\t1:DECEASED\tNA\t1:Recurred\t10\t1:Progressed\t8\t1:Died of disease\t10\n",
        );
    }

    #[test]
    fn query_type_from_flag_parses_supported_values() {
        assert!(matches!(
            StudyQueryType::from_flag("mutations").expect("mutations should parse"),
            StudyQueryType::Mutations
        ));
        assert!(matches!(
            StudyQueryType::from_flag("cna").expect("cna should parse"),
            StudyQueryType::Cna
        ));
        assert!(matches!(
            StudyQueryType::from_flag("expression").expect("expression should parse"),
            StudyQueryType::Expression
        ));
    }

    #[test]
    fn query_type_from_flag_rejects_unknown_type() {
        let err = StudyQueryType::from_flag("foo").expect_err("unknown type should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn survival_endpoint_from_flag_parses_supported_values() {
        assert!(matches!(
            SurvivalEndpoint::from_flag("os").expect("os should parse"),
            SurvivalEndpoint::Os
        ));
        assert!(matches!(
            SurvivalEndpoint::from_flag("DFS").expect("dfs should parse"),
            SurvivalEndpoint::Dfs
        ));
        assert!(matches!(
            SurvivalEndpoint::from_flag("progression_free")
                .expect("progression free synonym should parse"),
            SurvivalEndpoint::Pfs
        ));
        assert!(matches!(
            SurvivalEndpoint::from_flag("disease_specific")
                .expect("disease specific synonym should parse"),
            SurvivalEndpoint::Dss
        ));
    }

    #[test]
    fn survival_endpoint_rejects_unknown_value() {
        let err = SurvivalEndpoint::from_flag("foo").expect_err("unknown endpoint should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn normalize_study_id_rejects_path_like_input() {
        let err = normalize_study_id("../demo_study").expect_err("path-like study ID should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("Invalid study ID"));
    }

    #[tokio::test]
    async fn list_studies_returns_available_data() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("list");
        minimal_study_fixture(&fixture.root, "demo_study");
        // SAFETY: tests serialize env var mutation through a process-wide mutex.
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let studies = list_studies().await.expect("list studies");
        assert_eq!(studies.len(), 1);
        assert_eq!(studies[0].study_id, "demo_study");
        assert!(studies[0].available_data.contains(&"mutations".to_string()));
        assert!(studies[0].available_data.contains(&"cna".to_string()));
        assert!(
            studies[0]
                .available_data
                .contains(&"expression".to_string())
        );
        assert_eq!(studies[0].sample_count, Some(3));
    }

    #[tokio::test]
    async fn query_study_mutations_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("query-mutations");
        minimal_study_fixture(&fixture.root, "demo_study");
        // SAFETY: tests serialize env var mutation through a process-wide mutex.
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = query_study("demo_study", "TP53", StudyQueryType::Mutations)
            .await
            .expect("query should pass");
        match result {
            StudyQueryResult::MutationFrequency(result) => {
                assert_eq!(result.study_id, "demo_study");
                assert_eq!(result.gene, "TP53");
                assert_eq!(result.mutation_count, 2);
                assert_eq!(result.unique_samples, 2);
                assert_eq!(result.total_samples, 3);
            }
            other => panic!("unexpected query result: {other:?}"),
        }
    }

    #[tokio::test]
    async fn query_study_unknown_study_returns_not_found() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("unknown-study");
        minimal_study_fixture(&fixture.root, "demo_study");
        // SAFETY: tests serialize env var mutation through a process-wide mutex.
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let err = query_study("missing", "TP53", StudyQueryType::Mutations)
            .await
            .expect_err("unknown study should fail");
        assert!(matches!(err, BioMcpError::NotFound { .. }));
    }

    #[tokio::test]
    async fn co_occurrence_validates_gene_count() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("co-occur-count");
        minimal_study_fixture(&fixture.root, "demo_study");
        // SAFETY: tests serialize env var mutation through a process-wide mutex.
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let err = co_occurrence("demo_study", &["TP53".to_string()])
            .await
            .expect_err("one gene should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn cohort_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("cohort");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = cohort("demo_study", "TP53")
            .await
            .expect("cohort should pass");
        assert_eq!(result.study_id, "demo_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.stratification, "mutation");
        assert_eq!(result.mutant_samples, 2);
        assert_eq!(result.wildtype_samples, 1);
        assert_eq!(result.mutant_patients, 2);
        assert_eq!(result.wildtype_patients, 1);
    }

    #[tokio::test]
    async fn survival_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("survival");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = survival("demo_study", "TP53", SurvivalEndpoint::Os)
            .await
            .expect("survival should pass");
        assert_eq!(result.study_id, "demo_study");
        assert_eq!(result.gene, "TP53");
        assert_eq!(result.endpoint, SurvivalEndpoint::Os);
        assert_eq!(result.groups.len(), 2);
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].n_patients, 2);
        assert_eq!(result.groups[0].n_events, 1);
        assert_eq!(result.groups[0].km_median_months, Some(12.0));
        assert_eq!(result.groups[0].survival_1yr, Some(0.5));
        assert_eq!(result.groups[0].survival_3yr, Some(0.5));
        assert_eq!(result.groups[0].survival_5yr, Some(0.5));
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].n_patients, 0);
        assert_eq!(result.groups[1].km_median_months, None);
        assert_eq!(result.log_rank_p, None);
    }

    #[tokio::test]
    async fn compare_expression_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("compare-expression");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = compare_expression("demo_study", "TP53", "ERBB2")
            .await
            .expect("expression compare should pass");
        assert_eq!(result.study_id, "demo_study");
        assert_eq!(result.stratify_gene, "TP53");
        assert_eq!(result.target_gene, "ERBB2");
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].sample_count, 2);
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].sample_count, 1);
        assert_eq!(result.mann_whitney_u, Some(0.0));
        assert!(
            (result.mann_whitney_p.expect("mann-whitney p-value") - 0.5402913746074199).abs()
                < 1e-6
        );
    }

    #[tokio::test]
    async fn filter_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("filter");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = filter(
            "demo_study",
            vec![
                FilterCriterion::Mutated("tp53".to_string()),
                FilterCriterion::CancerType(" lung cancer ".to_string()),
            ],
        )
        .await
        .expect("filter should pass");

        assert_eq!(result.study_id, "demo_study");
        assert_eq!(result.criteria.len(), 2);
        assert_eq!(result.criteria[0].description, "mutated TP53");
        assert_eq!(result.criteria[0].matched_count, 2);
        assert_eq!(result.criteria[1].description, "cancer type = lung cancer");
        assert_eq!(result.criteria[1].matched_count, 3);
        assert_eq!(result.total_study_samples, Some(3));
        assert_eq!(result.matched_count, 2);
        assert_eq!(result.matched_sample_ids, vec!["S1", "S2"]);
    }

    #[tokio::test]
    async fn filter_validates_empty_criteria() {
        let err = filter("demo_study", Vec::new())
            .await
            .expect_err("empty criteria should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string()
                .contains("At least one filter criterion is required")
        );
    }

    #[tokio::test]
    async fn compare_mutations_round_trips_source_result() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("compare-mutations");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let result = compare_mutations("demo_study", "TP53", "KRAS")
            .await
            .expect("mutation compare should pass");
        assert_eq!(result.study_id, "demo_study");
        assert_eq!(result.stratify_gene, "TP53");
        assert_eq!(result.target_gene, "KRAS");
        assert_eq!(result.groups[0].group_name, "TP53-mutant");
        assert_eq!(result.groups[0].sample_count, 2);
        assert_eq!(result.groups[0].mutated_count, 1);
        assert_eq!(result.groups[1].group_name, "TP53-wildtype");
        assert_eq!(result.groups[1].sample_count, 1);
        assert_eq!(result.groups[1].mutated_count, 0);
    }

    #[tokio::test]
    async fn expression_values_returns_sorted_values() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("expression-values");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let values = expression_values("demo_study", "ERBB2")
            .await
            .expect("raw expression values should load");
        assert_eq!(values, vec![1.0, 2.0, 4.0]);
    }

    #[tokio::test]
    async fn compare_expression_values_returns_mutant_then_wildtype_groups() {
        let _guard = env_lock().lock().await;
        let fixture = TestStudyDir::new("compare-expression-values");
        minimal_study_fixture(&fixture.root, "demo_study");
        unsafe {
            std::env::set_var("BIOMCP_STUDY_DIR", &fixture.root);
        }

        let groups = compare_expression_values("demo_study", "TP53", "ERBB2")
            .await
            .expect("grouped expression values should load");
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, "TP53-mutant");
        assert_eq!(groups[0].1, vec![2.0, 4.0]);
        assert_eq!(groups[1].0, "TP53-wildtype");
        assert_eq!(groups[1].1, vec![1.0]);
    }
}
