use std::fmt::Write as _;
use std::path::Path;

use serde::Serialize;

use crate::error::BioMcpError;

const MAX_RANGE_LINES: usize = 500;
const MAX_RANGE_BYTES: usize = 65_536;
pub(super) type FulltextSummary = (usize, usize, usize);
pub(super) fn extract_pdf(sections: &[String]) -> (Vec<String>, bool) {
    let mut allow_pdf = false;
    let cleaned = sections
        .iter()
        .filter_map(|raw| {
            let trimmed = raw.trim();
            if trimmed.eq_ignore_ascii_case("--pdf") {
                allow_pdf = true;
                return None;
            }
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect();
    (cleaned, allow_pdf)
}

pub(super) fn extract_controls(
    sections: &[String],
    mut outline: bool,
    mut lines: Option<String>,
) -> Result<(Vec<String>, bool, Option<String>), BioMcpError> {
    let mut cleaned = Vec::new();
    let mut index = 0;
    while index < sections.len() {
        match sections[index].trim() {
            "--outline" => outline = true,
            "--lines" => {
                index += 1;
                lines = sections.get(index).cloned().or_else(|| Some(String::new()));
            }
            value if value.starts_with("--lines=") => lines = Some(value[8..].to_string()),
            value => cleaned.push(value.to_string()),
        }
        index += 1;
    }
    if outline && lines.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "--outline and --lines are mutually exclusive".into(),
        ));
    }
    Ok((cleaned, outline, lines))
}

pub(super) fn requested_response(
    article: &crate::entities::article::Article,
    id: &str,
    sections: &[String],
    outline: bool,
    lines: Option<&str>,
    json: bool,
) -> Result<Option<String>, BioMcpError> {
    if !outline && lines.is_none() {
        return Ok(None);
    }
    validate_controls(sections, outline, lines)?;
    let path = article
        .full_text_path
        .as_ref()
        .ok_or_else(|| BioMcpError::NotFound {
            entity: "article fulltext".into(),
            id: id.into(),
            suggestion: "Full text was not available from the configured providers.".into(),
        })?;
    render(path, outline, lines, json).map(Some)
}

pub(super) fn validate_controls(
    sections: &[String],
    outline: bool,
    lines: Option<&str>,
) -> Result<(), BioMcpError> {
    if !outline && lines.is_none() {
        return Ok(());
    }
    if sections.len() != 1 || !sections[0].eq_ignore_ascii_case("fulltext") {
        return Err(BioMcpError::InvalidArgument(
            "--outline and --lines require fulltext as the sole article section".into(),
        ));
    }
    if let Some(range) = lines {
        parse_range(range)?;
    }
    Ok(())
}

pub(super) fn article_summary(
    article: &crate::entities::article::Article,
) -> Result<Option<FulltextSummary>, BioMcpError> {
    article.full_text_path.as_deref().map(summary).transpose()
}

pub(super) fn decorate_human(
    article: &crate::entities::article::Article,
    mut human: String,
    summary: Option<FulltextSummary>,
) -> String {
    if let (Some(path), Some((bytes, lines, sections))) = (article.full_text_path.as_ref(), summary)
    {
        human = human.replace(
            &format!("Saved to: {}", path.display()),
            &format!(
                "Saved to: {}\nCached full text: {bytes} bytes, {lines} lines, {sections} sections.",
                path.display()
            ),
        );
    }
    human
}

pub(super) fn decorate_json(
    mut json: String,
    summary: Option<FulltextSummary>,
) -> Result<String, BioMcpError> {
    let Some((bytes, lines, sections)) = summary else {
        return Ok(json);
    };
    let mut value: serde_json::Value = serde_json::from_str(&json)?;
    if let Some(object) = value.as_object_mut() {
        object.insert("full_text_summary".into(), serde_json::json!({"byte_size": bytes, "total_lines": lines, "section_count": sections}));
    }
    json = crate::render::json::to_pretty(&value)?;
    Ok(json)
}

#[derive(Serialize)]
struct Heading {
    ordinal: usize,
    level: usize,
    title: String,
    start_line: usize,
    end_line: usize,
    title_truncated: bool,
}

#[derive(Serialize)]
struct Outline {
    headings: Vec<Heading>,
    returned: usize,
    total: usize,
    has_more: bool,
}

#[derive(Serialize)]
struct Lines {
    text: String,
    total_lines: usize,
    start_line: usize,
    end_line: usize,
    returned_bytes: usize,
    truncated: bool,
    next_line: Option<usize>,
}

pub(super) fn render(
    path: &Path,
    outline: bool,
    range: Option<&str>,
    json: bool,
) -> Result<String, BioMcpError> {
    let text = std::fs::read_to_string(path).map_err(BioMcpError::Io)?;
    if outline {
        let result = build_outline(&text);
        return if json {
            crate::render::json::to_pretty(&result)
        } else {
            Ok(outline_markdown(&result))
        };
    }
    let result = build_lines(&text, range.unwrap_or_default())?;
    if json {
        crate::render::json::to_pretty(&result)
    } else {
        Ok(lines_markdown(&result))
    }
}

pub(super) fn summary(path: &Path) -> Result<(usize, usize, usize), BioMcpError> {
    let text = std::fs::read_to_string(path).map_err(BioMcpError::Io)?;
    let lines = text.lines().count();
    let sections = text
        .lines()
        .filter(|line| {
            let hashes = line.chars().take_while(|ch| *ch == '#').count();
            (1..=6).contains(&hashes)
                && line
                    .as_bytes()
                    .get(hashes)
                    .is_some_and(u8::is_ascii_whitespace)
        })
        .count();
    Ok((text.len(), lines, sections))
}

fn parse_range(value: &str) -> Result<(usize, usize), BioMcpError> {
    let (start, end) = value.split_once(':').ok_or_else(|| {
        BioMcpError::InvalidArgument(
            "--lines must use START:END with inclusive one-based line numbers".into(),
        )
    })?;
    let start = start.parse::<usize>().map_err(|_| {
        BioMcpError::InvalidArgument("--lines start must be a positive integer".into())
    })?;
    let end = end.parse::<usize>().map_err(|_| {
        BioMcpError::InvalidArgument("--lines end must be a positive integer".into())
    })?;
    if start == 0 || end < start || end - start + 1 > MAX_RANGE_LINES {
        return Err(BioMcpError::InvalidArgument(
            "--lines must be ordered, one-based, and span at most 500 lines".into(),
        ));
    }
    Ok((start, end))
}

fn build_lines(text: &str, range: &str) -> Result<Lines, BioMcpError> {
    let (start, requested_end) = parse_range(range)?;
    let rows = text.split_inclusive('\n').collect::<Vec<_>>();
    if start > rows.len() {
        return Err(BioMcpError::InvalidArgument(format!(
            "--lines start {start} exceeds the document's {} lines",
            rows.len()
        )));
    }
    let end = requested_end.min(rows.len());
    let selected = &rows[start - 1..end];
    if selected.iter().any(|line| line.len() > MAX_RANGE_BYTES) {
        return Err(BioMcpError::InputTooLarge {
            limit_bytes: MAX_RANGE_BYTES,
        });
    }
    let mut output = String::new();
    let mut returned = 0;
    for line in selected {
        if output.len() + line.len() > MAX_RANGE_BYTES {
            break;
        }
        output.push_str(line);
        returned += 1;
    }
    let truncated = returned < selected.len();
    Ok(Lines {
        returned_bytes: output.len(),
        text: output,
        total_lines: rows.len(),
        start_line: start,
        end_line: start + returned.saturating_sub(1),
        truncated,
        next_line: truncated.then_some(start + returned),
    })
}

fn build_outline(text: &str) -> Outline {
    let rows = text.lines().collect::<Vec<_>>();
    let mut all = Vec::new();
    for (index, line) in rows.iter().enumerate() {
        let hashes = line.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&hashes)
            || !line
                .as_bytes()
                .get(hashes)
                .is_some_and(u8::is_ascii_whitespace)
        {
            continue;
        }
        let (title, title_truncated) = truncate_bytes(line[hashes..].trim(), 512);
        all.push(Heading {
            ordinal: all.len() + 1,
            level: hashes,
            title,
            start_line: index + 1,
            end_line: rows.len(),
            title_truncated,
        });
    }
    for index in 0..all.len().saturating_sub(1) {
        all[index].end_line = all[index + 1].start_line - 1;
    }
    let total = all.len();
    all.truncate(200);
    Outline {
        returned: all.len(),
        total,
        has_more: all.len() < total,
        headings: all,
    }
}

fn truncate_bytes(value: &str, max: usize) -> (String, bool) {
    if value.len() <= max {
        return (value.into(), false);
    }
    let mut end = max;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].into(), true)
}

fn outline_markdown(result: &Outline) -> String {
    let mut out = format!(
        "# Full-text outline\n\nReturned {} of {} headings; has more: {}.\n",
        result.returned, result.total, result.has_more
    );
    for heading in &result.headings {
        let _ = writeln!(
            out,
            "- {}. H{} {} (lines {}–{})",
            heading.ordinal, heading.level, heading.title, heading.start_line, heading.end_line
        );
    }
    out
}

fn lines_markdown(result: &Lines) -> String {
    format!(
        "# Full-text lines {}–{}\n\n{}\n\nReturned {} bytes; total lines: {}; truncated: {}; next line: {}.\n",
        result.start_line,
        result.end_line,
        result.text,
        result.returned_bytes,
        result.total_lines,
        result.truncated,
        result
            .next_line
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".into())
    )
}

#[cfg(test)]
#[path = "../../../tests/unit/cli/article_fulltext.rs"]
mod tests;
