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

fn expected_skill_submodule_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        root.join("src/cli/skill/mod.rs"),
        root.join("src/cli/skill/assets.rs"),
        root.join("src/cli/skill/catalog.rs"),
        root.join("src/cli/skill/install.rs"),
        root.join("src/cli/skill/tests/catalog.rs"),
        root.join("src/cli/skill/tests/install.rs"),
    ];
    files.sort();
    files
}

fn actual_skill_submodule_files(root: &Path) -> Vec<PathBuf> {
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

    let skill_dir = root.join("src/cli/skill");
    if !skill_dir.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    collect_rs_files(&skill_dir, &mut files);
    files
}

#[test]
fn skill_flat_module_is_replaced_by_directory_facade() {
    let root = repo_root();
    let flat_module = root.join("src/cli/skill.rs");
    assert!(
        !flat_module.exists(),
        "flat skill.rs must be replaced by src/cli/skill/mod.rs"
    );

    let skill_dir = root.join("src/cli/skill");
    assert!(
        skill_dir.is_dir(),
        "expected decomposed skill module directory: {}",
        skill_dir.display()
    );
}

#[test]
fn skill_split_files_exist_with_doc_headers() {
    let root = repo_root();
    let expected = expected_skill_submodule_files(&root);
    assert_eq!(
        actual_skill_submodule_files(&root),
        expected,
        "unexpected Rust file layout under src/cli/skill"
    );

    for path in expected {
        assert!(path.is_file(), "missing expected file: {}", path.display());
        assert_module_doc_header(&path);
    }

    for path in [
        "src/cli/skill/assets/mod.rs",
        "src/cli/skill/catalog/mod.rs",
        "src/cli/skill/install/mod.rs",
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
fn skill_submodule_files_stay_under_700_lines() {
    let root = repo_root();
    let skill_dir = root.join("src/cli/skill");
    assert!(
        skill_dir.is_dir(),
        "expected decomposed skill module directory: {}",
        skill_dir.display()
    );

    for path in actual_skill_submodule_files(&root) {
        let line_count = read_source(&path).lines().count();
        assert!(
            line_count <= 700,
            "{} exceeds 700 lines: {}",
            path.display(),
            line_count
        );
    }
}
