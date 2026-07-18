use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const QUICKGO_BASE: &str = "https://www.ebi.ac.uk/QuickGO/services";
const QUICKGO_API: &str = "quickgo";
const QUICKGO_BASE_ENV: &str = "BIOMCP_QUICKGO_BASE";

pub struct QuickGoClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl QuickGoClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(QUICKGO_BASE, QUICKGO_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::QUICKGO,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::QUICKGO),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::QUICKGO,
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
                api: QUICKGO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: QUICKGO_API.to_string(),
            source,
        })
    }

    pub(crate) fn annotations_plan(
        gene_product_id: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene_product_id = gene_product_id.trim();
        if gene_product_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "QuickGO geneProductId is required".into(),
            ));
        }

        let page_size = limit.clamp(1, 25).to_string();
        Ok(RequestPlan::get("annotation/search")
            .query("geneProductId", gene_product_id)
            .query("limit", page_size))
    }

    pub async fn annotations(
        &self,
        gene_product_id: &str,
        limit: usize,
    ) -> Result<Vec<QuickGoAnnotation>, BioMcpError> {
        let plan = Self::annotations_plan(gene_product_id, limit)?;
        let resp: QuickGoAnnotationResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;

        Ok(resp.results)
    }

    pub(crate) fn terms_plan(go_ids: &[String]) -> Option<RequestPlan> {
        let mut ids = go_ids
            .iter()
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        if ids.is_empty() {
            return None;
        }

        ids.sort();
        ids.dedup();
        let ids = ids.join(",");
        Some(RequestPlan::get(format!("ontology/go/terms/{ids}")))
    }

    pub async fn terms(&self, go_ids: &[String]) -> Result<Vec<QuickGoTerm>, BioMcpError> {
        let Some(plan) = Self::terms_plan(go_ids) else {
            return Ok(Vec::new());
        };
        let resp: QuickGoTermsResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(resp.results)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoAnnotationResponse {
    #[serde(default)]
    pub results: Vec<QuickGoAnnotation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickGoAnnotation {
    pub go_id: Option<String>,
    pub go_name: Option<String>,
    pub go_aspect: Option<String>,
    pub evidence_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoTermsResponse {
    #[serde(default)]
    pub results: Vec<QuickGoTerm>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuickGoTerm {
    pub id: Option<String>,
    pub name: Option<String>,
    pub aspect: Option<String>,
}

#[cfg(test)]
mod tests;
