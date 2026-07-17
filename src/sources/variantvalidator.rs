use std::borrow::Cow;

use crate::entities::variant::{
    VariantNormalizationService, VariantNormalizationServiceResult, VariantNormalizationStatus,
};
use crate::error::BioMcpError;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

const VARIANTVALIDATOR_BASE: &str = "https://rest.variantvalidator.org";
const VARIANTVALIDATOR_API: &str = "variantvalidator";
const VARIANTVALIDATOR_BASE_ENV: &str = "BIOMCP_VARIANTVALIDATOR_BASE_URL";

// dead-code reason: variantvalidator::VariantValidatorNormalizeRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct VariantValidatorNormalizeRequestPlan {
    pub method: &'static str,
    pub path: String,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
}

pub struct VariantValidatorClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl VariantValidatorClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(VARIANTVALIDATOR_BASE, VARIANTVALIDATOR_BASE_ENV),
        })
    }

    fn normalize_path(description: &str) -> Result<String, BioMcpError> {
        let mut url =
            reqwest::Url::parse("https://biomcp.local").map_err(|err| BioMcpError::Api {
                api: VARIANTVALIDATOR_API.to_string(),
                message: err.to_string(),
            })?;
        url.path_segments_mut()
            .map_err(|_| BioMcpError::Api {
                api: VARIANTVALIDATOR_API.to_string(),
                message: "invalid VariantValidator request path".to_string(),
            })?
            .push("VariantValidator")
            .push("variantvalidator")
            .push("GRCh38")
            .push(description)
            .push("all");
        Ok(url.path().to_string())
    }

    fn endpoint_url(&self, path: &str) -> Result<reqwest::Url, BioMcpError> {
        reqwest::Url::parse(&format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        ))
        .map_err(|err| BioMcpError::Api {
            api: VARIANTVALIDATOR_API.to_string(),
            message: err.to_string(),
        })
    }

    pub fn normalize_request_plan(
        &self,
        description: &str,
    ) -> Result<VariantValidatorNormalizeRequestPlan, BioMcpError> {
        let path = Self::normalize_path(description)?;
        debug_assert!(path.starts_with("/VariantValidator/variantvalidator/GRCh38/"));
        Ok(VariantValidatorNormalizeRequestPlan {
            method: "GET",
            path,
            query_params: vec![("content-type", "application/json".to_string())],
            cache_mode: "default",
            status_expectation: "400/422 invalid_input; 404 not_found; other non-2xx service_error",
            content_type_expectation: "application/json",
        })
    }

    pub async fn normalize(
        &self,
        description: &str,
    ) -> Result<VariantNormalizationServiceResult, BioMcpError> {
        let plan = self.normalize_request_plan(description)?;
        let mut url = self.endpoint_url(&plan.path)?;
        url.query_pairs_mut().extend_pairs(
            plan.query_params
                .iter()
                .map(|(name, value)| (*name, value.as_str())),
        );
        let resp = crate::sources::apply_cache_mode(self.client.get(url))
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, VARIANTVALIDATOR_API).await?;

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
        return http_error(status, bytes);
    }

    if let Err(err) =
        crate::sources::ensure_json_content_type(VARIANTVALIDATOR_API, content_type, bytes)
    {
        return message_result(VariantNormalizationStatus::ServiceError, err.to_string());
    }
    let value: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(source) => {
            return message_result(
                VariantNormalizationStatus::ServiceError,
                BioMcpError::ApiJson {
                    api: VARIANTVALIDATOR_API.to_string(),
                    source,
                }
                .to_string(),
            );
        }
    };
    result_from_value(&value)
}

fn result_from_value(value: &serde_json::Value) -> VariantNormalizationServiceResult {
    let Some((_, result)) = value.as_object().and_then(|object| {
        object.iter().find(|(key, value)| {
            key.as_str() != "flag" && key.as_str() != "metadata" && value.is_object()
        })
    }) else {
        return message_result(
            VariantNormalizationStatus::ServiceError,
            "VariantValidator returned no result object".to_string(),
        );
    };

    let warnings = result
        .get("validation_warnings")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let transcript = string_field(result, "hgvs_transcript_variant");
    let genomic_descriptions = genomic_descriptions(result);
    let protein = result.get("hgvs_predicted_protein_consequence").cloned();

    if transcript.is_none() {
        return VariantNormalizationServiceResult {
            service: VariantNormalizationService::VariantValidator
                .as_str()
                .to_string(),
            status: if warnings.is_empty() {
                VariantNormalizationStatus::ServiceError
            } else {
                VariantNormalizationStatus::InvalidInput
            },
            input_description: string_field(result, "submitted_variant"),
            normalized_description: None,
            corrected_description: None,
            transcript_description: None,
            protein: None,
            genomic_descriptions,
            warnings,
            message: Some(
                "VariantValidator did not return a normalized transcript variant".to_string(),
            ),
        };
    }

    VariantNormalizationServiceResult {
        service: VariantNormalizationService::VariantValidator
            .as_str()
            .to_string(),
        status: VariantNormalizationStatus::Success,
        input_description: string_field(result, "submitted_variant"),
        normalized_description: transcript.clone(),
        corrected_description: None,
        transcript_description: transcript,
        protein,
        genomic_descriptions,
        warnings,
        message: None,
    }
}

fn http_error(status: reqwest::StatusCode, bytes: &[u8]) -> VariantNormalizationServiceResult {
    let message = format!("HTTP {status}: {}", crate::sources::body_excerpt(bytes));
    message_result(
        match status.as_u16() {
            400 | 422 => VariantNormalizationStatus::InvalidInput,
            404 => VariantNormalizationStatus::NotFound,
            _ => VariantNormalizationStatus::ServiceError,
        },
        message,
    )
}

fn message_result(
    status: VariantNormalizationStatus,
    message: String,
) -> VariantNormalizationServiceResult {
    VariantNormalizationServiceResult {
        service: VariantNormalizationService::VariantValidator
            .as_str()
            .to_string(),
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

fn genomic_descriptions(result: &serde_json::Value) -> Vec<String> {
    let mut values = result
        .get("primary_assembly_loci")
        .and_then(|v| v.get("grch38"))
        .into_iter()
        .filter_map(|locus| locus.get("hgvs_genomic_description"))
        .filter_map(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
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
