//! Embedded skill asset access and prompt normalization.

use crate::error::BioMcpError;

pub(super) fn embedded_text(path: &str) -> Result<String, BioMcpError> {
    match crate::skill_assets::text(path) {
        Ok(text) => Ok(text),
        Err(BioMcpError::NotFound { .. }) => Err(BioMcpError::NotFound {
            entity: "skill".into(),
            id: path.to_string(),
            suggestion: "Try: biomcp skill".into(),
        }),
        Err(_) => Err(BioMcpError::InvalidArgument(
            "Embedded skill file is not valid UTF-8".into(),
        )),
    }
}

pub(super) fn canonical_prompt_body() -> Result<String, BioMcpError> {
    let mut body = embedded_text("SKILL.md")?;
    while body.ends_with('\n') {
        body.pop();
    }
    Ok(body)
}

pub(super) fn canonical_prompt_file_bytes() -> Result<Vec<u8>, BioMcpError> {
    let mut body = canonical_prompt_body()?;
    body.push('\n');
    Ok(body.into_bytes())
}

pub(super) fn parse_title_and_description(markdown: &str) -> (String, Option<String>) {
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
