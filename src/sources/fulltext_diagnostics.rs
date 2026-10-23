//! Request diagnostics for article retrieval. Never publish provider error bodies.
use reqwest::{Response, StatusCode, Url};
use reqwest_middleware::RequestBuilder;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::RequestBuilderSourceContextExt;

const DIAGNOSTIC_API: &str = "article-fulltext-route";

tokio::task_local! {
    pub(crate) static CURRENT_REQUEST: RequestObservation;
}

/// One route owns this observation, including cancellation before send returns.
#[derive(Clone, Default)]
pub(crate) struct RequestObservation(std::sync::Arc<std::sync::Mutex<Option<String>>>);

impl RequestObservation {
    pub(crate) fn timeout_detail(&self) -> String {
        let url = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        url.as_ref()
            .map(|url| format!(" Request {url}: transport error (timed out)."))
            .unwrap_or_default()
    }

    fn record(url: &str) {
        let _ = CURRENT_REQUEST.try_with(|observation| {
            *observation
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(url.into());
        });
    }
}

fn safe_url(url: &Url) -> String {
    let mut safe = url.clone();
    let _ = safe.set_username("");
    let _ = safe.set_password(None);
    safe.set_fragment(None);
    let pairs = url
        .query_pairs()
        .map(|(key, value)| {
            let public = match key.as_ref() {
                "db" => value == "pmc",
                "rettype" => value == "xml",
                "list-type" => value == "2",
                "delimiter" => value == "/",
                "id" => !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()),
                "prefix" => value.strip_prefix("PMC").is_some_and(|id| {
                    let id = id.strip_suffix('.').unwrap_or(id);
                    !id.is_empty() && id.chars().all(|c| c.is_ascii_digit())
                }),
                _ => false,
            };
            (
                key.into_owned(),
                if public {
                    value.into_owned()
                } else {
                    "REDACTED".into()
                },
            )
        })
        .collect::<Vec<_>>();
    safe.set_query(None);
    if !pairs.is_empty() {
        safe.query_pairs_mut().extend_pairs(pairs);
    }
    safe.to_string()
}

fn request_error(url: &str, context: SourceContext, detail: String) -> BioMcpError {
    BioMcpError::Api {
        api: DIAGNOSTIC_API.into(),
        message: format!("{detail}; request {url}"),
    }
    .with_source_context(context)
}

/// Only this module's closed projection may be copied into a route warning.
pub(crate) fn safe_error(error: &BioMcpError) -> String {
    if let BioMcpError::Api { api, message } = error.underlying()
        && api == DIAGNOSTIC_API
    {
        return message.clone();
    }
    error
        .external_failure(SourceProvider::UNKNOWN, "article retrieval")
        .message
}

pub(crate) trait FulltextRequestExt {
    fn send_fulltext(
        self,
        context: SourceContext,
        absence_is_empty: bool,
    ) -> impl std::future::Future<Output = Result<Response, BioMcpError>> + Send;
}

impl FulltextRequestExt for RequestBuilder {
    async fn send_fulltext(
        self,
        context: SourceContext,
        absence_is_empty: bool,
    ) -> Result<Response, BioMcpError> {
        let url = self
            .try_clone()
            .and_then(|req| req.build().ok())
            .map(|req| safe_url(req.url()))
            .unwrap_or_else(|| "unavailable URL".into());
        RequestObservation::record(&url);
        let response = self
            .send_with_source_context(context)
            .await
            .map_err(|error| {
                let failure = error.external_failure(context.provider(), "article retrieval");
                let detail = match failure.status {
                    Some(status) => format!("HTTP {status}"),
                    None => format!("transport error ({})", failure.class),
                };
                request_error(&url, context, detail)
            })?;
        let status = response.status();
        let absence =
            absence_is_empty && matches!(status, StatusCode::NOT_FOUND | StatusCode::NO_CONTENT);
        if !status.is_success() && !absence {
            return Err(request_error(&url, context, format!("HTTP {status}")));
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Secrecy is a security boundary, so exercise hostile URL fields directly.
    #[test]
    fn request_diagnostics_redact_credentials_and_untrusted_query_values() {
        let sentinel = "SYNTHETIC_CREDENTIAL_SENTINEL";
        for key in [
            "api_key",
            "API_KEY",
            "token",
            "access_token",
            "signature",
            "unknown",
            "id",
            "prefix",
        ] {
            let url = Url::parse(&format!("https://{sentinel}:{sentinel}@example.org/efetch.fcgi?{key}={sentinel}&db=pmc&id=8010000#{sentinel}")).unwrap();
            let safe = safe_url(&url);
            assert!(!safe.contains(sentinel));
            assert!(safe.contains("db=pmc&id=8010000"));
            assert!(safe.contains("REDACTED"));
        }
    }
}
