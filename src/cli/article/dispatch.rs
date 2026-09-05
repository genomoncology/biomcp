pub(super) use super::ArticleSuggestion;
use super::render::{render_loaded_card, render_loaded_card_with_summary};
pub(super) use super::workflow::article_entity_suggestion;
#[cfg(test)]
use super::workflow::article_follow_up_workflow;
use super::{
    ArticleCommand, ArticleGetArgs, ArticleSearchArgs, ArticleSearchCompactResult,
    ArticleSearchDetail, ArticleSearchJsonMeta, ArticleSearchJsonRows, article_search_json_meta,
    article_search_warnings,
};
use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_get(
    args: ArticleGetArgs,
    json: bool,
    include_metadata: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let (sections, pdf_from_sections) = super::fulltext_view::extract_pdf(&sections);
    let (sections, outline, lines) =
        super::fulltext_view::extract_controls(&sections, args.outline, args.lines)?;
    super::fulltext_view::validate_controls(&sections, outline, lines.as_deref())?;
    super::export::validate_request(args.out.is_some(), &sections, outline, lines.as_deref())?;
    let json_output = json || json_override;
    if super::assets::article_asset_route(&sections) && (args.pdf || pdf_from_sections) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--pdf requires the fulltext section (example: biomcp get article 22663011 fulltext --pdf)"
                .into(),
        )
        .into());
    }

    let asset_destination =
        super::assets::AssetOutput::from_paths(args.output.as_deref(), args.out.as_deref());
    if let Some(outcome) = super::assets::handle_asset_get(
        &args.id,
        &sections,
        json_output,
        asset_destination,
        &args.asset_view,
        args.asset_limit,
        args.asset_offset,
    )
    .await?
    {
        return Ok(outcome);
    }

    let article = crate::entities::article::get(
        &args.id,
        &sections,
        crate::entities::article::ArticleGetOptions {
            allow_pdf: args.pdf || pdf_from_sections,
            include_asset_summary: true,
        },
    )
    .await?;
    super::export::export_fulltext(&article, args.out.as_deref()).await?;
    if let Some(response) = super::fulltext_view::requested_response(
        &article,
        &args.id,
        &sections,
        outline,
        lines.as_deref(),
        json_output,
    )? {
        return Ok(CommandOutcome::stdout(response));
    }
    if !json_output && !include_metadata {
        return Ok(CommandOutcome::stdout(render_loaded_card(
            &article, &sections, false,
        )?));
    }
    let fulltext_summary = super::fulltext_view::article_summary(&article)?;
    let structured = render_loaded_card_with_summary(&article, &sections, true, fulltext_summary)?;
    let text = if json_output {
        structured.clone()
    } else {
        render_loaded_card_with_summary(&article, &sections, false, fulltext_summary)?
    };
    Ok(CommandOutcome::stdout(text).with_metadata_json(structured))
}

#[cfg(test)]
mod workflow_tests;
pub(super) fn is_exact_article_keyword_lookup_eligible(
    filters: &crate::entities::article::ArticleSearchFilters,
) -> bool {
    let Some(keyword) = filters
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|keyword| !keyword.is_empty())
    else {
        return false;
    };

    if filters
        .gene
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || filters
            .disease
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || filters
            .drug
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    {
        return false;
    }

    matches!(super::workflow::article_keyword_token_count(keyword), 1..=3)
}

#[derive(Debug, Clone)]
pub(super) struct ArticleSearchRequest {
    pub(super) filters: crate::entities::article::ArticleSearchFilters,
    pub(super) source_filter: crate::entities::article::ArticleSourceFilter,
    pub(super) limit: usize,
    pub(super) offset: usize,
    pub(super) sort: crate::entities::article::ArticleSort,
    pub(super) ranking: crate::entities::article::ArticleRankingOptions,
    pub(super) backend_plan: crate::entities::article::BackendPlan,
    pub(super) exact_keyword_lookup: Option<String>,
}

pub(super) fn article_search_request(
    args: ArticleSearchArgs,
) -> Result<ArticleSearchRequest, crate::error::BioMcpError> {
    let (date_from, date_to) = super::workflow::resolved_article_date_bounds(&args);
    let disease = super::super::normalize_cli_tokens(args.disease);
    let drug = super::super::normalize_cli_tokens(args.drug);
    let author = super::super::normalize_cli_tokens(args.author);
    let keyword = super::super::resolve_query_input(
        super::super::normalize_cli_tokens(args.keyword),
        args.positional_query,
        "--keyword/--query",
    )?;
    let journal = super::super::normalize_cli_tokens(args.journal);
    let sort = crate::entities::article::ArticleSort::from_flag(&args.sort)?;
    let source_filter = crate::entities::article::ArticleSourceFilter::from_flag(&args.source)?;
    let ranking = crate::entities::article::ArticleRankingOptions::from_inputs(
        args.ranking_mode.as_deref(),
        args.weight_semantic,
        args.weight_lexical,
        args.weight_citations,
        args.weight_position,
    )?;
    let gene = args.gene.map(|value| value.trim().to_string());
    let gene_anchored = gene
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && disease.is_none()
        && drug.is_none()
        && author.is_none()
        && keyword.is_none();
    let filters = crate::entities::article::ArticleSearchFilters {
        gene,
        gene_anchored,
        disease,
        drug,
        variant: None,
        author,
        keyword,
        date_from,
        date_to,
        article_type: args.article_type,
        journal,
        open_access: args.open_access,
        no_preprints: args.no_preprints,
        exclude_retracted: args.exclude_retracted || !args.include_retracted,
        max_per_source: args.max_per_source,
        sort,
        ranking,
    };

    crate::entities::article::validate_search_page_request(&filters, args.limit, source_filter)?;
    let backend_plan = crate::entities::article::plan_backends(&filters, source_filter)?;
    let exact_keyword_lookup = is_exact_article_keyword_lookup_eligible(&filters)
        .then(|| filters.keyword.clone())
        .flatten();

    Ok(ArticleSearchRequest {
        sort,
        ranking: filters.ranking.clone(),
        filters,
        source_filter,
        limit: args.limit,
        offset: args.offset,
        backend_plan,
        exact_keyword_lookup,
    })
}

pub(in crate::cli) async fn handle_search(
    args: ArticleSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let detail = if args.full {
        ArticleSearchDetail::Full
    } else {
        ArticleSearchDetail::Compact
    };
    let session_token = args.session.clone();
    if let Some(token) = session_token.as_deref() {
        if crate::sources::is_no_cache_enabled() {
            return Err(crate::error::BioMcpError::InvalidArgument(
                "--no-cache cannot be combined with --session because sessions require managed local state".into(),
            )
            .into());
        }
        super::session::validate_token(token)?;
    }
    let include_retracted = args.include_retracted;
    let debug_plan_requested = args.debug_plan;
    let request: ArticleSearchRequest = article_search_request(args)?;
    let filters = &request.filters;
    let source_filter = request.source_filter;
    let limit = request.limit;
    let offset = request.offset;

    let query = article_query_summary(filters, source_filter, include_retracted, limit, offset);

    let search_future =
        crate::entities::article::search_page(filters, limit, offset, source_filter);
    let (page, exact_entity) = if let Some(query) = request.exact_keyword_lookup.as_deref() {
        let exact_future = crate::entities::discover::resolve_exact_article_keyword_entity(query);
        let (page, exact_entity) = tokio::join!(search_future, exact_future);
        let exact_entity = match exact_entity {
            Ok(entity) => entity,
            Err(_err) => {
                tracing::warn!("Exact article keyword entity lookup unavailable");
                None
            }
        };
        (page?, exact_entity)
    } else {
        (search_future.await?, None)
    };
    let source_status = page.source_status;
    let results = page.results;
    let pagination = super::super::PaginationMeta::offset(offset, limit, results.len(), page.total);
    let _ = (request.sort, &request.ranking);
    let semantic_scholar_enabled =
        matches!(
            request.backend_plan,
            crate::entities::article::BackendPlan::Both
                | crate::entities::article::BackendPlan::SemanticScholarOnly
        ) && crate::entities::article::semantic_scholar_search_enabled(filters, source_filter);
    let debug_plan = if debug_plan_requested {
        Some(build_article_debug_plan(
            &query,
            filters,
            source_filter,
            limit,
            &results,
            &pagination,
            &source_status,
        )?)
    } else {
        None
    };
    let mut suggestions = exact_entity
        .as_ref()
        .map(article_entity_suggestion)
        .into_iter()
        .collect::<Vec<_>>();
    let exact_entity_commands = suggestions
        .iter()
        .map(|suggestion| suggestion.command.clone())
        .collect::<Vec<_>>();
    let next_commands = crate::render::markdown::search_next_commands_article(
        &results,
        filters,
        source_filter,
        &exact_entity_commands,
    );
    if let Some(token) = session_token.as_deref() {
        let session_suggestions = match crate::cache::resolve_cache_config() {
            Ok(config) => article_session_suggestions(
                &config.cache_root,
                token,
                filters,
                &results,
                &next_commands,
                super::session::current_epoch_secs(),
            ),
            Err(err) => {
                tracing::warn!("Article search session loop-breaker cache root unavailable: {err}");
                Vec::new()
            }
        };
        suggestions.extend(session_suggestions);
    }

    let article_type_note =
        crate::entities::article::article_type_limitation_note(filters, source_filter);
    let warnings = article_search_warnings(filters, &results);
    let text = if json {
        article_search_json_with_detail(
            &query,
            filters,
            source_filter,
            detail,
            semantic_scholar_enabled,
            article_type_note,
            debug_plan,
            ArticleSearchJsonPage {
                results,
                pagination,
                next_commands,
                suggestions,
                source_status: source_status.clone(),
            },
        )?
    } else {
        let footer = super::super::pagination_footer_offset(&pagination);
        crate::render::markdown::article_search_markdown_with_footer_and_context(
            &query,
            &results,
            &footer,
            filters,
            crate::render::markdown::ArticleSearchRenderContext {
                source_filter,
                semantic_scholar_enabled,
                warning: warnings.first().map(|warning| warning.message),
                note: article_type_note.as_deref(),
                debug_plan: debug_plan.as_ref(),
                exact_entity_commands: &exact_entity_commands,
                source_status: &source_status,
            },
        )?
    };

    Ok(CommandOutcome::stdout(text))
}

pub(super) fn article_session_suggestions(
    cache_root: &std::path::Path,
    token: &str,
    filters: &crate::entities::article::ArticleSearchFilters,
    results: &[crate::entities::article::ArticleSearchResult],
    next_commands: &[String],
    now_epoch_secs: u64,
) -> Vec<ArticleSuggestion> {
    let pmids = results
        .iter()
        .map(|result| result.pmid.clone())
        .collect::<Vec<_>>();
    super::session::record_success_and_suggestions(
        cache_root,
        super::session::SessionSearch {
            token,
            keyword: filters.keyword.as_deref(),
            pmids: &pmids,
            next_commands,
            now_epoch_secs,
        },
    )
}

pub(in crate::cli) async fn handle_command(
    cmd: ArticleCommand,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let text = match cmd {
        ArticleCommand::Authors { id } => {
            let result = crate::entities::article::authors(&id).await?;
            if json {
                crate::render::json::to_pretty(&result)?
            } else {
                crate::render::markdown::article_authors_markdown(&result)
            }
        }
        ArticleCommand::Entities { pmid, limit } => {
            let limit = super::super::paged_fetch_limit(limit, 0, 50)?;
            let sections = vec!["annotations".to_string()];
            let article = crate::entities::article::get(
                &pmid,
                &sections,
                crate::entities::article::ArticleGetOptions::default(),
            )
            .await?;
            let annotations = article
                .annotations
                .clone()
                .map(|value| super::truncate_article_annotations(value, limit));
            if json {
                #[derive(serde::Serialize)]
                struct ArticleEntitiesResponse {
                    pmid: String,
                    annotations: Option<crate::entities::article::ArticleAnnotations>,
                }

                crate::render::json::to_pretty(&ArticleEntitiesResponse { pmid, annotations })?
            } else {
                crate::render::markdown::article_entities_markdown(
                    article.pmid.as_deref().unwrap_or(&pmid),
                    annotations.as_ref(),
                    Some(limit),
                )?
            }
        }
        ArticleCommand::Batch { ids } => {
            if ids.len() > crate::entities::article::ARTICLE_BATCH_MAX_IDS {
                return Err(crate::error::BioMcpError::InvalidArgument(format!(
                    "Article batch is limited to {} IDs",
                    crate::entities::article::ARTICLE_BATCH_MAX_IDS
                ))
                .into());
            }
            let input_refs = ids.iter().map(String::as_str).collect::<Vec<_>>();
            let futures = ids
                .iter()
                .map(|id| crate::entities::article::get_compact(id));
            return crate::cli::system::settle_batch(
                "article",
                &input_refs,
                futures,
                json,
                |item| serde_json::to_value(item).map_err(crate::error::BioMcpError::Json),
                |item| crate::render::markdown::article_batch_markdown(std::slice::from_ref(item)),
            )
            .await;
        }
        ArticleCommand::Citations { id, limit } => {
            let limit = super::super::paged_fetch_limit(limit, 0, 100)?;
            let graph = crate::entities::article::citations(&id, limit).await?;
            if json {
                crate::render::json::to_pretty(&graph)?
            } else {
                crate::render::markdown::article_graph_markdown("Citations", &graph)?
            }
        }
        ArticleCommand::References { id, limit } => {
            let limit = super::super::paged_fetch_limit(limit, 0, 100)?;
            let graph = crate::entities::article::references(&id, limit).await?;
            if json {
                crate::render::json::to_pretty(&graph)?
            } else {
                crate::render::markdown::article_graph_markdown("References", &graph)?
            }
        }
        ArticleCommand::Recommendations {
            ids,
            negative,
            limit,
        } => {
            let limit = super::super::paged_fetch_limit(limit, 0, 100)?;
            let recommendations =
                crate::entities::article::recommendations(&ids, &negative, limit).await?;
            if json {
                crate::render::json::to_pretty(&recommendations)?
            } else {
                crate::render::markdown::article_recommendations_markdown(&recommendations)?
            }
        }
    };

    Ok(CommandOutcome::stdout(text))
}

pub(super) fn article_query_summary(
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    include_retracted: bool,
    limit: usize,
    offset: usize,
) -> String {
    let mut query = vec![
        filters.gene.as_deref().map(|v| format!("gene={v}")),
        filters.disease.as_deref().map(|v| format!("disease={v}")),
        filters.drug.as_deref().map(|v| format!("drug={v}")),
        filters.author.as_deref().map(|v| format!("author={v}")),
        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
        filters.article_type.as_deref().map(|v| format!("type={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.journal.as_deref().map(|v| format!("journal={v}")),
        filters.open_access.then(|| "open_access=true".to_string()),
        filters
            .no_preprints
            .then(|| "no_preprints=true".to_string()),
        if include_retracted {
            Some("include_retracted=true".to_string())
        } else {
            filters
                .exclude_retracted
                .then(|| "exclude_retracted=true".to_string())
        },
        Some(format!("sort={}", filters.sort.as_str())),
        (source_filter != crate::entities::article::ArticleSourceFilter::All)
            .then(|| format!("source={}", source_filter.as_str())),
        article_max_per_source_summary(filters.max_per_source, limit),
        (offset > 0).then(|| format!("offset={offset}")),
    ];
    if let Some(mode) = crate::entities::article::article_effective_ranking_mode(filters) {
        query.push(Some(format!("ranking_mode={}", mode.as_str())));
        query.push(
            crate::entities::article::article_relevance_ranking_policy(filters)
                .map(|policy| format!("ranking_policy={policy}")),
        );
    }
    query.into_iter().flatten().collect::<Vec<_>>().join(", ")
}

pub(super) fn article_max_per_source_summary(
    max_per_source: Option<usize>,
    limit: usize,
) -> Option<String> {
    match max_per_source {
        None => None,
        Some(0) => Some("max_per_source=default".to_string()),
        Some(value) if value == limit => Some("max_per_source=disabled".to_string()),
        Some(value) => Some(format!("max_per_source={value}")),
    }
}

pub(super) fn article_debug_filters(
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    limit: usize,
) -> Vec<String> {
    let mut values = vec![
        filters.gene.as_deref().map(|v| format!("gene={v}")),
        filters.disease.as_deref().map(|v| format!("disease={v}")),
        filters.drug.as_deref().map(|v| format!("drug={v}")),
        filters.author.as_deref().map(|v| format!("author={v}")),
        filters.keyword.as_deref().map(|v| format!("keyword={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.article_type.as_deref().map(|v| format!("type={v}")),
        filters.journal.as_deref().map(|v| format!("journal={v}")),
        filters.open_access.then(|| "open_access=true".to_string()),
        filters
            .no_preprints
            .then(|| "no_preprints=true".to_string()),
        Some(format!("exclude_retracted={}", filters.exclude_retracted)),
        Some(format!("sort={}", filters.sort.as_str())),
        Some(format!("source={}", source_filter.as_str())),
        article_max_per_source_summary(filters.max_per_source, limit),
    ];
    if let Some(mode) = crate::entities::article::article_effective_ranking_mode(filters) {
        values.push(Some(format!("ranking_mode={}", mode.as_str())));
        values.push(
            crate::entities::article::article_relevance_ranking_policy(filters)
                .map(|policy| format!("ranking_policy={policy}")),
        );
    }
    values.into_iter().flatten().collect()
}

pub(super) fn build_article_debug_plan(
    query: &str,
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    limit: usize,
    results: &[crate::entities::article::ArticleSearchResult],
    pagination: &crate::cli::PaginationMeta,
    source_status: &[crate::entities::article::ArticleSourceStatus],
) -> Result<crate::cli::debug_plan::DebugPlan, crate::error::BioMcpError> {
    let summary = crate::entities::article::summarize_debug_plan(filters, source_filter, results)?;
    let mut routing = summary.routing;
    routing.push(format!(
        "candidate_sources={}",
        summary.candidate_sources.join(",")
    ));
    routing.push(format!(
        "enrichment_sources={}",
        summary.enrichment_sources.join(",")
    ));
    Ok(crate::cli::debug_plan::DebugPlan {
        surface: "search_article",
        query: query.to_string(),
        anchor: None,
        legs: vec![crate::cli::debug_plan::DebugPlanLeg {
            leg: "article".to_string(),
            entity: "article".to_string(),
            filters: article_debug_filters(filters, source_filter, limit),
            routing,
            sources: summary.sources,
            matched_sources: summary.matched_sources,
            source_status: source_status.to_vec(),
            count: results.len(),
            total: pagination.total,
            note: crate::entities::article::article_type_limitation_note(filters, source_filter),
            error: None,
        }],
    })
}

pub(super) struct ArticleSearchJsonPage {
    pub results: Vec<crate::entities::article::ArticleSearchResult>,
    pub pagination: crate::cli::PaginationMeta,
    pub next_commands: Vec<String>,
    pub suggestions: Vec<ArticleSuggestion>,
    pub source_status: Vec<crate::entities::article::ArticleSourceStatus>,
}

#[cfg(test)]
pub(super) fn article_search_json(
    query: &str,
    filters: &crate::entities::article::ArticleSearchFilters,
    semantic_scholar_enabled: bool,
    note: Option<String>,
    debug_plan: Option<crate::cli::debug_plan::DebugPlan>,
    page: ArticleSearchJsonPage,
) -> anyhow::Result<String> {
    article_search_json_with_detail(
        query,
        filters,
        crate::entities::article::ArticleSourceFilter::All,
        ArticleSearchDetail::Full,
        semantic_scholar_enabled,
        note,
        debug_plan,
        page,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn article_search_json_with_detail(
    query: &str,
    filters: &crate::entities::article::ArticleSearchFilters,
    source_filter: crate::entities::article::ArticleSourceFilter,
    detail: ArticleSearchDetail,
    semantic_scholar_enabled: bool,
    note: Option<String>,
    debug_plan: Option<crate::cli::debug_plan::DebugPlan>,
    page: ArticleSearchJsonPage,
) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct ArticleSearchResponse<'a> {
        query: String,
        sort: String,
        semantic_scholar_enabled: bool,
        source_plan: crate::entities::article::ArticleSourcePlan,
        #[serde(skip_serializing_if = "Option::is_none")]
        ranking_policy: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        note: Option<String>,
        pagination: crate::cli::PaginationMeta,
        count: usize,
        results: ArticleSearchJsonRows<'a>,
        #[serde(skip_serializing_if = "Option::is_none")]
        debug_plan: Option<crate::cli::debug_plan::DebugPlan>,
        #[serde(skip_serializing_if = "Option::is_none")]
        _meta: Option<ArticleSearchJsonMeta>,
    }

    let count = page.results.len();
    let results = match detail {
        ArticleSearchDetail::Compact => ArticleSearchJsonRows::Compact(
            page.results
                .iter()
                .map(ArticleSearchCompactResult::from)
                .collect(),
        ),
        ArticleSearchDetail::Full => ArticleSearchJsonRows::Full(&page.results),
    };
    crate::render::json::to_pretty(&ArticleSearchResponse {
        query: query.to_string(),
        sort: filters.sort.as_str().to_string(),
        semantic_scholar_enabled,
        source_plan: crate::entities::article::article_source_plan(filters, source_filter)?,
        ranking_policy: crate::entities::article::article_relevance_ranking_policy(filters),
        note,
        pagination: page.pagination,
        count,
        results,
        debug_plan,
        _meta: article_search_json_meta(
            article_search_warnings(filters, &page.results),
            page.next_commands,
            page.suggestions,
            page.source_status,
        ),
    })
    .map_err(Into::into)
}
