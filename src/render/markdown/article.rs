//! Article markdown renderers and article-specific view helpers.

use chrono::{NaiveDate, Utc};

use super::*;

#[cfg(test)]
mod tests;

#[derive(serde::Serialize)]
struct ArticleSearchRenderRow {
    pmid: String,
    title: String,
    sources: String,
    date: Option<String>,
    why: String,
    citation_count: Option<u64>,
    is_retracted: Option<bool>,
}

pub struct ArticleSearchRenderContext<'a> {
    pub source_filter: crate::entities::article::ArticleSourceFilter,
    pub semantic_scholar_enabled: bool,
    pub note: Option<&'a str>,
    pub debug_plan: Option<&'a DebugPlan>,
    pub exact_entity_commands: &'a [String],
    pub source_status: &'a [crate::entities::article::ArticleSourceStatus],
}

pub fn article_markdown(
    article: &Article,
    requested_sections: &[String],
) -> Result<String, BioMcpError> {
    let tmpl = env()?.get_template("article.md.j2")?;
    let section_only = is_section_only_requested(requested_sections);
    let include_all = has_all_section(requested_sections);
    let requested = requested_section_names(requested_sections);
    let has_requested = |name: &str| requested.iter().any(|s| s.eq_ignore_ascii_case(name));
    let show_annotations_section = include_all || has_requested("annotations");
    let show_fulltext_section = include_all || has_requested("fulltext");
    let show_semantic_scholar_section = !section_only || include_all || has_requested("tldr");
    let article_label = if article.title.trim().is_empty() {
        "Article"
    } else {
        article.title.trim()
    };
    let body = tmpl.render(context! {
        section_only => section_only,
        section_header => section_header(article_label, requested_sections),
        pmid => &article.pmid,
        pmcid => &article.pmcid,
        doi => &article.doi,
        title => &article.title,
        authors => &article.authors,
        journal => &article.journal,
        date => &article.date,
        citation_count => &article.citation_count,
        publication_type => &article.publication_type,
        open_access => &article.open_access,
        abstract_text => &article.abstract_text,
        full_text_path => &article.full_text_path,
        full_text_note => &article.full_text_note,
        full_text_source => &article.full_text_source,
        annotations => &article.annotations,
        semantic_scholar => &article.semantic_scholar,
        pubtator_fallback => article.pubtator_fallback,
        show_annotations_section => show_annotations_section,
        show_fulltext_section => show_fulltext_section,
        show_semantic_scholar_section => show_semantic_scholar_section,
        sections_block => format_sections_block("article", article.pmid.as_deref().or(article.pmcid.as_deref()).or(article.doi.as_deref()).unwrap_or(""), sections_article(article, requested_sections)),
        related_block => format_related_block(related_article(article)),
    })?;
    Ok(append_evidence_urls(body, article_evidence_urls(article)))
}

pub fn article_entities_markdown(
    pmid: &str,
    annotations: Option<&ArticleAnnotations>,
    limit: Option<usize>,
) -> Result<String, BioMcpError> {
    #[derive(serde::Serialize)]
    struct EntityRow {
        text: String,
        count: u32,
        command: String,
    }

    fn row(text: &str, count: u32, command: String) -> EntityRow {
        EntityRow {
            text: text.to_string(),
            count,
            command,
        }
    }

    let (mut genes, mut diseases, mut chemicals, mut mutations) = if let Some(ann) = annotations {
        (
            ann.genes
                .iter()
                .filter_map(|g| {
                    let text = g.text.trim();
                    let command = article_annotation_command(ArticleAnnotationBucket::Gene, text)?;
                    Some(row(text, g.count, command))
                })
                .collect::<Vec<_>>(),
            ann.diseases
                .iter()
                .filter_map(|d| {
                    let text = d.text.trim();
                    let command =
                        article_annotation_command(ArticleAnnotationBucket::Disease, text)?;
                    Some(row(text, d.count, command))
                })
                .collect::<Vec<_>>(),
            ann.chemicals
                .iter()
                .filter_map(|c| {
                    let text = c.text.trim();
                    let command =
                        article_annotation_command(ArticleAnnotationBucket::Chemical, text)?;
                    Some(row(text, c.count, command))
                })
                .collect::<Vec<_>>(),
            ann.mutations
                .iter()
                .filter_map(|m| {
                    let text = m.text.trim();
                    let command =
                        article_annotation_command(ArticleAnnotationBucket::Mutation, text)?;
                    Some(row(text, m.count, command))
                })
                .collect::<Vec<_>>(),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    if let Some(limit) = limit {
        genes.truncate(limit);
        diseases.truncate(limit);
        chemicals.truncate(limit);
        mutations.truncate(limit);
    }

    let tmpl = env()?.get_template("article_entities.md.j2")?;
    Ok(tmpl.render(context! {
        pmid => pmid,
        genes => genes,
        diseases => diseases,
        chemicals => chemicals,
        mutations => mutations,
    })?)
}

fn article_batch_counts(label: &str, rows: &[AnnotationCount]) -> Option<String> {
    if rows.is_empty() {
        return None;
    }
    Some(format!(
        "{label}: {}",
        rows.iter()
            .map(|row| format!("{} ({})", row.text.trim(), row.count))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn article_batch_entities(summary: Option<&ArticleBatchEntitySummary>) -> Option<String> {
    let summary = summary?;
    let parts = [
        article_batch_counts("Genes", &summary.genes),
        article_batch_counts("Diseases", &summary.diseases),
        article_batch_counts("Chemicals", &summary.chemicals),
        article_batch_counts("Mutations", &summary.mutations),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

pub fn article_batch_markdown(items: &[ArticleBatchItem]) -> Result<String, BioMcpError> {
    let mut out = format!("# Article Batch ({})\n\n", items.len());
    for (idx, item) in items.iter().enumerate() {
        out.push_str(&format!("## {}. {}\n", idx + 1, item.title.trim()));
        if let Some(pmid) = &item.pmid {
            out.push_str(&format!("PMID: {}\n", pmid.trim()));
        } else if let Some(pmcid) = &item.pmcid {
            out.push_str(&format!("PMCID: {}\n", pmcid.trim()));
        } else if let Some(doi) = &item.doi {
            out.push_str(&format!("DOI: {}\n", doi.trim()));
        }
        if let Some(journal) = &item.journal {
            out.push_str(&format!("Journal: {}\n", journal.trim()));
        }
        if let Some(year) = item.year {
            out.push_str(&format!("Year: {}\n", year));
        }
        if let Some(entities) = article_batch_entities(item.entity_summary.as_ref()) {
            out.push_str(&format!("Entities: {}\n", entities));
        }
        if let Some(tldr) = &item.tldr {
            out.push_str(&format!("TLDR: {}\n", tldr.trim()));
        }
        match (item.citation_count, item.influential_citation_count) {
            (Some(c), Some(ic)) => out.push_str(&format!("Citations: {c} (influential: {ic})\n")),
            (Some(c), None) => out.push_str(&format!("Citations: {c}\n")),
            (None, Some(ic)) => out.push_str(&format!("Citations: influential {ic}\n")),
            (None, None) => {}
        }
        out.push('\n');
    }
    Ok(out)
}

pub fn article_graph_markdown(
    kind: &str,
    result: &ArticleGraphResult,
) -> Result<String, BioMcpError> {
    let mut out = format!(
        "# {} for {}\n\n",
        markdown_cell(kind),
        markdown_cell(&article_related_label(&result.article))
    );
    out.push_str("| PMID | Title | Intents | Influential | Context |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    if result.edges.is_empty() {
        out.push_str("| - | - | - | - | No related papers returned |\n");
        return Ok(out);
    }
    for edge in &result.edges {
        let intents = if edge.intents.is_empty() {
            "-".to_string()
        } else {
            markdown_cell(&edge.intents.join(", "))
        };
        let context = edge
            .contexts
            .first()
            .map(|value| markdown_cell(value))
            .unwrap_or_else(|| "-".to_string());
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            article_related_id(&edge.paper),
            markdown_cell(&edge.paper.title),
            intents,
            if edge.is_influential { "yes" } else { "no" },
            context,
        ));
    }
    Ok(out)
}

pub fn article_recommendations_markdown(
    result: &ArticleRecommendationsResult,
) -> Result<String, BioMcpError> {
    let positives = if result.positive_seeds.is_empty() {
        "article".to_string()
    } else {
        result
            .positive_seeds
            .iter()
            .map(article_related_label)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut out = format!("# Recommendations for {}\n\n", markdown_cell(&positives));
    if !result.negative_seeds.is_empty() {
        let negatives = result
            .negative_seeds
            .iter()
            .map(article_related_label)
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "Negative seeds: {}\n\n",
            markdown_cell(&negatives)
        ));
    }
    out.push_str("| PMID | Title | Journal | Year |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    if result.recommendations.is_empty() {
        out.push_str("| - | No recommendations returned | - | - |\n");
        return Ok(out);
    }
    for paper in &result.recommendations {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            article_related_id(paper),
            markdown_cell(&paper.title),
            paper
                .journal
                .as_deref()
                .map(markdown_cell)
                .unwrap_or_else(|| "-".to_string()),
            paper
                .year
                .map(|year| year.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ));
    }
    Ok(out)
}

fn article_sources_label(row: &ArticleSearchResult) -> String {
    let mut sources = if row.matched_sources.is_empty() {
        vec![row.source]
    } else {
        row.matched_sources.clone()
    };
    sources.dedup();
    sources
        .into_iter()
        .map(ArticleSource::display_name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn article_lexical_ranking_label(ranking: &ArticleRankingMetadata) -> Option<String> {
    if ranking.anchor_count == 0 {
        return None;
    }
    if ranking.all_anchors_in_title {
        return Some(format!(
            "title {}/{}",
            ranking.title_anchor_hits, ranking.anchor_count
        ));
    }
    if ranking.all_anchors_in_text {
        return Some(format!(
            "title+abstract {}/{}",
            ranking.combined_anchor_hits, ranking.anchor_count
        ));
    }
    if ranking.abstract_anchor_hits > 0 && ranking.title_anchor_hits > 0 {
        return Some(format!(
            "title+abstract {}/{}",
            ranking.combined_anchor_hits, ranking.anchor_count
        ));
    }
    if ranking.abstract_anchor_hits > 0 {
        return Some(format!(
            "abstract {}/{}",
            ranking.abstract_anchor_hits, ranking.anchor_count
        ));
    }
    if ranking.title_anchor_hits > 0 {
        return Some(format!(
            "title {}/{}",
            ranking.title_anchor_hits, ranking.anchor_count
        ));
    }
    None
}

fn article_lexical_reason(ranking: &ArticleRankingMetadata) -> Option<String> {
    let lexical_label = article_lexical_ranking_label(ranking);
    if ranking.pubmed_rescue {
        return Some(lexical_label.map_or_else(
            || "pubmed-rescue".to_string(),
            |label| format!("pubmed-rescue + {label}"),
        ));
    }
    lexical_label
}

fn format_article_score(value: f64) -> String {
    let mut out = format!("{value:.3}");
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" { "0".to_string() } else { out }
}

fn semantic_scholar_source_status_note(
    source_status: &[crate::entities::article::ArticleSourceStatus],
) -> Option<String> {
    source_status.iter().find_map(|status| {
        if status.source != ArticleSource::SemanticScholar {
            return None;
        }
        let availability = match status.status? {
            crate::entities::article::ArticleSourceAvailability::Degraded => "degraded",
            crate::entities::article::ArticleSourceAvailability::Unavailable => "unavailable",
            _ => return None,
        };
        let auth = match status.auth_mode {
            Some(crate::sources::semantic_scholar::SemanticScholarAuthMode::Authenticated) => {
                "authenticated"
            }
            Some(crate::sources::semantic_scholar::SemanticScholarAuthMode::SharedPool) => {
                "shared_pool"
            }
            None => "unknown_auth",
        };
        Some(format!(
            "Semantic Scholar source status: {availability} ({auth})"
        ))
    })
}

fn article_ranking_why(row: &ArticleSearchResult, filters: &ArticleSearchFilters) -> String {
    if filters.sort != ArticleSort::Relevance {
        return "-".to_string();
    }
    let Some(ranking) = row.ranking.as_ref() else {
        return "-".to_string();
    };
    let lexical_label = article_lexical_ranking_label(ranking);
    match ranking
        .mode
        .or_else(|| crate::entities::article::article_effective_ranking_mode(filters))
        .unwrap_or(ArticleRankingMode::Lexical)
    {
        ArticleRankingMode::Lexical => {
            article_lexical_reason(ranking).unwrap_or_else(|| "-".to_string())
        }
        ArticleRankingMode::Semantic => {
            let mut why = format!(
                "semantic {}",
                format_article_score(ranking.semantic_score.unwrap_or(0.0))
            );
            if let Some(label) = lexical_label {
                why.push_str(" + ");
                why.push_str(&label);
            }
            why
        }
        ArticleRankingMode::Hybrid => {
            let mut why = format!(
                "hybrid {}",
                format_article_score(ranking.composite_score.unwrap_or(0.0))
            );
            if let Some(label) = lexical_label {
                why.push_str(" + ");
                why.push_str(&label);
            }
            why
        }
    }
}

pub fn article_search_markdown_with_footer_and_context(
    query: &str,
    results: &[ArticleSearchResult],
    pagination_footer: &str,
    filters: &ArticleSearchFilters,
    context: ArticleSearchRenderContext<'_>,
) -> Result<String, BioMcpError> {
    let rows = results
        .iter()
        .map(|row| ArticleSearchRenderRow {
            pmid: row.pmid.clone(),
            title: row.title.clone(),
            sources: article_sources_label(row),
            date: row.date.clone(),
            why: article_ranking_why(row, filters),
            citation_count: row.citation_count,
            is_retracted: row.is_retracted,
        })
        .collect::<Vec<_>>();
    let related_block = format_related_block(
        crate::render::markdown::markdown_related_article_search_results(
            results,
            filters,
            context.source_filter,
            context.exact_entity_commands,
        ),
    );
    let index_date_footer = newest_indexed_footer(results);
    let semantic_scholar_source_status_note =
        semantic_scholar_source_status_note(context.source_status);

    let tmpl = env()?.get_template("article_search.md.j2")?;
    let body = tmpl.render(context! {
        query => query,
        count => results.len(),
        rows => rows,
        semantic_scholar_enabled => context.semantic_scholar_enabled,
        semantic_scholar_source_status_note => semantic_scholar_source_status_note,
        note => context.note,
        sort => filters.sort.as_str(),
        ranking_policy => crate::entities::article::article_relevance_ranking_policy(filters),
        related_block => related_block,
        pagination_footer => pagination_footer,
        index_date_footer => index_date_footer,
    })?;
    let body = with_pagination_footer(body, pagination_footer);
    if let Some(debug_plan) = context.debug_plan {
        Ok(format!("{}{}", render_debug_plan_block(debug_plan)?, body))
    } else {
        Ok(body)
    }
}

fn max_first_index_date(results: &[ArticleSearchResult]) -> Option<NaiveDate> {
    results.iter().filter_map(|row| row.first_index_date).max()
}

fn format_newest_indexed_footer(indexed: NaiveDate, today: NaiveDate) -> String {
    let days_ago = (today - indexed).num_days().max(0);
    format!(
        "Newest indexed: {} ({} days ago)",
        indexed.format("%Y-%m-%d"),
        days_ago
    )
}

fn newest_indexed_footer(results: &[ArticleSearchResult]) -> Option<String> {
    Some(format_newest_indexed_footer(
        max_first_index_date(results)?,
        Utc::now().date_naive(),
    ))
}
