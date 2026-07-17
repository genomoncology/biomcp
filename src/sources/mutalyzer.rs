use std::borrow::Cow;

use crate::entities::variant::{
    VariantNormalizationService, VariantNormalizationServiceResult, VariantNormalizationStatus,
};
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

const MUTALYZER_BASE: &str = "https://mutalyzer.nl/api";
const MUTALYZER_API: &str = "mutalyzer";
const MUTALYZER_BASE_ENV: &str = "BIOMCP_MUTALYZER_BASE_URL";

// dead-code reason: mutalyzer::MutalyzerNormalizeRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct MutalyzerNormalizeRequestPlan {
    pub method: &'static str,
    pub path: String,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
}

pub struct MutalyzerClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MutalyzerClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MUTALYZER_BASE, MUTALYZER_BASE_ENV),
        })
    }

    fn normalize_path(description: &str) -> Result<String, BioMcpError> {
        let mut url =
            reqwest::Url::parse("https://biomcp.local").map_err(|err| BioMcpError::Api {
                api: MUTALYZER_API.to_string(),
                message: err.to_string(),
            })?;
        url.path_segments_mut()
            .map_err(|_| BioMcpError::Api {
                api: MUTALYZER_API.to_string(),
                message: "invalid Mutalyzer request path".to_string(),
            })?
            .push("normalize")
            .push(description);
        Ok(url.path().to_string())
    }

    fn endpoint_url(&self, path: &str) -> Result<reqwest::Url, BioMcpError> {
        reqwest::Url::parse(&format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        ))
        .map_err(|err| BioMcpError::Api {
            api: MUTALYZER_API.to_string(),
            message: err.to_string(),
        })
    }

    pub fn normalize_request_plan(
        &self,
        description: &str,
    ) -> Result<MutalyzerNormalizeRequestPlan, BioMcpError> {
        let path = Self::normalize_path(description)?;
        debug_assert!(path.starts_with("/normalize/"));
        Ok(MutalyzerNormalizeRequestPlan {
            method: "GET",
            path,
            query_params: Vec::new(),
            cache_mode: "default",
            status_expectation: "invalid_input/not_found/service_error per HTTP and payload status",
            content_type_expectation: "json",
        })
    }

    pub async fn normalize(
        &self,
        description: &str,
    ) -> Result<VariantNormalizationServiceResult, BioMcpError> {
        let plan = self.normalize_request_plan(description)?;
        let url = self.endpoint_url(&plan.path)?;
        let resp = crate::sources::apply_cache_mode(self.client.get(url))
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MUTALYZER_API).await?;

        Ok(decode_normalize_response(
            status,
            content_type.as_ref(),
            &bytes,
        ))
    }
}

fn decode_normalize_response(
    status: StatusCode,
    content_type: Option<&HeaderValue>,
    bytes: &[u8],
) -> VariantNormalizationServiceResult {
    if !status.is_success() {
        return provider_error(status, bytes);
    }

    if let Err(err) = crate::sources::ensure_json_content_type(MUTALYZER_API, content_type, bytes) {
        return message_result(VariantNormalizationStatus::ServiceError, err.to_string());
    }
    let value: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(source) => {
            return message_result(
                VariantNormalizationStatus::ServiceError,
                BioMcpError::ApiJson {
                    api: MUTALYZER_API.to_string(),
                    source,
                }
                .to_string(),
            );
        }
    };
    result_from_value(&value)
}

fn result_from_value(value: &serde_json::Value) -> VariantNormalizationServiceResult {
    let warnings = value
        .get("infos")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.get("details")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("code").and_then(|v| v.as_str()))
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    let normalized_description = string_field(value, "normalized_description");
    let message = string_field(value, "message");
    let input_description = string_field(value, "input_description").or_else(|| {
        value
            .pointer("/custom/input_description")
            .and_then(|v| v.as_str())
            .filter(|v| !v.trim().is_empty())
            .map(str::to_string)
    });

    if normalized_description.is_none() {
        return VariantNormalizationServiceResult {
            service: VariantNormalizationService::Mutalyzer.as_str().to_string(),
            status: if message.is_some() {
                VariantNormalizationStatus::InvalidInput
            } else {
                VariantNormalizationStatus::ServiceError
            },
            input_description,
            normalized_description: None,
            corrected_description: None,
            transcript_description: None,
            protein: None,
            genomic_descriptions: Vec::new(),
            warnings,
            message: Some(
                message
                    .unwrap_or_else(|| "Mutalyzer returned no normalized description".to_string()),
            ),
        };
    }

    VariantNormalizationServiceResult {
        service: VariantNormalizationService::Mutalyzer.as_str().to_string(),
        status: VariantNormalizationStatus::Success,
        input_description,
        normalized_description,
        corrected_description: string_field(value, "corrected_description"),
        transcript_description: None,
        protein: value.get("protein").cloned(),
        genomic_descriptions: Vec::new(),
        warnings,
        message: None,
    }
}

fn provider_error(status: reqwest::StatusCode, bytes: &[u8]) -> VariantNormalizationServiceResult {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).ok();
    let message = value
        .as_ref()
        .and_then(|v| v.get("message"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| format!("HTTP {status}: {}", crate::sources::body_excerpt(bytes)));
    let input_description = value
        .as_ref()
        .and_then(|v| v.pointer("/custom/input_description"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    VariantNormalizationServiceResult {
        service: VariantNormalizationService::Mutalyzer.as_str().to_string(),
        status: match status.as_u16() {
            400 | 422 => VariantNormalizationStatus::InvalidInput,
            404 => VariantNormalizationStatus::NotFound,
            _ => VariantNormalizationStatus::ServiceError,
        },
        input_description,
        normalized_description: None,
        corrected_description: None,
        transcript_description: None,
        protein: None,
        genomic_descriptions: Vec::new(),
        warnings: Vec::new(),
        message: Some(message),
    }
}

fn message_result(
    status: VariantNormalizationStatus,
    message: String,
) -> VariantNormalizationServiceResult {
    VariantNormalizationServiceResult {
        service: VariantNormalizationService::Mutalyzer.as_str().to_string(),
        status,
        input_description: None,
        normalized_description: None,
        corrected_description: None,
        transcript_description: None,
        protein: None,
        genomic_descriptions: Vec::new(),
        warnings: Vec::new(),
        message: Some(message),
    }
}

fn string_field(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests;
