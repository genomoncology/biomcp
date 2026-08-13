use crate::cli::CommandOutcome;
use crate::error::BioMcpError;

pub(in crate::cli) async fn handle(
    gene: String,
    version: Option<String>,
    capture_id: Option<String>,
    files: bool,
    offset: usize,
    limit: usize,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    if files && version.is_none() && capture_id.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "gene cspec --files requires --version or --capture-id".into(),
        )
        .into());
    }
    if files {
        let response = match capture_id {
            Some(capture_id) => crate::entities::gene::cspec::files_capture(&capture_id, &gene)?,
            None => {
                crate::entities::gene::cspec::retrieve_files(
                    &gene,
                    version.as_deref().expect("files requires a version"),
                )
                .await?
            }
        };
        let text = if json {
            crate::render::json::to_pretty(&response)?
        } else {
            render_files(&response)
        };
        return Ok(CommandOutcome::stdout(text));
    }
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

fn render_files(response: &crate::entities::gene::cspec::CspecFilesResponse) -> String {
    let mut out = format!(
        "# ClinGen CSpec attachments\n\nSpecification: {}\nCapture: {}\nSHA-256: {}\n\n",
        response.resource_iri, response.capture.capture_id, response.capture.source_sha256
    );
    for file in &response.attachments {
        out.push_str(&format!(
            "## {}\n\n- Filename: {}\n- Media type: {}\n- Declared size: {}\n- Attachment ID: {}\n- Download URL: {}\n\n",
            file.label,
            file.filename,
            file.media_type,
            file.declared_size.map_or_else(|| "not declared".into(), |size| size.to_string()),
            file.attachment_id,
            file.download_url,
        ));
    }
    out
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
