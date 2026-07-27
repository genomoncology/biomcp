use reqwest::Url;
use serde_json::Value;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, RequestPlan, request_from_plan};

const LDH_BASE: &str = "https://ldh.genome.network";
const LDH_BASE_ENV: &str = "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN";
pub(crate) const MEDIUM_BODY_LIMIT: usize = 256 * 1024;
pub(crate) const DIRECT_BODY_LIMIT: usize = 512 * 1024;

pub(crate) struct ClinGenLdhClient {
    client: reqwest_middleware::ClientWithMiddleware,
    fixture_origin: Option<Url>,
}

impl ClinGenLdhClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::clingen_ldh()?;
        let fixture_origin = std::env::var(LDH_BASE_ENV)
            .ok()
            .and_then(|value| Url::parse(&value).ok());
        Ok(Self {
            client: crate::sources::provider_url_client(&policy)?,
            fixture_origin,
        })
    }

    pub(crate) async fn medium(&self, caid: &str) -> Result<Value, BioMcpError> {
        let response = crate::sources::apply_cache_mode(request_from_plan(
            &self.client,
            self.fixture_origin
                .as_ref()
                .map(Url::as_str)
                .unwrap_or(LDH_BASE),
            &RequestPlan::get(format!("ldh/Variant/id/{caid}/ld")).query("detail", "med"),
        ))
        .send_with_source_context(SourceContext::narrow(SourceProvider::CLINGEN_LDH))
        .await?;
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_LDH),
            MEDIUM_BODY_LIMIT,
        )
        .await?;
        serde_json::from_slice(&bytes).map_err(BioMcpError::Json)
    }

    pub(crate) async fn direct(&self, iri: &str) -> Result<Value, BioMcpError> {
        let mut url = Url::parse(iri).map_err(|_| {
            BioMcpError::InvalidArgument("invalid ClinGen LDH annotation IRI".into())
        })?;
        crate::sources::provider_url_policy::ProviderUrlPolicy::clingen_ldh()?
            .validate_url(&url)?;
        if let Some(origin) = &self.fixture_origin {
            url.set_scheme(origin.scheme()).map_err(|_| {
                BioMcpError::InvalidArgument("invalid ClinGen LDH fixture origin".into())
            })?;
            url.set_host(origin.host_str()).map_err(|_| {
                BioMcpError::InvalidArgument("invalid ClinGen LDH fixture origin".into())
            })?;
            url.set_port(origin.port()).map_err(|_| {
                BioMcpError::InvalidArgument("invalid ClinGen LDH fixture origin".into())
            })?;
        }
        let response = crate::sources::apply_cache_mode(self.client.get(url))
            .send_with_source_context(SourceContext::narrow(SourceProvider::CLINGEN_LDH))
            .await?;
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_LDH),
            DIRECT_BODY_LIMIT,
        )
        .await?;
        serde_json::from_slice(&bytes).map_err(BioMcpError::Json)
    }
}
