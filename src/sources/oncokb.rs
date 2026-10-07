use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use biodata::OncoKBAnnotationProjection;
use reqwest::StatusCode;
use tracing::debug;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const ONCOKB_PROD_BASE: &str = "https://www.oncokb.org/api/v1";
const ONCOKB_API: &str = "oncokb";
const ONCOKB_TOKEN_ENV: &str = "ONCOKB_TOKEN";
const ONCOKB_BASE_ENV: &str = "BIOMCP_ONCOKB_BASE";

pub struct OncoKBClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    token: Option<String>,
}

impl OncoKBClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let token = std::env::var(ONCOKB_TOKEN_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let base = std::env::var(ONCOKB_BASE_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(Cow::Owned)
            .unwrap_or_else(|| Cow::Borrowed(ONCOKB_PROD_BASE));

        Ok(Self {
            client: crate::sources::shared_client()?,
            base,
            token,
        })
    }

    fn require_token(&self) -> Result<&str, BioMcpError> {
        self.token
            .as_deref()
            .filter(|t| !t.trim().is_empty())
            .ok_or_else(|| BioMcpError::ApiKeyRequired {
                api: ONCOKB_API.to_string(),
                env_var: ONCOKB_TOKEN_ENV.to_string(),
                docs_url: "https://www.oncokb.org/".to_string(),
            })
    }

    pub(crate) fn annotate_by_protein_change_plan(
        gene: &str,
        alteration: &str,
        token: &str,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene = gene.trim();
        let alteration = alteration.trim();
        if gene.is_empty() || alteration.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OncoKB annotation requires gene and alteration".into(),
            ));
        }
        if token.trim().is_empty() {
            return Err(BioMcpError::ApiKeyRequired {
                api: ONCOKB_API.to_string(),
                env_var: ONCOKB_TOKEN_ENV.to_string(),
                docs_url: "https://www.oncokb.org/".to_string(),
            });
        }

        Ok(RequestPlan::get("annotate/mutations/byProteinChange")
            .query("hugoSymbol", gene)
            .query("alteration", alteration)
            .header("Authorization", format!("Bearer {}", token.trim())))
    }

    pub(crate) fn decode_json_response(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<OncoKBAnnotationProjection, BioMcpError> {
        let context = crate::error::SourceContext::retry(crate::error::SourceProvider::ONCOKB);
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: context.provider().label().to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            }
            .with_source_context(context));
        }
        OncoKBAnnotationProjection::decode_json(bytes).map_err(|_| {
            BioMcpError::ApiJson {
                api: context.provider().label().to_string(),
                source: <serde_json::Error as serde::de::Error>::custom("Invalid OncoKB response."),
            }
            .with_source_context(context)
        })
    }

    async fn get_json(
        &self,
        req: reqwest_middleware::RequestBuilder,
        authenticated: bool,
    ) -> Result<OncoKBAnnotationProjection, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, authenticated)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::ONCOKB,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::ONCOKB),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::ONCOKB,
            ))
        })
    }

    pub async fn annotate_by_protein_change(
        &self,
        gene: &str,
        alteration: &str,
    ) -> Result<OncoKBAnnotationProjection, BioMcpError> {
        let token = self.require_token()?;
        let plan = Self::annotate_by_protein_change_plan(gene, alteration, token)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);

        self.get_json(req, true).await
    }

    pub async fn annotate_best_effort(
        &self,
        gene: &str,
        alteration: &str,
    ) -> Result<OncoKBAnnotationProjection, BioMcpError> {
        let gene = gene.trim();
        let alteration = alteration.trim();
        if gene.is_empty() || alteration.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "OncoKB annotation requires gene and alteration".into(),
            ));
        }

        let mut last_err: Option<BioMcpError> = None;
        for alt in protein_change_attempts(alteration) {
            debug!(gene = %gene, alteration = %alt, "OncoKB annotate attempt");
            match self.annotate_by_protein_change(gene, &alt).await {
                Ok(ann) => return Ok(ann),
                Err(err) => last_err = Some(err),
            }
        }

        Err(last_err.unwrap_or_else(|| BioMcpError::Api {
            api: ONCOKB_API.to_string(),
            message: "No OncoKB annotation available".into(),
        }))
    }
}

pub(crate) fn protein_change_attempts(alteration: &str) -> Vec<String> {
    let mut attempts: Vec<String> = Vec::new();
    let mut push_attempt = |value: String| {
        let v = value.trim().to_string();
        if v.is_empty() {
            return;
        }
        if attempts.iter().any(|a| a.eq_ignore_ascii_case(&v)) {
            return;
        }
        attempts.push(v);
    };

    let alteration = alteration.trim();
    push_attempt(alteration.to_string());
    if alteration.starts_with("p.") || alteration.starts_with("P.") {
        push_attempt(alteration[2..].trim().to_string());
    } else {
        push_attempt(format!("p.{alteration}"));
    }
    attempts
}

#[cfg(test)]
mod tests;
