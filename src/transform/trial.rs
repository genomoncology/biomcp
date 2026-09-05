use std::collections::HashSet;

use crate::entities::trial::{
    Trial, TrialAge, TrialArm, TrialContact, TrialEligibility, TrialIntervention, TrialLocation,
    TrialOutcome, TrialOutcomes, TrialReference, TrialSearchResult, TrialSiteContact,
    format_age_range,
};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::{CtGovContact, CtGovLocation, CtGovStudy};

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

fn clean_list(values: &[String], max: usize) -> Vec<String> {
    values
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .take(max)
        .map(|s| s.to_string())
        .collect()
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

fn clean_opt(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn normalize_summary(value: Option<&str>) -> Option<String> {
    clean_opt(value)
}

fn is_meaningful_site(location: &CtGovLocation) -> bool {
    [
        location.facility.as_deref(),
        location.city.as_deref(),
        location.state.as_deref(),
        location.zip.as_deref(),
        location.country.as_deref(),
    ]
    .into_iter()
    .any(|value| clean_opt(value).is_some())
        || location.geo_point.as_ref().is_some_and(|point| {
            point.lat.is_some_and(f64::is_finite) || point.lon.is_some_and(f64::is_finite)
        })
        || location
            .contacts
            .iter()
            .any(|contact| clean_opt(contact.name.as_deref()).is_some())
}

fn clean_site_contact(contact: &CtGovContact) -> Option<TrialSiteContact> {
    Some(TrialSiteContact {
        name: clean_opt(contact.name.as_deref())?,
        role: clean_opt(contact.role.as_deref()),
        phone: clean_opt(contact.phone.as_deref()),
        email: clean_opt(contact.email.as_deref()),
    })
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
            if !is_meaningful_site(loc) {
                return None;
            }
            let contact = loc.contacts.first();
            Some(TrialLocation {
                facility: clean_opt(loc.facility.as_deref()),
                city: clean_opt(loc.city.as_deref()),
                state: clean_opt(loc.state.as_deref()),
                postal_code: clean_opt(loc.zip.as_deref()),
                country: clean_opt(loc.country.as_deref()),
                status: clean_opt(loc.status.as_deref()),
                contacts: loc.contacts.iter().filter_map(clean_site_contact).collect(),
                contact_name: contact.and_then(|c| clean_opt(c.name.as_deref())),
                contact_role: contact.and_then(|c| clean_opt(c.role.as_deref())),
                contact_phone: contact.and_then(|c| clean_opt(c.phone.as_deref())),
                contact_email: contact.and_then(|c| clean_opt(c.email.as_deref())),
                latitude: loc
                    .geo_point
                    .as_ref()
                    .and_then(|geo| geo.lat)
                    .filter(|value| value.is_finite()),
                longitude: loc
                    .geo_point
                    .as_ref()
                    .and_then(|geo| geo.lon)
                    .filter(|value| value.is_finite()),
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
    contact: &CtGovContact,
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
        if !is_meaningful_site(loc) {
            continue;
        }
        for contact in &loc.contacts {
            if let Some(site_contact) = clean_site_contact(contact) {
                out.push(TrialContact {
                    level: "site".to_string(),
                    name: site_contact.name,
                    role: site_contact.role,
                    phone: site_contact.phone,
                    email: site_contact.email,
                    facility: clean_opt(loc.facility.as_deref()),
                    city: clean_opt(loc.city.as_deref()),
                    state: clean_opt(loc.state.as_deref()),
                    country: clean_opt(loc.country.as_deref()),
                });
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
        minimum_age: module
            .minimum_age
            .as_ref()
            .and_then(|age| age.parsed().cloned()),
        maximum_age: module
            .maximum_age
            .as_ref()
            .and_then(|age| age.parsed().cloned()),
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

fn extract_references(study: &CtGovStudy) -> Result<Option<Vec<TrialReference>>, BioMcpError> {
    let Some(refs) = study
        .protocol_section
        .as_ref()
        .and_then(|p| p.references_module.as_ref())
        .map(|m| &m.references)
    else {
        return Ok(None);
    };

    let out = refs
        .iter()
        .filter_map(|r| {
            let citation = clean_opt(r.citation.as_deref())?;
            Some(TrialReference::new(
                r.pmid.clone(),
                citation,
                r.reference_type.clone(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((!out.is_empty()).then_some(out))
}

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
    let study_type = p
        .and_then(|p| p.design_module.as_ref())
        .and_then(|m| m.study_type.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let age_range = p
        .and_then(|p| p.eligibility_module.as_ref())
        .and_then(|module| {
            format_age_range(
                module.minimum_age.as_ref().and_then(|age| age.parsed()),
                module.maximum_age.as_ref().and_then(|age| age.parsed()),
            )
        });
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

    Ok(Trial {
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
        references: extract_references(study)?,
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

pub fn from_nci_trial(trial: &serde_json::Value) -> Result<Trial, BioMcpError> {
    let nct_id = json_get_string(trial, &["nct_id"]).unwrap_or_default();
    let title = json_get_string(trial, &["brief_title"]).unwrap_or_default();
    let status = json_get_string(trial, &["current_trial_status"]).unwrap_or_default();
    let phase = json_get_string(trial, &["phase"]).filter(|s| !s.is_empty());
    let study_type = json_get_string(trial, &["study_protocol_type"]);
    let structured_eligibility = trial
        .get("eligibility")
        .and_then(|value| value.as_object())
        .and_then(|value| value.get("structured"))
        .and_then(|value| value.as_object());
    let min_age = structured_eligibility
        .and_then(|value| json_get_string(&serde_json::Value::Object(value.clone()), &["min_age"]))
        .and_then(|value| TrialAge::from_provider(&value));
    let max_age = structured_eligibility
        .and_then(|value| json_get_string(&serde_json::Value::Object(value.clone()), &["max_age"]))
        .and_then(|value| {
            if value.eq_ignore_ascii_case("999 Years") {
                TrialAge::retained_unparsed(value)
            } else {
                TrialAge::from_provider(&value)
            }
        });
    let age_range = format_age_range(min_age.as_ref(), max_age.as_ref());
    let sex = structured_eligibility
        .and_then(|value| json_get_string(&serde_json::Value::Object(value.clone()), &["sex"]))
        .and_then(|value| format_sex(Some(&value)));
    let eligibility =
        (sex.is_some() || min_age.is_some() || max_age.is_some()).then_some(TrialEligibility {
            sex,
            minimum_age: min_age,
            maximum_age: max_age,
        });
    let sponsor = json_get_string(trial, &["lead_org"]).filter(|s| !s.is_empty());
    let enrollment = json_get_string(trial, &["minimum_target_accrual_number"])
        .and_then(|s| s.parse::<i32>().ok());
    let start_date = json_get_string(trial, &["start_date"]).filter(|s| !s.is_empty());
    let completion_date = json_get_string(trial, &["completion_date"]).filter(|s| !s.is_empty());
    let raw_summary = json_get_string(trial, &["brief_summary"]);
    let summary = normalize_summary(raw_summary.as_deref());
    let conditions = nci_conditions(trial, &["diseases"])?;
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
    let why_stopped = trial
        .get("why_study_stopped")
        .map(|_| json_get_string(trial, &["why_study_stopped"]));

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
        eligibility,
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
