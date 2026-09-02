use super::{DiseaseCommand, DiseaseGetArgs, DiseaseSearchArgs};
use crate::cli::CommandOutcome;

const DISEASE_LABEL_FALLBACK_NOTICE: &str = "Disease label unavailable; using the requested term.";

pub(super) fn validate_related_limit(
    command_name: &str,
    limit: usize,
    offset: usize,
) -> Result<(), crate::error::BioMcpError> {
    super::super::paged_fetch_limit_for(command_name, limit, offset, 50).map(|_| ())
}

pub(in crate::cli) async fn handle_get(
    args: DiseaseGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (name_or_id, raw_sections) = match args.name_or_id.as_deref() {
        Some(name_or_id) => (name_or_id, args.args.as_slice()),
        None => (
            args.args.first().map(String::as_str).unwrap_or_default(),
            args.args.get(1..).unwrap_or_default(),
        ),
    };
    let (sections, json_override) = super::super::extract_json_from_sections(raw_sections);
    let json_output = json || json_override;
    let context = crate::entities::disease::get_with_context(name_or_id, &sections).await?;
    let text = render_loaded_card_with_context(
        &context.disease,
        &sections,
        json_output,
        context.used_requested_label,
    )?;
    Ok(CommandOutcome::stdout(text))
}

pub(crate) fn render_loaded_card(
    disease: &crate::entities::disease::Disease,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    if json_output {
        Ok(crate::render::json::to_entity_json_with_suggestions(
            disease,
            crate::render::markdown::disease_evidence_urls(disease),
            crate::render::markdown::disease_next_commands(disease, sections),
            crate::render::markdown::related_disease(disease),
            crate::render::provenance::disease_section_sources(disease),
        )?)
    } else {
        Ok(crate::render::markdown::disease_markdown(
            disease, sections,
        )?)
    }
}

fn render_loaded_card_with_context(
    disease: &crate::entities::disease::Disease,
    sections: &[String],
    json_output: bool,
    used_requested_label: bool,
) -> anyhow::Result<String> {
    if !used_requested_label {
        return render_loaded_card(disease, sections, json_output);
    }
    if json_output {
        let mut card: serde_json::Value =
            serde_json::from_str(&render_loaded_card(disease, sections, true)?)?;
        card["_meta"]["identity_notice"] =
            serde_json::Value::String(DISEASE_LABEL_FALLBACK_NOTICE.to_string());
        Ok(crate::render::json::to_pretty(&card)?)
    } else {
        Ok(
            crate::render::markdown::disease_markdown_with_identity_notice(
                disease,
                sections,
                Some(DISEASE_LABEL_FALLBACK_NOTICE),
            )?,
        )
    }
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
        let workflow = disease_search_workflow(results.first())?;
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

fn disease_search_workflow(
    top_result: Option<&crate::entities::disease::DiseaseSearchResult>,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    let Some(top_result) = top_result else {
        return Ok(None);
    };
    let disease_name = top_result.name.trim();
    if disease_name.is_empty() {
        return Ok(None);
    }

    crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::MutationCatalog).map(Some)
}

fn disease_trial_filters(
    name: &str,
    trial_source: crate::entities::trial::TrialSource,
    limit: usize,
) -> crate::entities::trial::TrialSearchFilters {
    crate::entities::trial::TrialSearchFilters {
        condition: Some(name.to_string()),
        source: trial_source,
        no_count_total: matches!(
            trial_source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        ) && limit == 1,
        ..Default::default()
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::disease_trial_filters;

    #[test]
    fn disease_trials_limit_one_avoids_ctgov_total_count() {
        let fast_filters = disease_trial_filters(
            "Phelan-McDermid Syndrome",
            crate::entities::trial::TrialSource::ClinicalTrialsGov,
            1,
        );
        assert_eq!(
            fast_filters.condition.as_deref(),
            Some("Phelan-McDermid Syndrome")
        );
        assert!(fast_filters.no_count_total);

        let broader_filters = disease_trial_filters(
            "Phelan-McDermid Syndrome",
            crate::entities::trial::TrialSource::ClinicalTrialsGov,
            2,
        );
        assert!(!broader_filters.no_count_total);

        let nci_filters = disease_trial_filters(
            "Phelan-McDermid Syndrome",
            crate::entities::trial::TrialSource::NciCts,
            1,
        );
        assert!(!nci_filters.no_count_total);
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
            validate_related_limit("disease trials", limit, offset)?;
            let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
            let filters = disease_trial_filters(&name, trial_source, limit);
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
            validate_related_limit("disease articles", limit, offset)?;
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
                        warning: None,
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
            validate_related_limit("disease drugs", limit, offset)?;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workflow_playbook: Option<String>,
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
            workflow_rationale: meta.workflow_rationale,
            workflow_playbook: meta.workflow_playbook,
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
