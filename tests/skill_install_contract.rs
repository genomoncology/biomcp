//! Packaged skill installation preserves caller files on supported platforms.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(root: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.env_clear();
    for name in [
        "PATH",
        "SYSTEMROOT",
        "WINDIR",
        "TEMP",
        "TMP",
        "USERNAME",
        "LLVM_PROFILE_FILE",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let output = command
        .current_dir(root)
        .env("HOME", root)
        .env("USERPROFILE", root)
        .env("XDG_CONFIG_HOME", root)
        .envs(biomcp_mcp_contract_client::child_cache_env(
            root.join("cache"),
        ))
        .args(args)
        .output()
        .expect("run isolated skill command");
    assert!(output.status.success(), "{args:?}: {output:?}");
    output
}

#[test]
fn fresh_skill_install_matches_render_and_repeat_preserves_user_files() {
    let root = tempfile::tempdir().expect("isolated skill installation");
    let root = root.path();
    let rendered = run(root, &["skill", "render"]);
    let installed = run(root, &["skill", "install", ".", "--json"]);
    let installed: serde_json::Value = serde_json::from_slice(&installed.stdout).unwrap();
    assert_eq!(installed["status"], "installed");
    assert_eq!(installed["skill_status"]["state"], "current");
    let target = root.join("skills/biomcp");
    assert_eq!(fs::read(target.join("SKILL.md")).unwrap(), rendered.stdout);
    let status = run(root, &["skill", "status", ".", "--json"]);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["state"], "current");

    fs::write(target.join("SKILL.md"), "my guide").unwrap();
    fs::write(target.join("personal.txt"), "my notes").unwrap();
    let repeated = run(root, &["skill", "install", ".", "--json"]);
    let repeated: serde_json::Value = serde_json::from_slice(&repeated.stdout).unwrap();
    assert_eq!(repeated["status"], "unchanged");
    assert_eq!(repeated["changed"], false);
    assert_eq!(repeated["skill_status"]["state"], "locally_modified");
    assert_eq!(
        fs::read_to_string(target.join("SKILL.md")).unwrap(),
        "my guide"
    );
    assert_eq!(
        fs::read_to_string(target.join("personal.txt")).unwrap(),
        "my notes"
    );
    assert_eq!(fs::read_dir(root.join("skills")).unwrap().count(), 1);
}
