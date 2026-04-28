use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn assert_module_doc_header(path: &Path) {
    let source = read_source(path);
    let first_line = source.lines().next().unwrap_or_default();
    assert!(
        first_line.starts_with("//!"),
        "missing //! header: {}",
        path.display()
    );
}

fn expected_benchmark_submodule_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/cli/benchmark/run/mod.rs"),
        root.join("src/cli/benchmark/run/suite.rs"),
        root.join("src/cli/benchmark/run/execute.rs"),
        root.join("src/cli/benchmark/run/regression.rs"),
        root.join("src/cli/benchmark/run/render.rs"),
        root.join("src/cli/benchmark/run/tests/suite.rs"),
        root.join("src/cli/benchmark/run/tests/execute.rs"),
        root.join("src/cli/benchmark/run/tests/regression.rs"),
        root.join("src/cli/benchmark/run/tests/render.rs"),
        root.join("src/cli/benchmark/score/mod.rs"),
        root.join("src/cli/benchmark/score/parse.rs"),
        root.join("src/cli/benchmark/score/normalize.rs"),
        root.join("src/cli/benchmark/score/render.rs"),
        root.join("src/cli/benchmark/score/tests/parse.rs"),
        root.join("src/cli/benchmark/score/tests/normalize.rs"),
        root.join("src/cli/benchmark/score/tests/render.rs"),
    ];
    files.sort();
    files
}

fn actual_benchmark_submodule_files(root: &Path) -> Vec<PathBuf> {
    fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
        let mut entries = fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", dir.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|err| {
                        panic!("failed to read entry in {}: {err}", dir.display())
                    })
                    .path()
            })
            .collect::<Vec<_>>();
        entries.sort();

        for path in entries {
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    for dir in [
        root.join("src/cli/benchmark/run"),
        root.join("src/cli/benchmark/score"),
    ] {
        if dir.is_dir() {
            collect_rs_files(&dir, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_markdown_files(root: &Path, files: &mut Vec<PathBuf>) {
    let mut entries = fs::read_dir(root)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", root.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", root.display()))
                .path()
        })
        .collect::<Vec<_>>();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            collect_markdown_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            files.push(path);
        }
    }
}

fn help_lists_subcommand(help: &str, subcommand: &str) -> bool {
    help.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed == subcommand
            || trimmed
                .strip_prefix(subcommand)
                .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    })
}

fn assert_binary_rejects_benchmark_subcommand(binary: &Path) {
    let help = Command::new(binary)
        .arg("--help")
        .output()
        .unwrap_or_else(|err| panic!("failed to run {} --help: {err}", binary.display()));
    assert!(
        help.status.success(),
        "{} --help failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        binary.display(),
        help.status.code(),
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(
        !help_lists_subcommand(&stdout, "benchmark"),
        "production CLI help must not list internal benchmark harness:\n{stdout}"
    );

    let rejected = Command::new(binary)
        .args(["benchmark", "--help"])
        .output()
        .unwrap_or_else(|err| panic!("failed to run {} benchmark --help: {err}", binary.display()));
    assert!(
        !rejected.status.success(),
        "internal benchmark harness unexpectedly routed through {}",
        binary.display()
    );
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(
        stderr.contains("unrecognized subcommand") && stderr.contains("benchmark"),
        "unexpected benchmark rejection from {}:\nstdout:\n{}\nstderr:\n{}",
        binary.display(),
        String::from_utf8_lossy(&rejected.stdout),
        stderr
    );
}

#[test]
fn benchmark_internal_harness_flat_modules_are_replaced_by_directory_facades() {
    let root = repo_root();
    for flat_module in [
        root.join("src/cli/benchmark/run.rs"),
        root.join("src/cli/benchmark/score.rs"),
    ] {
        assert!(
            !flat_module.exists(),
            "flat benchmark module must be replaced by directory facade: {}",
            flat_module.display()
        );
    }

    for module_dir in [
        root.join("src/cli/benchmark/run"),
        root.join("src/cli/benchmark/score"),
    ] {
        assert!(
            module_dir.is_dir(),
            "expected decomposed benchmark module directory: {}",
            module_dir.display()
        );
    }
}

#[test]
fn benchmark_internal_harness_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let required = expected_benchmark_submodule_files(&root);
    for path in &required {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(path);
    }

    for path in actual_benchmark_submodule_files(&root) {
        assert_module_doc_header(&path);
    }

    for path in [
        "src/cli/benchmark/run/suite/mod.rs",
        "src/cli/benchmark/run/execute/mod.rs",
        "src/cli/benchmark/run/regression/mod.rs",
        "src/cli/benchmark/run/render/mod.rs",
        "src/cli/benchmark/score/parse/mod.rs",
        "src/cli/benchmark/score/normalize/mod.rs",
        "src/cli/benchmark/score/render/mod.rs",
    ] {
        let forbidden = root.join(path);
        assert!(
            !forbidden.exists(),
            "unexpected placeholder module present: {}",
            forbidden.display()
        );
    }
}

#[test]
fn benchmark_internal_harness_submodule_files_stay_under_700_lines() {
    let root = repo_root();
    for module_dir in [
        root.join("src/cli/benchmark/run"),
        root.join("src/cli/benchmark/score"),
    ] {
        assert!(
            module_dir.is_dir(),
            "expected decomposed benchmark module directory: {}",
            module_dir.display()
        );
    }

    for path in actual_benchmark_submodule_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}

#[test]
fn benchmark_internal_harness_contract_pins_runtime_and_docs() {
    let root = repo_root();

    let commands_source = read_source(&root.join("src/cli/commands.rs"));
    for forbidden in ["Benchmark(", "Benchmark {"] {
        assert!(
            !commands_source.contains(forbidden),
            "production Commands enum must not route the internal benchmark harness: found {forbidden}"
        );
    }

    assert_binary_rejects_benchmark_subcommand(Path::new(env!("CARGO_BIN_EXE_biomcp")));

    let release_binary = root.join("target/release/biomcp");
    if release_binary.is_file() {
        assert_binary_rejects_benchmark_subcommand(&release_binary);
    }

    let decomposition_doc =
        read_source(&root.join("architecture/technical/cli-decomposition-2026.md"));
    assert!(
        decomposition_doc.contains("internal harness"),
        "cli-decomposition-2026.md must name benchmark as an internal harness"
    );
    assert!(
        !decomposition_doc.contains("biomcp benchmark "),
        "cli-decomposition-2026.md must not reclaim public biomcp benchmark grammar"
    );

    let decision_doc_path = root.join("architecture/technical/benchmark-cli-ownership-decision.md");
    assert!(
        decision_doc_path.is_file(),
        "benchmark ownership decision doc is required"
    );
    let decision_doc = read_source(&decision_doc_path);
    let decision_doc_header = decision_doc.lines().take(60).collect::<Vec<_>>().join("\n");
    assert!(
        decision_doc_header.contains("Internal/dev regression harness"),
        "benchmark ownership decision must name the internal harness near the top"
    );

    let mut public_docs = vec![root.join("README.md"), root.join("spec/surface/cli.md")];
    for dir in [root.join("docs"), root.join("architecture/ux")] {
        if dir.is_dir() {
            collect_markdown_files(&dir, &mut public_docs);
        }
    }
    public_docs.sort();
    public_docs.dedup();

    for path in public_docs {
        let source = read_source(&path);
        assert!(
            !source.contains("biomcp benchmark"),
            "public docs must not advertise internal benchmark harness grammar: {}",
            path.display()
        );
    }
}
