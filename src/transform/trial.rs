use std::borrow::Cow;

use crate::entities::trial::{
    Trial, TrialArm, TrialLocation, TrialOutcome, TrialOutcomes, TrialReference, TrialSearchResult,
};
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
                contact_phone: contact.and_then(|c| clean_opt(c.phone.as_deref())),
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

    Trial {
        nct_id: id,
        source: None,
        title,
        status,
        phase,
        study_type,
        age_range,
        conditions,
        interventions,
        sponsor,
        enrollment,
        summary,
        start_date,
        completion_date,
        eligibility_text: None,
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

fn json_get_string_list(value: &serde_json::Value, keys: &[&str], max: usize) -> Vec<String> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return vec![],
    };
    for key in keys {
        let Some(v) = obj.get(*key) else { continue };
        match v {
            serde_json::Value::Array(arr) => {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .take(max)
                    .map(|s| s.to_string())
                    .collect();
            }
            serde_json::Value::String(s) if !s.trim().is_empty() => {
                return s
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .take(max)
                    .map(|s| s.to_string())
                    .collect();
            }
            _ => {}
        }
    }
    vec![]
}

pub fn from_nci_hit(hit: &serde_json::Value) -> TrialSearchResult {
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
    let conditions = json_get_string_list(hit, &["diseases", "conditions"], 10);

    TrialSearchResult {
        nct_id,
        title,
        status,
        phase,
        conditions,
        sponsor,
    }
}

pub fn from_nci_trial(trial: &serde_json::Value) -> Trial {
    let nct_id = json_get_string(trial, &["nct_id", "nctId", "nctID"]).unwrap_or_default();
    let title = json_get_string(trial, &["brief_title", "briefTitle", "title"]).unwrap_or_default();
    let status = json_get_string(trial, &["current_trial_status", "status", "overallStatus"])
        .unwrap_or_default();
    let phase =
        json_get_string(trial, &["phase", "phase_code", "phaseCode"]).filter(|s| !s.is_empty());
    let study_type = json_get_string(trial, &["study_type", "studyType", "primary_purpose"])
        .filter(|s| !s.is_empty());
    let age_range = format_age_range(
        json_get_string(trial, &["minimum_age", "minimumAge", "min_age"]).as_deref(),
        json_get_string(trial, &["maximum_age", "maximumAge", "max_age"]).as_deref(),
    );
    let sponsor = json_get_string(
        trial,
        &["lead_org", "lead_organization", "leadSponsor", "sponsor"],
    )
    .filter(|s| !s.is_empty());
    let enrollment = json_get_string(
        trial,
        &["enrollment", "enrollment_target", "target_enrollment"],
    )
    .and_then(|s| s.parse::<i32>().ok());
    let start_date = json_get_string(trial, &["start_date", "startDate"]).filter(|s| !s.is_empty());
    let completion_date =
        json_get_string(trial, &["completion_date", "completionDate"]).filter(|s| !s.is_empty());
    let summary = json_get_string(trial, &["brief_summary", "briefSummary", "summary"])
        .map(|s| truncate_summary(&s))
        .filter(|s| !s.is_empty());
    let conditions = json_get_string_list(trial, &["diseases", "conditions"], 25);
    let interventions = json_get_string_list(trial, &["interventions"], 25);

    Trial {
        nct_id,
        source: None,
        title,
        status,
        phase,
        study_type,
        age_range,
        conditions,
        interventions,
        sponsor,
        enrollment,
        summary,
        start_date,
        completion_date,
        eligibility_text: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn truncate_summary_two_sentences_and_length() {
        let s = "Sentence one. Sentence two. Sentence three.";
        let out = truncate_summary(s);
        assert_eq!(out, "Sentence one. Sentence two.");

        let long = "€".repeat(400);
        let out2 = truncate_summary(&long);
        assert!(out2.ends_with("..."));
        assert!(out2.len() <= 503);
    }

    #[test]
    fn format_age_range_handles_missing_bounds() {
        assert_eq!(
            format_age_range(Some("18 Years"), Some("65 Years")).as_deref(),
            Some("18 Years to 65 Years")
        );
        assert_eq!(
            format_age_range(Some("18 Years"), None).as_deref(),
            Some("18 Years to Any age")
        );
        assert_eq!(
            format_age_range(None, Some("65 Years")).as_deref(),
            Some("Any age to 65 Years")
        );
        assert_eq!(format_age_range(None, None), None);
    }

    #[test]
    fn from_ctgov_study_extracts_age_and_locations_sorted() {
        let study: CtGovStudy = serde_json::from_value(json!({
            "protocolSection": {
                "identificationModule": {"nctId": "NCT01234567", "briefTitle": "Test Trial"},
                "statusModule": {"overallStatus": "RECRUITING"},
                "designModule": {"phases": ["PHASE2"]},
                "eligibilityModule": {"minimumAge": "18 Years", "maximumAge": "75 Years"},
                "contactsLocationsModule": {
                    "locations": [
                        {
                            "facility": "Site B",
                            "city": "Boston",
                            "country": "USA",
                            "status": "COMPLETED",
                            "contacts": [{"name": "Late Contact", "phone": "333"}]
                        },
                        {
                            "facility": "Site A",
                            "city": "New York",
                            "country": "USA",
                            "status": "RECRUITING",
                            "contacts": [{"name": "Lead Contact", "phone": "111"}]
                        }
                    ]
                }
            }
        }))
        .unwrap();

        let trial = from_ctgov_study(&study);
        assert_eq!(trial.age_range.as_deref(), Some("18 Years to 75 Years"));
        let locations = trial.locations.expect("locations");
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].facility, "Site A");
        assert_eq!(locations[0].contact_name.as_deref(), Some("Lead Contact"));
    }

    #[test]
    fn from_ctgov_study_extracts_arms_and_outcomes() {
        let study: CtGovStudy = serde_json::from_value(json!({
            "protocolSection": {
                "identificationModule": {"nctId": "NCT09876543", "briefTitle": "Arms Trial"},
                "statusModule": {"overallStatus": "ACTIVE"},
                "armsInterventionsModule": {
                    "interventions": [
                        {
                            "name": "Pembrolizumab",
                            "armGroupLabels": ["Experimental Arm"]
                        }
                    ],
                    "armGroups": [
                        {
                            "label": "Experimental Arm",
                            "armGroupType": "EXPERIMENTAL",
                            "description": "Experimental group",
                            "interventionNames": []
                        }
                    ]
                },
                "outcomesModule": {
                    "primaryOutcomes": [
                        {
                            "measure": "Overall survival",
                            "description": "OS at 12 months",
                            "timeFrame": "12 months"
                        }
                    ],
                    "secondaryOutcomes": [
                        {
                            "measure": "Progression-free survival",
                            "description": "PFS",
                            "timeFrame": "6 months"
                        }
                    ]
                }
            }
        }))
        .unwrap();

        let trial = from_ctgov_study(&study);
        let arms = trial.arms.expect("arms");
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].label, "Experimental Arm");
        assert_eq!(arms[0].interventions, vec!["Pembrolizumab"]);

        let outcomes = trial.outcomes.expect("outcomes");
        assert_eq!(outcomes.primary.len(), 1);
        assert_eq!(outcomes.primary[0].measure, "Overall survival");
        assert_eq!(outcomes.secondary.len(), 1);
    }

    #[test]
    fn from_nci_trial_maps_alias_fields_and_age_range() {
        let trial = from_nci_trial(&json!({
            "nctId": "NCT11111111",
            "briefTitle": "NCI trial",
            "overallStatus": "RECRUITING",
            "phaseCode": "PHASE3",
            "studyType": "Interventional",
            "minimumAge": "21 Years",
            "maximumAge": "80 Years",
            "leadSponsor": "NCI",
            "target_enrollment": "120",
            "startDate": "2020-01-01",
            "completionDate": "2024-12-31",
            "briefSummary": "Sentence one. Sentence two. Sentence three.",
            "diseases": ["Melanoma"],
            "interventions": ["Drug X"]
        }));

        assert_eq!(trial.nct_id, "NCT11111111");
        assert_eq!(trial.phase.as_deref(), Some("PHASE3"));
        assert_eq!(trial.age_range.as_deref(), Some("21 Years to 80 Years"));
        assert_eq!(trial.enrollment, Some(120));
        assert_eq!(trial.conditions, vec!["Melanoma"]);
        assert_eq!(trial.interventions, vec!["Drug X"]);
        assert!(
            trial
                .summary
                .as_deref()
                .is_some_and(|v| v.contains("Sentence one. Sentence two."))
        );
    }

    #[test]
    fn trial_sections_maps_nci_format() {
        let trial = from_nci_trial(&json!({
            "nct_id": "NCT02296125",
            "brief_title": "Osimertinib in EGFR-mutant NSCLC",
            "current_trial_status": "ACTIVE",
            "phase_code": "PHASE3",
            "study_type": "Interventional",
            "minimum_age": "18 Years",
            "maximum_age": "75 Years",
            "lead_org": "AstraZeneca",
            "enrollment_target": "420",
            "diseases": ["Non-small cell lung cancer"],
            "interventions": ["Osimertinib"]
        }));

        assert_eq!(trial.nct_id, "NCT02296125");
        assert_eq!(trial.phase.as_deref(), Some("PHASE3"));
        assert_eq!(trial.sponsor.as_deref(), Some("AstraZeneca"));
        assert_eq!(trial.enrollment, Some(420));
        assert_eq!(trial.conditions, vec!["Non-small cell lung cancer"]);
    }

    #[test]
    fn trial_status_normalization_variants() {
        let hit_a = from_nci_hit(&json!({
            "nctId": "NCT02000622",
            "briefTitle": "Olaparib Study",
            "status": "recruiting"
        }));
        let hit_b = from_nci_hit(&json!({
            "nctId": "NCT04303780",
            "briefTitle": "KRAS G12C Study",
            "overallStatus": "RECRUITING"
        }));

        assert_eq!(hit_a.status.to_ascii_uppercase(), "RECRUITING");
        assert_eq!(hit_b.status.to_ascii_uppercase(), "RECRUITING");
    }
}
