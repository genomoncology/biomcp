use std::borrow::Cow;
use std::collections::HashSet;

use http_cache_reqwest::CacheMode;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const WIKIPATHWAYS_BASE: &str = "https://www.wikipathways.org/json";
const WIKIPATHWAYS_API: &str = "wikipathways";
const WIKIPATHWAYS_BASE_ENV: &str = "BIOMCP_WIKIPATHWAYS_BASE";
const WIKIPATHWAYS_MAX_BODY_BYTES: usize = 24 * 1024 * 1024;

pub struct WikiPathwaysClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl WikiPathwaysClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(WIKIPATHWAYS_BASE, WIKIPATHWAYS_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let req = crate::sources::apply_cache_mode(req);
        let resp = crate::sources::with_response_body_limit(
            req,
            WIKIPATHWAYS_MAX_BODY_BYTES,
            WIKIPATHWAYS_API,
        )
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body_with_limit(
            resp,
            WIKIPATHWAYS_API,
            WIKIPATHWAYS_MAX_BODY_BYTES,
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::summarize_http_error_body(content_type, bytes);
            return Err(BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(WIKIPATHWAYS_API, content_type, bytes)?;
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: WIKIPATHWAYS_API.to_string(),
            source,
        })
    }

    pub(crate) fn search_pathways_plan(query: &str) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "WikiPathways query is required".into(),
            ));
        }
        Ok(RequestPlan::get("findPathwaysByText.json"))
    }

    fn map_search_hits(
        resp: WikiPathwaysSearchResponse,
        query: &str,
        limit: usize,
    ) -> Vec<WikiPathwaysHit> {
        let mut ranked = Vec::new();
        let mut seen = HashSet::new();
        for row in resp.entries() {
            let is_human = row.species_is_human();
            let Some(score) = row.match_score(query) else {
                continue;
            };
            let Some(id) = row
                .id
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty() && is_wikipathways_id(value) && is_human)
            else {
                continue;
            };
            let Some(name) = row
                .name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            if !seen.insert(id.clone()) {
                continue;
            }
            ranked.push((score, id, name));
        }

        ranked.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| left.1.cmp(&right.1))
        });

        ranked
            .into_iter()
            .take(limit.clamp(1, 25))
            .map(|(_, id, name)| WikiPathwaysHit { id, name })
            .collect()
    }

    pub async fn search_pathways(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WikiPathwaysHit>, BioMcpError> {
        let plan = Self::search_pathways_plan(query)?;
        let resp: WikiPathwaysSearchResponse = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan)
                    .with_extension(CacheMode::NoStore),
            )
            .await?;
        Ok(Self::map_search_hits(resp, query, limit))
    }

    pub(crate) fn get_pathway_plan(pw_id: &str) -> Result<(RequestPlan, String), BioMcpError> {
        let pw_id = validate_wikipathways_id(pw_id)?;
        Ok((RequestPlan::get("getPathwayInfo.json"), pw_id))
    }

    fn map_pathway_record(
        resp: WikiPathwaysGetResponse,
        pw_id: &str,
    ) -> Result<WikiPathwaysRecord, BioMcpError> {
        let row = resp
            .entries()
            .into_iter()
            .find(|row| row.id.as_deref().map(str::trim) == Some(pw_id))
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "pathway".to_string(),
                id: pw_id.to_string(),
                suggestion:
                    "Try searching by pathway name, for example: biomcp search pathway -q apoptosis"
                        .to_string(),
            })?;
        let id = row
            .id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty() && is_wikipathways_id(value))
            .ok_or_else(|| BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: "WikiPathways detail response missing pathwayInfo.id".to_string(),
            })?;
        let name = row
            .name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: "WikiPathways detail response missing pathwayInfo.name".to_string(),
            })?;

        Ok(WikiPathwaysRecord {
            id,
            name,
            species: row
                .species
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    pub async fn get_pathway(&self, pw_id: &str) -> Result<WikiPathwaysRecord, BioMcpError> {
        let (plan, pw_id) = Self::get_pathway_plan(pw_id)?;
        let resp: WikiPathwaysGetResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Self::map_pathway_record(resp, &pw_id)
    }

    pub(crate) fn pathway_xrefs_plan(pw_id: &str) -> Result<(RequestPlan, String), BioMcpError> {
        let pw_id = validate_wikipathways_id(pw_id)?;
        Ok((RequestPlan::get("findPathwaysByXref.json"), pw_id))
    }

    fn map_pathway_entrez_gene_ids(resp: WikiPathwaysXrefResponse, pw_id: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for xref in resp.xrefs.unwrap_or_default() {
            let xref = xref.trim();
            if xref.is_empty() || !xref.chars().all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            if !seen.insert(xref.to_string()) {
                continue;
            }
            out.push(xref.to_string());
        }
        if let Some(row) = resp
            .pathway_info
            .into_iter()
            .find(|row| row.id.as_deref().map(str::trim) == Some(pw_id))
        {
            for xref in row.ncbigene.unwrap_or_default().split([',', ';']) {
                let xref = xref.trim();
                let xref = xref.strip_prefix("ncbigene:").unwrap_or(xref).trim();
                if xref.is_empty() || !xref.chars().all(|ch| ch.is_ascii_digit()) {
                    continue;
                }
                if !seen.insert(xref.to_string()) {
                    continue;
                }
                out.push(xref.to_string());
            }
        }
        out
    }

    pub async fn pathway_entrez_gene_ids(&self, pw_id: &str) -> Result<Vec<String>, BioMcpError> {
        let (plan, pw_id) = Self::pathway_xrefs_plan(pw_id)?;
        let resp: WikiPathwaysXrefResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        Ok(Self::map_pathway_entrez_gene_ids(resp, &pw_id))
    }
}

pub(crate) fn is_wikipathways_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes.starts_with(b"WP") && bytes[2..].iter().all(u8::is_ascii_digit)
}

fn validate_wikipathways_id(value: &str) -> Result<String, BioMcpError> {
    let value = value.trim();
    if !is_wikipathways_id(value) {
        return Err(BioMcpError::InvalidArgument(
            "WikiPathways ID must look like WP254. Example: biomcp get pathway WP254".into(),
        ));
    }
    Ok(value.to_string())
}

#[derive(Debug, Clone)]
pub struct WikiPathwaysHit {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct WikiPathwaysRecord {
    pub id: String,
    pub name: String,
    pub species: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysSearchResponse {
    #[serde(default)]
    result: Vec<WikiPathwaysSearchEntry>,
    #[serde(rename = "pathwayInfo", default)]
    pathway_info: Vec<WikiPathwaysSearchEntry>,
}

impl WikiPathwaysSearchResponse {
    fn entries(self) -> Vec<WikiPathwaysSearchEntry> {
        if self.pathway_info.is_empty() {
            self.result
        } else {
            self.pathway_info
        }
    }
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysSearchEntry {
    id: Option<String>,
    name: Option<String>,
    species: Option<String>,
    description: Option<String>,
    datanodes: Option<String>,
    annotations: Option<String>,
}

impl WikiPathwaysSearchEntry {
    fn species_is_human(&self) -> bool {
        self.species
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| value.eq_ignore_ascii_case("Homo sapiens"))
    }

    fn match_score(&self, query: &str) -> Option<u8> {
        let query = query.trim().to_ascii_lowercase();
        if query.is_empty() {
            return None;
        }

        let name = self.name.as_deref().map(str::trim).unwrap_or_default();
        let name_lower = name.to_ascii_lowercase();
        if name_lower == query {
            return Some(0);
        }
        if name_lower.starts_with(&query) {
            return Some(1);
        }
        if name_lower.contains(&query) {
            return Some(2);
        }
        if self
            .annotations
            .as_deref()
            .map(str::to_ascii_lowercase)
            .is_some_and(|value| value.contains(&query))
        {
            return Some(3);
        }
        if self
            .datanodes
            .as_deref()
            .map(str::to_ascii_lowercase)
            .is_some_and(|value| value.contains(&query))
        {
            return Some(4);
        }
        if self
            .description
            .as_deref()
            .map(str::to_ascii_lowercase)
            .is_some_and(|value| value.contains(&query))
        {
            return Some(5);
        }

        let searchable = format!(
            "{}\n{}\n{}\n{}",
            name_lower,
            self.description
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default(),
            self.datanodes
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default(),
            self.annotations
                .as_deref()
                .map(str::to_ascii_lowercase)
                .unwrap_or_default()
        );
        let tokens = query.split_whitespace().collect::<Vec<_>>();
        if tokens.iter().all(|token| searchable.contains(token)) {
            return Some(6);
        }

        None
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WikiPathwaysGetResponse {
    Legacy {
        #[serde(rename = "pathwayInfo")]
        pathway_info: WikiPathwaysGetEntry,
    },
    Bulk {
        #[serde(rename = "pathwayInfo")]
        pathway_info: Vec<WikiPathwaysGetEntry>,
    },
}

impl WikiPathwaysGetResponse {
    fn entries(self) -> Vec<WikiPathwaysGetEntry> {
        match self {
            Self::Legacy { pathway_info } => vec![pathway_info],
            Self::Bulk { pathway_info } => pathway_info,
        }
    }
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysGetEntry {
    id: Option<String>,
    name: Option<String>,
    species: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysXrefResponse {
    xrefs: Option<Vec<String>>,
    #[serde(rename = "pathwayInfo", default)]
    pathway_info: Vec<WikiPathwaysXrefEntry>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysXrefEntry {
    id: Option<String>,
    ncbigene: Option<String>,
}

#[cfg(test)]
mod tests;
