use super::{PatientGetArgs, PatientSearchArgs};
use crate::cli::CommandOutcome;

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

pub(in crate::cli) async fn handle_search(
    args: PatientSearchArgs,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let filters = crate::entities::patient::PatientSearchFilters {
        gender: args.gender,
        born_after: args.born_after,
        born_before: args.born_before,
        condition: args.condition,
    };
    let text = if args.count {
        let total = crate::entities::patient::count(&filters).await?;
        if json {
            #[derive(serde::Serialize)]
            struct CountResponse {
                source: &'static str,
                server_reported_total: Option<u64>,
                #[serde(skip_serializing_if = "Option::is_none")]
                message: Option<&'static str>,
            }
            crate::render::json::to_pretty(&CountResponse {
                source: crate::sources::fhir::FHIR_SOURCE,
                server_reported_total: total,
                message: total
                    .is_none()
                    .then_some(crate::render::markdown::PATIENT_NO_COUNT),
            })?
        } else {
            crate::render::markdown::patient_count_markdown(total)
        }
    } else {
        let rows = crate::entities::patient::search(&filters, args.limit).await?;
        if json {
            let pagination =
                super::super::PaginationMeta::offset(0, args.limit, rows.len(), None);
            let next_commands = crate::render::markdown::patient_search_next_commands(&rows);
            super::super::search_json_with_meta(rows, pagination, next_commands)?
        } else {
            crate::render::markdown::patient_search_markdown(&rows, args.limit)?
        }
    };
    Ok(CommandOutcome::stdout(text))
}
