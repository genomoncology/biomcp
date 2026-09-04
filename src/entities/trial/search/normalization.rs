//! Trial search normalization helpers shared across search backends.

#[cfg(test)]
use regex::Regex;
#[cfg(test)]
use std::sync::OnceLock;

use crate::error::BioMcpError;

use super::super::{TrialSearchFilters, TrialSearchResult};

fn normalize_enum_key(value: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_sep = false;
            continue;
        }
        if matches!(ch, ' ' | ',' | '-' | '_') && !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn invalid_status_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --status value '{raw}'. Expected one of: \
NOT_YET_RECRUITING, RECRUITING, ENROLLING_BY_INVITATION, ACTIVE_NOT_RECRUITING, \
COMPLETED, SUSPENDED, TERMINATED, WITHDRAWN. Aliases: comma/space forms."
    ))
}

fn ambiguous_active_status_error() -> BioMcpError {
    BioMcpError::InvalidArgument(
        "--status active is ambiguous: NCI uses \"active\" for a trial that is open and \
accruing, ClinicalTrials.gov uses it for one that has stopped accruing. Use --status \
recruiting for open and accruing, or --status active_not_recruiting for enrolled and no \
longer accruing."
            .into(),
    )
}

fn normalize_single_status(value: &str) -> Result<&'static str, BioMcpError> {
    let key = normalize_enum_key(value);
    match key.as_str() {
        "NOT_YET_RECRUITING" => Ok("NOT_YET_RECRUITING"),
        "RECRUITING" => Ok("RECRUITING"),
        "ENROLLING_BY_INVITATION" | "ENROLLING" => Ok("ENROLLING_BY_INVITATION"),
        "ACTIVE_NOT_RECRUITING" => Ok("ACTIVE_NOT_RECRUITING"),
        "ACTIVE" => Err(ambiguous_active_status_error()),
        "COMPLETED" | "COMPLETE" => Ok("COMPLETED"),
        "SUSPENDED" => Ok("SUSPENDED"),
        "TERMINATED" => Ok("TERMINATED"),
        "WITHDRAWN" => Ok("WITHDRAWN"),
        _ => Err(invalid_status_error(value)),
    }
}

fn normalize_status(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--status must not be empty".into(),
        ));
    }

    // Preserve existing single-value aliases, including
    // "active, not recruiting".
    let single_error = match normalize_single_status(raw) {
        Ok(single) => return Ok(single.to_string()),
        Err(error) => error,
    };

    let parts = raw
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return Err(single_error);
    }

    let mut normalized = Vec::with_capacity(parts.len());
    for part in parts {
        normalized.push(normalize_single_status(part)?.to_string());
    }
    Ok(normalized.join(","))
}

fn status_priority(value: &str) -> u8 {
    match normalize_enum_key(value).as_str() {
        "RECRUITING" => 0,
        "ACTIVE_NOT_RECRUITING" => 1,
        "ENROLLING_BY_INVITATION" => 2,
        "NOT_YET_RECRUITING" => 3,
        "COMPLETED" => 4,
        "UNKNOWN" => 5,
        "WITHDRAWN" => 6,
        "TERMINATED" => 7,
        "SUSPENDED" => 8,
        _ => 9,
    }
}

pub(super) fn sort_trials_by_status_priority(rows: &mut [TrialSearchResult]) {
    rows.sort_by(|a, b| {
        status_priority(&a.status)
            .cmp(&status_priority(&b.status))
            .then_with(|| a.nct_id.cmp(&b.nct_id))
    });
}

fn invalid_phase_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --phase value '{raw}'. Expected one of: NA, EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4, PHASE1/PHASE2, PHASE2/PHASE3. \
Aliases: 1-4, I-IV, 1/2, 2/3, I_II, II_III, early_phase1, early1, n/a."
    ))
}

fn invalid_sex_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --sex value '{raw}'. Expected one of: female, male, all."
    ))
}

pub(super) fn normalize_sex(value: &str) -> Result<Option<&'static str>, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--sex must not be empty".into(),
        ));
    }
    match normalize_enum_key(raw).as_str() {
        "FEMALE" | "F" => Ok(Some("f")),
        "MALE" | "M" => Ok(Some("m")),
        "ALL" | "ANY" | "BOTH" => Ok(None),
        _ => Err(invalid_sex_error(raw)),
    }
}

fn invalid_sponsor_type_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --sponsor-type value '{raw}'. Expected one of: nih, industry, fed, other."
    ))
}

pub(super) fn normalize_sponsor_type(value: &str) -> Result<&'static str, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--sponsor-type must not be empty".into(),
        ));
    }
    match normalize_enum_key(raw).as_str() {
        "NIH" => Ok("nih"),
        "INDUSTRY" => Ok("industry"),
        "FED" | "FEDERAL" => Ok("fed"),
        "OTHER" => Ok("other"),
        _ => Err(invalid_sponsor_type_error(raw)),
    }
}

fn normalize_phase(value: &str) -> Result<Vec<String>, BioMcpError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--phase must not be empty".into(),
        ));
    }

    let upper = v.to_ascii_uppercase();
    let slash_normalized = upper
        .split('/')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("/");
    match slash_normalized.as_str() {
        "NA" | "N/A" => Ok(vec!["NA".to_string()]),
        "EARLY_PHASE1" | "EARLY1" => Ok(vec!["EARLY_PHASE1".to_string()]),
        "PHASE1" | "1" | "I" => Ok(vec!["PHASE1".to_string()]),
        "PHASE2" | "2" | "II" => Ok(vec!["PHASE2".to_string()]),
        "PHASE3" | "3" | "III" => Ok(vec!["PHASE3".to_string()]),
        "PHASE4" | "4" | "IV" => Ok(vec!["PHASE4".to_string()]),
        "PHASE1/PHASE2" | "1/2" | "I_II" => Ok(vec!["PHASE1".to_string(), "PHASE2".to_string()]),
        "PHASE2/PHASE3" | "2/3" | "II_III" => Ok(vec!["PHASE2".to_string(), "PHASE3".to_string()]),
        _ => Err(invalid_phase_error(v)),
    }
}

pub(super) fn normalized_status_filter(
    filters: &TrialSearchFilters,
) -> Result<Option<String>, BioMcpError> {
    filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(normalize_status)
        .transpose()
}

pub(super) fn normalized_phase_filter(
    filters: &TrialSearchFilters,
) -> Result<Option<Vec<String>>, BioMcpError> {
    filters
        .phase
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(normalize_phase)
        .transpose()
}

#[cfg(test)]
pub(super) fn normalize_intervention_query(value: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"([A-Za-z]{2,})\s+(\d{2,})").expect("valid regex"));
    re.replace_all(value.trim(), "$1-$2").into_owned()
}

pub(super) fn normalized_facility_filter(filters: &TrialSearchFilters) -> Option<String> {
    filters
        .facility
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests;
