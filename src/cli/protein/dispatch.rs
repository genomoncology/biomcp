use super::{ProteinCommand, ProteinGetArgs, ProteinSearchArgs};
use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_get(
    args: ProteinGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let protein = crate::entities::protein::get(&args.accession, &sections).await?;
    let text = if json_output {
        let evidence_urls = crate::render::markdown::protein_evidence_urls(&protein);
        let next_commands = crate::render::markdown::related_protein(&protein, &sections);
        let section_sources = crate::render::provenance::protein_section_sources(&protein);
        let mut value = crate::render::json::to_entity_json_value(
            &protein,
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
        crate::render::json::to_pretty(&value)?
    } else {
        crate::render::markdown::protein_markdown(&protein, &sections)?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_search(
    args: ProteinSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?
        .unwrap_or_default();
    if args
        .next_page
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && args.offset > 0
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--next-page cannot be used together with --offset".into(),
        )
        .into());
    }
    let mut query_summary = crate::entities::protein::search_query_summary(
        &query,
        args.reviewed,
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
        args.reviewed,
        args.disease.as_deref(),
        args.existence,
    )
    .await?;
    let results = page.results;
    let pagination = super::super::PaginationMeta::cursor(
        args.offset,
        args.limit,
        results.len(),
        page.total,
        page.next_page_token,
    );
    let text = if json {
        super::super::search_json(results, pagination)?
    } else {
        let footer = super::super::pagination_footer_cursor(&pagination);
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
