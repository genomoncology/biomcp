//! Patient markdown renderer.

use super::*;
use crate::entities::patient::Patient;

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
