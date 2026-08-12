//! Search-all and discover markdown renderers.

use super::*;

#[cfg(test)]
mod tests;

pub fn search_all_markdown(
    results: &SearchAllResults,
    counts_only: bool,
) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct SearchAllSectionView {
        entity: String,
        label: String,
        count_label: String,
        error: Option<String>,
        note: Option<String>,
        links: Vec<crate::cli::search_all::SearchAllLink>,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    }

    let tmpl = env()?.get_template("search_all.md.j2")?;
    let sections = results
        .sections
        .iter()
        .map(|section| {
            let rows = if counts_only {
                Vec::new()
            } else {
                section.markdown_rows()
            };
            let count_label = if section.error.is_some() {
                "unknown".to_string()
            } else if let Some(total) = section.total {
                total.to_string()
            } else if section.count > 0 {
                format!("at least {}", section.count)
            } else {
                "at least 0".to_string()
            };
            SearchAllSectionView {
                entity: section.entity.clone(),
                label: section.label.clone(),
                count_label,
                error: section.error.clone(),
                note: section.note.clone(),
                links: section.links.clone(),
                columns: section
                    .markdown_columns()
                    .iter()
                    .map(|column| (*column).to_string())
                    .collect(),
                rows,
            }
        })
        .collect::<Vec<_>>();

    let body = tmpl.render(context! {
        query => &results.query,
        sections => sections,
        counts_only => counts_only,
        searches_dispatched => results.searches_dispatched,
        searches_with_results => results.searches_with_results,
        wall_time_ms => results.wall_time_ms,
    })?;

    if let Some(debug_plan) = results.debug_plan.as_ref() {
        Ok(format!("{}{}", render_debug_plan_block(debug_plan)?, body))
    } else {
        Ok(body)
    }
}

pub fn render_discover(result: &DiscoverResult) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct DiscoverConceptView {
        label: String,
        primary_id: Option<String>,
        synonyms: Vec<String>,
        xrefs: Vec<String>,
        sources: Vec<String>,
    }

    #[derive(serde::Serialize)]
    struct DiscoverGroupView {
        label: String,
        concepts: Vec<DiscoverConceptView>,
    }

    let tmpl = env()?.get_template("discover.md.j2")?;
    let groups = [
        DiscoverType::Gene,
        DiscoverType::Drug,
        DiscoverType::Disease,
        DiscoverType::Symptom,
        DiscoverType::Pathway,
        DiscoverType::Variant,
        DiscoverType::Unknown,
    ]
    .into_iter()
    .filter_map(|kind| {
        let concepts = result
            .concepts
            .iter()
            .filter(|concept| concept.primary_type == kind)
            .map(|concept| DiscoverConceptView {
                label: concept.label.clone(),
                primary_id: concept.primary_id.clone(),
                synonyms: concept.synonyms.clone(),
                xrefs: concept
                    .xrefs
                    .iter()
                    .map(|xref| format!("{}:{}", xref.source, xref.id))
                    .collect(),
                sources: concept
                    .sources
                    .iter()
                    .map(|source| format!("{} ({})", source.source, source.source_type))
                    .collect(),
            })
            .collect::<Vec<_>>();
        if concepts.is_empty() {
            None
        } else {
            Some(DiscoverGroupView {
                label: kind.label().to_string(),
                concepts,
            })
        }
    })
    .collect::<Vec<_>>();

    let body = tmpl.render(context! {
        query => &result.query,
        notes => &result.notes,
        ambiguous => result.ambiguous,
        groups => groups,
        plain_language => &result.plain_language,
        next_commands => &result.next_commands,
    })?;
    Ok(append_evidence_urls(body, discover_evidence_urls(result)))
}
