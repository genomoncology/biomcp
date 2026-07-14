//! Trial detail retrieval exposed through the stable trial facade.

use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::nci_cts::NciCtsClient;
use crate::transform;

use super::{
    TRIAL_SECTION_ALL, TRIAL_SECTION_ARMS, TRIAL_SECTION_CONTACTS, TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS, TRIAL_SECTION_NAMES, TRIAL_SECTION_OUTCOMES, TRIAL_SECTION_REFERENCES,
    Trial, TrialSource,
};

const ELIGIBILITY_MAX_CHARS: usize = 12_000;

#[derive(Debug, Clone, Copy, Default)]
struct TrialSections {
    include_eligibility: bool,
    include_eligibility_provenance: bool,
    include_contacts: bool,
    include_locations: bool,
    include_outcomes: bool,
    include_arms: bool,
    include_references: bool,
}

fn parse_sections(sections: &[String]) -> Result<TrialSections, BioMcpError> {
    let mut out = TrialSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            TRIAL_SECTION_ELIGIBILITY => {
                out.include_eligibility = true;
                out.include_eligibility_provenance = true;
            }
            TRIAL_SECTION_CONTACTS => out.include_contacts = true,
            TRIAL_SECTION_LOCATIONS => out.include_locations = true,
            TRIAL_SECTION_OUTCOMES => out.include_outcomes = true,
            TRIAL_SECTION_ARMS => out.include_arms = true,
            TRIAL_SECTION_REFERENCES => out.include_references = true,
            TRIAL_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for trial. Available: {}",
                    TRIAL_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_eligibility = true;
        out.include_contacts = true;
        out.include_locations = true;
        out.include_outcomes = true;
        out.include_arms = true;
        out.include_references = true;
    }

    Ok(out)
}

fn truncate_inline_text(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let truncated = value.chars().take(max_chars).collect::<String>();
    format!("{truncated}\n\n(truncated, {count} chars total)")
}

fn looks_like_nct_id(value: &str) -> bool {
    let v = value.trim().as_bytes();
    if v.len() != 11 {
        return false;
    }
    if &v[0..3] != b"NCT" {
        return false;
    }
    v[3..].iter().all(|b| b.is_ascii_digit())
}

fn normalize_nct_id(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(prefix) = trimmed.get(..3)
        && prefix.eq_ignore_ascii_case("NCT")
    {
        return format!("NCT{}", &trimmed[3..]);
    }
    trimmed.to_string()
}

pub(super) fn validated_nct_id(value: &str) -> Result<String, BioMcpError> {
    let nct_id = normalize_nct_id(value);
    let nct_id = nct_id.trim();
    if nct_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "NCT ID is required. Example: biomcp get trial NCT02576665".into(),
        ));
    }
    if nct_id.len() > 64 {
        return Err(BioMcpError::InvalidArgument("NCT ID is too long.".into()));
    }
    if !looks_like_nct_id(nct_id) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Expected an NCT ID like NCT02576665 (got '{nct_id}')"
        )));
    }
    Ok(nct_id.to_string())
}

pub async fn get(
    nct_id: &str,
    sections: &[String],
    source: TrialSource,
) -> Result<Trial, BioMcpError> {
    let nct_id = validated_nct_id(nct_id)?;
    let nct_id = nct_id.as_str();
    let section_flags = parse_sections(sections)?;

    match source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            let study = client.get(nct_id, sections).await?;
            let mut trial = transform::trial::from_ctgov_study(&study);
            trial.source = Some("ClinicalTrials.gov".into());
            if !section_flags.include_contacts {
                trial.contacts = None;
            }
            if !section_flags.include_eligibility {
                trial.eligibility = None;
            }
            if !section_flags.include_locations {
                trial.locations = None;
            }

            if section_flags.include_eligibility {
                if section_flags.include_eligibility_provenance {
                    trial.eligibility_provenance =
                        Some(super::documents::eligibility_provenance(nct_id, &study));
                }
                let criteria = study
                    .protocol_section
                    .as_ref()
                    .and_then(|p| p.eligibility_module.as_ref())
                    .and_then(|m| m.eligibility_criteria.as_deref())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());

                if let Some(criteria) = criteria {
                    trial.eligibility_text =
                        Some(truncate_inline_text(criteria, ELIGIBILITY_MAX_CHARS));
                }
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
        TrialSource::NciCts => {
            let client = NciCtsClient::new()?;
            let resp = client.get(nct_id).await?;
            let mut trial = transform::trial::from_nci_trial(&resp);
            trial.source = Some("NCI CTS".into());

            if section_flags.include_eligibility {
                // Best-effort: look for eligibility in common fields.
                let criteria = resp
                    .get("eligibility")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if let Some(criteria) = criteria {
                    trial.eligibility_text =
                        Some(truncate_inline_text(criteria, ELIGIBILITY_MAX_CHARS));
                } else {
                    warn!(nct_id, "NCI CTS eligibility criteria not found in response");
                }
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
    }
}

#[cfg(test)]
mod tests;
