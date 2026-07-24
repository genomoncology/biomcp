//! Transcript HGVS normalization proxy orchestration.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::error::BioMcpError;

const MAX_TRANSCRIPT_HGVS_LEN: usize = 512;

pub(crate) fn transcript_coding_hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{2}_[0-9]+\.[0-9]+:c\.[^\s]+$").expect("valid regex"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantNormalizationResponse {
    pub input: String,
    pub services: Vec<VariantNormalizationServiceResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantNormalizationServiceResult {
    pub service: String,
    pub status: VariantNormalizationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corrected_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protein: Option<serde_json::Value>,
    pub genomic_descriptions: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariantNormalizationStatus {
    Success,
    InvalidInput,
    UnsupportedNotation,
    NotFound,
    NotQueryable,
    ServiceError,
}

impl VariantNormalizationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::InvalidInput => "invalid_input",
            Self::UnsupportedNotation => "unsupported_notation",
            Self::NotFound => "not_found",
            Self::NotQueryable => "not_queryable",
            Self::ServiceError => "service_error",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantNormalizationService {
    Mutalyzer,
    VariantValidator,
}

impl VariantNormalizationService {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mutalyzer => "mutalyzer",
            Self::VariantValidator => "variantvalidator",
        }
    }
}

pub fn parse_variant_normalization_services(
    value: &str,
) -> Result<Vec<VariantNormalizationService>, BioMcpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "all" => Ok(vec![
            VariantNormalizationService::Mutalyzer,
            VariantNormalizationService::VariantValidator,
        ]),
        "mutalyzer" => Ok(vec![VariantNormalizationService::Mutalyzer]),
        "variantvalidator" => Ok(vec![VariantNormalizationService::VariantValidator]),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Invalid normalization service: {other}. Expected one of: all, mutalyzer, variantvalidator"
        ))),
    }
}

pub fn validate_transcript_hgvs_input(input: &str) -> Result<String, BioMcpError> {
    let trimmed = input.trim();
    if trimmed.is_empty()
        || trimmed.len() > MAX_TRANSCRIPT_HGVS_LEN
        || !transcript_coding_hgvs_re().is_match(trimmed)
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "unsupported_notation: variant normalize requires explicit transcript HGVS input such as NM_000248.3:c.135del; submitted input: '{trimmed}'. BioMCP does not parse report prose or choose/guess transcripts."
        )));
    }
    Ok(trimmed.to_string())
}

pub async fn normalize_variant(
    service: &str,
    input: &str,
) -> Result<VariantNormalizationResponse, BioMcpError> {
    let services = parse_variant_normalization_services(service)?;
    let input = validate_transcript_hgvs_input(input)?;
    let mut results = Vec::with_capacity(services.len());

    for service in services {
        let result = match service {
            VariantNormalizationService::Mutalyzer => {
                crate::sources::mutalyzer::MutalyzerClient::new()?
                    .normalize(&input)
                    .await
            }
            VariantNormalizationService::VariantValidator => {
                crate::sources::variantvalidator::VariantValidatorClient::new()?
                    .normalize(&input)
                    .await
            }
        }
        .unwrap_or_else(|err| service_error_result(service, err));
        results.push(result);
    }

    Ok(VariantNormalizationResponse {
        input,
        services: results,
    })
}

fn service_error_result(
    service: VariantNormalizationService,
    err: BioMcpError,
) -> VariantNormalizationServiceResult {
    VariantNormalizationServiceResult {
        service: service.as_str().to_string(),
        status: VariantNormalizationStatus::ServiceError,
        input_description: None,
        normalized_description: None,
        corrected_description: None,
        transcript_description: None,
        protein: None,
        genomic_descriptions: Vec::new(),
        warnings: Vec::new(),
        message: Some(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_first_slice_transcript_coding_hgvs() {
        assert_eq!(
            validate_transcript_hgvs_input(" NM_000248.3:c.135del ").unwrap(),
            "NM_000248.3:c.135del"
        );
        assert!(validate_transcript_hgvs_input("NM_004448.2:c.829G>T").is_ok());
    }

    #[test]
    fn all_selector_includes_car_after_existing_normalizers() {
        let services = parse_variant_normalization_services("all").expect("all selector");
        assert_eq!(
            services
                .into_iter()
                .map(VariantNormalizationService::as_str)
                .collect::<Vec<_>>(),
            vec!["mutalyzer", "variantvalidator", "car"]
        );
    }

    #[test]
    fn rejects_non_transcript_guardrail_inputs() {
        let err = validate_transcript_hgvs_input("BRAF V600E").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("unsupported_notation"));
        assert!(text.contains("BRAF V600E"));
        assert!(text.contains("transcript HGVS"));
    }
}
