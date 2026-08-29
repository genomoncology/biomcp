use super::{DiagnosticGetArgs, DiagnosticSearchArgs};
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(
    args: &DiagnosticSearchArgs,
) -> Result<(), crate::error::BioMcpError> {
    if args.limit == 0 || args.limit > 50 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    Ok(())
}

pub(in crate::cli) async fn handle_get(
    args: DiagnosticGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let diagnostic = crate::entities::diagnostic::get(&args.accession, &sections).await?;
    let text = render_loaded_card(&diagnostic, &sections, json_output)?;
    Ok(CommandOutcome::stdout(text))
}

pub(super) fn diagnostic_get_json(
    diagnostic: &crate::entities::diagnostic::Diagnostic,
    sections: &[String],
) -> anyhow::Result<String> {
    Ok(crate::render::json::to_entity_json(
        diagnostic,
        crate::render::markdown::diagnostic_evidence_urls(diagnostic),
        crate::render::markdown::diagnostic_next_commands(diagnostic, sections),
        crate::render::provenance::diagnostic_section_sources(diagnostic),
    )?)
}

pub(super) fn diagnostic_search_json(
    mut results: Vec<crate::entities::diagnostic::DiagnosticSearchResult>,
    total: Option<usize>,
    limit: usize,
    offset: usize,
    full: bool,
) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct DiagnosticSearchView {
        #[serde(flatten)]
        row: crate::entities::diagnostic::DiagnosticSearchResult,
        genes_total: usize,
        genes_has_more: bool,
        conditions_total: usize,
        conditions_has_more: bool,
    }
    let next_commands = crate::render::markdown::search_next_commands_diagnostic(&results);
    let results = results
        .drain(..)
        .map(|mut row| {
            let genes_total = row.genes.len();
            let conditions_total = row.conditions.len();
            if !full {
                row.genes.truncate(5);
                row.conditions.truncate(5);
            }
            DiagnosticSearchView {
                genes_has_more: row.genes.len() < genes_total,
                conditions_has_more: row.conditions.len() < conditions_total,
                genes_total,
                conditions_total,
                row,
            }
        })
        .collect::<Vec<_>>();
    let true_zero_result = results.is_empty() && total == Some(0);
    let pagination = super::super::PaginationMeta::offset(offset, limit, results.len(), total);
    let suggestions =
        true_zero_result.then(crate::render::markdown::diagnostic_zero_result_recovery_commands);
    super::super::search_json_with_meta_and_suggestions(
        results,
        pagination,
        next_commands,
        suggestions,
    )
}

pub(crate) fn render_loaded_card(
    entity: &crate::entities::diagnostic::Diagnostic,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        diagnostic_get_json(entity, sections)
    } else {
        Ok(crate::render::markdown::diagnostic_markdown(
            entity, sections,
        )?)
    }
}

pub(in crate::cli) async fn handle_search(
    args: DiagnosticSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    validate_search_args(&args)?;
    let filters = crate::entities::diagnostic::DiagnosticSearchFilters {
        source: args.source.into(),
        gene: args.gene,
        disease: args.disease,
        test_type: args.test_type,
        manufacturer: args.manufacturer,
    };
    let mut query_summary = crate::entities::diagnostic::search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = if query_summary.is_empty() {
            format!("offset={}", args.offset)
        } else {
            format!("{query_summary}, offset={}", args.offset)
        };
    }
    let page = crate::entities::diagnostic::search_page(&filters, args.limit, args.offset).await?;
    let total = page.total;
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), total);
    let text = if json {
        diagnostic_search_json(results, total, args.limit, args.offset, args.full)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::diagnostic_search_markdown_with_footer(
            &query_summary,
            &results,
            total,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}
