//! Benchmark run and baseline persistence facade.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, anyhow};

use super::types::{BenchmarkCaseStatus, BenchmarkRunReport, BenchmarkSummary};

mod execute;
mod regression;
mod render;
mod suite;

use execute::collect_report;
use regression::{RegressionThresholds, compare_against_baseline};
use render::render_human_report;
use suite::{
    DEFAULT_LATENCY_THRESHOLD_PCT, DEFAULT_MAX_FAIL_FAST_MS, DEFAULT_SIZE_THRESHOLD_PCT,
    default_baseline_path, default_iterations, default_timeout, discover_latest_baseline_path,
    load_baseline,
};

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
