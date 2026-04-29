//! Generic formatting, quoting, and debug-plan helpers shared across markdown renderers.

use super::*;

pub(super) fn quote_arg(value: &str) -> String {
    shell_quote_arg(value)
}

pub(super) fn force_quote_arg(value: &str) -> String {
    let v = value.trim();
    if v.is_empty() {
        return String::new();
    }
    let escaped = v.chars().fold(String::new(), |mut out, ch| {
        if matches!(ch, '\\' | '"' | '$' | '`') {
            out.push('\\');
        }
        out.push(ch);
        out
    });
    format!("\"{escaped}\"")
}

pub(super) fn alias_fallback_suggestion(
    decision: &crate::entities::discover::AliasFallbackDecision,
) -> String {
    match decision {
        crate::entities::discover::AliasFallbackDecision::Canonical(alias) => {
            let command = alias.next_commands.first().cloned().unwrap_or_else(|| {
                format!(
                    "biomcp get {} {}",
                    alias.requested_entity.cli_name(),
                    quote_arg(&alias.canonical)
                )
            });
            format!("Did you mean: `{command}`")
        }
        crate::entities::discover::AliasFallbackDecision::Ambiguous(alias) => {
            let mut out = format!(
                "BioMCP could not map '{}' to a single {}.\n\nTry:",
                alias.query,
                alias.requested_entity.cli_name()
            );
            for (idx, command) in alias.next_commands.iter().enumerate() {
                out.push_str(&format!("\n{}. {command}", idx + 1));
            }
            if !alias.candidates.is_empty() {
                out.push_str("\n\nPossible matches:");
                for candidate in &alias.candidates {
                    match candidate.primary_id.as_deref() {
                        Some(primary_id) => out.push_str(&format!(
                            "\n- {} ({}, {})",
                            candidate.label,
                            candidate.primary_type.label(),
                            primary_id
                        )),
                        None => out.push_str(&format!(
                            "\n- {} ({})",
                            candidate.label,
                            candidate.primary_type.label()
                        )),
                    }
                }
            }
            out
        }
        crate::entities::discover::AliasFallbackDecision::None => String::new(),
    }
}

pub(super) fn variant_guidance_suggestion(
    guidance: &crate::entities::variant::VariantGuidance,
) -> String {
    match &guidance.kind {
        crate::entities::variant::VariantGuidanceKind::GeneResidueAlias { .. } => {
            let mut out = format!(
                "BioMCP could not map '{}' to an exact variant.\n\nTry:",
                guidance.query
            );
            for (idx, command) in guidance.next_commands.iter().enumerate() {
                out.push_str(&format!("\n{}. {command}", idx + 1));
            }
            out
        }
        crate::entities::variant::VariantGuidanceKind::ProteinChangeOnly { .. } => {
            let mut out = format!(
                "BioMCP could not map '{}' to an exact variant without gene context.\n\nTry:",
                guidance.query
            );
            for (idx, command) in guidance.next_commands.iter().enumerate() {
                out.push_str(&format!("\n{}. {command}", idx + 1));
            }
            out
        }
    }
}

pub(super) fn shell_quote_arg(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let needs_quotes = trimmed.chars().any(|ch| {
        ch.is_whitespace()
            || matches!(
                ch,
                '"' | '\''
                    | '\\'
                    | '$'
                    | '`'
                    | '|'
                    | '&'
                    | ';'
                    | '<'
                    | '>'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '*'
                    | '?'
                    | '!'
                    | '#'
            )
    });
    if needs_quotes {
        let escaped = trimmed.chars().fold(String::new(), |mut out, ch| {
            if matches!(ch, '\\' | '"' | '$' | '`') {
                out.push('\\');
            }
            out.push(ch);
            out
        });
        return format!("\"{escaped}\"");
    }

    trimmed.to_string()
}

pub(super) fn discover_try_line(query: &str, description: &str) -> String {
    let query = shell_quote_arg(query);
    if query.is_empty() {
        return String::new();
    }
    format!("Try: biomcp discover {query}   - {description}")
}

pub(super) fn markdown_cell(value: &str) -> String {
    let value = value.replace(['\n', '\r'], " ").replace('|', "\\|");
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.is_empty() {
        "-".to_string()
    } else {
        value
    }
}

pub(super) fn dedupe_markdown_commands(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = trimmed.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(trimmed.to_string());
        }
    }
    out
}

pub(super) fn render_debug_plan_block(debug_plan: &DebugPlan) -> Result<String, BioMcpError> {
    Ok(format!(
        "## Debug plan\n\n```json\n{}\n```\n\n",
        serde_json::to_string_pretty(debug_plan)?
    ))
}

#[cfg(test)]
mod tests;
