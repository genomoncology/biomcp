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

fn expected_article_submodule_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/transform/article/anchors.rs"),
        root.join("src/transform/article/anchors/tests.rs"),
        root.join("src/transform/article/annotations.rs"),
        root.join("src/transform/article/annotations/tests.rs"),
        root.join("src/transform/article/federation.rs"),
        root.join("src/transform/article/federation/tests.rs"),
        root.join("src/transform/article/html.rs"),
        root.join("src/transform/article/jats.rs"),
        root.join("src/transform/article/jats/refs.rs"),
        root.join("src/transform/article/jats/supplements.rs"),
        root.join("src/transform/article/jats/tables.rs"),
        root.join("src/transform/article/jats/tests.rs"),
        root.join("src/transform/article/pdf.rs"),
    ];
    files.sort();
    files
}

fn actual_article_submodule_files(root: &Path) -> Vec<PathBuf> {
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
    collect_rs_files(&root.join("src/transform/article"), &mut files);
    files
}

#[test]
fn transform_article_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let facade = root.join("src/transform/article.rs");
    assert_module_doc_header(&facade);

    let expected = expected_article_submodule_files(&root);
    assert_eq!(
        actual_article_submodule_files(&root),
        expected,
        "unexpected Rust file layout under src/transform/article"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(&path);
    }

    for path in [
        "src/transform/article/ranking.rs",
        "src/transform/article/ranking/mod.rs",
        "src/transform/article/calibration.rs",
        "src/transform/article/calibration/mod.rs",
        "src/transform/article/types.rs",
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
fn transform_article_submodule_files_stay_under_700_lines() {
    let root = repo_root();
    let article_dir = root.join("src/transform/article");
    assert!(
        article_dir.is_dir(),
        "expected article submodule directory: {}",
        article_dir.display()
    );

    for path in actual_article_submodule_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}
