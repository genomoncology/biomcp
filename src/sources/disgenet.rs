use std::borrow::Cow;

use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::entities::disease::Disease;
use crate::entities::gene::Gene;
use crate::error::BioMcpError;

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

    #[cfg(test)]
    fn new_for_test(base: String, api_key: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key: api_key
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
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
        let retry_after = parse_retry_after_seconds(resp.headers());
        let content_type = resp.headers().get(CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, DISGENET_API).await?;
        crate::sources::ensure_json_content_type(DISGENET_API, content_type.as_ref(), &bytes)?;

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let excerpt = crate::sources::body_excerpt(&bytes);
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
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: DISGENET_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: DISGENET_API.to_string(),
            source,
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
        let url = self.endpoint("/api/v1/gda/summary");
        let mut req = self
            .client
            .get(&url)
            .header("Authorization", api_key)
            .header("accept", "application/json")
            .query(&[("page_number", "0")]);

        if !gene.entrez_id.trim().is_empty() {
            req = req.query(&[("gene_ncbi_id", gene.entrez_id.trim())]);
        } else if !gene.symbol.trim().is_empty() {
            req = req.query(&[("gene_symbol", gene.symbol.trim())]);
        } else {
            return Err(BioMcpError::InvalidArgument(
                "DisGeNET gene lookup requires a gene symbol or Entrez ID".into(),
            ));
        }

        let resp: DisgenetResponse<DisgenetGdaSummaryRow> = self.get_json(req).await?;
        Ok(validate_response(resp)?
            .into_iter()
            .take(limit)
            .map(DisgenetAssociationRecord::from)
            .collect())
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

        let url = self.endpoint("/api/v1/gda/summary");
        let req = self
            .client
            .get(&url)
            .header("Authorization", api_key)
            .header("accept", "application/json")
            .query(&[("disease", disease_id.as_str()), ("page_number", "0")]);

        let resp: DisgenetResponse<DisgenetGdaSummaryRow> = self.get_json(req).await?;
        Ok(validate_response(resp)?
            .into_iter()
            .take(limit)
            .map(DisgenetAssociationRecord::from)
            .collect())
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
        let url = self.endpoint("/api/v1/entity/disease");
        let req = self
            .client
            .get(&url)
            .header("Authorization", api_key)
            .header("accept", "application/json")
            .query(&[("disease_free_text_search_string", query)]);
        let resp: DisgenetResponse<DisgenetDiseaseRow> = self.get_json(req).await?;
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
mod tests {
    use super::*;
    use std::collections::HashMap;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_gene(entrez_id: &str) -> Gene {
        Gene {
            symbol: "TP53".to_string(),
            name: "tumor protein p53".to_string(),
            entrez_id: entrez_id.to_string(),
            ensembl_id: None,
            location: None,
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: None,
            summary: None,
            gene_type: None,
            aliases: Vec::new(),
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
        }
    }

    fn test_disease(name: &str, umls_cui: Option<&str>) -> Disease {
        let mut xrefs = HashMap::new();
        if let Some(cui) = umls_cui {
            xrefs.insert("umls_cui".to_string(), cui.to_string());
        }
        Disease {
            id: "MONDO:0007254".to_string(),
            name: name.to_string(),
            definition: None,
            synonyms: Vec::new(),
            parents: Vec::new(),
            associated_genes: Vec::new(),
            gene_associations: Vec::new(),
            top_genes: Vec::new(),
            top_gene_scores: Vec::new(),
            treatment_landscape: Vec::new(),
            recruiting_trial_count: None,
            pathways: Vec::new(),
            phenotypes: Vec::new(),
            variants: Vec::new(),
            models: Vec::new(),
            prevalence: Vec::new(),
            prevalence_note: None,
            civic: None,
            disgenet: None,
            xrefs,
        }
    }

    fn summary_response() -> serde_json::Value {
        serde_json::json!({
            "status": "OK",
            "httpStatus": 200,
            "paging": {
                "pageSize": 100,
                "totalElements": 2,
                "totalElementsInPage": 2,
                "currentPageNumber": 0
            },
            "warnings": [],
            "payload": [
                {
                    "symbolOfGene": "TP53",
                    "geneNcbiID": 7157,
                    "diseaseName": "Breast Carcinoma",
                    "diseaseUMLSCUI": "C0678222",
                    "score": 0.91,
                    "numPMIDs": 1234,
                    "numCTsupportingAssociation": 4,
                    "ei": 0.72,
                    "el": "Definitive"
                },
                {
                    "symbolOfGene": "TP53",
                    "geneNcbiID": 7157,
                    "diseaseName": "Li-Fraumeni Syndrome",
                    "diseaseUMLSCUI": "C0085390",
                    "score": 0.88,
                    "numPMIDs": 400,
                    "numCTsupportingAssociation": 1,
                    "ei": 0.66,
                    "el": "Strong"
                }
            ]
        })
    }

    #[tokio::test]
    async fn fetch_gene_associations_sends_auth_header_and_gene_ncbi_id() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .and(query_param("gene_ncbi_id", "7157"))
            .and(query_param("page_number", "0"))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(summary_response()))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let rows = client
            .fetch_gene_associations(&test_gene("7157"), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].disease_name, "Breast Carcinoma");
        assert_eq!(rows[0].publication_count, Some(1234));
        assert_eq!(rows[0].clinical_trial_count, Some(4));
        assert_eq!(rows[0].evidence_level.as_deref(), Some("Definitive"));
    }

    #[tokio::test]
    async fn fetch_gene_associations_falls_back_to_gene_symbol() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .and(query_param("gene_symbol", "TP53"))
            .and(query_param("page_number", "0"))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(summary_response()))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let rows = client
            .fetch_gene_associations(&test_gene(""), 1)
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].gene_symbol, "TP53");
    }

    #[tokio::test]
    async fn fetch_disease_associations_uses_umls_cui_when_available() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .and(query_param("disease", "UMLS_C0678222"))
            .and(query_param("page_number", "0"))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(summary_response()))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let rows = client
            .fetch_disease_associations(&test_disease("breast cancer", Some("C0678222")), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].disease_umls_cui, "C0678222");
    }

    #[tokio::test]
    async fn fetch_disease_associations_resolves_free_text_to_umls_cui() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/entity/disease"))
            .and(query_param(
                "disease_free_text_search_string",
                "breast cancer",
            ))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "OK",
                "httpStatus": 200,
                "payload": [
                    {
                        "name": "Breast carcinoma",
                        "diseaseUMLSCUI": "C0678222",
                        "search_rank": 0.82,
                        "synonyms": [
                            {"name": "Breast cancer"}
                        ]
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .and(query_param("disease", "UMLS_C0678222"))
            .and(query_param("page_number", "0"))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(summary_response()))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let rows = client
            .fetch_disease_associations(&test_disease("breast cancer", None), 10)
            .await
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].gene_symbol, "TP53");
    }

    #[tokio::test]
    async fn fetch_disease_associations_returns_source_unavailable_when_resolution_fails() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/entity/disease"))
            .and(query_param(
                "disease_free_text_search_string",
                "completely unknown disease",
            ))
            .and(header("Authorization", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "OK",
                "httpStatus": 200,
                "payload": []
            })))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let err = client
            .fetch_disease_associations(&test_disease("completely unknown disease", None), 10)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    }

    #[tokio::test]
    async fn missing_key_returns_api_key_required_error() {
        let server = MockServer::start().await;
        let client = DisgenetClient::new_for_test(server.uri(), None).unwrap();

        let err = client
            .fetch_gene_associations(&test_gene("7157"), 10)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::ApiKeyRequired { .. }));
    }

    #[tokio::test]
    async fn rate_limit_error_includes_retry_after_seconds() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Content-Type", "application/json")
                    .insert_header("X-Rate-Limit-Retry-After-Seconds", "85564")
                    .set_body_json(serde_json::json!({"message": "Too many requests"})),
            )
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let err = client
            .fetch_gene_associations(&test_gene("7157"), 10)
            .await
            .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("85564"));
        assert!(message.contains("Too many requests"));
    }

    #[tokio::test]
    async fn http_500_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .respond_with(
                ResponseTemplate::new(500)
                    .insert_header("Content-Type", "application/json")
                    .set_body_json(serde_json::json!({"message": "upstream failure"})),
            )
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let err = client
            .fetch_gene_associations(&test_gene("7157"), 10)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::Api { .. }));
    }

    #[tokio::test]
    async fn empty_payload_returns_empty_vec() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/gda/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "OK",
                "httpStatus": 200,
                "payload": []
            })))
            .mount(&server)
            .await;

        let client = DisgenetClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let rows = client
            .fetch_gene_associations(&test_gene("7157"), 10)
            .await
            .unwrap();
        assert!(rows.is_empty());
    }
}
