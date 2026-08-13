use std::borrow::Cow;
use std::time::Duration;

use reqwest::{StatusCode, Url};
use serde_json::Value;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{
    RequestBuilderSourceContextExt, RequestPlan, ensure_json_content_type,
    read_limited_source_body_with_limit, request_from_plan,
};

pub(crate) const CSPEC_BASE: &str = "https://cspec.clinicalgenome.org";
const CSPEC_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const CSPEC_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MANIFEST_LIMIT: usize = 256 * 1024;
const DOCUMENT_LIMIT: usize = 4 * 1024 * 1024;

pub(crate) struct CspecClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    fixture_origin: Option<Url>,
}

#[derive(Clone, Copy)]
struct CspecTimeouts {
    connect: Duration,
    request: Duration,
}

impl Default for CspecTimeouts {
    fn default() -> Self {
        Self {
            connect: CSPEC_CONNECT_TIMEOUT,
            request: CSPEC_REQUEST_TIMEOUT,
        }
    }
}

impl CspecClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::cspec()?;
        let client = reqwest::Client::builder()
            .connect_timeout(CspecTimeouts::default().connect)
            .timeout(CspecTimeouts::default().request)
            .dns_resolver(policy.dns_resolver())
            .redirect(policy.redirect_policy())
            .build()
            .map_err(BioMcpError::from)?;
        Ok(Self {
            client: reqwest_middleware::ClientBuilder::new(client).build(),
            base: Cow::Borrowed(CSPEC_BASE),
            fixture_origin: crate::sources::provider_url_policy::cspec_fixture_origin()?,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_test_client(client: reqwest_middleware::ClientWithMiddleware) -> Self {
        Self {
            client,
            base: Cow::Borrowed(CSPEC_BASE),
            fixture_origin: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_client_at(
        client: reqwest_middleware::ClientWithMiddleware,
        base: String,
    ) -> Self {
        Self {
            client,
            base: Cow::Owned(base),
            fixture_origin: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_timeouts_at(
        base: String,
        connect: Duration,
        request: Duration,
    ) -> Result<Self, BioMcpError> {
        let client = reqwest::Client::builder()
            .connect_timeout(connect)
            .timeout(request)
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(BioMcpError::HttpClientInit)?;
        Ok(Self {
            client: reqwest_middleware::ClientBuilder::new(client).build(),
            base: Cow::Owned(base),
            fixture_origin: None,
        })
    }

    pub(crate) fn manifest_plan(gene: &str) -> RequestPlan {
        RequestPlan::get(format!(
            "cspec/Gene/id/{gene}/SequenceVariantInterpretation/version"
        ))
        .query("detail", "low")
    }

    pub(crate) fn document_plan(iri: &Url) -> RequestPlan {
        RequestPlan::get(iri.path())
    }

    pub(crate) async fn manifest(&self, gene: &str) -> Result<Value, BioMcpError> {
        let base = self.fetch_base(&Url::parse(self.base.as_ref()).expect("CSpec base is valid"));
        self.get(Self::manifest_plan(gene), base.as_str(), MANIFEST_LIMIT)
            .await
            .map(|(_, body)| decode_envelope(&body))?
    }

    pub(crate) async fn document(&self, iri: &Url) -> Result<Vec<u8>, BioMcpError> {
        crate::sources::provider_url_policy::ProviderUrlPolicy::cspec()?.validate_url(iri)?;
        let base = self.fetch_base(iri);
        let (_, bytes) = self
            .get(Self::document_plan(iri), base.as_str(), DOCUMENT_LIMIT)
            .await?;
        decode_envelope(&bytes)?;
        Ok(bytes)
    }

    fn fetch_base(&self, iri: &Url) -> Url {
        let mut base = iri.clone();
        base.set_path("");
        base.set_query(None);
        base.set_fragment(None);
        if let Some(origin) = &self.fixture_origin {
            base.set_scheme(origin.scheme())
                .expect("fixture origin has scheme");
            base.set_host(origin.host_str())
                .expect("fixture origin has host");
            base.set_port(origin.port())
                .expect("fixture origin has valid port");
        }
        #[cfg(test)]
        if self.fixture_origin.is_none() && self.base != CSPEC_BASE {
            return Url::parse(self.base.as_ref()).expect("test CSpec base is valid");
        }
        base
    }

    async fn get(
        &self,
        plan: RequestPlan,
        base: &str,
        limit: usize,
    ) -> Result<(StatusCode, Vec<u8>), BioMcpError> {
        let response = request_from_plan(&self.client, base, &plan)
            .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CSPEC))
            .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let context = SourceContext::narrow(SourceProvider::CLINGEN_CSPEC);
        let bytes = read_limited_source_body_with_limit(response, context, limit).await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: "ClinGen CSpec".into(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }
        ensure_json_content_type(context, content_type.as_ref(), &bytes)?;
        Ok((status, bytes))
    }
}

fn decode_envelope(bytes: &[u8]) -> Result<Value, BioMcpError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: "ClinGen CSpec".into(),
        source,
    })?;
    if value.pointer("/status/code").and_then(Value::as_i64) != Some(200)
        || value.get("metadata").is_none()
        || value.get("data").is_none()
    {
        return Err(BioMcpError::Api {
            api: "ClinGen CSpec".into(),
            message: "invalid CSpec response envelope".into(),
        });
    }
    Ok(value)
}

#[cfg(test)]
mod tests;
