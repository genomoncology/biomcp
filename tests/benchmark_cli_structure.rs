use std::fs;
use std::path::{Path, PathBuf};

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

#[test]
fn benchmark_flat_modules_are_replaced_by_directory_facades() {
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
fn benchmark_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let expected = expected_benchmark_submodule_files(&root);
    assert_eq!(
        actual_benchmark_submodule_files(&root),
        expected,
        "unexpected Rust file layout under src/cli/benchmark/run and src/cli/benchmark/score"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
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
fn benchmark_submodule_files_stay_under_700_lines() {
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
