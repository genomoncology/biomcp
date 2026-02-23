use std::path::PathBuf;

use clap::Subcommand;

mod run;
mod score;
pub mod types;

#[derive(Subcommand, Debug, Clone)]
pub enum BenchmarkCommand {
    /// Run benchmark suite and optionally compare against baseline
    Run {
        /// Run a smaller core benchmark subset
        #[arg(long)]
        quick: bool,

        /// Override iteration count (default: full=3, quick=2)
        #[arg(long)]
        iterations: Option<u32>,

        /// Baseline report path (default: latest benchmarks/v*.json if present)
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Exit non-zero when regressions are detected
        #[arg(long)]
        fail_on_regression: bool,

        /// Exit non-zero when transient upstream failures are detected
        #[arg(long)]
        fail_on_transient: bool,

        /// Latency regression threshold percentage (default: 20)
        #[arg(long, default_value = "20")]
        latency_threshold_pct: f64,

        /// Output-size regression threshold percentage (default: 10)
        #[arg(long, default_value = "10")]
        size_threshold_pct: f64,

        /// Max allowed fail-fast latency for contract checks (default: 1500ms)
        #[arg(long, default_value = "1500")]
        max_fail_fast_ms: u64,
    },

    /// Run benchmark suite and persist as baseline JSON
    SaveBaseline {
        /// Run a smaller core benchmark subset
        #[arg(long)]
        quick: bool,

        /// Override iteration count (default: full=3, quick=2)
        #[arg(long)]
        iterations: Option<u32>,

        /// Output path (default: benchmarks/v<CARGO_PKG_VERSION>.json)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Score a PI JSONL agent session for BioMCP usage and coverage
    ScoreSession {
        /// Session JSONL path
        session: PathBuf,

        /// Expected command set file for coverage scoring
        #[arg(long)]
        expected: Option<PathBuf>,

        /// Show a shorter markdown summary
        #[arg(long)]
        brief: bool,
    },
}

pub async fn run(command: BenchmarkCommand, json_output: bool) -> anyhow::Result<String> {
    match command {
        BenchmarkCommand::Run {
            quick,
            iterations,
            baseline,
            fail_on_regression,
            fail_on_transient,
            latency_threshold_pct,
            size_threshold_pct,
            max_fail_fast_ms,
        } => {
            let opts = run::RunOptions {
                quick,
                iterations,
                baseline,
                fail_on_regression,
                fail_on_transient,
                latency_threshold_pct,
                size_threshold_pct,
                max_fail_fast_ms,
            };
            run::run_benchmark(opts, json_output).await
        }
        BenchmarkCommand::SaveBaseline {
            quick,
            iterations,
            output,
        } => {
            let opts = run::SaveBaselineOptions {
                quick,
                iterations,
                output,
            };
            run::save_baseline(opts, json_output).await
        }
        BenchmarkCommand::ScoreSession {
            session,
            expected,
            brief,
        } => {
            let opts = score::ScoreSessionOptions {
                session,
                expected,
                brief,
            };
            score::score_session(opts, json_output)
        }
    }
}
