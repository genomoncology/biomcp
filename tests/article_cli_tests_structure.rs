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

fn expected_article_cli_test_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/cli/article/tests/mod.rs"),
        root.join("src/cli/article/tests/help.rs"),
        root.join("src/cli/article/tests/exact_lookup.rs"),
        root.join("src/cli/article/tests/json.rs"),
        root.join("src/cli/article/tests/filters.rs"),
    ];
    files.sort();
    files
}

fn actual_article_cli_test_files(root: &Path) -> Vec<PathBuf> {
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

    let article_tests_dir = root.join("src/cli/article/tests");
    if !article_tests_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rs_files(&article_tests_dir, &mut files);
    files
}

#[test]
fn article_cli_flat_test_sidecar_is_replaced_by_directory() {
    let root = repo_root();
    let flat_sidecar = root.join("src/cli/article/tests.rs");
    assert!(
        !flat_sidecar.exists(),
        "flat article tests.rs must be replaced by src/cli/article/tests/mod.rs"
    );

    let article_tests_dir = root.join("src/cli/article/tests");
    assert!(
        article_tests_dir.is_dir(),
        "expected decomposed article CLI test directory: {}",
        article_tests_dir.display()
    );
}

#[test]
fn article_cli_test_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let expected = expected_article_cli_test_files(&root);
    assert_eq!(
        actual_article_cli_test_files(&root),
        expected,
        "unexpected Rust file layout under src/cli/article/tests"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(&path);
    }

    for path in [
        "src/cli/article/tests/help/mod.rs",
        "src/cli/article/tests/exact_lookup/mod.rs",
        "src/cli/article/tests/json/mod.rs",
        "src/cli/article/tests/filters/mod.rs",
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
fn article_cli_test_sidecar_files_stay_under_700_lines() {
    let root = repo_root();
    let article_tests_dir = root.join("src/cli/article/tests");
    assert!(
        article_tests_dir.is_dir(),
        "expected decomposed article CLI test directory: {}",
        article_tests_dir.display()
    );

    for path in actual_article_cli_test_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}
