//! Patient markdown renderer.

use super::*;
use crate::entities::patient::Patient;
use crate::entities::patient::search::PatientSearchRow;

/// One `get patient` command per row whose id passes the FHIR id rule.
/// A row without a usable id contributes no command. Kept here rather than
/// beside the other entities' builders: the related module sits at its
/// line-count ceiling, and the patient renderer already owns the search row.
pub fn search_next_commands_patient(rows: &[PatientSearchRow]) -> Vec<String> {
    let out = rows
        .iter()
        .filter_map(|row| row.id.as_deref())
        .map(|id| format!("biomcp get patient {id}"))
        .collect::<Vec<_>>();
    dedupe_markdown_commands(out)
}

/// The command text one search row prints for a usable id. A bad id
/// prints no id and no command.
fn patient_search_next_command(id: &str) -> String {
    markdown_command_code_span(&format!("biomcp get patient {id}"))
}

#[derive(serde::Serialize)]
struct PatientSearchRowView<'a> {
    id: Option<&'a str>,
    gender: Option<&'a str>,
    birth_date: Option<&'a str>,
    next: Option<String>,
}

impl<'a> PatientSearchRowView<'a> {
    fn new(row: &'a PatientSearchRow) -> Self {
        Self {
            id: row.id.as_deref(),
            gender: row.gender.as_deref(),
            birth_date: row.birth_date.as_deref(),
            next: row.id.as_deref().map(patient_search_next_command),
        }
    }
}

pub fn patient_search_markdown(rows: &[PatientSearchRow]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("patient_search.md.j2")?;
    let body = tmpl.render(context! {
        count => rows.len(),
        results => rows.iter().map(PatientSearchRowView::new).collect::<Vec<_>>(),
    })?;
    Ok(body)
}

/// The count the server reported, labeled as server-reported. BioMCP
/// never counts entries itself.
pub fn patient_count_markdown(total: Option<u64>) -> String {
    match total {
        Some(total) => format!("Server-reported patient count: {total}"),
        None => "The server reported no count.".to_string(),
    }
}

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
