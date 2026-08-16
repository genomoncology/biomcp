//! Tests for BioMCP skill installation target discovery and filesystem writes.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;

use super::super::assets::canonical_prompt_file_bytes;
use super::super::install::{
    candidate_entries, find_best_target, find_existing_install, install_to_dir,
};
use super::super::status::{MANIFEST_NAME, skill_status};

fn status_state(target: &Path) -> String {
    let target = target.to_str().expect("UTF-8 test path");
    let json = skill_status(Some(target), true).expect("skill status");
    serde_json::from_str::<serde_json::Value>(&json).expect("status JSON")["state"]
        .as_str()
        .expect("state string")
        .to_string()
}

struct TestPaths {
    _guard: TempDirGuard,
    home: PathBuf,
    cwd: PathBuf,
}

impl TestPaths {
    fn new(name: &str) -> Self {
        let guard = TempDirGuard::new(&format!("skill-{name}"));
        let root = guard.path();
        let home = root.join("home");
        let cwd = root.join("cwd");

        fs::create_dir_all(&home).expect("create test home dir");
        fs::create_dir_all(&cwd).expect("create test cwd dir");

        Self {
            _guard: guard,
            home,
            cwd,
        }
    }

    fn create_file(&self, path: &Path) {
        let parent = path.parent().expect("path has parent");
        fs::create_dir_all(parent).expect("create parent dirs");
        fs::write(path, "# test").expect("write test file");
    }
}

#[test]
fn install_to_dir_writes_canonical_skill_md_and_assets() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("install-canonical-skill");
    let target = paths.cwd.join("skills/biomcp");

    install_to_dir(&target, true)?;

    assert_eq!(
        fs::read(target.join("SKILL.md"))?,
        canonical_prompt_file_bytes()?
    );
    assert!(target.join("use-cases").is_dir());
    assert!(target.join("jq-examples.md").is_file());
    assert!(target.join("examples").is_dir());
    assert!(target.join("schemas").is_dir());

    Ok(())
}

#[test]
fn managed_status_distinguishes_current_unmanaged_stale_and_modified() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("managed-status-states");
    let target = paths.cwd.join("skills/biomcp");

    assert_eq!(status_state(&target), "missing");
    let missing_json = skill_status(Some(target.to_str().expect("UTF-8 test path")), true)?;
    assert!(!missing_json.contains(&target.display().to_string()));
    install_to_dir(&target, true)?;
    assert_eq!(status_state(&target), "current");

    let sidecar = target.join(MANIFEST_NAME);
    let original_manifest = fs::read(&sidecar)?;
    fs::remove_file(&sidecar)?;
    assert_eq!(status_state(&target), "unmanaged");

    fs::write(&sidecar, b"not JSON")?;
    assert_eq!(status_state(&target), "unmanaged");

    let mut manifest: serde_json::Value = serde_json::from_slice(&original_manifest)?;
    manifest["schema_version"] = serde_json::Value::from(2);
    fs::write(&sidecar, serde_json::to_vec_pretty(&manifest)?)?;
    assert_eq!(status_state(&target), "unmanaged");

    manifest = serde_json::from_slice(&original_manifest)?;
    manifest["biomcp_version"] = serde_json::Value::String("0.0.0".into());
    fs::write(&sidecar, serde_json::to_vec_pretty(&manifest)?)?;
    assert_eq!(status_state(&target), "stale");

    fs::write(&sidecar, &original_manifest)?;
    fs::remove_file(target.join("jq-examples.md"))?;
    assert_eq!(status_state(&target), "locally_modified");

    fs::write(
        target.join("jq-examples.md"),
        crate::skill_assets::bytes("jq-examples.md")?,
    )?;
    fs::write(target.join("SKILL.md"), "locally changed")?;
    assert_eq!(status_state(&target), "locally_modified");
    Ok(())
}

#[test]
fn malformed_manifest_cannot_authorize_removing_unrelated_files() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("malformed-manifest-removal");
    let target = paths.cwd.join("skills/biomcp");
    install_to_dir(&target, true)?;
    fs::write(target.join("notes.txt"), "keep me")?;

    let sidecar = target.join(MANIFEST_NAME);
    let mut manifest: serde_json::Value = serde_json::from_slice(&fs::read(&sidecar)?)?;
    manifest["managed_files"]["notes.txt"] = serde_json::Value::String("not-a-digest".into());
    fs::write(&sidecar, serde_json::to_vec_pretty(&manifest)?)?;

    assert_eq!(status_state(&target), "unmanaged");
    install_to_dir(&target, true)?;
    assert_eq!(fs::read_to_string(target.join("notes.txt"))?, "keep me");
    assert_eq!(status_state(&target), "current");
    Ok(())
}

#[test]
fn force_rejects_a_directory_at_a_manifest_recorded_file_path() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("managed-file-became-directory");
    let target = paths.cwd.join("skills/biomcp");
    install_to_dir(&target, true)?;
    fs::remove_file(target.join("SKILL.md"))?;
    fs::create_dir(target.join("SKILL.md"))?;
    fs::write(target.join("SKILL.md/keep.txt"), "keep me")?;

    let error = install_to_dir(&target, true).expect_err("directory must not be deleted");
    assert!(error.to_string().contains("not a regular file"));
    assert_eq!(
        fs::read_to_string(target.join("SKILL.md/keep.txt"))?,
        "keep me"
    );
    Ok(())
}

#[test]
fn plain_install_preserves_edits_and_force_repairs_managed_files() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("force-repair");
    let target = paths.cwd.join("skills/biomcp");
    install_to_dir(&target, true)?;
    fs::write(target.join("notes.txt"), "keep me")?;
    fs::write(target.join("SKILL.md"), "local edit")?;

    install_to_dir(&target, false)?;
    assert_eq!(fs::read_to_string(target.join("SKILL.md"))?, "local edit");
    assert_eq!(status_state(&target), "locally_modified");

    install_to_dir(&target, true)?;
    assert_eq!(status_state(&target), "current");
    assert_eq!(fs::read_to_string(target.join("notes.txt"))?, "keep me");
    Ok(())
}

#[test]
fn install_result_reports_installed_unchanged_and_repaired_truthfully() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("typed-install-result");
    let target = paths.cwd.join("skills/biomcp");

    let installed = serde_json::to_value(install_to_dir(&target, false)?)?;
    assert_eq!(installed["status"], "installed");
    assert_eq!(installed["changed"], true);
    assert_eq!(installed["skill_status"]["state"], "current");

    let unchanged = serde_json::to_value(install_to_dir(&target, false)?)?;
    assert_eq!(unchanged["status"], "unchanged");
    assert_eq!(unchanged["changed"], false);
    assert_eq!(unchanged["skill_status"]["state"], "current");

    fs::write(target.join("SKILL.md"), "local edit")?;
    let repaired = serde_json::to_value(install_to_dir(&target, true)?)?;
    assert_eq!(repaired["status"], "repaired");
    assert_eq!(repaired["changed"], true);
    assert_eq!(repaired["skill_status"]["state"], "current");
    assert_eq!(repaired["target"], target.display().to_string());
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn fresh_atomic_rename_refuses_a_racing_target() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("fresh-race");
    let source = paths.cwd.join("candidate");
    let target = paths.cwd.join("target");
    fs::create_dir(&source)?;
    fs::write(source.join("SKILL.md"), "candidate")?;
    fs::create_dir(&target)?;
    fs::write(target.join("racing-file"), "preserve")?;

    super::super::atomic_swap::rename_absent(&source, &target)
        .expect_err("a racing target must not be replaced");
    assert_eq!(fs::read_to_string(source.join("SKILL.md"))?, "candidate");
    assert_eq!(fs::read_to_string(target.join("racing-file"))?, "preserve");
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn failed_atomic_exchange_leaves_existing_target_unchanged() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("exchange-failure");
    let target = paths.cwd.join("skills/biomcp");
    install_to_dir(&target, true)?;
    fs::write(target.join("SKILL.md"), "old complete tree")?;

    super::super::atomic_swap::fail_next_exchange();
    let error = install_to_dir(&target, true).expect_err("exchange should fail");
    assert!(
        error
            .to_string()
            .contains("injected atomic exchange failure")
    );
    assert_eq!(
        fs::read_to_string(target.join("SKILL.md"))?,
        "old complete tree"
    );
    assert!(
        fs::read_dir(target.parent().expect("target parent"))?.all(|entry| {
            !entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".biomcp-install-")
        })
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn force_preserves_unrelated_symlinks_without_following_them() -> Result<(), BioMcpError> {
    use std::os::unix::fs::symlink;

    let paths = TestPaths::new("preserve-symlink");
    let target = paths.cwd.join("skills/biomcp");
    install_to_dir(&target, true)?;
    let outside = paths.cwd.join("outside.txt");
    fs::write(&outside, "outside")?;
    symlink(&outside, target.join("unrelated-link"))?;

    install_to_dir(&target, true)?;
    assert_eq!(fs::read_link(target.join("unrelated-link"))?, outside);
    assert_eq!(fs::read_to_string(&outside)?, "outside");
    Ok(())
}

#[test]
fn find_existing_install_detects_claude() {
    let paths = TestPaths::new("existing-claude");
    let skill_md = paths.home.join(".claude/skills/biomcp/SKILL.md");
    paths.create_file(&skill_md);

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, also_found) =
        find_existing_install(&candidates).expect("expected existing install");

    assert_eq!(target, paths.home.join(".claude/skills/biomcp"));
    assert!(also_found.is_empty());
}

#[test]
fn find_existing_install_prefers_agents_and_reports_others() {
    let paths = TestPaths::new("existing-prefer-agents");
    paths.create_file(&paths.home.join(".agents/skills/biomcp/SKILL.md"));
    paths.create_file(&paths.home.join(".claude/skills/biomcp/SKILL.md"));

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, also_found) =
        find_existing_install(&candidates).expect("expected existing installs");

    assert_eq!(target, paths.home.join(".agents/skills/biomcp"));
    assert_eq!(also_found, vec![paths.home.join(".claude/skills/biomcp")]);
}

#[test]
fn find_existing_install_ignores_skill_md_directory() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("existing-ignore-directory");
    fs::create_dir_all(paths.home.join(".claude/skills/biomcp/SKILL.md"))?;

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let existing = find_existing_install(&candidates);

    assert!(existing.is_none());
    Ok(())
}

#[test]
fn find_best_target_prefers_agents_populated_skills_dir() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("best-populated-prefer-agents");
    paths.create_file(&paths.home.join(".agents/skills/example/SKILL.md"));
    paths.create_file(&paths.home.join(".claude/skills/other/SKILL.md"));

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, reason) = find_best_target(&candidates)?;

    assert_eq!(target, paths.home.join(".agents/skills/biomcp"));
    assert_eq!(reason, "existing skills directory detected");
    Ok(())
}

#[test]
fn find_best_target_ignores_non_skill_files_in_skills_dir() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("best-ignore-non-skill-files");
    paths.create_file(&paths.home.join(".claude/skills/.DS_Store"));
    paths.create_file(&paths.home.join(".codex/skills/example/SKILL.md"));

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, reason) = find_best_target(&candidates)?;

    assert_eq!(target, paths.home.join(".codex/skills/biomcp"));
    assert_eq!(reason, "existing skills directory detected");
    Ok(())
}

#[test]
fn find_best_target_falls_back_to_agents_root_then_claude_root() -> Result<(), BioMcpError> {
    let agents = TestPaths::new("best-root-agents");
    fs::create_dir_all(agents.home.join(".agents"))?;
    let (agents_target, agents_reason) =
        find_best_target(&candidate_entries(&agents.home, &agents.cwd))?;
    assert_eq!(agents_target, agents.home.join(".agents/skills/biomcp"));
    assert_eq!(agents_reason, "existing agent root detected");

    let claude = TestPaths::new("best-root-claude");
    fs::create_dir_all(claude.home.join(".claude"))?;
    let (claude_target, claude_reason) =
        find_best_target(&candidate_entries(&claude.home, &claude.cwd))?;
    assert_eq!(claude_target, claude.home.join(".claude/skills/biomcp"));
    assert_eq!(claude_reason, "existing agent root detected");

    Ok(())
}

#[test]
fn find_best_target_preserves_pi_agent_skills_path() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("best-pi");
    fs::create_dir_all(paths.home.join(".pi"))?;

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, reason) = find_best_target(&candidates)?;

    assert_eq!(target, paths.home.join(".pi/agent/skills/biomcp"));
    assert_eq!(reason, "existing agent root detected");
    Ok(())
}

#[test]
fn find_best_target_defaults_to_home_agents_when_nothing_exists() -> Result<(), BioMcpError> {
    let paths = TestPaths::new("best-default");

    let candidates = candidate_entries(&paths.home, &paths.cwd);
    let (target, reason) = find_best_target(&candidates)?;

    assert_eq!(target, paths.home.join(".agents/skills/biomcp"));
    assert_eq!(
        reason,
        "no existing agent directories found; using cross-tool default"
    );
    Ok(())
}
