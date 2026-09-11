use crate::entities::trial::TrialSearchResult;
#[cfg(test)]
use crate::entities::trial::{Trial, TrialDesign};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::CtGovStudy;

fn truncate_utf8(s: &str, max_bytes: usize, suffix: &str) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let mut out = s[..boundary].trim_end().to_string();
    out.push_str(suffix);
    out
}

fn normalize_phase(phases: &[String]) -> Option<String> {
    if phases.is_empty() {
        return None;
    }
    Some(phases.join("/"))
}

fn clean_conditions(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub(crate) fn format_conditions(conditions: &[String]) -> String {
    const MAX_ITEMS: usize = 10;
    const MAX_BYTES: usize = 80;

    let cleaned = conditions
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let joined = cleaned
        .iter()
        .take(MAX_ITEMS)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if cleaned.len() <= MAX_ITEMS && joined.len() <= MAX_BYTES {
        return joined;
    }

    let suffix = format!("… [abridged; {} conditions total]", cleaned.len());
    let prefix = truncate_utf8(&joined, MAX_BYTES.saturating_sub(suffix.len()), "");
    format!("{prefix}{suffix}")
}

#[cfg(test)]
fn clean_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
fn normalize_summary(value: Option<&str>) -> Option<String> {
    clean_opt(value)
}

#[cfg(test)]
pub fn from_ctgov_study(study: &CtGovStudy) -> Result<Trial, BioMcpError> {
    let p = study.protocol_section.as_ref();
    let id = p
        .and_then(|p| p.identification_module.as_ref())
        .and_then(|m| m.nct_id.as_deref())
        .unwrap_or_default()
        .to_string();
    let title = p
        .and_then(|p| p.identification_module.as_ref())
        .and_then(|m| m.brief_title.as_deref())
        .unwrap_or_default()
        .trim()
        .to_string();
    let status = p
        .and_then(|p| p.status_module.as_ref())
        .and_then(|m| m.overall_status.as_deref())
        .unwrap_or_default()
        .trim()
        .to_string();
    let why_stopped = ["TERMINATED", "WITHDRAWN", "SUSPENDED"]
        .iter()
        .any(|stopped| status.eq_ignore_ascii_case(stopped))
        .then(|| {
            p.and_then(|p| p.status_module.as_ref())
                .and_then(|m| clean_opt(m.why_stopped.as_deref()))
        });
    let phase = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.phases.as_ref())
        .and_then(|phases| normalize_phase(phases));
    let phases = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.phases.as_ref())
        .map(|values| values.iter().map(|value| value.trim().to_owned()).collect())
        .unwrap_or_default();
    let study_type = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.study_type.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let sponsor = p
        .and_then(|p| p.sponsor_collaborators_module.as_ref())
        .and_then(|m| m.lead_sponsor.as_ref())
        .and_then(|s| s.name.as_deref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let enrollment = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.enrollment_info.as_ref())
        .and_then(|e| e.count);
    let summary = p
        .and_then(|p| p.description_module.as_ref())
        .and_then(|m| normalize_summary(m.brief_summary.as_deref()));
    let start_date = p
        .and_then(|p| p.status_module.as_ref())
        .and_then(|m| m.start_date_struct.as_ref())
        .and_then(|d| d.date.as_deref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let completion_date = p
        .and_then(|p| p.status_module.as_ref())
        .and_then(|m| m.completion_date_struct.as_ref())
        .and_then(|d| d.date.as_deref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let conditions = p
        .and_then(|p| p.conditions_module.as_ref())
        .map(|m| clean_conditions(&m.conditions))
        .unwrap_or_default();
    Ok(Trial {
        identities: vec![crate::entities::trial::TrialIdentity {
            authority: "clinicaltrials.gov".to_owned(),
            identifier: id.clone(),
        }],
        nct_id: id,
        source: None,
        title,
        official_title: None,
        status,
        why_stopped,
        phase,
        phases,
        study_type,
        conditions,
        design: TrialDesign::default(),
        sponsor,
        enrollment: enrollment.and_then(|value| u64::try_from(value).ok()),
        summary,
        start_date,
        completion_date,
        eligibility: None,
        eligibility_provenance: None,
        site_directory: None,
        site_offset: 0,
        site_limit: None,
        outcomes: None,
        references: None,
    })
}

pub fn from_ctgov_hit(study: &CtGovStudy) -> TrialSearchResult {
    let p = study.protocol_section.as_ref();
    let nct_id = p
        .and_then(|p| p.identification_module.as_ref())
        .and_then(|m| m.nct_id.as_deref())
        .unwrap_or_default()
        .to_string();
    let title = p
        .and_then(|p| p.identification_module.as_ref())
        .and_then(|m| m.brief_title.as_deref())
        .unwrap_or_default()
        .trim()
        .to_string();
    let status = p
        .and_then(|p| p.status_module.as_ref())
        .and_then(|m| m.overall_status.as_deref())
        .unwrap_or_default()
        .trim()
        .to_string();
    let phase = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.phases.as_ref())
        .and_then(|phases| normalize_phase(phases));
    let sponsor = p
        .and_then(|p| p.sponsor_collaborators_module.as_ref())
        .and_then(|m| m.lead_sponsor.as_ref())
        .and_then(|s| s.name.as_deref())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let conditions = p
        .and_then(|p| p.conditions_module.as_ref())
        .map(|m| clean_conditions(&m.conditions))
        .unwrap_or_default();

    TrialSearchResult {
        nct_id,
        title,
        status,
        phase,
        conditions,
        sponsor,
        matched_intervention_label: None,
    }
}

fn json_get_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    let obj = value.as_object()?;
    for key in keys {
        let Some(v) = obj.get(*key) else { continue };
        match v {
            serde_json::Value::String(s) if !s.trim().is_empty() => {
                return Some(s.trim().to_string());
            }
            serde_json::Value::Number(n) => return Some(n.to_string()),
            _ => {}
        }
    }
    None
}

fn nci_conditions(value: &serde_json::Value, keys: &[&str]) -> Result<Vec<String>, BioMcpError> {
    let Some(obj) = value.as_object() else {
        return Ok(Vec::new());
    };

    for key in keys {
        let Some(value) = obj.get(*key) else {
            continue;
        };
        match value {
            serde_json::Value::Array(values) => {
                let names = values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .or_else(|| value.get("name").and_then(serde_json::Value::as_str))
                            .map(str::trim)
                            .filter(|name| !name.is_empty())
                            .map(str::to_string)
                            .ok_or_else(|| BioMcpError::Api {
                                api: "nci_cts".to_string(),
                                message: "NCI condition has no readable name".to_string(),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(names);
            }
            serde_json::Value::String(name) if !name.trim().is_empty() => {
                return Ok(vec![name.trim().to_string()]);
            }
            _ => {}
        }
    }

    Ok(Vec::new())
}

pub fn from_nci_hit(hit: &serde_json::Value) -> Result<TrialSearchResult, BioMcpError> {
    let nct_id = json_get_string(hit, &["nct_id"]).unwrap_or_default();
    let title = json_get_string(hit, &["brief_title"]).unwrap_or_default();
    let status = json_get_string(hit, &["current_trial_status"]).unwrap_or_default();
    let phase = json_get_string(hit, &["phase"]).filter(|s| !s.is_empty());
    let sponsor = json_get_string(hit, &["lead_org"]).filter(|s| !s.is_empty());
    let conditions = nci_conditions(hit, &["diseases"])?;

    Ok(TrialSearchResult {
        nct_id,
        title,
        status,
        phase,
        conditions,
        sponsor,
        matched_intervention_label: None,
    })
}

#[cfg(test)]
#[path = "trial/tests.rs"]
mod tests;
