use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::entities::disease::Disease;
use crate::entities::gene::Gene;
use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const DISGENET_BASE: &str = "https://api.disgenet.com";
const DISGENET_API: &str = "disgenet";
const DISGENET_API_KEY_ENV: &str = "DISGENET_API_KEY";
const DISGENET_BASE_ENV: &str = "BIOMCP_DISGENET_BASE";
const DISGENET_DOCS_URL: &str = "https://www.disgenet.com/";

#[derive(Clone)]
pub struct DisgenetClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl DisgenetClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(DISGENET_BASE, DISGENET_BASE_ENV),
            api_key: std::env::var(DISGENET_API_KEY_ENV)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    fn require_api_key(&self) -> Result<&str, BioMcpError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| BioMcpError::ApiKeyRequired {
                api: DISGENET_API.to_string(),
                env_var: DISGENET_API_KEY_ENV.to_string(),
                docs_url: DISGENET_DOCS_URL.to_string(),
            })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send()
            .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = crate::sources::read_limited_body(resp, DISGENET_API).await?;
        Self::decode_json_response(status, &headers, &bytes)
    }

    fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        headers: &HeaderMap,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
            return Err(BioMcpError::ApiKeyRejected {
                api: DISGENET_API.to_string(),
                env_var: DISGENET_API_KEY_ENV.to_string(),
                docs_url: DISGENET_DOCS_URL.to_string(),
            });
        }

        let retry_after = parse_retry_after_seconds(headers);
        let content_type = headers.get(CONTENT_TYPE).cloned();
        crate::sources::ensure_json_content_type(DISGENET_API, content_type.as_ref(), bytes)?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            let excerpt = crate::sources::body_excerpt(bytes);
            let detail = match retry_after {
                Some(seconds) => format!("{excerpt}. Retry after {seconds} seconds."),
                None => excerpt,
            };
            return Err(BioMcpError::Api {
                api: DISGENET_API.to_string(),
                message: format!("HTTP {status}: {detail}"),
            });
        }

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: DISGENET_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: DISGENET_API.to_string(),
            source,
        })
    }

    fn gene_associations_plan(
        gene: &Gene,
        limit: usize,
        api_key: &str,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        if limit == 0 {
            return Ok(None);
        }

        let mut plan = authenticated_get("api/v1/gda/summary", api_key).query("page_number", "0");
        if !gene.entrez_id.trim().is_empty() {
            plan = plan.query("gene_ncbi_id", gene.entrez_id.trim());
        } else if !gene.symbol.trim().is_empty() {
            plan = plan.query("gene_symbol", gene.symbol.trim());
        } else {
            return Err(BioMcpError::InvalidArgument(
                "DisGeNET gene lookup requires a gene symbol or Entrez ID".into(),
            ));
        }

        Ok(Some(plan))
    }

    fn disease_associations_plan(
        disease_id: &str,
        limit: usize,
        api_key: &str,
    ) -> Option<RequestPlan> {
        if limit == 0 {
            return None;
        }

        Some(
            authenticated_get("api/v1/gda/summary", api_key)
                .query("disease", disease_id)
                .query("page_number", "0"),
        )
    }

    fn disease_resolution_plan(query: &str, api_key: &str) -> RequestPlan {
        authenticated_get("api/v1/entity/disease", api_key)
            .query("disease_free_text_search_string", query)
    }

    fn associations_from_response(
        resp: DisgenetResponse<DisgenetGdaSummaryRow>,
        limit: usize,
    ) -> Result<Vec<DisgenetAssociationRecord>, BioMcpError> {
        Ok(validate_response(resp)?
            .into_iter()
            .take(limit)
            .map(DisgenetAssociationRecord::from)
            .collect())
    }

    fn disease_id_from_response(
        query: &str,
        resp: DisgenetResponse<DisgenetDiseaseRow>,
    ) -> Result<String, BioMcpError> {
        let rows = validate_response(resp)?;

        if let Some(row) = select_disease_match(query, &rows) {
            return Ok(format!("UMLS_{}", row.disease_umls_cui));
        }

        Err(BioMcpError::SourceUnavailable {
            source_name: DISGENET_API.to_string(),
            reason: format!("No DisGeNET disease identifier matched \"{query}\"."),
            suggestion:
                "Try a more specific disease query or resolve a disease with a UMLS CUI first."
                    .into(),
        })
    }

    pub async fn fetch_gene_associations(
        &self,
        gene: &Gene,
        limit: usize,
    ) -> Result<Vec<DisgenetAssociationRecord>, BioMcpError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let api_key = self.require_api_key()?;
        let plan = Self::gene_associations_plan(gene, limit, api_key)?.expect("limit checked");
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);

        let resp: DisgenetResponse<DisgenetGdaSummaryRow> = self.get_json(req).await?;
        Self::associations_from_response(resp, limit)
    }

    pub async fn fetch_disease_associations(
        &self,
        disease: &Disease,
        limit: usize,
    ) -> Result<Vec<DisgenetAssociationRecord>, BioMcpError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let api_key = self.require_api_key()?;
        let disease_id = match disease
            .xrefs
            .get("umls_cui")
            .and_then(|value| normalize_umls_cui(value))
        {
            Some(value) => value,
            None => self.resolve_disease_id(&disease.name).await?,
        };

        let plan = Self::disease_associations_plan(disease_id.as_str(), limit, api_key)
            .expect("limit checked");
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);

        let resp: DisgenetResponse<DisgenetGdaSummaryRow> = self.get_json(req).await?;
        Self::associations_from_response(resp, limit)
    }

    async fn resolve_disease_id(&self, name: &str) -> Result<String, BioMcpError> {
        let query = name.trim();
        if query.is_empty() {
            return Err(BioMcpError::SourceUnavailable {
                source_name: DISGENET_API.to_string(),
                reason: "Disease name is required for DisGeNET disease resolution.".into(),
                suggestion: "Try a more specific disease query or use a disease with a UMLS CUI."
                    .into(),
            });
        }

        let api_key = self.require_api_key()?;
        let plan = Self::disease_resolution_plan(query, api_key);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let resp: DisgenetResponse<DisgenetDiseaseRow> = self.get_json(req).await?;
        Self::disease_id_from_response(query, resp)
    }
}

fn authenticated_get(path: &str, api_key: &str) -> RequestPlan {
    RequestPlan::get(path)
        .header("Authorization", api_key)
        .header("accept", "application/json")
}

fn validate_response<T>(resp: DisgenetResponse<T>) -> Result<Vec<T>, BioMcpError> {
    let DisgenetResponse {
        status,
        http_status,
        paging,
        warnings,
        payload,
    } = resp;

    if let Some(status_text) = status.as_deref()
        && status_text != "OK"
    {
        let message = match http_status {
            Some(code) => format!("response status {status_text} (httpStatus {code})"),
            None => format!("response status {status_text}"),
        };
        return Err(BioMcpError::Api {
            api: DISGENET_API.to_string(),
            message,
        });
    }

    if !warnings.is_empty() || paging.is_some() {
        let page_size = paging.as_ref().map(|value| value.page_size);
        let total_elements = paging.as_ref().map(|value| value.total_elements);
        let total_elements_in_page = paging.as_ref().map(|value| value.total_elements_in_page);
        let current_page_number = paging.as_ref().map(|value| value.current_page_number);
        debug!(
            ?warnings,
            ?page_size,
            ?total_elements,
            ?total_elements_in_page,
            ?current_page_number,
            "DisGeNET response metadata"
        );
    }

    Ok(payload.unwrap_or_default())
}

fn parse_retry_after_seconds(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("X-Rate-Limit-Retry-After-Seconds")
        .or_else(|| headers.get("x-rate-limit-retry-after-seconds"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
}

fn normalize_umls_cui(value: &str) -> Option<String> {
    let mut cui = value.trim().to_ascii_uppercase();
    if cui.is_empty() {
        return None;
    }
    if let Some(stripped) = cui.strip_prefix("UMLS_") {
        cui = stripped.to_string();
    }
    if let Some(stripped) = cui.strip_prefix("UMLS:") {
        cui = stripped.to_string();
    }
    (!cui.is_empty())
        .then_some(cui)
        .map(|cui| format!("UMLS_{cui}"))
}

fn normalize_label(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn select_disease_match<'a>(
    query: &str,
    rows: &'a [DisgenetDiseaseRow],
) -> Option<&'a DisgenetDiseaseRow> {
    let normalized_query = normalize_label(query);
    if normalized_query.is_empty() {
        return rows.first();
    }

    rows.iter()
        .find(|row| normalize_label(&row.name) == normalized_query)
        .or_else(|| {
            rows.iter().find(|row| {
                row.synonyms
                    .iter()
                    .any(|synonym| normalize_label(&synonym.name) == normalized_query)
            })
        })
        .or_else(|| {
            rows.iter().max_by(|left, right| {
                left.search_rank
                    .partial_cmp(&right.search_rank)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        })
}

#[derive(Debug, Clone)]
pub struct DisgenetAssociationRecord {
    pub gene_symbol: String,
    pub gene_ncbi_id: Option<u32>,
    pub disease_name: String,
    pub disease_umls_cui: String,
    pub score: f64,
    pub publication_count: Option<u32>,
    pub clinical_trial_count: Option<u32>,
    pub evidence_index: Option<f64>,
    pub evidence_level: Option<String>,
}

impl From<DisgenetGdaSummaryRow> for DisgenetAssociationRecord {
    fn from(value: DisgenetGdaSummaryRow) -> Self {
        Self {
            gene_symbol: value.symbol_of_gene,
            gene_ncbi_id: value.gene_ncbi_id,
            disease_name: value.disease_name,
            disease_umls_cui: value.disease_umls_cui,
            score: value.score,
            publication_count: value.num_pmids,
            clinical_trial_count: value.num_ct_supporting_association,
            evidence_index: value.ei,
            evidence_level: value.el,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisgenetResponse<T> {
    status: Option<String>,
    http_status: Option<u16>,
    paging: Option<DisgenetPaging>,
    #[serde(default)]
    warnings: Vec<String>,
    payload: Option<Vec<T>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisgenetPaging {
    page_size: u32,
    total_elements: u32,
    total_elements_in_page: u32,
    current_page_number: u32,
}

#[derive(Debug, Deserialize)]
struct DisgenetGdaSummaryRow {
    #[serde(rename = "symbolOfGene")]
    symbol_of_gene: String,
    #[serde(rename = "geneNcbiID")]
    gene_ncbi_id: Option<u32>,
    #[serde(rename = "diseaseName")]
    disease_name: String,
    #[serde(rename = "diseaseUMLSCUI")]
    disease_umls_cui: String,
    score: f64,
    #[serde(rename = "numPMIDs")]
    num_pmids: Option<u32>,
    #[serde(rename = "numCTsupportingAssociation")]
    num_ct_supporting_association: Option<u32>,
    ei: Option<f64>,
    el: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DisgenetDiseaseRow {
    name: String,
    #[serde(rename = "diseaseUMLSCUI")]
    disease_umls_cui: String,
    #[serde(default)]
    search_rank: f64,
    #[serde(default)]
    synonyms: Vec<DisgenetDiseaseSynonym>,
}

#[derive(Debug, Deserialize)]
struct DisgenetDiseaseSynonym {
    name: String,
}

#[cfg(test)]
mod tests;
