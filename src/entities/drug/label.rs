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

pub(super) fn extract_label_effective_date(response: &serde_json::Value) -> Option<String> {
    let raw = response
        .get("results")?
        .as_array()?
        .first()?
        .get("effective_time")?
        .as_str()?;
    chrono::NaiveDate::parse_from_str(raw, "%Y%m%d")
        .ok()
        .map(|date| date.format("%Y-%m-%d").to_string())
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

/// The brands a label response's own products carry for the card's drug.
/// A record that states an openFDA generic line contributes its brands only
/// when that line names the card's drug as the card's own product, so
/// another drug's record that only matched the search never donates its
/// brand. A sparse record — no `openfda` identity block at all — contributes
/// the brand its SPL product data element opens with: such records reach a
/// card only through the identity-guarded elements escalation, which keeps
/// records whose own element identity matches the searched name.
pub(super) fn card_own_label_brands(
    label_response: &serde_json::Value,
    card_name: &str,
) -> Vec<String> {
    let Some(results) = label_response.get("results").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for result in results {
        if is_sparse_identity_record(result) {
            out.extend(sparse_element_identity_names(result).0);
            continue;
        }
        let generics: Vec<_> = extract_openfda_values_from_result(result, "generic_name")
            .into_iter()
            .filter(|generic| label_generic_names_card(generic, card_name))
            .collect();
        if !generics.is_empty() {
            out.extend(
                extract_openfda_values_from_result(result, "brand_name")
                    .into_iter()
                    .filter(|brand| {
                        !generics
                            .iter()
                            .any(|generic| generic.trim().eq_ignore_ascii_case(brand.trim()))
                    }),
            );
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

/// Whether two token runs read the same, ignoring letter case: element
/// lines mix "DARZALEX Daratumumab DARATUMUMAB" spellings of one name.
fn tokens_equal_ignore_case(left: &[String], right: &[String]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(l, r)| l.eq_ignore_ascii_case(r))
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

/// The element's identity tokens as written, so the brand run keeps its
/// original spelling while every structural comparison stays
/// case-insensitive.
fn element_tokens(element: &str) -> Vec<String> {
    element
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

/// The identity token runs of one SPL product data element.
/// A real element concatenates one entry per strength, and every entry
/// opens with the same names ("TAGRISSO osimertinib OSIMERTINIB OSIMERTINIB
/// MANNITOL ... AZ;40 TAGRISSO ..."), so an inactive excipient repeats
/// across strengths and raw repetition cannot separate names from
/// excipients. Entries split where the element's leading two-token name
/// window recurs, which is each next strength's opening. Within one entry
/// the identity is the leading name run: the entry's brand tokens, then
/// the established name's tokens, which the entry spells out and then
/// lists again whole or componentwise, so the longest prefix of the
/// remaining tokens that recurs later marks them.
fn spl_element_identity_heads(element: &str) -> Vec<(Vec<String>, Vec<String>)> {
    let tokens = element_tokens(element);
    if tokens.is_empty() {
        return Vec::new();
    }
    let window = tokens.len().min(2);
    let mut heads = Vec::new();
    let mut entry_start = 0;
    for position in 1..=(tokens.len() - window) {
        if tokens_equal_ignore_case(&tokens[position..position + window], &tokens[..window]) {
            heads.push(entry_identity_run(&tokens[entry_start..position]));
            entry_start = position;
        }
    }
    heads.push(entry_identity_run(&tokens[entry_start..]));
    heads
}

/// The leading identity runs of one element entry: the brand tokens, then
/// the established name's tokens. The established name recurs in the entry
/// the entry spells it out and then repeats it — so the first token that
/// recurs later marks where the established name begins, and every token
/// before it belongs to the brand: a multi-word brand stays whole
/// ("Herceptin Hylecta Trastuzumab and hyaluronidase-oysk TRASTUZUMAB …"
/// names HERCEPTIN HYLECTA over trastuzumab). An entry whose
/// tokens never recur names no established ingredient, so it contributes
/// neither brand nor generic.
fn entry_identity_run(entry: &[String]) -> (Vec<String>, Vec<String>) {
    let Some(start) = (1..entry.len()).find(|&position| {
        entry[position + 1..]
            .iter()
            .any(|later| later.eq_ignore_ascii_case(&entry[position]))
    }) else {
        return (Vec::new(), Vec::new());
    };
    let brand = entry[..start].to_vec();
    let tail = &entry[start..];
    let mut name_end = 1;
    loop {
        let end = name_end + 1;
        if end > tail.len() {
            break;
        }
        let prefix = &tail[..end];
        let recurs = tail[end..]
            .windows(prefix.len())
            .any(|window| tokens_equal_ignore_case(window, prefix));
        if !recurs {
            break;
        }
        name_end = end;
    }
    let name: std::collections::HashSet<String> = tail[..name_end]
        .iter()
        .map(|token| token.to_ascii_lowercase())
        .collect();
    let mut established: Vec<String> = Vec::new();
    for token in tail {
        if !name.contains(&token.to_ascii_lowercase()) {
            break;
        }
        established.push(token.clone());
    }
    (brand, established)
}

/// Whether a broad-search result is the requested drug's own record.
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
            .flat_map(|element| spl_element_identity_heads(element))
            .map(|(brand, established)| brand.into_iter().chain(established).collect::<Vec<_>>()),
    );
    identity_runs.into_iter().any(|tokens| {
        tokens
            .windows(name_tokens.len())
            .any(|window| tokens_equal_ignore_case(window, &name_tokens))
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
/// qualified names, not combinations.
fn is_combination_product_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains(" and ") || lower.contains(',')
}

/// Split one "A and B" pairing name into its halves: exactly two names,
/// joined by one " and ", with no comma list on the left ("pertuzumab,
/// trastuzumab, and hyaluronidase-zzxf" stays whole).
fn split_pairing_name(name: &str) -> Option<(&str, &str)> {
    let trimmed = name.trim();
    let mut parts = trimmed.split(" and ");
    let left = parts.next()?.trim();
    let right = parts.next()?.trim();
    if parts.next().is_some() || left.is_empty() || right.is_empty() || left.contains(',') {
        return None;
    }
    Some((left, right))
}

/// Whether a name run is one of the record's listed ingredients, token for
/// token, ignoring letter case (element lines mix "DARZALEX Daratumumab
/// DARATUMUMAB" spellings of one name).
fn run_is_listed_ingredient(name: &str, ingredients: &[String]) -> bool {
    let run = identity_tokens(name);
    !run.is_empty()
        && ingredients
            .iter()
            .any(|ingredient| tokens_equal_ignore_case(&identity_tokens(ingredient), &run))
}

/// Whether a name run is a qualified form of one of the record's listed
/// ingredients: the record lists a qualified multi-word stem — the enzyme
/// spreaders carry qualifier words ("hyaluronidase (human recombinant)",
/// "berahyaluronidase alfa") — and the run extends that stem with tokens
/// the record does not list, a proper-name suffix ("-fihj", "-pmph"). A
/// single-word stem never qualifies that way ("relatlimab-rmbw" is its own
/// established ingredient, so Opdualag stays whole), and a run with fewer
/// words than the stem is a truncated salt ("tipiracil" against
/// "tipiracil hydrochloride", so Lonsurf stays whole).
fn run_qualifies_listed_ingredient(name: &str, ingredients: &[String]) -> bool {
    let run = identity_tokens(name);
    if run.is_empty() {
        return false;
    }
    ingredients.iter().any(|ingredient| {
        let listed = identity_tokens(ingredient);
        if listed.len() < 2 || run.len() <= listed.len() {
            return false;
        }
        let run_set: std::collections::HashSet<String> =
            run.iter().map(|token| token.to_ascii_lowercase()).collect();
        listed
            .iter()
            .all(|token| run_set.contains(&token.to_ascii_lowercase()))
    })
}

/// The plain ingredient of an under-the-skin pairing, derived from the
/// record's own ingredient list: subcutaneous antibody
/// products carry one "ingredient and enzyme" generic name, the record
/// lists the plain ingredient exactly, and the paired half is only a
/// qualified form of another listed ingredient — so
/// "daratumumab and hyaluronidase-fihj (human recombinant)" over
/// [daratumumab, hyaluronidase (human recombinant)] names daratumumab, and
/// "pembrolizumab and berahyaluronidase alfa-pmph" over [pembrolizumab,
/// berahyaluronidase alfa] names pembrolizumab. A name whose halves the
/// record lists as two established ingredients is a true two-drug
/// combination ("nivolumab and relatlimab-rmbw" over [nivolumab,
/// relatlimab-rmbw] names no single plain ingredient, so the whole name
/// stands and a combination never becomes another product's card name.
/// A comma-list combination names none, and a record that lists neither
/// half supports no split at all.
pub(super) fn plain_paired_ingredient_name<'a>(name: &'a str, ingredients: &[String]) -> &'a str {
    let Some((left, right)) = split_pairing_name(name) else {
        return name.trim();
    };
    let left_member = run_is_listed_ingredient(left, ingredients);
    let right_member = run_is_listed_ingredient(right, ingredients);
    if left_member == right_member {
        return name.trim();
    }
    let (plain, paired) = if left_member {
        (left, right)
    } else {
        (right, left)
    };
    if run_qualifies_listed_ingredient(paired, ingredients) {
        plain
    } else {
        name.trim()
    }
}

/// The rank tiers of one label result for the requested drug. The request decides between the two sides of the rule: a brand
/// query takes the record whose own brand name equals it, and an ingredient
/// query takes the plain single-ingredient product's record, never a
/// combination, biosimilar or under-the-skin form. A brand whose labels
/// name a family of the brand's products ("Motrin IB" for Motrin) takes
/// that family's plain-ingredient record before any other plain record
///and brand equality reads the whole word set, because
/// labelers write the same brand's words in different orders ("TYLENOL
/// Extra Strength" for "Extra Strength Tylenol").
const LABEL_RANK_BRAND_MATCH: u8 = 4;
const LABEL_RANK_BRAND_FAMILY: u8 = 3;
const LABEL_RANK_PLAIN_INGREDIENT: u8 = 2;
const LABEL_RANK_QUALIFIED_FORM: u8 = 1;
const LABEL_RANK_OPENFDA_ORDER: u8 = 0;

/// The lowercased, sorted word tokens of a brand line, so brand equality
/// tolerates word-order and spelling-case differences between labelers.
fn brand_word_set(brand: &str) -> std::collections::BTreeSet<String> {
    identity_tokens(brand).into_iter().collect()
}

/// Whether a brand line names the requested brand: the same words, in any
/// order and any casing.
fn brand_names_requested(brand: &str, requested: &str) -> bool {
    let requested_words = brand_word_set(requested);
    !requested_words.is_empty() && brand_word_set(brand) == requested_words
}

/// The generic names one label result states for its drug: the openFDA
/// generic names, plus the established names a sparse record's element line
/// carries.
pub(super) fn label_result_generic_names(result: &serde_json::Value) -> Vec<String> {
    let mut names = extract_openfda_values_from_result(result, "generic_name");
    if is_sparse_identity_record(result) {
        names.extend(sparse_element_identity_names(result).1);
    }
    names
}

/// The ingredient names one label result's own record lists: the openFDA
/// substance names, the active-ingredient names, and — for a sparse record
/// whose only identity is the element line — the established names that
/// line carries. This is the structural ingredient set the pairing split
/// reads.
pub(super) fn label_result_ingredient_names(result: &serde_json::Value) -> Vec<String> {
    let mut names = extract_openfda_values_from_result(result, "substance_name");
    if let Some(ingredients) = result.get("active_ingredients").and_then(|v| v.as_array()) {
        for ingredient in ingredients {
            if let Some(name) = ingredient.get("name").and_then(|v| v.as_str()) {
                names.push(name.to_string());
            }
        }
    }
    if is_sparse_identity_record(result) {
        names.extend(sparse_element_identity_names(result).1);
    }
    names
}

/// Whether a generic name is a qualified form of the requested or card
/// drug: the stem followed by more words (a salt, hydrate or suffix form).
fn generic_extends_stem(name: &str, requested: &str, card: &str) -> bool {
    [requested, card].into_iter().any(|stem| {
        !stem.is_empty()
            && (name.starts_with(&format!("{stem} ")) || name.starts_with(&format!("{stem}-")))
    })
}

/// Whether a label record's generic line names the card's drug as the
/// card's own product, so the record's brands list on the card. The
/// card's own line matches whatever separators each side writes ("A, B, C"
/// against "A, B and C"); a plain ingredient also matches its qualified
/// forms in either direction — a salt or hydrate ("erlotinib
/// hydrochloride" for erlotinib) or a biosimilar suffix
/// ("trastuzumab-dkst" for trastuzumab). Everything else is another
/// product's line: a second active ingredient makes a combination, and a
/// conjugate or prefixed derivative ("fam-trastuzumab deruxtecan-nxki")
/// names its own drug, never the plain card's.
fn label_generic_names_card(generic: &str, card: &str) -> bool {
    let generic = generic.trim();
    let card = card.trim();
    if generic.is_empty() || card.is_empty() {
        return false;
    }
    let generic_words = ingredient_word_set(generic);
    if generic_words == ingredient_word_set(card) {
        return true;
    }
    if is_combination_product_name(generic) || is_combination_product_name(card) {
        return false;
    }
    let generic = generic.to_ascii_lowercase();
    let card = card.to_ascii_lowercase();
    generic.starts_with(&format!("{card} "))
        || generic.starts_with(&format!("{card}-"))
        || card.starts_with(&format!("{generic} "))
        || card.starts_with(&format!("{generic}-"))
}

/// The ingredient words one generic line names: tokens split at punctuation
/// and the combination joiner, so "A, B and C" and "A, B, C" compare by
/// the ingredients they name rather than by their separators.
pub(super) fn ingredient_word_set(name: &str) -> std::collections::BTreeSet<String> {
    identity_tokens(name)
        .into_iter()
        .filter(|token| token != "and")
        .collect()
}

/// Whether a brand-equal label's generic name fits the card:
/// a brand alone never confirms a record, because the brand-equal label may
/// be another product's — a shelf name on a card it shares no ingredient
/// with. The generic fits when it names the requested or card drug (equal,
/// a qualified form, or the plain side of a pairing the record's own
/// ingredient list supports), or when its ingredient words meet the card's
/// own — either side containing the other, because a sparse record's
/// element line spells a combination's ingredients differently than the
/// card's NDC line does.
fn generic_name_fits_card(
    generic: &str,
    requested: &str,
    card: &str,
    ingredients: &[String],
) -> bool {
    let plain = plain_paired_ingredient_name(generic, ingredients)
        .trim()
        .to_ascii_lowercase();
    if plain.is_empty() {
        return false;
    }
    if plain == requested || plain == card || generic_extends_stem(&plain, requested, card) {
        return true;
    }
    let generic_words = ingredient_word_set(&plain);
    let card_stems = [card.trim()];
    card_stems.into_iter().any(|stem| {
        !stem.is_empty() && {
            let stem_words = ingredient_word_set(stem);
            generic_words.is_subset(&stem_words) || stem_words.is_subset(&generic_words)
        }
    })
}

/// The brand and established-name identity a sparse record's SPL product
/// data elements carry, the only identity such a record has: each
/// per-strength entry opens "BRAND ESTABLISHED [ESTABLISHED repeats]
/// excipients…" ("DARZALEX Daratumumab DARATUMUMAB DARATUMUMAB ACETIC ACID…"
/// names the DARZALEX brand and the daratumumab ingredient; "Herceptin
/// Hylecta Trastuzumab and hyaluronidase-oysk TRASTUZUMAB …" names the
/// HERCEPTIN HYLECTA brand and the trastuzumab pairing), so
/// the entry head's leading tokens are the brand and its recurring name
/// run after them is the established name. An entry that names no
/// established ingredient contributes no brand either.
fn sparse_element_identity_names(result: &serde_json::Value) -> (Vec<String>, Vec<String>) {
    let mut brands = Vec::new();
    let mut generics = Vec::new();
    for (brand, established) in spl_product_data_elements(result)
        .iter()
        .flat_map(|element| spl_element_identity_heads(element))
    {
        if brand.is_empty() || established.is_empty() {
            continue;
        }
        brands.push(brand.join(" "));
        let mut tokens: Vec<&String> = Vec::new();
        for token in &established {
            if !tokens.iter().any(|seen| seen.eq_ignore_ascii_case(token)) {
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
    let ingredients = label_result_ingredient_names(result);
    let element_brands = if is_sparse_identity_record(result) {
        sparse_element_identity_names(result).0
    } else {
        Vec::new()
    };
    let brand_equal = extract_openfda_values_from_result(result, "brand_name")
        .iter()
        .any(|brand| brand_names_requested(brand, &requested))
        || element_brands
            .iter()
            .any(|brand| brand_names_requested(brand, &requested));
    if brand_equal
        && label_result_generic_names(result)
            .iter()
            .any(|generic| generic_name_fits_card(generic, &requested, &card, &ingredients))
    {
        return LABEL_RANK_BRAND_MATCH;
    }
    let generics = label_result_generic_names(result)
        .iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    // The brand-family tier: a brand line that opens with the requested
    // brand ("Motrin IB" for Motrin) on a record whose generic is a plain
    // single ingredient — never a combination — names that family's own
    // product before any other plain record answers.
    let brand_opens_with_requested = |brand: &str| {
        let brand = brand.trim().to_ascii_lowercase();
        !brand.is_empty() && brand.starts_with(&requested) && brand.len() > requested.len()
    };
    if !requested.is_empty()
        && generics.iter().any(|name| {
            !is_combination_product_name(name)
                && (name == &requested
                    || name == &card
                    || generic_extends_stem(name, &requested, &card))
        })
        && (extract_openfda_values_from_result(result, "brand_name")
            .iter()
            .any(|brand| brand_opens_with_requested(brand))
            || element_brands
                .iter()
                .any(|brand| brand_opens_with_requested(brand)))
    {
        return LABEL_RANK_BRAND_FAMILY;
    }
    if !requested.is_empty()
        && generics
            .iter()
            .any(|name| !is_combination_product_name(name) && (name == &requested || name == &card))
    {
        return LABEL_RANK_PLAIN_INGREDIENT;
    }
    if !requested.is_empty()
        && generics.iter().any(|name| {
            !is_combination_product_name(name) && generic_extends_stem(name, &requested, &card)
        })
    {
        return LABEL_RANK_QUALIFIED_FORM;
    }
    LABEL_RANK_OPENFDA_ORDER
}

/// Choose the response's label record for the requested drug: the
/// highest-ranked result moves to the front and the rest keep openFDA's
/// order behind it, so every downstream reader of `results[0]` sees the
/// chosen record. Returns the reordered response with the chosen record's
/// rank.
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
/// before it. A fetch error skips the escalation; the final
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
/// sparse-metadata elements fallback.
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
    record_brand_spellings: &[String],
) -> Result<LabelLookup, BioMcpError> {
    let mut fallback: Option<serde_json::Value> = None;
    let mut fallback_rank = LABEL_RANK_OPENFDA_ORDER;
    let mut queries: Vec<&str> = Vec::new();
    // The label lookup follows the resolved ingredient — the
    // card's own name — as well as the request, and it follows the brand
    // as the resolved record itself spells it: labelers write one brand's
    // words in different orders ("TYLENOL Extra Strength" against the
    // request "Extra Strength Tylenol"), so the record's own spelling of
    // the requested brand's words reaches the record the request's phrase
    // order never matches.
    for query in [requested_name, card_name]
        .into_iter()
        .chain(record_brand_spellings.iter().map(String::as_str))
    {
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
