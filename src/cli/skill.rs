use std::borrow::Cow;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use clap::Subcommand;
use rust_embed::RustEmbed;

use crate::error::BioMcpError;

#[derive(RustEmbed)]
#[folder = "skills/"]
struct EmbeddedSkills;

#[derive(Subcommand, Debug)]
pub enum SkillCommand {
    /// List available BioMCP skill use-cases
    List,
    /// Show a specific use-case by number or name
    #[command(external_subcommand)]
    Show(Vec<String>),
    /// Install BioMCP skills to an agent directory
    Install {
        /// Agent root or skills directory (e.g. ~/.claude, ~/.claude/skills, ~/.claude/skills/biomcp)
        dir: Option<String>,
        /// Replace existing installation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone)]
struct UseCaseMeta {
    number: String,
    slug: String,
    title: String,
    description: Option<String>,
    embedded_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UseCaseRef {
    pub slug: String,
    pub title: String,
}

fn embedded_text(path: &str) -> Result<String, BioMcpError> {
    let Some(asset) = EmbeddedSkills::get(path) else {
        return Err(BioMcpError::NotFound {
            entity: "skill".into(),
            id: path.to_string(),
            suggestion: "Try: biomcp skill list".into(),
        });
    };

    let bytes: Cow<'static, [u8]> = asset.data;
    String::from_utf8(bytes.into_owned())
        .map_err(|_| BioMcpError::InvalidArgument("Embedded skill file is not valid UTF-8".into()))
}

fn parse_title_and_description(markdown: &str) -> (String, Option<String>) {
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;

    for line in markdown.lines() {
        let line = line.trim_end();
        if title.is_none() && line.starts_with("# ") {
            title = Some(line.trim_start_matches("# ").trim().to_string());
            continue;
        }
        if title.is_some() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // First non-empty line after the title.
            description = Some(trimmed.to_string());
            break;
        }
    }

    (title.unwrap_or_else(|| "Untitled".into()), description)
}

fn use_case_index() -> Result<Vec<UseCaseMeta>, BioMcpError> {
    let mut out: Vec<UseCaseMeta> = Vec::new();

    for file in EmbeddedSkills::iter() {
        let path = file.as_ref();
        if !path.starts_with("use-cases/") || !path.ends_with(".md") {
            continue;
        }

        let file_name = path
            .rsplit('/')
            .next()
            .unwrap_or(path)
            .trim_end_matches(".md");

        let (number, slug) = match file_name.split_once('-') {
            Some((n, rest)) if n.len() == 2 && n.chars().all(|c| c.is_ascii_digit()) => {
                (n.to_string(), rest.to_string())
            }
            _ => continue,
        };

        let content = embedded_text(path)?;
        let (title, description) = parse_title_and_description(&content);

        out.push(UseCaseMeta {
            number,
            slug,
            title,
            description,
            embedded_path: path.to_string(),
        });
    }

    out.sort_by_key(|m| m.number.parse::<u32>().unwrap_or(999));
    Ok(out)
}

/// Returns the embedded BioMCP skill overview document.
///
/// # Errors
///
/// Returns an error if the embedded overview document cannot be loaded.
pub fn show_overview() -> Result<String, BioMcpError> {
    embedded_text("SKILL.md")
}

/// Lists available embedded skill use-cases.
///
/// # Errors
///
/// Returns an error if embedded skill metadata cannot be loaded.
pub fn list_use_cases() -> Result<String, BioMcpError> {
    let cases = use_case_index()?;
    if cases.is_empty() {
        return Ok("No skills found".into());
    }

    let mut out = String::new();
    out.push_str("# BioMCP Skill Use-Cases\n\n");
    out.push_str(
        "Skills are step-by-step investigation workflows. Run `biomcp skill <name>` to view.\n\n",
    );
    for c in cases {
        out.push_str(&format!("{} {} - {}\n", c.number, c.slug, c.title));
        if let Some(desc) = c.description {
            out.push_str(&format!("  {desc}\n"));
        }
        out.push('\n');
    }
    Ok(out)
}

pub(crate) fn list_use_case_refs() -> Result<Vec<UseCaseRef>, BioMcpError> {
    Ok(use_case_index()?
        .into_iter()
        .map(|c| UseCaseRef {
            slug: c.slug,
            title: c.title,
        })
        .collect())
}

fn normalize_use_case_key(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Accept "01", "1", "01-variant-to-treatment", or "variant-to-treatment"
    if trimmed.chars().all(|c| c.is_ascii_digit())
        && let Ok(n) = trimmed.parse::<u32>()
    {
        return format!("{n:02}");
    }

    let lowered = trimmed.to_ascii_lowercase();
    if lowered.len() >= 3
        && lowered.as_bytes()[0].is_ascii_digit()
        && lowered.as_bytes()[1].is_ascii_digit()
        && lowered.as_bytes()[2] == b'-'
    {
        return lowered[3..].to_string();
    }

    lowered
}

/// Shows one skill use-case by number or slug.
///
/// # Errors
///
/// Returns an error if the requested skill does not exist or cannot be loaded.
pub fn show_use_case(name: &str) -> Result<String, BioMcpError> {
    let key = normalize_use_case_key(name);
    if key.is_empty() {
        return show_overview();
    }

    let cases = use_case_index()?;
    let found = cases.into_iter().find(|c| c.number == key || c.slug == key);
    let Some(found) = found else {
        return Err(BioMcpError::NotFound {
            entity: "skill".into(),
            id: name.to_string(),
            suggestion: "Try: biomcp skill list".into(),
        });
    };

    embedded_text(&found.embedded_path)
}

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

fn agent_candidates() -> Result<Vec<PathBuf>, BioMcpError> {
    Ok(vec![
        expand_tilde("~/.claude")?,
        expand_tilde("~/.codex")?,
        expand_tilde("~/.config/opencode")?,
        expand_tilde("~/.gemini")?,
        expand_tilde("~/.pi")?,
    ])
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

fn install_to_dir(dir: &Path, force: bool) -> Result<String, BioMcpError> {
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

    for file in EmbeddedSkills::iter() {
        let rel = file.as_ref();
        let Some(asset) = EmbeddedSkills::get(rel) else {
            continue;
        };

        let out_path = staging.join(rel);
        if let Some(p) = out_path.parent() {
            fs::create_dir_all(p)?;
        }
        fs::write(&out_path, asset.data)?;
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

    // Auto-detect installed agents.
    let candidates = agent_candidates()?;
    let mut found_any = false;
    for base in candidates {
        if !base.exists() {
            continue;
        }
        found_any = true;
        let target = resolve_install_dir(base);
        if !std::io::stdin().is_terminal() {
            return Err(BioMcpError::InvalidArgument(
                "Non-interactive session: specify a directory (e.g. biomcp skill install ~/.claude)"
                    .into(),
            ));
        }
        if prompt_confirm(&target)? {
            return install_to_dir(&target, force);
        }
    }

    if !found_any {
        return Err(BioMcpError::InvalidArgument(
            "No supported agent directories found. Specify a directory: biomcp skill install ~/.claude"
                .into(),
        ));
    }

    Ok("No installation selected".into())
}
