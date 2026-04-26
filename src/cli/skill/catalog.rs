//! Read-only BioMCP skill catalog rendering and use-case lookup.

use crate::error::BioMcpError;

use super::assets::{canonical_prompt_body, embedded_text, parse_title_and_description};

#[derive(Debug, Clone)]
pub(super) struct UseCaseMeta {
    pub(super) number: String,
    pub(super) slug: String,
    pub(super) title: String,
    pub(super) description: Option<String>,
    pub(super) embedded_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct UseCaseRef {
    pub slug: String,
    pub title: String,
}

/// Renders the canonical agent-facing BioMCP prompt.
///
/// # Errors
///
/// Returns an error if the embedded prompt cannot be loaded.
pub fn render_system_prompt() -> Result<String, BioMcpError> {
    canonical_prompt_body()
}

pub(super) fn use_case_index() -> Result<Vec<UseCaseMeta>, BioMcpError> {
    let mut out: Vec<UseCaseMeta> = Vec::new();

    for file in crate::skill_assets::iter() {
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
    canonical_prompt_body()
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
    out.push_str("# BioMCP Worked Examples\n\n");
    out.push_str(
        "Worked examples are short, executable investigation patterns. Run `biomcp skill <name>` to open one.\n\n",
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

    // Accept "01", "1", "01-treatment-lookup", or "treatment-lookup"
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
