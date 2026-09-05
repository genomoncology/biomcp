use super::PhenotypeSearchArgs;
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(
    args: &PhenotypeSearchArgs,
) -> Result<(), crate::error::BioMcpError> {
    crate::entities::disease::validate_phenotype_search_window(args.limit, args.offset)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct PhenotypeJsonResponse {
    pagination: crate::entities::disease::PhenotypePagination,
    count: usize,
    results: Vec<crate::entities::disease::PhenotypeSearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _meta: Option<super::super::SearchJsonMeta>,
}

pub(crate) fn pagination_footer(meta: &crate::entities::disease::PhenotypePagination) -> String {
    let mut footer = if let Some((limit, offset)) = meta.next_window() {
        format!("More results available. Continue with `--limit {limit} --offset {offset}`.")
    } else {
        format!("Showing {} results (total unknown).", meta.returned)
    };
    if meta.provider_window_exhausted {
        footer.push_str(&format!(
            " Warning: additional provider matches may exist beyond the {}-result window; refine the HPO terms for different coverage.",
            meta.provider_window_limit
        ));
    }
    footer.push_str(&format!(
        " Provider window: {} raw rows received; limit {}; exhausted: {}.",
        meta.provider_raw_row_count, meta.provider_window_limit, meta.provider_window_exhausted
    ));
    footer
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
    let pagination = page.pagination;
    let text = if json {
        let mut next_commands = crate::render::markdown::search_next_commands_phenotype(&results);
        if let Some((limit, offset)) = pagination.next_window() {
            next_commands.insert(
                0,
                format!(
                    "biomcp search phenotype {} --limit {} --offset {}",
                    crate::render::markdown::shell_quote_arg(&args.terms),
                    limit,
                    offset
                ),
            );
        }
        crate::render::json::to_pretty(&PhenotypeJsonResponse {
            count: results.len(),
            results,
            pagination,
            _meta: super::super::search_meta(next_commands),
        })?
    } else {
        let footer = pagination_footer(&pagination);
        crate::render::markdown::phenotype_search_markdown_with_footer(
            &query_summary,
            &results,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}
