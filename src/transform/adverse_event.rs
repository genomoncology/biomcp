use std::collections::HashSet;

use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventSearchResult, DeviceEvent, DeviceEventSearchResult,
    RecallSearchResult,
};
use crate::sources::openfda::{
    DeviceEventResult, EnforcementResult, FaersEventResult, FaersPatient,
};

fn normalize_drug_name(value: &str) -> String {
    value.trim().trim_matches('.').to_ascii_lowercase()
}

fn drug_name_matches_query(candidate: &str, query: &str) -> bool {
    let candidate = normalize_drug_name(candidate);
    let query = normalize_drug_name(query);
    if candidate.is_empty() || query.is_empty() {
        return false;
    }
    if candidate == query || candidate.contains(&query) {
        return true;
    }

    // Fallback token matching catches common forms like:
    // "sitagliptin and metformin hydrochloride" vs query "metformin".
    let candidate_tokens = candidate
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|v| !v.is_empty())
        .collect::<HashSet<_>>();
    query
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|v| !v.is_empty())
        .all(|token| candidate_tokens.contains(token))
}

fn suspect_drug_names(patient: Option<&FaersPatient>) -> Vec<String> {
    let Some(patient) = patient else {
        return Vec::new();
    };

    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for d in &patient.drug {
        if d.drugcharacterization.as_deref() != Some("1") {
            continue;
        }

        let mut candidates: Vec<String> = d
            .openfda
            .as_ref()
            .map(|o| {
                o.generic_name
                    .iter()
                    .map(|v| normalize_drug_name(v))
                    .filter(|v| !v.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(med) = d.medicinalproduct.as_deref() {
            let med = normalize_drug_name(med);
            if !med.is_empty() {
                candidates.push(med);
            }
        }

        for name in candidates {
            if !seen.insert(name.clone()) {
                continue;
            }
            out.push(name);
        }
    }

    out
}

pub fn faers_report_matches_suspect_drug_query(r: &FaersEventResult, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }
    suspect_drug_names(r.patient.as_ref())
        .iter()
        .any(|candidate| drug_name_matches_query(candidate, query))
}

fn normalize_patient_sex(code: Option<&str>) -> Option<&'static str> {
    match code.map(str::trim) {
        Some("1") => Some("Male"),
        Some("2") => Some("Female"),
        _ => None,
    }
}

fn normalize_age_unit(unit: Option<&str>) -> Option<&'static str> {
    match unit.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("801") | Some("year") | Some("years") => Some("years"),
        Some("802") | Some("month") | Some("months") => Some("months"),
        Some("803") | Some("week") | Some("weeks") => Some("weeks"),
        Some("804") | Some("day") | Some("days") => Some("days"),
        Some("805") | Some("hour") | Some("hours") => Some("hours"),
        Some("806") | Some("decade") | Some("decades") => Some("decades"),
        _ => None,
    }
}

fn patient_demographics(patient: Option<&FaersPatient>) -> Option<String> {
    let patient = patient?;
    let age = patient
        .patientonsetage
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|age| {
            if age.chars().any(|c| c.is_ascii_alphabetic()) {
                age.to_string()
            } else if let Some(unit) = normalize_age_unit(patient.patientonsetageunit.as_deref()) {
                format!("{age} {unit}")
            } else {
                age.to_string()
            }
        });

    let sex = normalize_patient_sex(patient.patientsex.as_deref()).map(str::to_string);
    let weight = patient
        .patientweight
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| format!("{v} kg"));

    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = age {
        parts.push(v);
    }
    if let Some(v) = sex {
        parts.push(v);
    }
    if let Some(v) = weight {
        parts.push(v);
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn pick_suspect_drug(
    patient: Option<&FaersPatient>,
    preferred_query: Option<&str>,
) -> Option<String> {
    let suspects = suspect_drug_names(patient);
    if let Some(query) = preferred_query {
        for candidate in &suspects {
            if drug_name_matches_query(candidate, query) {
                return Some(candidate.clone());
            }
        }
    }
    suspects.first().cloned()
}

fn outcomes_from_flags(r: &FaersEventResult) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if r.seriousnessdeath.as_deref() == Some("1") {
        out.push("Death".into());
    }
    if r.seriousnesslifethreatening.as_deref() == Some("1") {
        out.push("Life-threatening".into());
    }
    if r.seriousnesshospitalization.as_deref() == Some("1") {
        out.push("Hospitalization".into());
    }
    if r.seriousnessdisabling.as_deref() == Some("1") {
        out.push("Disability".into());
    }
    if r.seriousnesscongenitalanomali.as_deref() == Some("1") {
        out.push("Congenital anomaly".into());
    }
    if r.seriousnessother.as_deref() == Some("1") {
        out.push("Other serious".into());
    }
    out
}

fn concomitant_medications(patient: Option<&FaersPatient>, limit: usize) -> Vec<String> {
    let Some(patient) = patient else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for drug in &patient.drug {
        if drug.drugcharacterization.as_deref() != Some("2") {
            continue;
        }
        let Some(name) = drug
            .medicinalproduct
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        let key = name.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(name.to_string());
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn reporter_type(r: &FaersEventResult) -> Option<String> {
    match r
        .primarysource
        .as_ref()
        .and_then(|s| s.qualification.as_deref())
        .map(str::trim)
    {
        Some("1") => Some("Physician".into()),
        Some("2") => Some("Pharmacist".into()),
        Some("3") => Some("Other".into()),
        Some("4") => Some("Lawyer".into()),
        Some("5") => Some("Consumer".into()),
        Some(v) if !v.is_empty() => Some(v.to_string()),
        _ => None,
    }
}

fn reporter_country(r: &FaersEventResult) -> Option<String> {
    r.primarysource
        .as_ref()
        .and_then(|s| s.reportercountry.as_deref())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn indication(patient: Option<&FaersPatient>) -> Option<String> {
    let patient = patient?;
    for drug in &patient.drug {
        if drug.drugcharacterization.as_deref() != Some("1") {
            continue;
        }
        let value = drug
            .drugindication
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        if value.is_some() {
            return value;
        }
    }

    patient.drug.iter().find_map(|drug| {
        drug.drugindication
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    })
}

fn normalize_date_yyyymmdd(value: Option<&str>) -> Option<String> {
    let v = value?.trim();
    if v.len() != 8 || !v.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("{}-{}-{}", &v[0..4], &v[4..6], &v[6..8]))
}

fn reactions_from_patient(patient: Option<&FaersPatient>, limit: usize) -> Vec<String> {
    let Some(patient) = patient else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for rx in &patient.reaction {
        let Some(term) = rx
            .reactionmeddrapt
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        let key = term.to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        out.push(term.to_string());
        if out.len() >= limit {
            break;
        }
    }

    out
}

pub fn from_openfda_faers_search_result(
    r: &FaersEventResult,
    requested_drug: Option<&str>,
) -> AdverseEventSearchResult {
    let drug =
        pick_suspect_drug(r.patient.as_ref(), requested_drug).unwrap_or_else(|| "unknown".into());
    AdverseEventSearchResult {
        report_id: r.safetyreportid.clone(),
        drug,
        reactions: reactions_from_patient(r.patient.as_ref(), 3),
        serious: r.serious.as_deref() == Some("1"),
    }
}

pub fn from_openfda_faers_get_result(r: &FaersEventResult) -> AdverseEvent {
    let drug = pick_suspect_drug(r.patient.as_ref(), None).unwrap_or_else(|| "unknown".into());
    AdverseEvent {
        report_id: r.safetyreportid.clone(),
        drug,
        reactions: reactions_from_patient(r.patient.as_ref(), 15),
        outcomes: outcomes_from_flags(r),
        patient: patient_demographics(r.patient.as_ref()),
        concomitant_medications: concomitant_medications(r.patient.as_ref(), 5),
        reporter_type: reporter_type(r),
        reporter_country: reporter_country(r),
        indication: indication(r.patient.as_ref()),
        serious: r.serious.as_deref() == Some("1"),
        date: normalize_date_yyyymmdd(r.receivedate.as_deref()),
    }
}

pub fn from_openfda_enforcement_result(r: &EnforcementResult) -> RecallSearchResult {
    RecallSearchResult {
        recall_number: r.recall_number.trim().to_string(),
        classification: r.classification.trim().to_string(),
        product_description: r.product_description.trim().to_string(),
        reason_for_recall: r.reason_for_recall.trim().to_string(),
        status: r.status.trim().to_string(),
        distribution_pattern: r
            .distribution_pattern
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        recall_initiation_date: normalize_date_yyyymmdd(r.recall_initiation_date.as_deref()),
    }
}

fn pick_device_name(r: &DeviceEventResult) -> String {
    for d in &r.device {
        if let Some(name) = d
            .brand_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return name.to_string();
        }
        if let Some(name) = d
            .generic_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return name.to_string();
        }
    }
    r.manufacturer_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn pick_device_manufacturer(r: &DeviceEventResult) -> Option<String> {
    for d in &r.device {
        if let Some(name) = d
            .manufacturer_d_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(name.to_string());
        }
    }
    r.manufacturer_name
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

fn pick_device_description(r: &DeviceEventResult) -> Option<String> {
    let mut fallback: Option<&str> = None;
    for t in &r.mdr_text {
        let text = t.text.as_deref().map(str::trim).filter(|v| !v.is_empty());
        let Some(text) = text else {
            continue;
        };
        if fallback.is_none() {
            fallback = Some(text);
        }
        let kind = t
            .text_type_code
            .as_deref()
            .map(str::trim)
            .unwrap_or("")
            .to_ascii_lowercase();
        if kind.contains("description") || kind.contains("narrative") || kind.contains("event") {
            return Some(text.to_string());
        }
    }

    let text = fallback?;
    Some(text.to_string())
}

fn truncate_text(value: Option<String>, max_bytes: usize) -> Option<String> {
    let mut v = value?;
    if v.len() <= max_bytes {
        return Some(v);
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !v.is_char_boundary(boundary) {
        boundary -= 1;
    }
    v.truncate(boundary);
    v = v.trim_end().to_string();
    v.push('…');
    Some(v)
}

pub fn from_openfda_device_search_result(r: &DeviceEventResult) -> DeviceEventSearchResult {
    DeviceEventSearchResult {
        report_id: r.mdr_report_key.trim().to_string(),
        device: pick_device_name(r),
        event_type: r
            .event_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        date: normalize_date_yyyymmdd(r.date_of_event.as_deref().or(r.date_received.as_deref())),
        description: truncate_text(pick_device_description(r), 220),
    }
}

pub fn from_openfda_device_get_result(r: &DeviceEventResult) -> DeviceEvent {
    DeviceEvent {
        report_id: r.mdr_report_key.trim().to_string(),
        report_number: r
            .report_number
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        device: pick_device_name(r),
        manufacturer: pick_device_manufacturer(r),
        event_type: r
            .event_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string()),
        date: normalize_date_yyyymmdd(r.date_of_event.as_deref().or(r.date_received.as_deref())),
        description: truncate_text(pick_device_description(r), 1200),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::openfda::{FaersDrug, FaersOpenFdaDrug, FaersReaction};

    #[test]
    fn normalize_drug_name_trims_and_lowercases() {
        assert_eq!(normalize_drug_name("  Pembrolizumab "), "pembrolizumab");
        assert_eq!(normalize_drug_name("Metformin."), "metformin");
        assert_eq!(
            normalize_drug_name("SITAGLIPTIN and METFORMIN"),
            "sitagliptin and metformin"
        );
    }

    #[test]
    fn outcomes_from_flags_maps_all_fields() {
        let report = FaersEventResult {
            safetyreportid: "9".into(),
            serious: Some("1".into()),
            receivedate: None,
            seriousnessdeath: Some("1".into()),
            seriousnesslifethreatening: Some("1".into()),
            seriousnesshospitalization: Some("1".into()),
            seriousnessdisabling: Some("1".into()),
            seriousnesscongenitalanomali: Some("1".into()),
            seriousnessother: Some("1".into()),
            primarysource: None,
            patient: None,
        };

        let outcomes = outcomes_from_flags(&report);
        assert_eq!(
            outcomes,
            vec![
                "Death",
                "Life-threatening",
                "Hospitalization",
                "Disability",
                "Congenital anomaly",
                "Other serious",
            ]
        );
    }

    #[test]
    fn patient_demographics_handles_missing_fields() {
        let patient = FaersPatient {
            patientonsetage: None,
            patientonsetageunit: None,
            patientsex: None,
            patientweight: None,
            reaction: Vec::new(),
            drug: Vec::new(),
        };

        assert_eq!(patient_demographics(Some(&patient)), None);
        assert_eq!(patient_demographics(None), None);
    }

    #[test]
    fn faers_report_filter_matches_suspect_drug_name() {
        let report = FaersEventResult {
            safetyreportid: "1".into(),
            serious: Some("1".into()),
            receivedate: None,
            seriousnessdeath: None,
            seriousnesslifethreatening: None,
            seriousnesshospitalization: None,
            seriousnessdisabling: None,
            seriousnesscongenitalanomali: None,
            seriousnessother: None,
            primarysource: None,
            patient: Some(FaersPatient {
                patientonsetage: None,
                patientonsetageunit: None,
                patientsex: None,
                patientweight: None,
                reaction: vec![FaersReaction {
                    reactionmeddrapt: Some("Nausea".into()),
                    reactionoutcome: None,
                }],
                drug: vec![
                    FaersDrug {
                        medicinalproduct: Some("Tofacitinib".into()),
                        drugcharacterization: Some("1".into()),
                        drugindication: None,
                        openfda: Some(FaersOpenFdaDrug {
                            generic_name: vec!["tofacitinib".into()],
                        }),
                    },
                    FaersDrug {
                        medicinalproduct: Some("Metformin Hydrochloride".into()),
                        drugcharacterization: Some("1".into()),
                        drugindication: None,
                        openfda: Some(FaersOpenFdaDrug {
                            generic_name: vec!["metformin".into()],
                        }),
                    },
                ],
            }),
        };

        assert!(faers_report_matches_suspect_drug_query(
            &report,
            "metformin"
        ));
        assert!(!faers_report_matches_suspect_drug_query(
            &report,
            "pembrolizumab"
        ));
        let row = from_openfda_faers_search_result(&report, Some("metformin"));
        assert_eq!(row.drug, "metformin");
    }
}
