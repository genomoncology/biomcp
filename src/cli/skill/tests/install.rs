//! Tests for BioMCP skill installation target discovery and filesystem writes.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::BioMcpError;
use crate::test_support::TempDirGuard;

use super::super::assets::canonical_prompt_file_bytes;
use super::super::install::{
    candidate_entries, find_best_target, find_existing_install, install_to_dir,
};

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
