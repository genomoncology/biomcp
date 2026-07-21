//! Filesystem installation target discovery and atomic copy orchestration for BioMCP skills.

use std::collections::HashSet;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::BioMcpError;

use super::status::{
    MANIFEST_NAME, ManagedPayload, classify_target, current_payload, parse_valid_manifest,
    validate_managed_path,
};

static STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn expand_tilde(path: &str) -> Result<PathBuf, BioMcpError> {
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

pub(super) fn resolve_install_dir(input: PathBuf) -> PathBuf {
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

#[derive(Debug)]
pub(super) struct ResolvedSkillTarget {
    pub path: PathBuf,
    pub safe_dir: String,
    reason: &'static str,
    also_found: Vec<PathBuf>,
}

fn safe_dir_spelling(input: Option<&str>, target: &Path, home: &Path, cwd: &Path) -> String {
    let display = if let Some(input) = input {
        if !Path::new(input).is_absolute() {
            input.to_string()
        } else if let Ok(relative) = target.strip_prefix(home) {
            format!("~/{}", relative.display())
        } else {
            "<skill-dir>".to_string()
        }
    } else if let Ok(relative) = target.strip_prefix(home) {
        format!("~/{}", relative.display())
    } else if let Ok(relative) = target.strip_prefix(cwd) {
        format!("./{}", relative.display())
    } else {
        "<skill-dir>".to_string()
    };
    crate::render::markdown::shell_quote_arg(&display)
}

pub(super) fn resolve_skill_target(dir: Option<&str>) -> Result<ResolvedSkillTarget, BioMcpError> {
    let home = expand_tilde("~")?;
    let cwd = std::env::current_dir().map_err(BioMcpError::Io)?;
    if let Some(dir) = dir {
        let path = resolve_install_dir(expand_tilde(dir)?);
        return Ok(ResolvedSkillTarget {
            safe_dir: safe_dir_spelling(Some(dir), &path, &home, &cwd),
            path,
            reason: "explicit install target",
            also_found: Vec::new(),
        });
    }

    let candidates = candidate_entries(&home, &cwd);
    let (path, reason, also_found) =
        if let Some((path, also_found)) = find_existing_install(&candidates) {
            (path, "existing BioMCP skill found", also_found)
        } else {
            let (path, reason) = find_best_target(&candidates)?;
            (path, reason, Vec::new())
        };
    Ok(ResolvedSkillTarget {
        safe_dir: safe_dir_spelling(None, &path, &home, &cwd),
        path,
        reason,
        also_found,
    })
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

fn staging_dir(parent: &Path) -> Result<PathBuf, BioMcpError> {
    for _ in 0..100 {
        let sequence = STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = parent.join(format!(".biomcp-install-{}-{sequence}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(BioMcpError::Io(error)),
        }
    }
    Err(BioMcpError::Io(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create unique skill staging directory",
    )))
}

fn ensure_real_directory(path: &Path) -> Result<(), BioMcpError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BioMcpError::InvalidArgument(
            "Skill target must be a real directory".into(),
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, destination: &Path) -> Result<(), BioMcpError> {
    use std::os::unix::fs::symlink;
    symlink(fs::read_link(source)?, destination).map_err(BioMcpError::Io)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), BioMcpError> {
    ensure_real_directory(source)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            #[cfg(unix)]
            copy_symlink(&source_path, &destination_path)?;
            #[cfg(not(unix))]
            return Err(BioMcpError::InvalidArgument(
                "Symbolic-link preservation is unsupported on this platform".into(),
            ));
        } else if kind.is_dir() {
            fs::create_dir(&destination_path)?;
            copy_tree(&source_path, &destination_path)?;
        } else if kind.is_file() {
            fs::copy(&source_path, &destination_path)?;
        } else {
            return Err(BioMcpError::InvalidArgument(
                "Unsupported filesystem object in skill directory".into(),
            ));
        }
    }
    Ok(())
}

fn remove_managed_path(root: &Path, relative: &str) -> Result<(), BioMcpError> {
    let relative = validate_managed_path(relative)?;
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        cursor.push(component.as_os_str());
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(BioMcpError::InvalidArgument(
                    "Managed skill path contains a symbolic link".into(),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(BioMcpError::Io(error)),
        }
    }
    let path = root.join(&relative);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() || metadata.is_file() => {
            fs::remove_file(&path)?;
        }
        Ok(_) => {
            return Err(BioMcpError::InvalidArgument(
                "Managed skill path is not a regular file".into(),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(BioMcpError::Io(error)),
    }

    let mut parent = path.parent();
    while let Some(directory) = parent {
        if directory == root {
            break;
        }
        match fs::remove_dir(directory) {
            Ok(()) => parent = directory.parent(),
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                parent = directory.parent();
            }
            Err(error) => return Err(BioMcpError::Io(error)),
        }
    }
    Ok(())
}

fn write_managed_file(root: &Path, relative: &str, bytes: &[u8]) -> Result<(), BioMcpError> {
    let relative = validate_managed_path(relative)?;
    let destination = root.join(&relative);
    let mut directory = root.to_path_buf();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            directory.push(component.as_os_str());
            match fs::symlink_metadata(&directory) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err(BioMcpError::InvalidArgument(
                        "Managed skill directory is not a real directory".into(),
                    ));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir(&directory)?;
                }
                Err(error) => return Err(BioMcpError::Io(error)),
            }
        }
    }
    match fs::symlink_metadata(&destination) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            return Err(BioMcpError::InvalidArgument(
                "Managed skill destination is not a regular file".into(),
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(BioMcpError::Io(error)),
    }
    fs::write(destination, bytes).map_err(BioMcpError::Io)
}

fn remove_sidecar(root: &Path) -> Result<(), BioMcpError> {
    let path = root.join(MANIFEST_NAME);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {
            fs::remove_file(path).map_err(BioMcpError::Io)
        }
        Ok(_) => Err(BioMcpError::InvalidArgument(
            "Skill management sidecar is not a regular file".into(),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(BioMcpError::Io(error)),
    }
}

fn write_payload(root: &Path, payload: &ManagedPayload) -> Result<(), BioMcpError> {
    for (path, bytes) in &payload.files {
        write_managed_file(root, path, bytes)?;
    }
    let manifest_bytes = serde_json::to_vec_pretty(&payload.manifest)?;
    fs::write(root.join(MANIFEST_NAME), manifest_bytes).map_err(BioMcpError::Io)
}

fn prepare_candidate(
    target: &Path,
    force_existing: bool,
    payload: &ManagedPayload,
) -> Result<PathBuf, BioMcpError> {
    let parent = target.parent().ok_or_else(|| {
        BioMcpError::InvalidArgument("Install path has no parent directory".into())
    })?;
    fs::create_dir_all(parent)?;
    let candidate = staging_dir(parent)?;
    let prepared = (|| {
        if force_existing {
            ensure_real_directory(target)?;
            copy_tree(target, &candidate)?;
            if let Some(manifest) = parse_valid_manifest(target)? {
                for path in manifest.managed_files.keys() {
                    remove_managed_path(&candidate, path)?;
                }
            }
            remove_sidecar(&candidate)?;
        }
        write_payload(&candidate, payload)
    })();
    if let Err(error) = prepared {
        let _ = fs::remove_dir_all(&candidate);
        return Err(error);
    }
    Ok(candidate)
}

fn target_exists(path: &Path) -> Result<bool, BioMcpError> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(BioMcpError::Io(error)),
    }
}

fn install_to_dir_with_display(
    target: &Path,
    safe_dir: &str,
    force: bool,
) -> Result<String, BioMcpError> {
    let payload = current_payload()?;
    let exists = target_exists(target)?;
    if exists && !force {
        let status = classify_target(target, safe_dir, &payload)?;
        return Ok(format!(
            "Existing BioMCP skill was not changed.\n{}",
            crate::render::json::to_pretty(&status)?
        ));
    }

    let candidate = prepare_candidate(target, exists, &payload)?;
    if !exists {
        if let Err(error) = super::atomic_swap::rename_absent(&candidate, target) {
            let _ = fs::remove_dir_all(&candidate);
            return Err(error);
        }
    } else if let Err(error) = super::atomic_swap::exchange_directories(&candidate, target) {
        let _ = fs::remove_dir_all(&candidate);
        return Err(error);
    } else if fs::remove_dir_all(&candidate).is_err() {
        let _ =
            write_stderr_line("Warning: skill repair committed, but old staging cleanup failed");
    }

    Ok(format!("Installed BioMCP skills to {}", target.display()))
}

#[cfg(test)]
pub(super) fn install_to_dir(dir: &Path, force: bool) -> Result<String, BioMcpError> {
    install_to_dir_with_display(dir, "<skill-dir>", force)
}

/// Installs embedded skills into a supported agent directory.
///
/// # Errors
///
/// Returns an error when the destination path is invalid, not writable, or no
/// supported installation directory can be determined.
pub fn install_skills(dir: Option<&str>, force: bool) -> Result<String, BioMcpError> {
    let resolved = resolve_skill_target(dir)?;
    if !resolved.also_found.is_empty() {
        let extra = resolved
            .also_found
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write_stderr_line(&format!("Note: BioMCP skill also found at: {extra}"))?;
    }
    if dir.is_none() {
        write_stderr_line(&format!(
            "Auto-detected: {} ({})",
            resolved.path.display(),
            resolved.reason
        ))?;
        if io::stdin().is_terminal() && !prompt_confirm(&resolved.path)? {
            return Ok("No installation selected".into());
        }
    }
    install_to_dir_with_display(&resolved.path, &resolved.safe_dir, force)
}
