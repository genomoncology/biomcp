use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, anyhow};
use semver::Version;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::types::{
    BENCHMARK_SCHEMA_VERSION, BenchmarkCaseKind, BenchmarkCaseStatus, BenchmarkCommandReport,
    BenchmarkEnvironment, BenchmarkMode, BenchmarkRegression, BenchmarkRunReport, BenchmarkSummary,
    BenchmarkTransientFailure,
};

const SUITE_VERSION: &str = "2026-02-17";
const DEFAULT_LATENCY_THRESHOLD_PCT: f64 = 20.0;
const DEFAULT_SIZE_THRESHOLD_PCT: f64 = 10.0;
const DEFAULT_MAX_FAIL_FAST_MS: u64 = 1500;
const DEFAULT_FULL_ITERATIONS: u32 = 3;
const DEFAULT_QUICK_ITERATIONS: u32 = 2;
const DEFAULT_FULL_TIMEOUT_MS: u64 = 45_000;
const DEFAULT_QUICK_TIMEOUT_MS: u64 = 20_000;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub quick: bool,
    pub iterations: Option<u32>,
    pub baseline: Option<PathBuf>,
    pub fail_on_regression: bool,
    pub fail_on_transient: bool,
    pub latency_threshold_pct: f64,
    pub size_threshold_pct: f64,
    pub max_fail_fast_ms: u64,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            quick: false,
            iterations: None,
            baseline: None,
            fail_on_regression: false,
            fail_on_transient: false,
            latency_threshold_pct: DEFAULT_LATENCY_THRESHOLD_PCT,
            size_threshold_pct: DEFAULT_SIZE_THRESHOLD_PCT,
            max_fail_fast_ms: DEFAULT_MAX_FAIL_FAST_MS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveBaselineOptions {
    pub quick: bool,
    pub iterations: Option<u32>,
    pub output: Option<PathBuf>,
}

impl Default for SaveBaselineOptions {
    fn default() -> Self {
        Self {
            quick: false,
            iterations: None,
            output: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RegressionThresholds {
    latency_pct: f64,
    size_pct: f64,
    max_fail_fast_ms: u64,
}

#[derive(Debug)]
struct CommandExecution {
    latency_ms: f64,
    stdout_bytes: u64,
    stderr_excerpt: String,
    exit_code: i32,
    timed_out: bool,
}

#[derive(Debug, Clone, Copy)]
struct CaseSpec {
    id: &'static str,
    kind: BenchmarkCaseKind,
    args: &'static [&'static str],
    tags: &'static [&'static str],
}

const FULL_SUITE: &[CaseSpec] = &[
    CaseSpec {
        id: "get_gene_braf",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "gene", "BRAF"],
        tags: &["core"],
    },
    CaseSpec {
        id: "get_variant_braf_v600e",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "variant", "BRAF V600E"],
        tags: &["core"],
    },
    CaseSpec {
        id: "get_trial_nct02576665",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "trial", "NCT02576665"],
        tags: &["core"],
    },
    CaseSpec {
        id: "search_article_braf_limit_5",
        kind: BenchmarkCaseKind::Success,
        args: &["search", "article", "-g", "BRAF", "--limit", "5"],
        tags: &["core"],
    },
    CaseSpec {
        id: "get_drug_imatinib",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "drug", "imatinib"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "search_trial_melanoma_limit_5",
        kind: BenchmarkCaseKind::Success,
        args: &["search", "trial", "-c", "melanoma", "--limit", "5"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "get_pgx_cyp2d6",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "pgx", "CYP2D6"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "search_variant_egfr_limit_5",
        kind: BenchmarkCaseKind::Success,
        args: &["search", "variant", "-g", "EGFR", "--limit", "5"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "get_pathway_r_hsa_5673001",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "pathway", "R-HSA-5673001"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "get_disease_mondo_0005105",
        kind: BenchmarkCaseKind::Success,
        args: &["get", "disease", "MONDO:0005105"],
        tags: &["extended"],
    },
    CaseSpec {
        id: "contract_invalid_article_since_2024_13_01",
        kind: BenchmarkCaseKind::ContractFailure,
        args: &[
            "search",
            "article",
            "-g",
            "BRAF",
            "--since",
            "2024-13-01",
            "--limit",
            "1",
        ],
        tags: &["contract", "contract_core"],
    },
    CaseSpec {
        id: "contract_invalid_trial_since_2024_02_30",
        kind: BenchmarkCaseKind::ContractFailure,
        args: &[
            "search",
            "trial",
            "-c",
            "melanoma",
            "--since",
            "2024-02-30",
            "--limit",
            "1",
        ],
        tags: &["contract"],
    },
];

pub async fn run_benchmark(opts: RunOptions, json_output: bool) -> anyhow::Result<String> {
    let mode = if opts.quick {
        BenchmarkMode::Quick
    } else {
        BenchmarkMode::Full
    };
    let iterations = opts.iterations.unwrap_or_else(|| default_iterations(mode));
    let timeout_ms = default_timeout(mode);

    let mut report =
        collect_report(mode, iterations, timeout_ms, opts.max_fail_fast_ms, None).await?;

    let baseline_path = if let Some(explicit) = opts.baseline.as_ref() {
        Some(explicit.clone())
    } else {
        discover_latest_baseline_path()
    };

    if let Some(path) = baseline_path {
        if path.exists() {
            let baseline = load_baseline(&path)?;
            compare_against_baseline(
                &mut report,
                &baseline,
                RegressionThresholds {
                    latency_pct: opts.latency_threshold_pct,
                    size_pct: opts.size_threshold_pct,
                    max_fail_fast_ms: opts.max_fail_fast_ms,
                },
            );
            report.baseline_path = Some(path.display().to_string());
        }
    }

    report.summary = build_summary(&report);
    let rendered = if json_output {
        crate::render::json::to_pretty(&report)?
    } else {
        render_human_report(&report)
    };

    if opts.fail_on_regression && !report.regressions.is_empty() {
        return Err(anyhow!(format!(
            "benchmark regressions detected ({}).\n{}",
            report.regressions.len(),
            rendered
        )));
    }

    if opts.fail_on_transient && !report.transient_failures.is_empty() {
        return Err(anyhow!(format!(
            "transient benchmark failures detected ({}).\n{}",
            report.transient_failures.len(),
            rendered
        )));
    }

    Ok(rendered)
}

pub async fn save_baseline(opts: SaveBaselineOptions, json_output: bool) -> anyhow::Result<String> {
    let mode = if opts.quick {
        BenchmarkMode::Quick
    } else {
        BenchmarkMode::Full
    };
    let iterations = opts.iterations.unwrap_or_else(|| default_iterations(mode));
    let timeout_ms = default_timeout(mode);

    let report =
        collect_report(mode, iterations, timeout_ms, DEFAULT_MAX_FAIL_FAST_MS, None).await?;

    let output_path = opts.output.unwrap_or_else(default_baseline_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create baseline directory {}",
                parent.to_string_lossy()
            )
        })?;
    }

    let mut serialized = crate::render::json::to_pretty(&report)?;
    serialized.push('\n');
    fs::write(&output_path, serialized).with_context(|| {
        format!(
            "failed to write baseline file {}",
            output_path.to_string_lossy()
        )
    })?;

    if json_output {
        #[derive(serde::Serialize)]
        struct SaveBaselineResponse {
            path: String,
            report: BenchmarkRunReport,
        }

        return Ok(crate::render::json::to_pretty(&SaveBaselineResponse {
            path: output_path.display().to_string(),
            report,
        })?);
    }

    Ok(format!(
        "Saved benchmark baseline: {}\ncases: {} | ok: {} | failed: {} | transient: {}",
        output_path.display(),
        report.summary.total_cases,
        report.summary.ok_cases,
        report.summary.failed_cases,
        report.summary.transient_failures,
    ))
}

fn build_summary(report: &BenchmarkRunReport) -> BenchmarkSummary {
    let total_cases = report.commands.len();
    let ok_cases = report
        .commands
        .iter()
        .filter(|case| case.status == BenchmarkCaseStatus::Ok)
        .count();
    let transient_failures = report
        .commands
        .iter()
        .filter(|case| case.status == BenchmarkCaseStatus::TransientFailure)
        .count();
    let failed_cases = total_cases.saturating_sub(ok_cases + transient_failures);

    BenchmarkSummary {
        total_cases,
        ok_cases,
        failed_cases,
        transient_failures,
        regression_count: report.regressions.len(),
    }
}

async fn collect_report(
    mode: BenchmarkMode,
    iterations: u32,
    timeout_ms: u64,
    max_fail_fast_ms: u64,
    baseline_path: Option<String>,
) -> anyhow::Result<BenchmarkRunReport> {
    let suite = select_suite(mode);
    let suite_hash = compute_suite_hash(&suite);

    let cache_root = create_temp_cache_root()?;
    let _cache_guard = TempDirGuard::new(cache_root.clone());

    let exe = std::env::current_exe().context("failed to resolve biomcp executable path")?;

    let mut commands = Vec::with_capacity(suite.len());
    for case in suite {
        let case_cache_root = cache_root.join(case.id);
        let report = match case.kind {
            BenchmarkCaseKind::Success => {
                run_success_case(case, iterations, timeout_ms, &exe, &case_cache_root).await?
            }
            BenchmarkCaseKind::ContractFailure => {
                run_contract_case(case, iterations, max_fail_fast_ms, &exe, &case_cache_root)
                    .await?
            }
        };
        commands.push(report);
    }

    commands.sort_by(|a, b| a.id.cmp(&b.id));

    let mut report = BenchmarkRunReport {
        schema_version: BENCHMARK_SCHEMA_VERSION,
        suite_version: SUITE_VERSION.to_string(),
        suite_hash,
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        generated_at: now_rfc3339()?,
        environment: BenchmarkEnvironment {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: std::env::var("HOSTNAME").ok(),
        },
        mode,
        iterations,
        baseline_path,
        commands,
        regressions: Vec::new(),
        transient_failures: Vec::new(),
        summary: BenchmarkSummary {
            total_cases: 0,
            ok_cases: 0,
            failed_cases: 0,
            transient_failures: 0,
            regression_count: 0,
        },
    };

    report.summary = build_summary(&report);
    Ok(report)
}

async fn run_success_case(
    case: CaseSpec,
    iterations: u32,
    timeout_ms: u64,
    exe: &Path,
    case_cache_root: &Path,
) -> anyhow::Result<BenchmarkCommandReport> {
    let mut cold_samples = Vec::with_capacity(iterations as usize);
    let mut warm_samples = Vec::with_capacity(iterations as usize);
    let mut markdown_bytes = Vec::with_capacity(iterations as usize);
    let mut json_bytes = Vec::with_capacity(iterations as usize);

    let mut had_transient_failure = false;
    let mut had_non_transient_failure = false;
    let mut stderr_excerpt = None;
    let mut last_exit_code = None;

    for _ in 0..iterations {
        reset_case_cache(case_cache_root)?;

        let cold = execute_case_command(exe, case.args, false, case_cache_root, timeout_ms).await?;
        if cold.exit_code == 0 && !cold.timed_out {
            cold_samples.push(cold.latency_ms);
            markdown_bytes.push(cold.stdout_bytes);
        } else {
            record_failure(
                &cold,
                &mut had_transient_failure,
                &mut had_non_transient_failure,
                &mut stderr_excerpt,
            );
        }

        let warm = execute_case_command(exe, case.args, false, case_cache_root, timeout_ms).await?;
        if warm.exit_code == 0 && !warm.timed_out {
            warm_samples.push(warm.latency_ms);
        } else {
            record_failure(
                &warm,
                &mut had_transient_failure,
                &mut had_non_transient_failure,
                &mut stderr_excerpt,
            );
        }

        let json = execute_case_command(exe, case.args, true, case_cache_root, timeout_ms).await?;
        last_exit_code = Some(json.exit_code);
        if json.exit_code == 0 && !json.timed_out {
            json_bytes.push(json.stdout_bytes);
        } else {
            record_failure(
                &json,
                &mut had_transient_failure,
                &mut had_non_transient_failure,
                &mut stderr_excerpt,
            );
        }
    }

    let status = if had_non_transient_failure {
        BenchmarkCaseStatus::Failed
    } else if had_transient_failure {
        BenchmarkCaseStatus::TransientFailure
    } else {
        BenchmarkCaseStatus::Ok
    };

    Ok(BenchmarkCommandReport {
        id: case.id.to_string(),
        kind: BenchmarkCaseKind::Success,
        command: format_command(case.args),
        tags: case.tags.iter().map(|tag| (*tag).to_string()).collect(),
        status,
        iterations,
        cold_latency_ms: median_f64(&cold_samples),
        warm_latency_ms: median_f64(&warm_samples),
        markdown_bytes: median_u64(&markdown_bytes),
        json_bytes: median_u64(&json_bytes),
        fail_fast_latency_ms: None,
        exit_code: last_exit_code,
        stderr_excerpt,
    })
}

async fn run_contract_case(
    case: CaseSpec,
    iterations: u32,
    max_fail_fast_ms: u64,
    exe: &Path,
    case_cache_root: &Path,
) -> anyhow::Result<BenchmarkCommandReport> {
    let timeout_ms = max_fail_fast_ms.saturating_mul(4).max(3000);
    let mut latencies = Vec::with_capacity(iterations as usize);
    let mut exit_codes = Vec::with_capacity(iterations as usize);
    let mut stderr_excerpt = None;
    let mut saw_success_exit = false;

    for _ in 0..iterations {
        reset_case_cache(case_cache_root)?;
        let exec = execute_case_command(exe, case.args, false, case_cache_root, timeout_ms).await?;
        latencies.push(exec.latency_ms);
        exit_codes.push(exec.exit_code);
        if exec.exit_code == 0 {
            saw_success_exit = true;
            if stderr_excerpt.is_none() {
                stderr_excerpt = Some(exec.stderr_excerpt.clone());
            }
        }
    }

    let fail_fast_latency_ms = median_f64(&latencies);
    let status = if saw_success_exit {
        BenchmarkCaseStatus::Failed
    } else if fail_fast_latency_ms
        .map(|latency| latency > max_fail_fast_ms as f64)
        .unwrap_or(true)
    {
        BenchmarkCaseStatus::Failed
    } else {
        BenchmarkCaseStatus::Ok
    };

    let exit_code = median_i32(&exit_codes);

    Ok(BenchmarkCommandReport {
        id: case.id.to_string(),
        kind: BenchmarkCaseKind::ContractFailure,
        command: format_command(case.args),
        tags: case.tags.iter().map(|tag| (*tag).to_string()).collect(),
        status,
        iterations,
        cold_latency_ms: None,
        warm_latency_ms: None,
        markdown_bytes: None,
        json_bytes: None,
        fail_fast_latency_ms,
        exit_code,
        stderr_excerpt,
    })
}

fn record_failure(
    exec: &CommandExecution,
    had_transient_failure: &mut bool,
    had_non_transient_failure: &mut bool,
    stderr_excerpt: &mut Option<String>,
) {
    if is_transient_failure(exec) {
        *had_transient_failure = true;
    } else {
        *had_non_transient_failure = true;
    }
    if stderr_excerpt.is_none() {
        *stderr_excerpt = Some(exec.stderr_excerpt.clone());
    }
}

async fn execute_case_command(
    exe: &Path,
    args: &[&str],
    as_json: bool,
    cache_home: &Path,
    timeout_ms: u64,
) -> anyhow::Result<CommandExecution> {
    let mut cmd = tokio::process::Command::new(exe);
    cmd.kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("XDG_CACHE_HOME", cache_home)
        .args(build_child_args(args, as_json));

    let start = tokio::time::Instant::now();
    let output = tokio::time::timeout(Duration::from_millis(timeout_ms), cmd.output()).await;

    match output {
        Ok(Ok(out)) => {
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            Ok(CommandExecution {
                latency_ms,
                stdout_bytes: out.stdout.len() as u64,
                stderr_excerpt: trim_excerpt(&stderr),
                exit_code: out.status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        Ok(Err(err)) => Err(err).context("failed to run benchmark command"),
        Err(_) => Ok(CommandExecution {
            latency_ms: timeout_ms as f64,
            stdout_bytes: 0,
            stderr_excerpt: format!("timed out after {}ms", timeout_ms),
            exit_code: -1,
            timed_out: true,
        }),
    }
}

fn compare_against_baseline(
    report: &mut BenchmarkRunReport,
    baseline: &BenchmarkRunReport,
    thresholds: RegressionThresholds,
) {
    let baseline_by_id = baseline
        .commands
        .iter()
        .map(|command| (command.id.as_str(), command))
        .collect::<BTreeMap<_, _>>();

    let mut regressions = Vec::new();
    let mut transient = Vec::new();
    let mut seen = BTreeSet::new();

    for command in &report.commands {
        seen.insert(command.id.clone());

        if command.status == BenchmarkCaseStatus::TransientFailure {
            transient.push(BenchmarkTransientFailure {
                command_id: command.id.clone(),
                message: command
                    .stderr_excerpt
                    .clone()
                    .unwrap_or_else(|| "transient upstream failure".to_string()),
            });
            continue;
        }

        let Some(base) = baseline_by_id.get(command.id.as_str()) else {
            continue;
        };

        if base.kind != command.kind {
            regressions.push(BenchmarkRegression {
                command_id: command.id.clone(),
                metric: "kind".to_string(),
                baseline_value: format!("{:?}", base.kind),
                current_value: format!("{:?}", command.kind),
                delta_pct: None,
                message: "benchmark case kind changed".to_string(),
            });
            continue;
        }

        if base.status == BenchmarkCaseStatus::Ok && command.status != BenchmarkCaseStatus::Ok {
            regressions.push(BenchmarkRegression {
                command_id: command.id.clone(),
                metric: "status".to_string(),
                baseline_value: "ok".to_string(),
                current_value: format!("{:?}", command.status),
                delta_pct: None,
                message: "case no longer succeeds".to_string(),
            });
        }

        match command.kind {
            BenchmarkCaseKind::Success => compare_success_case(
                &mut regressions,
                base,
                command,
                thresholds.latency_pct,
                thresholds.size_pct,
            ),
            BenchmarkCaseKind::ContractFailure => {
                compare_contract_case(&mut regressions, base, command, thresholds.max_fail_fast_ms)
            }
        }
    }

    for command in &baseline.commands {
        if !seen.contains(&command.id) {
            regressions.push(BenchmarkRegression {
                command_id: command.id.clone(),
                metric: "missing_case".to_string(),
                baseline_value: "present".to_string(),
                current_value: "missing".to_string(),
                delta_pct: None,
                message: "command missing from current benchmark run".to_string(),
            });
        }
    }

    regressions.sort_by(|a, b| {
        a.command_id
            .cmp(&b.command_id)
            .then_with(|| a.metric.cmp(&b.metric))
    });
    transient.sort_by(|a, b| a.command_id.cmp(&b.command_id));

    report.regressions = regressions;
    report.transient_failures = transient;
}

fn compare_success_case(
    regressions: &mut Vec<BenchmarkRegression>,
    baseline: &BenchmarkCommandReport,
    current: &BenchmarkCommandReport,
    latency_threshold_pct: f64,
    size_threshold_pct: f64,
) {
    maybe_push_numeric_regression(
        regressions,
        current.id.as_str(),
        "warm_latency_ms",
        baseline.warm_latency_ms,
        current.warm_latency_ms,
        latency_threshold_pct,
        "warm latency",
    );
    maybe_push_numeric_regression(
        regressions,
        current.id.as_str(),
        "cold_latency_ms",
        baseline.cold_latency_ms,
        current.cold_latency_ms,
        latency_threshold_pct,
        "cold latency",
    );

    maybe_push_numeric_regression(
        regressions,
        current.id.as_str(),
        "markdown_bytes",
        baseline.markdown_bytes.map(|v| v as f64),
        current.markdown_bytes.map(|v| v as f64),
        size_threshold_pct,
        "markdown output size",
    );

    maybe_push_numeric_regression(
        regressions,
        current.id.as_str(),
        "json_bytes",
        baseline.json_bytes.map(|v| v as f64),
        current.json_bytes.map(|v| v as f64),
        size_threshold_pct,
        "json output size",
    );

    if let (Some(base_exit), Some(cur_exit)) = (baseline.exit_code, current.exit_code)
        && base_exit != cur_exit
    {
        regressions.push(BenchmarkRegression {
            command_id: current.id.clone(),
            metric: "exit_code".to_string(),
            baseline_value: base_exit.to_string(),
            current_value: cur_exit.to_string(),
            delta_pct: None,
            message: "exit code changed".to_string(),
        });
    }
}

fn compare_contract_case(
    regressions: &mut Vec<BenchmarkRegression>,
    baseline: &BenchmarkCommandReport,
    current: &BenchmarkCommandReport,
    max_fail_fast_ms: u64,
) {
    let baseline_exit = baseline.exit_code.unwrap_or(1);
    let current_exit = current.exit_code.unwrap_or(0);

    if baseline_exit != 0 && current_exit == 0 {
        regressions.push(BenchmarkRegression {
            command_id: current.id.clone(),
            metric: "invalid_date_exit_code".to_string(),
            baseline_value: baseline_exit.to_string(),
            current_value: current_exit.to_string(),
            delta_pct: None,
            message: "invalid date case no longer fails".to_string(),
        });
    }

    if let Some(latency) = current.fail_fast_latency_ms
        && latency > max_fail_fast_ms as f64
    {
        regressions.push(BenchmarkRegression {
            command_id: current.id.clone(),
            metric: "fail_fast_latency_ms".to_string(),
            baseline_value: baseline
                .fail_fast_latency_ms
                .map(|value| format_float(value))
                .unwrap_or_else(|| "n/a".to_string()),
            current_value: format_float(latency),
            delta_pct: None,
            message: format!("fail-fast latency exceeds {}ms limit", max_fail_fast_ms),
        });
    }
}

fn maybe_push_numeric_regression(
    regressions: &mut Vec<BenchmarkRegression>,
    command_id: &str,
    metric: &str,
    baseline: Option<f64>,
    current: Option<f64>,
    threshold_pct: f64,
    label: &str,
) {
    let (Some(base), Some(cur)) = (baseline, current) else {
        return;
    };

    if base <= 0.0 {
        if cur > 0.0 {
            regressions.push(BenchmarkRegression {
                command_id: command_id.to_string(),
                metric: metric.to_string(),
                baseline_value: format_float(base),
                current_value: format_float(cur),
                delta_pct: None,
                message: format!("{label} changed from zero baseline"),
            });
        }
        return;
    }

    let delta_pct = ((cur - base) / base) * 100.0;
    if delta_pct > threshold_pct {
        regressions.push(BenchmarkRegression {
            command_id: command_id.to_string(),
            metric: metric.to_string(),
            baseline_value: format_float(base),
            current_value: format_float(cur),
            delta_pct: Some(delta_pct),
            message: format!(
                "{label} increased by {:.2}% (threshold {:.2}%)",
                delta_pct, threshold_pct
            ),
        });
    }
}

fn is_transient_failure(exec: &CommandExecution) -> bool {
    if exec.timed_out {
        return true;
    }

    let msg = exec.stderr_excerpt.to_ascii_lowercase();
    msg.contains("timed out")
        || msg.contains("timeout")
        || msg.contains("temporary")
        || msg.contains("connection")
        || msg.contains("dns")
        || msg.contains("http 429")
        || msg.contains("http 502")
        || msg.contains("http 503")
        || msg.contains("http 504")
}

fn render_human_report(report: &BenchmarkRunReport) -> String {
    let mut out = String::new();
    out.push_str("# BioMCP Benchmark Report\n\n");
    out.push_str(&format!(
        "- Mode: {}\n- Iterations: {}\n- Suite version: {}\n- Suite hash: {}\n- Generated: {}\n",
        mode_label(report.mode),
        report.iterations,
        report.suite_version,
        report.suite_hash,
        report.generated_at,
    ));

    if let Some(path) = &report.baseline_path {
        out.push_str(&format!("- Baseline: {}\n", path));
    }

    out.push_str(&format!(
        "- Summary: total={} ok={} failed={} transient={} regressions={}\n",
        report.summary.total_cases,
        report.summary.ok_cases,
        report.summary.failed_cases,
        report.summary.transient_failures,
        report.summary.regression_count,
    ));

    out.push_str("\n## Command Metrics\n\n");
    out.push_str(
        "| id | kind | status | cold_ms | warm_ms | md_bytes | json_bytes | fail_fast_ms |\n",
    );
    out.push_str("|---|---|---|---:|---:|---:|---:|---:|\n");
    for case in &report.commands {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            case.id,
            kind_label(case.kind),
            status_label(case.status),
            fmt_opt_f64(case.cold_latency_ms),
            fmt_opt_f64(case.warm_latency_ms),
            fmt_opt_u64(case.markdown_bytes),
            fmt_opt_u64(case.json_bytes),
            fmt_opt_f64(case.fail_fast_latency_ms),
        ));
    }

    if !report.regressions.is_empty() {
        out.push_str("\n## Regressions\n\n");
        out.push_str("| command_id | metric | baseline | current | delta_pct | message |\n");
        out.push_str("|---|---|---:|---:|---:|---|\n");
        for regression in &report.regressions {
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                regression.command_id,
                regression.metric,
                regression.baseline_value,
                regression.current_value,
                regression
                    .delta_pct
                    .map(format_float)
                    .unwrap_or_else(|| "n/a".to_string()),
                regression.message,
            ));
        }
    }

    if !report.transient_failures.is_empty() {
        out.push_str("\n## Transient Failures\n\n");
        for failure in &report.transient_failures {
            out.push_str(&format!("- {}: {}\n", failure.command_id, failure.message));
        }
    }

    out
}

fn mode_label(mode: BenchmarkMode) -> &'static str {
    match mode {
        BenchmarkMode::Full => "full",
        BenchmarkMode::Quick => "quick",
    }
}

fn kind_label(kind: BenchmarkCaseKind) -> &'static str {
    match kind {
        BenchmarkCaseKind::Success => "success",
        BenchmarkCaseKind::ContractFailure => "contract_failure",
    }
}

fn status_label(status: BenchmarkCaseStatus) -> &'static str {
    match status {
        BenchmarkCaseStatus::Ok => "ok",
        BenchmarkCaseStatus::Failed => "failed",
        BenchmarkCaseStatus::TransientFailure => "transient_failure",
    }
}

fn trim_excerpt(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.len() <= 240 {
        compact
    } else {
        format!("{}...", &compact[..240])
    }
}

fn build_child_args(args: &[&str], as_json: bool) -> Vec<OsString> {
    let mut full = Vec::with_capacity(args.len() + usize::from(as_json));
    if as_json {
        full.push(OsString::from("--json"));
    }
    for arg in args {
        full.push(OsString::from(arg));
    }
    full
}

fn create_temp_cache_root() -> anyhow::Result<PathBuf> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("failed to build benchmark temp cache timestamp")?
        .as_millis();
    let pid = std::process::id();
    let root = std::env::temp_dir().join(format!("biomcp-benchmark-{}-{}", pid, now_ms));
    fs::create_dir_all(&root).with_context(|| {
        format!(
            "failed to create benchmark cache root {}",
            root.to_string_lossy()
        )
    })?;
    Ok(root)
}

fn reset_case_cache(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| {
            format!("failed to clear benchmark cache {}", path.to_string_lossy())
        })?;
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create cache {}", path.to_string_lossy()))
}

fn now_rfc3339() -> anyhow::Result<String> {
    Ok(OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format benchmark timestamp")?)
}

fn default_timeout(mode: BenchmarkMode) -> u64 {
    match mode {
        BenchmarkMode::Full => DEFAULT_FULL_TIMEOUT_MS,
        BenchmarkMode::Quick => DEFAULT_QUICK_TIMEOUT_MS,
    }
}

fn default_iterations(mode: BenchmarkMode) -> u32 {
    match mode {
        BenchmarkMode::Full => DEFAULT_FULL_ITERATIONS,
        BenchmarkMode::Quick => DEFAULT_QUICK_ITERATIONS,
    }
}

fn default_baseline_path() -> PathBuf {
    PathBuf::from("benchmarks").join(format!("v{}.json", env!("CARGO_PKG_VERSION")))
}

fn discover_latest_baseline_path() -> Option<PathBuf> {
    let dir = Path::new("benchmarks");
    let entries = fs::read_dir(dir).ok()?;
    let mut candidates = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_str()?;
        if !name.starts_with('v') || !name.ends_with(".json") {
            continue;
        }
        let version_text = name
            .strip_prefix('v')
            .and_then(|v| v.strip_suffix(".json"))?;
        let version = Version::parse(version_text).ok()?;
        candidates.push((version, path));
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    candidates.pop().map(|(_, path)| path)
}

fn load_baseline(path: &Path) -> anyhow::Result<BenchmarkRunReport> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read baseline file {}", path.to_string_lossy()))?;
    let report = serde_json::from_str::<BenchmarkRunReport>(&text)
        .with_context(|| format!("failed to parse baseline file {}", path.to_string_lossy()))?;
    Ok(report)
}

fn select_suite(mode: BenchmarkMode) -> Vec<CaseSpec> {
    match mode {
        BenchmarkMode::Full => FULL_SUITE.to_vec(),
        BenchmarkMode::Quick => FULL_SUITE
            .iter()
            .copied()
            .filter(|case| case.tags.contains(&"core") || case.tags.contains(&"contract_core"))
            .collect(),
    }
}

fn compute_suite_hash(cases: &[CaseSpec]) -> String {
    let mut hasher = Sha256::new();
    for case in cases {
        hasher.update(case.id.as_bytes());
        hasher.update(b"\0");
        for arg in case.args {
            hasher.update(arg.as_bytes());
            hasher.update(b"\0");
        }
        hasher.update(b"\n");
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn format_command(args: &[&str]) -> String {
    let mut command = String::from("biomcp");
    for arg in args {
        command.push(' ');
        if arg.contains(' ') {
            command.push('"');
            command.push_str(arg);
            command.push('"');
        } else {
            command.push_str(arg);
        }
    }
    command
}

fn median_f64(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

fn median_u64(samples: &[u64]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    Some(sorted[sorted.len() / 2])
}

fn median_i32(samples: &[i32]) -> Option<i32> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    Some(sorted[sorted.len() / 2])
}

fn fmt_opt_f64(value: Option<f64>) -> String {
    value.map(format_float).unwrap_or_else(|| "n/a".to_string())
}

fn fmt_opt_u64(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "n/a".to_string())
}

fn format_float(value: f64) -> String {
    format!("{value:.2}")
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success_case(
        id: &str,
        warm_ms: f64,
        cold_ms: f64,
        md_bytes: u64,
        json_bytes: u64,
    ) -> BenchmarkCommandReport {
        BenchmarkCommandReport {
            id: id.to_string(),
            kind: BenchmarkCaseKind::Success,
            command: "biomcp get gene BRAF".to_string(),
            tags: vec!["core".to_string()],
            status: BenchmarkCaseStatus::Ok,
            iterations: 3,
            cold_latency_ms: Some(cold_ms),
            warm_latency_ms: Some(warm_ms),
            markdown_bytes: Some(md_bytes),
            json_bytes: Some(json_bytes),
            fail_fast_latency_ms: None,
            exit_code: Some(0),
            stderr_excerpt: None,
        }
    }

    fn contract_case(id: &str, latency_ms: f64, exit_code: i32) -> BenchmarkCommandReport {
        BenchmarkCommandReport {
            id: id.to_string(),
            kind: BenchmarkCaseKind::ContractFailure,
            command: "biomcp search article -g BRAF --since 2024-13-01 --limit 1".to_string(),
            tags: vec!["contract".to_string()],
            status: BenchmarkCaseStatus::Ok,
            iterations: 3,
            cold_latency_ms: None,
            warm_latency_ms: None,
            markdown_bytes: None,
            json_bytes: None,
            fail_fast_latency_ms: Some(latency_ms),
            exit_code: Some(exit_code),
            stderr_excerpt: None,
        }
    }

    fn report(commands: Vec<BenchmarkCommandReport>) -> BenchmarkRunReport {
        BenchmarkRunReport {
            schema_version: BENCHMARK_SCHEMA_VERSION,
            suite_version: SUITE_VERSION.to_string(),
            suite_hash: "abc".to_string(),
            cli_version: "0.3.0".to_string(),
            generated_at: "2026-02-17T00:00:00Z".to_string(),
            environment: BenchmarkEnvironment {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                hostname: None,
            },
            mode: BenchmarkMode::Full,
            iterations: 3,
            baseline_path: None,
            commands,
            regressions: Vec::new(),
            transient_failures: Vec::new(),
            summary: BenchmarkSummary {
                total_cases: 0,
                ok_cases: 0,
                failed_cases: 0,
                transient_failures: 0,
                regression_count: 0,
            },
        }
    }

    #[test]
    fn detects_latency_and_size_regressions_above_threshold() {
        let baseline = report(vec![success_case("case", 100.0, 120.0, 1000, 1500)]);
        let mut current = report(vec![success_case("case", 130.0, 160.0, 1200, 1700)]);

        compare_against_baseline(
            &mut current,
            &baseline,
            RegressionThresholds {
                latency_pct: 20.0,
                size_pct: 10.0,
                max_fail_fast_ms: 1500,
            },
        );

        let metrics = current
            .regressions
            .iter()
            .map(|r| r.metric.clone())
            .collect::<BTreeSet<_>>();

        assert!(metrics.contains("warm_latency_ms"));
        assert!(metrics.contains("cold_latency_ms"));
        assert!(metrics.contains("markdown_bytes"));
        assert!(metrics.contains("json_bytes"));
    }

    #[test]
    fn ignores_changes_below_thresholds() {
        let baseline = report(vec![success_case("case", 100.0, 120.0, 1000, 1500)]);
        let mut current = report(vec![success_case("case", 118.0, 140.0, 1080, 1600)]);

        compare_against_baseline(
            &mut current,
            &baseline,
            RegressionThresholds {
                latency_pct: 20.0,
                size_pct: 10.0,
                max_fail_fast_ms: 1500,
            },
        );

        assert!(current.regressions.is_empty());
    }

    #[test]
    fn flags_invalid_date_contract_that_starts_succeeding() {
        let baseline = report(vec![contract_case("contract", 300.0, 1)]);
        let mut current = report(vec![contract_case("contract", 350.0, 0)]);

        compare_against_baseline(
            &mut current,
            &baseline,
            RegressionThresholds {
                latency_pct: 20.0,
                size_pct: 10.0,
                max_fail_fast_ms: 1500,
            },
        );

        assert!(
            current
                .regressions
                .iter()
                .any(|regression| regression.metric == "invalid_date_exit_code")
        );
    }

    #[test]
    fn flags_fail_fast_latency_over_limit() {
        let baseline = report(vec![contract_case("contract", 300.0, 1)]);
        let mut current = report(vec![contract_case("contract", 2000.0, 1)]);

        compare_against_baseline(
            &mut current,
            &baseline,
            RegressionThresholds {
                latency_pct: 20.0,
                size_pct: 10.0,
                max_fail_fast_ms: 1500,
            },
        );

        assert!(
            current
                .regressions
                .iter()
                .any(|regression| regression.metric == "fail_fast_latency_ms")
        );
    }

    #[test]
    fn quick_suite_keeps_core_and_one_contract_case() {
        let quick = select_suite(BenchmarkMode::Quick);
        let contract_count = quick
            .iter()
            .filter(|case| case.kind == BenchmarkCaseKind::ContractFailure)
            .count();

        assert!(quick.len() >= 4);
        assert_eq!(contract_count, 1);
        assert!(
            quick
                .iter()
                .all(|case| case.tags.contains(&"core") || case.tags.contains(&"contract_core"))
        );
    }

    #[test]
    fn baseline_discovery_picks_highest_semver() {
        let root = std::env::temp_dir().join(format!(
            "biomcp-benchmark-discovery-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let benchmarks_dir = root.join("benchmarks");
        fs::create_dir_all(&benchmarks_dir).expect("mkdir");
        fs::write(benchmarks_dir.join("v0.1.0.json"), "{}").expect("write");
        fs::write(benchmarks_dir.join("v0.3.0.json"), "{}").expect("write");
        fs::write(benchmarks_dir.join("v0.2.5.json"), "{}").expect("write");

        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(&root).expect("set cwd");
        let selected = discover_latest_baseline_path();
        std::env::set_current_dir(cwd).expect("restore cwd");

        fs::remove_dir_all(&root).expect("cleanup");

        let selected_name = selected
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str());
        assert_eq!(selected_name, Some("v0.3.0.json"));
    }
}
