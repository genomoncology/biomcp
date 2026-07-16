//! Gene markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn gene_markdown(gene: &Gene, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gene.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_civic_section = include_all || has_requested("civic");
    let show_expression_section = include_all || has_requested("expression");
    let show_hpa_section = include_all || has_requested("hpa");
    let show_druggability_section =
        include_all || has_requested("druggability") || has_requested("drugs");
    let show_clingen_section = include_all || has_requested("clingen");
    let show_constraint_section = include_all || has_requested("constraint");
    let show_disgenet_section = has_requested("disgenet");
    let show_funding_section = has_requested("funding");
    let show_diagnostics_section = has_requested("diagnostics");
    let funding_rows = funding_rows(gene.funding.as_ref());
    let funding_summary = funding_summary_line(gene.funding.as_ref());
    let diagnostic_rows =
        super::diagnostic::diagnostic_search_rows(gene.diagnostics.as_deref().unwrap_or(&[]));
    let mut body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(&gene.symbol, requested_sections),
        symbol => &gene.symbol,
        name => &gene.name,
        entrez_id => &gene.entrez_id,
        ensembl_id => &gene.ensembl_id,
        location => &gene.location,
        genomic_coordinates => &gene.genomic_coordinates,
        omim_id => &gene.omim_id,
        uniprot_id => &gene.uniprot_id,
        summary => &gene.summary,
        gene_type => &gene.gene_type,
        aliases => &gene.aliases,
        clinical_diseases => &gene.clinical_diseases,
        clinical_drugs => &gene.clinical_drugs,
        pathways => &gene.pathways,
        ontology => &gene.ontology,
        diseases => &gene.diseases,
        protein => &gene.protein,
        go_terms => &gene.go,
        interactions => &gene.interactions,
        civic => &gene.civic,
        expression => &gene.expression,
        hpa => &gene.hpa,
        druggability => &gene.druggability,
        clingen => &gene.clingen,
        constraint => &gene.constraint,
        disgenet => &gene.disgenet,
        funding => &gene.funding,
        funding_note => &gene.funding_note,
        funding_rows => funding_rows,
        funding_summary => funding_summary,
        diagnostics_note => &gene.diagnostics_note,
        diagnostic_rows => diagnostic_rows,
        show_civic_section => show_civic_section,
        show_expression_section => show_expression_section,
        show_hpa_section => show_hpa_section,
        show_druggability_section => show_druggability_section,
        show_clingen_section => show_clingen_section,
        show_constraint_section => show_constraint_section,
        show_disgenet_section => show_disgenet_section,
        show_funding_section => show_funding_section,
        show_diagnostics_section => show_diagnostics_section,
        sections_block => format_sections_block("gene", &gene.symbol, sections_gene(gene, requested_sections)),
        related_block => format_related_block(related_gene(gene)),
    })?;
    for (key, outcome) in gene.section_outcomes.iter() {
        if matches!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Degraded
                | crate::entities::section_outcome::SectionOutcomeState::Unavailable
        ) && let Some(message) = outcome.message()
        {
            body.push_str(&format!("\n\n**{key} status:** {message}"));
        }
    }
    Ok(append_evidence_urls(body, gene_evidence_urls(gene)))
}

#[allow(dead_code)]
pub fn gene_search_markdown(
    query: &str,
    results: &[GeneSearchResult],
) -> Result<String, BioMcpError> {
    gene_search_markdown_with_footer(query, results, "")
}

pub fn gene_search_markdown_with_footer(
    query: &str,
    results: &[GeneSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("gene_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
