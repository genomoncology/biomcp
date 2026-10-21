//! One retry budget for article-source HTTP sends, inside provider pacing.
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use http::Extensions;
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::{Middleware, Next};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

use crate::error::{BioMcpError, SourceProvider};

#[derive(Clone, Copy, Debug, thiserror::Error)]
#[error("failure_class={class}; retry_attempted={retry_attempted}")]
pub(crate) struct Failure {
    pub(crate) class: &'static str,
    pub(crate) retry_attempted: bool,
}

#[derive(Default)]
struct State {
    class: Option<&'static str>,
    sends: usize,
    terminal: bool,
}
#[derive(Clone, Default)]
pub(crate) struct Observation(Arc<Mutex<State>>);
impl Observation {
    pub(super) fn start(&self) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = State::default();
    }
    fn set(&self, class: Option<&'static str>) {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.class = class;
        state.terminal = false;
    }
    fn finish(&self, class: &'static str) -> Failure {
        self.set(Some(class));
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .terminal = true;
        self.get().expect("terminal failure has a class")
    }
    pub(super) fn terminal(&self) -> Option<Failure> {
        let terminal = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .terminal;
        terminal.then(|| self.get()).flatten()
    }
    fn admit(&self) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sends += 1;
    }
    pub(crate) fn get(&self) -> Option<Failure> {
        let state = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.class.map(|class| Failure {
            class,
            retry_attempted: state.sends > 1,
        })
    }
    pub(crate) fn timeout(&self) -> Failure {
        self.finish("timeout")
    }
}

tokio::task_local! {
    static LEG_DEADLINE: tokio::time::Instant;
    static LEG_OBSERVATION: Observation;
}

pub(crate) async fn within_leg<F: Future>(
    limit: Duration,
    observation: Observation,
    future: F,
) -> F::Output {
    LEG_DEADLINE
        .scope(
            tokio::time::Instant::now() + limit,
            LEG_OBSERVATION.scope(observation, future),
        )
        .await
}

pub(super) fn current_observation() -> Observation {
    LEG_OBSERVATION.try_with(Clone::clone).unwrap_or_default()
}

pub(super) fn applies(provider: SourceProvider) -> bool {
    super::current_variant_article_deadline().is_some()
        && matches!(
            provider,
            SourceProvider::EUROPE_PMC
                | SourceProvider::SEMANTIC_SCHOLAR
                | SourceProvider::PUBTATOR3
        )
}

fn remaining(deadline: &super::VariantArticleDeadline) -> Duration {
    LEG_DEADLINE
        .try_with(|at| at.saturating_duration_since(tokio::time::Instant::now()))
        .unwrap_or_else(|_| deadline.remaining())
        .min(deadline.remaining())
}

fn response_class(status: StatusCode) -> &'static str {
    match status.as_u16() {
        400 => "http_400",
        401 => "http_401",
        403 => "http_403",
        404 => "http_404",
        429 => "http_429",
        502 => "http_502",
        503 => "http_503",
        _ => "http_error",
    }
}

fn retry_after(response: &Response) -> Option<Duration> {
    let value = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim();
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let date = chrono::DateTime::parse_from_rfc2822(value).ok()?;
    Some(
        (date.with_timezone(&chrono::Utc) - chrono::Utc::now())
            .to_std()
            .unwrap_or_default(),
    )
}

pub(super) struct LiteratureRetryMiddleware;
#[async_trait::async_trait]
impl Middleware for LiteratureRetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let Some(observation) = extensions.get::<Observation>().cloned() else {
            return RetryTransientMiddleware::new_with_policy_and_strategy(
                ExponentialBackoff::builder().build_with_max_retries(3),
                super::NoTrustFailureStrategy,
            )
            .handle(req, extensions, next)
            .await;
        };
        let deadline = extensions
            .get::<super::VariantArticleDeadline>()
            .cloned()
            .expect("article retry observation requires an invocation deadline");
        for attempt in 0..=1 {
            let Some(request) = req.try_clone() else {
                return next.run(req, extensions).await;
            };
            let response = next.clone().run(request, extensions).await;
            if response
                .as_ref()
                .is_ok_and(|response| response.status().is_success())
            {
                return response;
            }
            let (class, retryable, delay) = match &response {
                Ok(response) => (
                    response_class(response.status()),
                    matches!(response.status().as_u16(), 502 | 503 | 429),
                    retry_after(response),
                ),
                Err(reqwest_middleware::Error::Reqwest(error)) if error.is_timeout() => {
                    ("timeout", true, None)
                }
                // Shared-pool refusal and TLS/configuration failures retain their existing policies.
                Err(_) => {
                    observation.set(None);
                    return response;
                }
            };
            observation.set(Some(class));
            let failure = observation.get().expect("failed response has a class");
            if attempt == 1 || !retryable {
                return Err(reqwest_middleware::Error::middleware(
                    observation.finish(failure.class),
                ));
            }
            let delay = delay.unwrap_or_default().max(Duration::from_millis(100));
            if delay >= remaining(&deadline) {
                return Err(reqwest_middleware::Error::middleware(
                    observation.finish(failure.class),
                ));
            }
            if !super::retry_sleep_disabled() {
                // sleep reason: provider Retry-After/backoff inside the invocation and leg deadlines
                #[allow(clippy::disallowed_methods)]
                deadline
                    .run(tokio::time::sleep(delay))
                    .await
                    .map_err(reqwest_middleware::Error::middleware)?;
            }
        }
        unreachable!("one retry has two attempts")
    }
}

pub(crate) fn failure(error: &BioMcpError) -> Option<Failure> {
    match error {
        BioMcpError::WithSourceContext { source, .. } => failure(source),
        BioMcpError::Api { api, message }
            if api == super::semantic_scholar::SEMANTIC_SCHOLAR_API
                && message
                    .starts_with(super::semantic_scholar::SEMANTIC_SCHOLAR_RATE_LIMITED_PREFIX) =>
        {
            Some(Failure {
                class: "rate_limited",
                retry_attempted: false,
            })
        }
        BioMcpError::HttpMiddleware(reqwest_middleware::Error::Middleware(error)) => {
            error.downcast_ref::<Failure>().copied()
        }
        _ => None,
    }
}

/// This boundary sits after pacing, immediately before transport/body collection.
/// Only an admitted second send counts as a retry; waiting for a permit does not.
pub(super) struct ObserveTransport;
#[async_trait::async_trait]
impl Middleware for ObserveTransport {
    async fn handle(
        &self,
        mut req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        if let Some(observation) = extensions.get::<Observation>() {
            let deadline = extensions
                .get::<super::VariantArticleDeadline>()
                .expect("article transport requires an invocation deadline");
            let remaining = remaining(deadline);
            if remaining.is_zero() {
                return Err(reqwest_middleware::Error::middleware(observation.timeout()));
            }
            // Recompute after pacing. Reserve time for the retry and other routes.
            let budget = remaining / 3;
            *req.timeout_mut() = Some(
                req.timeout()
                    .copied()
                    .unwrap_or(super::HTTP_REQUEST_TIMEOUT)
                    .min(budget),
            );
            observation.admit();
        }
        next.run(req, extensions).await
    }
}
