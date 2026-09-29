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

/// Rewrites a probe URL's scheme and authority onto a test endpoint
/// when `BIOMCP_HEALTH_PROBE_BASE` is allowed, keeping the path and
/// query. Test-only seam (ticket 1254, item 8): the health probes
/// carry static catalog URLs, so the handshake test needs one address
/// override through the shared client. Authed probes attach operator
/// credentials, so the override is gated exactly like the GenCC
/// endpoint override: debug builds, or a base that names the same
/// loopback origin as `BIOMCP_TEST_UNPACED_ORIGIN`. Unset, not
/// allowed, or unparseable (either side) leaves the catalog URL
/// unchanged.
fn probe_url(url: &str) -> String {
    let Ok(base) = std::env::var("BIOMCP_HEALTH_PROBE_BASE") else {
        return url.to_string();
    };
    if !(cfg!(debug_assertions) || fixture_override_allowed(&base)) {
        return url.to_string();
    }
    let Ok(base) = reqwest::Url::parse(base.trim()) else {
        return url.to_string();
    };
    let Ok(mut parsed) = reqwest::Url::parse(url) else {
        return url.to_string();
    };
    let _ = parsed.set_scheme(base.scheme());
    let Ok(()) = parsed.set_host(base.host_str()) else {
        return url.to_string();
    };
    if let Some(port) = base.port() {
        let _ = parsed.set_port(Some(port));
    } else {
        let _ = parsed.set_port(None);
    }
    parsed.to_string()
}

/// Mirrors `sources::gencc::fixture_override_allowed` (kept there at
/// its pinned size): the release-build half of the endpoint override
/// gate. The base must name exactly the loopback origin the test
/// signal allows.
#[rustfmt::skip]
fn fixture_override_allowed(value: &str) -> bool {
    let Ok(endpoint) = reqwest::Url::parse(value) else { return false };
    let Ok(signal) = std::env::var("BIOMCP_TEST_UNPACED_ORIGIN") else { return false };
    let Ok(signal) = reqwest::Url::parse(signal.trim()) else { return false };
    signal.username().is_empty()
        && signal.password().is_none()
        && signal.path() == "/"
        && signal.query().is_none()
        && signal.fragment().is_none()
        && signal
            .host_str()
            .and_then(|host| host.parse::<std::net::IpAddr>().ok())
            .is_some_and(|address| address.is_loopback())
        && endpoint.scheme() == signal.scheme()
        && endpoint.host_str() == signal.host_str()
        && endpoint.port_or_known_default() == signal.port_or_known_default()
}

pub(in crate::cli::health) async fn check_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    send_request(api, affects, client.get(probe_url(url)), None).await
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
            .post(probe_url(url))
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
        client.get(probe_url(url)).header(header_name, header_value),
        Some(true),
    )
    .await
}

/// Ticket 1142 health row: the ORCID token validator decides before any
/// network work. Missing or ASCII-space-only tokens exclude the row; an
/// invalid nonblank token is an error with zero GETs; a valid token performs
/// exactly one bearer GET.
pub(in crate::cli::health) async fn check_orcid_get(
    client: reqwest::Client,
    api: &str,
    url: &str,
    env_var: &str,
    affects: Option<&'static str>,
) -> ProbeOutcome {
    match crate::sources::orcid::OrcidClient::credential_state() {
        "excluded" => excluded_outcome(api, env_var, affects),
        "error" => {
            let mut row = health_row(api, HealthStatus::Error, "n/a".into(), affects, Some(true));
            row.required_env_var = Some(env_var.to_string());
            outcome(row, ProbeClass::Error)
        }
        _ => {
            let token = std::env::var(env_var).unwrap_or_default();
            let token = token.trim_matches(' ');
            send_request(
                api,
                affects,
                client
                    .get(probe_url(url))
                    .header("Accept", "application/vnd.orcid+json")
                    .header("Authorization", format!("Bearer {token}")),
                Some(true),
            )
            .await
        }
    }
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
            .get(probe_url(url))
            .header(header_name, format!("{header_value_prefix}{key}")),
        None => client.get(probe_url(url)),
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

    let req = match reqwest::Url::parse(&probe_url(url)) {
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
            .post(probe_url(url))
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

#[cfg(test)]
mod probe_override_tests {
    use super::{fixture_override_allowed, probe_url};

    #[test]
    #[serial_test::serial(unpaced_origin)]
    fn a_non_loopback_probe_base_is_never_allowed() {
        // No signal is set in this process: without the fixture signal
        // nothing is allowed, and a non-loopback base is not allowed
        // even by a matching-looking signal (checked below).
        unsafe {
            std::env::remove_var("BIOMCP_TEST_UNPACED_ORIGIN");
        }
        assert!(!fixture_override_allowed("https://example.com"));
        assert!(!fixture_override_allowed("https://127.0.0.1:9"));
        assert!(!fixture_override_allowed("not a url"));
    }

    #[test]
    #[serial_test::serial(unpaced_origin)]
    fn a_non_loopback_base_with_a_loopback_signal_is_rejected() {
        unsafe {
            std::env::set_var("BIOMCP_TEST_UNPACED_ORIGIN", "https://127.0.0.1:9443");
        }
        assert!(!fixture_override_allowed("https://example.com"));
        unsafe {
            std::env::remove_var("BIOMCP_TEST_UNPACED_ORIGIN");
        }
    }

    #[test]
    #[serial_test::serial(unpaced_origin)]
    fn the_exact_loopback_origin_the_signal_names_is_allowed() {
        unsafe {
            std::env::set_var("BIOMCP_TEST_UNPACED_ORIGIN", "https://127.0.0.1:9443");
        }
        assert!(fixture_override_allowed("https://127.0.0.1:9443"));
        unsafe {
            std::env::remove_var("BIOMCP_TEST_UNPACED_ORIGIN");
        }
    }

    #[test]
    #[serial_test::serial(unpaced_origin)]
    fn an_unparseable_base_leaves_the_catalog_url_unchanged() {
        // Serial with the other override tests: both set process
        // variables. The base cannot parse, so in release builds the
        // gate blocks the rewrite and in debug builds the parse
        // fallback returns the original — the observable is the same
        // URL either way, which is what this pins (ticket 1257).
        unsafe {
            std::env::set_var("BIOMCP_HEALTH_PROBE_BASE", "::not a url::");
        }
        let url = "https://mygene.info/v3/query?q=BRAF&size=1";
        assert_eq!(probe_url(url), url.to_string());
        unsafe {
            std::env::remove_var("BIOMCP_HEALTH_PROBE_BASE");
        }
    }

    #[test]
    #[serial_test::serial(unpaced_origin)]
    fn an_allowed_base_keeps_the_path_and_query() {
        // The exact pair the release gate demands: the base names the
        // same loopback origin as the signal, so `probe_url` rewrites
        // in debug AND release builds — the old test set only the
        // base, which debug builds let through and release builds
        // correctly refused (ticket 1257).
        unsafe {
            std::env::set_var("BIOMCP_HEALTH_PROBE_BASE", "https://127.0.0.1:9443");
            std::env::set_var("BIOMCP_TEST_UNPACED_ORIGIN", "https://127.0.0.1:9443");
        }
        let rewritten = probe_url("https://mygene.info/v3/query?q=BRAF&size=1");
        assert_eq!(rewritten, "https://127.0.0.1:9443/v3/query?q=BRAF&size=1");
        unsafe {
            std::env::remove_var("BIOMCP_HEALTH_PROBE_BASE");
            std::env::remove_var("BIOMCP_TEST_UNPACED_ORIGIN");
        }
    }
}
