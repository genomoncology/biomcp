use super::{AdverseEventGetArgs, AdverseEventSearchArgs};
use crate::cli::CommandOutcome;

fn vaers_only_next_commands(query: &str) -> Vec<String> {
    vec![
        crate::next_command::NextCommand::biomcp()
            .args(["search", "adverse-event", query, "--source", "faers"])
            .render_shell(),
        crate::next_command::NextCommand::biomcp()
            .args(["search", "drug", query])
            .render_shell(),
        "biomcp health".to_string(),
        "biomcp list adverse-event".to_string(),
    ]
}

pub(super) fn search_plan_from_args(
    args: &AdverseEventSearchArgs,
) -> Result<
    (
        Option<String>,
        crate::entities::adverse_event::AdverseEventQueryType,
        crate::entities::adverse_event::AdverseEventSourceFilter,
    ),
    crate::error::BioMcpError,
> {
    let drug = super::super::resolve_query_input(
        args.drug.clone(),
        args.positional_query.clone(),
        "--drug",
    )?;
    let query_type =
        crate::entities::adverse_event::AdverseEventQueryType::from_flag(&args.r#type)?;
    let source_filter =
        crate::entities::adverse_event::AdverseEventSourceFilter::from_flag(&args.source)?;

    match query_type {
        crate::entities::adverse_event::AdverseEventQueryType::Faers => {
            if args.device.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--device can only be used with --type device".into(),
                ));
            }
            if args.manufacturer.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--manufacturer can only be used with --type device".into(),
                ));
            }
            if args.product_code.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--product-code can only be used with --type device".into(),
                ));
            }
            if args.count.is_some()
                && matches!(
                    source_filter,
                    crate::entities::adverse_event::AdverseEventSourceFilter::Vaers
                )
            {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--count is not supported with --source vaers".into(),
                ));
            }
        }
        crate::entities::adverse_event::AdverseEventQueryType::Recall => {
            if !matches!(
                source_filter,
                crate::entities::adverse_event::AdverseEventSourceFilter::All
            ) {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                ));
            }
            if args.date_from.is_some()
                || args.date_to.is_some()
                || args.suspect_only
                || args.sex.is_some()
                || args.age_min.is_some()
                || args.age_max.is_some()
                || args.reporter.is_some()
                || args.count.is_some()
            {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--date-from/--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                ));
            }
            if args.device.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--device can only be used with --type device".into(),
                ));
            }
            if args.manufacturer.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--manufacturer can only be used with --type device".into(),
                ));
            }
            if args.product_code.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--product-code can only be used with --type device".into(),
                ));
            }
            if args.outcome.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--outcome is only valid for --type faers".into(),
                ));
            }
        }
        crate::entities::adverse_event::AdverseEventQueryType::Device => {
            if !matches!(
                source_filter,
                crate::entities::adverse_event::AdverseEventSourceFilter::All
            ) {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                ));
            }
            if drug.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--drug cannot be used with --type device (use --device)".into(),
                ));
            }
            if args.reaction.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--reaction is not supported with --type device".into(),
                ));
            }
            if args.outcome.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--outcome is only valid for --type faers".into(),
                ));
            }
            if args.classification.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--classification is only valid for --type recall".into(),
                ));
            }
            if args.date_to.is_some()
                || args.suspect_only
                || args.sex.is_some()
                || args.age_min.is_some()
                || args.age_max.is_some()
                || args.reporter.is_some()
                || args.count.is_some()
            {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                ));
            }
        }
    }

    Ok((drug, query_type, source_filter))
}

pub(crate) async fn handle_get(
    args: AdverseEventGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let event = crate::entities::adverse_event::get(&args.report_id).await?;
    let text = if json_output {
        match &event {
            crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                crate::render::json::to_entity_json(
                    &event,
                    crate::render::markdown::adverse_event_evidence_urls(report),
                    crate::render::markdown::related_adverse_event(report),
                    crate::render::provenance::adverse_event_report_section_sources(&event),
                )?
            }
            crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                crate::render::json::to_entity_json(
                    &event,
                    crate::render::markdown::device_event_evidence_urls(report),
                    crate::render::markdown::related_device_event(report),
                    crate::render::provenance::adverse_event_report_section_sources(&event),
                )?
            }
        }
    } else {
        match &event {
            crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                crate::render::markdown::adverse_event_markdown(report, &sections)?
            }
            crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                crate::render::markdown::device_event_markdown(report)?
            }
        }
    };
    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_search(
    args: AdverseEventSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (drug, query_type, source_filter) = search_plan_from_args(&args)?;

    let text = match query_type {
        crate::entities::adverse_event::AdverseEventQueryType::Faers => {
            if args.device.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--device can only be used with --type device".into(),
                )
                .into());
            }
            if args.manufacturer.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--manufacturer can only be used with --type device".into(),
                )
                .into());
            }
            if args.product_code.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--product-code can only be used with --type device".into(),
                )
                .into());
            }

            let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                drug,
                reaction: args.reaction,
                outcome: args.outcome,
                serious: args.serious,
                since: args.date_from,
                date_to: args.date_to,
                suspect_only: args.suspect_only,
                sex: args.sex,
                age_min: args.age_min,
                age_max: args.age_max,
                reporter: args.reporter,
            };
            let mut query_summary = crate::entities::adverse_event::search_query_summary(&filters);
            if let Some(count_field) = args
                .count
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            {
                if query_summary.is_empty() {
                    query_summary = format!("count={count_field}");
                } else {
                    query_summary = format!("{query_summary}, count={count_field}");
                }
            }
            if args.offset > 0 {
                query_summary = format!("{query_summary}, offset={}", args.offset);
            }

            if let Some(count_field) = args.count.as_deref().map(str::trim) {
                if matches!(
                    source_filter,
                    crate::entities::adverse_event::AdverseEventSourceFilter::Vaers
                ) {
                    return Err(crate::error::BioMcpError::InvalidArgument(
                        "--count is not supported with --source vaers".into(),
                    )
                    .into());
                }
                let response =
                    crate::entities::adverse_event::search_count(&filters, count_field, args.limit)
                        .await?;
                if json {
                    #[derive(serde::Serialize)]
                    struct CountResponse {
                        query: String,
                        count_field: String,
                        buckets: Vec<crate::entities::adverse_event::AdverseEventCountBucket>,
                    }

                    crate::render::json::to_pretty(&CountResponse {
                        query: query_summary,
                        count_field: response.count_field,
                        buckets: response.buckets,
                    })?
                } else {
                    crate::render::markdown::adverse_event_count_markdown(
                        &query_summary,
                        &response.count_field,
                        &response.buckets,
                    )?
                }
            } else {
                let source_response = crate::entities::adverse_event::search_with_source(
                    &filters,
                    source_filter,
                    args.limit,
                    args.offset,
                )
                .await?;
                let raw_query = filters.drug.clone().unwrap_or_default();
                if json {
                    #[derive(serde::Serialize)]
                    struct FaersSearchResponse {
                        source: &'static str,
                        pagination: super::super::PaginationMeta,
                        count: usize,
                        summary: crate::entities::adverse_event::AdverseEventSearchSummary,
                        results: Vec<crate::entities::adverse_event::AdverseEventSearchResult>,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        _meta: Option<crate::cli::SearchJsonMeta>,
                    }

                    #[derive(serde::Serialize)]
                    struct CombinedSearchResponse {
                        source: &'static str,
                        pagination: super::super::PaginationMeta,
                        count: usize,
                        summary: crate::entities::adverse_event::AdverseEventSearchSummary,
                        results: Vec<crate::entities::adverse_event::AdverseEventSearchResult>,
                        vaers: crate::entities::adverse_event::VaersSearchPayload,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        _meta: Option<crate::cli::SearchJsonMeta>,
                    }

                    #[derive(serde::Serialize)]
                    struct VaersOnlyResponse {
                        source: &'static str,
                        query: String,
                        vaers: crate::entities::adverse_event::VaersSearchPayload,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        _meta: Option<crate::cli::SearchJsonMeta>,
                    }

                    match source_response.source {
                        crate::entities::adverse_event::AdverseEventSourceFilter::Faers => {
                            let status = source_response.faers.expect("faers status");
                            let (results, summary) = match status {
                                crate::entities::adverse_event::FaersSearchStatus::NotFound => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                ),
                                crate::entities::adverse_event::FaersSearchStatus::Results(
                                    response,
                                ) => (response.results, response.summary),
                            };
                            let pagination = super::super::PaginationMeta::offset(
                                args.offset,
                                args.limit,
                                results.len(),
                                Some(summary.total_reports),
                            );
                            let next_commands =
                                crate::render::markdown::search_next_commands_faers(&results);
                            crate::render::json::to_pretty(&FaersSearchResponse {
                                source: "faers",
                                pagination,
                                count: results.len(),
                                summary,
                                results,
                                _meta: crate::cli::search_meta(next_commands),
                            })?
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::Vaers => {
                            let vaers = source_response.vaers.expect("vaers payload");
                            let next_commands = vaers_only_next_commands(&raw_query);
                            crate::render::json::to_pretty(&VaersOnlyResponse {
                                source: "vaers",
                                query: raw_query.clone(),
                                vaers,
                                _meta: crate::cli::search_meta(next_commands),
                            })?
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::All => {
                            let status = source_response.faers.expect("faers status");
                            let vaers = source_response.vaers.expect("vaers payload");
                            let (results, summary) = match status {
                                crate::entities::adverse_event::FaersSearchStatus::NotFound => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                ),
                                crate::entities::adverse_event::FaersSearchStatus::Results(
                                    response,
                                ) => (response.results, response.summary),
                            };
                            let pagination = super::super::PaginationMeta::offset(
                                args.offset,
                                args.limit,
                                results.len(),
                                Some(summary.total_reports),
                            );
                            let next_commands = if results.is_empty() {
                                vaers_only_next_commands(&raw_query)
                            } else {
                                crate::render::markdown::search_next_commands_faers(&results)
                            };
                            crate::render::json::to_pretty(&CombinedSearchResponse {
                                source: "all",
                                pagination,
                                count: results.len(),
                                summary,
                                results,
                                vaers,
                                _meta: crate::cli::search_meta(next_commands),
                            })?
                        }
                    }
                } else {
                    match source_response.source {
                        crate::entities::adverse_event::AdverseEventSourceFilter::Faers => {
                            let status = source_response.faers.expect("faers status");
                            let (results, summary, empty_state_message) = match status {
                                crate::entities::adverse_event::FaersSearchStatus::NotFound => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                    Some(
                                        "Drug not found in FAERS. FAERS is a post-marketing database; expect no records for investigational, newly approved, or name-variant drugs.",
                                    ),
                                ),
                                crate::entities::adverse_event::FaersSearchStatus::Results(
                                    response,
                                ) => {
                                    let message = response.results.is_empty().then_some(
                                        "Drug found in FAERS, but no events matched your filters. Try broadening the search.",
                                    );
                                    (response.results, response.summary, message)
                                }
                            };
                            let pagination = super::super::PaginationMeta::offset(
                                args.offset,
                                args.limit,
                                results.len(),
                                Some(summary.total_reports),
                            );
                            let footer = super::super::pagination_footer_offset(&pagination);
                            crate::render::markdown::adverse_event_search_markdown_with_source_label(
                                &query_summary,
                                &results,
                                &summary,
                                &footer,
                                empty_state_message,
                                &[],
                                None,
                                "OpenFDA FAERS",
                            )?
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::Vaers => {
                            let vaers = source_response.vaers.expect("vaers payload");
                            crate::render::markdown::vaers_only_markdown(&raw_query, &vaers)
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::All => {
                            let status = source_response.faers.expect("faers status");
                            let vaers = source_response.vaers.expect("vaers payload");
                            let (results, summary, empty_state_message) = match status {
                                crate::entities::adverse_event::FaersSearchStatus::NotFound => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                    Some(
                                        "Drug not found in FAERS. FAERS is a post-marketing database; expect no records for investigational, newly approved, or name-variant drugs.",
                                    ),
                                ),
                                crate::entities::adverse_event::FaersSearchStatus::Results(
                                    response,
                                ) => {
                                    let message = response.results.is_empty().then_some(
                                        "Drug found in FAERS, but no events matched your filters. Try broadening the search.",
                                    );
                                    (response.results, response.summary, message)
                                }
                            };
                            let pagination = super::super::PaginationMeta::offset(
                                args.offset,
                                args.limit,
                                results.len(),
                                Some(summary.total_reports),
                            );
                            let footer = super::super::pagination_footer_offset(&pagination);
                            crate::render::markdown::combined_adverse_event_search_markdown(
                                &query_summary,
                                &results,
                                &summary,
                                &footer,
                                empty_state_message,
                                Some(&vaers),
                            )?
                        }
                    }
                }
            }
        }
        crate::entities::adverse_event::AdverseEventQueryType::Recall => {
            if !matches!(
                source_filter,
                crate::entities::adverse_event::AdverseEventSourceFilter::All
            ) {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                )
                .into());
            }
            if args.date_from.is_some()
                || args.date_to.is_some()
                || args.suspect_only
                || args.sex.is_some()
                || args.age_min.is_some()
                || args.age_max.is_some()
                || args.reporter.is_some()
                || args.count.is_some()
            {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--date-from/--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                )
                .into());
            }
            if args.device.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--device can only be used with --type device".into(),
                )
                .into());
            }
            if args.manufacturer.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--manufacturer can only be used with --type device".into(),
                )
                .into());
            }
            if args.product_code.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--product-code can only be used with --type device".into(),
                )
                .into());
            }
            if args.outcome.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--outcome is only valid for --type faers".into(),
                )
                .into());
            }
            let filters = crate::entities::adverse_event::RecallSearchFilters {
                drug,
                classification: args.classification,
            };
            let mut query_summary = crate::entities::adverse_event::recall_query_summary(&filters);
            if args.offset > 0 {
                query_summary = format!("{query_summary}, offset={}", args.offset);
            }
            let page = crate::entities::adverse_event::search_recalls_page(
                &filters,
                args.limit,
                args.offset,
            )
            .await?;
            let results = page.results;
            let pagination = super::super::PaginationMeta::offset(
                args.offset,
                args.limit,
                results.len(),
                page.total,
            );
            if json {
                let next_commands = crate::render::markdown::search_next_commands_recalls(&results);
                return super::super::search_json_with_meta(results, pagination, next_commands)
                    .map(CommandOutcome::stdout);
            }
            let footer = super::super::pagination_footer_offset(&pagination);
            crate::render::markdown::recall_search_markdown_with_footer(
                &query_summary,
                &results,
                &footer,
            )?
        }
        crate::entities::adverse_event::AdverseEventQueryType::Device => {
            if !matches!(
                source_filter,
                crate::entities::adverse_event::AdverseEventSourceFilter::All
            ) {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--source is only supported for --type faers adverse-event search".into(),
                )
                .into());
            }
            if drug.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--drug cannot be used with --type device (use --device)".into(),
                )
                .into());
            }
            if args.reaction.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--reaction is not supported with --type device".into(),
                )
                .into());
            }
            if args.outcome.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--outcome is only valid for --type faers".into(),
                )
                .into());
            }
            if args.classification.is_some() {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--classification is only valid for --type recall".into(),
                )
                .into());
            }
            if args.date_to.is_some()
                || args.suspect_only
                || args.sex.is_some()
                || args.age_min.is_some()
                || args.age_max.is_some()
                || args.reporter.is_some()
                || args.count.is_some()
            {
                return Err(crate::error::BioMcpError::InvalidArgument(
                    "--date-to/--suspect-only/--sex/--age-min/--age-max/--reporter/--count are only valid for --type faers".into(),
                )
                .into());
            }

            let filters = crate::entities::adverse_event::DeviceEventSearchFilters {
                device: args.device,
                manufacturer: args.manufacturer,
                product_code: args.product_code,
                serious: args.serious.is_some(),
                since: args.date_from,
            };
            let mut query_summary = crate::entities::adverse_event::device_query_summary(&filters);
            if args.offset > 0 {
                query_summary = format!("{query_summary}, offset={}", args.offset);
            }
            let page = crate::entities::adverse_event::search_device_page(
                &filters,
                args.limit,
                args.offset,
            )
            .await?;
            let results = page.results;
            let pagination = super::super::PaginationMeta::offset(
                args.offset,
                args.limit,
                results.len(),
                page.total,
            );
            if json {
                let next_commands =
                    crate::render::markdown::search_next_commands_device_events(&results);
                return super::super::search_json_with_meta(results, pagination, next_commands)
                    .map(CommandOutcome::stdout);
            }
            let footer = super::super::pagination_footer_offset(&pagination);
            crate::render::markdown::device_event_search_markdown_with_footer(
                &query_summary,
                &results,
                &footer,
            )?
        }
    };

    Ok(CommandOutcome::stdout(text))
}
