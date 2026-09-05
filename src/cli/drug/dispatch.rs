use super::render::{drug_search_json, render_drug_card_outcome};
use super::workflow::drug_search_page_has_results;
use super::{DrugCommand, DrugGetArgs, DrugSearchArgs, WhoProductTypeArg};
use crate::cli::CommandOutcome;
use crate::entities::drug::DrugRegion;
use crate::sources::who_pq::WhoProductTypeFilter;

pub(crate) async fn handle_get(
    args: DrugGetArgs,
    json: bool,
    alias_suggestions_as_json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (name, raw_sections) = match args.name.as_deref() {
        Some(name) => (name, args.args.as_slice()),
        None => (
            args.args.first().map(String::as_str).unwrap_or_default(),
            args.args.get(1..).unwrap_or_default(),
        ),
    };
    let (sections, json_override) = super::super::extract_json_from_sections(raw_sections);
    let region = args.region.map(DrugRegion::from);
    let json_output = json || json_override;
    render_drug_card_outcome(
        name,
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
    let who_product_type_arg = args.who_product_type;
    let (filters, region, who_product_type) = search_plan_from_args(&args)?;
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
            let footers = crate::render::markdown::DrugSearchRegionFooters {
                us: super::render::drug_region_continuation(
                    filters.query.as_deref(),
                    "us",
                    args.offset,
                    args.limit,
                    us.results.len(),
                    us.total
                        .is_some_and(|total| args.offset + us.results.len() < total),
                ),
                eu: super::render::drug_region_continuation(
                    filters.query.as_deref(),
                    "eu",
                    args.offset,
                    args.limit,
                    eu.results.len(),
                    eu.total
                        .is_some_and(|total| args.offset + eu.results.len() < total),
                ),
                who: super::render::drug_region_continuation(
                    filters.query.as_deref(),
                    "who",
                    args.offset,
                    args.limit,
                    who.results.len(),
                    who.total
                        .is_some_and(|total| args.offset + who.results.len() < total),
                ),
            };
            crate::render::markdown::drug_search_markdown_all_regions(
                &query_summary,
                &us.results,
                us.total,
                &eu.results,
                eu.total,
                &who.results,
                who.total,
                &footers,
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
                    let trial_source = validate_trial_args(&source, no_alias_expand)?;
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
                    reaction,
                    outcome,
                    serious,
                    date_from,
                    date_to,
                    suspect_only,
                    sex,
                    age_min,
                    age_max,
                    reporter,
                    count,
                    r#type,
                    limit,
                    offset,
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

                    let query_type =
                        crate::entities::adverse_event::AdverseEventQueryType::from_flag(&r#type)?;
                    if !matches!(
                        query_type,
                        crate::entities::adverse_event::AdverseEventQueryType::Faers
                    ) {
                        let outcome = crate::cli::adverse_event::handle_search(
                            crate::cli::adverse_event::AdverseEventSearchArgs {
                                drug: Some(name),
                                positional_query: None,
                                device: None,
                                manufacturer: None,
                                product_code: None,
                                reaction,
                                outcome,
                                serious,
                                date_from,
                                date_to,
                                suspect_only,
                                sex,
                                age_min,
                                age_max,
                                reporter,
                                count,
                                r#type,
                                source: "all".to_string(),
                                classification: None,
                                limit,
                                offset,
                            },
                            json,
                        )
                        .await?;
                        return Ok(outcome);
                    }

                    let filters = crate::entities::adverse_event::AdverseEventSearchFilters {
                        drug: Some(name.clone()),
                        reaction,
                        outcome,
                        serious,
                        since: date_from,
                        date_to,
                        suspect_only,
                        sex,
                        age_min,
                        age_max,
                        reporter,
                    };
                    let mut query_summary =
                        crate::entities::adverse_event::search_query_summary(&filters);
                    if let Some(count_field) =
                        count.as_deref().map(str::trim).filter(|v| !v.is_empty())
                    {
                        if query_summary.is_empty() {
                            query_summary = format!("count={count_field}");
                        } else {
                            query_summary = format!("{query_summary}, count={count_field}");
                        }
                    }
                    if offset > 0 {
                        query_summary = format!("{query_summary}, offset={offset}");
                    }
                    if let Some(count_field) = count.as_deref().map(str::trim) {
                        let response = crate::entities::adverse_event::search_count(
                            &filters,
                            count_field,
                            limit,
                        )
                        .await?;
                        if json {
                            #[derive(serde::Serialize)]
                            struct CountResponse {
                                query: String,
                                count_field: String,
                                buckets:
                                    Vec<crate::entities::adverse_event::AdverseEventCountBucket>,
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
                        let fetch_limit = super::super::paged_fetch_limit(limit, offset, 50)?;
                        let status = crate::entities::adverse_event::search_with_status(
                            &filters,
                            fetch_limit,
                            0,
                        )
                        .await?;
                        match status {
                            crate::entities::adverse_event::FaersSearchStatus::Results(
                                response,
                            ) => {
                                let (results, observed_total) =
                                    super::super::paginate_results(response.results, offset, limit);
                                super::super::log_pagination_truncation(
                                    observed_total,
                                    offset,
                                    results.len(),
                                );
                                let count_response = crate::entities::adverse_event::search_count(
                                    &filters,
                                    "patient.reaction.reactionmeddrapt.exact",
                                    10,
                                )
                                .await?;
                                let summary =
                                    crate::entities::adverse_event::summarize_count_buckets(
                                        response.summary.total_reports,
                                        results.len(),
                                        &count_response.buckets,
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
                                    crate::render::markdown::adverse_event_search_markdown_with_source_label(
                                    &query_summary,
                                    &results,
                                    &summary,
                                    "",
                                    empty_state_message,
                                    &[],
                                    None,
                                    "OpenFDA FAERS aggregate",
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
                                        percentage_context: None,
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
                            crate::entities::adverse_event::FaersSearchStatus::Unavailable => {
                                return Err(anyhow::anyhow!(
                                    "OpenFDA FAERS adverse events are unavailable"
                                ));
                            }
                        }
                    }
                }
                DrugCommand::Interactions {
                    name,
                    limit,
                    offset,
                } => {
                    if limit == 0 {
                        return Err(crate::error::BioMcpError::InvalidArgument(
                            "--limit for drug interactions must be at least 1".into(),
                        )
                        .into());
                    }
                    let effective_limit =
                        limit.min(crate::entities::drug::interactions::MAX_INTERACTION_LIMIT);
                    let report =
                        crate::entities::drug::interaction_report(name, effective_limit, offset)
                            .await?;
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

pub(super) fn search_plan_from_args(
    args: &DrugSearchArgs,
) -> Result<
    (
        crate::entities::drug::DrugSearchFilters,
        DrugRegion,
        WhoProductTypeFilter,
    ),
    crate::error::BioMcpError,
> {
    let region_arg = args.region;
    let who_product_type = args
        .who_product_type
        .map(|value| match value {
            WhoProductTypeArg::FinishedPharma => WhoProductTypeFilter::FinishedPharma,
            WhoProductTypeArg::Api => WhoProductTypeFilter::Api,
            WhoProductTypeArg::Vaccine => WhoProductTypeFilter::Vaccine,
        })
        .unwrap_or_default();
    if args.who_product_type.is_some()
        && !matches!(region_arg, Some(crate::cli::DrugRegionArg::Who))
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "The WHO-only --product-type filter requires explicit --region who; rerun with --region who.".into(),
        ));
    }

    let query = super::super::resolve_query_input(
        args.query.clone(),
        args.positional_query.clone(),
        "--query",
    )?;
    let filters = crate::entities::drug::DrugSearchFilters {
        query,
        target: args.target.clone(),
        indication: args.indication.clone(),
        mechanism: args.mechanism.clone(),
        drug_type: args.drug_type.clone(),
        atc: args.atc.clone(),
        pharm_class: args.pharm_class.clone(),
        interactions: args.interactions.clone(),
    };
    let region = resolve_drug_search_region(region_arg, &filters)?;
    if matches!(region, DrugRegion::Who)
        && matches!(who_product_type, WhoProductTypeFilter::Vaccine)
        && filters.has_structured_filters()
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "WHO vaccine search is plain name/brand only; remove structured filters or search by vaccine name with --region who --product-type vaccine.".into(),
        ));
    }
    Ok((filters, region, who_product_type))
}

pub(super) fn validate_trial_args(
    source: &str,
    no_alias_expand: bool,
) -> Result<crate::entities::trial::TrialSource, crate::error::BioMcpError> {
    let trial_source = crate::entities::trial::TrialSource::from_flag(source)?;
    if no_alias_expand
        && !matches!(
            trial_source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        )
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--no-alias-expand is only supported for CTGov intervention searches".into(),
        ));
    }
    Ok(trial_source)
}

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
