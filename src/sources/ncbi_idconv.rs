use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

// NCBI PMC ID Converter API
// Docs: https://pmc.ncbi.nlm.nih.gov/tools/id-converter-api/
const NCBI_IDCONV_BASE: &str = "https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles";
const NCBI_IDCONV_API: &str = "ncbi-idconv";
const NCBI_IDCONV_BASE_ENV: &str = "BIOMCP_NCBI_IDCONV_BASE";

#[derive(Clone)]
pub struct NcbiIdConverterClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl NcbiIdConverterClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCBI_IDCONV_BASE, NCBI_IDCONV_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
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

    fn endpoint(&self) -> String {
        self.base.as_ref().trim_end_matches('/').to_string()
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send()
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, NCBI_IDCONV_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: NCBI_IDCONV_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: NCBI_IDCONV_API.to_string(),
            source,
        })
    }

    async fn lookup(&self, idtype: &str, id: &str) -> Result<NcbiIdConvResponse, BioMcpError> {
        let url = self.endpoint();
        let req =
            self.client
                .get(&url)
                .query(&[("format", "json"), ("idtype", idtype), ("ids", id)]);
        let req = crate::sources::append_ncbi_api_key(req, self.api_key.as_deref());
        self.get_json(req).await
    }

    pub async fn pmid_to_pmcid(&self, pmid: &str) -> Result<Option<String>, BioMcpError> {
        let pmid = pmid.trim();
        if pmid.is_empty() {
            return Ok(None);
        }
        if pmid.len() > 32 {
            return Err(BioMcpError::InvalidArgument("PMID is too long.".into()));
        }
        if !pmid.chars().all(|c| c.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PMID must contain only digits.".into(),
            ));
        }

        let resp = self.lookup("pmid", pmid).await?;
        Ok(resp
            .records
            .into_iter()
            .next()
            .and_then(|r| r.pmcid)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()))
    }

    pub async fn doi_to_pmcid(&self, doi: &str) -> Result<Option<String>, BioMcpError> {
        let doi = doi.trim();
        if doi.is_empty() {
            return Ok(None);
        }
        if doi.len() > 256 {
            return Err(BioMcpError::InvalidArgument("DOI is too long.".into()));
        }
        if !doi.starts_with("10.") || !doi.contains('/') {
            return Err(BioMcpError::InvalidArgument(
                "DOI must start with 10. and include a slash.".into(),
            ));
        }

        let resp = self.lookup("doi", doi).await?;
        Ok(resp
            .records
            .into_iter()
            .next()
            .and_then(|r| r.pmcid)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()))
    }
}

#[derive(Debug, Deserialize)]
pub struct NcbiIdConvResponse {
    #[allow(dead_code)]
    pub status: Option<String>,
    #[serde(default)]
    pub records: Vec<NcbiIdConvRecord>,
}

#[derive(Debug, Deserialize)]
pub struct NcbiIdConvRecord {
    pub pmcid: Option<String>,
    #[allow(dead_code)]
    pub pmid: Option<u64>,
    #[allow(dead_code)]
    pub doi: Option<String>,
    #[allow(dead_code)]
    pub status: Option<String>,
    #[allow(dead_code)]
    pub errmsg: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "requested-id")]
    pub requested_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn pmid_to_pmcid_validates_numeric_input() {
        let client = NcbiIdConverterClient::new_for_test("http://127.0.0.1".into(), None).unwrap();
        let err = client.pmid_to_pmcid("abc").await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn pmid_to_pmcid_parses_lookup_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("format", "json"))
            .and(query_param("idtype", "pmid"))
            .and(query_param("ids", "22663011"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [{"pmcid": "PMC123456"}]
            })))
            .mount(&server)
            .await;

        let client = NcbiIdConverterClient::new_for_test(server.uri(), None).unwrap();
        let pmcid = client.pmid_to_pmcid("22663011").await.unwrap();
        assert_eq!(pmcid.as_deref(), Some("PMC123456"));
    }

    #[tokio::test]
    async fn pmid_to_pmcid_includes_api_key_when_configured() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .and(query_param("format", "json"))
            .and(query_param("idtype", "pmid"))
            .and(query_param("ids", "22663011"))
            .and(query_param("api_key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "records": [{"pmcid": "PMC123456"}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client =
            NcbiIdConverterClient::new_for_test(server.uri(), Some("test-key".into())).unwrap();
        let pmcid = client.pmid_to_pmcid("22663011").await.unwrap();
        assert_eq!(pmcid.as_deref(), Some("PMC123456"));
    }
}
