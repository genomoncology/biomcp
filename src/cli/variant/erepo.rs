use tokio::io::AsyncReadExt;

use crate::cli::CommandOutcome;
use crate::entities::variant::{ERepoBatchInput, ERepoResponse, retrieve_erepo};
use crate::error::BioMcpError;

async fn read_input(path: &str) -> Result<Vec<u8>, BioMcpError> {
    const READ_LIMIT: u64 = 64 * 1024 + 1;
    let mut bytes = Vec::new();
    if path == "-" {
        tokio::io::stdin()
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    } else {
        tokio::fs::File::open(path)
            .await
            .map_err(|_| BioMcpError::InvalidArgument("unable to read ERepo input file".into()))?
            .take(READ_LIMIT)
            .read_to_end(&mut bytes)
            .await
    }
    .map_err(|_| BioMcpError::InvalidArgument("unable to read ERepo input".into()))?;
    Ok(bytes)
}

pub(super) async fn handle(
    caid: Option<String>,
    input: Option<String>,
    detail: bool,
    assertion: Option<String>,
    version: Option<String>,
    json: bool,
) -> anyhow::Result<CommandOutcome> {
    if caid.is_some() && input.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo CAid cannot be combined with --input".into(),
        )
        .into());
    }
    if input.is_some() && !json {
        return Err(
            BioMcpError::InvalidArgument("variant erepo --input requires --json".into()).into(),
        );
    }
    let caids = match (caid, input) {
        (Some(caid), None) => vec![caid],
        (None, Some(path)) => serde_json::from_slice::<ERepoBatchInput>(&read_input(&path).await?)
            .map_err(|_| {
                BioMcpError::InvalidArgument(
                    "variant erepo input must be a JSON array or {\"caids\": [...]}".into(),
                )
            })?
            .into_caids(),
        _ => {
            return Err(BioMcpError::InvalidArgument(
                "variant erepo requires a CAid or --input".into(),
            )
            .into());
        }
    };
    if caids.len() > 1 && (detail || assertion.is_some() || version.is_some()) {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo detail selectors are only available for one CAid".into(),
        )
        .into());
    }
    let response = retrieve_erepo(caids, detail, assertion.as_deref(), version.as_deref()).await?;
    let text = if json {
        crate::render::json::to_pretty(&response)?
    } else {
        render_markdown(&response)
    };
    Ok(CommandOutcome::stdout(text))
}

fn render_markdown(response: &ERepoResponse) -> String {
    let mut out = String::from("# ClinGen ERepo expert assertions\n\n");
    for item in &response.items {
        out.push_str(&format!("## {}\n\n", item.caid));
        if item.assertions.is_empty() {
            out.push_str("No expert assertions were returned.\n\n");
            continue;
        }
        for assertion in &item.assertions {
            out.push_str(&format!(
                "### {} (version {})\n\n",
                assertion.assertion_id, assertion.doc_version
            ));
            if let Some(classification) = &assertion.classification {
                out.push_str(&format!("- Classification: {classification}\n"));
            }
            if let Some(condition) = &assertion.condition {
                out.push_str(&format!("- Condition: {condition}\n"));
            }
            out.push_str(&format!(
                "- Unmet criteria coverage: {}\n",
                assertion.unmet_codes_state
            ));
            for criterion in &assertion.criteria {
                out.push_str(&format!(
                    "- {}: {}\n",
                    criterion.status, criterion.source_token
                ));
            }
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::variant::{ERepoAssertion, ERepoCriterion, ERepoItem, ERepoSourceStatus};

    #[test]
    fn markdown_reports_source_facts_without_json() {
        let response = ERepoResponse {
            items: vec![ERepoItem {
                caid: "CA015543".into(),
                assertions: vec![ERepoAssertion {
                    assertion_id: "assertion-id".into(),
                    doc_version: "1.0.0".into(),
                    versions: vec!["1.0.0".into()],
                    classification: Some("Pathogenic".into()),
                    condition: Some("Example condition".into()),
                    mondo_id: None,
                    moi: None,
                    vcep: None,
                    gene: None,
                    gene_ncbi_id: None,
                    hgvs: Vec::new(),
                    preferred_variant_title: None,
                    approved_date: None,
                    published_date: None,
                    retracted: None,
                    pcer_doc_id: None,
                    summary_description: None,
                    source_url: "https://example.test".into(),
                    criteria: vec![ERepoCriterion {
                        source_token: "PS4".into(),
                        code: "PS4".into(),
                        status: "met",
                        explicit_strength: None,
                    }],
                    unmet_codes_state: "provided",
                    detail: None,
                }],
                complete: true,
            }],
            complete: true,
            source_status: vec![ERepoSourceStatus {
                source: "clingen_erepo",
                status: "available",
            }],
            provider: "ClinGen ERepo",
        };

        let markdown = render_markdown(&response);
        assert!(markdown.contains("# ClinGen ERepo expert assertions"));
        assert!(markdown.contains("Classification: Pathogenic"));
        assert!(markdown.contains("met: PS4"));
    }
}
