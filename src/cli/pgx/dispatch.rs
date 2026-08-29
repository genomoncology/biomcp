use super::{PgxGetArgs, PgxSearchArgs};
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(args: &PgxSearchArgs) -> Result<(), crate::error::BioMcpError> {
    if args.limit == 0 || args.limit > 50 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    Ok(())
}

pub(in crate::cli) async fn handle_get(
    args: PgxGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let options = crate::entities::pgx::PgxGetOptions {
        sections: sections.clone(),
        limit: args.limit,
        offset: args.offset,
        full: args.full,
    };
    let pgx = crate::entities::pgx::get_with_options(&args.query, &options).await?;
    let text = render_loaded_card(&pgx, &sections, json_output)?;
    Ok(CommandOutcome::stdout(text))
}

pub(crate) fn render_loaded_card(
    entity: &crate::entities::pgx::Pgx,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        Ok(crate::render::json::to_entity_json(
            entity,
            crate::render::markdown::pgx_evidence_urls(entity),
            crate::render::markdown::related_pgx(entity),
            crate::render::provenance::pgx_section_sources(entity),
        )?)
    } else {
        Ok(crate::render::markdown::pgx_markdown(entity, sections)?)
    }
}

pub(in crate::cli) async fn handle_search(
    args: PgxSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    validate_search_args(&args)?;
    let gene = super::super::resolve_query_input(args.gene, args.positional_query, "--gene")?;
    let filters = crate::entities::pgx::PgxSearchFilters {
        gene,
        drug: args.drug,
        cpic_level: args.cpic_level,
        pgx_testing: args.pgx_testing,
        evidence: args.evidence,
    };
    let mut query_summary = crate::entities::pgx::search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let page = crate::entities::pgx::search_page(&filters, args.limit, args.offset).await?;
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), page.total);
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_pgx(
            &results,
            filters.gene.as_deref(),
            filters.drug.as_deref(),
        );
        super::super::search_json_with_meta(results, pagination, next_commands)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::pgx_search_markdown_with_footer(&query_summary, &results, &footer)?
    };
    Ok(CommandOutcome::stdout(text))
}
