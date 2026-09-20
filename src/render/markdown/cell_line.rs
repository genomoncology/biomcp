//! Cell line markdown renderers.

use super::*;
use crate::entities::cell_line::{CellLine, CellLineSearchResult, cell_line_attribution};

#[cfg(test)]
mod tests;

pub fn cell_line_markdown(
    cell_line: &CellLine,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("cell_line.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_variants_section = !section_only || include_all || has_requested("variants");
    let show_xrefs_section = !section_only || include_all || has_requested("xrefs");
    // `all` never includes the ChEMBL section, so it renders only when asked for.
    let show_chembl_section = !section_only || has_requested("chembl");
    // The PharmacoDB section is opt-in: asked for by name, never by `all` and
    // never on the plain card.
    let show_drug_response_section = has_requested("drug_response");
    let label = if cell_line.name.trim().is_empty() {
        cell_line.accession.as_str()
    } else {
        cell_line.name.as_str()
    };
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(label, requested_sections),
        source => &cell_line.source,
        accession => &cell_line.accession,
        rrid => &cell_line.rrid,
        resolved_from => &cell_line.resolved_from,
        secondary_accessions => &cell_line.secondary_accessions,
        name => &cell_line.name,
        synonyms => &cell_line.synonyms,
        species => &cell_line.species,
        diseases => &cell_line.diseases,
        category => &cell_line.category,
        sex => &cell_line.sex,
        age => &cell_line.age,
        variants => &cell_line.variants,
        xrefs => &cell_line.xrefs,
        chembl => &cell_line.chembl,
        chembl_empty_message => crate::entities::cell_line::chembl::CELL_LINE_CHEMBL_EMPTY_MESSAGE,
        chembl_attribution => cell_line.chembl.as_ref().map(|section| {
            crate::entities::cell_line::chembl::chembl_attribution(
                &section.data_as_of,
                &section.data_as_of_kind,
            )
        }),
        show_variants_section => show_variants_section,
        show_xrefs_section => show_xrefs_section,
        show_chembl_section => show_chembl_section,
        show_drug_response_section => show_drug_response_section,
        drug_response => &cell_line.drug_response,
        drug_response_counts_line => cell_line.drug_response.as_ref().map(|section| {
            crate::entities::pharmacodb::counts_line(section.total, &section.datasets)
        }),
        drug_response_empty_message =>
            crate::entities::cell_line::pharmacodb::drug_response_empty_message(),
        drug_response_attribution => cell_line.drug_response.as_ref().map(|section| {
            crate::entities::pharmacodb::pharmacodb_attribution(&section.data_as_of)
        }),
        attribution => cell_line_attribution(&cell_line.data_as_of, &cell_line.data_as_of_kind),
        sections_block => format_sections_block(
            "cell-line",
            &cell_line.accession,
            sections_cell_line(cell_line, requested_sections),
        ),
        related_block => format_related_block(related_cell_line(cell_line)),
        source_states => section_render_contexts(
            "cell_line",
            &cell_line.accession,
            &cell_line.section_outcomes,
        ),
    })?;
    Ok(body)
}

pub fn cell_line_search_markdown_with_footer(
    query: &str,
    results: &[CellLineSearchResult],
    total: Option<usize>,
    notes: &[String],
    data_as_of: &str,
    data_as_of_kind: &str,
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("cell_line_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        total => total,
        results => results,
        notes => notes,
        attribution => cell_line_attribution(data_as_of, data_as_of_kind),
        pagination_footer => pagination_footer,
    })?;
    Ok(body)
}
