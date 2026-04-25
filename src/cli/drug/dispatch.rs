use super::{DrugCommand, DrugGetArgs, DrugSearchArgs, WhoProductTypeArg};
use crate::cli::CommandOutcome;
use crate::entities::drug::DrugRegion;
use crate::sources::who_pq::WhoProductTypeFilter;

pub(crate) async fn handle_get(
    args: DrugGetArgs,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let region = args.region.map(DrugRegion::from);
    let json_output = json || json_override;
    render_drug_card_outcome(
        &args.name,
        &sections,
        region,
        args.raw,
        json_output,
        alias_suggestions_as_json,
    )
    .await
}

pub(crate) async fn handle_search(
    args: DrugSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let region_arg = args.region;
    let who_product_type_arg = args.who_product_type;
    if who_product_type_arg.is_some() && !matches!(region_arg, Some(crate::cli::DrugRegionArg::Who))
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "The WHO-only --product-type filter requires explicit --region who; rerun with --region who.".into(),
        )
        .into());
    }

    let who_product_type = who_product_type_arg
        .map(|value| match value {
            WhoProductTypeArg::FinishedPharma => WhoProductTypeFilter::FinishedPharma,
            WhoProductTypeArg::Api => WhoProductTypeFilter::Api,
            WhoProductTypeArg::Vaccine => WhoProductTypeFilter::Vaccine,
        })
        .unwrap_or_default();
    let query = super::super::resolve_query_input(args.query, args.positional_query, "--query")?;
    let filters = crate::entities::drug::DrugSearchFilters {
        query,
        target: args.target,
        indication: args.indication,
        mechanism: args.mechanism,
        drug_type: args.drug_type,
        atc: args.atc,
        pharm_class: args.pharm_class,
        interactions: args.interactions,
    };
    let region = resolve_drug_search_region(region_arg, &filters)?;
    let page_with_region = crate::entities::drug::search_page_with_region(
        &filters,
        args.limit,
        args.offset,
        region,
        who_product_type,
    )
    .await?;
    if json {
        let workflow = (filters
            .indication
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            && drug_search_page_has_results(&page_with_region))
        .then(|| {
            crate::workflow_ladders::meta_for(crate::workflow_ladders::Workflow::TreatmentLookup)
        })
        .transpose()?;
        return drug_search_json(
            page_with_region,
            filters.query.as_deref(),
            args.offset,
            args.limit,
            workflow,
        )
        .map(CommandOutcome::stdout);
    }

    let mut query_summary = crate::entities::drug::search_query_summary(&filters);
    if let Some(product_type) = who_product_type_arg {
        let value = match product_type {
            WhoProductTypeArg::FinishedPharma => "finished_pharma",
            WhoProductTypeArg::Api => "api",
            WhoProductTypeArg::Vaccine => "vaccine",
        };
        query_summary = format!("{query_summary}, product_type={value}");
    }
    if args.offset > 0 {
        query_summary = format!("{query_summary}, offset={}", args.offset);
    }
    let text = match page_with_region {
        crate::entities::drug::DrugSearchPageWithRegion::Us(page) => {
            let results = page.results;
            let pagination = super::super::PaginationMeta::offset(
                args.offset,
                args.limit,
                results.len(),
                page.total,
            );
            let footer = super::super::pagination_footer_offset(&pagination);
            crate::render::markdown::drug_search_markdown_with_region(
                &query_summary,
                region,
                &results,
                pagination.total,
                &[],
                None,
                &[],
                None,
                &footer,
            )?
        }
        crate::entities::drug::DrugSearchPageWithRegion::Eu(page) => {
            let results = page.results;
            let pagination = super::super::PaginationMeta::offset(
                args.offset,
                args.limit,
                results.len(),
                page.total,
            );
            let footer = super::super::pagination_footer_offset(&pagination);
            crate::render::markdown::drug_search_markdown_with_region(
                &query_summary,
                region,
                &[],
                None,
                &results,
                pagination.total,
                &[],
                None,
                &footer,
            )?
        }
        crate::entities::drug::DrugSearchPageWithRegion::Who(page) => {
            let results = page.results;
            let pagination = super::super::PaginationMeta::offset(
                args.offset,
                args.limit,
                results.len(),
                page.total,
            );
            let footer = super::super::pagination_footer_offset(&pagination);
            crate::render::markdown::drug_search_markdown_with_region(
                &query_summary,
                region,
                &[],
                None,
                &[],
                None,
                &results,
                pagination.total,
                &footer,
            )?
        }
        crate::entities::drug::DrugSearchPageWithRegion::All { us, eu, who } => {
            crate::render::markdown::drug_search_markdown_with_region(
                &query_summary,
                region,
                &us.results,
                us.total,
                &eu.results,
                eu.total,
                &who.results,
                who.total,
                "",
            )?
        }
    };

    Ok(CommandOutcome::stdout(text))
}

pub(crate) async fn handle_command(
    cmd: DrugCommand,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    match cmd {
        DrugCommand::External(args) => {
            let name = args.join(" ");
            render_drug_card_outcome(
                &name,
                super::super::empty_sections(),
                None,
                false,
                json,
                alias_suggestions_as_json,
            )
            .await
        }
        other => {
            let text = match other {
                DrugCommand::Trials {
                    name,
                    limit,
                    offset,
                    source,
                    no_alias_expand,
                } => {
                    let trial_source = crate::entities::trial::TrialSource::from_flag(&source)?;
                    let filters = crate::entities::trial::TrialSearchFilters {
                        intervention: Some(name.clone()),
                        no_alias_expand,
                        source: trial_source,
                        ..Default::default()
                    };
                    let (results, total) =
                        crate::entities::trial::search(&filters, limit, offset).await?;
                    if let Some(total) = total {
                        super::super::log_pagination_truncation(
                            total as usize,
                            offset,
                            results.len(),
                        );
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
                        let mut query_parts = vec![format!("intervention={name}")];
                        if no_alias_expand
                            && matches!(
                                trial_source,
                                crate::entities::trial::TrialSource::ClinicalTrialsGov
                            )
                        {
                            query_parts.push("alias_expand=off".to_string());
                        }
                        if offset > 0 {
                            query_parts.push(format!("offset={offset}"));
                        }
                        let query = query_parts.join(", ");
                        crate::render::markdown::trial_search_markdown(&query, &results, total)?
                    }
                }
                DrugCommand::AdverseEvents {
                    name,
                    limit,
                    offset,
                    serious,
                } => {
                    #[derive(serde::Serialize)]
                    struct SearchResponse {
                        total: Option<usize>,
                        count: usize,
                        summary: crate::entities::adverse_event::AdverseEventSearchSummary,
                        results: Vec<crate::entities::adverse_event::AdverseEventSearchResult>,
                        faers_not_found: bool,
                        #[serde(skip_serializing_if = "Option::is_none")]
                        trial_adverse_events:
                            Option<Vec<crate::entities::adverse_event::TrialAdverseEventTerm>>,
                    }

                    let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                        drug: Some(name.clone()),
                        serious: serious.then_some("any".to_string()),
                        ..Default::default()
                    };
                    let query_summary =
                        crate::entities::adverse_event::search_query_summary(&filters);
                    let fetch_limit = super::super::paged_fetch_limit(limit, offset, 50)?;
                    let status = crate::entities::adverse_event::search_with_status(
                        &filters,
                        fetch_limit,
                        0,
                    )
                    .await?;
                    match status {
                        crate::entities::adverse_event::FaersSearchStatus::Results(response) => {
                            let (results, observed_total) =
                                super::super::paginate_results(response.results, offset, limit);
                            super::super::log_pagination_truncation(
                                observed_total,
                                offset,
                                results.len(),
                            );
                            let summary = crate::entities::adverse_event::summarize_search_results(
                                response.summary.total_reports,
                                &results,
                            );
                            if json {
                                crate::render::json::to_pretty(&SearchResponse {
                                    total: Some(summary.total_reports),
                                    count: results.len(),
                                    summary,
                                    results,
                                    faers_not_found: false,
                                    trial_adverse_events: None,
                                })?
                            } else {
                                let empty_state_message = results.is_empty().then_some(
                                    "Drug found in FAERS, but no events matched your filters. Try broadening the search.",
                                );
                                crate::render::markdown::adverse_event_search_markdown_with_context(
                                    &query_summary,
                                    &results,
                                    &summary,
                                    "",
                                    empty_state_message,
                                    &[],
                                    None,
                                )?
                            }
                        }
                        crate::entities::adverse_event::FaersSearchStatus::NotFound => {
                            let trial_adverse_events =
                                match crate::entities::adverse_event::trial_adverse_events(&name)
                                    .await
                                {
                                    Ok(crate::entities::adverse_event::TrialAdverseEventOutcome::Found(
                                        rows,
                                    )) => Some(rows),
                                    Ok(crate::entities::adverse_event::TrialAdverseEventOutcome::Empty) => {
                                        None
                                    }
                                    Err(err) => {
                                        return Err(anyhow::anyhow!(
                                            "drug not found in FAERS and ClinicalTrials.gov fallback failed: {err}"
                                        ));
                                    }
                                };
                            let summary =
                                crate::entities::adverse_event::AdverseEventSearchSummary {
                                    total_reports: 0,
                                    returned_report_count: 0,
                                    top_reactions: Vec::new(),
                                };
                            let results = Vec::new();
                            if json {
                                crate::render::json::to_pretty(&SearchResponse {
                                    total: Some(0),
                                    count: 0,
                                    summary,
                                    results,
                                    faers_not_found: true,
                                    trial_adverse_events,
                                })?
                            } else {
                                let empty_state_message = if trial_adverse_events.is_some() {
                                    "Drug not found in FAERS. FAERS is a post-marketing database; expect no records for investigational, newly approved, or name-variant drugs. Falling back to ClinicalTrials.gov trial-reported adverse events."
                                } else {
                                    "Drug not found in FAERS. FAERS is a post-marketing database; expect no records for investigational, newly approved, or name-variant drugs. Falling back to ClinicalTrials.gov trial-reported adverse events. ClinicalTrials.gov did not return posted trial adverse events for this drug."
                                };
                                let trial_rows = trial_adverse_events.unwrap_or_default();
                                crate::render::markdown::adverse_event_search_markdown_with_context(
                                    &query_summary,
                                    &results,
                                    &summary,
                                    "",
                                    Some(empty_state_message),
                                    &trial_rows,
                                    Some(&name),
                                )?
                            }
                        }
                    }
                }
                DrugCommand::Interactions { name } => {
                    let report = crate::entities::drug::interaction_report(name).await?;
                    if json {
                        crate::render::json::to_entity_json(
                            &report,
                            crate::render::markdown::drug_interaction_report_evidence_urls(&report),
                            crate::render::markdown::related_drug_interactions(&report.name),
                            crate::render::provenance::drug_interaction_report_section_sources(
                                &report,
                            ),
                        )?
                    } else {
                        crate::render::markdown::drug_interaction_report_markdown(&report)?
                    }
                }
                DrugCommand::External(_) => unreachable!("handled above"),
            };

            Ok(CommandOutcome::stdout(text))
        }
    }
}

pub(super) const DRUG_SEARCH_EMA_STRUCTURED_FILTER_ERROR: &str = "EMA and all-region search currently support name/alias lookups only; use --region us for structured MyChem filters or --region who to filter structured U.S. hits through WHO prequalification.";

pub(super) fn resolve_drug_search_region(
    region_arg: Option<crate::cli::DrugRegionArg>,
    filters: &crate::entities::drug::DrugSearchFilters,
) -> Result<DrugRegion, crate::error::BioMcpError> {
    match (region_arg, filters.has_structured_filters()) {
        (None, false) => Ok(DrugRegion::All),
        (None, true) | (Some(crate::cli::DrugRegionArg::Us), _) => Ok(DrugRegion::Us),
        (Some(crate::cli::DrugRegionArg::Who), _) => Ok(DrugRegion::Who),
        (Some(crate::cli::DrugRegionArg::Eu), false) => Ok(DrugRegion::Eu),
        (Some(crate::cli::DrugRegionArg::All), false) => Ok(DrugRegion::All),
        (Some(crate::cli::DrugRegionArg::Eu | crate::cli::DrugRegionArg::All), true) => {
            Err(crate::error::BioMcpError::InvalidArgument(
                DRUG_SEARCH_EMA_STRUCTURED_FILTER_ERROR.into(),
            ))
        }
    }
}

pub(super) fn resolve_drug_get_region(
    sections: &[String],
    region: Option<DrugRegion>,
) -> DrugRegion {
    if let Some(region) = region {
        return region;
    }

    if matches!(sections, [section] if section.eq_ignore_ascii_case("regulatory")) {
        DrugRegion::All
    } else {
        DrugRegion::Us
    }
}

pub(super) async fn render_drug_card_outcome(
    name: &str,
    sections: &[String],
    region: Option<DrugRegion>,
    raw_label: bool,
    json_output: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let effective_region = resolve_drug_get_region(sections, region);
    match crate::entities::drug::get_with_region(
        name,
        sections,
        effective_region,
        region.is_some(),
        raw_label,
    )
    .await
    {
        Ok(drug) => {
            let text = if json_output {
                let workflow = drug_pharmacogene_workflow(&drug, name).await?;
                crate::render::json::to_entity_json_with_workflow(
                    &drug,
                    crate::render::markdown::drug_evidence_urls(&drug),
                    crate::render::markdown::related_drug(&drug),
                    crate::render::provenance::drug_section_sources(&drug),
                    workflow,
                )?
            } else {
                crate::render::markdown::drug_markdown_with_region(
                    &drug,
                    sections,
                    effective_region,
                    raw_label,
                )?
            };
            Ok(CommandOutcome::stdout(text))
        }
        Err(err @ crate::error::BioMcpError::NotFound { .. }) => {
            if let Some(outcome) = super::super::try_alias_fallback_outcome(
                name,
                crate::entities::discover::DiscoverType::Drug,
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

async fn drug_pharmacogene_workflow(
    drug: &crate::entities::drug::Drug,
    requested_name: &str,
) -> Result<Option<crate::workflow_ladders::WorkflowMeta>, crate::error::BioMcpError> {
    const CPIC_GENE_THRESHOLD: usize = 3;

    let has_signal = match crate::workflow_ladders::probe_workflow(
        crate::workflow_ladders::Workflow::PharmacogeneCumulative,
        Box::pin(async {
            let primary_count = crate::entities::pgx::distinct_actionable_cpic_gene_count_for_drug(
                &drug.name,
                CPIC_GENE_THRESHOLD,
            )
            .await?;
            if primary_count >= CPIC_GENE_THRESHOLD {
                return Ok(true);
            }

            let requested = requested_name.trim();
            if requested.is_empty() || requested.eq_ignore_ascii_case(drug.name.trim()) {
                return Ok(false);
            }

            let requested_count =
                crate::entities::pgx::distinct_actionable_cpic_gene_count_for_drug(
                    requested,
                    CPIC_GENE_THRESHOLD,
                )
                .await?;
            Ok(requested_count >= CPIC_GENE_THRESHOLD)
        }),
    )
    .await?
    {
        crate::workflow_ladders::WorkflowProbeOutcome::Triggered(meta) => Some(meta),
        crate::workflow_ladders::WorkflowProbeOutcome::NotTriggered
        | crate::workflow_ladders::WorkflowProbeOutcome::Unavailable => None,
    };

    Ok(has_signal)
}

#[derive(serde::Serialize)]
pub(super) struct DrugSearchRegionBucket<T: serde::Serialize> {
    pagination: crate::cli::PaginationMeta,
    count: usize,
    results: Vec<T>,
}

#[derive(Default, serde::Serialize)]
pub(super) struct DrugSearchJsonRegions {
    #[serde(skip_serializing_if = "Option::is_none")]
    us: Option<DrugSearchRegionBucket<crate::entities::drug::DrugSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eu: Option<DrugSearchRegionBucket<crate::entities::drug::EmaDrugSearchResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    who: Option<DrugSearchRegionBucket<crate::entities::drug::WhoPrequalificationSearchResult>>,
}

#[derive(serde::Serialize)]
pub(super) struct DrugSearchJsonResponse {
    region: &'static str,
    regions: DrugSearchJsonRegions,
    #[serde(skip_serializing_if = "Option::is_none")]
    _meta: Option<crate::cli::SearchJsonMeta>,
}

pub(super) fn bucket_from_page<T: serde::Serialize>(
    page: crate::entities::SearchPage<T>,
    offset: usize,
    limit: usize,
) -> DrugSearchRegionBucket<T> {
    let count = page.results.len();
    DrugSearchRegionBucket {
        pagination: crate::cli::PaginationMeta::offset(offset, limit, count, page.total),
        count,
        results: page.results,
    }
}

fn drug_search_page_has_results(
    page_with_region: &crate::entities::drug::DrugSearchPageWithRegion,
) -> bool {
    match page_with_region {
        crate::entities::drug::DrugSearchPageWithRegion::Us(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::Eu(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::Who(page) => !page.results.is_empty(),
        crate::entities::drug::DrugSearchPageWithRegion::All { us, eu, who } => {
            !us.results.is_empty() || !eu.results.is_empty() || !who.results.is_empty()
        }
    }
}

pub(super) fn drug_search_json(
    page_with_region: crate::entities::drug::DrugSearchPageWithRegion,
    requested_name: Option<&str>,
    offset: usize,
    limit: usize,
    workflow: Option<crate::workflow_ladders::WorkflowMeta>,
) -> anyhow::Result<String> {
    let response = match page_with_region {
        crate::entities::drug::DrugSearchPageWithRegion::Us(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                Some(&page.results),
                None,
                None,
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Us.as_str(),
                regions: DrugSearchJsonRegions {
                    us: Some(bucket_from_page(page, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::Eu(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                None,
                Some(&page.results),
                None,
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Eu.as_str(),
                regions: DrugSearchJsonRegions {
                    eu: Some(bucket_from_page(page, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::Who(page) => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                None,
                None,
                Some(&page.results),
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::Who.as_str(),
                regions: DrugSearchJsonRegions {
                    who: Some(bucket_from_page(page, offset, limit)),
                    ..Default::default()
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow.clone()),
            }
        }
        crate::entities::drug::DrugSearchPageWithRegion::All { us, eu, who } => {
            let next_commands = crate::render::markdown::search_next_commands_drug_regions(
                requested_name,
                Some(&us.results),
                Some(&eu.results),
                Some(&who.results),
            );
            DrugSearchJsonResponse {
                region: crate::entities::drug::DrugRegion::All.as_str(),
                regions: DrugSearchJsonRegions {
                    us: Some(bucket_from_page(us, offset, limit)),
                    eu: Some(bucket_from_page(eu, offset, limit)),
                    who: Some(bucket_from_page(who, offset, limit)),
                },
                _meta: crate::cli::search_meta_with_workflow(next_commands, None, workflow),
            }
        }
    };

    crate::render::json::to_pretty(&response).map_err(Into::into)
}
