//! PGx markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn pgx_markdown(pgx: &Pgx, requested_sections: &[String]) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pgx.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_recommendations_section = include_all || has_requested("recommendations");
    let show_frequencies_section = include_all || has_requested("frequencies");
    let show_guidelines_section = include_all || has_requested("guidelines");
    let show_annotations_section = include_all || has_requested("annotations");
    let label = pgx
        .gene
        .as_deref()
        .or(pgx.drug.as_deref())
        .unwrap_or(pgx.query.as_str());

    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(label, requested_sections),
        query => &pgx.query,
        gene => &pgx.gene,
        drug => &pgx.drug,
        interactions => &pgx.interactions,
        recommendations => &pgx.recommendations,
        frequencies => &pgx.frequencies,
        guidelines => &pgx.guidelines,
        annotations => &pgx.annotations,
        annotations_note => &pgx.annotations_note,
        show_recommendations_section => show_recommendations_section,
        show_frequencies_section => show_frequencies_section,
        show_guidelines_section => show_guidelines_section,
        show_annotations_section => show_annotations_section,
        sections_block => format_sections_block("pgx", &pgx.query, sections_pgx(pgx, requested_sections)),
        related_block => format_related_block(related_pgx(pgx)),
    })?;
    let body = append_source_state_messages(body, "pgx", &pgx.section_outcomes);
    Ok(append_evidence_urls(body, pgx_evidence_urls(pgx)))
}

pub fn pgx_search_markdown_with_footer(
    query: &str,
    results: &[PgxSearchResult],
    pagination_footer: &str,
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("pgx_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        results => results,
        pagination_footer => pagination_footer,
    })?;
    Ok(with_pagination_footer(body, pagination_footer))
}
