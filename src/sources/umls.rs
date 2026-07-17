use std::borrow::Cow;

use futures::future::join_all;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const UMLS_BASE: &str = "https://uts-ws.nlm.nih.gov";
const UMLS_API: &str = "umls";
const UMLS_BASE_ENV: &str = "BIOMCP_UMLS_BASE";
const UMLS_API_KEY_ENV: &str = "UMLS_API_KEY";
const UMLS_ATOM_PAGE_SIZE: &str = "200";

pub struct UmlsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: String,
}

impl UmlsClient {
    pub fn new() -> Result<Option<Self>, BioMcpError> {
        let Some(api_key) = std::env::var(UMLS_API_KEY_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        Ok(Some(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(UMLS_BASE, UMLS_BASE_ENV),
            api_key,
        }))
    }

    pub(crate) fn search_plan(query: &str, api_key: &str) -> Option<RequestPlan> {
        let query = query.trim();
        if query.is_empty() {
            return None;
        }

        Some(
            RequestPlan::get("rest/search/current")
                .query("string", query)
                .query("pageSize", "5")
                .query("apiKey", api_key),
        )
    }

    pub(crate) fn atoms_plan(cui: &str, api_key: &str) -> RequestPlan {
        RequestPlan::get(format!("rest/content/current/CUI/{cui}/atoms"))
            .query("apiKey", api_key)
            .query("pageSize", UMLS_ATOM_PAGE_SIZE)
            .query("language", "ENG")
    }

    pub(crate) fn decode_json_response<T: serde::de::DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: UMLS_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(bytes)),
            });
        }
        crate::sources::ensure_json_content_type(UMLS_API, content_type, bytes)?;
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: UMLS_API.to_string(),
            source,
        })
    }

    fn search_hits(search: UmlsSearchEnvelope) -> Vec<UmlsHit> {
        search
            .result
            .results
            .into_iter()
            .filter(|hit| hit.ui != "NONE")
            .take(5)
            .collect()
    }

    fn concept_from_hit(hit: UmlsHit, xrefs: Vec<UmlsXref>) -> UmlsConcept {
        UmlsConcept {
            cui: hit.ui,
            name: hit.name,
            semantic_types: hit.semantic_types,
            xrefs,
            uri: hit.uri,
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<UmlsConcept>, BioMcpError> {
        let Some(plan) = Self::search_plan(query, &self.api_key) else {
            return Ok(Vec::new());
        };

        let resp = crate::sources::apply_cache_mode_with_auth(
            request_from_plan(&self.client, self.base.as_ref(), &plan),
            true,
        )
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, UMLS_API).await?;
        let search: UmlsSearchEnvelope =
            Self::decode_json_response(status, content_type.as_ref(), &bytes)?;

        let tasks = Self::search_hits(search)
            .into_iter()
            .map(|hit| async move {
                let xrefs = self.fetch_atoms(&hit.ui).await?;
                Ok::<_, BioMcpError>(Self::concept_from_hit(hit, xrefs))
            })
            .collect::<Vec<_>>();

        let mut out = Vec::new();
        for result in join_all(tasks).await {
            out.push(result?);
        }
        Ok(out)
    }

    async fn fetch_atoms(&self, cui: &str) -> Result<Vec<UmlsXref>, BioMcpError> {
        let plan = Self::atoms_plan(cui, &self.api_key);
        let resp = crate::sources::apply_cache_mode_with_auth(
            request_from_plan(&self.client, self.base.as_ref(), &plan),
            true,
        )
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, UMLS_API).await?;
        let atoms: UmlsAtomsEnvelope =
            Self::decode_json_response(status, content_type.as_ref(), &bytes)?;

        Ok(Self::map_atoms(atoms))
    }

    fn map_atoms(atoms: UmlsAtomsEnvelope) -> Vec<UmlsXref> {
        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for atom in atoms.result {
            if !atom.language.eq_ignore_ascii_case("ENG") {
                continue;
            }
            let id = atom
                .code
                .rsplit('/')
                .next()
                .map(str::trim)
                .unwrap_or_default();
            if id.is_empty() {
                continue;
            }
            let key = format!("{}:{id}", atom.root_source.to_ascii_uppercase());
            if seen.insert(key) {
                out.push(UmlsXref {
                    vocab: atom.root_source,
                    id: id.to_string(),
                    label: atom.name,
                });
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct UmlsConcept {
    pub cui: String,
    pub name: String,
    pub semantic_types: Vec<String>,
    pub xrefs: Vec<UmlsXref>,
    // dead-code reason: umls::uri preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UmlsXref {
    pub vocab: String,
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsSearchEnvelope {
    result: UmlsSearchResult,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsSearchResult {
    #[serde(default)]
    results: Vec<UmlsHit>,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsHit {
    ui: String,
    name: String,
    #[serde(default, rename = "semanticTypes")]
    semantic_types: Vec<String>,
    uri: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsAtomsEnvelope {
    #[serde(default)]
    result: Vec<UmlsAtom>,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsAtom {
    #[serde(default, rename = "rootSource")]
    root_source: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    language: String,
    #[serde(default)]
    name: String,
}

#[cfg(test)]
mod tests;
