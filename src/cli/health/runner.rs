//! Probe orchestration and health report assembly for `biomcp health`.

use std::sync::OnceLock;
use std::time::Duration;

use futures::stream::{self, StreamExt};

use crate::error::BioMcpError;

use super::catalog::{ProbeKind, SourceDescriptor, health_sources};
use super::http::{
    check_alphagenome_connect, check_auth_get, check_auth_post_json, check_auth_query_param,
    check_get, check_optional_auth_get, check_post_json, check_vaers_query, configured_key,
};
use super::local::{
    check_cache_dir, check_cache_limits, check_cvx_local_data, check_ddinter_local_data,
    check_ema_local_data, check_gtr_local_data, check_who_ivd_local_data, check_who_local_data,
};
use super::{HealthReport, HealthRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::cli::health) enum ProbeClass {
    Healthy,
    Warning,
    Error,
    Excluded,
}

#[derive(Debug, Clone)]
pub(in crate::cli::health) struct ProbeOutcome {
    pub(in crate::cli::health) row: HealthRow,
    pub(in crate::cli::health) class: ProbeClass,
}

pub(in crate::cli::health) fn health_row(
    api: &str,
    status: String,
    latency: String,
    affects: Option<&'static str>,
    key_configured: Option<bool>,
) -> HealthRow {
    HealthRow {
        api: api.to_string(),
        status,
        latency,
        affects: affects.map(str::to_string),
        key_configured,
    }
}

pub(in crate::cli::health) fn outcome(row: HealthRow, class: ProbeClass) -> ProbeOutcome {
    ProbeOutcome { row, class }
}

pub(in crate::cli::health) async fn probe_source(
    client: reqwest::Client,
    source: &SourceDescriptor,
) -> ProbeOutcome {
    match source.probe {
        ProbeKind::Get { url } => check_get(client, source.api, url, source.affects).await,
        ProbeKind::PostJson { url, payload } => {
            check_post_json(client, source.api, url, payload, source.affects).await
        }
        ProbeKind::AuthGet {
            url,
            env_var,
            header_name,
            header_value_prefix,
        } => {
            check_auth_get(
                client,
                source.api,
                url,
                env_var,
                header_name,
                header_value_prefix,
                source.affects,
            )
            .await
        }
        ProbeKind::OptionalAuthGet {
            url,
            env_var,
            header_name,
            header_value_prefix,
            unauthenticated_ok_status,
            authenticated_ok_status,
            unauthenticated_rate_limited_status,
        } => {
            check_optional_auth_get(
                client,
                source.api,
                url,
                env_var,
                header_name,
                header_value_prefix,
                unauthenticated_ok_status,
                authenticated_ok_status,
                unauthenticated_rate_limited_status,
                source.affects,
            )
            .await
        }
        ProbeKind::AuthQueryParam {
            url,
            env_var,
            param_name,
        } => {
            check_auth_query_param(client, source.api, url, env_var, param_name, source.affects)
                .await
        }
        ProbeKind::AuthPostJson {
            url,
            payload,
            env_var,
            header_name,
            header_value_prefix,
        } => {
            check_auth_post_json(
                client,
                source.api,
                url,
                payload,
                env_var,
                header_name,
                header_value_prefix,
                source.affects,
            )
            .await
        }
        ProbeKind::AlphaGenomeConnect { env_var } => {
            check_alphagenome_connect(source.api, env_var, source.affects).await
        }
        ProbeKind::VaersQuery => check_vaers_query(source.api, source.affects).await,
    }
}

pub(in crate::cli::health) const HEALTH_API_PROBE_CONCURRENCY_LIMIT: usize = 16;
const HEALTH_API_PROBE_TIMEOUT: Duration = Duration::from_secs(12);

pub(in crate::cli::health) async fn run_buffered_in_order<T, O, F, Fut, I>(
    items: I,
    concurrency_limit: usize,
    runner: F,
) -> Vec<O>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T) -> Fut,
    Fut: std::future::Future<Output = O>,
{
    assert!(
        concurrency_limit > 0,
        "concurrency_limit must be greater than zero"
    );
    stream::iter(items)
        .map(runner)
        .buffered(concurrency_limit)
        .collect()
        .await
}

fn timeout_key_configured(source: SourceDescriptor) -> Option<bool> {
    match source.probe {
        ProbeKind::AuthGet { .. }
        | ProbeKind::AuthQueryParam { .. }
        | ProbeKind::AuthPostJson { .. }
        | ProbeKind::AlphaGenomeConnect { .. } => Some(true),
        ProbeKind::OptionalAuthGet { env_var, .. } => Some(configured_key(env_var).is_some()),
        ProbeKind::Get { .. } | ProbeKind::PostJson { .. } | ProbeKind::VaersQuery => None,
    }
}

fn timed_out_probe_outcome(source: SourceDescriptor, timeout: Duration) -> ProbeOutcome {
    outcome(
        health_row(
            source.api,
            "error".into(),
            format!("{}ms (timeout)", timeout.as_millis()),
            source.affects,
            timeout_key_configured(source),
        ),
        ProbeClass::Error,
    )
}

pub(in crate::cli::health) async fn probe_source_with_timeout_for_test(
    client: reqwest::Client,
    source: SourceDescriptor,
    timeout: Duration,
) -> ProbeOutcome {
    match tokio::time::timeout(timeout, probe_source(client, &source)).await {
        Ok(outcome) => outcome,
        Err(_) => timed_out_probe_outcome(source, timeout),
    }
}

pub(in crate::cli::health) async fn probe_source_with_timeout(
    client: reqwest::Client,
    source: SourceDescriptor,
) -> ProbeOutcome {
    probe_source_with_timeout_for_test(client, source, HEALTH_API_PROBE_TIMEOUT).await
}

async fn run_api_probes(client: reqwest::Client) -> Vec<ProbeOutcome> {
    run_buffered_in_order(
        health_sources().iter().copied(),
        HEALTH_API_PROBE_CONCURRENCY_LIMIT,
        move |source| probe_source_with_timeout(client.clone(), source),
    )
    .await
}

fn health_http_client() -> Result<reqwest::Client, BioMcpError> {
    static HEALTH_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

    if let Some(client) = HEALTH_HTTP_CLIENT.get() {
        return Ok(client.clone());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)?;

    match HEALTH_HTTP_CLIENT.set(client.clone()) {
        Ok(()) => Ok(client),
        Err(_) => HEALTH_HTTP_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| BioMcpError::Api {
                api: "health".into(),
                message: "Health HTTP client initialization race".into(),
            }),
    }
}

pub(in crate::cli::health) fn report_from_outcomes(outcomes: Vec<ProbeOutcome>) -> HealthReport {
    let healthy = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Healthy)
        .count();
    let warning = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Warning)
        .count();
    let excluded = outcomes
        .iter()
        .filter(|outcome| outcome.class == ProbeClass::Excluded)
        .count();
    let rows = outcomes
        .into_iter()
        .map(|outcome| outcome.row)
        .collect::<Vec<_>>();

    HealthReport {
        healthy,
        warning,
        excluded,
        total: rows.len(),
        rows,
    }
}

/// Runs connectivity checks for configured upstream APIs and local EMA/CVX/WHO/GTR/WHO IVD/cache readiness.
///
/// # Errors
///
/// Returns an error when the shared HTTP client cannot be created.
pub(super) async fn check(apis_only: bool) -> Result<HealthReport, BioMcpError> {
    let client = health_http_client()?;
    let mut outcomes = run_api_probes(client).await;

    if !apis_only {
        outcomes.push(check_ema_local_data());
        outcomes.push(check_cvx_local_data());
        outcomes.push(check_ddinter_local_data());
        outcomes.push(check_who_local_data());
        outcomes.push(check_gtr_local_data());
        outcomes.push(check_who_ivd_local_data());
        outcomes.push(check_cache_dir().await);
        outcomes.push(check_cache_limits().await);
    }

    Ok(report_from_outcomes(outcomes))
}
