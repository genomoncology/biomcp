use crate::cli::CommandOutcome;
use crate::entities::trial::{TrialDocumentsManifest, TrialSource};
use crate::error::BioMcpError;

pub(super) async fn handle_document_get(
    nct_id: &str,
    sections: &[String],
    source: TrialSource,
    json_output: bool,
    location_options_present: bool,
) -> anyhow::Result<Option<CommandOutcome>> {
    if !document_route(sections) {
        return Ok(None);
    }
    if location_options_present {
        return Err(BioMcpError::InvalidArgument(
            "--offset and --limit are only valid with the 'locations' section".into(),
        )
        .into());
    }
    if matches!(source, TrialSource::NciCts) {
        return Err(BioMcpError::InvalidArgument(
            "Trial documents are available only from ClinicalTrials.gov; --source nci does not support documents or document retrieval."
                .into(),
        )
        .into());
    }
    if let Some(filename) = document_request(sections)? {
        let bytes = crate::entities::trial::trial_document_bytes(nct_id, &filename).await?;
        return Ok(Some(CommandOutcome::stdout_bytes(bytes)));
    }
    documents_request(sections)?;
    if !json_output {
        return Err(BioMcpError::InvalidArgument(
            "Trial document manifests are JSON-only; rerun with --json (example: biomcp --json get trial NCT03361748 documents)"
                .into(),
        )
        .into());
    }
    let manifest = crate::entities::trial::trial_documents_manifest(nct_id).await?;
    let commands = manifest_next_commands(&manifest);
    #[derive(serde::Serialize)]
    struct DocumentsResponse {
        #[serde(flatten)]
        manifest: TrialDocumentsManifest,
        #[serde(skip_serializing_if = "Option::is_none")]
        _meta: Option<super::super::SearchJsonMeta>,
    }
    Ok(Some(CommandOutcome::stdout(
        crate::render::json::to_pretty(&DocumentsResponse {
            manifest,
            _meta: crate::cli::search_meta(commands),
        })?,
    )))
}

fn document_route(sections: &[String]) -> bool {
    sections.first().is_some_and(|section| {
        matches!(
            section.trim().to_ascii_lowercase().as_str(),
            "document" | "documents"
        )
    })
}

fn documents_request(sections: &[String]) -> Result<(), BioMcpError> {
    if sections.len() != 1 || !sections[0].trim().eq_ignore_ascii_case("documents") {
        return Err(BioMcpError::InvalidArgument(
            "documents is a standalone JSON-only trial section; do not combine it with other sections"
                .into(),
        ));
    }
    Ok(())
}

fn document_request(sections: &[String]) -> Result<Option<String>, BioMcpError> {
    let Some(first) = sections.first() else {
        return Ok(None);
    };
    if !first.trim().eq_ignore_ascii_case("document") {
        return Ok(None);
    }
    if sections.len() != 2 || sections[1].trim().is_empty() {
        return Err(document_arity_error());
    }
    Ok(Some(sections[1].clone()))
}

fn document_arity_error() -> BioMcpError {
    BioMcpError::InvalidArgument(
        "document requires exactly one advertised filename and is a standalone raw-byte retrieval form (example: biomcp get trial NCT03361748 document Prot_SAP_000.pdf)"
            .into(),
    )
}

fn manifest_next_commands(manifest: &TrialDocumentsManifest) -> Vec<String> {
    let mut commands = vec![
        crate::next_command::NextCommand::biomcp()
            .args(["--json", "get", "trial", &manifest.nct_id, "documents"])
            .render_shell(),
    ];
    commands.extend(
        manifest
            .documents
            .iter()
            .filter_map(|document| document.handle.clone()),
    );
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sections(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn document_routes_require_standalone_arity() {
        assert!(documents_request(&sections(&["documents"])).is_ok());
        assert_eq!(
            document_request(&sections(&["document", "Protocol.pdf"]))
                .unwrap()
                .as_deref(),
            Some("Protocol.pdf")
        );
        assert_eq!(
            document_request(&sections(&["document", " documents "]))
                .unwrap()
                .as_deref(),
            Some(" documents ")
        );
        assert!(!document_route(&sections(&[
            "eligibility",
            "document",
            "Protocol.pdf"
        ])));
        for invalid in [
            sections(&["documents", "eligibility"]),
            sections(&["document"]),
            sections(&["eligibility", "document", "Protocol.pdf"]),
            sections(&["document", "Protocol.pdf", "eligibility"]),
        ] {
            assert!(documents_request(&invalid).is_err() || document_request(&invalid).is_err());
        }
    }

    #[tokio::test]
    async fn manifest_is_json_only_and_document_forms_are_ctgov_only() {
        let error = handle_document_get(
            "NCT03361748",
            &sections(&["documents"]),
            TrialSource::ClinicalTrialsGov,
            false,
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("JSON-only"));

        let error = handle_document_get(
            "NCT03361748",
            &sections(&["documents"]),
            TrialSource::NciCts,
            true,
            false,
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("only from ClinicalTrials.gov"));

        let error = handle_document_get(
            "NCT03361748",
            &sections(&["documents"]),
            TrialSource::ClinicalTrialsGov,
            true,
            true,
        )
        .await
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("only valid with the 'locations' section")
        );
    }
}
