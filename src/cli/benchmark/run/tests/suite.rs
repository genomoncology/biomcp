//! Suite-side benchmark tests.

use std::fs;

use super::super::super::types::{BenchmarkCaseKind, BenchmarkMode};
use super::*;

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
    let root = crate::test_support::TempDirGuard::new("benchmark-discovery");
    let benchmarks_dir = root.path().join("benchmarks");
    fs::create_dir_all(&benchmarks_dir).expect("mkdir");
    fs::write(benchmarks_dir.join("v0.1.0.json"), "{}").expect("write");
    fs::write(benchmarks_dir.join("v0.3.0.json"), "{}").expect("write");
    fs::write(benchmarks_dir.join("v0.2.5.json"), "{}").expect("write");

    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(root.path()).expect("set cwd");
    let selected = discover_latest_baseline_path();
    std::env::set_current_dir(cwd).expect("restore cwd");

    let selected_name = selected
        .as_ref()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str());
    assert_eq!(selected_name, Some("v0.3.0.json"));
}
