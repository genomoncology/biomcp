//! Packaged skill installation preserves caller files on supported platforms.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn output(root: &Path, args: &[&str]) -> Output {
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
    command
        .current_dir(root)
        .env("HOME", root)
        .env("USERPROFILE", root)
        .env("XDG_CONFIG_HOME", root)
        .envs(biomcp_mcp_contract_client::child_cache_env(
            root.join("cache"),
        ))
        .args(args)
        .output()
        .expect("run isolated skill command")
}

fn run(root: &Path, args: &[&str]) -> Output {
    let output = output(root, args);
    assert!(output.status.success(), "{args:?}: {output:?}");
    output
}

#[test]
fn skill_install_help_explains_windows_manual_repair() {
    let root = tempfile::tempdir().unwrap();
    let help = run(root.path(), &["skill", "install", "--help"]);
    let help = String::from_utf8(help.stdout).unwrap();
    let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(help.contains("not supported on Windows"), "{help}");
    assert!(help.contains("Back up unrelated files"), "{help}");
    assert!(help.contains("skills/biomcp"), "{help}");
    assert!(help.contains("without --force"), "{help}");
}

#[cfg(windows)]
fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn collect(
        root: &Path,
        directory: &Path,
        files: &mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut files = std::collections::BTreeMap::new();
    collect(root, root, &mut files);
    files
}

#[cfg(windows)]
#[test]
fn windows_force_refuses_preserves_bundle_and_manual_reinstall_succeeds() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    run(root, &["skill", "install", "."]);
    let target = root.join("skills/biomcp");
    fs::write(target.join("SKILL.md"), "my edited guide").unwrap();
    fs::write(target.join("personal.txt"), "my notes").unwrap();
    fs::create_dir(root.join("skills/other")).unwrap();
    fs::write(root.join("skills/other/SKILL.md"), "another skill").unwrap();
    let before = snapshot(&target);

    let refused = output(root, &["skill", "install", ".", "--force"]);
    assert_eq!(refused.status.code(), Some(2));
    let message = String::from_utf8(refused.stderr).unwrap();
    for required in [
        "unsupported on Windows",
        "Back up unrelated files",
        "skills/biomcp",
        "biomcp skill install",
        "without --force",
    ] {
        assert!(message.contains(required), "{message}");
    }
    assert!(
        snapshot(&target) == before,
        "forced repair changed existing skill files"
    );
    assert!(fs::read_dir(root.join("skills")).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".biomcp-install-")
    }));

    // Follow the documented manual procedure, keeping the backup outside the
    // removed directory and restoring unrelated files after the fresh install.
    let backup = root.join("personal-backup.txt");
    fs::copy(target.join("personal.txt"), &backup).unwrap();
    fs::remove_dir_all(&target).unwrap();
    run(root, &["skill", "install", "."]);
    fs::copy(&backup, target.join("personal.txt")).unwrap();
    let rendered = run(root, &["skill", "render"]);
    assert!(fs::read(target.join("SKILL.md")).unwrap() == rendered.stdout);
    let status = run(root, &["skill", "status", ".", "--json"]);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["state"], "current");
    assert_eq!(
        fs::read_to_string(target.join("personal.txt")).unwrap(),
        "my notes"
    );
    assert_eq!(
        fs::read_to_string(root.join("skills/other/SKILL.md")).unwrap(),
        "another skill"
    );
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
    let installed_guide = fs::read(target.join("SKILL.md")).unwrap();
    assert!(
        installed_guide == rendered.stdout,
        "guide bytes differ: installed length {}, rendered length {}, first mismatch {:?}",
        installed_guide.len(),
        rendered.stdout.len(),
        installed_guide
            .iter()
            .zip(&rendered.stdout)
            .position(|(left, right)| left != right)
    );
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
