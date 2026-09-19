use super::{CellLineCommand, CellLineGetArgs, CellLineSearchArgs};
use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_get(
    args: CellLineGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let cell_line = crate::entities::cell_line::get(&args.id, &sections).await?;
    let text = render_loaded_card(&cell_line, &sections, json_output)?;
    Ok(CommandOutcome::stdout(text))
}

pub(crate) fn render_loaded_card(
    entity: &crate::entities::cell_line::CellLine,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        Ok(crate::render::json::to_entity_json(
            entity,
            Vec::<(&str, String)>::new(),
            crate::render::markdown::cell_line_next_commands(entity, sections),
            crate::render::provenance::cell_line_section_sources(entity),
        )?)
    } else {
        Ok(crate::render::markdown::cell_line_markdown(
            entity, sections,
        )?)
    }
}

pub(in crate::cli) async fn handle_search(
    args: CellLineSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?
        .ok_or_else(|| {
            crate::error::BioMcpError::InvalidArgument(
                "search cell-line requires a name or accession".into(),
            )
        })?;
    let fetch_limit =
        super::super::paged_fetch_limit_for("search cell-line", args.limit, args.offset, 25)?;
    let page = crate::entities::cell_line::search(&query, fetch_limit).await?;
    let total = page.total;
    let (results, observed_total) =
        super::super::paginate_results(page.results, args.offset, args.limit);
    super::super::log_pagination_truncation(observed_total, args.offset, results.len());
    // A filled provider window leaves the total unknown, so it stays None.
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), total);

    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_cell_line(&results);
        super::super::search_json_with_data_as_of(
            results,
            pagination,
            next_commands,
            page.notes.clone(),
            page.data_as_of.clone(),
            page.data_as_of_kind.clone(),
        )?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        let query_summary = if args.offset > 0 {
            format!("{query}, offset={}", args.offset)
        } else {
            query.clone()
        };
        crate::render::markdown::cell_line_search_markdown_with_footer(
            &query_summary,
            &results,
            total,
            &page.notes,
            &page.data_as_of,
            &page.data_as_of_kind,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_command(
    cmd: CellLineCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    match cmd {
        CellLineCommand::DrugResponse {
            id,
            dataset,
            limit,
            offset,
        } => {
            super::super::paged_fetch_limit(limit, offset, 100)?;
            let page = crate::entities::cell_line::pharmacodb::load_drug_response_rows(
                &id, &dataset, offset, limit,
            )
            .await?;
            super::super::log_pagination_truncation(page.matched, offset, page.rows.len());
            let text = if json {
                crate::render::json::to_pretty(&page)?
            } else {
                crate::render::markdown::pharmacodb_rows_markdown(
                    &page,
                    &format!("{} drug response (PharmacoDB)", page.subject),
                    "Compound",
                    false,
                )?
            };
            Ok(CommandOutcome::stdout(text))
        }
    }
}
