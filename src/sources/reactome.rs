use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::utils::serde::StringOrVec;

const REACTOME_BASE: &str = "https://reactome.org/ContentService";
const REACTOME_API: &str = "reactome";
const REACTOME_BASE_ENV: &str = "BIOMCP_REACTOME_BASE";

pub struct ReactomeClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ReactomeClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(REACTOME_BASE, REACTOME_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::REACTOME,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::REACTOME),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::REACTOME,
            ))
        })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: REACTOME_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: REACTOME_API.to_string(),
            source,
        })
    }

    pub(crate) fn search_pathways_plan(
        query: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome query is required".into(),
            ));
        }

        let page_size = limit.clamp(1, 25).to_string();
        Ok(RequestPlan::get("search/query")
            .query("query", query)
            .query("species", "Homo sapiens")
            .query("pageSize", page_size))
    }

    fn map_search_response(
        resp: ReactomeSearchResponse,
        limit: usize,
    ) -> (Vec<ReactomePathwayHit>, Option<usize>) {
        let total_results = resp.total_results;

        let mut out = Vec::new();
        for row in resp.results {
            for entry in row.entries {
                let id = entry
                    .st_id
                    .or(entry.id)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty());
                let Some(id) = id else { continue };
                let Some(name) = entry
                    .name
                    .map(|v| strip_html(v.trim()))
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                out.push(ReactomePathwayHit { id, name });
                if out.len() >= limit {
                    return (out, total_results);
                }
            }
        }

        (out, total_results)
    }

    pub async fn search_pathways(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<(Vec<ReactomePathwayHit>, Option<usize>), BioMcpError> {
        let plan = Self::search_pathways_plan(query, limit)?;
        let resp: ReactomeSearchResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::map_search_response(resp, limit))
    }

    pub(crate) fn top_level_pathways_plan() -> RequestPlan {
        RequestPlan::get("data/pathways/top/Homo%20sapiens")
    }

    fn map_top_level_pathways(
        rows: Vec<ReactomeTopLevelPathway>,
        limit: usize,
    ) -> Vec<ReactomePathwayHit> {
        let mut out = Vec::new();
        for row in rows.into_iter().take(limit.clamp(1, 200)) {
            let Some(id) = row
                .st_id
                .or(row.id)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            let Some(name) = row
                .display_name
                .map(|v| strip_html(v.trim()))
                .filter(|v| !v.is_empty())
                .or_else(|| {
                    row.name
                        .first()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(strip_html)
                })
            else {
                continue;
            };
            out.push(ReactomePathwayHit { id, name });
        }
        out
    }

    pub async fn top_level_pathways(
        &self,
        limit: usize,
    ) -> Result<Vec<ReactomePathwayHit>, BioMcpError> {
        let plan = Self::top_level_pathways_plan();
        let rows: Vec<ReactomeTopLevelPathway> = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::map_top_level_pathways(rows, limit))
    }

    fn normalize_stable_id(st_id: &str) -> Result<String, BioMcpError> {
        let st_id = st_id.trim();
        if st_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome stable ID is required".into(),
            ));
        }
        Ok(st_id.to_string())
    }

    pub(crate) fn get_pathway_plan(st_id: &str) -> Result<RequestPlan, BioMcpError> {
        let st_id = Self::normalize_stable_id(st_id)?;
        Ok(RequestPlan::get(format!("data/query/{st_id}")))
    }

    fn map_pathway_record(
        resp: ReactomePathwayRecordRaw,
        fallback_id: &str,
    ) -> ReactomePathwayRecord {
        ReactomePathwayRecord {
            id: resp.st_id.unwrap_or_else(|| fallback_id.to_string()),
            name: resp.display_name.unwrap_or_else(|| fallback_id.to_string()),
            species: resp.species_name,
            summary: resp
                .summation
                .and_then(|v| v.into_iter().next())
                .and_then(|v| v.text)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        }
    }

    pub async fn get_pathway(&self, st_id: &str) -> Result<ReactomePathwayRecord, BioMcpError> {
        let fallback_id = Self::normalize_stable_id(st_id)?;
        let plan = Self::get_pathway_plan(&fallback_id)?;
        let resp: ReactomePathwayRecordRaw = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        Ok(Self::map_pathway_record(resp, &fallback_id))
    }

    pub(crate) fn participants_plan(st_id: &str) -> Result<RequestPlan, BioMcpError> {
        let st_id = Self::normalize_stable_id(st_id)?;
        Ok(RequestPlan::get(format!("data/participants/{st_id}")))
    }

    fn map_participants(resp: Vec<ReactomeParticipant>, limit: usize) -> Vec<String> {
        let mut out = Vec::new();
        for row in resp.into_iter().take(limit.clamp(1, 200)) {
            let Some(name) = row
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            out.push(name.to_string());
        }

        out
    }

    pub async fn participants(
        &self,
        st_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let plan = Self::participants_plan(st_id)?;
        let resp: Vec<ReactomeParticipant> = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        Ok(Self::map_participants(resp, limit))
    }

    pub(crate) fn contained_events_plan(st_id: &str) -> Result<RequestPlan, BioMcpError> {
        let st_id = Self::normalize_stable_id(st_id)?;
        Ok(RequestPlan::get(format!(
            "data/pathway/{st_id}/containedEvents"
        )))
    }

    fn map_contained_events(resp: Vec<ReactomeContainedEvent>, limit: usize) -> Vec<String> {
        let mut out = Vec::new();
        for row in resp.into_iter().take(limit.clamp(1, 200)) {
            let ReactomeContainedEvent::Event(row) = row else {
                continue;
            };
            let Some(name) = row
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            out.push(name.to_string());
        }
        out
    }

    pub async fn contained_events(
        &self,
        st_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let plan = Self::contained_events_plan(st_id)?;
        let resp: Vec<ReactomeContainedEvent> = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::map_contained_events(resp, limit))
    }
}

fn strip_html(value: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in value.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out.replace("  ", " ").trim().to_string()
}

#[derive(Debug, Clone)]
pub struct ReactomePathwayHit {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ReactomePathwayRecord {
    pub id: String,
    pub name: String,
    pub species: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSearchResponse {
    #[serde(rename = "totalResults")]
    total_results: Option<usize>,
    #[serde(default)]
    results: Vec<ReactomeSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSearchResult {
    #[serde(default)]
    entries: Vec<ReactomeSearchEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeSearchEntry {
    st_id: Option<String>,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeTopLevelPathway {
    st_id: Option<String>,
    id: Option<String>,
    display_name: Option<String>,
    #[serde(default)]
    name: StringOrVec,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomePathwayRecordRaw {
    st_id: Option<String>,
    display_name: Option<String>,
    species_name: Option<String>,
    summation: Option<Vec<ReactomeSummation>>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSummation {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeParticipant {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeEvent {
    display_name: Option<String>,
}

// dead-code reason: reactome::ReactomeContainedEvent preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ReactomeContainedEvent {
    Event(ReactomeEvent),
    Id(i64),
}

#[cfg(test)]
mod tests;
