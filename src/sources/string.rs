use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const STRING_BASE: &str = "https://string-db.org/api";
const STRING_API: &str = "string";
const STRING_BASE_ENV: &str = "BIOMCP_STRING_BASE";

pub struct StringClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl StringClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(STRING_BASE, STRING_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::STRING,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::STRING),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::STRING,
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
                api: STRING_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: STRING_API.to_string(),
            source,
        })
    }

    pub(crate) fn interactions_plan(
        identifiers: &str,
        species: u32,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let identifiers = identifiers.trim();
        if identifiers.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "STRING identifiers are required".into(),
            ));
        }

        let species = species.to_string();
        let limit = limit.clamp(1, 25).to_string();
        Ok(RequestPlan::get("json/network")
            .query("identifiers", identifiers)
            .query("species", species)
            .query("limit", limit))
    }

    pub async fn interactions(
        &self,
        identifiers: &str,
        species: u32,
        limit: usize,
    ) -> Result<Vec<StringInteraction>, BioMcpError> {
        let plan = Self::interactions_plan(identifiers, species, limit)?;
        self.get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StringInteraction {
    #[serde(rename = "preferredName_A", alias = "preferredNameA")]
    pub preferred_name_a: Option<String>,
    #[serde(rename = "preferredName_B", alias = "preferredNameB")]
    pub preferred_name_b: Option<String>,
    pub score: Option<f64>,
}

#[cfg(test)]
mod tests;
