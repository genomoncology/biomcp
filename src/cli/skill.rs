use std::borrow::Cow;
use std::collections::HashSet;
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
    /// Legacy compatibility command for the removed embedded skill catalog
    #[command(hide = true)]
    List,
    /// Show a specific use-case by number or name
    #[command(external_subcommand)]
    Show(Vec<String>),
    /// Install BioMCP skill guidance to an agent directory
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
            suggestion: "Try: biomcp skill".into(),
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
            suggestion: "Try: biomcp skill".into(),
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

#[derive(Debug, Clone)]
struct CandidateEntry {
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

fn candidate_entries(home: &Path, cwd: &Path) -> Vec<CandidateEntry> {
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

fn find_existing_install(candidates: &[CandidateEntry]) -> Option<(PathBuf, Vec<PathBuf>)> {
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

fn find_best_target(candidates: &[CandidateEntry]) -> Result<(PathBuf, &'static str), BioMcpError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestPaths {
        root: PathBuf,
        home: PathBuf,
        cwd: PathBuf,
    }

    impl TestPaths {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "biomcp-skill-test-{name}-{}-{unique}",
                std::process::id()
            ));
            let home = root.join("home");
            let cwd = root.join("cwd");

            fs::create_dir_all(&home).expect("create test home dir");
            fs::create_dir_all(&cwd).expect("create test cwd dir");

            Self { root, home, cwd }
        }

        fn create_file(&self, path: &Path) {
            let parent = path.parent().expect("path has parent");
            fs::create_dir_all(parent).expect("create parent dirs");
            fs::write(path, "# test").expect("write test file");
        }
    }

    impl Drop for TestPaths {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn read_json_fixture(path: &Path) -> Value {
        let contents = fs::read_to_string(path).expect("read JSON fixture");
        serde_json::from_str(&contents).expect("parse JSON fixture")
    }

    #[test]
    fn embedded_skill_overview_includes_t017_t018_polish() -> Result<(), BioMcpError> {
        let overview = show_overview()?;

        assert!(overview.contains("biomcp search gene BRAF --limit 5"));
        assert!(overview.contains("biomcp search variant BRAF V600E"));
        assert!(overview.contains("biomcp search trial melanoma --status recruiting --limit 5"));
        assert!(overview.contains("biomcp search all BRAF"));
        assert!(overview.contains("`expression`, `hpa`, `druggability`, `clingen`"));

        Ok(())
    }

    #[test]
    fn validate_skills_target_uses_uv_dev_environment() {
        let makefile = fs::read_to_string(repo_root().join("Makefile")).expect("read Makefile");
        let pyproject =
            fs::read_to_string(repo_root().join("pyproject.toml")).expect("read pyproject");

        assert!(makefile.contains("validate-skills:"));
        assert!(makefile.contains("uv run --extra dev sh -c"));
        assert!(makefile.contains("./scripts/validate-skills.sh"));
        assert!(makefile.contains("PATH=\"$(CURDIR)/target/release:$$PATH\""));
        assert!(pyproject.contains("\"jsonschema>="));
    }

    #[test]
    fn refreshed_search_examples_are_non_empty() {
        for file_name in ["search-article.json", "search-drug.json"] {
            let path = repo_root().join("skills/examples").join(file_name);
            let payload = read_json_fixture(&path);
            let count = payload
                .get("count")
                .and_then(Value::as_u64)
                .expect("count should be present");
            let returned = payload
                .pointer("/pagination/returned")
                .and_then(Value::as_u64)
                .expect("pagination.returned should be present");
            let results = payload
                .get("results")
                .and_then(Value::as_array)
                .expect("results should be an array");

            assert!(
                count > 0,
                "{file_name} should keep at least one example row"
            );
            assert!(returned > 0, "{file_name} should report returned rows");
            assert!(
                !results.is_empty(),
                "{file_name} should keep non-empty results"
            );
        }
    }

    #[test]
    fn embedded_use_case_catalog_is_empty() -> Result<(), BioMcpError> {
        assert!(list_use_case_refs()?.is_empty());
        assert_eq!(list_use_cases()?, "No skills found");
        assert!(show_use_case("01").is_err());
        Ok(())
    }

    #[test]
    fn missing_skill_suggests_skill_overview() {
        let err = show_use_case("01").expect_err("missing skill should error");
        let msg = err.to_string();

        assert!(msg.contains("skill '01' not found"));
        assert!(msg.contains("Try: biomcp skill"));
        assert!(!msg.contains("Try: biomcp skill list"));
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
}
