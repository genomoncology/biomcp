//! Regression comparison benchmark tests.

use std::collections::BTreeSet;

use super::super::super::types::{
    BENCHMARK_SCHEMA_VERSION, BenchmarkCaseKind, BenchmarkCaseStatus, BenchmarkCommandReport,
    BenchmarkEnvironment, BenchmarkMode, BenchmarkRunReport, BenchmarkSummary,
};
use super::super::suite::SUITE_VERSION;
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
