use super::{SectionSource, has_opt_text, has_text, outcome_section_sources, push_section};
use crate::entities::adverse_event::{
    AdverseEvent, AdverseEventReport, AdverseEventSections, AdverseEventSourceSearch, DeviceEvent,
};

pub(crate) fn source_search_section_sources(
    search: &AdverseEventSourceSearch,
) -> Vec<SectionSource> {
    outcome_section_sources(
        "adverse_event",
        &search.section_outcomes,
        &[("faers", "OpenFDA FAERS"), ("vaers", "CDC CVX/VAERS")],
    )
}

fn section_sources(event: &AdverseEvent) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let overview_present = has_text(&event.report_id)
        || has_text(&event.drug)
        || has_opt_text(&event.patient)
        || has_opt_text(&event.reporter_type)
        || has_opt_text(&event.reporter_country)
        || has_opt_text(&event.indication)
        || has_opt_text(&event.date);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.reactions.is_empty(),
        "reactions",
        "Reactions",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.outcomes.is_empty(),
        "outcomes",
        "Outcomes",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        !event.concomitant_medications.is_empty(),
        "concomitant_drugs",
        "Concomitant Drugs",
        ["OpenFDA"],
    );
    out
}

pub(crate) fn subset_section_sources(
    event: &AdverseEvent,
    sections: AdverseEventSections,
) -> Vec<SectionSource> {
    section_sources(event)
        .into_iter()
        .filter(|source| match source.key.as_str() {
            "reactions" => sections.include_reactions,
            "outcomes" => sections.include_outcomes,
            "concomitant_drugs" => sections.include_concomitant,
            _ => false,
        })
        .collect()
}

fn device_section_sources(event: &DeviceEvent) -> Vec<SectionSource> {
    let mut out = Vec::new();
    let overview_present = has_text(&event.report_id)
        || has_text(&event.device)
        || has_opt_text(&event.report_number)
        || has_opt_text(&event.manufacturer)
        || has_opt_text(&event.event_type)
        || has_opt_text(&event.date);
    push_section(
        &mut out,
        overview_present,
        "overview",
        "Overview",
        ["OpenFDA"],
    );
    push_section(
        &mut out,
        has_opt_text(&event.description),
        "description",
        "Description",
        ["OpenFDA"],
    );
    out
}

pub(crate) fn report_section_sources(report: &AdverseEventReport) -> Vec<SectionSource> {
    match report {
        AdverseEventReport::Faers(event) => section_sources(event),
        AdverseEventReport::Device(event) => device_section_sources(event),
    }
}
