use super::{GeneCommand, GeneGetArgs, GeneSearchArgs};
use crate::cli::CommandOutcome;

pub(crate) async fn handle_get(
    args: GeneGetArgs,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    render_gene_card_outcome(
        &args.symbol,
        &sections,
        json_output,
        alias_suggestions_as_json,
    )
    .await
}

pub(crate) async fn handle_search(
    args: GeneSearchArgs,
    json: bool,
    include_metadata: bool,
) -> anyhow::Result<CommandOutcome> {
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?;
    let filters = crate::entities::gene::GeneSearchFilters {
        query,
        gene_type: args.gene_type,
        chromosome: args.chromosome,
        region: args.region,
        pathway: args.pathway,
        go_term: args.go_term,
    };
    let mut query_summary = crate::entities::gene::search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let page = crate::entities::gene::search_page(&filters, args.limit, args.offset).await?;
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), page.total);
    let footer = super::super::pagination_footer_offset(&pagination);
    let human = crate::render::markdown::gene_search_markdown_with_footer(
        &query_summary,
        &results,
        &footer,
    )?;
    if !json && !include_metadata {
        return Ok(CommandOutcome::stdout(human));
    }
    let next_commands = crate::render::markdown::search_next_commands_gene(&results);
    let structured = super::super::search_json_with_meta(results, pagination, next_commands)?;
    let text = if json { structured.clone() } else { human };
    Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
}

pub(crate) async fn handle_command(
    cmd: GeneCommand,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match cmd {
        GeneCommand::Definition { symbol } => {
            render_gene_card_outcome(
                &symbol,
                super::super::empty_sections(),
                json,
                alias_suggestions_as_json,
            )
            .await
        }
        GeneCommand::Cspec(args) => match args.command {
            Some(super::CspecCommand::Document { .. }) => {
                unreachable!("raw CSpec documents bypass text rendering")
            }
            None => {
                let gene = args.gene.ok_or_else(|| {
                    crate::error::BioMcpError::InvalidArgument(
                        "gene cspec requires a gene symbol or the document subcommand".into(),
                    )
                })?;
                super::cspec::handle(
                    gene,
                    args.version,
                    args.capture_id,
                    args.files,
                    args.offset,
                    args.limit,
                    json,
                )
                .await
            }
        },
        GeneCommand::External(args) => {
            let symbol = args.join(" ");
            render_gene_card_outcome(
                &symbol,
                super::super::empty_sections(),
                json,
                alias_suggestions_as_json,
            )
            .await
        }
        other => super::related::handle_related_command(other, json).await,
    }
}

pub(super) async fn render_gene_card_outcome(
    symbol: &str,
    sections: &[String],
    json_output: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match crate::gene::get(symbol, sections).await {
        Ok(gene) => {
            let human = crate::render::markdown::gene_markdown(&gene, sections)?;
            if !json_output && !alias_suggestions_as_json {
                return Ok(CommandOutcome::stdout(human));
            }
            let workflow = gene_mechanism_workflow(&gene)?;
            let structured = crate::render::json::to_entity_json_with_suggestions_and_workflow(
                &gene,
                crate::render::markdown::gene_evidence_urls(&gene),
                crate::render::markdown::gene_next_commands(&gene, sections),
                crate::render::markdown::related_gene(&gene),
                crate::render::provenance::gene_section_sources(&gene),
                workflow,
            )?;
            let text = if json_output {
                structured.clone()
            } else {
                human
            };
            Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
        }
        Err(err @ crate::error::BioMcpError::NotFound { .. }) => {
            if let Some(outcome) = super::super::try_alias_fallback_outcome(
                symbol,
                crate::entities::discover::DiscoverType::Gene,
                json_output || alias_suggestions_as_json,
            )
            .await?
            {
                Ok(outcome)
            } else {
                Err(err.into())
            }
        }
        Err(err) => Err(err.into()),
    }
}

fn gene_mechanism_workflow(
    gene: &crate::entities::gene::Gene,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    (!gene.symbol.trim().is_empty())
        .then(|| {
            crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::MechanismPathway)
        })
        .transpose()
}
