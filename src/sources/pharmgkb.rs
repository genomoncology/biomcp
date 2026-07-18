use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashSet;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const PHARMGKB_BASE: &str = "https://api.pharmgkb.org/v1";
const PHARMGKB_BASE_ENV: &str = "BIOMCP_PHARMGKB_BASE";

pub struct PharmGkbClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl PharmGkbClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PHARMGKB_BASE, PHARMGKB_BASE_ENV),
        })
    }

    pub(crate) fn drug_annotation_plans(
        drug_name: &str,
        limit: usize,
    ) -> Result<Vec<AnnotationPlan>, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 100);
        Ok(vec![
            annotation_plan(
                "clinicalAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Clinical Annotation",
                limit,
            ),
            annotation_plan(
                "guidelineAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Guideline Annotation",
                limit,
            ),
            annotation_plan(
                "labelAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Label Annotation",
                limit,
            ),
        ])
    }

    pub(crate) fn gene_annotation_plans(
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<AnnotationPlan>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 100);
        Ok(vec![
            annotation_plan(
                "clinicalAnnotation",
                "location.genes.symbol",
                &gene_symbol,
                "Clinical Annotation",
                limit,
            ),
            annotation_plan(
                "guidelineAnnotation",
                "relatedGenes.symbol",
                &gene_symbol,
                "Guideline Annotation",
                limit,
            ),
            annotation_plan(
                "labelAnnotation",
                "relatedGenes.symbol",
                &gene_symbol,
                "Label Annotation",
                limit,
            ),
        ])
    }

    pub(crate) fn decode_json_optional<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<Option<T>, BioMcpError> {
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::PHARMGKB),
            status,
            content_type,
            bytes,
            true,
        )
        .map(Some)
    }

    pub(crate) fn annotations_from_response(
        resp: PharmGkbDataResponse,
        fallback_kind: &str,
        limit: usize,
    ) -> Vec<PharmGkbAnnotation> {
        let mut out = Vec::new();
        for row in resp.data {
            if let Some(annotation) = map_annotation(&row, fallback_kind) {
                out.push(annotation);
            }
            if out.len() >= limit {
                break;
            }
        }
        out
    }

    async fn get_json_optional<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<Option<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PHARMGKB,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PHARMGKB),
        )
        .await?;
        Self::decode_json_optional(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PHARMGKB,
            ))
        })
    }

    pub async fn annotations_by_drug(
        &self,
        drug_name: &str,
        limit: usize,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let mut out = Vec::new();
        for plan in Self::drug_annotation_plans(drug_name, limit)? {
            out.extend(self.fetch_annotations(plan).await?);
        }

        Ok(dedupe_and_limit(out, limit.clamp(1, 100)))
    }

    pub async fn annotations_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let mut out = Vec::new();
        for plan in Self::gene_annotation_plans(gene_symbol, limit)? {
            out.extend(self.fetch_annotations(plan).await?);
        }

        Ok(dedupe_and_limit(out, limit.clamp(1, 100)))
    }

    async fn fetch_annotations(
        &self,
        plan: AnnotationPlan,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan.request);
        let Some(resp): Option<PharmGkbDataResponse> = self.get_json_optional(req).await? else {
            return Ok(Vec::new());
        };
        Ok(Self::annotations_from_response(
            resp,
            plan.fallback_kind,
            plan.limit,
        ))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AnnotationPlan {
    pub request: RequestPlan,
    pub fallback_kind: &'static str,
    pub limit: usize,
}

fn annotation_plan(
    endpoint: &str,
    criteria_key: &str,
    criteria_value: &str,
    fallback_kind: &'static str,
    limit: usize,
) -> AnnotationPlan {
    AnnotationPlan {
        request: RequestPlan::get(format!("data/{endpoint}"))
            .query(criteria_key, criteria_value)
            .query("view", "min"),
        fallback_kind,
        limit,
    }
}

fn normalize_drug_name(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "PGx drug is required. Example: biomcp get pgx warfarin".into(),
        ));
    }
    if normalized.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is too long.".into(),
        ));
    }
    Ok(normalized)
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "PGx gene is required. Example: biomcp get pgx CYP2D6".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn dedupe_and_limit(rows: Vec<PharmGkbAnnotation>, limit: usize) -> Vec<PharmGkbAnnotation> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let key = format!("{}|{}|{}", row.kind, row.id, row.title.to_ascii_lowercase());
        if !seen.insert(key) {
            continue;
        }
        out.push(row);
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn map_annotation(row: &serde_json::Value, fallback_kind: &str) -> Option<PharmGkbAnnotation> {
    let obj = row.as_object()?;

    let id = obj
        .get("id")
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty())?;

    let kind = obj
        .get("objCls")
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| fallback_kind.to_string());

    let title = obj
        .get("name")
        .and_then(to_string_value)
        .or_else(|| obj.get("accessionId").and_then(to_string_value))
        .or_else(|| {
            obj.get("location")
                .and_then(|v| v.get("displayName"))
                .and_then(to_string_value)
        })
        .unwrap_or_else(|| id.clone());

    let level = obj
        .get("levelOfEvidence")
        .and_then(|v| v.get("term"))
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty());

    let url = obj
        .get("crossReferences")
        .and_then(|v| v.as_array())
        .and_then(|rows| {
            rows.iter().find_map(|xref| {
                xref.get("_url")
                    .and_then(to_string_value)
                    .or_else(|| xref.get("resourceId").and_then(to_string_value))
            })
        })
        .filter(|v| v.starts_with("http"));

    Some(PharmGkbAnnotation {
        source: "PharmGKB".to_string(),
        kind,
        id,
        title,
        level,
        url,
    })
}

fn to_string_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(v) => Some(v.trim().to_string()),
        serde_json::Value::Number(v) => Some(v.to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PharmGkbDataResponse {
    #[serde(default)]
    data: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PharmGkbAnnotation {
    pub source: String,
    pub kind: String,
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests;
