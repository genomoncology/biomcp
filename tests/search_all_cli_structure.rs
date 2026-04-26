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

fn expected_search_all_submodule_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/cli/search_all/mod.rs"),
        root.join("src/cli/search_all/plan.rs"),
        root.join("src/cli/search_all/dispatch.rs"),
        root.join("src/cli/search_all/links.rs"),
        root.join("src/cli/search_all/format.rs"),
        root.join("src/cli/search_all/tests/plan.rs"),
        root.join("src/cli/search_all/tests/dispatch.rs"),
        root.join("src/cli/search_all/tests/links.rs"),
        root.join("src/cli/search_all/tests/format.rs"),
    ];
    files.sort();
    files
}

fn actual_search_all_submodule_files(root: &Path) -> Vec<PathBuf> {
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

    let search_all_dir = root.join("src/cli/search_all");
    if !search_all_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rs_files(&search_all_dir, &mut files);
    files
}

#[test]
fn search_all_flat_module_is_replaced_by_directory_facade() {
    let root = repo_root();
    let flat_module = root.join("src/cli/search_all.rs");
    assert!(
        !flat_module.exists(),
        "flat search_all.rs must be replaced by src/cli/search_all/mod.rs"
    );

    let search_all_dir = root.join("src/cli/search_all");
    assert!(
        search_all_dir.is_dir(),
        "expected decomposed search_all module directory: {}",
        search_all_dir.display()
    );
}

#[test]
fn search_all_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let expected = expected_search_all_submodule_files(&root);
    assert_eq!(
        actual_search_all_submodule_files(&root),
        expected,
        "unexpected Rust file layout under src/cli/search_all"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(&path);
    }

    for path in [
        "src/cli/search_all/plan/mod.rs",
        "src/cli/search_all/dispatch/mod.rs",
        "src/cli/search_all/links/mod.rs",
        "src/cli/search_all/format/mod.rs",
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
fn search_all_submodule_files_stay_under_700_lines() {
    let root = repo_root();
    let search_all_dir = root.join("src/cli/search_all");
    assert!(
        search_all_dir.is_dir(),
        "expected decomposed search_all module directory: {}",
        search_all_dir.display()
    );

    for path in actual_search_all_submodule_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}
