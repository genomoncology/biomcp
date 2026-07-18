use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use roxmltree::{Document, Node};
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const HPA_BASE: &str = "https://www.proteinatlas.org";
const HPA_API: &str = "hpa";
const HPA_BASE_ENV: &str = "BIOMCP_HPA_BASE";

pub struct HpaClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl HpaClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(HPA_BASE, HPA_BASE_ENV),
        })
    }

    pub(crate) fn protein_data_plan(ensembl_id: &str) -> Result<RequestPlan, BioMcpError> {
        let ensembl_id = normalize_ensembl_id(ensembl_id)?;
        Ok(RequestPlan::get(format!("{ensembl_id}.xml")))
    }

    pub(crate) fn decode_protein_data_xml(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: Vec<u8>,
    ) -> Result<Option<String>, BioMcpError> {
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: HPA_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        reject_html_content_type(content_type, &bytes)?;

        String::from_utf8(bytes)
            .map(Some)
            .map_err(|_| BioMcpError::Api {
                api: HPA_API.to_string(),
                message: "Response body was not valid UTF-8 XML".to_string(),
            })
    }

    pub async fn protein_data(&self, ensembl_id: &str) -> Result<GeneHpa, BioMcpError> {
        let plan = Self::protein_data_plan(ensembl_id)?;
        let resp = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &plan,
        ))
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::HPA,
        ))
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::HPA),
        )
        .await?;

        let Some(xml) =
            Self::decode_protein_data_xml(status, content_type.as_ref(), bytes.to_vec()).map_err(
                |error| {
                    error.with_source_context(crate::error::SourceContext::retry(
                        crate::error::SourceProvider::HPA,
                    ))
                },
            )?
        else {
            return Ok(GeneHpa::default());
        };

        tokio::task::spawn_blocking(move || parse_gene_hpa(&xml))
            .await
            .map_err(|err| {
                BioMcpError::Api {
                    api: HPA_API.to_string(),
                    message: format!("XML parse task failed: {err}"),
                }
                .with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::HPA,
                ))
            })?
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::HPA,
                ))
            })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneHpa {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tissues: Vec<HpaTissueExpression>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcellular_main_location: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subcellular_additional_location: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reliability: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein_summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rna_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HpaTissueExpression {
    pub tissue: String,
    pub level: String,
}

fn reject_html_content_type(
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    let Some(content_type) = content_type else {
        return Ok(());
    };
    let Ok(raw) = content_type.to_str() else {
        return Ok(());
    };
    let media_type = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    if matches!(media_type.as_str(), "text/html" | "application/xhtml+xml") {
        return Err(BioMcpError::Api {
            api: HPA_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }
    Ok(())
}

fn normalize_ensembl_id(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim().to_ascii_uppercase();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Ensembl gene ID is required for HPA protein expression".into(),
        ));
    }

    let core = raw.split('.').next().unwrap_or(&raw).trim();
    if core.is_empty()
        || !core.starts_with("ENSG")
        || !core.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid Ensembl gene ID: {value}"
        )));
    }
    Ok(core.to_string())
}

fn parse_gene_hpa(xml: &str) -> Result<GeneHpa, BioMcpError> {
    let doc = Document::parse(xml).map_err(|source| BioMcpError::Api {
        api: HPA_API.to_string(),
        message: format!("Invalid XML response: {source}"),
    })?;
    let root = doc.root_element();
    let entry = if root.has_tag_name("entry") {
        root
    } else {
        direct_child(root, "entry").ok_or_else(|| BioMcpError::Api {
            api: HPA_API.to_string(),
            message: "HPA XML response did not contain an entry element".to_string(),
        })?
    };

    let mut out = GeneHpa::default();

    if let Some(node) = direct_child_with_attrs(
        entry,
        "tissueExpression",
        &[
            ("source", "HPA"),
            ("technology", "IHC"),
            ("assayType", "tissue"),
        ],
    ) {
        parse_tissue_expression(node, &mut out);
    }

    if let Some(node) = direct_child_with_attrs(
        entry,
        "cellExpression",
        &[("source", "HPA"), ("technology", "ICC/IF")],
    ) {
        parse_cell_expression(node, &mut out);
    }

    if let Some(node) = direct_child_with_attrs(
        entry,
        "rnaExpression",
        &[
            ("source", "HPA"),
            ("technology", "RNAseq"),
            ("assayType", "consensusTissue"),
        ],
    ) {
        parse_rna_expression(node, &mut out);
    }

    Ok(out)
}

fn parse_tissue_expression(node: Node<'_, '_>, out: &mut GeneHpa) {
    out.protein_summary = direct_child(node, "summary")
        .filter(|summary| summary.attribute("type") == Some("tissue"))
        .and_then(node_text);

    if out.reliability.is_none() {
        out.reliability = direct_child(node, "verification")
            .filter(|verification| verification.attribute("type") == Some("reliability"))
            .and_then(node_text)
            .and_then(|value| normalize_reliability(&value));
    }

    for data in element_children(node).filter(|child| child.has_tag_name("data")) {
        let Some(tissue) = direct_child(data, "tissue").and_then(node_text) else {
            continue;
        };
        let Some(level) = element_children(data)
            .find(|child| {
                child.has_tag_name("level") && child.attribute("type") == Some("expression")
            })
            .and_then(node_text)
            .and_then(|value| normalize_expression_level(&value))
        else {
            continue;
        };

        if out
            .tissues
            .iter()
            .any(|existing| existing.tissue.eq_ignore_ascii_case(&tissue))
        {
            continue;
        }

        out.tissues.push(HpaTissueExpression { tissue, level });
    }
}

fn parse_cell_expression(node: Node<'_, '_>, out: &mut GeneHpa) {
    if out.reliability.is_none() {
        out.reliability = direct_child(node, "verification")
            .filter(|verification| verification.attribute("type") == Some("reliability"))
            .and_then(node_text)
            .and_then(|value| normalize_reliability(&value));
    }

    for data in element_children(node).filter(|child| child.has_tag_name("data")) {
        for location in element_children(data).filter(|child| child.has_tag_name("location")) {
            let Some(text) = node_text(location) else {
                continue;
            };
            match location.attribute("status") {
                Some("main") => {
                    push_unique_case_insensitive(&mut out.subcellular_main_location, text)
                }
                Some("additional") => {
                    push_unique_case_insensitive(&mut out.subcellular_additional_location, text)
                }
                _ => {}
            }
        }
    }
}

fn parse_rna_expression(node: Node<'_, '_>, out: &mut GeneHpa) {
    let specificity = direct_child(node, "rnaSpecificity").and_then(|child| {
        child
            .attribute("specificity")
            .and_then(normalize_whitespace)
            .or_else(|| node_text(child))
    });
    let distribution = direct_child(node, "rnaDistribution").and_then(|child| {
        node_text(child).or_else(|| {
            child
                .attribute("description")
                .and_then(normalize_whitespace)
        })
    });

    out.rna_summary = match (specificity, distribution) {
        (Some(specificity), Some(distribution)) => Some(format!("{specificity}; {distribution}")),
        (Some(specificity), None) => Some(specificity),
        (None, Some(distribution)) => Some(distribution),
        (None, None) => None,
    };
}

fn direct_child_with_attrs<'a>(
    root: Node<'a, 'a>,
    tag: &str,
    attrs: &[(&str, &str)],
) -> Option<Node<'a, 'a>> {
    element_children(root).find(|child| {
        child.has_tag_name(tag)
            && attrs
                .iter()
                .all(|(name, expected)| child.attribute(*name) == Some(*expected))
    })
}

fn direct_child<'a>(node: Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> {
    element_children(node).find(|child| child.has_tag_name(tag))
}

fn element_children<'a>(node: Node<'a, 'a>) -> impl Iterator<Item = Node<'a, 'a>> {
    node.children().filter(|child| child.is_element())
}

fn node_text(node: Node<'_, '_>) -> Option<String> {
    let mut text = String::new();
    for child in node.children() {
        if let Some(part) = child.text() {
            text.push_str(part);
        }
    }
    normalize_whitespace(&text)
}

fn normalize_whitespace(value: &str) -> Option<String> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn normalize_expression_level(value: &str) -> Option<String> {
    let normalized = match value.trim().to_ascii_lowercase().as_str() {
        "high" => "High".to_string(),
        "medium" => "Medium".to_string(),
        "low" => "Low".to_string(),
        "not detected" => "Not detected".to_string(),
        other => title_case_words(other)?,
    };
    Some(normalized)
}

fn normalize_reliability(value: &str) -> Option<String> {
    let normalized = match value.trim().to_ascii_lowercase().as_str() {
        "enhanced" => "Enhanced".to_string(),
        "supported" => "Supported".to_string(),
        "approved" => "Approved".to_string(),
        "uncertain" => "Uncertain".to_string(),
        other => title_case_words(other)?,
    };
    Some(normalized)
}

fn title_case_words(value: &str) -> Option<String> {
    let words = value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut out = String::new();
            out.extend(first.to_uppercase());
            out.push_str(&chars.as_str().to_ascii_lowercase());
            out
        })
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>();
    (!words.is_empty()).then(|| words.join(" "))
}

fn push_unique_case_insensitive(values: &mut Vec<String>, value: String) {
    if values
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&value))
    {
        return;
    }
    values.push(value);
}

#[cfg(test)]
mod tests;
