//! Patient markdown renderer.

use super::*;
use crate::entities::patient::{Patient, PatientSearchRow};

pub fn patient_markdown(
    patient: &Patient,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("patient.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let show_conditions = has_all_section(requested_sections)
        || requested_section_names(requested_sections)
            .iter()
            .any(|name| name == "conditions");
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&format!("Patient {}", patient.id), requested_sections),
        source => &patient.source,
        id => &patient.id,
        gender => &patient.gender,
        birth_date => &patient.birth_date,
        show_conditions => show_conditions,
        conditions => &patient.conditions,
        sections_block => format_sections_block(
            "patient",
            &patient.id,
            sections_patient(patient, requested_sections),
        ),
        source_states => section_render_contexts(
            "patient",
            &patient.id,
            &patient.section_outcomes,
        ),
    })?;
    Ok(body)
}

/// `biomcp get patient <id>` for each row whose id passed the FHIR id rule.
pub fn patient_search_next_commands(rows: &[PatientSearchRow]) -> Vec<String> {
    rows.iter()
        .filter_map(|row| row.id.as_deref())
        .map(|id| format!("biomcp get patient {id}"))
        .collect()
}

pub fn patient_search_markdown(
    rows: &[PatientSearchRow],
    limit: usize,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("patient_search.md.j2")?;
    Ok(tmpl.render(context! {
        count => rows.len(),
        limit => limit,
        results => rows,
        next_commands => patient_search_next_commands(rows),
    })?)
}

/// The patient count the server reports. It never counts entries itself.
pub fn patient_count_markdown(total: Option<u64>) -> String {
    let line = match total {
        Some(total) => format!("Server-reported total: {total}"),
        None => PATIENT_NO_COUNT.to_string(),
    };
    format!("# Patient count on the FHIR server\n\n{line}\n")
}

/// What `search patient --count` says when the Bundle has no `total`.
pub const PATIENT_NO_COUNT: &str = "The FHIR server reported no count.";
