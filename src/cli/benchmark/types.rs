use serde::{Deserialize, Serialize};

pub const BENCHMARK_SCHEMA_VERSION: u32 = 1;
pub const SESSION_SCORE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkMode {
    Full,
    Quick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCaseKind {
    Success,
    ContractFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCaseStatus {
    Ok,
    Failed,
    TransientFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkEnvironment {
    pub os: String,
    pub arch: String,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkCommandReport {
    pub id: String,
    pub kind: BenchmarkCaseKind,
    pub command: String,
    pub tags: Vec<String>,
    pub status: BenchmarkCaseStatus,
    pub iterations: u32,
    pub cold_latency_ms: Option<f64>,
    pub warm_latency_ms: Option<f64>,
    pub markdown_bytes: Option<u64>,
    pub json_bytes: Option<u64>,
    pub fail_fast_latency_ms: Option<f64>,
    pub exit_code: Option<i32>,
    pub stderr_excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRegression {
    pub command_id: String,
    pub metric: String,
    pub baseline_value: String,
    pub current_value: String,
    pub delta_pct: Option<f64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTransientFailure {
    pub command_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_cases: usize,
    pub ok_cases: usize,
    pub failed_cases: usize,
    pub transient_failures: usize,
    pub regression_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRunReport {
    pub schema_version: u32,
    pub suite_version: String,
    pub suite_hash: String,
    pub cli_version: String,
    pub generated_at: String,
    pub environment: BenchmarkEnvironment,
    pub mode: BenchmarkMode,
    pub iterations: u32,
    pub baseline_path: Option<String>,
    pub commands: Vec<BenchmarkCommandReport>,
    pub regressions: Vec<BenchmarkRegression>,
    pub transient_failures: Vec<BenchmarkTransientFailure>,
    pub summary: BenchmarkSummary,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionErrorCategories {
    pub ghost: u64,
    pub quoting: u64,
    pub api: u64,
    pub other: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionCoverage {
    pub expected_total: usize,
    pub hits: usize,
    pub misses: usize,
    pub extras: usize,
    pub missing_commands: Vec<String>,
    pub extra_commands: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionTokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionScoreReport {
    pub schema_version: u32,
    pub generated_at: String,
    pub session_path: String,
    pub total_tool_calls: u64,
    pub biomcp_commands: u64,
    pub help_calls: u64,
    pub skill_reads: u64,
    pub errors_total: u64,
    pub error_categories: SessionErrorCategories,
    pub coverage: Option<SessionCoverage>,
    pub tokens: SessionTokenUsage,
    pub wall_time_ms: Option<u64>,
    pub command_shapes: Vec<String>,
}
