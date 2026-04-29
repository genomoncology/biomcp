use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const CLI_LINE_CAP: usize = 700;
const TICKET_SLUG: &str = "347-decompose-residual-over-cap-src-cli-files-under-global-ratchet";
const RESIDUAL_PATHS: &[&str] = &[
    "src/cli/drug/tests.rs",
    "src/cli/trial/tests.rs",
    "src/cli/cache.rs",
    "src/cli/article/session.rs",
    "src/cli/article/dispatch.rs",
    "src/cli/variant/dispatch.rs",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn tracked_cli_rust_files(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("ls-files")
        .arg("--")
        .arg("src/cli/*.rs")
        .arg("src/cli/**/*.rs")
        .output()
        .unwrap_or_else(|err| panic!("failed to run git ls-files: {err}"));

    assert!(
        output.status.success(),
        "git ls-files failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("git ls-files output should be utf-8")
        .lines()
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[test]
fn ticket_347_residual_allowlist_entries_are_absorbed() {
    let root = repo_root();
    let allowlist_path = root.join("tools/cli-line-cap-allowlist.json");
    let allowlist: serde_json::Value = serde_json::from_str(&read_source(&allowlist_path))
        .unwrap_or_else(|err| panic!("invalid allowlist JSON {}: {err}", allowlist_path.display()));
    let entries = allowlist["entries"].as_array().unwrap_or_else(|| {
        panic!(
            "allowlist entries must be an array in {}",
            allowlist_path.display()
        )
    });

    let residual_paths = RESIDUAL_PATHS.iter().copied().collect::<BTreeSet<_>>();
    let residual_entries = entries
        .iter()
        .filter_map(|entry| entry.get("path").and_then(serde_json::Value::as_str))
        .filter(|path| residual_paths.contains(path))
        .collect::<Vec<_>>();
    assert!(
        residual_entries.is_empty(),
        "ticket 347 residual allowlist entries must be removed after decomposition: {residual_entries:?}"
    );

    let ticket_entries = entries
        .iter()
        .filter(|entry| {
            entry
                .get("follow_up_ticket")
                .and_then(serde_json::Value::as_str)
                == Some(TICKET_SLUG)
        })
        .filter_map(|entry| entry.get("path").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        ticket_entries.is_empty(),
        "ticket 347 follow-up entries must be fully absorbed, found: {ticket_entries:?}"
    );
}

#[test]
fn tracked_src_cli_rust_files_stay_under_global_cap() {
    let root = repo_root();
    let tracked = tracked_cli_rust_files(&root);
    assert!(
        tracked
            .iter()
            .any(|path| path.starts_with("src/cli/") && path.ends_with(".rs")),
        "git ls-files did not return tracked src/cli Rust files"
    );

    let over_cap = tracked
        .iter()
        .filter_map(|relative_path| {
            let line_count = read_source(&root.join(relative_path)).lines().count();
            (line_count > CLI_LINE_CAP).then(|| format!("{relative_path}: {line_count}"))
        })
        .collect::<Vec<_>>();

    assert!(
        over_cap.is_empty(),
        "tracked src/cli Rust files must be at or below {CLI_LINE_CAP} lines: {over_cap:?}"
    );
}
