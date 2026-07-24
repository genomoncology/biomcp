use std::borrow::Cow;

use reqwest::StatusCode;
use serde_json::Value;

use crate::entities::variant::{
    CarAliasCollection, CarNormalizationBatchResponse, CarNormalizationItem,
    CarNormalizationStatus, CarProvenance,
};
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const CAR_BASE: &str = "https://reg.genome.network";
const CAR_BASE_ENV: &str = "BIOMCP_CLINGEN_CAR_BASE";
const CAR_BODY_LIMIT: usize = 256 * 1024;
const CAR_BATCH_BODY_LIMIT: usize = 2 * 1024 * 1024;
const CAR_FIELDS: &str = "none @id communityStandardTitle genomicAlleles transcriptAlleles.MANE externalRecords.dbSNP externalRecords.ClinVarVariations";

pub(crate) struct ClinGenAlleleRegistryClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ClinGenAlleleRegistryClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CAR_BASE, CAR_BASE_ENV),
        })
    }

    pub(crate) fn normalize_plan(hgvs: &str) -> RequestPlan {
        RequestPlan::get("allele")
            .query("hgvs", hgvs)
            .query("fields", CAR_FIELDS)
    }

    pub(crate) async fn normalize(&self, input: &str) -> Result<CarNormalizationItem, BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.base.as_ref(),
            &Self::normalize_plan(input),
        ))
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CAR))
        .await?;
        let status = response.status();
        let version = response
            .headers()
            .get("X-CAR-Version")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CAR),
            CAR_BODY_LIMIT,
        )
        .await?;
        Ok(decode_normalize_response(input, status, version, &bytes))
    }

    pub(crate) async fn normalize_batch(
        &self,
        inputs: &[String],
    ) -> Result<CarNormalizationBatchResponse, BioMcpError> {
        let url = format!("{}/alleles", self.base.trim_end_matches('/'));
        let response = crate::sources::apply_cache_mode(
            self.client
                .post(url)
                .query(&[("file", "hgvs"), ("fields", CAR_FIELDS)])
                .header(reqwest::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(format!("{}\n", inputs.join("\n"))),
        )
        .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CAR))
        .await?;
        let status = response.status();
        let version = response
            .headers()
            .get("X-CAR-Version")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CAR),
            CAR_BATCH_BODY_LIMIT,
        )
        .await?;
        let rows = serde_json::from_slice::<Vec<Value>>(&bytes).ok();
        let items: Vec<CarNormalizationItem> = rows
            .filter(|rows| rows.len() == inputs.len())
            .map(|rows| {
                rows.iter()
                    .zip(inputs)
                    .map(|(row, input)| {
                        decode_normalize_response(
                            input,
                            status,
                            version.clone(),
                            &serde_json::to_vec(row).unwrap_or_default(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                inputs
                    .iter()
                    .map(|input| {
                        empty(
                            input,
                            CarNormalizationStatus::Indeterminate,
                            false,
                            Some("CAR batch response did not preserve input positions".into()),
                            version.clone(),
                        )
                    })
                    .collect()
            });
        let complete = items.iter().all(|item| {
            !matches!(
                item.status,
                CarNormalizationStatus::Indeterminate | CarNormalizationStatus::Unavailable
            )
        });
        Ok(CarNormalizationBatchResponse {
            items,
            complete,
            provider: "ClinGen Allele Registry".into(),
        })
    }
}

fn empty(
    input: &str,
    status: CarNormalizationStatus,
    exhaustive: bool,
    error: Option<String>,
    version: Option<String>,
) -> CarNormalizationItem {
    CarNormalizationItem {
        input: input.to_owned(),
        status,
        exhaustive,
        caid: None,
        canonical_title: None,
        genomic_aliases: CarAliasCollection::default(),
        transcript_aliases: CarAliasCollection::default(),
        protein_aliases: CarAliasCollection::default(),
        external_ids: CarAliasCollection::default(),
        source: "clingen_car".into(),
        query: input.to_owned(),
        warnings: Vec::new(),
        error,
        provenance: CarProvenance {
            request_template_version: "1".into(),
            car_version: version,
        },
    }
}

pub(crate) fn decode_normalize_response(
    input: &str,
    status: StatusCode,
    version: Option<String>,
    bytes: &[u8],
) -> CarNormalizationItem {
    if status == StatusCode::BAD_REQUEST
        && serde_json::from_slice::<Value>(bytes)
            .ok()
            .is_some_and(|value| value.to_string().contains("HgvsParsingError"))
    {
        return empty(
            input,
            CarNormalizationStatus::Invalid,
            true,
            Some("CAR rejected the HGVS input".into()),
            version,
        );
    }
    if !status.is_success() {
        return empty(
            input,
            CarNormalizationStatus::Unavailable,
            false,
            Some(format!("CAR returned HTTP {}", status.as_u16())),
            version,
        );
    }
    let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR returned invalid JSON".into()),
            version,
        );
    };
    let Some(id) = value.get("@id").and_then(Value::as_str) else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR response lacked a canonical identity".into()),
            version,
        );
    };
    if id == "_:CA" {
        return empty(input, CarNormalizationStatus::NotFound, true, None, version);
    }
    let caid = id
        .rsplit('/')
        .next()
        .filter(|value| {
            value.starts_with("CA") && value[2..].bytes().all(|byte| byte.is_ascii_digit())
        })
        .map(str::to_owned);
    let Some(caid) = caid else {
        return empty(
            input,
            CarNormalizationStatus::Indeterminate,
            false,
            Some("CAR response had an unrecognized canonical identity".into()),
            version,
        );
    };
    let mut item = empty(input, CarNormalizationStatus::Resolved, true, None, version);
    item.caid = Some(caid);
    item.canonical_title = value
        .get("communityStandardTitle")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut genomic = Vec::new();
    for allele in value
        .get("genomicAlleles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        genomic.extend(
            allele
                .get("hgvs")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned),
        );
    }
    item.genomic_aliases = CarAliasCollection::bounded(genomic, 12);
    let mane = value
        .get("transcriptAlleles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|allele| {
            allele
                .pointer("/MANE/nucleotide/RefSeq/hgvs")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    item.transcript_aliases = CarAliasCollection::bounded(mane, 12);
    let proteins = value
        .get("transcriptAlleles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|allele| {
            allele
                .pointer("/MANE/protein/RefSeq/hgvs")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    item.protein_aliases = CarAliasCollection::bounded(proteins, 8);
    let mut external = value
        .pointer("/externalRecords/dbSNP")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|record| {
            record
                .get("rs")
                .and_then(Value::as_i64)
                .map(|id| format!("rs{id}"))
        })
        .collect::<Vec<_>>();
    external.extend(
        value
            .pointer("/externalRecords/ClinVarVariations")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|record| {
                record
                    .get("variationId")
                    .and_then(Value::as_i64)
                    .map(|id| format!("ClinVar:{id}"))
            }),
    );
    item.external_ids = CarAliasCollection::bounded(external, 8);
    item
}
