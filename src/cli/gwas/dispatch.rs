use super::GwasSearchArgs;
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(args: &GwasSearchArgs) -> Result<(), crate::error::BioMcpError> {
    crate::entities::variant::validate_gwas_window(args.limit, args.offset)?;
    crate::entities::variant::validate_gwas_p_value(args.p_value)
}

#[derive(serde::Serialize)]
struct GwasJsonResponse {
    count: usize,
    results: Vec<crate::entities::variant::VariantGwasAssociation>,
    _meta: GwasJsonMeta,
}

#[derive(serde::Serialize)]
struct GwasJsonMeta {
    pagination: crate::entities::variant::GwasPagination,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    next_commands: Vec<String>,
}

fn pagination_footer(meta: &crate::entities::variant::GwasPagination) -> String {
    if meta.truncated_by_provider_budget {
        return "More GWAS rows may exist, but BioMCP's 50-row provider budget was reached. Narrow the filters; no next offset is available.".into();
    }
    if let Some(next) = meta.next_offset {
        return format!("More results available. Continue with `--offset {next}`.");
    }
    String::new()
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
        p_value: args.p_value,
    };
    let mut query_summary = crate::entities::variant::gwas_search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let page =
        crate::entities::variant::search_gwas_page(&filters, args.limit, args.offset).await?;
    let results = page.results;
    let pagination = page.pagination;
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_gwas(&results);
        crate::render::json::to_pretty(&GwasJsonResponse {
            count: results.len(),
            results,
            _meta: GwasJsonMeta {
                pagination,
                next_commands,
            },
        })?
    } else {
        let footer = pagination_footer(&pagination);
        crate::render::markdown::gwas_search_markdown_with_footer(
            &query_summary,
            &results,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pagination(
        returned: usize,
        has_more: bool,
        next_offset: Option<usize>,
        truncated_by_provider_budget: bool,
    ) -> crate::entities::variant::GwasPagination {
        crate::entities::variant::GwasPagination {
            limit: 10,
            offset: 40,
            returned,
            has_more,
            next_offset,
            truncated_by_provider_budget,
        }
    }

    #[test]
    fn gwas_pagination_json_has_only_the_followable_contract() {
        let response = GwasJsonResponse {
            count: 10,
            results: Vec::new(),
            _meta: GwasJsonMeta {
                pagination: pagination(10, false, None, true),
                next_commands: Vec::new(),
            },
        };
        let value = serde_json::to_value(response).expect("serialize GWAS response");
        assert_eq!(
            value["_meta"]["pagination"],
            serde_json::json!({
                "limit": 10,
                "offset": 40,
                "returned": 10,
                "has_more": false,
                "next_offset": null,
                "truncated_by_provider_budget": true
            })
        );
        assert!(value["_meta"].get("next_commands").is_none());
    }

    #[test]
    fn gwas_pagination_human_guidance_distinguishes_all_three_states() {
        assert_eq!(
            pagination_footer(&pagination(10, false, None, true)),
            "More GWAS rows may exist, but BioMCP's 50-row provider budget was reached. Narrow the filters; no next offset is available."
        );
        assert_eq!(
            pagination_footer(&pagination(10, true, Some(20), false)),
            "More results available. Continue with `--offset 20`."
        );
        assert_eq!(pagination_footer(&pagination(3, false, None, false)), "");
    }
}
