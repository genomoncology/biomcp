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
    let show_interactions_section =
        requested.is_empty() || include_all || has_requested("interactions");
    let show_recommendations_section = include_all || has_requested("recommendations");
    let show_frequencies_section = include_all || has_requested("frequencies");
    let show_guidelines_section = include_all || has_requested("guidelines");
    let show_annotations_section = include_all || has_requested("annotations");
    let label = pgx
        .gene
        .as_deref()
        .or(pgx.drug.as_deref())
        .unwrap_or(pgx.query.as_str());
    let page_recommendation_drugs: std::collections::BTreeSet<_> = pgx
        .recommendations
        .iter()
        .map(|row| row.drugname.as_str())
        .collect();
    let held_recommendation_drugs: Vec<_> = pgx
        .recommendation_drugs
        .iter()
        .filter(|drug| !page_recommendation_drugs.contains(drug.as_str()))
        .collect();

    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(label, requested_sections),
        query => &pgx.query,
        gene => &pgx.gene,
        drug => &pgx.drug,
        interactions => &pgx.interactions,
        recommendations => &pgx.recommendations,
        recommendation_drugs => &pgx.recommendation_drugs,
        page_recommendation_drugs => page_recommendation_drugs,
        held_recommendation_drugs => held_recommendation_drugs,
        frequencies => &pgx.frequencies,
        guidelines => &pgx.guidelines,
        annotations => &pgx.annotations,
        annotations_note => &pgx.annotations_note,
        section_pagination => &pgx.section_pagination,
        show_interactions_section => show_interactions_section,
        show_recommendations_section => show_recommendations_section,
        show_frequencies_section => show_frequencies_section,
        show_guidelines_section => show_guidelines_section,
        show_annotations_section => show_annotations_section,
        show_annotations_empty => section_has_healthy_empty(&pgx.section_outcomes, "annotations"),
        sections_block => format_sections_block("pgx", &pgx.query, sections_pgx(pgx, requested_sections)),
        related_block => format_related_block(related_pgx(pgx)),
        source_states => section_render_contexts("pgx", &pgx.query, &pgx.section_outcomes),
    })?;
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
