use super::GwasSearchArgs;
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(args: &GwasSearchArgs) -> Result<(), crate::error::BioMcpError> {
    if args.limit == 0 || args.limit > 50 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    crate::entities::variant::validate_gwas_p_value(args.p_value)
}

pub(in crate::cli) async fn handle_search(
    args: GwasSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    validate_search_args(&args)?;
    let gene = super::super::resolve_query_input(args.gene, args.positional_query, "--gene")?;
    let filters = crate::entities::variant::GwasSearchFilters {
        gene,
        trait_query: args.trait_query,
        region: args.region,
        p_value: args.p_value,
    };
    let mut query_summary = crate::entities::variant::gwas_search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let page =
        crate::entities::variant::search_gwas_page(&filters, args.limit, args.offset).await?;
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), page.total);
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_gwas(&results);
        super::super::search_json_with_meta(results, pagination, next_commands)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::gwas_search_markdown_with_footer(
            &query_summary,
            &results,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}
