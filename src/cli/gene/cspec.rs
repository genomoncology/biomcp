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
    let text = if json {
        crate::render::json::to_pretty(&response)?
    } else {
        render_markdown(&response)
    };
    Ok(CommandOutcome::stdout(text))
}

fn render_markdown(response: &serde_json::Value) -> String {
    if let Some(resource_iris) = response
        .get("resource_iris")
        .and_then(serde_json::Value::as_array)
    {
        return render_manifest_markdown(response, resource_iris);
    }
    render_page_markdown(response)
}

fn render_manifest_markdown(
    response: &serde_json::Value,
    resource_iris: &[serde_json::Value],
) -> String {
    let mut out = format!(
        "# ClinGen CSpec specifications\n\nGene: {}\nProvider: {}\n\n## Available resources\n\n",
        required_value(response, "gene"),
        required_value(response, "provider"),
    );
    for iri in resource_iris {
        if let Some(iri) = iri.as_str() {
            out.push_str(&format!(
                "- {}\n",
                crate::render::human::sanitize_inline(iri)
            ));
        }
    }
    out
}

fn render_page_markdown(response: &serde_json::Value) -> String {
    let criteria = response
        .get("criteria")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut out = format!(
        "# ClinGen CSpec criteria\n\nSpecification: {}\nGene: {}\n\n",
        required_value(response, "resource_iri"),
        required_value(response, "gene"),
    );
    render_optional_fact(&mut out, "Disease", response, "disease");
    render_optional_fact(&mut out, "Expert panel", response, "vcep");
    render_optional_fact(&mut out, "Status", response, "status");
    out.push_str(&format!(
        "Current: {}\n",
        required_value(response, "current")
    ));
    out.push_str(&format!(
        "Attachments: {}\nPaging: {} returned at offset {} (limit {}; total {})\n\n",
        required_value(response, "attachment_count"),
        criteria.len(),
        required_value(response, "offset"),
        required_value(response, "limit"),
        required_value(response, "total"),
    ));
    out.push_str("## Capture provenance\n\n");
    for (label, field) in [
        ("Capture", "capture_id"),
        ("Source SHA-256", "source_sha256"),
        ("Source bytes", "byte_length"),
        ("Media type", "media_type"),
        ("Captured at", "captured_at"),
        ("Expires at", "expires_at"),
    ] {
        out.push_str(&format!("- {label}: {}\n", required_value(response, field)));
    }
    if let Some(binding) = response.get("capture_binding") {
        out.push_str(&format!(
            "- Bound gene: {}\n- Bound resource: {}\n- Bound specification: {}\n",
            required_value(binding, "normalized_gene"),
            required_value(binding, "resource_iri"),
            required_value(binding, "specification_id"),
        ));
    }
    out.push_str("\n## Criteria\n\n");
    for criterion in criteria {
        let label = optional_string(criterion, "label")
            .map(crate::render::human::sanitize_inline)
            .unwrap_or_else(|| "Unnamed criterion".into());
        out.push_str(&format!("### {label}\n\n"));
        for (label, field) in [
            ("Source ID", "source_id"),
            ("Code", "code"),
            ("Source text", "source_text"),
            ("Source strength", "source_strength"),
            ("Configuration", "configuration"),
            ("Thresholds", "thresholds"),
            ("Assay restrictions", "assay_restrictions"),
        ] {
            render_optional_fact(&mut out, label, criterion, field);
        }
        out.push_str(&format!(
            "- Source locator: {}\n- Capture hash: {}\n",
            required_value(criterion, "source_locator"),
            required_value(criterion, "capture_hash"),
        ));
        if let Some(citations) = criterion
            .get("citations")
            .and_then(serde_json::Value::as_array)
        {
            for citation in citations {
                if let Some(citation) = citation.as_str() {
                    out.push_str(&format!(
                        "- Citation: {}\n",
                        crate::render::human::sanitize_inline(citation)
                    ));
                }
            }
        }
        if let Some(fields) = criterion
            .get("truncated_fields")
            .and_then(serde_json::Value::as_array)
            .filter(|fields| !fields.is_empty())
        {
            let fields = fields
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(crate::render::human::sanitize_inline)
                .collect::<Vec<_>>();
            out.push_str(&format!(
                "- Omitted oversized fields: {}\n",
                fields.join(", ")
            ));
        }
        out.push('\n');
    }
    out
}

fn required_value(value: &serde_json::Value, field: &str) -> String {
    match value.get(field) {
        Some(serde_json::Value::String(value)) => crate::render::human::sanitize_inline(value),
        Some(value) => crate::render::human::sanitize_inline(&value.to_string()),
        None => "not provided".into(),
    }
}

fn optional_string<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value.get(field).and_then(serde_json::Value::as_str)
}

fn render_optional_fact(out: &mut String, label: &str, value: &serde_json::Value, field: &str) {
    if let Some(value) = optional_string(value, field) {
        out.push_str(&format!(
            "{label}: {}\n",
            crate::render::human::sanitize_inline(value)
        ));
    }
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
