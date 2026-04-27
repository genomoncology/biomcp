//! Benchmark baseline comparison and regression classification.

use std::collections::{BTreeMap, BTreeSet};

use super::super::types::{
    BenchmarkCaseKind, BenchmarkCaseStatus, BenchmarkCommandReport, BenchmarkRegression,
    BenchmarkRunReport, BenchmarkTransientFailure,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct RegressionThresholds {
    pub(super) latency_pct: f64,
    pub(super) size_pct: f64,
    pub(super) max_fail_fast_ms: u64,
}

pub(super) fn compare_against_baseline(
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
                .map(format_float)
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

fn format_float(value: f64) -> String {
    format!("{value:.2}")
}

#[cfg(test)]
#[path = "tests/regression.rs"]
mod tests;
