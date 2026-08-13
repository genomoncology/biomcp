use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use roxmltree::{Document, Node};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const VAERS_BASE: &str = "https://wonder.cdc.gov";
const VAERS_API: &str = "vaers";
pub(crate) const VAERS_BASE_ENV: &str = "BIOMCP_VAERS_BASE";
const VAERS_REQUEST_PATH: &str = "/controller/datarequest/D8";
const VAERS_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
const LIVE_REQUEST_GAP: Duration = Duration::from_secs(16);
const CDC_WONDER_COMPATIBLE_USER_AGENT: &str = "Wget/1.21.4";
const REQUEST_TEMPLATE: &str = include_str!("../../spec/fixtures/vaers/reactions-request.xml");
const REQUEST_RESTRICTIONS_FLAG: (&str, &str) = ("accept_datause_restrictions", "true");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VaersAggregateKind {
    Reactions,
    Seriousness,
    Age,
}

impl VaersAggregateKind {
    fn group_by_code(self) -> &'static str {
        match self {
            Self::Reactions => "D8.V13-level2",
            Self::Seriousness => "D8.V10",
            Self::Age => "D8.V1",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VaersAggregateRow {
    pub label: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VaersAggregateTable {
    pub total_events: usize,
    pub rows: Vec<VaersAggregateRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VaersSeriousBreakdown {
    pub total_reports: usize,
    pub serious_reports: usize,
    pub non_serious_reports: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VaersSummaryTables {
    pub total_reports: usize,
    pub reactions: Vec<VaersAggregateRow>,
    pub serious_reports: usize,
    pub non_serious_reports: usize,
    pub age_distribution: Vec<VaersAggregateRow>,
}

pub(crate) struct VaersClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl VaersClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(VAERS_BASE, VAERS_BASE_ENV);
        Ok(Self {
            client: vaers_http_client(base.as_ref())?,
            base,
        })
    }

    fn uses_live_rate_limit_delay(&self) -> bool {
        self.base.as_ref().contains("wonder.cdc.gov")
    }

    async fn pause_between_live_requests(&self) {
        if self.uses_live_rate_limit_delay() {
            tokio::time::sleep(LIVE_REQUEST_GAP).await;
        }
    }

    pub(crate) async fn health_check(&self) -> Result<(), BioMcpError> {
        self.reaction_counts("MMR").await.map(|_| ())
    }

    pub(crate) async fn reaction_counts(
        &self,
        wonder_code: &str,
    ) -> Result<VaersAggregateTable, BioMcpError> {
        self.fetch_aggregate_table(VaersAggregateKind::Reactions, wonder_code)
            .await
    }

    pub(crate) async fn seriousness_breakdown(
        &self,
        wonder_code: &str,
    ) -> Result<VaersSeriousBreakdown, BioMcpError> {
        let table = self
            .fetch_aggregate_table(VaersAggregateKind::Seriousness, wonder_code)
            .await?;
        let serious_reports = table
            .rows
            .iter()
            .find(|row| row.label.eq_ignore_ascii_case("yes"))
            .map(|row| row.count)
            .unwrap_or_default();
        let non_serious_reports = table
            .rows
            .iter()
            .find(|row| row.label.eq_ignore_ascii_case("no"))
            .map(|row| row.count)
            .unwrap_or_default();

        Ok(VaersSeriousBreakdown {
            total_reports: table.total_events,
            serious_reports,
            non_serious_reports,
        })
    }

    pub(crate) async fn age_distribution(
        &self,
        wonder_code: &str,
    ) -> Result<VaersAggregateTable, BioMcpError> {
        self.fetch_aggregate_table(VaersAggregateKind::Age, wonder_code)
            .await
    }

    pub(crate) async fn summary(
        &self,
        wonder_code: &str,
    ) -> Result<VaersSummaryTables, BioMcpError> {
        let reactions = self.reaction_counts(wonder_code).await?;
        self.pause_between_live_requests().await;
        let seriousness = self.seriousness_breakdown(wonder_code).await?;
        self.pause_between_live_requests().await;
        let age_distribution = self.age_distribution(wonder_code).await?;

        let total_reports = reactions
            .total_events
            .max(seriousness.total_reports)
            .max(age_distribution.total_events);

        Ok(VaersSummaryTables {
            total_reports,
            reactions: reactions.rows,
            serious_reports: seriousness.serious_reports,
            non_serious_reports: seriousness.non_serious_reports,
            age_distribution: age_distribution.rows,
        })
    }

    async fn fetch_aggregate_table(
        &self,
        kind: VaersAggregateKind,
        wonder_code: &str,
    ) -> Result<VaersAggregateTable, BioMcpError> {
        let plan = aggregate_request_plan(kind, wonder_code)?;
        let response = self
            .request_from_plan(&plan)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::VAERS,
            ))
            .await?;
        let status = response.status();
        let content_type = response.headers().get(CONTENT_TYPE).cloned();
        let body = crate::sources::read_limited_source_body_with_limit(
            response,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::VAERS),
            VAERS_MAX_BODY_BYTES,
        )
        .await?;
        let xml =
            decode_aggregate_response(status, content_type.as_ref(), body).map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::VAERS,
                ))
            })?;
        tokio::task::spawn_blocking(move || parse_aggregate_response(&xml))
            .await
            .map_err(|err| {
                BioMcpError::Api {
                    api: VAERS_API.to_string(),
                    message: format!("CDC WONDER VAERS XML parse task failed: {err}"),
                }
                .with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::VAERS,
                ))
            })?
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::VAERS,
                ))
            })
    }

    fn request_from_plan(&self, plan: &RequestPlan) -> reqwest_middleware::RequestBuilder {
        request_from_plan(&self.client, self.base.as_ref(), plan)
    }
}

fn vaers_http_client(base: &str) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError> {
    // CDC WONDER's edge denies the shared `biomcp-cli/<version>` user-agent for
    // VAERS XML POSTs while allowing common command-line clients.
    crate::sources::ordinary_middleware_client_for_base(base, VAERS_BASE_ENV, |builder| {
        builder
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(CDC_WONDER_COMPATIBLE_USER_AGENT)
    })
}

fn aggregate_request_plan(
    kind: VaersAggregateKind,
    wonder_code: &str,
) -> Result<RequestPlan, BioMcpError> {
    let request_xml = build_request_xml(kind, wonder_code)?;
    Ok(RequestPlan::post(VAERS_REQUEST_PATH).form(vec![
        ("request_xml".to_string(), request_xml),
        (
            REQUEST_RESTRICTIONS_FLAG.0.to_string(),
            REQUEST_RESTRICTIONS_FLAG.1.to_string(),
        ),
    ]))
}

fn build_request_xml(kind: VaersAggregateKind, wonder_code: &str) -> Result<String, BioMcpError> {
    let wonder_code = wonder_code.trim();
    if wonder_code.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "CDC WONDER VAERS query requires a vaccine code".into(),
        ));
    }

    let request = REQUEST_TEMPLATE.replace(
        "<name>B_1</name><value>D8.V13-level2</value>",
        &format!("<name>B_1</name><value>{}</value>", kind.group_by_code()),
    );
    Ok(request.replace(
        "<name>F_D8.V14</name><value>MMR</value>",
        &format!(
            "<name>F_D8.V14</name><value>{}</value>",
            xml_escape(wonder_code)
        ),
    ))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn reject_html_gateway(content_type: Option<&HeaderValue>, body: &[u8]) -> Result<(), BioMcpError> {
    let sniff = String::from_utf8_lossy(&body[..body.len().min(128)])
        .trim_start()
        .to_ascii_lowercase();
    if sniff.starts_with("<!doctype html") || sniff.starts_with("<html") {
        return Err(BioMcpError::Api {
            api: VAERS_API.to_string(),
            message: format!(
                "CDC WONDER VAERS returned an HTML gateway page: {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }

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
    if matches!(media_type.as_str(), "text/html" | "application/xhtml+xml")
        && !(sniff.starts_with("<?xml") || sniff.starts_with("<page"))
    {
        return Err(BioMcpError::Api {
            api: VAERS_API.to_string(),
            message: format!(
                "CDC WONDER VAERS returned unexpected HTML content: {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }
    Ok(())
}

fn decode_aggregate_response(
    status: StatusCode,
    content_type: Option<&HeaderValue>,
    body: Vec<u8>,
) -> Result<String, BioMcpError> {
    if !status.is_success() {
        let excerpt = crate::sources::summarize_http_error_body(content_type, &body);
        return Err(BioMcpError::Api {
            api: VAERS_API.to_string(),
            message: format!("CDC WONDER VAERS HTTP {status}: {excerpt}"),
        });
    }

    reject_html_gateway(content_type, &body)?;

    String::from_utf8(body).map_err(|_| BioMcpError::Api {
        api: VAERS_API.to_string(),
        message: "CDC WONDER VAERS response body was not valid UTF-8 XML".to_string(),
    })
}

fn parse_aggregate_response(xml: &str) -> Result<VaersAggregateTable, BioMcpError> {
    let doc = Document::parse(xml).map_err(|source| BioMcpError::Api {
        api: VAERS_API.to_string(),
        message: format!("Invalid CDC WONDER VAERS XML response: {source}"),
    })?;

    if let Some(message) = processing_error_message(&doc) {
        return Err(BioMcpError::Api {
            api: VAERS_API.to_string(),
            message,
        });
    }

    let data_table = doc
        .descendants()
        .find(|node| node.has_tag_name("data-table"))
        .ok_or_else(|| BioMcpError::Api {
            api: VAERS_API.to_string(),
            message: "CDC WONDER VAERS response did not contain a data-table".to_string(),
        })?;
    let total_events = total_events_message(&doc).ok_or_else(|| BioMcpError::Api {
        api: VAERS_API.to_string(),
        message: "CDC WONDER VAERS response did not report total events".to_string(),
    })?;

    let mut rows = Vec::new();
    for row in element_children(data_table).filter(|node| node.has_tag_name("r")) {
        let cells = element_children(row)
            .filter(|node| node.has_tag_name("c"))
            .collect::<Vec<_>>();
        if cells.len() < 3 {
            continue;
        }
        if cells[1].attribute("dt").is_some() {
            continue;
        }
        let Some(label) = cells[0]
            .attribute("l")
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let Some(count) = cells[1].attribute("v").and_then(parse_number) else {
            continue;
        };
        let Some(percentage) = cells[2].attribute("v").and_then(parse_percentage) else {
            continue;
        };
        rows.push(VaersAggregateRow {
            label: label.to_string(),
            count,
            percentage,
        });
    }

    Ok(VaersAggregateTable { total_events, rows })
}

fn processing_error_message(doc: &Document<'_>) -> Option<String> {
    let title = doc
        .descendants()
        .find(|node| node.has_tag_name("title"))
        .and_then(node_text)
        .unwrap_or_default();
    let message = doc
        .descendants()
        .filter(|node| node.has_tag_name("message"))
        .find_map(node_text)
        .unwrap_or_default();

    if title.eq_ignore_ascii_case("Processing Error") {
        let trimmed = message.trim();
        return Some(if trimmed.is_empty() {
            "CDC WONDER VAERS returned a processing error".to_string()
        } else {
            format!("CDC WONDER VAERS returned a processing error: {trimmed}")
        });
    }
    None
}

fn total_events_message(doc: &Document<'_>) -> Option<usize> {
    doc.descendants()
        .filter(|node| node.has_tag_name("message"))
        .filter_map(node_text)
        .find_map(|text| {
            let marker = "These results are for ";
            let start = text.find(marker)?;
            let remainder = &text[start + marker.len()..];
            let end = remainder.find(" total events")?;
            parse_number(&remainder[..end])
        })
}

fn parse_number(raw: &str) -> Option<usize> {
    let digits = raw
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<usize>().ok()
    }
}

fn parse_percentage(raw: &str) -> Option<f64> {
    raw.trim().trim_end_matches('%').parse::<f64>().ok()
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
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests;
