//! Request diagnostics for article retrieval. Never publish provider error bodies.
use reqwest::{Response, Url};
use reqwest_middleware::RequestBuilder;

use crate::error::{BioMcpError, SourceContext, SourceProvider};

const DIAGNOSTIC_API: &str = "article-fulltext-route";

tokio::task_local! {
    pub(crate) static CURRENT_REQUEST: RequestObservation;
}

/// One route owns this observation, including cancellation before send returns.
#[derive(Clone, Default)]
pub(crate) struct RequestObservation(std::sync::Arc<std::sync::Mutex<Option<RequestDiagnostic>>>);

#[derive(Clone)]
struct RequestDiagnostic {
    url: String,
    status: Option<u16>,
    context: SourceContext,
}

impl RequestObservation {
    pub(crate) fn timeout_detail(&self) -> String {
        self.snapshot()
            .map(|request| format!(" Request {}: transport error (timed out).", request.url))
            .unwrap_or_else(|| " No request URL was observed.".into())
    }

    fn snapshot(&self) -> Option<RequestDiagnostic> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn message(&self, error: &BioMcpError) -> Option<(SourceContext, String)> {
        let request = self.snapshot()?;
        let failure = error.external_failure(request.context.provider(), "article retrieval");
        let status = failure.status.or(request.status);
        let detail = match status {
            Some(status) if (200..300).contains(&status) => {
                let class = if failure.class == "internal" {
                    "decode"
                } else {
                    failure.class
                };
                format!("HTTP {status}; response error ({class})")
            }
            Some(status) => format!("HTTP {status}"),
            None => format!("transport error ({})", failure.class),
        };
        Some((
            request.context,
            format!("{detail}; request {}", request.url),
        ))
    }

    pub(crate) fn annotate(&self, error: BioMcpError) -> BioMcpError {
        if is_projected(&error) {
            return error;
        }
        match self.message(&error) {
            Some((context, message)) => BioMcpError::Api {
                api: DIAGNOSTIC_API.into(),
                message,
            }
            .with_source_context(context),
            None => error,
        }
    }
}

pub(crate) fn record_request(request: &RequestBuilder, context: SourceContext) {
    let _ = CURRENT_REQUEST.try_with(|observation| {
        let diagnostic = request
            .try_clone()
            .and_then(|req| req.build().ok())
            .map(|request| RequestDiagnostic {
                url: safe_url(request.url()),
                status: None,
                context,
            });
        *observation
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = diagnostic;
    });
}

pub(crate) fn record_response(response: &Response, context: SourceContext) {
    let _ = CURRENT_REQUEST.try_with(|observation| {
        // Replace the whole snapshot from this response. Do not combine one
        // concurrent request's status with a different request's URL.
        *observation
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(RequestDiagnostic {
            url: safe_url(response.url()),
            status: Some(response.status().as_u16()),
            context,
        });
    });
}

pub(crate) fn annotate_current_error(error: BioMcpError) -> BioMcpError {
    match CURRENT_REQUEST.try_with(Clone::clone) {
        Ok(observation) => observation.annotate(error),
        Err(_) => error,
    }
}

fn is_projected(error: &BioMcpError) -> bool {
    matches!(error.underlying(), BioMcpError::Api { api, .. } if api == DIAGNOSTIC_API)
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

/// Only this module's closed projection may be copied into a route warning.
pub(crate) fn safe_error(error: &BioMcpError) -> String {
    if let BioMcpError::Api { message, .. } = error.underlying()
        && is_projected(error)
    {
        return message.clone();
    }
    if let Ok(Some((_, message))) =
        CURRENT_REQUEST.try_with(|observation| observation.message(error))
    {
        return message;
    }
    format!(
        "Request details unavailable; {}",
        error
            .external_failure(SourceProvider::UNKNOWN, "article retrieval")
            .message
    )
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

    // Isolation is a diagnostic security contract: concurrent providers cannot
    // lend each other URLs or HTTP statuses, even on the same executor task.
    #[tokio::test]
    async fn concurrent_source_observations_keep_their_own_request_and_status() {
        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::Client::builder().no_proxy().build().unwrap(),
        )
        .build();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
        let observe = |path: &'static str, status: u16| {
            let client = client.clone();
            let barrier = barrier.clone();
            async move {
                CURRENT_REQUEST
                    .scope(RequestObservation::default(), async move {
                        record_request(
                            &client
                                .get(format!("https://example.org/{path}?token=SYNTHETIC_SECRET")),
                            SourceContext::retry(SourceProvider::PMC_OPEN_ACCESS),
                        );
                        CURRENT_REQUEST.with(|observation| {
                            let mut diagnostic = observation.snapshot().unwrap();
                            diagnostic.status = Some(status);
                            *observation.0.lock().unwrap() = Some(diagnostic);
                        });
                        barrier.wait().await;
                        safe_error(&BioMcpError::Api {
                            api: "provider".into(),
                            message: "SYNTHETIC_UNSAFE_BODY".into(),
                        })
                    })
                    .await
            }
        };
        let (first, second) = tokio::join!(observe("first", 503), observe("second", 200));
        assert!(first.contains("HTTP 503; request https://example.org/first?token=REDACTED"));
        assert!(second.contains(
            "HTTP 200; response error (decode); request https://example.org/second?token=REDACTED"
        ));
        assert!(!first.contains("second"));
        assert!(!second.contains("first"));
        assert!(!format!("{first}{second}").contains("SYNTHETIC"));
        assert_eq!(
            RequestObservation::default().timeout_detail(),
            " No request URL was observed."
        );
    }
}
