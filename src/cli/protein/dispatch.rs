use super::{ProteinCommand, ProteinGetArgs, ProteinSearchArgs};
use crate::cli::CommandOutcome;

pub(super) fn validate_search_args(
    args: &ProteinSearchArgs,
) -> Result<(), crate::error::BioMcpError> {
    if args.limit == 0 || args.limit > 100 {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit for search protein must be 1-100".into(),
        ));
    }
    if args
        .next_page
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && args.offset > 0
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--next-page cannot be used together with --offset".into(),
        ));
    }
    Ok(())
}

pub(in crate::cli) async fn handle_get(
    args: ProteinGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let protein = crate::entities::protein::get(&args.accession, &sections).await?;
    let text = render_loaded_card(&protein, &sections, json_output)?;
    Ok(CommandOutcome::stdout(text))
}

pub(crate) fn render_loaded_card(
    protein: &crate::entities::protein::Protein,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        let evidence_urls = crate::render::markdown::protein_evidence_urls(protein);
        let next_commands = crate::render::markdown::related_protein(protein, sections);
        let section_sources = crate::render::provenance::protein_section_sources(protein);
        let mut value = crate::render::json::to_entity_json_value(
            protein,
            evidence_urls,
            next_commands,
            section_sources,
        )?;
        let provenance_urls = value
            .get("_meta")
            .and_then(|meta| meta.get("evidence_urls"))
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "_provenance".to_string(),
                serde_json::json!({ "evidence_urls": provenance_urls }),
            );
        }
        Ok(crate::render::json::to_pretty(&value)?)
    } else {
        Ok(crate::render::markdown::protein_markdown(
            protein, sections,
        )?)
    }
}

pub(in crate::cli) async fn handle_search(
    args: ProteinSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    validate_search_args(&args)?;
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?
        .unwrap_or_default();
    let mut query_summary = crate::entities::protein::search_query_summary(
        &query,
        args.include_unreviewed,
        args.disease.as_deref(),
        args.existence,
        args.all_species,
    );
    if args.offset > 0 {
        query_summary = if query_summary.is_empty() {
            format!("offset={}", args.offset)
        } else {
            format!("{query_summary}, offset={}", args.offset)
        };
    }
    let page = crate::entities::protein::search_page(
        &query,
        args.limit,
        args.offset,
        args.next_page,
        args.all_species,
        args.include_unreviewed,
        args.disease.as_deref(),
        args.existence,
    )
    .await?;
    let results = page.results;
    #[derive(serde::Serialize)]
    struct ProteinPagination {
        offset: usize,
        limit: usize,
        returned: usize,
        total: Option<usize>,
        has_more: bool,
        next_page_token: Option<String>,
        next_offset: Option<usize>,
    }
    let pagination = ProteinPagination {
        offset: args.offset,
        limit: args.limit,
        returned: results.len(),
        total: page.total,
        has_more: page.has_more,
        next_page_token: page.next_page_token,
        next_offset: page
            .has_more
            .then(|| args.offset.saturating_add(results.len())),
    };
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_protein(&results);
        let count = results.len();
        crate::render::json::to_pretty(&serde_json::json!({
            "pagination": pagination,
            "count": count,
            "results": results,
            "_meta": super::super::search_meta(next_commands),
        }))?
    } else {
        let footer = crate::render::markdown::pagination_footer(
            crate::render::markdown::PaginationFooterMode::Offset,
            pagination.offset,
            pagination.limit,
            pagination.returned,
            pagination.total,
            None,
        );
        crate::render::markdown::protein_search_markdown_with_footer(
            &query_summary,
            &results,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_command(
    cmd: ProteinCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        ProteinCommand::Structures {
            accession,
            limit,
            offset,
        } => {
            let sections = vec!["structures".to_string()];
            let protein = crate::entities::protein::get_with_structure_limit(
                &accession,
                &sections,
                Some(limit),
                Some(offset),
            )
            .await?;
            if json {
                crate::render::json::to_pretty(&protein)?
            } else {
                crate::render::markdown::protein_markdown(&protein, &sections)?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}
