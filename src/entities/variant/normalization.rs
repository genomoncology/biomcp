//! Transcript HGVS normalization proxy orchestration.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::error::BioMcpError;

const MAX_TRANSCRIPT_HGVS_LEN: usize = 512;
const MAX_CAR_HGVS_LEN: usize = 512;

pub(crate) fn car_hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^(?:NM_[0-9]+\.[0-9]+:c\.|NC_[0-9]+\.[0-9]+:g\.)[^\s]+$").expect("valid regex")
    })
}

pub(crate) fn transcript_coding_hgvs_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[A-Z]{2}_[0-9]+\.[0-9]+:c\.[^\s]+$").expect("valid regex"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantNormalizationResponse {
    pub input: String,
    pub services: Vec<VariantNormalizationAggregate>,
}

/// Aggregate rows preserve legacy provider shape while CAR retains its own item schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VariantNormalizationAggregate {
    Legacy(VariantNormalizationServiceResult),
    Car(CarNormalizationAggregate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarNormalizationAggregate {
    pub service: String,
    #[serde(flatten)]
    pub item: CarNormalizationItem,
}

impl From<VariantNormalizationServiceResult> for VariantNormalizationAggregate {
    fn from(value: VariantNormalizationServiceResult) -> Self {
        Self::Legacy(value)
    }
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
    pub genomic_descriptions: Vec<crate::entities::GenomicCoordinate>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarAliasCollection {
    pub values: Vec<String>,
    pub source_count: usize,
    pub truncated: bool,
}

impl CarAliasCollection {
    pub(crate) fn bounded(values: Vec<String>, limit: usize) -> Self {
        let mut unique = Vec::with_capacity(values.len());
        for value in values {
            if !unique.contains(&value) {
                unique.push(value);
            }
        }
        let source_count = unique.len();
        unique.truncate(limit);
        Self {
            values: unique,
            source_count,
            truncated: source_count > limit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CarNormalizationStatus {
    Resolved,
    NotFound,
    Invalid,
    Indeterminate,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarProvenance {
    pub request_template_version: String,
    pub car_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarNormalizationItem {
    pub input: String,
    pub status: CarNormalizationStatus,
    pub exhaustive: bool,
    pub caid: Option<String>,
    pub canonical_title: Option<String>,
    pub genomic_aliases: CarAliasCollection,
    pub transcript_aliases: CarAliasCollection,
    pub protein_aliases: CarAliasCollection,
    pub external_ids: CarAliasCollection,
    pub source: String,
    pub query: String,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub provenance: CarProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarNormalizationBatchResponse {
    pub items: Vec<CarNormalizationItem>,
    pub complete: bool,
    pub provider: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantNormalizationService {
    Mutalyzer,
    VariantValidator,
    Car,
}

impl VariantNormalizationService {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mutalyzer => "mutalyzer",
            Self::VariantValidator => "variantvalidator",
            Self::Car => "car",
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
            VariantNormalizationService::Car,
        ]),
        "mutalyzer" => Ok(vec![VariantNormalizationService::Mutalyzer]),
        "variantvalidator" => Ok(vec![VariantNormalizationService::VariantValidator]),
        "car" => Ok(vec![VariantNormalizationService::Car]),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Invalid normalization service: {other}. Expected one of: all, mutalyzer, variantvalidator, car"
        ))),
    }
}

pub fn validate_car_hgvs_input(input: &str) -> Result<String, BioMcpError> {
    let trimmed = input.trim();
    let valid = trimmed.len() <= MAX_CAR_HGVS_LEN && car_hgvs_re().is_match(trimmed);
    if !valid {
        return Err(BioMcpError::InvalidArgument(
            "unsupported_notation: CAR requires versioned RefSeq NM_:c. or NC_:g. HGVS input"
                .into(),
        ));
    }
    Ok(trimmed.to_owned())
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

pub async fn normalize_car(input: &str) -> Result<CarNormalizationItem, BioMcpError> {
    let input = validate_car_hgvs_input(input)?;
    crate::sources::clingen_allele_registry::ClinGenAlleleRegistryClient::new()?
        .normalize(&input)
        .await
}

pub async fn normalize_car_batch(
    inputs: Vec<String>,
) -> Result<CarNormalizationBatchResponse, BioMcpError> {
    if !(1..=50).contains(&inputs.len()) {
        return Err(BioMcpError::InvalidArgument(
            "CAR batch input must contain 1-50 HGVS strings".into(),
        ));
    }
    let inputs = inputs
        .iter()
        .map(|input| validate_car_hgvs_input(input))
        .collect::<Result<Vec<_>, _>>()?;
    crate::sources::clingen_allele_registry::ClinGenAlleleRegistryClient::new()?
        .normalize_batch(&inputs)
        .await
}

pub async fn normalize_variant(
    service: &str,
    input: &str,
) -> Result<VariantNormalizationResponse, BioMcpError> {
    let services = parse_variant_normalization_services(service)?;
    let input = if services == [VariantNormalizationService::Car] {
        validate_car_hgvs_input(input)?
    } else {
        validate_transcript_hgvs_input(input)?
    };
    if services
        == [
            VariantNormalizationService::Mutalyzer,
            VariantNormalizationService::VariantValidator,
            VariantNormalizationService::Car,
        ]
    {
        let (mutalyzer, variantvalidator, car) = tokio::join!(
            async {
                crate::sources::mutalyzer::MutalyzerClient::new()?
                    .normalize(&input)
                    .await
            },
            async {
                crate::sources::variantvalidator::VariantValidatorClient::new()?
                    .normalize(&input)
                    .await
            },
            async { normalize_car(&input).await },
        );
        return Ok(VariantNormalizationResponse {
            input: input.clone(),
            services: vec![
                VariantNormalizationAggregate::Legacy(mutalyzer.unwrap_or_else(|err| {
                    service_error_result(VariantNormalizationService::Mutalyzer, err)
                })),
                VariantNormalizationAggregate::Legacy(variantvalidator.unwrap_or_else(|err| {
                    service_error_result(VariantNormalizationService::VariantValidator, err)
                })),
                VariantNormalizationAggregate::Car(CarNormalizationAggregate {
                    service: "car".into(),
                    item: car.unwrap_or_else(|err| unavailable_car_item(&input, err)),
                }),
            ],
        });
    }

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
            VariantNormalizationService::Car => {
                results.push(VariantNormalizationAggregate::Car(
                    CarNormalizationAggregate {
                        service: "car".into(),
                        item: normalize_car(&input).await?,
                    },
                ));
                continue;
            }
        }
        .unwrap_or_else(|err| service_error_result(service, err));
        results.push(VariantNormalizationAggregate::Legacy(result));
    }

    Ok(VariantNormalizationResponse {
        input,
        services: results,
    })
}

fn unavailable_car_item(input: &str, _error: BioMcpError) -> CarNormalizationItem {
    CarNormalizationItem {
        input: input.into(),
        status: CarNormalizationStatus::Unavailable,
        exhaustive: false,
        caid: None,
        canonical_title: None,
        genomic_aliases: CarAliasCollection::default(),
        transcript_aliases: CarAliasCollection::default(),
        protein_aliases: CarAliasCollection::default(),
        external_ids: CarAliasCollection::default(),
        source: "clingen_car".into(),
        query: input.into(),
        warnings: Vec::new(),
        error: Some("ClinGen Allele Registry request failed".into()),
        provenance: CarProvenance {
            request_template_version: "1".into(),
            car_version: None,
            response_sha256: None,
        },
    }
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
    fn rejects_non_transcript_guardrail_inputs() {
        let err = validate_transcript_hgvs_input("BRAF V600E").unwrap_err();
        let text = err.to_string();
        assert!(text.contains("unsupported_notation"));
        assert!(text.contains("BRAF V600E"));
        assert!(text.contains("transcript HGVS"));
    }

    #[test]
    fn car_grammar_accepts_only_versioned_refseq_coding_or_genomic_hgvs() {
        for input in ["NM_000546.6:c.215C>G", "NC_000017.11:g.7674220C>G"] {
            assert!(validate_car_hgvs_input(input).is_ok(), "{input}");
        }
        for input in [
            "NM_fake.1:x:c.",
            "NM_000546:c.215C>G",
            "NC_000017.11:c.7674220C>G",
            "NM_000546.6:g.215C>G",
            "chr17:g.7674220C>G",
        ] {
            assert!(validate_car_hgvs_input(input).is_err(), "{input}");
        }
    }

    #[tokio::test]
    async fn invalid_grammar_is_rejected_before_any_request_is_constructed() {
        let result = normalize_car("BRCA1").await;

        assert!(matches!(result, Err(BioMcpError::InvalidArgument(_))));
    }

    #[tokio::test]
    async fn batch_limit_is_rejected_before_any_request_is_constructed() {
        let result = normalize_car_batch(vec!["NM_000546.6:c.215C>G".into(); 51]).await;

        assert!(matches!(result, Err(BioMcpError::InvalidArgument(_))));
    }

    #[test]
    fn car_alias_metadata_describes_distinct_returned_aliases() {
        let aliases = CarAliasCollection::bounded(vec!["NC_000017.11:g.1A>G".into(); 20], 12);

        assert_eq!(aliases.values, ["NC_000017.11:g.1A>G"]);
        assert_eq!(aliases.source_count, aliases.values.len());
        assert!(!aliases.truncated);
    }
}
