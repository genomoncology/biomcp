//! Drug label parsing and OpenFDA label-field extraction helpers.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;

use super::{DrugLabel, DrugLabelIndication};
use crate::error::BioMcpError;
use crate::sources::openfda::OpenFdaClient;

const LABEL_MAX_CHARS: usize = 2000;

pub(super) const LABEL_UNAVAILABLE_MESSAGE: &str =
    "OpenFDA label evidence is temporarily unavailable.";
pub(super) const NO_SPL_RECORD_NOTE: &str = "No openFDA SPL label record matched this drug.";
pub(super) const NO_LABEL_TEXT_NOTE: &str =
    "The matched openFDA SPL record carries no label section text.";
pub(super) const FULLTEXT_OVERSIZE_NOTE: &str = "The openFDA full-text label search response was too large to read, so no label record could be confirmed.";

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

/// The DailyMed full-label URL for a SET ID; empty ids map to None.
pub(crate) fn dailymed_setid_url(set_id: &str) -> Option<String> {
    let set_id = set_id.trim();
    if set_id.is_empty() {
        return None;
    }
    let mut url = reqwest::Url::parse("https://dailymed.nlm.nih.gov/dailymed/drugInfo.cfm").ok()?;
    url.query_pairs_mut().append_pair("setid", set_id);
    Some(url.into())
}

fn truncate_with_note(value: &str, max_chars: usize, label_set_id: Option<&str>) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let truncated = value.chars().take(max_chars).collect::<String>();
    let total = value.chars().count();
    let full_label = label_set_id.and_then(dailymed_setid_url);
    match full_label {
        Some(url) => format!("{truncated}\n\n(truncated, {total} chars total; full label: {url})"),
        None => format!("{truncated}\n\n(truncated, {total} chars total)"),
    }
}

/// The Markdown view of a label: whole sections stay in JSON, while readable
/// output keeps a capped short form that points to the rest of the label.
pub(crate) fn markdown_label_view(label: &DrugLabel, label_set_id: Option<&str>) -> DrugLabel {
    let shorten = |value: &Option<String>| {
        value
            .as_deref()
            .map(|text| truncate_with_note(text, LABEL_MAX_CHARS, label_set_id))
    };
    DrugLabel {
        indication_summary: label.indication_summary.clone(),
        indications: shorten(&label.indications),
        boxed_warning: shorten(&label.boxed_warning),
        warnings: shorten(&label.warnings),
        dosage: shorten(&label.dosage),
    }
}

/// Whether the Markdown view cuts any label section it carries.
///
/// The short-form pointer may only print when the output actually omits
/// content, so a view whose sections all fit the cap points nowhere.
pub(crate) fn markdown_label_view_truncates(label: &DrugLabel) -> bool {
    [
        label.indications.as_deref(),
        label.boxed_warning.as_deref(),
        label.warnings.as_deref(),
        label.dosage.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|text| text.chars().count() > LABEL_MAX_CHARS)
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
    // JSON carries whole label sections; Markdown applies its own cap through
    // `markdown_label_view` at render time.
    let raw_indications =
        label_text(top.get("indications_and_usage")).map(|v| normalize_label_whitespace(&v));
    let boxed_warning =
        label_text(top.get("boxed_warning")).map(|v| normalize_label_whitespace(&v));
    let raw_warnings = label_text(top.get("warnings_and_cautions"))
        .or_else(|| label_text(top.get("warnings")))
        .map(|v| normalize_label_whitespace(&v));
    let raw_dosage =
        label_text(top.get("dosage_and_administration")).map(|v| normalize_label_whitespace(&v));

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

/// Lowercased alphanumeric tokens, so hyphenated and punctuated identity
/// names compare by word rather than by raw substring.
fn identity_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn result_identity_values(result: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    for key in ["generic_name", "brand_name", "substance_name"] {
        out.extend(extract_openfda_values_from_result(result, key));
    }
    if let Some(ingredients) = result.get("active_ingredients").and_then(|v| v.as_array()) {
        for ingredient in ingredients {
            if let Some(name) = ingredient.get("name").and_then(|v| v.as_str()) {
                out.push(name.to_string());
            }
        }
    }
    out
}

fn spl_product_data_elements(result: &serde_json::Value) -> Vec<&str> {
    let Some(elements) = result
        .get("spl_product_data_elements")
        .and_then(|v| v.as_array())
    else {
        return Vec::new();
    };
    elements.iter().filter_map(|v| v.as_str()).collect()
}

/// The leading run of an SPL product data element: the product's own names.
///
/// Each element opens with the brand and the active ingredient, the generic
/// repeated in several letter cases, and ends with the inactive excipient
/// list ("TAGRISSO osimertinib OSIMERTINIB OSIMERTINIB MANNITOL ..."). A
/// token belongs to the run while it is the element's first token or its
/// case-insensitive form appears more than once in the element; the run ends
/// at the first single-use token, which is the first inactive ingredient.
/// Strength and lot codes that trail the names are single-use too.
fn spl_active_name_run(element: &str) -> Vec<String> {
    let tokens = identity_tokens(element);
    if tokens.is_empty() {
        return Vec::new();
    }
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for token in &tokens {
        *counts.entry(token.as_str()).or_insert(0) += 1;
    }
    let mut run = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if index > 0 && counts.get(token.as_str()).is_none_or(|count| *count < 2) {
            break;
        }
        run.push(token.clone());
    }
    run
}

/// Whether a full-text result is the requested drug's own record.
///
/// A full-text search for one drug routinely ranks other drugs' labels first
/// because their section text mentions the queried name (a full-text
/// "osimertinib" search returns amivantamab's label first). A result only
/// counts when the requested name matches the record's own identity fields as
/// a word sequence within a single field — never a sequence joined across two
/// fields — or the leading active-name run of an SPL product data element, so
/// an inactive excipient such as mannitol never identifies a record.
pub(super) fn label_result_matches_identity(result: &serde_json::Value, name: &str) -> bool {
    let name_tokens = identity_tokens(name);
    if name_tokens.is_empty() {
        return false;
    }
    let mut identity_runs: Vec<Vec<String>> = result_identity_values(result)
        .iter()
        .map(|value| identity_tokens(value))
        .collect();
    identity_runs.extend(
        spl_product_data_elements(result)
            .iter()
            .map(|element| spl_active_name_run(element)),
    );
    identity_runs.into_iter().any(|tokens| {
        tokens
            .windows(name_tokens.len())
            .any(|window| window == name_tokens.as_slice())
    })
}

fn label_response_has_result(response: &serde_json::Value) -> bool {
    response
        .get("results")
        .and_then(|v| v.as_array())
        .is_some_and(|results| !results.is_empty())
}

/// Keep only full-text results whose own identity matches the drug name.
pub(super) fn filter_label_response_to_identity(
    response: &serde_json::Value,
    name: &str,
) -> Option<serde_json::Value> {
    let results = response.get("results")?.as_array()?;
    let matching = results
        .iter()
        .filter(|result| label_result_matches_identity(result, name))
        .cloned()
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return None;
    }
    let mut filtered = response.clone();
    if let Some(counts) = filtered
        .get_mut("meta")
        .and_then(|v| v.get_mut("results"))
        .and_then(|v| v.as_object_mut())
    {
        counts.insert(
            "total".to_string(),
            serde_json::Value::from(matching.len() as u64),
        );
    }
    filtered["results"] = serde_json::Value::Array(matching);
    Some(filtered)
}

/// What the openFDA label lookup established for a drug.
pub(super) enum LabelLookup {
    /// A label response to use.
    Response(serde_json::Value),
    /// Both lookups answered and no SPL record for the drug exists.
    NoSplRecord,
    /// The full-text fallback response exceeded the body read limit. The
    /// same query would download the same oversize response, so this is a
    /// confirmed inability to read a match, not a retryable fetch failure.
    FulltextTooLarge,
}

/// Whether an openFDA error is the response-body read limit, including the
/// source-context wrapping the transport layers add.
fn is_body_limit_error(error: &BioMcpError) -> bool {
    match error {
        BioMcpError::WithSourceContext { source, .. } => is_body_limit_error(source),
        BioMcpError::BodyLimit { .. } => true,
        _ => false,
    }
}

/// Field-scoped label lookup with the sparse-metadata full-text fallback.
///
/// When the `openfda.generic_name`/`brand_name` query returns no match, fall
/// back to a full-text search for the name and keep only results whose own
/// identity matches. `NoSplRecord` means openFDA answered and no SPL record
/// for the drug exists; real fetch errors surface as `Err`, except the
/// oversize full-text response, which settles as `FulltextTooLarge` because
/// no retry can succeed.
pub(super) async fn lookup_label_response(
    client: &OpenFdaClient,
    name: &str,
) -> Result<LabelLookup, BioMcpError> {
    if let Some(response) = client.label_search(name).await?
        && label_response_has_result(&response)
    {
        return Ok(LabelLookup::Response(response));
    }
    let fulltext = match client.label_fulltext_search(name).await {
        Ok(Some(response)) => response,
        Ok(None) => return Ok(LabelLookup::NoSplRecord),
        Err(error) => {
            return if is_body_limit_error(&error) {
                Ok(LabelLookup::FulltextTooLarge)
            } else {
                Err(error)
            };
        }
    };
    Ok(filter_label_response_to_identity(&fulltext, name)
        .map_or(LabelLookup::NoSplRecord, LabelLookup::Response))
}

#[cfg(test)]
mod tests;
