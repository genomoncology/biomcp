//! Benchmark markdown rendering and display labels.

use super::super::types::{
    BenchmarkCaseKind, BenchmarkCaseStatus, BenchmarkMode, BenchmarkRunReport,
};

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

fn format_float(value: f64) -> String {
    format!("{value:.2}")
}

#[cfg(test)]
#[path = "tests/render.rs"]
mod tests;
