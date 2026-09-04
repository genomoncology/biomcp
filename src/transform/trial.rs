use std::borrow::Cow;
use std::collections::HashSet;

use crate::entities::trial::{
    Trial, TrialArm, TrialContact, TrialEligibility, TrialIntervention, TrialLocation,
    TrialOutcome, TrialOutcomes, TrialReference, TrialSearchResult,
};
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

fn first_n_sentences(text: &str, n: usize) -> Cow<'_, str> {
    let trimmed = text.trim();
    if trimmed.is_empty() || n == 0 {
        return Cow::Borrowed("");
    }

    let mut end = 0;
    let mut count = 0;
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'.' {
            // Sentence boundary: '.' followed by whitespace or end-of-string.
            let next = i + 1;
            if next == bytes.len() || bytes.get(next).is_some_and(|b| b.is_ascii_whitespace()) {
                count += 1;
                if count >= n {
                    end = next;
                    break;
                }
            }
        }
        i += 1;
    }

    if end == 0 {
        Cow::Borrowed(trimmed)
    } else {
        Cow::Borrowed(&trimmed[..end])
    }
}

fn normalize_phase(phases: &[String]) -> Option<String> {
    if phases.is_empty() {
        return None;
    }
    Some(phases.join("/"))
}

fn clean_list(values: &[String], max: usize) -> Vec<String> {
    values
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .take(max)
        .map(|s| s.to_string())
        .collect()
}

fn normalize_age(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .filter(|v| !v.eq_ignore_ascii_case("n/a"))
        .map(str::to_string)
}

fn format_age_range(min_age: Option<&str>, max_age: Option<&str>) -> Option<String> {
    let min_age = normalize_age(min_age);
    let max_age = normalize_age(max_age);
    match (min_age, max_age) {
        (Some(min), Some(max)) => Some(format!("{min} to {max}")),
        (Some(min), None) => Some(format!("{min} to Any age")),
        (None, Some(max)) => Some(format!("Any age to {max}")),
        (None, None) => None,
    }
}

pub(crate) fn truncate_summary(s: &str) -> String {
    let short = first_n_sentences(s, 2);
    truncate_utf8(short.trim(), 500, "...")
}

pub(crate) fn format_conditions(conditions: &[String]) -> String {
    let joined = conditions
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .take(10)
        .collect::<Vec<_>>()
        .join(", ");
    truncate_utf8(&joined, 80, "…")
}

fn clean_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn extract_locations(study: &CtGovStudy) -> Option<Vec<TrialLocation>> {
    let locations = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.contacts_locations_module.as_ref())
        .map(|m| &m.locations)?;

    let mut out = locations
        .iter()
        .filter_map(|loc| {
            let facility = clean_opt(loc.facility.as_deref())?;
            let city = clean_opt(loc.city.as_deref())?;
            let country = clean_opt(loc.country.as_deref())?;
            let contact = loc
                .contacts
                .first()
                .or_else(|| loc.central_contacts.first());
            Some(TrialLocation {
                facility,
                city,
                state: clean_opt(loc.state.as_deref()),
                country,
                status: clean_opt(loc.status.as_deref()),
                contact_name: contact.and_then(|c| clean_opt(c.name.as_deref())),
                contact_role: contact.and_then(|c| clean_opt(c.role.as_deref())),
                contact_phone: contact.and_then(|c| clean_opt(c.phone.as_deref())),
                contact_email: contact.and_then(|c| clean_opt(c.email.as_deref())),
                latitude: loc.geo_point.as_ref().and_then(|geo| geo.lat),
                longitude: loc.geo_point.as_ref().and_then(|geo| geo.lon),
            })
        })
        .collect::<Vec<_>>();

    out.sort_by(|a, b| {
        let a_recruiting = a
            .status
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("RECRUITING"));
        let b_recruiting = b
            .status
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("RECRUITING"));
        b_recruiting.cmp(&a_recruiting)
    });

    (!out.is_empty()).then_some(out)
}

fn extract_contact(
    level: &str,
    contact: &crate::sources::clinicaltrials::CtGovContact,
    facility: Option<&str>,
    city: Option<&str>,
    state: Option<&str>,
    country: Option<&str>,
) -> Option<TrialContact> {
    Some(TrialContact {
        level: level.to_string(),
        name: clean_opt(contact.name.as_deref())?,
        role: clean_opt(contact.role.as_deref()),
        phone: clean_opt(contact.phone.as_deref()),
        email: clean_opt(contact.email.as_deref()),
        facility: clean_opt(facility),
        city: clean_opt(city),
        state: clean_opt(state),
        country: clean_opt(country),
    })
}

fn extract_contacts(study: &CtGovStudy) -> Option<Vec<TrialContact>> {
    let module = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.contacts_locations_module.as_ref())?;
    let mut out = module
        .central_contacts
        .iter()
        .filter_map(|contact| extract_contact("central", contact, None, None, None, None))
        .collect::<Vec<_>>();

    for loc in &module.locations {
        for contact in loc.contacts.iter().chain(loc.central_contacts.iter()) {
            if let Some(contact) = extract_contact(
                "site",
                contact,
                loc.facility.as_deref(),
                loc.city.as_deref(),
                loc.state.as_deref(),
                loc.country.as_deref(),
            ) {
                out.push(contact);
            }
        }
    }

    (!out.is_empty()).then_some(out)
}

fn format_sex(value: Option<&str>) -> Option<String> {
    clean_opt(value).map(|sex| match sex.to_ascii_lowercase().as_str() {
        "female" | "f" => "Female".to_string(),
        "male" | "m" => "Male".to_string(),
        "all" => "All".to_string(),
        _ => sex,
    })
}

fn extract_eligibility(study: &CtGovStudy) -> Option<TrialEligibility> {
    let module = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.eligibility_module.as_ref())?;
    let eligibility = TrialEligibility {
        sex: format_sex(module.sex.as_deref()),
        minimum_age: normalize_age(module.minimum_age.as_deref()),
        maximum_age: normalize_age(module.maximum_age.as_deref()),
    };
    (eligibility.sex.is_some()
        || eligibility.minimum_age.is_some()
        || eligibility.maximum_age.is_some())
    .then_some(eligibility)
}

fn extract_outcomes(study: &CtGovStudy) -> Option<TrialOutcomes> {
    let module = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.outcomes_module.as_ref())?;

    let primary = module
        .primary_outcomes
        .iter()
        .filter_map(|row| {
            let measure = clean_opt(row.measure.as_deref())?;
            Some(TrialOutcome {
                measure,
                description: clean_opt(row.description.as_deref()),
                time_frame: clean_opt(row.time_frame.as_deref()),
            })
        })
        .collect::<Vec<_>>();

    let secondary = module
        .secondary_outcomes
        .iter()
        .filter_map(|row| {
            let measure = clean_opt(row.measure.as_deref())?;
            Some(TrialOutcome {
                measure,
                description: clean_opt(row.description.as_deref()),
                time_frame: clean_opt(row.time_frame.as_deref()),
            })
        })
        .collect::<Vec<_>>();

    if primary.is_empty() && secondary.is_empty() {
        None
    } else {
        Some(TrialOutcomes { primary, secondary })
    }
}

fn extract_arms(study: &CtGovStudy) -> Option<Vec<TrialArm>> {
    let module = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.arms_interventions_module.as_ref())?;

    let out = module
        .arm_groups
        .iter()
        .filter_map(|arm| {
            let label = clean_opt(arm.label.as_deref())?;
            Some(TrialArm {
                label: label.clone(),
                arm_type: clean_opt(arm.arm_group_type.as_deref()),
                description: clean_opt(arm.description.as_deref()),
                interventions: if arm.intervention_names.is_empty() {
                    module
                        .interventions
                        .iter()
                        .filter(|i| i.arm_group_labels.iter().any(|v| v == &label))
                        .filter_map(|i| clean_opt(i.name.as_deref()))
                        .collect::<Vec<_>>()
                } else {
                    clean_list(&arm.intervention_names, 25)
                },
            })
        })
        .collect::<Vec<_>>();

    (!out.is_empty()).then_some(out)
}

fn extract_references(study: &CtGovStudy) -> Option<Vec<TrialReference>> {
    let refs = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.references_module.as_ref())
        .map(|m| &m.references)?;

    let out = refs
        .iter()
        .filter_map(|r| {
            Some(TrialReference {
                pmid: clean_opt(r.pmid.as_deref()),
                citation: clean_opt(r.citation.as_deref())?,
                reference_type: clean_opt(r.reference_type.as_deref()),
            })
        })
        .collect::<Vec<_>>();

    (!out.is_empty()).then_some(out)
}

pub fn from_ctgov_study(study: &CtGovStudy) -> Trial {
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
    let study_type = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.study_type.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let age_range = p
        .and_then(|p| p.eligibility_module.as_ref())
        .and_then(|m| format_age_range(m.minimum_age.as_deref(), m.maximum_age.as_deref()));
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
        .and_then(|m| m.brief_summary.as_deref())
        .map(truncate_summary)
        .filter(|s| !s.is_empty());
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
        .map(|m| clean_list(&m.conditions, 25))
        .unwrap_or_default();
    let interventions = p
        .and_then(|p| p.arms_interventions_module.as_ref())
        .map(|m| {
            m.interventions
                .iter()
                .filter_map(|i| i.name.as_deref())
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .take(25)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let intervention_details = p
        .and_then(|p| p.arms_interventions_module.as_ref())
        .map(|m| {
            m.interventions
                .iter()
                .filter_map(|i| {
                    let name = clean_opt(i.name.as_deref())?;
                    let mut seen = HashSet::new();
                    let other_names = i
                        .other_names
                        .iter()
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .filter(|s| !s.eq_ignore_ascii_case(&name))
                        .filter(|s| seen.insert(s.to_ascii_lowercase()))
                        .take(25)
                        .map(|s| s.to_string())
                        .collect();
                    Some(TrialIntervention {
                        name,
                        intervention_type: clean_opt(i.intervention_type.as_deref()),
                        description: clean_opt(i.description.as_deref()),
                        other_names,
                    })
                })
                .take(25)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Trial {
        nct_id: id,
        source: None,
        title,
        status,
        why_stopped,
        phase,
        study_type,
        age_range,
        conditions,
        interventions,
        intervention_details,
        sponsor,
        enrollment,
        summary,
        start_date,
        completion_date,
        eligibility_text: None,
        eligibility: extract_eligibility(study),
        eligibility_provenance: None,
        contacts: extract_contacts(study),
        locations: extract_locations(study),
        outcomes: extract_outcomes(study),
        arms: extract_arms(study),
        references: extract_references(study),
    }
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
        .map(|m| clean_list(&m.conditions, 10))
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

fn nci_conditions(
    value: &serde_json::Value,
    keys: &[&str],
    max: usize,
) -> Result<Vec<String>, BioMcpError> {
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
                return Ok(names.into_iter().take(max).collect());
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
    let nct_id = json_get_string(hit, &["nct_id", "nctId", "nctID"]).unwrap_or_default();
    let title = json_get_string(hit, &["brief_title", "briefTitle", "title"]).unwrap_or_default();
    let status = json_get_string(hit, &["current_trial_status", "status", "overallStatus"])
        .unwrap_or_default();
    let phase =
        json_get_string(hit, &["phase", "phase_code", "phaseCode"]).filter(|s| !s.is_empty());
    let sponsor = json_get_string(
        hit,
        &["lead_org", "lead_organization", "leadSponsor", "sponsor"],
    )
    .filter(|s| !s.is_empty());
    let conditions = nci_conditions(hit, &["diseases", "conditions"], 10)?;

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

pub fn from_nci_trial(trial: &serde_json::Value) -> Result<Trial, BioMcpError> {
    let nct_id = json_get_string(trial, &["nct_id", "nctId", "nctID"]).unwrap_or_default();
    let title = json_get_string(trial, &["brief_title", "briefTitle", "title"]).unwrap_or_default();
    let status = json_get_string(trial, &["current_trial_status", "status", "overallStatus"])
        .unwrap_or_default();
    let phase =
        json_get_string(trial, &["phase", "phase_code", "phaseCode"]).filter(|s| !s.is_empty());
    let study_type = json_get_string(trial, &["study_protocol_type"]);
    let structured_eligibility = trial
        .get("eligibility")
        .and_then(|value| value.get("structured"));
    let min_age = structured_eligibility.and_then(|value| json_get_string(value, &["min_age"]));
    let max_age = structured_eligibility
        .and_then(|value| json_get_string(value, &["max_age"]))
        .filter(|age| !age.eq_ignore_ascii_case("999 Years"));
    let age_range = format_age_range(min_age.as_deref(), max_age.as_deref());
    let sponsor = json_get_string(
        trial,
        &["lead_org", "lead_organization", "leadSponsor", "sponsor"],
    )
    .filter(|s| !s.is_empty());
    let enrollment = json_get_string(trial, &["minimum_target_accrual_number"])
        .and_then(|s| s.parse::<i32>().ok());
    let start_date = json_get_string(trial, &["start_date", "startDate"]).filter(|s| !s.is_empty());
    let completion_date =
        json_get_string(trial, &["completion_date", "completionDate"]).filter(|s| !s.is_empty());
    let summary = json_get_string(trial, &["brief_summary", "briefSummary", "summary"])
        .map(|s| truncate_summary(&s))
        .filter(|s| !s.is_empty());
    let conditions = nci_conditions(trial, &["diseases", "conditions"], 25)?;
    let interventions = trial
        .get("arms")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|arm| {
            arm.get("interventions")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|intervention| json_get_string(intervention, &["name"]))
        .take(25)
        .collect();
    let why_stopped = json_get_string(trial, &["why_study_stopped"]).map(Some);

    Ok(Trial {
        nct_id,
        source: None,
        title,
        status,
        why_stopped,
        phase,
        study_type,
        age_range,
        conditions,
        interventions,
        intervention_details: Vec::new(),
        sponsor,
        enrollment,
        summary,
        start_date,
        completion_date,
        eligibility_text: None,
        eligibility: None,
        eligibility_provenance: None,
        contacts: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    })
}

#[cfg(test)]
#[path = "trial/tests.rs"]
mod tests;
