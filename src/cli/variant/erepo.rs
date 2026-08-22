use tokio::io::AsyncReadExt;

use crate::cli::CommandOutcome;
use crate::entities::variant::{ERepoBatchInput, ERepoResponse, retrieve_erepo};
use crate::error::BioMcpError;

pub(crate) const MAX_EREPO_INPUT_BYTES: usize = 65_536;

async fn read_limited_input<R>(reader: R) -> Result<Vec<u8>, BioMcpError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut bytes = Vec::new();
    reader
        .take((MAX_EREPO_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(BioMcpError::Io)?;
    if bytes.len() > MAX_EREPO_INPUT_BYTES {
        return Err(BioMcpError::InputTooLarge {
            limit_bytes: MAX_EREPO_INPUT_BYTES,
        });
    }
    Ok(bytes)
}

async fn read_input(path: &str) -> Result<Vec<u8>, BioMcpError> {
    if path == "-" {
        read_limited_input(tokio::io::stdin()).await
    } else {
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|_| BioMcpError::InvalidArgument("unable to read ERepo input file".into()))?;
        read_limited_input(file).await
    }
}

pub(super) struct Request {
    pub caid: Option<String>,
    pub input: Option<String>,
    pub gene: Option<String>,
    pub limit: usize,
    pub offset: usize,
    pub detail: bool,
    pub assertion: Option<String>,
    pub version: Option<String>,
}

pub(super) async fn handle(request: Request, json: bool) -> anyhow::Result<CommandOutcome> {
    let Request {
        caid,
        input,
        gene,
        limit,
        offset,
        detail,
        assertion,
        version,
    } = request;
    if let Some(gene) = gene {
        let response = crate::entities::variant::search_erepo_gene(&gene, limit, offset).await?;
        let text = if json {
            crate::render::json::to_pretty(&response)?
        } else {
            render_gene_markdown(&response)
        };
        return Ok(CommandOutcome::stdout(text));
    }
    if caid.is_some() && input.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "variant erepo CAid cannot be combined with --input".into(),
        )
        .into());
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
        let empty_caid_gene = empty_caid_gene(&response).await;
        let repository_assertion_count = match empty_caid_gene.as_deref() {
            Some(gene) => empty_caid_repository_assertion_count(gene).await.ok(),
            None => None,
        };
        render_markdown(
            &response,
            empty_caid_gene.as_deref(),
            repository_assertion_count,
        )
    };
    Ok(CommandOutcome::stdout(text))
}

fn render_gene_markdown(response: &crate::entities::variant::ERepoGenePage) -> String {
    let mut out = String::from("# ClinGen ERepo gene assertions\n\n");
    out.push_str(&format!(
        "Returned {} results at offset {} (limit {}; more: {}).\n\n",
        response.returned, response.offset, response.limit, response.has_more
    ));
    for row in &response.results {
        out.push_str(&format!(
            "## {}\n\n",
            row.caid
                .as_deref()
                .unwrap_or("CAID omitted by safety limit")
        ));
        if let Some(value) = &row.classification {
            out.push_str(&format!("- Classification: {value}\n"));
        }
        if let Some(value) = &row.condition {
            out.push_str(&format!("- Condition: {value}\n"));
        }
        if let Some(value) = &row.guideline_label {
            out.push_str(&format!("- Guideline: {value}\n"));
        }
        if let Some(value) = &row.expert_panel {
            out.push_str(&format!("- Expert panel: {value}\n"));
        }
        if let Some(value) = &row.published_date {
            out.push_str(&format!("- Published: {value}\n"));
        }
        out.push_str(&format!(
            "- HGVS: {} of {} shown\n",
            row.hgvs.len(),
            row.hgvs_count
        ));
        if !row.met_evidence_codes.is_empty() {
            out.push_str(&format!(
                "- Met evidence codes: {}\n",
                row.met_evidence_codes.join(", ")
            ));
        }
        if !row.truncated_fields.is_empty() {
            out.push_str(&format!(
                "- Omitted oversized fields: {}\n",
                row.truncated_fields.join(", ")
            ));
        }
        out.push('\n');
    }
    out
}

async fn empty_caid_gene(response: &ERepoResponse) -> Option<String> {
    let [item] = response.items.as_slice() else {
        return None;
    };
    if !item.assertions.is_empty() {
        return None;
    }
    crate::sources::clingen_allele_registry::ClinGenAlleleRegistryClient::new()
        .ok()?
        .gene_for_caid(&item.caid)
        .await
}

async fn empty_caid_repository_assertion_count(gene: &str) -> Result<usize, BioMcpError> {
    const MAX_GENE_SEARCH_PAGE_SIZE: usize = 100;

    let mut count = 0;
    let mut offset = 0;
    loop {
        let page =
            crate::entities::variant::search_erepo_gene(gene, MAX_GENE_SEARCH_PAGE_SIZE, offset)
                .await?;
        count += page.returned;
        if !page.has_more {
            return Ok(count);
        }
        offset += page.returned;
    }
}

fn render_markdown(
    response: &ERepoResponse,
    empty_caid_gene: Option<&str>,
    repository_assertion_count: Option<usize>,
) -> String {
    let mut out = String::from("# ClinGen ERepo expert assertions\n\n");
    for item in &response.items {
        out.push_str(&format!("## {}\n\n", item.caid));
        if item.assertions.is_empty() {
            out.push_str("No expert assertions were returned.\n\n");
            if let Some(gene) = empty_caid_gene {
                out.push_str(&format!("Gene: {gene}\n"));
                if let Some(count) = repository_assertion_count {
                    out.push_str(&format!("repository assertions: {count} assertions\n"));
                }
                out.push('\n');
            }
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
            if let Some(label) = &assertion.guideline_label {
                out.push_str(&format!("- Guideline: {label}\n"));
                if let Some(version) = &assertion.guideline_version {
                    out.push_str(&format!("- Guideline version: {version}\n"));
                }
            }
            if let Some(condition) = &assertion.condition {
                out.push_str(&format!("- Condition: {condition}\n"));
            }
            if let Some(detail) = &assertion.detail {
                if let Some(summary) = &assertion.summary_description {
                    out.push_str(&format!("- Summary: {summary}\n"));
                }
                out.push_str(&format!("- Source: {}\n", detail.source_url));
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

    #[tokio::test]
    async fn input_reader_accepts_exact_limit_and_rejects_one_extra_byte() {
        let exact = read_limited_input(std::io::Cursor::new(vec![b' '; MAX_EREPO_INPUT_BYTES]))
            .await
            .expect("exact limit");
        assert_eq!(exact.len(), MAX_EREPO_INPUT_BYTES);

        let error = read_limited_input(std::io::Cursor::new(vec![b' '; MAX_EREPO_INPUT_BYTES + 1]))
            .await
            .expect_err("sentinel byte must be rejected");
        assert!(matches!(
            error,
            BioMcpError::InputTooLarge {
                limit_bytes: MAX_EREPO_INPUT_BYTES
            }
        ));
    }

    #[test]
    fn markdown_reports_source_facts_without_json() {
        let response = ERepoResponse {
            items: vec![ERepoItem {
                caid: "CA015543".into(),
                assertions: vec![ERepoAssertion {
                    assertion_id: "assertion-id".into(),
                    doc_version: "1.0.0".into(),
                    guideline_label: Some("Example specifications Version 2.1.0".into()),
                    guideline_version: Some("2.1.0".into()),
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

        let markdown = render_markdown(&response, None, None);
        assert!(markdown.contains("# ClinGen ERepo expert assertions"));
        assert!(markdown.contains("Classification: Pathogenic"));
        assert!(markdown.contains("met: PS4"));
    }
}
