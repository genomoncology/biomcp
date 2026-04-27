//! Benchmark child-process execution, cache isolation, and report collection.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::super::types::{
    BENCHMARK_SCHEMA_VERSION, BenchmarkCaseKind, BenchmarkCaseStatus, BenchmarkCommandReport,
    BenchmarkEnvironment, BenchmarkMode, BenchmarkRunReport, BenchmarkSummary,
};
use super::build_summary;
use super::suite::{CaseSpec, SUITE_VERSION, compute_suite_hash, select_suite};

#[derive(Debug)]
struct CommandExecution {
    latency_ms: f64,
    stdout_bytes: u64,
    stderr_excerpt: String,
    exit_code: i32,
    timed_out: bool,
}

pub(super) async fn collect_report(
    mode: BenchmarkMode,
    iterations: u32,
    timeout_ms: u64,
    max_fail_fast_ms: u64,
    baseline_path: Option<String>,
) -> anyhow::Result<BenchmarkRunReport> {
    let suite = select_suite(mode);
    let suite_hash = compute_suite_hash(&suite);

    let cache_root = create_temp_cache_root()?;
    let _cache_guard = TempDirCleanup::new(cache_root.clone());

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
    let status = if saw_success_exit
        || fail_fast_latency_ms
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
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("failed to format benchmark timestamp")
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
    if sorted.len().is_multiple_of(2) {
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

struct TempDirCleanup {
    path: PathBuf,
}

impl TempDirCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempDirCleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
#[path = "tests/execute.rs"]
mod tests;
