use super::{PatientGetArgs, PatientSearchArgs, SEARCH_NOT_YET_AVAILABLE};
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

pub(in crate::cli) fn handle_search(_args: PatientSearchArgs) -> anyhow::Result<CommandOutcome> {
    Err(crate::error::BioMcpError::InvalidArgument(SEARCH_NOT_YET_AVAILABLE.into()).into())
}
