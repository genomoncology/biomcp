use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const NCI_CTS_BASE: &str = "https://clinicaltrialsapi.cancer.gov/api/v2";
const NCI_CTS_API: &str = "nci_cts";
const NCI_CTS_BASE_ENV: &str = "BIOMCP_NCI_CTS_BASE";
const NCI_API_KEY_ENV: &str = "NCI_API_KEY";

#[derive(Clone)]
pub struct NciCtsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: String,
}

#[derive(Debug, Clone, Default)]
pub struct NciSearchParams {
    pub diseases: Option<String>,
    pub interventions: Option<String>,
    pub sites_org_name: Option<String>,
    pub recruitment_status: Option<String>,
    pub phase: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance: Option<u32>,
    pub biomarkers: Option<String>,
    pub size: usize,
    pub from: usize,
}

#[derive(Debug, Clone, Default)]
pub struct NciOrganizationSearchParams {
    pub name: Option<String>,
    pub organization_type: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub size: usize,
    pub from: usize,
}

#[derive(Debug, Clone, Default)]
pub struct NciInterventionSearchParams {
    pub name: Option<String>,
    pub intervention_type: Option<String>,
    pub category: Option<String>,
    pub codes: Option<String>,
    pub size: usize,
    pub from: usize,
}

#[derive(Debug, Clone, Default)]
pub struct NciBiomarkerSearchParams {
    pub name: Option<String>,
    pub biomarker_type: Option<String>,
    pub eligibility_criterion: Option<String>,
    pub assay_purpose: Option<String>,
    pub codes: Option<String>,
    pub size: usize,
    pub from: usize,
}

#[derive(Debug, Deserialize)]
pub struct NciSearchResponse {
    #[serde(default)]
    pub data: Vec<serde_json::Value>,
    #[serde(default)]
    pub trials: Vec<serde_json::Value>,
    #[serde(default, alias = "total", alias = "total_count", alias = "totalCount")]
    pub total: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct NciListResponse {
    pub rows: Vec<serde_json::Value>,
    pub total: Option<usize>,
}

impl NciSearchResponse {
    pub fn hits(&self) -> &[serde_json::Value] {
        if !self.data.is_empty() {
            &self.data
        } else {
            &self.trials
        }
    }
}

impl NciCtsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let api_key = std::env::var(NCI_API_KEY_ENV)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| BioMcpError::ApiKeyRequired {
                api: NCI_CTS_API.to_string(),
                env_var: NCI_API_KEY_ENV.to_string(),
                docs_url: "https://clinicaltrialsapi.cancer.gov/".to_string(),
            })?;

        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCI_CTS_BASE, NCI_CTS_BASE_ENV),
            api_key,
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String, api_key: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key,
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, NCI_CTS_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: NCI_CTS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: NCI_CTS_API.to_string(),
            source,
        })
    }

    fn extract_list(value: serde_json::Value, keys: &[&str]) -> Vec<serde_json::Value> {
        match value {
            serde_json::Value::Array(rows) => rows,
            serde_json::Value::Object(mut map) => {
                for key in keys {
                    if let Some(serde_json::Value::Array(rows)) = map.remove(*key) {
                        return rows;
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn extract_total(value: &serde_json::Value) -> Option<usize> {
        value
            .as_object()
            .and_then(|obj| {
                obj.get("total")
                    .or_else(|| obj.get("total_count"))
                    .or_else(|| obj.get("totalCount"))
            })
            .and_then(|value| match value {
                serde_json::Value::Number(n) => n.as_u64().map(|v| v as usize),
                serde_json::Value::String(s) => s.parse::<usize>().ok(),
                _ => None,
            })
    }

    pub async fn search(&self, params: &NciSearchParams) -> Result<NciSearchResponse, BioMcpError> {
        let url = self.endpoint("trials");
        let mut req = self.client.get(&url).header("X-API-KEY", &self.api_key);

        if let Some(v) = params
            .diseases
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("diseases", v)]);
        }
        if let Some(v) = params
            .interventions
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("interventions", v)]);
        }
        if let Some(v) = params
            .sites_org_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("sites.org_name", v)]);
        }
        if let Some(v) = params
            .recruitment_status
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("recruitment_status", v)]);
        }
        if let Some(v) = params
            .phase
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("phase", v)]);
        }
        if let Some(lat) = params.latitude {
            req = req.query(&[("latitude", lat)]);
        }
        if let Some(lon) = params.longitude {
            req = req.query(&[("longitude", lon)]);
        }
        if let Some(distance) = params.distance {
            req = req.query(&[("distance", distance)]);
        }
        if let Some(v) = params
            .biomarkers
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("biomarkers", v)]);
        }

        let size = params.size.to_string();
        req = req.query(&[("size", size.as_str())]);
        let from = params.from.to_string();
        req = req.query(&[("from", from.as_str())]);

        self.get_json(req).await
    }

    pub async fn get(&self, nct_id: &str) -> Result<serde_json::Value, BioMcpError> {
        let url = self.endpoint(&format!("trials/{nct_id}"));
        self.get_json(self.client.get(&url).header("X-API-KEY", &self.api_key))
            .await
    }

    #[allow(dead_code)]
    pub async fn search_organizations(
        &self,
        params: &NciOrganizationSearchParams,
    ) -> Result<Vec<serde_json::Value>, BioMcpError> {
        Ok(self.search_organizations_page(params).await?.rows)
    }

    pub async fn search_organizations_page(
        &self,
        params: &NciOrganizationSearchParams,
    ) -> Result<NciListResponse, BioMcpError> {
        let url = self.endpoint("organizations");
        let mut req = self.client.get(&url).header("X-API-KEY", &self.api_key);

        if let Some(v) = params
            .name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("name", v)]);
        }
        if let Some(v) = params
            .organization_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("type", v)]);
        }
        if let Some(v) = params
            .city
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("org_city", v)]);
        }
        if let Some(v) = params
            .state
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("org_state_or_province", v)]);
        }

        let size = params.size.to_string();
        req = req.query(&[("size", size.as_str())]);
        let from = params.from.to_string();
        req = req.query(&[("from", from.as_str())]);

        let payload: serde_json::Value = self.get_json(req).await?;
        Ok(NciListResponse {
            rows: Self::extract_list(payload.clone(), &["data", "organizations"]),
            total: Self::extract_total(&payload),
        })
    }

    #[allow(dead_code)]
    pub async fn search_interventions(
        &self,
        params: &NciInterventionSearchParams,
    ) -> Result<Vec<serde_json::Value>, BioMcpError> {
        Ok(self.search_interventions_page(params).await?.rows)
    }

    pub async fn search_interventions_page(
        &self,
        params: &NciInterventionSearchParams,
    ) -> Result<NciListResponse, BioMcpError> {
        let url = self.endpoint("interventions");
        let mut req = self.client.get(&url).header("X-API-KEY", &self.api_key);

        if let Some(v) = params
            .name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("name", v)]);
        }
        if let Some(v) = params
            .intervention_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("type", v)]);
        }
        if let Some(v) = params
            .category
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("category", v)]);
        }
        if let Some(v) = params
            .codes
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("codes", v)]);
        }

        let size = params.size.to_string();
        req = req.query(&[("size", size.as_str())]);
        let from = params.from.to_string();
        req = req.query(&[("from", from.as_str())]);

        let payload: serde_json::Value = self.get_json(req).await?;
        Ok(NciListResponse {
            rows: Self::extract_list(payload.clone(), &["data", "interventions"]),
            total: Self::extract_total(&payload),
        })
    }

    #[allow(dead_code)]
    pub async fn search_biomarkers(
        &self,
        params: &NciBiomarkerSearchParams,
    ) -> Result<Vec<serde_json::Value>, BioMcpError> {
        Ok(self.search_biomarkers_page(params).await?.rows)
    }

    pub async fn search_biomarkers_page(
        &self,
        params: &NciBiomarkerSearchParams,
    ) -> Result<NciListResponse, BioMcpError> {
        let url = self.endpoint("biomarkers");
        let mut req = self.client.get(&url).header("X-API-KEY", &self.api_key);

        if let Some(v) = params
            .name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("name", v)]);
        }
        if let Some(v) = params
            .biomarker_type
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("type", v)]);
        }
        if let Some(v) = params
            .eligibility_criterion
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("eligibility_criterion", v)]);
        }
        if let Some(v) = params
            .assay_purpose
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("assay_purpose", v)]);
        }
        if let Some(v) = params
            .codes
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            req = req.query(&[("codes", v)]);
        }

        let size = params.size.to_string();
        req = req.query(&[("size", size.as_str())]);
        let from = params.from.to_string();
        req = req.query(&[("from", from.as_str())]);

        let payload: serde_json::Value = self.get_json(req).await?;
        Ok(NciListResponse {
            rows: Self::extract_list(payload.clone(), &["data", "biomarkers"]),
            total: Self::extract_total(&payload),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_includes_api_key_header_and_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/trials"))
            .and(header("X-API-KEY", "test-key"))
            .and(query_param("diseases", "melanoma"))
            .and(query_param("size", "2"))
            .and(query_param("from", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let _ = client
            .search(&NciSearchParams {
                diseases: Some("melanoma".into()),
                interventions: None,
                sites_org_name: None,
                recruitment_status: None,
                phase: None,
                latitude: None,
                longitude: None,
                distance: None,
                biomarkers: None,
                size: 2,
                from: 0,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_surfaces_http_error_context() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/trials"))
            .and(header("X-API-KEY", "test-key"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failure"))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let err = client
            .search(&NciSearchParams {
                diseases: Some("melanoma".into()),
                interventions: None,
                sites_org_name: None,
                recruitment_status: None,
                phase: None,
                latitude: None,
                longitude: None,
                distance: None,
                biomarkers: None,
                size: 2,
                from: 0,
            })
            .await
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("nci_cts"));
        assert!(msg.contains("500"));
    }

    #[tokio::test]
    async fn search_includes_sites_org_name_param() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/trials"))
            .and(header("X-API-KEY", "test-key"))
            .and(query_param("diseases", "melanoma"))
            .and(query_param("sites.org_name", "MD Anderson"))
            .and(query_param("size", "2"))
            .and(query_param("from", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let _ = client
            .search(&NciSearchParams {
                diseases: Some("melanoma".into()),
                interventions: None,
                sites_org_name: Some("MD Anderson".into()),
                recruitment_status: None,
                phase: None,
                latitude: None,
                longitude: None,
                distance: None,
                biomarkers: None,
                size: 2,
                from: 0,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_organizations_includes_expected_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/organizations"))
            .and(header("X-API-KEY", "test-key"))
            .and(query_param("name", "MD Anderson"))
            .and(query_param("type", "academic"))
            .and(query_param("org_city", "Houston"))
            .and(query_param("org_state_or_province", "TX"))
            .and(query_param("size", "7"))
            .and(query_param("from", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let _ = client
            .search_organizations(&NciOrganizationSearchParams {
                name: Some("MD Anderson".into()),
                organization_type: Some("academic".into()),
                city: Some("Houston".into()),
                state: Some("TX".into()),
                size: 7,
                from: 0,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_interventions_includes_expected_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/interventions"))
            .and(header("X-API-KEY", "test-key"))
            .and(query_param("name", "Pembrolizumab"))
            .and(query_param("type", "drug"))
            .and(query_param("category", "agent"))
            .and(query_param("codes", "C82416"))
            .and(query_param("size", "3"))
            .and(query_param("from", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "interventions": []
            })))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let _ = client
            .search_interventions(&NciInterventionSearchParams {
                name: Some("Pembrolizumab".into()),
                intervention_type: Some("drug".into()),
                category: Some("agent".into()),
                codes: Some("C82416".into()),
                size: 3,
                from: 0,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn search_biomarkers_includes_expected_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/biomarkers"))
            .and(header("X-API-KEY", "test-key"))
            .and(query_param("name", "PD-L1"))
            .and(query_param("type", "reference_gene"))
            .and(query_param("eligibility_criterion", "positive"))
            .and(query_param("assay_purpose", "eligibility"))
            .and(query_param("codes", "C104743"))
            .and(query_param("size", "4"))
            .and(query_param("from", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "biomarkers": []
            })))
            .mount(&server)
            .await;

        let client = NciCtsClient::new_for_test(server.uri(), "test-key".into()).unwrap();
        let _ = client
            .search_biomarkers(&NciBiomarkerSearchParams {
                name: Some("PD-L1".into()),
                biomarker_type: Some("reference_gene".into()),
                eligibility_criterion: Some("positive".into()),
                assay_purpose: Some("eligibility".into()),
                codes: Some("C104743".into()),
                size: 4,
                from: 0,
            })
            .await
            .unwrap();
    }
}
