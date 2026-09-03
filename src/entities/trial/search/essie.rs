//! ESSIE query helpers for trial search filters.

use regex::Regex;
use std::sync::OnceLock;

use crate::error::BioMcpError;

use super::super::TrialSearchFilters;

pub(super) fn essie_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '\"'
                | '+'
                | '-'
                | '!'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '^'
                | '~'
                | '*'
                | '?'
                | ':'
                | '/'
                | '|'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

pub(super) fn quote_essie_literal(value: &str) -> String {
    format!("\"{}\"", essie_escape(value))
}

fn boolean_operator_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(OR|AND|NOT)\b").expect("valid boolean operator regex"))
}

fn split_boolean_expression(value: &str) -> Option<(Vec<String>, Vec<String>)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut terms = Vec::new();
    let mut operators: Vec<String> = Vec::new();
    let mut last = 0;
    for caps in boolean_operator_regex().captures_iter(trimmed) {
        let Some(matched) = caps.get(0) else {
            continue;
        };
        let op = caps
            .get(1)
            .map(|m| m.as_str().to_ascii_uppercase())
            .unwrap_or_else(|| "OR".to_string());
        let term = trimmed[last..matched.start()].trim();
        if term.is_empty() {
            // Consecutive operators (e.g., "AND NOT") — merge into previous operator.
            if let Some(prev) = operators.last_mut() {
                prev.push(' ');
                prev.push_str(&op);
            } else {
                // Leading operator (e.g., "NOT dMMR") — treat as prefix.
                operators.push(op);
            }
        } else {
            terms.push(term.to_string());
            operators.push(op);
        }
        last = matched.end();
    }

    let tail = trimmed[last..].trim();
    if tail.is_empty() {
        return None;
    }
    terms.push(tail.to_string());
    Some((terms, operators))
}

pub(super) fn has_boolean_operators(value: &str) -> bool {
    split_boolean_expression(value).is_some_and(|(_, operators)| !operators.is_empty())
}

pub(super) fn essie_escape_boolean_expression(value: &str) -> String {
    let trimmed = value.trim();
    let Some((terms, operators)) = split_boolean_expression(trimmed) else {
        return quote_essie_literal(trimmed);
    };
    if operators.is_empty() {
        return quote_essie_literal(&terms[0]);
    }

    let mut rendered = String::new();
    let has_leading_unary_operator = operators.len() == terms.len();
    for (idx, term) in terms.iter().enumerate() {
        if idx == 0 && has_leading_unary_operator {
            rendered.push_str(&operators[0]);
            rendered.push(' ');
        } else if idx > 0 {
            rendered.push(' ');
            let operator_idx = if has_leading_unary_operator {
                idx
            } else {
                idx - 1
            };
            rendered.push_str(&operators[operator_idx]);
            rendered.push(' ');
        }
        rendered.push_str(&quote_essie_literal(term));
    }
    rendered
}

pub(super) fn has_essie_filters(filters: &TrialSearchFilters) -> bool {
    filters
        .prior_therapies
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .progression_on
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .line_of_therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
}

fn line_of_therapy_patterns(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_uppercase().as_str() {
        "1L" => Some(
            "\"first line\" OR \"first-line\" OR \"1st line\" OR \"frontline\" OR \"treatment naive\" OR \"previously untreated\"",
        ),
        "2L" => Some(
            "\"second line\" OR \"second-line\" OR \"2nd line\" OR \"one prior line\" OR \"1 prior line\"",
        ),
        "3L+" => Some(
            "\"third line\" OR \"third-line\" OR \"3rd line\" OR \"≥2 prior\" OR \"at least 2 prior\" OR \"heavily pretreated\"",
        ),
        _ => None,
    }
}

pub(super) fn build_essie_fragments(
    filters: &TrialSearchFilters,
) -> Result<Vec<String>, BioMcpError> {
    let mut fragments = Vec::new();

    if let Some(therapy) = filters
        .prior_therapies
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let therapy = essie_escape(therapy);
        fragments.push(format!(
            "AREA[EligibilityCriteria](\"{therapy}\" AND (prior OR previous OR received))"
        ));
    }

    if let Some(drug) = filters
        .progression_on
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let drug = essie_escape(drug);
        fragments.push(format!(
            "AREA[EligibilityCriteria](\"{drug}\" AND (progression OR resistant OR refractory))"
        ));
    }

    if let Some(line) = filters
        .line_of_therapy
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let patterns = line_of_therapy_patterns(line).ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "Invalid --line-of-therapy value. Expected one of: 1L, 2L, 3L+".into(),
            )
        })?;
        fragments.push(format!("AREA[EligibilityCriteria]({patterns})"));
    }

    Ok(fragments)
}

#[cfg(test)]
mod tests;
