//! Filesystem installation target discovery and copy orchestration for BioMCP skills.

use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use crate::error::BioMcpError;

use super::assets::canonical_prompt_file_bytes;

fn expand_tilde(path: &str) -> Result<PathBuf, BioMcpError> {
    if path == "~" {
        let home = std::env::var("HOME")
            .map_err(|_| BioMcpError::InvalidArgument("HOME is not set".into()))?;
        return Ok(PathBuf::from(home));
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME")
            .map_err(|_| BioMcpError::InvalidArgument("HOME is not set".into()))?;
        return Ok(PathBuf::from(home).join(rest));
    }
    Ok(PathBuf::from(path))
}

fn resolve_install_dir(input: PathBuf) -> PathBuf {
    let ends_with = |path: &Path, a: &str, b: &str| -> bool {
        let mut comps = path.components().rev();
        let Some(last) = comps.next().and_then(|c| c.as_os_str().to_str()) else {
            return false;
        };
        let Some(prev) = comps.next().and_then(|c| c.as_os_str().to_str()) else {
            return false;
        };
        prev == a && last == b
    };

    if ends_with(&input, "skills", "biomcp") {
        return input;
    }

    if input.file_name().and_then(|v| v.to_str()) == Some("skills") {
        return input.join("biomcp");
    }

    input.join("skills").join("biomcp")
}

#[derive(Debug, Clone)]
pub(super) struct CandidateEntry {
    key: &'static str,
    agent_root: PathBuf,
    skills_dir: PathBuf,
    biomcp_dir: PathBuf,
    skill_md: PathBuf,
}

fn candidate_entry(key: &'static str, agent_root: PathBuf, skills_rel: &[&str]) -> CandidateEntry {
    let skills_dir = skills_rel
        .iter()
        .fold(agent_root.clone(), |path, component| path.join(component));
    let biomcp_dir = skills_dir.join("biomcp");
    let skill_md = biomcp_dir.join("SKILL.md");

    CandidateEntry {
        key,
        agent_root,
        skills_dir,
        biomcp_dir,
        skill_md,
    }
}

pub(super) fn candidate_entries(home: &Path, cwd: &Path) -> Vec<CandidateEntry> {
    vec![
        candidate_entry("home-agents", home.join(".agents"), &["skills"]),
        candidate_entry("home-claude", home.join(".claude"), &["skills"]),
        candidate_entry("home-codex", home.join(".codex"), &["skills"]),
        candidate_entry(
            "home-opencode",
            home.join(".config").join("opencode"),
            &["skills"],
        ),
        candidate_entry("home-pi", home.join(".pi"), &["agent", "skills"]),
        candidate_entry("home-gemini", home.join(".gemini"), &["skills"]),
        candidate_entry("cwd-agents", cwd.join(".agents"), &["skills"]),
        candidate_entry("cwd-claude", cwd.join(".claude"), &["skills"]),
    ]
}

pub(super) fn find_existing_install(
    candidates: &[CandidateEntry],
) -> Option<(PathBuf, Vec<PathBuf>)> {
    let mut primary: Option<PathBuf> = None;
    let mut also_found: Vec<PathBuf> = Vec::new();

    for candidate in candidates {
        if !candidate.skill_md.is_file() {
            continue;
        }
        if primary.is_none() {
            primary = Some(candidate.biomcp_dir.clone());
        } else {
            also_found.push(candidate.biomcp_dir.clone());
        }
    }

    primary.map(|path| (path, also_found))
}

fn skills_dir_has_other_skills(skills_dir: &Path) -> bool {
    if !skills_dir.exists() {
        return false;
    }

    let Ok(entries) = fs::read_dir(skills_dir) else {
        return false;
    };

    entries.flatten().any(|entry| {
        if entry.file_name() == "biomcp" {
            return false;
        }

        entry.file_type().is_ok_and(|kind| kind.is_dir())
    })
}

pub(super) fn find_best_target(
    candidates: &[CandidateEntry],
) -> Result<(PathBuf, &'static str), BioMcpError> {
    let mut seen_skills_dirs: HashSet<PathBuf> = HashSet::new();
    let mut populated_entries: Vec<&CandidateEntry> = Vec::new();

    for candidate in candidates {
        if !seen_skills_dirs.insert(candidate.skills_dir.clone()) {
            continue;
        }
        if skills_dir_has_other_skills(&candidate.skills_dir) {
            populated_entries.push(candidate);
        }
    }

    if let Some(home_agents) = populated_entries
        .iter()
        .find(|candidate| candidate.key == "home-agents")
    {
        return Ok((
            home_agents.biomcp_dir.clone(),
            "existing skills directory detected",
        ));
    }

    if let Some(first_populated) = populated_entries.first() {
        return Ok((
            first_populated.biomcp_dir.clone(),
            "existing skills directory detected",
        ));
    }

    if let Some(home_agents) = candidates
        .iter()
        .find(|candidate| candidate.key == "home-agents")
        && home_agents.agent_root.exists()
    {
        return Ok((
            home_agents.biomcp_dir.clone(),
            "existing agent root detected",
        ));
    }

    if let Some(home_claude) = candidates
        .iter()
        .find(|candidate| candidate.key == "home-claude")
        && home_claude.agent_root.exists()
    {
        return Ok((
            home_claude.biomcp_dir.clone(),
            "existing agent root detected",
        ));
    }

    if let Some(first_existing_root) = candidates
        .iter()
        .find(|candidate| candidate.agent_root.exists())
    {
        return Ok((
            first_existing_root.biomcp_dir.clone(),
            "existing agent root detected",
        ));
    }

    let home_agents = candidates
        .iter()
        .find(|candidate| candidate.key == "home-agents")
        .ok_or_else(|| {
            BioMcpError::InvalidArgument("Missing home-agents install candidate".into())
        })?;

    Ok((
        home_agents.biomcp_dir.clone(),
        "no existing agent directories found; using cross-tool default",
    ))
}

fn prompt_confirm(path: &Path) -> Result<bool, BioMcpError> {
    let mut stderr = io::stderr();
    write!(
        &mut stderr,
        "Install BioMCP skills to {}? [y/N]: ",
        path.display()
    )
    .map_err(BioMcpError::Io)?;
    stderr.flush().map_err(BioMcpError::Io)?;

    let mut line = String::new();
    io::stdin().read_line(&mut line).map_err(BioMcpError::Io)?;
    let ans = line.trim().to_ascii_lowercase();
    Ok(ans == "y" || ans == "yes")
}

fn write_stderr_line(line: &str) -> Result<(), BioMcpError> {
    let mut stderr = io::stderr();
    writeln!(&mut stderr, "{line}").map_err(BioMcpError::Io)
}

pub(super) fn install_to_dir(dir: &Path, force: bool) -> Result<String, BioMcpError> {
    let target = dir.to_path_buf();
    let installed_marker = target.join("SKILL.md");
    if installed_marker.exists() && !force {
        return Ok(format!(
            "Skills already installed at {} (use --force to replace)",
            target.display()
        ));
    }

    // Write into a sibling temp directory, then swap into place.
    // This avoids the remove_dir_all + create_dir_all race (EEXIST on
    // macOS) and ensures stale files from older releases are cleaned up.
    let parent = target.parent().ok_or_else(|| {
        BioMcpError::InvalidArgument("Install path has no parent directory".into())
    })?;
    fs::create_dir_all(parent)?;
    let staging = parent.join(".biomcp-install-tmp");
    if staging.exists() {
        fs::remove_dir_all(&staging)?;
    }
    fs::create_dir(&staging)?;

    for file in crate::skill_assets::iter() {
        let rel = file.as_ref();
        let Ok(asset) = crate::skill_assets::bytes(rel) else {
            continue;
        };

        let out_path = staging.join(rel);
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        let bytes = if rel == "SKILL.md" {
            canonical_prompt_file_bytes()?
        } else {
            asset.into_owned()
        };
        fs::write(&out_path, bytes)?;
    }

    // Swap: remove old target (if any), rename staging into place.
    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    fs::rename(&staging, &target)
        .map_err(BioMcpError::Io)
        .or_else(|_| {
            // rename fails across filesystems; fall back to copy + remove.
            copy_dir_all(&staging, &target)?;
            fs::remove_dir_all(&staging).map_err(BioMcpError::Io)
        })?;

    Ok(format!("Installed BioMCP skills to {}", target.display()))
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), BioMcpError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).map_err(BioMcpError::Io)? {
        let entry = entry.map_err(BioMcpError::Io)?;
        let dest = dst.join(entry.file_name());
        if entry.file_type().map_err(BioMcpError::Io)?.is_dir() {
            copy_dir_all(&entry.path(), &dest)?;
        } else {
            fs::write(&dest, fs::read(entry.path()).map_err(BioMcpError::Io)?)?;
        }
    }
    Ok(())
}

/// Installs embedded skills into a supported agent directory.
///
/// # Errors
///
/// Returns an error when the destination path is invalid, not writable, or no
/// supported installation directory can be determined.
pub fn install_skills(dir: Option<&str>, force: bool) -> Result<String, BioMcpError> {
    if let Some(dir) = dir {
        let base = expand_tilde(dir)?;
        let target = resolve_install_dir(base);
        return install_to_dir(&target, force);
    }

    let home = expand_tilde("~")?;
    let cwd = std::env::current_dir().map_err(BioMcpError::Io)?;
    let candidates = candidate_entries(&home, &cwd);

    let (target, reason, also_found) =
        if let Some((target, also_found)) = find_existing_install(&candidates) {
            (target, "existing BioMCP skill found", also_found)
        } else {
            let (target, reason) = find_best_target(&candidates)?;
            (target, reason, Vec::new())
        };

    if !also_found.is_empty() {
        let extra = also_found
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write_stderr_line(&format!("Note: BioMCP skill also found at: {extra}"))?;
    }

    write_stderr_line(&format!("Auto-detected: {} ({reason})", target.display()))?;

    if std::io::stdin().is_terminal() && !prompt_confirm(&target)? {
        return Ok("No installation selected".into());
    }

    install_to_dir(&target, force)
}
