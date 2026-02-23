use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

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

    #[cfg(test)]
    fn new_for_test(base: String, api_key: Option<String>) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key: api_key
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
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

        if status.as_u16() == 404 {
            return Ok(None);
        }

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: OPENFDA_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|source| BioMcpError::ApiJson {
                api: OPENFDA_API.to_string(),
                source,
            })
    }

    pub async fn faers_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<FaersEventResult>>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search adverse-event -d pembrolizumab".into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let url = self.endpoint("drug/event.json");
        let skip = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("search", query),
            ("limit", &limit.to_string()),
            ("skip", skip.as_str()),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }
        self.get_json_optional(req).await
    }

    pub async fn faers_count(
        &self,
        query: &str,
        count_field: &str,
        limit: usize,
    ) -> Result<Option<OpenFdaCountResponse>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search adverse-event -d pembrolizumab --count patient.reaction.reactionmeddrapt".into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        let count_field = count_field.trim();
        if count_field.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "--count requires a field name".into(),
            ));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let mut count_fields = vec![count_field.to_string()];
        if !count_field.ends_with(".exact") {
            count_fields.push(format!("{count_field}.exact"));
        }

        let url = self.endpoint("drug/event.json");
        for count_field in count_fields {
            let mut req = self.client.get(&url).query(&[
                ("search", query),
                ("count", count_field.as_str()),
                ("limit", &limit.to_string()),
            ]);
            if let Some(key) = self.api_key.as_deref() {
                req = req.query(&[("api_key", key)]);
            }
            let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
                .send()
                .await?;
            let status = resp.status();
            let bytes = crate::sources::read_limited_body(resp, OPENFDA_API).await?;

            if status.as_u16() == 404 {
                return Ok(None);
            }
            if !status.is_success() {
                let excerpt = crate::sources::body_excerpt(&bytes);
                return Err(BioMcpError::Api {
                    api: OPENFDA_API.to_string(),
                    message: format!("HTTP {status}: {excerpt}"),
                });
            }

            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                    api: OPENFDA_API.to_string(),
                    source,
                })?;

            if let Some(error) = value.get("error").and_then(serde_json::Value::as_object) {
                let code = error
                    .get("code")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                let details = error
                    .get("details")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                if code.eq_ignore_ascii_case("SERVER_ERROR")
                    && details.to_ascii_lowercase().contains("keyword field")
                    && !count_field.ends_with(".exact")
                {
                    continue;
                }
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

        let url = self.endpoint("drug/label.json");
        let mut req = self.client.get(&url).query(&[
            ("search", q.as_str()),
            ("limit", "5"),
            ("sort", "effective_time:desc"),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }

        self.get_json_optional(req).await
    }

    pub async fn drugsfda_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DrugsFdaResult>>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp get drug dabrafenib approvals".into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let url = self.endpoint("drug/drugsfda.json");
        let skip = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("search", query),
            ("limit", &limit.to_string()),
            ("skip", skip.as_str()),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }
        self.get_json_optional(req).await
    }

    pub async fn enforcement_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<EnforcementResult>>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search adverse-event --type recall".into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let url = self.endpoint("drug/enforcement.json");
        let skip = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("search", query),
            ("limit", &limit.to_string()),
            ("skip", skip.as_str()),
            ("sort", "recall_initiation_date:desc"),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }
        self.get_json_optional(req).await
    }

    pub async fn shortage_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DrugShortageResult>>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp get drug carboplatin shortage".into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let url = self.endpoint("drug/shortages.json");
        let skip = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("search", query),
            ("limit", &limit.to_string()),
            ("skip", skip.as_str()),
            ("sort", "update_date:desc"),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }
        self.get_json_optional(req).await
    }

    pub async fn device_event_search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Option<OpenFdaResponse<DeviceEventResult>>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search adverse-event --type device --device \"insulin pump\""
                    .into(),
            ));
        }
        if query.len() > 1024 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "--limit must be between 1 and 50".into(),
            ));
        }

        let url = self.endpoint("device/event.json");
        let skip = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("search", query),
            ("limit", &limit.to_string()),
            ("skip", skip.as_str()),
            ("sort", "date_received:desc"),
        ]);
        if let Some(key) = self.api_key.as_deref() {
            req = req.query(&[("api_key", key)]);
        }
        self.get_json_optional(req).await
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaResponse<T> {
    #[allow(dead_code)]
    pub meta: OpenFdaMeta,
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaMeta {
    #[allow(dead_code)]
    pub results: OpenFdaMetaResults,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaMetaResults {
    #[allow(dead_code)]
    pub skip: usize,
    #[allow(dead_code)]
    pub limit: usize,
    #[allow(dead_code)]
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct OpenFdaCountResponse {
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
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn escape_query_value_escapes_lucene_special_chars() {
        assert_eq!(
            OpenFdaClient::escape_query_value(r#"PD-1 "checkpoint"\test"#),
            r#"PD\-1 \"checkpoint\"\\test"#
        );
    }

    #[tokio::test]
    async fn faers_search_validates_limit_bounds() {
        let client = OpenFdaClient::new_for_test("http://127.0.0.1".into(), None).unwrap();
        let err = client.faers_search("drug:x", 0, 0).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));

        let err = client.faers_search("drug:x", 51, 0).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn label_search_includes_api_key_when_configured() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/drug/label.json"))
            .and(query_param("limit", "5"))
            .and(query_param("sort", "effective_time:desc"))
            .and(query_param("api_key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 1, "total": 1}},
                "results": [{"id": "x"}]
            })))
            .mount(&server)
            .await;

        let client = OpenFdaClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let resp = client.label_search("pembrolizumab").await.unwrap();
        assert!(resp.is_some());
    }

    #[tokio::test]
    async fn drugsfda_search_validates_limit_bounds() {
        let client = OpenFdaClient::new_for_test("http://127.0.0.1".into(), None).unwrap();
        let err = client
            .drugsfda_search("openfda.brand_name:test", 0, 0)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));

        let err = client
            .drugsfda_search("openfda.brand_name:test", 51, 0)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn drugsfda_search_hits_expected_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/drug/drugsfda.json"))
            .and(query_param("limit", "3"))
            .and(query_param("skip", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": {"results": {"skip": 0, "limit": 3, "total": 1}},
                "results": [{
                    "application_number": "NDA123456",
                    "sponsor_name": "Example Pharma"
                }]
            })))
            .mount(&server)
            .await;

        let client = OpenFdaClient::new_for_test(server.uri(), None).unwrap();
        let resp = client
            .drugsfda_search("openfda.brand_name:test", 3, 0)
            .await
            .unwrap();
        assert!(resp.is_some());
    }
}
