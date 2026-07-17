use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const OPENFDA_BASE: &str = "https://api.fda.gov";
const OPENFDA_API: &str = "openfda";
const OPENFDA_BASE_ENV: &str = "BIOMCP_OPENFDA_BASE";

pub struct OpenFdaClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl OpenFdaClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(OPENFDA_BASE, OPENFDA_BASE_ENV),
            api_key: std::env::var("OPENFDA_API_KEY")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    pub(crate) fn faers_search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp search adverse-event -d pembrolizumab",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("drug/event.json")
                .query("search", query)
                .query("limit", limit.to_string())
                .query("skip", offset.to_string()),
            api_key,
        ))
    }

    pub(crate) fn faers_count_plans(
        query: &str,
        count_field: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<Vec<(String, RequestPlan)>, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp search adverse-event -d pembrolizumab --count patient.reaction.reactionmeddrapt",
        )?;
        let count_field = count_field.trim();
        if count_field.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "--count requires a field name".into(),
            ));
        }
        validate_limit(limit)?;

        let mut count_fields = vec![count_field.to_string()];
        if !count_field.ends_with(".exact") {
            count_fields.push(format!("{count_field}.exact"));
        }

        Ok(count_fields
            .into_iter()
            .map(|field| {
                let plan = with_api_key(
                    RequestPlan::get("drug/event.json")
                        .query("search", query.clone())
                        .query("count", field.clone())
                        .query("limit", limit.to_string()),
                    api_key,
                );
                (field, plan)
            })
            .collect())
    }

    pub(crate) fn label_search_plan(
        drug_name: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let drug_name = drug_name.trim();
        if drug_name.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Drug name is required. Example: biomcp get drug vemurafenib label".into(),
            ));
        }
        if drug_name.len() > 256 {
            return Err(BioMcpError::InvalidArgument(
                "Drug name is too long.".into(),
            ));
        }

        let escaped = Self::escape_query_value(drug_name);
        let q = format!("openfda.generic_name:\"{escaped}\" OR openfda.brand_name:\"{escaped}\"");
        Ok(with_api_key(
            RequestPlan::get("drug/label.json")
                .query("search", q)
                .query("limit", "5")
                .query("sort", "effective_time:desc"),
            api_key,
        ))
    }

    pub(crate) fn drugsfda_search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp get drug dabrafenib approvals",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("drug/drugsfda.json")
                .query("search", query)
                .query("limit", limit.to_string())
                .query("skip", offset.to_string()),
            api_key,
        ))
    }

    pub(crate) fn device_510k_search_plan(
        query: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp get diagnostic \"FoundationOne CDx\" regulatory",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("device/510k.json")
                .query("search", query)
                .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub(crate) fn device_pma_search_plan(
        query: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp get diagnostic \"FoundationOne CDx\" regulatory",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("device/pma.json")
                .query("search", query)
                .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub(crate) fn enforcement_search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp search adverse-event --type recall",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("drug/enforcement.json")
                .query("search", query)
                .query("limit", limit.to_string())
                .query("skip", offset.to_string())
                .query("sort", "recall_initiation_date:desc"),
            api_key,
        ))
    }

    pub(crate) fn shortage_search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp get drug carboplatin shortage",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("drug/shortages.json")
                .query("search", query)
                .query("limit", limit.to_string())
                .query("skip", offset.to_string())
                .query("sort", "update_date:desc"),
            api_key,
        ))
    }

    pub(crate) fn device_event_search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = validate_query(
            query,
            "Query is required. Example: biomcp search adverse-event --type device --device \"insulin pump\"",
        )?;
        validate_limit(limit)?;
        Ok(with_api_key(
            RequestPlan::get("device/event.json")
                .query("search", query)
                .query("limit", limit.to_string())
                .query("skip", offset.to_string())
                .query("sort", "date_received:desc"),
            api_key,
        ))
    }

    pub(crate) fn decode_json_optional<T: DeserializeOwned>(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Option<T>, BioMcpError> {
        if status.as_u16() == 404 {
            return Ok(None);
        }
        crate::sources::decode_json(OPENFDA_API, status, None, bytes, false).map(Some)
    }

    fn count_value_requests_exact_retry(value: &serde_json::Value, count_field: &str) -> bool {
        let Some(error) = value.get("error").and_then(serde_json::Value::as_object) else {
            return false;
        };
        let code = error
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let details = error
            .get("details")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        code.eq_ignore_ascii_case("SERVER_ERROR")
            && details.to_ascii_lowercase().contains("keyword field")
            && !count_field.ends_with(".exact")
    }

    async fn get_json_optional<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<Option<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, OPENFDA_API).await?;
        Self::decode_json_optional(status, &bytes)
    }

    pub async fn faers_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<FaersEventResult>>, BioMcpError> {
        let plan = Self::faers_search_plan(query, limit, offset, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn faers_count(
        &self,
        query: &str,
        count_field: &str,
        limit: usize,
    ) -> Result<Option<OpenFdaCountResponse>, BioMcpError> {
        let plans = Self::faers_count_plans(query, count_field, limit, self.api_key.as_deref())?;
        for (count_field, plan) in plans {
            let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
            let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
                .send()
                .await?;
            let status = resp.status();
            let bytes = crate::sources::read_limited_body(resp, OPENFDA_API).await?;

            let Some(value) = Self::decode_json_optional::<serde_json::Value>(status, &bytes)?
            else {
                return Ok(None);
            };

            if Self::count_value_requests_exact_retry(&value, &count_field) {
                continue;
            }
            if value.get("error").is_some() {
                return Ok(None);
            }

            return serde_json::from_value::<OpenFdaCountResponse>(value)
                .map(Some)
                .map_err(|source| BioMcpError::ApiJson {
                    api: OPENFDA_API.to_string(),
                    source,
                });
        }
        Ok(None)
    }

    pub async fn label_search(
        &self,
        drug_name: &str,
    ) -> Result<Option<serde_json::Value>, BioMcpError> {
        let plan = Self::label_search_plan(drug_name, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn drugsfda_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DrugsFdaResult>>, BioMcpError> {
        let plan = Self::drugsfda_search_plan(query, limit, offset, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn device_510k_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Option<OpenFdaResponse<Fda510kResult>>, BioMcpError> {
        let plan = Self::device_510k_search_plan(query, limit, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn device_pma_search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Option<OpenFdaResponse<FdaPmaResult>>, BioMcpError> {
        let plan = Self::device_pma_search_plan(query, limit, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn enforcement_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<EnforcementResult>>, BioMcpError> {
        let plan = Self::enforcement_search_plan(query, limit, offset, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn shortage_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DrugShortageResult>>, BioMcpError> {
        let plan = Self::shortage_search_plan(query, limit, offset, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }

    pub async fn device_event_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DeviceEventResult>>, BioMcpError> {
        let plan = Self::device_event_search_plan(query, limit, offset, self.api_key.as_deref())?;
        self.get_json_optional(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }
}

fn validate_query(query: &str, required_message: &str) -> Result<String, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(required_message.into()));
    }
    if query.len() > 1024 {
        return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
    }
    Ok(query.to_string())
}

fn validate_limit(limit: usize) -> Result<(), BioMcpError> {
    if limit == 0 || limit > 50 {
        return Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        ));
    }
    Ok(())
}

fn with_api_key(mut plan: RequestPlan, api_key: Option<&str>) -> RequestPlan {
    if let Some(key) = api_key.map(str::trim).filter(|key| !key.is_empty()) {
        plan = plan.query("api_key", key);
    }
    plan
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaResponse<T> {
    // dead-code reason: openfda::meta preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub meta: OpenFdaMeta,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaMeta {
    // dead-code reason: openfda::results preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub results: OpenFdaMetaResults,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaMetaResults {
    // dead-code reason: openfda::skip preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub skip: usize,
    // dead-code reason: openfda::limit preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub limit: usize,
    // dead-code reason: openfda::total preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaCountResponse {
    // dead-code reason: openfda::meta preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub meta: serde_json::Value,
    #[serde(default)]
    pub results: Vec<OpenFdaCountBucket>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenFdaCountBucket {
    pub term: String,
    pub count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersEventResult {
    pub safetyreportid: String,
    #[serde(default)]
    pub serious: Option<String>,
    #[serde(default)]
    pub receivedate: Option<String>,
    #[serde(default)]
    pub seriousnessdeath: Option<String>,
    #[serde(default)]
    pub seriousnesslifethreatening: Option<String>,
    #[serde(default)]
    pub seriousnesshospitalization: Option<String>,
    #[serde(default)]
    pub seriousnessdisabling: Option<String>,
    #[serde(default)]
    pub seriousnesscongenitalanomali: Option<String>,
    #[serde(default)]
    pub seriousnessother: Option<String>,
    #[serde(default)]
    pub patient: Option<FaersPatient>,
    #[serde(default)]
    pub primarysource: Option<FaersPrimarySource>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersPatient {
    #[serde(default)]
    pub patientonsetage: Option<String>,
    #[serde(default)]
    pub patientonsetageunit: Option<String>,
    #[serde(default)]
    pub patientsex: Option<String>,
    #[serde(default)]
    pub patientweight: Option<String>,
    #[serde(default)]
    pub reaction: Vec<FaersReaction>,
    #[serde(default)]
    pub drug: Vec<FaersDrug>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersReaction {
    #[serde(default)]
    pub reactionmeddrapt: Option<String>,
    #[serde(default)]
    // dead-code reason: openfda::reactionoutcome preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub reactionoutcome: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersDrug {
    #[serde(default)]
    pub medicinalproduct: Option<String>,
    #[serde(default)]
    pub drugcharacterization: Option<String>,
    #[serde(default)]
    pub drugindication: Option<String>,
    #[serde(default)]
    pub openfda: Option<FaersOpenFdaDrug>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersOpenFdaDrug {
    #[serde(default)]
    pub generic_name: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaersPrimarySource {
    #[serde(default)]
    pub qualification: Option<String>,
    #[serde(default)]
    pub reportercountry: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnforcementResult {
    pub recall_number: String,
    pub classification: String,
    pub product_description: String,
    pub reason_for_recall: String,
    pub status: String,
    #[serde(default)]
    pub distribution_pattern: Option<String>,
    #[serde(default)]
    pub recall_initiation_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugShortageResult {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub availability: Option<String>,
    #[serde(default)]
    pub company_name: Option<String>,
    #[serde(default)]
    pub generic_name: Option<String>,
    #[serde(default)]
    pub related_info: Option<String>,
    #[serde(default)]
    pub update_date: Option<String>,
    #[serde(default)]
    pub initial_posting_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugsFdaResult {
    #[serde(default)]
    pub application_number: Option<String>,
    #[serde(default)]
    pub sponsor_name: Option<String>,
    #[serde(default)]
    pub products: Vec<DrugsFdaProduct>,
    #[serde(default)]
    pub submissions: Vec<DrugsFdaSubmission>,
    #[serde(default)]
    pub openfda: Option<DrugsFdaOpenFda>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugsFdaProduct {
    #[serde(default)]
    pub brand_name: Option<String>,
    #[serde(default)]
    pub dosage_form: Option<String>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub marketing_status: Option<String>,
    #[serde(default)]
    pub active_ingredients: Vec<DrugsFdaActiveIngredient>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugsFdaActiveIngredient {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub strength: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugsFdaSubmission {
    #[serde(default)]
    pub submission_type: Option<String>,
    #[serde(default)]
    pub submission_number: Option<String>,
    #[serde(default)]
    pub submission_status: Option<String>,
    #[serde(default)]
    pub submission_status_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrugsFdaOpenFda {
    #[serde(default)]
    pub brand_name: Vec<String>,
    #[serde(default)]
    pub generic_name: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fda510kResult {
    #[serde(default)]
    pub k_number: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub applicant: Option<String>,
    #[serde(default)]
    pub decision_date: Option<String>,
    #[serde(default)]
    pub decision_description: Option<String>,
    #[serde(default)]
    pub advisory_committee_description: Option<String>,
    #[serde(default)]
    pub product_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FdaPmaResult {
    #[serde(default)]
    pub pma_number: Option<String>,
    #[serde(default)]
    pub trade_name: Option<String>,
    #[serde(default)]
    pub generic_name: Option<String>,
    #[serde(default)]
    pub applicant: Option<String>,
    #[serde(default)]
    pub decision_date: Option<String>,
    #[serde(default)]
    pub decision_description: Option<String>,
    #[serde(default)]
    pub advisory_committee_description: Option<String>,
    #[serde(default)]
    pub product_code: Option<String>,
    #[serde(default)]
    pub supplement_number: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceEventResult {
    pub mdr_report_key: String,
    #[serde(default)]
    pub report_number: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub date_of_event: Option<String>,
    #[serde(default)]
    pub date_received: Option<String>,
    #[serde(default)]
    pub manufacturer_name: Option<String>,
    #[serde(default)]
    pub device: Vec<DeviceEventDevice>,
    #[serde(default)]
    pub mdr_text: Vec<DeviceEventText>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceEventDevice {
    #[serde(default)]
    pub brand_name: Option<String>,
    #[serde(default)]
    pub generic_name: Option<String>,
    #[serde(default)]
    pub manufacturer_d_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceEventText {
    #[serde(default)]
    pub text_type_code: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[cfg(test)]
mod tests;
