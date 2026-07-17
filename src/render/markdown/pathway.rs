//! Pathway markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn pathway_markdown(
    pathway: &Pathway,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pathway.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_genes_section = !section_only || include_all || has_requested("genes");
    let show_events_section = !section_only || include_all || has_requested("events");
    let show_enrichment_section = !section_only || include_all || has_requested("enrichment");
    let pathway_label = if pathway.name.trim().is_empty() {
        pathway.id.as_str()
    } else {
        pathway.name.as_str()
    };
    let mut body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(pathway_label, requested_sections),
        pathway_source_label => crate::render::provenance::pathway_source_label(&pathway.source),
        source => &pathway.source,
        id => &pathway.id,
        name => &pathway.name,
        species => &pathway.species,
        summary => &pathway.summary,
        genes => &pathway.genes,
        events => &pathway.events,
        enrichment => &pathway.enrichment,
        show_genes_section => show_genes_section,
        show_events_section => show_events_section,
        show_enrichment_section => show_enrichment_section,
        sections_block => format_sections_block("pathway", &pathway.id, sections_pathway(pathway, requested_sections)),
        related_block => format_related_block(related_pathway(pathway)),
    })?;
    for key in ["genes", "events", "enrichment"] {
        if let Some(outcome) = pathway.section_outcomes.get(key)
            && matches!(
                outcome.outcome(),
                crate::entities::section_outcome::SectionOutcomeState::Degraded
                    | crate::entities::section_outcome::SectionOutcomeState::Unavailable
            )
            && let Some(message) = outcome.message()
        {
            body.push_str(&format!("\n\n**{key} status:** {message}"));
        }
    }
    Ok(append_evidence_urls(body, pathway_evidence_urls(pathway)))
}

pub fn pathway_search_markdown_with_footer(
    query: &str,
    results: &[PathwaySearchResult],
    total: Option<usize>,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pathway_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
