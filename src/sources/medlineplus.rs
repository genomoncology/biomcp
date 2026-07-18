use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::sync::OnceLock;

use regex::Regex;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use roxmltree::Document;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const MEDLINEPLUS_BASE: &str = "https://wsearch.nlm.nih.gov";
const MEDLINEPLUS_API: &str = "medlineplus";
const MEDLINEPLUS_BASE_ENV: &str = "BIOMCP_MEDLINEPLUS_BASE";
const MEDLINEPLUS_MAX_RETMAX: u8 = 50;

pub struct MedlinePlusClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MedlinePlusClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MEDLINEPLUS_BASE, MEDLINEPLUS_BASE_ENV),
        })
    }

    fn search_plan(query: &str, retmax: u8) -> Result<Option<RequestPlan>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(None);
        }
        if retmax == 0 || retmax > MEDLINEPLUS_MAX_RETMAX {
            return Err(BioMcpError::InvalidArgument(format!(
                "MedlinePlus retmax must be between 1 and {MEDLINEPLUS_MAX_RETMAX}"
            )));
        }

        Ok(Some(
            RequestPlan::get("ws/query")
                .query("db", "healthTopics")
                .query("term", query)
                .query("retmax", retmax.to_string()),
        ))
    }

    fn decode_response_body(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: Vec<u8>,
    ) -> Result<String, BioMcpError> {
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: MEDLINEPLUS_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }

        reject_html_content_type(content_type, &bytes)?;

        String::from_utf8(bytes).map_err(|_| BioMcpError::Api {
            api: MEDLINEPLUS_API.to_string(),
            message: "Response body was not valid UTF-8 XML".to_string(),
        })
    }

    pub async fn search_n(
        &self,
        query: &str,
        retmax: u8,
    ) -> Result<Vec<MedlinePlusTopic>, BioMcpError> {
        let Some(plan) = Self::search_plan(query, retmax)? else {
            return Ok(Vec::new());
        };

        let resp = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &plan,
        ))
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::MEDLINEPLUS,
        ))
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MEDLINEPLUS),
        )
        .await?;
        let xml =
            Self::decode_response_body(status, content_type.as_ref(), bytes).map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::MEDLINEPLUS,
                ))
            })?;

        tokio::task::spawn_blocking(move || parse_topics(&xml))
            .await
            .map_err(|err| {
                BioMcpError::Api {
                    api: MEDLINEPLUS_API.to_string(),
                    message: format!("XML parse task failed: {err}"),
                }
                .with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::MEDLINEPLUS,
                ))
            })?
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::MEDLINEPLUS,
                ))
            })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<MedlinePlusTopic>, BioMcpError> {
        self.search_n(query, 3).await
    }
}

#[derive(Debug, Clone)]
pub struct MedlinePlusTopic {
    pub title: String,
    pub url: String,
    pub summary_excerpt: String,
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
            api: MEDLINEPLUS_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }
    Ok(())
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

fn strip_inline_html_tags(value: &str) -> String {
    static HTML_TAG_RE: OnceLock<Regex> = OnceLock::new();
    let re = HTML_TAG_RE.get_or_init(|| Regex::new(r"(?is)<[^>]+>").expect("valid regex"));
    re.replace_all(value, "").to_string()
}

fn clean_text(value: &str) -> String {
    strip_inline_html_tags(&decode_html_entities(value))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_topics(xml: &str) -> Result<Vec<MedlinePlusTopic>, BioMcpError> {
    let doc = Document::parse(xml).map_err(|source| BioMcpError::Api {
        api: MEDLINEPLUS_API.to_string(),
        message: format!("Invalid XML response: {source}"),
    })?;

    let mut out = Vec::new();
    for document in doc
        .descendants()
        .filter(|node| node.is_element() && node.has_tag_name("document"))
    {
        let url = document.attribute("url").unwrap_or_default().trim();
        if url.is_empty() {
            continue;
        }

        let mut title = String::new();
        let mut summary = String::new();
        for content in document
            .children()
            .filter(|child| child.is_element() && child.has_tag_name("content"))
        {
            let name = content.attribute("name").unwrap_or_default();
            let text = clean_text(content.text().unwrap_or_default());
            if text.is_empty() {
                continue;
            }
            match name {
                "title" if title.is_empty() => title = text,
                "FullSummary" if summary.is_empty() => summary = text,
                "snippet" if summary.is_empty() => summary = text,
                _ => {}
            }
        }

        if title.is_empty() {
            continue;
        }

        out.push(MedlinePlusTopic {
            title,
            url: url.to_string(),
            summary_excerpt: summary,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests;
