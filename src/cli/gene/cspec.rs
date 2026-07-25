use crate::cli::CommandOutcome;
use crate::error::BioMcpError;

pub(in crate::cli) async fn handle(
    gene: String,
    version: Option<String>,
    capture_id: Option<String>,
    offset: usize,
    limit: usize,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    let response = match capture_id {
        Some(capture_id) => serde_json::to_value(crate::entities::gene::cspec::page_capture(
            &capture_id,
            &gene,
            offset,
            limit,
        )?)?,
        None => {
            crate::entities::gene::cspec::retrieve(&gene, version.as_deref(), offset, limit).await?
        }
    };
    let _ = json;
    let text = crate::render::json::to_pretty(&response)?;
    Ok(CommandOutcome::stdout(text))
}

pub(in crate::cli) fn document(capture_id: String, json: bool) -> anyhow::Result<CommandOutcome> {
    if json {
        return Err(BioMcpError::InvalidArgument(
            "CSpec document raw retrieval does not support --json".into(),
        )
        .into());
    }
    Ok(CommandOutcome::stdout_bytes(
        crate::entities::gene::cspec::read_capture(&capture_id)?,
    ))
}
