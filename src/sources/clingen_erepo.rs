use std::borrow::Cow;

use reqwest::StatusCode;
use serde_json::Value;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const EREPO_BASE: &str = "https://erepo.clinicalgenome.org";
const EREPO_BASE_ENV: &str = "BIOMCP_CLINGEN_EREPO_BASE";
const SUMMARY_BODY_LIMIT: usize = 1024 * 1024;
const GENE_SEARCH_BODY_LIMIT: usize = 2 * 1024 * 1024;
const DETAIL_BODY_LIMIT: usize = 4 * 1024 * 1024;
const GUIDELINE_BODY_LIMIT: usize = 1024 * 1024;

pub(crate) struct ERepoClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ERepoClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(EREPO_BASE, EREPO_BASE_ENV),
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_client(
        client: reqwest_middleware::ClientWithMiddleware,
        base: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            client,
            base: base.into(),
        }
    }

    pub(crate) fn summary_plan(caid: &str) -> RequestPlan {
        RequestPlan::get("evrepo/api/summary/classifications")
            .query("columns", "caId")
            .query("values", caid)
            .query("matchTypes", "exact")
            .query("pgSize", "25")
            .query("pg", "1")
    }

    pub(crate) fn gene_plan(gene: &str, limit: usize, offset: usize) -> RequestPlan {
        RequestPlan::get("evrepo/api/classifications")
            .query("gene", gene)
            .query("matchLimit", limit.saturating_add(1).to_string())
            .query("matchSkip", offset.to_string())
    }

    pub(crate) fn detail_plan(uuid: &str, version: &str) -> RequestPlan {
        RequestPlan::get(detail_path(uuid, version))
    }

    pub(crate) fn guideline_plan(uuid: &str, version: &str) -> RequestPlan {
        RequestPlan::get(guideline_path(uuid)).query("version", version)
    }

    pub(crate) fn detail_url(&self, uuid: &str, version: &str) -> String {
        format!(
            "{}/{}",
            self.base.trim_end_matches('/'),
            detail_path(uuid, version)
        )
    }

    async fn get(
        &self,
        plan: RequestPlan,
        limit: usize,
    ) -> Result<(StatusCode, Vec<u8>), BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &plan,
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_EREPO))
        .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_EREPO),
            limit,
        )
        .await?;
        crate::sources::ensure_json_content_type(
            SourceContext::retry(SourceProvider::CLINGEN_EREPO),
            content_type.as_ref(),
            &bytes,
        )?;
        Ok((status, bytes))
    }

    pub(crate) async fn summary(&self, caid: &str) -> Result<Value, BioMcpError> {
        let (status, bytes) = self
            .get(Self::summary_plan(caid), SUMMARY_BODY_LIMIT)
            .await?;
        if status == StatusCode::NOT_FOUND && is_no_records_404(&bytes) {
            return Ok(serde_json::json!({"status":{"code":200}, "metadata":{}, "data":[]}));
        }
        decode_envelope(status, &bytes)
    }

    pub(crate) async fn gene(
        &self,
        gene: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Value, BioMcpError> {
        let (status, bytes) = self
            .get(Self::gene_plan(gene, limit, offset), GENE_SEARCH_BODY_LIMIT)
            .await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: "ClinGen ERepo".into(),
                message: format!("HTTP {status}"),
            });
        }
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                api: "ClinGen ERepo".into(),
                source,
            })?;
        if !value
            .get("variantInterpretations")
            .is_some_and(Value::is_array)
        {
            return Err(BioMcpError::Api {
                api: "ClinGen ERepo".into(),
                message: "gene response has no variantInterpretations array".into(),
            });
        }
        Ok(value)
    }

    pub(crate) async fn detail(
        &self,
        uuid: &str,
        version: &str,
    ) -> Result<(Value, Vec<u8>), BioMcpError> {
        let (status, bytes) = self
            .get(Self::detail_plan(uuid, version), DETAIL_BODY_LIMIT)
            .await?;
        let value = decode_envelope(status, &bytes)?;
        Ok((value, bytes))
    }

    pub(crate) async fn guideline_page(
        &self,
        uuid: &str,
        version: &str,
    ) -> Result<Vec<u8>, BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &Self::guideline_plan(uuid, version),
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_EREPO))
        .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_EREPO),
            GUIDELINE_BODY_LIMIT,
        )
        .await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: "ClinGen ERepo".into(),
                message: format!("HTTP {status}"),
            });
        }
        if !content_type
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().starts_with("text/html"))
        {
            return Err(BioMcpError::Api {
                api: "ClinGen ERepo".into(),
                message: "guideline page did not return HTML".into(),
            });
        }
        Ok(bytes)
    }
}

fn detail_path(uuid: &str, version: &str) -> String {
    let mut url = reqwest::Url::parse("https://erepo.clinicalgenome.org/")
        .expect("static ERepo origin is valid");
    url.path_segments_mut()
        .expect("static ERepo origin accepts path segments")
        .extend([
            "evrepo",
            "api",
            "summary",
            "classification",
            uuid,
            "doc",
            "sepio",
            "version",
            version,
        ]);
    url.path().trim_start_matches('/').to_owned()
}

fn guideline_path(uuid: &str) -> String {
    let mut url = reqwest::Url::parse("https://erepo.clinicalgenome.org/")
        .expect("static ERepo origin is valid");
    url.path_segments_mut()
        .expect("static ERepo origin accepts path segments")
        .extend(["evrepo", "ui", "classification", uuid]);
    url.path().trim_start_matches('/').to_owned()
}

fn is_no_records_404(bytes: &[u8]) -> bool {
    let Some(status) = serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|value| value.get("status").cloned())
    else {
        return false;
    };
    status.get("code").and_then(Value::as_i64) == Some(404)
        && ["message", "msg"]
            .into_iter()
            .filter_map(|key| status.get(key).and_then(Value::as_str))
            .any(|message| {
                message.eq_ignore_ascii_case("No records found")
                    || message.eq_ignore_ascii_case("No records were found for given query")
            })
}

fn decode_envelope(status: StatusCode, bytes: &[u8]) -> Result<Value, BioMcpError> {
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: "ClinGen ERepo".into(),
            message: format!("HTTP {status}: {}", crate::sources::body_excerpt(bytes)),
        });
    }
    let value: Value = serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: "ClinGen ERepo".into(),
        source,
    })?;
    let code = value.pointer("/status/code").and_then(Value::as_i64);
    if value.get("metadata").is_none() || value.get("data").is_none() || code != Some(200) {
        return Err(BioMcpError::Api {
            api: "ClinGen ERepo".into(),
            message: "invalid ERepo response envelope".into(),
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
