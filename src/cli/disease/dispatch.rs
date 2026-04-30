use super::{DiseaseCommand, DiseaseGetArgs, DiseaseSearchArgs};
use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_get(
    args: DiseaseGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let disease = crate::entities::disease::get(&args.name_or_id, &sections).await?;
    let text = if json_output {
        crate::render::json::to_entity_json_with_suggestions(
            &disease,
            crate::render::markdown::disease_evidence_urls(&disease),
            crate::render::markdown::disease_next_commands(&disease, &sections),
            crate::render::markdown::related_disease(&disease),
            crate::render::provenance::disease_section_sources(&disease),
        )?
    } else {
        crate::render::markdown::disease_markdown(&disease, &sections)?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_search(
    args: DiseaseSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?;
    let filters = crate::entities::disease::DiseaseSearchFilters {
        query,
        source: args.source,
        inheritance: args.inheritance,
        phenotype: args.phenotype,
        onset: args.onset,
    };
    let mut query_summary = crate::entities::disease::search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let mut page = crate::entities::disease::search_page(&filters, args.limit, args.offset).await?;
    let mut fallback_used = false;
    if page.results.is_empty()
        && !args.no_fallback
        && let Some(fallback_page) =
            crate::entities::disease::fallback_search_page(&filters, args.limit, args.offset)
                .await?
    {
        page = fallback_page;
        fallback_used = true;
    }
    let results = page.results;
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), page.total);
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_disease(&results);
        let workflow = disease_search_workflow(results.first()).await?;
        disease_search_json(results, pagination, fallback_used, next_commands, workflow)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::disease_search_markdown_with_footer(
            filters.query.as_deref().map(str::trim).unwrap_or_default(),
            &query_summary,
            &results,
            fallback_used,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}

async fn disease_search_workflow(
    top_result: Option<&crate::entities::disease::DiseaseSearchResult>,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    let Some(top_result) = top_result else {
        return Ok(None);
    };
    let disease_name = top_result.name.trim();
    if disease_name.is_empty() {
        return Ok(None);
    }

    match crate::workflow_ladders::probe_workflow(
        crate::workflow_ladders::Workflow::MutationCatalog,
        Box::pin(disease_has_pathogenic_variant_catalog(disease_name)),
    )
    .await?
    {
        crate::workflow_ladders::WorkflowProbeOutcome::Triggered(meta) => return Ok(Some(meta)),
        crate::workflow_ladders::WorkflowProbeOutcome::Unavailable => return Ok(None),
        crate::workflow_ladders::WorkflowProbeOutcome::NotTriggered => {}
    }

    match crate::workflow_ladders::probe_workflow(
        crate::workflow_ladders::Workflow::TrialRecruitment,
        Box::pin(disease_has_recruiting_trials(disease_name)),
    )
    .await?
    {
        crate::workflow_ladders::WorkflowProbeOutcome::Triggered(meta) => Ok(Some(meta)),
        crate::workflow_ladders::WorkflowProbeOutcome::NotTriggered
        | crate::workflow_ladders::WorkflowProbeOutcome::Unavailable => Ok(None),
    }
}

async fn disease_has_pathogenic_variant_catalog(
    disease_name: &str,
) -> Result<bool, crate::error::BioMcpError> {
    let filters = pathogenic_variant_catalog_filters(disease_name);
    let page = crate::entities::variant::search_page(&filters, 3, 0).await?;
    Ok(page.results.len() >= 3 || page.total.unwrap_or(page.results.len()) >= 3)
}

fn pathogenic_variant_catalog_filters(
    disease_name: &str,
) -> crate::entities::variant::VariantSearchFilters {
    crate::entities::variant::VariantSearchFilters {
        gene: None,
        hgvsp: None,
        hgvsc: None,
        rsid: None,
        protein_alias: None,
        significance: Some("pathogenic".to_string()),
        max_frequency: None,
        min_cadd: None,
        consequence: None,
        review_status: None,
        population: None,
        revel_min: None,
        gerp_min: None,
        tumor_site: None,
        condition: Some(disease_name.to_string()),
        impact: None,
        lof: false,
        has: None,
        missing: None,
        therapy: None,
    }
}

async fn disease_has_recruiting_trials(
    disease_name: &str,
) -> Result<bool, crate::error::BioMcpError> {
    let filters = recruiting_trial_filters(disease_name);
    let page = crate::entities::trial::search_page(&filters, 1, 0, None).await?;
    Ok(!page.results.is_empty() || page.total.unwrap_or(0) > 0)
}

fn recruiting_trial_filters(disease_name: &str) -> crate::entities::trial::TrialSearchFilters {
    crate::entities::trial::TrialSearchFilters {
        condition: Some(disease_name.to_string()),
        status: Some("recruiting".to_string()),
        source: crate::entities::trial::TrialSource::ClinicalTrialsGov,
        ..Default::default()
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::{pathogenic_variant_catalog_filters, recruiting_trial_filters};

    #[test]
    fn disease_workflow_probe_filters_use_top_result_name_and_bounded_queries() {
        let variant_filters = pathogenic_variant_catalog_filters("tuberculosis");
        assert_eq!(variant_filters.condition.as_deref(), Some("tuberculosis"));
        assert_eq!(variant_filters.significance.as_deref(), Some("pathogenic"));
        assert!(variant_filters.gene.is_none());

        let trial_filters = recruiting_trial_filters("tuberculosis");
        assert_eq!(trial_filters.condition.as_deref(), Some("tuberculosis"));
        assert_eq!(trial_filters.status.as_deref(), Some("recruiting"));
        assert!(matches!(
            trial_filters.source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        ));
    }
}

pub(in crate::cli) async fn handle_command(
    cmd: DiseaseCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        DiseaseCommand::Trials {
            name,
            limit,
            offset,
            source,
        } => {
            let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
            let filters = crate::entities::trial::TrialSearchFilters {
                condition: Some(name.clone()),
                source: trial_source,
                ..Default::default()
            };
            let (results, total) = crate::entities::trial::search(&filters, limit, offset).await?;
            if let Some(total) = total {
                super::super::log_pagination_truncation(total as usize, offset, results.len());
            }
            if json {
                #[derive(serde::Serialize)]
                struct SearchResponse {
                    count: usize,
                    total: Option<u32>,
                    results: Vec<crate::entities::trial::TrialSearchResult>,
                }

                crate::render::json::to_pretty(&SearchResponse {
                    count: results.len(),
                    total,
                    results,
                })?
            } else {
                let query = if offset > 0 {
                    format!("condition={name}, offset={offset}")
                } else {
                    format!("condition={name}")
                };
                crate::render::markdown::trial_search_markdown(&query, &results, total)?
            }
        }
        DiseaseCommand::Articles {
            name,
            limit,
            offset,
        } => {
            let filters = crate::entities::article::ArticleSearchFilters {
                disease: Some(name.clone()),
                ..super::super::related_article_filters()
            };
            let query = if offset > 0 {
                format!("disease={name}, offset={offset}")
            } else {
                format!("disease={name}")
            };
            let fetch_limit = super::super::paged_fetch_limit(limit, offset, 50)?;
            let rows = crate::entities::article::search(&filters, fetch_limit).await?;
            let (results, total) = super::super::paginate_results(rows, offset, limit);
            super::super::log_pagination_truncation(total, offset, results.len());
            if json {
                #[derive(serde::Serialize)]
                struct SearchResponse {
                    total: Option<usize>,
                    count: usize,
                    results: Vec<crate::entities::article::ArticleSearchResult>,
                }

                crate::render::json::to_pretty(&SearchResponse {
                    total: Some(total),
                    count: results.len(),
                    results,
                })?
            } else {
                crate::render::markdown::article_search_markdown_with_footer_and_context(
                    &query,
                    &results,
                    "",
                    &filters,
                    crate::render::markdown::ArticleSearchRenderContext {
                        source_filter: crate::entities::article::ArticleSourceFilter::All,
                        semantic_scholar_enabled:
                            crate::entities::article::semantic_scholar_search_enabled(
                                &filters,
                                crate::entities::article::ArticleSourceFilter::All,
                            ),
                        note: None,
                        debug_plan: None,
                        exact_entity_commands: &[],
                        source_status: &[],
                    },
                )?
            }
        }
        DiseaseCommand::Drugs {
            name,
            limit,
            offset,
        } => {
            let filters = crate::entities::drug::DrugSearchFilters {
                indication: Some(name.clone()),
                ..Default::default()
            };
            let mut query_summary = crate::entities::drug::search_query_summary(&filters);
            if offset > 0 {
                query_summary = format!("{query_summary}, offset={offset}");
            }
            let fetch_limit = super::super::paged_fetch_limit(limit, offset, 50)?;
            let rows = crate::entities::drug::search(&filters, fetch_limit).await?;
            let (results, total) = super::super::paginate_results(rows, offset, limit);
            super::super::log_pagination_truncation(total, offset, results.len());
            if json {
                #[derive(serde::Serialize)]
                struct SearchResponse {
                    total: Option<usize>,
                    count: usize,
                    results: Vec<crate::entities::drug::DrugSearchResult>,
                }

                crate::render::json::to_pretty(&SearchResponse {
                    total: Some(total),
                    count: results.len(),
                    results,
                })?
            } else {
                crate::render::markdown::drug_search_markdown(&query_summary, &results)?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}

#[derive(serde::Serialize)]
pub(super) struct DiseaseSearchMeta {
    next_commands: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    fallback_used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ladder: Vec<crate::workflow_ladders::WorkflowLadderStep>,
}

#[derive(serde::Serialize)]
pub(super) struct DiseaseSearchJsonResponse<T: serde::Serialize> {
    pagination: crate::cli::PaginationMeta,
    count: usize,
    results: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    _meta: Option<DiseaseSearchMeta>,
}

pub(super) fn disease_search_json(
    results: Vec<crate::entities::disease::DiseaseSearchResult>,
    pagination: crate::cli::PaginationMeta,
    fallback_used: bool,
    next_commands: Vec<String>,
    workflow: Option<crate::workflow_ladders::WorkflowMeta>,
) -> anyhow::Result<String> {
    let count = results.len();
    let meta = crate::cli::search_meta_with_workflow(next_commands, None, workflow).map(|meta| {
        DiseaseSearchMeta {
            next_commands: meta.next_commands,
            fallback_used,
            workflow: meta.workflow,
            ladder: meta.ladder,
        }
    });
    crate::render::json::to_pretty(&DiseaseSearchJsonResponse {
        pagination,
        count,
        results,
        _meta: meta,
    })
    .map_err(Into::into)
}
