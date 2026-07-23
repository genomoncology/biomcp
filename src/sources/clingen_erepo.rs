use std::borrow::Cow;

use reqwest::StatusCode;
use serde_json::Value;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const EREPO_BASE: &str = "https://erepo.clinicalgenome.org";
const EREPO_BASE_ENV: &str = "BIOMCP_CLINGEN_EREPO_BASE";
const SUMMARY_BODY_LIMIT: usize = 1024 * 1024;
const DETAIL_BODY_LIMIT: usize = 4 * 1024 * 1024;

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

    pub(crate) fn summary_plan(caid: &str) -> RequestPlan {
        RequestPlan::get("evrepo/api/summary/classifications")
            .query("columns", "caId")
            .query("values", caid)
            .query("matchTypes", "exact")
            .query("pgSize", "25")
            .query("pg", "1")
    }

    pub(crate) fn detail_plan(uuid: &str, version: &str) -> RequestPlan {
        RequestPlan::get(format!(
            "evrepo/api/summary/classification/{uuid}/doc/sepio/version/{version}"
        ))
    }

    pub(crate) fn detail_url(&self, uuid: &str, version: &str) -> String {
        format!(
            "{}/evrepo/api/summary/classification/{uuid}/doc/sepio/version/{version}",
            self.base.trim_end_matches('/')
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
}

fn is_no_records_404(bytes: &[u8]) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|value| {
            value
                .pointer("/status/message")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|message| message.eq_ignore_ascii_case("No records found"))
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
