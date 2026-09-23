//! Drug label parsing and OpenFDA label-field extraction helpers.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use super::{DrugLabel, DrugLabelIndication};

const LABEL_MAX_CHARS: usize = 2000;

fn label_text(value: Option<&serde_json::Value>) -> Option<String> {
    let value = value?;
    let text = match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => String::new(),
    };
    let text = text.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn truncate_with_note(value: &str, max_chars: usize, label_set_id: Option<&str>) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let truncated = value.chars().take(max_chars).collect::<String>();
    let total = value.chars().count();
    let full_label = label_set_id
        .map(str::trim)
        .filter(|set_id| !set_id.is_empty())
        .and_then(|set_id| {
            let mut url =
                reqwest::Url::parse("https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm").ok()?;
            url.query_pairs_mut().append_pair("setid", set_id);
            Some(url.to_string())
        });
    match full_label {
        Some(url) => format!("{truncated}\n\n(truncated, {total} chars total; full label: {url})"),
        None => format!("{truncated}\n\n(truncated, {total} chars total)"),
    }
}

fn label_subsection_boundary_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\(\s*1\.\d+\s*\)").expect("valid label subsection regex"))
}

fn label_numbered_subsection_heading_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\b1\.\d+\s+[A-Z]").expect("valid subsection heading regex"))
}

fn label_numbered_subsection_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"^\s*1\.\d+\s+").expect("valid subsection heading prefix regex")
    })
}

fn label_heading_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)^\s*1\s+indications and usage\b[:\s-]*").expect("valid heading regex")
    })
}

fn normalize_label_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }

    haystack
        .to_ascii_lowercase()
        .find(&needle.to_ascii_lowercase())
}

fn strip_label_intro_prefix(segment: &str) -> &str {
    let lower = segment.to_ascii_lowercase();
    for needle in ["indicated:", "indicated for:", "indicated for"] {
        if let Some(idx) = lower.rfind(needle) {
            return segment[idx + needle.len()..].trim();
        }
    }
    segment.trim()
}

fn strip_leading_label_indication_prefixes(mut segment: &str) -> &str {
    loop {
        let lower = segment.to_ascii_lowercase();
        let mut next = None;
        for needle in [
            "the treatment of ",
            "treatment of ",
            "adult patients with ",
            "adult patient with ",
            "adults with ",
            "pediatric patients with ",
            "pediatric patient with ",
            "patients with ",
            "patient with ",
            "children with ",
            "women with ",
            "people with ",
        ] {
            if lower.starts_with(needle) {
                next = Some(segment[needle.len()..].trim());
                break;
            }
        }
        match next {
            Some(trimmed) => segment = trimmed,
            None => return segment.trim(),
        }
    }
}

fn label_continuation_prefix(lower: &str) -> bool {
    [
        "for ",
        "as ",
        "in combination",
        "continued as ",
        "following ",
        "after ",
        "where ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
}

fn label_patient_phrase_start<'a>(segment: &'a str, lower: &str) -> Option<&'a str> {
    [
        "adults with ",
        "adult patients with ",
        "adult patient with ",
        "pediatric patients with ",
        "pediatric patient with ",
        "patients with ",
        "patient with ",
        "children with ",
        "women with ",
        "people with ",
        "treatment of ",
    ]
    .iter()
    .find_map(|needle| lower.find(needle).map(|idx| &segment[idx + needle.len()..]))
}

fn label_candidate_cutoff(segment: &str) -> &str {
    let lower = segment.to_ascii_lowercase();
    let mut end = segment.len();
    for needle in [
        ",",
        ";",
        " in combination",
        " as a single agent",
        " as first-line",
        " as first line",
        " as adjuvant",
        " as determined",
        " for ",
        " in adult",
        " in pediatric",
        " in patients",
        " after ",
        " following ",
        " who ",
        " who:",
        " whose ",
        " where ",
    ] {
        if let Some(idx) = lower.find(needle) {
            end = end.min(idx);
        }
    }
    segment[..end].trim()
}

fn normalize_label_indication_name(segment: &str) -> Option<String> {
    let candidate = strip_leading_label_indication_prefixes(label_candidate_cutoff(segment))
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, ':' | ';' | '.' | '-'))
        .trim();
    if candidate.is_empty() {
        return None;
    }
    let lower = candidate.to_ascii_lowercase();
    let has_disease_signal = [
        "cancer",
        "carcinoma",
        "melanoma",
        "lymphoma",
        "leukemia",
        "tumor",
        "tumours",
        "myeloma",
        "sarcoma",
        "nsclc",
        "hnscc",
        "rcc",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    if !has_disease_signal {
        return None;
    }
    Some(candidate.to_string())
}

fn extract_label_indication_name(segment: &str) -> Option<String> {
    let segment = strip_label_intro_prefix(segment);
    let lower = segment.to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    if !label_continuation_prefix(&lower)
        && let Some(candidate) = normalize_label_indication_name(segment)
    {
        return Some(candidate);
    }
    let patient_slice = label_patient_phrase_start(segment, &lower)?;
    normalize_label_indication_name(patient_slice)
}

fn label_drug_markers(label_response: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for key in ["brand_name", "generic_name"] {
        for value in extract_openfda_values(label_response, key) {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            let dedupe_key = trimmed.to_ascii_lowercase();
            if seen.insert(dedupe_key) {
                out.push(trimmed.to_string());
            }
        }
    }
    out
}

fn extract_label_numbered_subsection_name(
    section: &str,
    drug_markers: &[String],
) -> Option<String> {
    let section = label_numbered_subsection_prefix_regex()
        .replace(section, "")
        .into_owned();
    let section = section.trim();
    if section.is_empty() {
        return None;
    }

    let end = drug_markers
        .iter()
        .filter_map(|marker| find_ascii_case_insensitive(section, marker))
        .filter(|idx| *idx > 0)
        .min();
    let candidate = end.map(|idx| &section[..idx]).unwrap_or(section);
    let candidate = candidate
        .trim()
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, ':' | ';' | '.' | '-' | '•'))
        .trim();
    if candidate.is_empty() {
        return None;
    }
    Some(candidate.to_string())
}

fn push_label_indication_summary_row(
    out: &mut Vec<DrugLabelIndication>,
    seen: &mut HashSet<String>,
    name: String,
    max_rows: usize,
) -> bool {
    let dedupe_key = name.to_ascii_lowercase();
    if !seen.insert(dedupe_key) {
        return false;
    }

    out.push(DrugLabelIndication {
        name,
        approval_date: None,
        pivotal_trial: None,
    });
    out.len() >= max_rows
}

fn extract_label_indication_summary(
    label_response: &serde_json::Value,
) -> Vec<DrugLabelIndication> {
    const MAX_SUMMARY_ROWS: usize = 20;

    let Some(indications_text) = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .and_then(|top| label_text(top.get("indications_and_usage")))
    else {
        return Vec::new();
    };

    let normalized = normalize_label_whitespace(&indications_text);
    let stripped = label_heading_regex().replace(&normalized, "").into_owned();

    let mut out: Vec<DrugLabelIndication> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let subsection_starts = label_numbered_subsection_heading_regex()
        .find_iter(&stripped)
        .map(|m| m.start())
        .collect::<Vec<_>>();
    if !subsection_starts.is_empty() {
        let drug_markers = label_drug_markers(label_response);
        for (idx, start) in subsection_starts.iter().enumerate() {
            let end = subsection_starts
                .get(idx + 1)
                .copied()
                .unwrap_or(stripped.len());
            let section = stripped[*start..end].trim();
            let Some(name) = extract_label_numbered_subsection_name(section, &drug_markers)
                .or_else(|| extract_label_indication_name(section))
            else {
                continue;
            };
            if push_label_indication_summary_row(&mut out, &mut seen, name, MAX_SUMMARY_ROWS) {
                return out;
            }
        }
        if !out.is_empty() {
            return out;
        }
    }

    for segment in label_subsection_boundary_regex().split(&stripped) {
        let Some(name) = extract_label_indication_name(segment) else {
            continue;
        };
        if push_label_indication_summary_row(&mut out, &mut seen, name, MAX_SUMMARY_ROWS) {
            break;
        }
    }
    out
}

pub(super) fn extract_inline_label(
    label_response: &serde_json::Value,
    raw_mode: bool,
) -> Option<DrugLabel> {
    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;

    let indication_summary = extract_label_indication_summary(label_response);
    let label_set_id = label_set_id_from_result(top);
    let raw_indications = label_text(top.get("indications_and_usage")).map(|v| {
        truncate_with_note(
            &normalize_label_whitespace(&v),
            LABEL_MAX_CHARS,
            label_set_id,
        )
    });
    let boxed_warning = label_text(top.get("boxed_warning")).map(|v| {
        truncate_with_note(
            &normalize_label_whitespace(&v),
            LABEL_MAX_CHARS,
            label_set_id,
        )
    });
    let raw_warnings = label_text(top.get("warnings_and_cautions"))
        .or_else(|| label_text(top.get("warnings")))
        .map(|v| {
            truncate_with_note(
                &normalize_label_whitespace(&v),
                LABEL_MAX_CHARS,
                label_set_id,
            )
        });
    let raw_dosage = label_text(top.get("dosage_and_administration")).map(|v| {
        truncate_with_note(
            &normalize_label_whitespace(&v),
            LABEL_MAX_CHARS,
            label_set_id,
        )
    });

    let indications = if raw_mode || indication_summary.is_empty() {
        raw_indications
    } else {
        None
    };
    let warnings = if raw_mode { raw_warnings } else { None };
    let dosage = if raw_mode { raw_dosage } else { None };

    if indication_summary.is_empty()
        && indications.is_none()
        && boxed_warning.is_none()
        && warnings.is_none()
        && dosage.is_none()
    {
        return None;
    }

    Some(DrugLabel {
        indication_summary,
        indications,
        boxed_warning,
        warnings,
        dosage,
    })
}

pub(super) fn extract_label_warnings_text(label_response: &serde_json::Value) -> Option<String> {
    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;
    label_text(top.get("warnings_and_cautions"))
        .or_else(|| label_text(top.get("warnings")))
        .map(|value| {
            truncate_with_note(
                &normalize_label_whitespace(&value),
                LABEL_MAX_CHARS,
                label_set_id_from_result(top),
            )
        })
}

pub(super) fn extract_label_boxed_warning(label_response: &serde_json::Value) -> Option<String> {
    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;
    label_text(top.get("boxed_warning")).map(|value| {
        truncate_with_note(
            &normalize_label_whitespace(&value),
            LABEL_MAX_CHARS,
            label_set_id_from_result(top),
        )
    })
}

fn label_set_id_from_result(top: &serde_json::Value) -> Option<&str> {
    top.get("set_id")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            top.get("openfda")
                .and_then(|v| v.get("spl_set_id"))
                .and_then(|v| match v {
                    serde_json::Value::String(s) => Some(s.as_str()),
                    serde_json::Value::Array(items) => items.iter().find_map(|item| item.as_str()),
                    _ => None,
                })
        })
        .map(str::trim)
        .filter(|v| !v.is_empty())
}

pub(super) fn extract_label_set_id(label_response: &serde_json::Value) -> Option<String> {
    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;

    label_set_id_from_result(top).map(str::to_string)
}

pub(super) fn extract_interaction_text_from_label(
    label_response: &serde_json::Value,
) -> Option<String> {
    const LABEL_MAX_CHARS: usize = 2000;

    let top = label_response
        .get("results")
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())?;

    label_text(top.get("drug_interactions"))
        .map(|v| truncate_with_note(&v, LABEL_MAX_CHARS, label_set_id_from_result(top)))
}

pub(super) fn extract_openfda_values_from_result(
    result: &serde_json::Value,
    key: &str,
) -> Vec<String> {
    let Some(top) = result.get("openfda").and_then(|v| v.get(key)) else {
        return Vec::new();
    };

    match top {
        serde_json::Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                Vec::new()
            } else {
                vec![s.to_string()]
            }
        }
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn extract_openfda_values(label_response: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for result in results {
        let values = extract_openfda_values_from_result(result, key);
        for value in values {
            let key = value.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }
            out.push(value);
        }
    }
    out
}

#[cfg(test)]
mod tests;
