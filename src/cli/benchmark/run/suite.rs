//! Benchmark suite definitions, defaults, hashing, and baseline discovery.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use semver::Version;
use sha2::{Digest, Sha256};

use super::super::types::{BenchmarkCaseKind, BenchmarkMode, BenchmarkRunReport};

pub(super) const SUITE_VERSION: &str = "2026-02-17";
pub(super) const DEFAULT_LATENCY_THRESHOLD_PCT: f64 = 20.0;
pub(super) const DEFAULT_SIZE_THRESHOLD_PCT: f64 = 10.0;
pub(super) const DEFAULT_MAX_FAIL_FAST_MS: u64 = 1500;
pub(super) const DEFAULT_FULL_ITERATIONS: u32 = 3;
pub(super) const DEFAULT_QUICK_ITERATIONS: u32 = 2;
pub(super) const DEFAULT_FULL_TIMEOUT_MS: u64 = 45_000;
pub(super) const DEFAULT_QUICK_TIMEOUT_MS: u64 = 20_000;

#[derive(Debug, Clone, Copy)]
pub(super) struct CaseSpec {
    pub(super) id: &'static str,
    pub(super) kind: BenchmarkCaseKind,
    pub(super) args: &'static [&'static str],
    pub(super) tags: &'static [&'static str],
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

pub(super) fn default_timeout(mode: BenchmarkMode) -> u64 {
    match mode {
        BenchmarkMode::Full => DEFAULT_FULL_TIMEOUT_MS,
        BenchmarkMode::Quick => DEFAULT_QUICK_TIMEOUT_MS,
    }
}

pub(super) fn default_iterations(mode: BenchmarkMode) -> u32 {
    match mode {
        BenchmarkMode::Full => DEFAULT_FULL_ITERATIONS,
        BenchmarkMode::Quick => DEFAULT_QUICK_ITERATIONS,
    }
}

pub(super) fn default_baseline_path() -> PathBuf {
    PathBuf::from("benchmarks").join(format!("v{}.json", env!("CARGO_PKG_VERSION")))
}

pub(super) fn discover_latest_baseline_path() -> Option<PathBuf> {
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

pub(super) fn load_baseline(path: &Path) -> anyhow::Result<BenchmarkRunReport> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read baseline file {}", path.to_string_lossy()))?;
    let report = serde_json::from_str::<BenchmarkRunReport>(&text)
        .with_context(|| format!("failed to parse baseline file {}", path.to_string_lossy()))?;
    Ok(report)
}

pub(super) fn select_suite(mode: BenchmarkMode) -> Vec<CaseSpec> {
    match mode {
        BenchmarkMode::Full => FULL_SUITE.to_vec(),
        BenchmarkMode::Quick => FULL_SUITE
            .iter()
            .copied()
            .filter(|case| case.tags.contains(&"core") || case.tags.contains(&"contract_core"))
            .collect(),
    }
}

pub(super) fn compute_suite_hash(cases: &[CaseSpec]) -> String {
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

#[cfg(test)]
#[path = "tests/suite.rs"]
mod tests;
