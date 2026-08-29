pub(super) use super::plan::search_plan_from_args;
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

pub(super) fn validate_resolved_sections(
    report: &crate::entities::adverse_event::AdverseEventReport,
    sections_requested: bool,
) -> Result<(), crate::error::BioMcpError> {
    if sections_requested
        && matches!(
            report,
            crate::entities::adverse_event::AdverseEventReport::Device(_)
        )
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "Named sections are supported only for FAERS adverse-event reports; this ID resolved to a device report. Retry without sections."
                .into(),
        ));
    }
    Ok(())
}

pub(crate) async fn handle_get(
    args: AdverseEventGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    crate::entities::adverse_event::parse_sections(&sections)?;
    let sections_requested = !sections.is_empty();
    let event = crate::entities::adverse_event::get(&args.report_id).await?;
    validate_resolved_sections(&event, sections_requested)?;
    let text = render_loaded_card(&event, &sections, json_output)?;
    Ok(CommandOutcome::stdout(text))
}

pub(crate) fn render_loaded_card(
    event: &crate::entities::adverse_event::AdverseEventReport,
    sections: &[String],
    json_output: bool,
) -> anyhow::Result<String> {
    let parsed_sections = crate::entities::adverse_event::parse_sections(sections)?;
    let sections_requested = !sections.is_empty();
    let subset_requested = sections_requested
        && !sections
            .iter()
            .any(|section| section.eq_ignore_ascii_case("all"));
    if json_output {
        match event {
            crate::entities::adverse_event::AdverseEventReport::Faers(report)
                if subset_requested =>
            {
                let commands = if parsed_sections.include_guidance {
                    crate::render::markdown::adverse_event_guidance_commands(report)
                } else {
                    Vec::new()
                };
                Ok(crate::render::json::to_entity_json(
                    &crate::entities::adverse_event::FaersSubsetReport::new(
                        report,
                        parsed_sections,
                    ),
                    crate::render::markdown::adverse_event_evidence_urls(report),
                    commands,
                    crate::render::provenance::adverse_event_subset_section_sources(
                        report,
                        parsed_sections,
                    ),
                )?)
            }
            crate::entities::adverse_event::AdverseEventReport::Faers(report) => {
                Ok(crate::render::json::to_entity_json(
                    event,
                    crate::render::markdown::adverse_event_evidence_urls(report),
                    crate::render::markdown::related_adverse_event(report),
                    crate::render::provenance::adverse_event_report_section_sources(event),
                )?)
            }
            crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                Ok(crate::render::json::to_entity_json(
                    event,
                    crate::render::markdown::device_event_evidence_urls(report),
                    crate::render::markdown::related_device_event(report),
                    crate::render::provenance::adverse_event_report_section_sources(event),
                )?)
            }
        }
    } else {
        match event {
            crate::entities::adverse_event::AdverseEventReport::Faers(report) => Ok(
                crate::render::markdown::adverse_event_markdown(report, sections)?,
            ),
            crate::entities::adverse_event::AdverseEventReport::Device(report) => {
                debug_assert!(!sections_requested);
                Ok(crate::render::markdown::device_event_markdown(report)?)
            }
        }
    }
}

pub(crate) async fn handle_search(
    args: AdverseEventSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let plan = search_plan_from_args(&args)?;
    let drug = plan.drug;
    let query_type = plan.query_type;
    let source_filter = plan.source_filter;
    let device_seriousness = plan.device_seriousness;

    let text = match query_type {
        crate::entities::adverse_event::AdverseEventQueryType::Faers => {
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
                let section_outcomes = source_response.section_outcomes.clone();
                let section_sources =
                    crate::render::provenance::adverse_event_source_search_section_sources(
                        &source_response,
                    );
                if json {
                    #[derive(serde::Serialize)]
                    struct FaersSearchResponse {
                        section_outcomes: crate::entities::section_outcome::SectionOutcomes,
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
                        section_outcomes: crate::entities::section_outcome::SectionOutcomes,
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
                        section_outcomes: crate::entities::section_outcome::SectionOutcomes,
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
                                crate::entities::adverse_event::FaersSearchStatus::NotFound
                                | crate::entities::adverse_event::FaersSearchStatus::Unavailable => {
                                    let response =
                                        crate::entities::adverse_event::empty_search_response();
                                    (response.results, response.summary)
                                }
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
                                section_outcomes,
                                source: "faers",
                                pagination,
                                count: results.len(),
                                summary,
                                results,
                                _meta: crate::cli::search_meta_with_section_sources(
                                    next_commands,
                                    section_sources,
                                ),
                            })?
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::Vaers => {
                            let vaers = source_response.vaers.expect("vaers payload");
                            let next_commands = vaers_only_next_commands(&raw_query);
                            crate::render::json::to_pretty(&VaersOnlyResponse {
                                section_outcomes,
                                source: "vaers",
                                query: raw_query.clone(),
                                vaers,
                                _meta: crate::cli::search_meta_with_section_sources(
                                    next_commands,
                                    section_sources,
                                ),
                            })?
                        }
                        crate::entities::adverse_event::AdverseEventSourceFilter::All => {
                            let status = source_response.faers.expect("faers status");
                            let vaers = source_response.vaers.expect("vaers payload");
                            let (results, summary) = match status {
                                crate::entities::adverse_event::FaersSearchStatus::NotFound
                                | crate::entities::adverse_event::FaersSearchStatus::Unavailable => {
                                    let response =
                                        crate::entities::adverse_event::empty_search_response();
                                    (response.results, response.summary)
                                }
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
                                section_outcomes,
                                source: "all",
                                pagination,
                                count: results.len(),
                                summary,
                                results,
                                vaers,
                                _meta: crate::cli::search_meta_with_section_sources(
                                    next_commands,
                                    section_sources,
                                ),
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
                                crate::entities::adverse_event::FaersSearchStatus::Unavailable => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                    Some("OpenFDA FAERS adverse events are unavailable."),
                                ),
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
                                crate::entities::adverse_event::FaersSearchStatus::Unavailable => (
                                    Vec::new(),
                                    crate::entities::adverse_event::AdverseEventSearchSummary {
                                        total_reports: 0,
                                        returned_report_count: 0,
                                        top_reactions: Vec::new(),
                                    },
                                    Some("OpenFDA FAERS adverse events are unavailable."),
                                ),
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
            let filters = crate::entities::adverse_event::DeviceEventSearchFilters {
                device: args.device,
                manufacturer: args.manufacturer,
                product_code: args.product_code,
                serious: device_seriousness,
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
                #[derive(serde::Serialize)]
                struct DeviceSearchResponse {
                    query: String,
                    pagination: super::super::PaginationMeta,
                    count: usize,
                    results: Vec<crate::entities::adverse_event::DeviceEventSearchResult>,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    _meta: Option<crate::cli::SearchJsonMeta>,
                }
                let next_commands =
                    crate::render::markdown::search_next_commands_device_events(&results);
                return Ok(CommandOutcome::stdout(crate::render::json::to_pretty(
                    &DeviceSearchResponse {
                        query: query_summary,
                        count: results.len(),
                        results,
                        pagination,
                        _meta: crate::cli::search_meta(next_commands),
                    },
                )?));
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
