use reqwest::{StatusCode, Url};
use serde_json::Value;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{RequestBuilderSourceContextExt, read_limited_source_body_with_limit};

pub(crate) const CSPEC_BASE: &str = "https://cspec.clinicalgenome.org";
const MANIFEST_LIMIT: usize = 256 * 1024;
const DOCUMENT_LIMIT: usize = 4 * 1024 * 1024;

pub(crate) struct CspecClient {
    client: reqwest_middleware::ClientWithMiddleware,
}

impl CspecClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        let policy = crate::sources::provider_url_policy::ProviderUrlPolicy::cspec()?;
        let client = reqwest::Client::builder()
            .dns_resolver(policy.dns_resolver())
            .redirect(policy.redirect_policy())
            .build()
            .map_err(BioMcpError::from)?;
        Ok(Self {
            client: reqwest_middleware::ClientBuilder::new(client).build(),
        })
    }

    pub(crate) async fn manifest(&self, gene: &str) -> Result<Value, BioMcpError> {
        let mut url = Url::parse(CSPEC_BASE).expect("static CSpec origin is valid");
        url.path_segments_mut()
            .expect("static origin accepts path segments")
            .extend([
                "cspec",
                "Gene",
                "id",
                gene,
                "SequenceVariantInterpretation",
                "version",
            ]);
        url.query_pairs_mut().append_pair("detail", "low");
        self.get(url, MANIFEST_LIMIT)
            .await
            .map(|(_, body)| decode_envelope(&body))?
    }

    pub(crate) async fn document(&self, iri: &Url) -> Result<Vec<u8>, BioMcpError> {
        crate::sources::provider_url_policy::ProviderUrlPolicy::cspec()?.validate_url(iri)?;
        let (_, bytes) = self.get(iri.clone(), DOCUMENT_LIMIT).await?;
        decode_envelope(&bytes)?;
        Ok(bytes)
    }

    async fn get(&self, url: Url, limit: usize) -> Result<(StatusCode, Vec<u8>), BioMcpError> {
        let response = self
            .client
            .get(url)
            .send_with_source_context(SourceContext::retry(SourceProvider::CLINGEN_CSPEC))
            .await?;
        let status = response.status();
        let bytes = read_limited_source_body_with_limit(
            response,
            SourceContext::narrow(SourceProvider::CLINGEN_CSPEC),
            limit,
        )
        .await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: "ClinGen CSpec".into(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }
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
