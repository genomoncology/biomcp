//! HTTP transport and API-specific probe helpers for `biomcp health`.

use std::time::Instant;

use crate::error::BioMcpError;

use super::HealthStatus;
use super::runner::{ProbeClass, ProbeOutcome, health_row, outcome};

pub(in crate::cli::health) async fn check_gencc_head(
    api: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let start = Instant::now();
    let healthy = match crate::sources::gencc::GenCcClient::new() {
        Ok(client) => client.health().await,
        Err(()) => false,
    };
    let elapsed = start.elapsed().as_millis();
    if healthy {
        outcome(
            health_row(api, HealthStatus::Ok, format!("{elapsed}ms"), None, None),
            ProbeClass::Healthy,
        )
    } else {
        outcome(
            health_row(
                api,
                HealthStatus::Error,
                format!("{elapsed}ms (error)"),
                affects,
                None,
            ),
            ProbeClass::Error,
        )
    }
}

pub(in crate::cli::health) fn configured_key(env_var: &str) -> Option<String> {
    configured_key_from_value(std::env::var(env_var).ok())
}

pub(in crate::cli::health) fn configured_key_from_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(in crate::cli::health) fn excluded_outcome(
    api: &str,
    env_var: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let mut row = health_row(
        api,
        HealthStatus::Excluded,
        "n/a".into(),
        affects,
        Some(false),
    );
    row.required_env_var = Some(env_var.to_string());
    outcome(row, ProbeClass::Excluded)
}

fn transport_error_latency(start: Instant, err: &reqwest::Error) -> String {
    let elapsed = start.elapsed().as_millis();
    if err.is_timeout() {
        format!("{elapsed}ms (timeout)")
    } else if err.is_connect() {
        format!("{elapsed}ms (connect)")
    } else {
        format!("{elapsed}ms (error)")
    }
}

fn api_error_latency(start: Instant, err: &BioMcpError) -> String {
    let elapsed = start.elapsed().as_millis();
    match err {
        BioMcpError::Api { message, .. } if message.contains("connect failed") => {
            format!("{elapsed}ms (connect)")
        }
        _ => format!("{elapsed}ms (error)"),
    }
}

pub(in crate::cli::health) async fn send_request(
    api: &str,
    affects: Option<&'static str>,
    request: reqwest::RequestBuilder,
    key_configured: Option<bool>,
) -> ProbeOutcome {
    let start = Instant::now();
    let response = request.send().await;

    match response {
        Ok(response) => {
            let status = response.status();
            let elapsed = start.elapsed().as_millis();
            if status.is_success() {
                outcome(
                    health_row(
                        api,
                        HealthStatus::Ok,
                        format!("{elapsed}ms"),
                        None,
                        key_configured,
                    ),
                    ProbeClass::Healthy,
                )
            } else {
                outcome(
                    health_row(
                        api,
                        HealthStatus::Error,
                        format!("{elapsed}ms (HTTP {})", status.as_u16()),
                        affects,
                        key_configured,
                    ),
                    ProbeClass::Error,
                )
            }
        }
        Err(err) => outcome(
            health_row(
                api,
                HealthStatus::Error,
                transport_error_latency(start, &err),
                affects,
                key_configured,
            ),
            ProbeClass::Error,
        ),
    }
}

pub(in crate::cli::health) async fn check_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    send_request(api, affects, client.get(url), None).await
}

pub(in crate::cli::health) async fn check_post_json(
    client: reqwest::Client,
    api: &str,
    url: &str,
    payload: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    send_request(
        api,
        affects,
        client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(payload.to_string()),
        None,
    )
    .await
}

pub(in crate::cli::health) async fn check_auth_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client.get(url).header(header_name, header_value),
        Some(true),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cli::health) fn optional_auth_status_outcome(
    api: &str,
    status: reqwest::StatusCode,
    elapsed_ms: u128,
    key_configured: Option<bool>,
    env_var: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    if status.is_success() {
        return outcome(
            health_row(
                api,
                if key_configured == Some(true) {
                    HealthStatus::Configured
                } else {
                    HealthStatus::Available
                },
                format!("{elapsed_ms}ms"),
                None,
                key_configured,
            ),
            ProbeClass::Healthy,
        );
    }

    if key_configured == Some(false) && status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let mut row = health_row(
            api,
            HealthStatus::Unavailable,
            format!("{elapsed_ms}ms"),
            None,
            key_configured,
        );
        row.required_env_var = Some(env_var.to_string());
        return outcome(row, ProbeClass::Healthy);
    }

    outcome(
        health_row(
            api,
            HealthStatus::Error,
            format!("{elapsed_ms}ms (HTTP {})", status.as_u16()),
            affects,
            key_configured,
        ),
        ProbeClass::Error,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cli::health) async fn check_optional_auth_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let key = configured_key(env_var);
    let key_configured = Some(key.is_some());
    let request = match key {
        Some(key) => client
            .get(url)
            .header(header_name, format!("{header_value_prefix}{key}")),
        None => client.get(url),
    };
    let start = Instant::now();
    let error_outcome = |latency: String| {
        outcome(
            health_row(api, HealthStatus::Error, latency, affects, key_configured),
            ProbeClass::Error,
        )
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let elapsed = start.elapsed().as_millis();
            optional_auth_status_outcome(api, status, elapsed, key_configured, env_var, affects)
        }
        Err(err) => error_outcome(transport_error_latency(start, &err)),
    }
}

pub(in crate::cli::health) async fn check_auth_query_param(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    param_name: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let req = match reqwest::Url::parse(url) {
        Ok(mut parsed) => {
            parsed.query_pairs_mut().append_pair(param_name, &key);
            client.get(parsed)
        }
        Err(err) => {
            return outcome(
                health_row(
                    api,
                    HealthStatus::Error,
                    format!("invalid url: {err}"),
                    affects,
                    Some(true),
                ),
                ProbeClass::Error,
            );
        }
    };

    send_request(api, affects, req, Some(true)).await
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cli::health) async fn check_auth_post_json(
    client: reqwest::Client,
    api: &str,
    url: &str,
    payload: &str,
    env_var: &str,
    header_name: &str,
    header_value_prefix: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let header_value = format!("{header_value_prefix}{key}");

    send_request(
        api,
        affects,
        client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(header_name, header_value)
            .body(payload.to_string()),
        Some(true),
    )
    .await
}

#[cfg(feature = "alphagenome")]
pub(in crate::cli::health) async fn check_alphagenome_connect(
    api: &str,
    env_var: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let Some(_key) = configured_key(env_var) else {
        return excluded_outcome(api, env_var, affects);
    };

    let start = Instant::now();

    match crate::sources::alphagenome::AlphaGenomeClient::new().await {
        Ok(_) => outcome(
            health_row(
                api,
                HealthStatus::Ok,
                format!("{}ms", start.elapsed().as_millis()),
                None,
                Some(true),
            ),
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            health_row(
                api,
                HealthStatus::Error,
                api_error_latency(start, &err),
                affects,
                Some(true),
            ),
            ProbeClass::Error,
        ),
    }
}

pub(in crate::cli::health) async fn check_vaers_query(
    api: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    let start = Instant::now();
    let client = match crate::sources::vaers::VaersClient::new() {
        Ok(client) => client,
        Err(err) => return vaers_query_outcome(api, affects, start, Err(err)),
    };

    vaers_query_outcome(api, affects, start, client.health_check().await)
}

pub(in crate::cli::health) fn vaers_query_outcome(
    api: &str,
    affects: Option<&'static str>,
    start: Instant,
    result: Result<(), BioMcpError>,
) -> ProbeOutcome {
    match result {
        Ok(()) => outcome(
            health_row(
                api,
                HealthStatus::Ok,
                format!("{}ms", start.elapsed().as_millis()),
                None,
                None,
            ),
            ProbeClass::Healthy,
        ),
        Err(err) => outcome(
            health_row(
                api,
                HealthStatus::Error,
                api_error_latency(start, &err),
                affects,
                None,
            ),
            ProbeClass::Error,
        ),
    }
}
