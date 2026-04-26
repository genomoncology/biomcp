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

fn expected_health_submodule_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/cli/health/mod.rs"),
        root.join("src/cli/health/catalog.rs"),
        root.join("src/cli/health/http.rs"),
        root.join("src/cli/health/local.rs"),
        root.join("src/cli/health/runner.rs"),
        root.join("src/cli/health/tests/catalog.rs"),
        root.join("src/cli/health/tests/http.rs"),
        root.join("src/cli/health/tests/local.rs"),
        root.join("src/cli/health/tests/runner.rs"),
    ];
    files.sort();
    files
}

fn actual_health_submodule_files(root: &Path) -> Vec<PathBuf> {
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

    let health_dir = root.join("src/cli/health");
    if !health_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rs_files(&health_dir, &mut files);
    files
}

#[test]
fn health_flat_module_is_replaced_by_directory_facade() {
    let root = repo_root();
    let flat_module = root.join("src/cli/health.rs");
    assert!(
        !flat_module.exists(),
        "flat health.rs must be replaced by src/cli/health/mod.rs"
    );

    let health_dir = root.join("src/cli/health");
    assert!(
        health_dir.is_dir(),
        "expected decomposed health module directory: {}",
        health_dir.display()
    );
}

#[test]
fn health_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let expected = expected_health_submodule_files(&root);
    assert_eq!(
        actual_health_submodule_files(&root),
        expected,
        "unexpected Rust file layout under src/cli/health"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(&path);
    }

    for path in [
        "src/cli/health/catalog/mod.rs",
        "src/cli/health/http/mod.rs",
        "src/cli/health/local/mod.rs",
        "src/cli/health/runner/mod.rs",
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
fn health_submodule_files_stay_under_700_lines() {
    let root = repo_root();
    let health_dir = root.join("src/cli/health");
    assert!(
        health_dir.is_dir(),
        "expected decomposed health module directory: {}",
        health_dir.display()
    );

    for path in actual_health_submodule_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}
