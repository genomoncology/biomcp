use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::OnceLock;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const GTEX_BASE: &str = "https://gtexportal.org";
const GTEX_BASE_ENV: &str = "BIOMCP_GTEX_BASE";
const GTEX_DATASET_ID: &str = "gtex_v8";
const GTEX_GENCODE_VERSION: &str = "v26";
const GTEX_TOP_TISSUES: usize = 10;
const GTEX_LOW_TISSUES: usize = 3;

pub struct GtexClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl GtexClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(GTEX_BASE, GTEX_BASE_ENV),
        })
    }

    pub(crate) fn gene_search_plan(ensembl_id: &str) -> Result<RequestPlan, BioMcpError> {
        let ensembl_id = normalize_ensembl_id(ensembl_id)?;
        Ok(RequestPlan::get("api/v2/reference/geneSearch")
            .query("geneId", ensembl_id)
            .query("gencodeVersion", GTEX_GENCODE_VERSION))
    }

    pub(crate) fn median_expression_plan(versioned_gencode_id: &str) -> RequestPlan {
        RequestPlan::get("api/v2/expression/medianGeneExpression")
            .query("gencodeId", versioned_gencode_id.trim())
            .query("datasetId", GTEX_DATASET_ID)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::GTEX),
            status,
            content_type,
            bytes,
            true,
        )
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GTEX,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::GTEX),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GTEX,
            ))
        })
    }

    pub async fn median_gene_expression(
        &self,
        ensembl_id: &str,
    ) -> Result<Vec<TissueExpression>, BioMcpError> {
        let ensembl_id = normalize_ensembl_id(ensembl_id)?;
        let _guard = gtex_sequence_lock().lock().await;
        let Some(versioned_id) = self
            .resolve_versioned_gencode_id_unlocked(&ensembl_id)
            .await?
        else {
            return Ok(Vec::new());
        };
        let rows = self.fetch_median_expression_unlocked(&versioned_id).await?;
        Ok(compact_tissue_rows(rows))
    }

    async fn resolve_versioned_gencode_id_unlocked(
        &self,
        ensembl_id: &str,
    ) -> Result<Option<String>, BioMcpError> {
        let plan = Self::gene_search_plan(ensembl_id)?;
        let resp: GtexGeneSearchResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Self::resolve_versioned_gencode_id_from_response(ensembl_id, resp)
    }

    fn resolve_versioned_gencode_id_from_response(
        ensembl_id: &str,
        resp: GtexGeneSearchResponse,
    ) -> Result<Option<String>, BioMcpError> {
        let mut first_non_empty: Option<String> = None;
        for row in resp.data {
            let Some(gencode_id) = clean_optional(row.gencode_id) else {
                continue;
            };

            if first_non_empty.is_none() {
                first_non_empty = Some(gencode_id.clone());
            }

            if gencode_id == ensembl_id
                || gencode_id
                    .strip_suffix(".0")
                    .is_some_and(|base| base == ensembl_id)
                || gencode_id.starts_with(&format!("{ensembl_id}."))
            {
                return Ok(Some(gencode_id));
            }
        }

        Ok(first_non_empty)
    }

    async fn fetch_median_expression_unlocked(
        &self,
        versioned_gencode_id: &str,
    ) -> Result<Vec<TissueExpression>, BioMcpError> {
        let plan = Self::median_expression_plan(versioned_gencode_id);
        let resp: GtexMedianExpressionResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::median_expression_rows_from_response(resp))
    }

    fn median_expression_rows_from_response(
        resp: GtexMedianExpressionResponse,
    ) -> Vec<TissueExpression> {
        let mut rows: Vec<TissueExpression> = resp
            .data
            .into_iter()
            .filter_map(|row| {
                let tissue = normalize_tissue_label(row.tissue_site_detail_id)?;
                let median_tpm = row.median?;
                median_tpm
                    .is_finite()
                    .then_some(TissueExpression { tissue, median_tpm })
            })
            .collect();

        rows.sort_by(|a, b| {
            b.median_tpm
                .partial_cmp(&a.median_tpm)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.tissue.cmp(&b.tissue))
        });
        rows
    }
}

fn gtex_sequence_lock() -> &'static Mutex<()> {
    static GTEX_SEQUENCE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    GTEX_SEQUENCE_LOCK.get_or_init(|| Mutex::new(()))
}

fn normalize_ensembl_id(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim().to_ascii_uppercase();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Ensembl gene ID is required for GTEx expression".into(),
        ));
    }

    let core = raw.split('.').next().unwrap_or(&raw).trim();
    if core.is_empty() || !core.starts_with("ENSG") {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid Ensembl gene ID: {value}"
        )));
    }
    if !core.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid Ensembl gene ID: {value}"
        )));
    }
    Ok(core.to_string())
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalize_tissue_label(value: Option<String>) -> Option<String> {
    let value = value?;
    let value = value.replace('_', " ");
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn compact_tissue_rows(rows: Vec<TissueExpression>) -> Vec<TissueExpression> {
    if rows.len() <= GTEX_TOP_TISSUES + GTEX_LOW_TISSUES {
        return rows;
    }

    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for row in rows.iter().take(GTEX_TOP_TISSUES) {
        let key = row.tissue.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(row.clone());
        }
    }

    for row in rows.iter().rev().take(GTEX_LOW_TISSUES) {
        let key = row.tissue.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(row.clone());
        }
    }

    out
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneExpression {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tissues: Vec<TissueExpression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueExpression {
    pub tissue: String,
    pub median_tpm: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct GtexGeneSearchResponse {
    #[serde(default)]
    data: Vec<GtexGeneSearchRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct GtexMedianExpressionResponse {
    #[serde(default)]
    data: Vec<GtexMedianExpressionRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct GtexGeneSearchRow {
    #[serde(rename = "gencodeId")]
    gencode_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GtexMedianExpressionRow {
    median: Option<f64>,
    #[serde(rename = "tissueSiteDetailId")]
    tissue_site_detail_id: Option<String>,
}

#[cfg(test)]
mod tests;
