use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const HPO_BASE: &str = "https://ontology.jax.org/api/hp";
const HPO_API: &str = "hpo";
const HPO_BASE_ENV: &str = "BIOMCP_HPO_BASE";

pub struct HpoClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl HpoClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(HPO_BASE, HPO_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::HPO,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::HPO),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::HPO,
            ))
        })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if status == StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "hpo".into(),
                id: "term".into(),
                suggestion: "Use an HPO ID like HP:0001653".into(),
            });
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: HPO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        require_json_content_type(
            crate::error::SourceContext::retry(crate::error::SourceProvider::HPO),
            content_type,
            bytes,
        )?;
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: HPO_API.to_string(),
            source,
        })
    }

    pub(crate) fn term_plan(hpo_id: &str) -> Result<RequestPlan, BioMcpError> {
        let hpo_id = normalize_hpo_id(hpo_id).ok_or_else(|| {
            BioMcpError::InvalidArgument("HPO term ID is required (e.g., HP:0001653)".into())
        })?;
        Ok(RequestPlan::get(format!("terms/{hpo_id}")))
    }

    pub async fn term(&self, hpo_id: &str) -> Result<HpoTerm, BioMcpError> {
        let plan = Self::term_plan(hpo_id)?;
        self.get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    fn normalize_term_ids(ids: &[String], max_terms: usize) -> Vec<String> {
        let mut normalized = ids
            .iter()
            .filter_map(|id| normalize_hpo_id(id))
            .collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        normalized.truncate(max_terms.clamp(1, 20));
        normalized
    }

    pub async fn resolve_terms(
        &self,
        ids: &[String],
        max_terms: usize,
    ) -> Result<HashMap<String, String>, BioMcpError> {
        let normalized = Self::normalize_term_ids(ids, max_terms);

        let lookups = normalized
            .iter()
            .map(|id| async move { (id.clone(), self.term(id).await) })
            .collect::<Vec<_>>();

        let mut out: HashMap<String, String> = HashMap::new();
        for (id, result) in join_all(lookups).await {
            match result {
                Ok(term) => {
                    let name = term.name.trim();
                    if !name.is_empty() {
                        out.insert(id, name.to_string());
                    }
                }
                Err(err) if err.is_not_found() => {}
                Err(err) => return Err(err),
            }
        }
        Ok(out)
    }

    pub(crate) fn search_term_ids_plan(query: &str) -> Option<RequestPlan> {
        let query = query.trim();
        if query.is_empty() {
            return None;
        }
        Some(RequestPlan::get("search").query("q", query))
    }

    fn decode_search_term_ids(response: HpoSearchResponse, max_terms: usize) -> Vec<String> {
        Self::decode_search_terms(response)
            .into_iter()
            .take(max_terms.clamp(1, 20))
            .map(|term| term.id)
            .collect()
    }

    fn decode_search_terms(response: HpoSearchResponse) -> Vec<HpoResolvedTerm> {
        let mut out = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for row in response.terms {
            if let Some(id) = normalize_hpo_id(&row.id)
                && seen.insert(id.clone())
            {
                out.push(HpoResolvedTerm {
                    id,
                    label: row.name,
                });
            }
        }
        out
    }

    pub async fn search_term_ids(
        &self,
        query: &str,
        max_terms: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let Some(plan) = Self::search_term_ids_plan(query) else {
            return Ok(Vec::new());
        };
        let response: HpoSearchResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::decode_search_term_ids(response, max_terms))
    }

    pub(crate) async fn search_terms(
        &self,
        query: &str,
    ) -> Result<Vec<HpoResolvedTerm>, BioMcpError> {
        let Some(plan) = Self::search_term_ids_plan(query) else {
            return Ok(Vec::new());
        };
        let response: HpoSearchResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        let terms = Self::decode_search_terms(response);
        if let Some(term) = terms.iter().find(|term| term.label.trim().is_empty()) {
            return Err(BioMcpError::Api {
                api: HPO_API.into(),
                message: format!("HPO returned a blank label for {}", term.id),
            });
        }
        Ok(terms)
    }
}

fn require_json_content_type(
    context: crate::error::SourceContext,
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    crate::sources::ensure_json_content_type(context, content_type, body)?;
    let media_type = content_type
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if media_type.is_some_and(|value| {
        value.eq_ignore_ascii_case("application/json")
            || value.eq_ignore_ascii_case("text/json")
            || value.to_ascii_lowercase().ends_with("+json")
    }) {
        return Ok(());
    }
    Err(BioMcpError::Api {
        api: context.provider().label().into(),
        message: "Provider response did not declare a JSON content type".into(),
    }
    .with_source_context(context))
}

fn normalize_hpo_id(value: &str) -> Option<String> {
    let mut id = value.trim().to_ascii_uppercase();
    if id.is_empty() {
        return None;
    }
    id = id.replace('_', ":");
    if !id.starts_with("HP:") {
        return None;
    }
    let suffix = id.trim_start_matches("HP:");
    if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("HP:{suffix}"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct HpoTerm {
    // dead-code reason: hpo::id preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HpoSearchResponse {
    terms: Vec<HpoSearchTerm>,
}

#[derive(Debug, Clone, Deserialize)]
struct HpoSearchTerm {
    id: String,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HpoResolvedTerm {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[cfg(test)]
mod tests;
