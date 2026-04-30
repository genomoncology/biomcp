use super::{TrialGetArgs, TrialSearchArgs};
use crate::cli::CommandOutcome;

pub(in crate::cli) async fn handle_get(
    args: TrialGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, legacy_offset, legacy_limit) = parse_trial_location_paging(&args.sections)?;
    if args.offset.is_some() && legacy_offset.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--offset supplied twice; place named options before 'locations'".into(),
        )
        .into());
    }
    if args.limit.is_some() && legacy_limit.is_some() {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit supplied twice; place named options before 'locations'".into(),
        )
        .into());
    }
    let location_offset = args.offset.or(legacy_offset);
    let location_limit = args.limit.or(legacy_limit);
    if location_limit.is_some_and(|value| value == 0) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be >= 1 for trial location pagination".into(),
        )
        .into());
    }
    let (sections, json_override) = super::super::extract_json_from_sections(&sections);
    let json_output = json || json_override;
    let trial_source = crate::entities::trial::TrialSource::from_flag(&args.source)?;
    let includes_locations = sections
        .iter()
        .any(|section| section.trim().eq_ignore_ascii_case("locations"));
    if !includes_locations && (location_offset.is_some() || location_limit.is_some()) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--offset and --limit are only valid with the 'locations' section".into(),
        )
        .into());
    }

    let mut trial = crate::entities::trial::get(&args.nct_id, &sections, trial_source).await?;
    let mut location_pagination = None;
    if includes_locations {
        let offset = location_offset.unwrap_or(0);
        let limit = location_limit.unwrap_or(20);
        location_pagination = Some(paginate_trial_locations(&mut trial, offset, limit));
    }

    let text = if json_output {
        if let Some(loc_page) = location_pagination {
            trial_locations_json(&trial, loc_page)?
        } else {
            crate::render::json::to_entity_json(
                &trial,
                crate::render::markdown::trial_evidence_urls(&trial),
                crate::render::markdown::related_trial(&trial),
                crate::render::provenance::trial_section_sources(&trial),
            )?
        }
    } else {
        let mut md = crate::render::markdown::trial_markdown(&trial, &sections)?;
        if let Some(loc_page) = location_pagination {
            md.push_str(&format!(
                "\n\n---\n*Locations: showing {} of {} (offset {}, limit {}{})*",
                trial.locations.as_ref().map_or(0, |value| value.len()),
                loc_page.total,
                loc_page.offset,
                loc_page.limit,
                if loc_page.has_more {
                    ", more available"
                } else {
                    ""
                },
            ));
        }
        md
    };

    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) async fn handle_search(
    args: TrialSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let positional_trial_query = args
        .positional_query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let condition = super::super::resolve_query_input(
        super::super::normalize_cli_tokens(args.condition),
        args.positional_query,
        "--condition",
    )?;
    let intervention = super::super::normalize_cli_tokens(args.intervention);
    let facility = super::super::normalize_cli_tokens(args.facility);
    let mutation = super::super::normalize_cli_tokens(args.mutation);
    let criteria = super::super::normalize_cli_tokens(args.criteria);
    let biomarker = super::super::normalize_cli_tokens(args.biomarker);
    let prior_therapies = super::super::normalize_cli_tokens(args.prior_therapies);
    let progression_on = super::super::normalize_cli_tokens(args.progression_on);
    let sponsor = super::super::normalize_cli_tokens(args.sponsor);
    let trial_source = crate::entities::trial::TrialSource::from_flag(&args.source)?;
    let filters = crate::entities::trial::TrialSearchFilters {
        condition,
        intervention,
        no_alias_expand: args.no_alias_expand,
        facility,
        status: args.status,
        phase: args.phase,
        study_type: args.study_type,
        age: args.age,
        sex: args.sex,
        sponsor,
        sponsor_type: args.sponsor_type,
        date_from: args.date_from,
        date_to: args.date_to,
        mutation,
        criteria,
        biomarker,
        prior_therapies,
        progression_on,
        line_of_therapy: args.line_of_therapy,
        lat: args.lat,
        lon: args.lon,
        distance: args.distance,
        results_available: args.results_available,
        source: trial_source,
    };

    if args
        .next_page
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && args.offset > 0
    {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--next-page cannot be used together with --offset".into(),
        )
        .into());
    }

    let query_intervention = match filters
        .intervention
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(intervention)
            if matches!(
                trial_source,
                crate::entities::trial::TrialSource::ClinicalTrialsGov
            ) && !filters.no_alias_expand =>
        {
            match crate::entities::drug::resolve_trial_canonical_name(intervention).await {
                Ok(canonical) if !canonical.trim().is_empty() => Some(canonical),
                Err(_) => Some(intervention.to_string()),
                _ => Some(intervention.to_string()),
            }
        }
        Some(intervention) => Some(intervention.to_string()),
        None => None,
    };
    let query = trial_search_query_summary(
        &filters,
        query_intervention.as_deref(),
        args.offset,
        args.next_page.as_deref(),
    );
    let text = if args.count_only {
        let count = crate::entities::trial::count_all(&filters).await?;
        if json {
            use crate::entities::trial::TrialCount;

            #[derive(serde::Serialize)]
            struct TrialCountOnlyJson {
                total: Option<usize>,
                #[serde(skip_serializing_if = "Option::is_none")]
                approximate: Option<bool>,
            }

            let (total, approximate) = match count {
                TrialCount::Exact(total) => (Some(total), None),
                TrialCount::Approximate(total) => (Some(total), Some(true)),
                TrialCount::Unknown => (None, None),
            };
            crate::render::json::to_pretty(&TrialCountOnlyJson { total, approximate })?
        } else {
            match count {
                crate::entities::trial::TrialCount::Exact(total) => format!("Total: {total}"),
                crate::entities::trial::TrialCount::Approximate(total) => {
                    format!("Total: {total} (approximate, age post-filtered)")
                }
                crate::entities::trial::TrialCount::Unknown => {
                    "Total: unknown (traversal limit reached)".to_string()
                }
            }
        }
    } else {
        let page =
            crate::entities::trial::search_page(&filters, args.limit, args.offset, args.next_page)
                .await?;
        let results = page.results;
        let pagination = super::super::PaginationMeta::cursor(
            args.offset,
            args.limit,
            results.len(),
            page.total,
            page.next_page_token,
        );
        if json {
            let next_commands = crate::render::markdown::search_next_commands_trial(&results);
            return super::super::search_json_with_meta(results, pagination, next_commands)
                .map(CommandOutcome::stdout);
        }

        let footer = if matches!(
            trial_source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        ) {
            super::super::pagination_footer_cursor(&pagination)
        } else {
            super::super::pagination_footer_offset(&pagination)
        };
        let total = pagination.total.and_then(|value| u32::try_from(value).ok());
        let show_zero_result_nickname_hint = should_show_trial_zero_result_nickname_hint(
            positional_trial_query.as_deref(),
            trial_source,
            results.len(),
        );
        crate::render::markdown::trial_search_markdown_with_footer(
            &query,
            &results,
            total,
            &footer,
            show_zero_result_nickname_hint,
            positional_trial_query.as_deref(),
        )?
    };

    Ok(CommandOutcome::stdout(text))
}

fn parse_usize_arg(flag: &str, value: &str) -> Result<usize, crate::error::BioMcpError> {
    value.parse::<usize>().map_err(|_| {
        crate::error::BioMcpError::InvalidArgument(format!("{flag} must be a non-negative integer"))
    })
}

pub(super) type LocationPaging = (Vec<String>, Option<usize>, Option<usize>);

pub(super) fn parse_trial_location_paging(
    sections: &[String],
) -> Result<LocationPaging, crate::error::BioMcpError> {
    let mut cleaned: Vec<String> = Vec::new();
    let mut location_offset: Option<usize> = None;
    let mut location_limit: Option<usize> = None;
    let mut i = 0usize;
    while i < sections.len() {
        let token = sections[i].trim();
        if token.is_empty() {
            i += 1;
            continue;
        }

        if let Some(value) = token.strip_prefix("--offset=") {
            location_offset = Some(parse_usize_arg("--offset", value)?);
            i += 1;
            continue;
        }
        if token == "--offset" {
            let value = sections.get(i + 1).ok_or_else(|| {
                crate::error::BioMcpError::InvalidArgument(
                    "--offset requires a value for trial location pagination".into(),
                )
            })?;
            location_offset = Some(parse_usize_arg("--offset", value.trim())?);
            i += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("--limit=") {
            location_limit = Some(parse_usize_arg("--limit", value)?);
            i += 1;
            continue;
        }
        if token == "--limit" {
            let value = sections.get(i + 1).ok_or_else(|| {
                crate::error::BioMcpError::InvalidArgument(
                    "--limit requires a value for trial location pagination".into(),
                )
            })?;
            location_limit = Some(parse_usize_arg("--limit", value.trim())?);
            i += 2;
            continue;
        }
        cleaned.push(sections[i].clone());
        i += 1;
    }

    if location_limit.is_some_and(|value| value == 0) {
        return Err(crate::error::BioMcpError::InvalidArgument(
            "--limit must be >= 1 for trial location pagination".into(),
        ));
    }

    Ok((cleaned, location_offset, location_limit))
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct LocationPaginationMeta {
    pub(super) total: usize,
    pub(super) offset: usize,
    pub(super) limit: usize,
    pub(super) has_more: bool,
}

pub(super) fn trial_locations_json(
    trial: &crate::entities::trial::Trial,
    location_pagination: LocationPaginationMeta,
) -> anyhow::Result<String> {
    #[derive(serde::Serialize)]
    struct TrialWithLocationPagination<'a> {
        #[serde(flatten)]
        trial: &'a crate::entities::trial::Trial,
        location_pagination: LocationPaginationMeta,
    }

    crate::render::json::to_entity_json(
        &TrialWithLocationPagination {
            trial,
            location_pagination,
        },
        crate::render::markdown::trial_evidence_urls(trial),
        crate::render::markdown::related_trial(trial),
        crate::render::provenance::trial_section_sources(trial),
    )
    .map_err(Into::into)
}

pub(super) fn paginate_trial_locations(
    trial: &mut crate::entities::trial::Trial,
    offset: usize,
    limit: usize,
) -> LocationPaginationMeta {
    let locations = trial.locations.take().unwrap_or_default();
    let total = locations.len();
    let paged: Vec<_> = locations.into_iter().skip(offset).take(limit).collect();
    let has_more = offset.saturating_add(paged.len()) < total;
    trial.locations = Some(paged);
    LocationPaginationMeta {
        total,
        offset,
        limit,
        has_more,
    }
}

pub(super) fn trial_search_query_summary(
    filters: &crate::entities::trial::TrialSearchFilters,
    query_intervention: Option<&str>,
    offset: usize,
    next_page: Option<&str>,
) -> String {
    let shows_alias_opt_out = filters.no_alias_expand
        && matches!(
            filters.source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        )
        && filters
            .intervention
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty());

    vec![
        filters
            .condition
            .as_deref()
            .map(|v| format!("condition={v}")),
        query_intervention.map(|v| format!("intervention={v}")),
        shows_alias_opt_out.then(|| "alias_expand=off".to_string()),
        filters.facility.as_deref().map(|v| format!("facility={v}")),
        filters.age.map(|v| format!("age={v}")),
        filters.sex.as_deref().map(|v| format!("sex={v}")),
        filters.status.as_deref().map(|v| format!("status={v}")),
        filters.phase.as_deref().map(|v| format!("phase={v}")),
        filters
            .study_type
            .as_deref()
            .map(|v| format!("study_type={v}")),
        filters.sponsor.as_deref().map(|v| format!("sponsor={v}")),
        filters
            .sponsor_type
            .as_deref()
            .map(|v| format!("sponsor_type={v}")),
        filters
            .date_from
            .as_deref()
            .map(|v| format!("date_from={v}")),
        filters.date_to.as_deref().map(|v| format!("date_to={v}")),
        filters.mutation.as_deref().map(|v| format!("mutation={v}")),
        filters.criteria.as_deref().map(|v| format!("criteria={v}")),
        filters
            .biomarker
            .as_deref()
            .map(|v| format!("biomarker={v}")),
        filters
            .prior_therapies
            .as_deref()
            .map(|v| format!("prior_therapies={v}")),
        filters
            .progression_on
            .as_deref()
            .map(|v| format!("progression_on={v}")),
        filters
            .line_of_therapy
            .as_deref()
            .map(|v| format!("line_of_therapy={v}")),
        filters.lat.map(|v| format!("lat={v}")),
        filters.lon.map(|v| format!("lon={v}")),
        filters.distance.map(|v| format!("distance={v}")),
        matches!(filters.source, crate::entities::trial::TrialSource::NciCts)
            .then(|| "source=nci".to_string()),
        filters
            .results_available
            .then(|| "has_results=true".to_string()),
        (offset > 0).then(|| format!("offset={offset}")),
        next_page
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("next_page={value}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

pub(super) fn should_show_trial_zero_result_nickname_hint(
    positional_query: Option<&str>,
    source: crate::entities::trial::TrialSource,
    result_count: usize,
) -> bool {
    positional_query
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && matches!(
            source,
            crate::entities::trial::TrialSource::ClinicalTrialsGov
        )
        && result_count == 0
}
