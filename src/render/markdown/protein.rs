//! Protein markdown renderers and protein-specific view helpers.

use super::*;

#[cfg(test)]
mod tests;

#[derive(serde::Serialize)]
struct ProteinComplexSummaryRow {
    accession: String,
    name: String,
    component_count: usize,
    curation: String,
}

#[derive(serde::Serialize)]
struct ProteinComplexDetailRow {
    accession: String,
    component_count: usize,
    component_preview: String,
    remaining_count: usize,
    description: Option<String>,
}

fn format_protein_complex_component(component: &ProteinComplexComponent) -> String {
    let accession = component.accession.trim();
    let name = component.name.trim();
    let label = if name.is_empty() { accession } else { name };
    let stoichiometry = component
        .stoichiometry
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match stoichiometry {
        Some(stoichiometry) => format!("{label} ({stoichiometry})"),
        None => label.to_string(),
    }
}

fn protein_complex_summary_rows(complexes: &[ProteinComplex]) -> Vec<ProteinComplexSummaryRow> {
    complexes
        .iter()
        .map(|complex| ProteinComplexSummaryRow {
            accession: markdown_cell(&complex.accession),
            name: markdown_cell(&complex.name),
            component_count: complex.components.len(),
            curation: match &complex.curation {
                ProteinComplexCuration::Curated => "curated".to_string(),
                ProteinComplexCuration::Predicted => "predicted".to_string(),
            },
        })
        .collect()
}

fn protein_complex_detail_rows(complexes: &[ProteinComplex]) -> Vec<ProteinComplexDetailRow> {
    complexes
        .iter()
        .map(|complex| {
            let component_count = complex.components.len();
            let preview_components = complex
                .components
                .iter()
                .take(5)
                .map(format_protein_complex_component)
                .map(|component| markdown_cell(&component))
                .collect::<Vec<_>>();
            ProteinComplexDetailRow {
                accession: markdown_cell(&complex.accession),
                component_count,
                component_preview: if preview_components.is_empty() {
                    "none listed".to_string()
                } else {
                    preview_components.join(", ")
                },
                remaining_count: component_count.saturating_sub(preview_components.len()),
                description: complex
                    .description
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(markdown_cell),
            }
        })
        .collect()
}

pub fn protein_markdown(
    protein: &Protein,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("protein.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_domains_section = !section_only || include_all || has_requested("domains");
    let show_interactions_section = !section_only || include_all || has_requested("interactions");
    let show_complexes_section = !section_only || include_all || has_requested("complexes");
    let show_structures_section = !section_only || include_all || has_requested("structures");
    let protein_label = if protein.name.trim().is_empty() {
        protein.accession.as_str()
    } else {
        protein.name.as_str()
    };
    let complex_summaries = protein_complex_summary_rows(&protein.complexes);
    let complex_details = protein_complex_detail_rows(&protein.complexes);
    let mut body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(protein_label, requested_sections),
        accession => &protein.accession,
        entry_id => &protein.entry_id,
        name => &protein.name,
        gene_symbol => &protein.gene_symbol,
        organism => &protein.organism,
        length => &protein.length,
        function => &protein.function,
        structures => &protein.structures,
        structure_count => &protein.structure_count,
        domains => &protein.domains,
        interactions => &protein.interactions,
        complexes => complex_summaries,
        complex_details => complex_details,
        show_domains_section => show_domains_section,
        show_interactions_section => show_interactions_section,
        show_complexes_section => show_complexes_section,
        show_structures_section => show_structures_section,
        sections_block => format_sections_block("protein", &protein.accession, sections_protein(protein, requested_sections)),
        related_block => format_related_block(related_protein(protein, requested_sections)),
    })?;
    for key in ["domains", "interactions", "complexes", "structures"] {
        if let Some(outcome) = protein.section_outcomes.get(key)
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
    Ok(append_evidence_urls(body, protein_evidence_urls(protein)))
}

#[allow(dead_code)]
pub fn protein_search_markdown(
    query: &str,
    results: &[ProteinSearchResult],
) -> Result<String, BioMcpError> {
    protein_search_markdown_with_footer(query, results, "")
}

pub fn protein_search_markdown_with_footer(
    query: &str,
    results: &[ProteinSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("protein_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
