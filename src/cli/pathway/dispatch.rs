use super::{PathwayCommand, PathwayGetArgs, PathwaySearchArgs};
use crate::cli::CommandOutcome;
use futures::StreamExt;
use tracing::{debug, warn};

pub(in crate::cli) async fn handle_get(
    args: PathwayGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let pathway = crate::entities::pathway::get(&args.id, &sections).await?;
    let text = if json_output {
        crate::render::json::to_entity_json(
            &pathway,
            crate::render::markdown::pathway_evidence_urls(&pathway),
            crate::render::markdown::related_pathway(&pathway),
            crate::render::provenance::pathway_section_sources(&pathway),
        )?
    } else {
        crate::render::markdown::pathway_markdown(&pathway, &sections)?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_search(
    args: PathwaySearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?;
    let filters = crate::entities::pathway::PathwaySearchFilters {
        query,
        pathway_type: args.pathway_type,
        top_level: args.top_level,
    };
    let fetch_limit = super::super::paged_fetch_limit(args.limit, args.offset, 25)?;
    let mut query_summary = crate::entities::pathway::search_query_summary(&filters);
    if args.offset > 0 {
        query_summary = if query_summary.is_empty() {
            format!("offset={}", args.offset)
        } else {
            format!("{query_summary}, offset={}", args.offset)
        };
    }
    let (rows, total) =
        crate::entities::pathway::search_with_filters(&filters, fetch_limit).await?;
    let (results, observed_total) = super::super::paginate_results(rows, args.offset, args.limit);
    super::super::log_pagination_truncation(observed_total, args.offset, results.len());
    let total = total.or(Some(observed_total));
    let pagination =
        super::super::PaginationMeta::offset(args.offset, args.limit, results.len(), total);
    let text = if json {
        let next_commands = crate::render::markdown::search_next_commands_pathway(&results);
        super::super::search_json_with_meta(results, pagination, next_commands)?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::pathway_search_markdown_with_footer(
            &query_summary,
            &results,
            total,
            &footer,
        )?
    };
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_command(
    cmd: PathwayCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        PathwayCommand::Drugs { id, limit, offset } => {
            let fetch_limit = super::super::paged_fetch_limit(limit, offset, 50)?;
            let rows = pathway_drug_results(&id, fetch_limit).await?;
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
                let query = if offset > 0 {
                    format!("pathway={id}, offset={offset}")
                } else {
                    format!("pathway={id}")
                };
                crate::render::markdown::drug_search_markdown(&query, &results)?
            }
        }
        PathwayCommand::Articles { id, limit, offset } => {
            let pathway =
                crate::entities::pathway::get(&id, super::super::empty_sections()).await?;
            let pathway_name = pathway.name.trim();
            let keyword = if pathway_name.is_empty() {
                id.clone()
            } else {
                pathway_name.to_string()
            };
            let filters = crate::entities::article::ArticleSearchFilters {
                keyword: Some(keyword.clone()),
                ..super::super::related_article_filters()
            };
            let query = if offset > 0 {
                format!("keyword={keyword}, offset={offset}")
            } else {
                format!("keyword={keyword}")
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
        PathwayCommand::Trials {
            id,
            limit,
            offset,
            source,
        } => {
            let pathway =
                crate::entities::pathway::get(&id, super::super::empty_sections()).await?;
            let pathway_name = pathway.name.trim();
            let condition = if pathway_name.is_empty() {
                id.clone()
            } else {
                pathway_name.to_string()
            };
            let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
            let filters = crate::entities::trial::TrialSearchFilters {
                condition: Some(condition.clone()),
                source: trial_source,
                ..Default::default()
            };
            let (mut results, mut total) =
                crate::entities::trial::search(&filters, limit, offset).await?;
            let mut query = if offset > 0 {
                format!("condition={condition}, offset={offset}")
            } else {
                format!("condition={condition}")
            };

            if should_try_pathway_trial_fallback(results.len(), offset, total) {
                let pathway_with_genes =
                    crate::entities::pathway::get(&id, &["genes".to_string()]).await?;
                let fallback_limit = limit.saturating_add(offset).clamp(1, 50);

                for gene in pathway_with_genes.genes.into_iter().take(10) {
                    let gene = gene.trim().to_string();
                    if gene.is_empty() {
                        continue;
                    }

                    let fallback_filters = crate::entities::trial::TrialSearchFilters {
                        biomarker: Some(gene.clone()),
                        source: trial_source,
                        ..Default::default()
                    };

                    match crate::entities::trial::search(&fallback_filters, fallback_limit, 0).await
                    {
                        Ok((fallback_rows, fallback_total)) if !fallback_rows.is_empty() => {
                            debug!(
                                pathway_id = %id,
                                fallback_gene = %gene,
                                "Pathway trial condition search returned no rows; using biomarker fallback",
                            );
                            results = fallback_rows.into_iter().skip(offset).take(limit).collect();
                            total = fallback_total;
                            query = if offset > 0 {
                                format!(
                                    "condition={condition}, fallback_biomarker={gene}, offset={offset}"
                                )
                            } else {
                                format!("condition={condition}, fallback_biomarker={gene}")
                            };
                            break;
                        }
                        Ok(_) => {}
                        Err(err) => {
                            warn!(
                                pathway_id = %id,
                                fallback_gene = %gene,
                                "Pathway trial fallback failed: {err}"
                            );
                        }
                    }
                }
            }

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
                crate::render::markdown::trial_search_markdown(&query, &results, total)?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}

pub(super) fn should_try_pathway_trial_fallback(
    results_len: usize,
    offset: usize,
    total: Option<u32>,
) -> bool {
    if results_len != 0 || offset > 0 {
        return false;
    }
    total.is_none_or(|value| value == 0)
}

pub(super) async fn pathway_drug_results(
    id: &str,
    fetch_limit: usize,
) -> Result<Vec<crate::entities::drug::DrugSearchResult>, crate::error::BioMcpError> {
    let sections = vec!["genes".to_string()];
    let pathway = crate::entities::pathway::get(id, &sections).await?;

    let search_limit = fetch_limit.clamp(1, 10);
    let mut stream = futures::stream::iter(pathway.genes.into_iter().map(|gene| async move {
        let filters = crate::entities::drug::DrugSearchFilters {
            target: Some(gene.clone()),
            ..Default::default()
        };
        let result = crate::entities::drug::search(&filters, search_limit).await;
        (gene, result)
    }))
    .buffer_unordered(5);

    let mut results: Vec<Vec<crate::entities::drug::DrugSearchResult>> = Vec::new();
    let mut attempted: usize = 0;
    let mut failures: usize = 0;
    while let Some((gene, next)) = stream.next().await {
        attempted += 1;
        match next {
            Ok(rows) => results.push(rows),
            Err(err) => {
                failures += 1;
                warn!(gene = %gene, "pathway drug lookup failed: {err}");
            }
        }
    }

    if attempted > 0 && failures.saturating_mul(2) > attempted {
        return Err(crate::error::BioMcpError::Api {
            api: "pathway-drugs".into(),
            message: format!(
                "Failed to resolve {failures} of {attempted} pathway gene target lookups while collecting drugs"
            ),
        });
    }

    let mut out: Vec<crate::entities::drug::DrugSearchResult> = Vec::new();
    for rows in results {
        for row in rows {
            if out.iter().any(|v| v.name.eq_ignore_ascii_case(&row.name)) {
                continue;
            }
            out.push(row);
            if out.len() >= fetch_limit {
                return Ok(out);
            }
        }
    }

    Ok(out)
}
