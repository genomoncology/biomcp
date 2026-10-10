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
pub(super) const ELEMENTS_SEARCH_OVERSIZE_NOTE: &str = "The openFDA label product-data-elements search response was too large to read, so no label record could be confirmed.";

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

/// The brand that opens each SPL product data element entry, as written:
/// the only brand identity a record with an empty `openfda` block carries
/// ("DARZALEX Daratumumab DARATUMUMAB …" names DARZALEX), so a card whose
/// chosen label is a sparse record still lists its own brand (ticket 2047).
pub(super) fn element_leading_brand_names(label_response: &serde_json::Value) -> Vec<String> {
    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for result in results {
        if !is_sparse_identity_record(result) {
            continue;
        }
        for element in spl_product_data_elements(result) {
            let brand = element
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_matches(|c: char| matches!(c, ',' | ';' | ':' | '.'))
                .trim();
            if brand.is_empty() {
                continue;
            }
            if seen.insert(brand.to_ascii_lowercase()) {
                out.push(brand.to_string());
            }
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

/// The identity token runs of one SPL product data element.
///
/// A real element concatenates one entry per strength, and every entry
/// opens with the same names ("TAGRISSO osimertinib OSIMERTINIB OSIMERTINIB
/// MANNITOL ... AZ;40 TAGRISSO ..."), so an inactive excipient repeats
/// across strengths and raw repetition cannot separate names from
/// excipients. Entries split where the element's leading two-token name
/// window recurs, which is each next strength's opening. Within one entry
/// the identity is the leading name run: the entry's first token, then the
/// established name's tokens, which the entry spells out and then lists
/// again whole or componentwise, so the longest prefix of the remaining
/// tokens that recurs later marks them. The run ends at the first token the
/// name never repeats, which is the entry's first inactive excipient or
/// strength code.
fn spl_element_identity_heads(element: &str) -> Vec<Vec<String>> {
    let tokens = identity_tokens(element);
    if tokens.is_empty() {
        return Vec::new();
    }
    let window = tokens.len().min(2);
    let mut heads = Vec::new();
    let mut entry_start = 0;
    for position in 1..=(tokens.len() - window) {
        if tokens[position..position + window] == tokens[..window] {
            heads.push(entry_name_run(&tokens[entry_start..position]));
            entry_start = position;
        }
    }
    heads.push(entry_name_run(&tokens[entry_start..]));
    heads
}

/// The leading identity run of one element entry: the first token, then the
/// recurring established-name tokens, ending at the first token the name
/// never repeats.
fn entry_name_run(entry: &[String]) -> Vec<String> {
    if entry.is_empty() {
        return Vec::new();
    }
    let mut name_end = 1;
    loop {
        let end = name_end + 1;
        if end > entry.len() {
            break;
        }
        let prefix = &entry[1..end];
        let recurs = entry[end..]
            .windows(prefix.len())
            .any(|window| window == prefix);
        if !recurs {
            break;
        }
        name_end = end;
    }
    let name: std::collections::HashSet<&str> =
        entry[1..name_end].iter().map(String::as_str).collect();
    let mut run = vec![entry[0].clone()];
    for token in &entry[1..] {
        if !name.contains(token.as_str()) {
            break;
        }
        run.push(token.clone());
    }
    run
}

/// Whether a broad-search result is the requested drug's own record.
///
/// A broad search for one drug routinely ranks other drugs' labels first
/// because their section text mentions the queried name. A result only
/// counts when the requested name matches the record's own identity fields
/// as a word sequence within a single field — never a sequence joined across
/// two fields — or the per-strength name runs of an SPL product data element,
/// so an inactive excipient such as mannitol never identifies a record.
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
            .flat_map(|element| spl_element_identity_heads(element)),
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

/// Keep only broad-search results whose own identity matches the drug name.
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
    /// The product-data-elements fallback response exceeded the body read
    /// limit. The same query would download the same oversize response, so
    /// this is a confirmed inability to read a match, not a retryable fetch
    /// failure.
    ElementsSearchTooLarge,
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

/// Whether a product name names a combination: a second ingredient joined
/// by "and" or a comma-separated list. Salts and hydrates ("niraparib
/// tosylate monohydrate", "erlotinib hydrochloride") are single-ingredient
/// qualified names, not combinations (ticket 2043).
fn is_combination_product_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains(" and ") || lower.contains(',')
}

/// The plain ingredient of an under-the-skin pairing: subcutaneous antibody
/// products carry one "ingredient and hyaluronidase-xxxx" generic name, and
/// the ingredient before "and" is the drug itself ("daratumumab and
/// hyaluronidase-fihj" names daratumumab). The split serves only that
/// two-name hyaluronidase pairing shape: a comma-list combination
/// ("pertuzumab, trastuzumab, and hyaluronidase-zzxf") and any other
/// "X and Y" product ("nivolumab and relatlimab-rmbw") name no single
/// plain ingredient, so the whole name stands and a combination never
/// becomes another product's card name (tickets 2043 and 2047).
pub(super) fn plain_paired_ingredient_name(name: &str) -> &str {
    let lower = name.to_ascii_lowercase();
    match lower.find(" and hyaluronidase") {
        Some(index) => {
            let plain = name[..index].trim();
            if plain.contains(" and ") || plain.contains(',') {
                name.trim()
            } else {
                plain
            }
        }
        None => name.trim(),
    }
}

/// The rank tiers of one label result for the requested drug (ticket
/// 2043). The request decides between the two sides of the rule: a brand
/// query takes the record whose own brand name equals it, and an ingredient
/// query takes the plain single-ingredient product's record, never a
/// combination, biosimilar or under-the-skin form.
const LABEL_RANK_BRAND_MATCH: u8 = 3;
const LABEL_RANK_PLAIN_INGREDIENT: u8 = 2;
const LABEL_RANK_QUALIFIED_FORM: u8 = 1;
const LABEL_RANK_OPENFDA_ORDER: u8 = 0;

fn generic_name_matches(result: &serde_json::Value, matches: impl Fn(&str) -> bool) -> bool {
    extract_openfda_values_from_result(result, "generic_name")
        .iter()
        .any(|name| {
            let lower = name.trim().to_ascii_lowercase();
            !lower.is_empty() && matches(&lower)
        })
}

/// The brand and established-name identity a sparse record's SPL product
/// data elements carry, the only identity such a record has: each
/// per-strength entry opens "BRAND ESTABLISHED [ESTABLISHED repeats]
/// excipients…" ("DARZALEX Daratumumab DARATUMUMAB DARATUMUMAB ACETIC ACID…"
/// names the DARZALEX brand and the daratumumab ingredient), so the entry
/// head's leading token is the brand and its recurring name run after it is
/// the established name. A head that carries only the brand token (an
/// "X and hyaluronidase" combination whose second name never recurs) names
/// no established ingredient, so it contributes no brand either (ticket
/// 2047).
fn sparse_element_identity_names(result: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    let mut brands = Vec::new();
    let mut generics = Vec::new();
    for run in spl_product_data_elements(result)
        .iter()
        .flat_map(|element| spl_element_identity_heads(element))
    {
        let Some((brand, established)) = run.split_first() else {
            continue;
        };
        if established.is_empty() {
            continue;
        }
        brands.push(brand.clone());
        let mut tokens: Vec<&String> = Vec::new();
        for token in established {
            if !tokens.contains(&token) {
                tokens.push(token);
            }
        }
        generics.push(tokens.into_iter().cloned().collect::<Vec<_>>().join(" "));
    }
    (brands, generics)
}

/// Whether a record's identity fields are the sparse shape whose only
/// identity is the SPL product data element line: no `openfda` generic or
/// brand name at all (the plain DARZALEX and PHESGO records ship this way).
fn is_sparse_identity_record(result: &serde_json::Value) -> bool {
    extract_openfda_values_from_result(result, "generic_name").is_empty()
        && extract_openfda_values_from_result(result, "brand_name").is_empty()
}

fn label_result_rank(result: &serde_json::Value, requested: &str, card: &str) -> u8 {
    let requested = requested.trim().to_ascii_lowercase();
    let card = card.trim().to_ascii_lowercase();
    let (element_brands, element_generics) = if is_sparse_identity_record(result) {
        sparse_element_identity_names(result)
    } else {
        (Vec::new(), Vec::new())
    };
    if extract_openfda_values_from_result(result, "brand_name")
        .iter()
        .any(|brand| brand.trim().to_ascii_lowercase() == requested)
        || element_brands.contains(&requested)
    {
        return LABEL_RANK_BRAND_MATCH;
    }
    if !requested.is_empty()
        && (generic_name_matches(result, |name| {
            !is_combination_product_name(name) && (name == requested || name == card)
        }) || element_generics.iter().any(|name| {
            !is_combination_product_name(name) && (name == &requested || name == &card)
        }))
    {
        return LABEL_RANK_PLAIN_INGREDIENT;
    }
    if !requested.is_empty()
        && (generic_name_matches(result, |name| {
            !is_combination_product_name(name)
                && (name.starts_with(&format!("{requested} "))
                    || name.starts_with(&format!("{requested}-"))
                    || name.starts_with(&format!("{card} "))
                    || name.starts_with(&format!("{card}-")))
        }) || element_generics.iter().any(|name| {
            !is_combination_product_name(name)
                && (name.starts_with(&format!("{requested} "))
                    || name.starts_with(&format!("{requested}-"))
                    || name.starts_with(&format!("{card} "))
                    || name.starts_with(&format!("{card}-")))
        }))
    {
        return LABEL_RANK_QUALIFIED_FORM;
    }
    LABEL_RANK_OPENFDA_ORDER
}

/// Choose the response's label record for the requested drug: the
/// highest-ranked result moves to the front and the rest keep openFDA's
/// order behind it, so every downstream reader of `results[0]` sees the
/// chosen record. Returns the reordered response with the chosen record's
/// rank (ticket 2043).
fn choose_label_response(
    response: serde_json::Value,
    requested: &str,
    card: &str,
) -> Option<(serde_json::Value, u8)> {
    let results = response.get("results")?.as_array()?.clone();
    if results.is_empty() {
        return None;
    }
    let mut best_index = 0;
    let mut best_rank = LABEL_RANK_OPENFDA_ORDER;
    for (index, result) in results.iter().enumerate() {
        let rank = label_result_rank(result, requested, card);
        if rank > best_rank {
            best_rank = rank;
            best_index = index;
        }
    }
    let mut reordered = results;
    let chosen = reordered.remove(best_index);
    reordered.insert(0, chosen);
    let mut chosen_response = response;
    chosen_response["results"] = serde_json::Value::Array(reordered);
    Some((chosen_response, best_rank))
}

/// The sparse-record escalation for the label choice: a plain product's
/// current record can carry no `openfda` identity block at all (the plain
/// DARZALEX and PHESGO records ship that way), so no field-scoped search
/// or tier ever sees it and the field-scoped answer serves the
/// under-the-skin pairing instead. The product-data-elements search
/// reaches the sparse record, the identity guard keeps only records whose
/// own names match the searched name, and the element-derived identity
/// ranks them, so a record that outranks the unconfirmed fallback serves
/// before it (ticket 2047). A fetch error skips the escalation; the final
/// elements fallback re-runs the card's own query and settles its errors
/// there.
async fn sparse_elements_choice(
    client: &OpenFdaClient,
    requested_name: &str,
    card_name: &str,
    fallback_rank: u8,
) -> Result<Option<serde_json::Value>, BioMcpError> {
    let mut names: Vec<&str> = Vec::new();
    for name in [requested_name, card_name] {
        if !names
            .iter()
            .any(|seen| seen.trim().eq_ignore_ascii_case(name.trim()))
            && !name.trim().is_empty()
        {
            names.push(name);
        }
    }
    for name in names {
        let Ok(Some(response)) = client.label_elements_search(name).await else {
            continue;
        };
        let Some(filtered) = filter_label_response_to_identity(&response, name) else {
            continue;
        };
        if let Some((chosen, rank)) = choose_label_response(filtered, requested_name, card_name)
            && rank > fallback_rank
        {
            return Ok(Some(chosen));
        }
    }
    Ok(None)
}

/// Field-scoped label lookup with the two-sided label choice and the
/// sparse-metadata elements fallback (tickets 2043, 1300 and 2047).
///
/// The request's own name runs first, the card's canonical name second,
/// and a brand-equal or plain-ingredient record locks the answer. When the
/// newest-first page holds only biosimilars and qualified forms, the
/// exact-field escalation asks for the plain product's own record
/// directly, and the sparse-record elements escalation reaches a plain
/// product whose record carries no `openfda` identity block at all; a
/// surviving qualified form (a salt or a proper-name suffix such as
/// "amivantamab-vmjw") serves next, and openFDA's own order is the
/// last resort, so a brand keeps a biosimilar's label only when nothing
/// else answers. `NoSplRecord` means openFDA answered and no SPL record for
/// the drug exists; real fetch errors surface as `Err`, except the oversize
/// elements response, which settles as `ElementsSearchTooLarge` because no
/// retry can succeed.
pub(super) async fn lookup_label_response(
    client: &OpenFdaClient,
    requested_name: &str,
    card_name: &str,
) -> Result<LabelLookup, BioMcpError> {
    let mut fallback: Option<serde_json::Value> = None;
    let mut fallback_rank = LABEL_RANK_OPENFDA_ORDER;
    let mut queries: Vec<&str> = Vec::new();
    for query in [requested_name, card_name] {
        if !queries
            .iter()
            .any(|seen| seen.trim().eq_ignore_ascii_case(query.trim()))
        {
            queries.push(query);
        }
    }
    for query in queries {
        let Some(response) = client.label_search(query).await? else {
            continue;
        };
        let Some((chosen, rank)) = choose_label_response(response, requested_name, card_name)
        else {
            continue;
        };
        if rank >= LABEL_RANK_PLAIN_INGREDIENT {
            return Ok(LabelLookup::Response(chosen));
        }
        if fallback.is_none() {
            fallback_rank = rank;
            fallback = Some(chosen);
        }
    }
    if let Ok(Some(response)) = client
        .label_generic_exact_search(&card_name.trim().to_ascii_uppercase())
        .await
        && label_response_has_result(&response)
    {
        return Ok(LabelLookup::Response(response));
    }
    if let Some(chosen) =
        sparse_elements_choice(client, requested_name, card_name, fallback_rank).await?
    {
        return Ok(LabelLookup::Response(chosen));
    }
    if let Some(fallback) = fallback {
        return Ok(LabelLookup::Response(fallback));
    }
    let elements = match client.label_elements_search(card_name).await {
        Ok(Some(response)) => response,
        Ok(None) => return Ok(LabelLookup::NoSplRecord),
        Err(error) => {
            return if is_body_limit_error(&error) {
                Ok(LabelLookup::ElementsSearchTooLarge)
            } else {
                Err(error)
            };
        }
    };
    Ok(filter_label_response_to_identity(&elements, card_name)
        .map_or(LabelLookup::NoSplRecord, LabelLookup::Response))
}

/// The withdrawn Propulsid record's product data elements exactly as openFDA
/// serves them (captured 2026-10-07): three per-strength entries with
/// differing inactive excipient lists and no populated `openfda` identity
/// fields. Shared by the identity-guard tests and the fallback fixture
/// server.
#[cfg(test)]
pub(crate) const PROPULSID_ELEMENTS: &str = "Propulsid cisapride cisapride cisapride silicon dioxide lactose monohydrate magnesium stearate cellulose, microcrystalline polysorbate 20 povidone Janssen;P;10 Propulsid cisapride cisapride cisapride silicon dioxide lactose monohydrate magnesium stearate cellulose, microcrystalline polysorbate 20 povidone FD&C Blue No. 2 aluminum oxide Janssen;P;20 Propulsid cisapride cisapride cisapride methylparaben cellulose, microcrystalline carboxymethylcellulose sodium polysorbate 20 propylparaben sodium chloride sorbitol FD&C Red No. 40 water bright pink cherry cream";

#[cfg(test)]
mod tests;
