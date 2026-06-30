use super::PhenotypeSearchArgs;
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(
    args: &PhenotypeSearchArgs,
) -> Result<(), crate::error::BioMcpError> {
    if args.limit == 0 || args.limit > 50 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    Ok(())
}

pub(in crate::cli) async fn handle_search(
    args: PhenotypeSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    validate_search_args(&args)?;
    let mut query_summary = args.terms.trim().to_string();
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let page =
        crate::entities::disease::search_phenotype_page(&args.terms, args.limit, args.offset)
            .await?;
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), page.total);
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_phenotype(&results);
        super::super::search_json_with_meta(results, pagination, next_commands)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::phenotype_search_markdown_with_footer(
            &query_summary,
            &results,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}
