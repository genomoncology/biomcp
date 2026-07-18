use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

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

#[derive(Debug, Clone)]
pub enum NciDiseaseFilter {
    Keyword(String),
    ConceptId(String),
}

#[derive(Debug, Clone)]
pub enum NciStatusFilter {
    CurrentTrialStatus(String),
    SiteRecruitmentStatus(String),
}

#[derive(Debug, Clone)]
pub struct NciGeoFilter {
    pub lat: f64,
    pub lon: f64,
    pub distance_miles: u32,
}

#[derive(Debug, Clone, Default)]
pub struct NciSearchParams {
    pub disease: Option<NciDiseaseFilter>,
    pub interventions: Option<String>,
    pub sites_org_name: Option<String>,
    pub status: Option<NciStatusFilter>,
    pub phases: Vec<String>,
    pub geo: Option<NciGeoFilter>,
    pub biomarkers: Option<String>,
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

impl NciSearchResponse {
    pub fn hits(&self) -> &[serde_json::Value] {
        if !self.data.is_empty() {
            &self.data
        } else {
            &self.trials
        }
    }
}

fn trimmed_non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, true)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NCI_CTS,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NCI_CTS),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::NCI_CTS),
            status,
            None,
            &bytes,
            false,
        )
    }

    /// Build the outbound trials-search request (pure — Tier-2 testable, never sent).
    pub(crate) fn search_plan(api_key: &str, params: &NciSearchParams) -> RequestPlan {
        let mut plan = RequestPlan::get("trials").header("X-API-KEY", api_key);

        if let Some(disease) = &params.disease {
            match disease {
                NciDiseaseFilter::Keyword(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("keyword", v);
                    }
                }
                NciDiseaseFilter::ConceptId(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("diseases.nci_thesaurus_concept_id", v);
                    }
                }
            }
        }
        if let Some(v) = params
            .interventions
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("interventions", v);
        }
        if let Some(v) = params
            .sites_org_name
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("sites.org_name", v);
        }
        if let Some(status) = &params.status {
            match status {
                NciStatusFilter::CurrentTrialStatus(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("current_trial_status", v);
                    }
                }
                NciStatusFilter::SiteRecruitmentStatus(v) => {
                    if let Some(v) = trimmed_non_empty(v.as_str()) {
                        plan = plan.query("sites.recruitment_status", v);
                    }
                }
            }
        }
        for phase in &params.phases {
            let phase = phase.trim();
            if phase.is_empty() {
                continue;
            }
            plan = plan.query("phase", phase);
        }
        if let Some(geo) = &params.geo {
            plan = plan.query("sites.org_coordinates_lat", geo.lat.to_string());
            plan = plan.query("sites.org_coordinates_lon", geo.lon.to_string());
            plan = plan.query(
                "sites.org_coordinates_dist",
                format!("{}mi", geo.distance_miles),
            );
        }
        if let Some(v) = params
            .biomarkers
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            plan = plan.query("biomarkers", v);
        }

        plan = plan.query("size", params.size.to_string());
        plan = plan.query("from", params.from.to_string());
        plan
    }

    pub async fn search(&self, params: &NciSearchParams) -> Result<NciSearchResponse, BioMcpError> {
        let plan = Self::search_plan(&self.api_key, params);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    /// Build the outbound single-trial request (pure — Tier-2 testable).
    pub(crate) fn get_plan(api_key: &str, nct_id: &str) -> RequestPlan {
        RequestPlan::get(format!("trials/{nct_id}")).header("X-API-KEY", api_key)
    }

    pub async fn get(&self, nct_id: &str) -> Result<serde_json::Value, BioMcpError> {
        let plan = Self::get_plan(&self.api_key, nct_id);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }
}

#[cfg(test)]
mod tests;
