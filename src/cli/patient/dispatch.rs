use super::{PatientGetArgs, PatientSearchArgs};
use crate::cli::CommandOutcome;
use crate::entities::patient::search::{PatientSearchOutcome, PatientSearchRow};

pub(in crate::cli) async fn handle_get(
    args: PatientGetArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let (sections, json_override) = super::super::extract_json_from_sections(&args.sections);
    let json_output = json || json_override;
    let patient = crate::entities::patient::get(&args.id, &sections).await?;
    let text = if json_output {
        crate::render::json::to_entity_json(
            &patient,
            Vec::<(&str, String)>::new(),
            crate::render::markdown::patient_next_commands(&patient, &sections),
            crate::render::provenance::patient_section_sources(&patient),
        )?
    } else {
        crate::render::markdown::patient_markdown(&patient, &sections)?
    };
    Ok(CommandOutcome::stdout(text))
}

#[derive(serde::Serialize)]
struct PatientSearchJsonResponse {
    count: usize,
    results: Vec<PatientSearchRow>,
    _meta: Option<crate::cli::SearchJsonMeta>,
}

#[derive(serde::Serialize)]
struct PatientCountJsonResponse {
    /// The count the server reported, or null when it sent none.
    server_reported_count: Option<u64>,
}

pub(in crate::cli) async fn handle_search(
    args: PatientSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    // Every value is checked here, before any request.
    let filters = crate::entities::patient::search::parse_filters(
        args.gender.as_deref(),
        args.born_after.as_deref(),
        args.born_before.as_deref(),
        args.condition.as_deref(),
    )?;
    let outcome =
        crate::entities::patient::search::search(&filters, args.count, args.limit).await?;
    let text = match outcome {
        PatientSearchOutcome::Rows(rows) => {
            let next_commands = crate::render::markdown::search_next_commands_patient(&rows);
            if json {
                crate::render::json::to_pretty(&PatientSearchJsonResponse {
                    count: rows.len(),
                    results: rows,
                    _meta: crate::cli::search_meta(next_commands),
                })?
            } else {
                crate::render::markdown::patient_search_markdown(&rows)?
            }
        }
        PatientSearchOutcome::Count(total) => {
            if json {
                crate::render::json::to_pretty(&PatientCountJsonResponse {
                    server_reported_count: total,
                })?
            } else {
                crate::render::markdown::patient_count_markdown(total)
            }
        }
    };
    Ok(CommandOutcome::stdout(text))
}
