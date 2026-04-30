use super::GeneCommand;
use crate::cli::CommandOutcome;

pub(super) async fn handle_related_command(
    cmd: GeneCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        GeneCommand::Trials {
            symbol,
            limit,
            offset,
            source,
        } => {
            let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
            let filters = crate::entities::trial::TrialSearchFilters {
                biomarker: Some(symbol.clone()),
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
                    format!("biomarker={symbol}, offset={offset}")
                } else {
                    format!("biomarker={symbol}")
                };
                crate::render::markdown::trial_search_markdown(&query, &results, total)?
            }
        }
        GeneCommand::Drugs {
            symbol,
            limit,
            offset,
        } => {
            let filters = crate::entities::drug::DrugSearchFilters {
                target: Some(symbol.clone()),
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
        GeneCommand::Articles {
            symbol,
            limit,
            offset,
        } => {
            let filters = crate::entities::article::ArticleSearchFilters {
                gene: Some(symbol.clone()),
                gene_anchored: true,
                ..super::super::related_article_filters()
            };
            let query = if offset > 0 {
                format!("gene={symbol}, offset={offset}")
            } else {
                format!("gene={symbol}")
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
        GeneCommand::Pathways {
            symbol,
            limit,
            offset,
        } => {
            let fetch_limit = super::super::paged_fetch_limit(limit, offset, 25)?;
            let sections = vec!["pathways".to_string()];
            let mut gene = crate::gene::get(&symbol, &sections).await?;
            if let Some(pathways) = gene.pathways.take() {
                let fetched = pathways.into_iter().take(fetch_limit).collect::<Vec<_>>();
                let (results, observed_total) =
                    super::super::paginate_results(fetched, offset, limit);
                super::super::log_pagination_truncation(observed_total, offset, results.len());
                gene.pathways = (!results.is_empty()).then_some(results);
            }
            if json {
                crate::render::json::to_pretty(&gene)?
            } else {
                crate::render::markdown::gene_markdown(&gene, &sections)?
            }
        }
        GeneCommand::Definition { .. } | GeneCommand::External(_) => {
            unreachable!("handled by gene dispatch")
        }
    };

    Ok(CommandOutcome::stdout(text))
}
