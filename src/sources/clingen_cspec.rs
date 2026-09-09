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
        let client = crate::sources::provider_policy_client_builder(&policy)
            .connect_timeout(CspecTimeouts::default().connect)
            .timeout(CspecTimeouts::default().request)
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
mod tests {
    mod construction {
        use super::super::*;
        use crate::sources::HttpMethod;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        #[test]
        fn cspec_production_timeouts_match_shared_provider_policy() {
            let timeouts = CspecTimeouts::default();
            assert_eq!(timeouts.connect, Duration::from_secs(10));
            assert_eq!(timeouts.request, Duration::from_secs(30));
        }

        #[test]
        fn cspec_plans_keep_manifest_and_document_provider_paths() {
            let manifest = CspecClient::manifest_plan("ATM");
            assert_eq!(manifest.method, HttpMethod::Get);
            assert_eq!(
                manifest.path,
                "cspec/Gene/id/ATM/SequenceVariantInterpretation/version"
            );
            assert_eq!(manifest.query_value("detail"), Some("low"));

            let iri = Url::parse(
        "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1",
    )
    .expect("CSpec IRI");
            let document = CspecClient::document_plan(&iri);
            assert_eq!(document.method, HttpMethod::Get);
            assert_eq!(
                document.path,
                "/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
            );
            assert!(document.query.is_empty());
        }

        #[tokio::test]
        async fn cspec_execution_methods_consume_manifest_and_document_plans() {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind CSpec fixture");
            let base = format!("http://{}", listener.local_addr().expect("fixture address"));
            let manifest = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen_cspec/atm-manifest.json"
            ));
            let document = include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen_cspec/atm-gn020-1.5.1.json"
            ));
            let server = tokio::spawn(async move {
                let mut requests = Vec::new();
                for body in [manifest, document] {
                    let (mut stream, _) = listener.accept().await.expect("accept CSpec request");
                    let mut request = Vec::new();
                    loop {
                        let mut chunk = [0_u8; 4096];
                        let read = stream.read(&mut chunk).await.expect("read CSpec request");
                        if read == 0 {
                            break;
                        }
                        request.extend_from_slice(&chunk[..read]);
                        if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                            break;
                        }
                    }
                    requests.push(String::from_utf8_lossy(&request).into_owned());
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body,
                    );
                    stream
                        .write_all(response.as_bytes())
                        .await
                        .expect("write response");
                }
                requests
            });
            let client = CspecClient::with_test_client_at(
                crate::sources::test_client().expect("test client"),
                base,
            );

            let manifest = client.manifest("ATM").await.expect("manifest response");
            assert_eq!(
                manifest["data"][0]["@id"].as_str(),
                Some(
                    "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
                )
            );
            client
        .document(&Url::parse("https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1").expect("CSpec IRI"))
        .await
        .expect("document response");

            let requests = server.await.expect("CSpec fixture server");
            assert!(requests[0].starts_with(
                "GET /cspec/Gene/id/ATM/SequenceVariantInterpretation/version?detail=low "
            ));
            assert!(
                requests[1].starts_with(
                    "GET /cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1 "
                )
            );
        }

        #[tokio::test]
        async fn cspec_request_deadline_covers_headers_and_body_with_safe_attribution() {
            for stall_after_headers in [false, true] {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind timeout fixture");
                let base = format!("http://{}", listener.local_addr().expect("fixture address"));
                let server = tokio::spawn(async move {
                    let (mut stream, _) = listener.accept().await.expect("accept request");
                    let mut request = [0_u8; 1024];
                    let _ = stream.read(&mut request).await.expect("read request");
                    if stall_after_headers {
                        stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 100\r\n\r\n{")
                    .await
                    .expect("write partial response");
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                });
                let client = CspecClient::with_test_timeouts_at(
                    base,
                    Duration::from_millis(25),
                    Duration::from_millis(25),
                )
                .expect("test client");

                let error = client
                    .manifest("PTEN")
                    .await
                    .expect_err("stalled request must time out");
                let projection = error.public_projection();
                assert_eq!(projection.source, Some("ClinGen CSpec"));
                assert!(!projection.message.contains("127.0.0.1"));
                server.await.expect("timeout fixture");
            }
        }
    }
}
