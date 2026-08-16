use axum::{
    Json, Router,
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, Uri, header::CONTENT_LENGTH},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::BioMcpServer;

const MCP_HTTP_BODY_LIMIT: usize = 65_536;

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

async fn index_handler() -> Json<serde_json::Value> {
    let identity = crate::build_identity::current();
    Json(json!({
        "name": "biomcp",
        "version": identity.version,
        "git_revision": identity.git_revision,
        "build_timestamp": identity.build_date,
        "transport": "streamable-http",
        "mcp": "/mcp"
    }))
}

fn http_allowed_hosts(
    ip: std::net::IpAddr,
    allowed_hosts: Vec<String>,
    unsafe_allow_any_host: bool,
) -> anyhow::Result<HostPolicy> {
    if unsafe_allow_any_host && !allowed_hosts.is_empty() {
        anyhow::bail!("--allowed-hosts cannot be combined with --unsafe-allow-any-host");
    }
    if unsafe_allow_any_host {
        return Ok(HostPolicy::default());
    }
    if !allowed_hosts.is_empty() {
        return allowed_hosts
            .iter()
            .map(|host| normalize_allowed_host(host))
            .collect::<anyhow::Result<_>>()
            .map(HostPolicy::new);
    }
    if ip.is_loopback() {
        return Ok(HostPolicy::new(vec![
            normalized_authority("localhost").expect("valid loopback hostname"),
            normalized_authority("127.0.0.1").expect("valid loopback IPv4 address"),
            normalized_authority("::1").expect("valid loopback IPv6 address"),
        ]));
    }
    anyhow::bail!(
        "A non-loopback serve-http bind requires --allowed-hosts or --unsafe-allow-any-host"
    )
}

#[derive(Clone, Debug, Default)]
struct HostPolicy {
    allowed_hosts: Vec<NormalizedAuthority>,
}

impl HostPolicy {
    fn new(allowed_hosts: Vec<NormalizedAuthority>) -> Self {
        let mut policy = Self::default();
        for host in allowed_hosts {
            if !policy.allowed_hosts.contains(&host) {
                policy.allowed_hosts.push(host);
            }
        }
        policy
    }

    fn transport_hosts(&self) -> Vec<String> {
        self.allowed_hosts
            .iter()
            .map(NormalizedAuthority::as_host_header)
            .collect()
    }

    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.allowed_hosts.is_empty()
    }
}

impl<const N: usize> PartialEq<[&str; N]> for HostPolicy {
    fn eq(&self, other: &[&str; N]) -> bool {
        self.transport_hosts().iter().map(String::as_str).eq(*other)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalizedAuthority {
    host: String,
    port: Option<u16>,
}

impl NormalizedAuthority {
    fn as_host_header(&self) -> String {
        let host = if self.host.contains(':') && self.port.is_some() {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        };
        self.port
            .map_or(host.clone(), |port| format!("{host}:{port}"))
    }
}

fn normalized_authority(value: &str) -> Option<NormalizedAuthority> {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return None;
    }

    let host_without_trailing_dot = value.trim_end_matches('.');
    if let Ok(address) = host_without_trailing_dot.parse::<std::net::IpAddr>() {
        return Some(NormalizedAuthority {
            host: address.to_string(),
            port: None,
        });
    }
    if looks_like_ipv4_address(host_without_trailing_dot) {
        return None;
    }

    if let Ok(authority) = axum::http::uri::Authority::try_from(value) {
        let port = authority.port_u16();
        if has_explicit_port(value) && port.is_none() {
            return None;
        }
        let host = authority
            .host()
            .trim_matches(['[', ']'])
            .trim_end_matches('.')
            .to_ascii_lowercase();
        if port == Some(0) || !is_valid_host(&host) {
            return None;
        }
        return Some(NormalizedAuthority { host, port });
    }
    None
}

fn has_explicit_port(value: &str) -> bool {
    if let Some(rest) = value.strip_prefix('[') {
        return rest.find(']').is_some_and(|index| index + 1 < rest.len());
    }
    value.contains(':')
}

fn looks_like_ipv4_address(value: &str) -> bool {
    let mut labels = value.split('.');
    (0..4).all(|_| {
        labels
            .next()
            .is_some_and(|label| !label.is_empty() && label.bytes().all(|b| b.is_ascii_digit()))
    }) && labels.next().is_none()
}

fn normalize_allowed_host(value: &str) -> anyhow::Result<NormalizedAuthority> {
    normalized_authority(value).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid --allowed-hosts entry {value:?}: use a hostname or IP address with an optional port from 1 to 65535"
        )
    })
}

fn is_valid_host(host: &str) -> bool {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return true;
    }
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn request_authority(uri: &Uri, headers: &HeaderMap) -> Option<NormalizedAuthority> {
    if let Some(host) = headers.get(axum::http::header::HOST) {
        return host.to_str().ok().and_then(normalized_authority);
    }
    uri.authority()
        .and_then(|value| normalized_authority(value.as_str()))
}

fn normalized_host_is_allowed(
    host: &NormalizedAuthority,
    allowed_hosts: &[NormalizedAuthority],
) -> bool {
    allowed_hosts.is_empty()
        || allowed_hosts.iter().any(|allowed| {
            allowed.host == host.host && allowed.port.is_none_or(|port| host.port == Some(port))
        })
}

async fn enforce_host_policy(
    State(policy): State<HostPolicy>,
    request: Request,
    next: Next,
) -> Response {
    let Some(host) = request_authority(request.uri(), request.headers()) else {
        return (
            StatusCode::BAD_REQUEST,
            "Bad Request: invalid or missing Host header",
        )
            .into_response();
    };
    if !normalized_host_is_allowed(&host, &policy.allowed_hosts) {
        return (
            StatusCode::FORBIDDEN,
            "Forbidden: Host header is not allowed",
        )
            .into_response();
    }
    enforce_mcp_body_limit(request, next).await
}

fn payload_too_large() -> Response {
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        "Payload Too Large: POST /mcp request bodies are limited to 65,536 bytes",
    )
        .into_response()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum McpBodyReadError {
    TooLarge,
    Read,
}

async fn collect_mcp_body(body: Body) -> Result<Bytes, McpBodyReadError> {
    to_bytes(body, MCP_HTTP_BODY_LIMIT).await.map_err(|error| {
        if std::error::Error::source(&error)
            .is_some_and(|source| source.is::<http_body_util::LengthLimitError>())
        {
            McpBodyReadError::TooLarge
        } else {
            McpBodyReadError::Read
        }
    })
}

fn mcp_body_error_response(error: McpBodyReadError) -> Response {
    match error {
        McpBodyReadError::TooLarge => payload_too_large(),
        McpBodyReadError::Read => (
            StatusCode::BAD_REQUEST,
            "Bad Request: failed to read POST /mcp request body",
        )
            .into_response(),
    }
}

async fn enforce_mcp_body_limit(request: Request, next: Next) -> Response {
    if request.method() != Method::POST || request.uri().path() != "/mcp" {
        return next.run(request).await;
    }

    if request
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > MCP_HTTP_BODY_LIMIT as u64)
    {
        return payload_too_large();
    }

    let (parts, body) = request.into_parts();
    match collect_mcp_body(body).await {
        Ok(bytes) => {
            next.run(Request::from_parts(parts, Body::from(bytes)))
                .await
        }
        Err(error) => mcp_body_error_response(error),
    }
}

pub(in crate::mcp) async fn run_http(
    host: &str,
    port: u16,
    allowed_hosts: Vec<String>,
    unsafe_allow_any_host: bool,
) -> anyhow::Result<()> {
    let ip: std::net::IpAddr = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid host address: {e}"))?;
    let bind = std::net::SocketAddr::new(ip, port);
    let allowed_hosts = http_allowed_hosts(ip, allowed_hosts, unsafe_allow_any_host)?;
    let shutdown = CancellationToken::new();

    #[allow(clippy::field_reassign_with_default)]
    let http_config = {
        let mut http_config = StreamableHttpServerConfig::default();
        http_config.stateful_mode = true;
        http_config.cancellation_token = shutdown.child_token();
        http_config.allowed_hosts = allowed_hosts.transport_hosts();
        http_config
    };

    let service: StreamableHttpService<BioMcpServer, LocalSessionManager> =
        StreamableHttpService::new(|| Ok(BioMcpServer::new()), Default::default(), http_config);

    let router = Router::new()
        .route_service("/mcp", service)
        .route("/health", get(health_handler))
        .route("/readyz", get(health_handler))
        .route("/", get(index_handler))
        .layer(from_fn_with_state(allowed_hosts, enforce_host_policy));
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind HTTP server: {e}"))?;

    tracing::info!("BioMCP Streamable HTTP server listening on http://{bind}");
    if unsafe_allow_any_host {
        tracing::warn!(
            "Host header checks are disabled; this does not provide authentication or encryption"
        );
    }
    tracing::info!("  MCP endpoint:   POST/GET http://{bind}/mcp");
    tracing::info!("  Health probe:   GET      http://{bind}/health");
    tracing::info!("  Ready probe:    GET      http://{bind}/readyz");
    tracing::info!("  Status:         GET      http://{bind}/");

    let cancel = shutdown.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            cancel.cancel();
        }
    });

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            shutdown.cancelled_owned().await;
        })
        .await
        .map_err(|e| anyhow::anyhow!("HTTP server exited: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        MCP_HTTP_BODY_LIMIT, McpBodyReadError, collect_mcp_body, http_allowed_hosts, index_handler,
        mcp_body_error_response, normalized_authority, normalized_host_is_allowed,
    };
    use axum::{
        Json,
        body::{Body, Bytes},
        http::StatusCode,
    };
    use futures::stream;

    #[test]
    fn loopback_http_defaults_to_local_host_headers() {
        let hosts = http_allowed_hosts("127.0.0.1".parse().unwrap(), vec![], false).unwrap();
        assert_eq!(hosts, ["localhost", "127.0.0.1", "::1"]);
    }

    #[test]
    fn non_loopback_http_requires_an_explicit_policy() {
        let error = http_allowed_hosts("0.0.0.0".parse().unwrap(), vec![], false).unwrap_err();
        assert!(error.to_string().contains("--allowed-hosts"));
        assert!(error.to_string().contains("--unsafe-allow-any-host"));

        assert!(
            http_allowed_hosts("0.0.0.0".parse().unwrap(), vec![], true)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            http_allowed_hosts(
                "0.0.0.0".parse().unwrap(),
                vec!["api.example".into()],
                false
            )
            .unwrap(),
            ["api.example"]
        );
    }

    #[test]
    fn global_host_policy_matches_names_ports_case_and_ipv6() {
        let policy = http_allowed_hosts(
            "127.0.0.1".parse().unwrap(),
            vec![
                "Example.COM.:8443".into(),
                "[::1]:8080".into(),
                "example.com:8443".into(),
            ],
            false,
        )
        .unwrap();
        assert_eq!(policy.transport_hosts(), ["example.com:8443", "[::1]:8080"]);
        assert!(normalized_host_is_allowed(
            &normalized_authority("example.com:8443").unwrap(),
            &policy.allowed_hosts
        ));
        assert!(!normalized_host_is_allowed(
            &normalized_authority("example.com:8080").unwrap(),
            &policy.allowed_hosts
        ));
        assert!(normalized_host_is_allowed(
            &normalized_authority("[::1]:8080").unwrap(),
            &policy.allowed_hosts
        ));
    }

    #[test]
    fn explicit_http_policy_rejects_invalid_entries_before_listening() {
        for host in [
            "",
            "bad host",
            "https://example.com",
            "example.com/path",
            "*.example.com",
            "256.256.256.256",
            "example.com:0",
            "example.com:65536",
        ] {
            let error = http_allowed_hosts("127.0.0.1".parse().unwrap(), vec![host.into()], false)
                .unwrap_err();
            assert!(error.to_string().contains("Invalid --allowed-hosts entry"));
            assert!(error.to_string().contains(host));
        }
    }

    #[tokio::test]
    async fn index_handler_reports_streamable_http_surface() {
        let Json(payload) = index_handler().await;
        let identity = crate::build_identity::current();
        assert_eq!(payload["name"], "biomcp");
        assert_eq!(payload["version"], identity.version);
        assert_eq!(payload["git_revision"], identity.git_revision);
        assert_eq!(payload["build_timestamp"], identity.build_date);
        assert_eq!(payload["transport"], "streamable-http");
        assert_eq!(payload["mcp"], "/mcp");
    }

    #[tokio::test]
    async fn body_exhaustion_and_stream_errors_have_distinct_statuses() {
        let too_large = collect_mcp_body(Body::from(vec![0; MCP_HTTP_BODY_LIMIT + 1]))
            .await
            .expect_err("body above limit should fail");
        assert_eq!(too_large, McpBodyReadError::TooLarge);
        assert_eq!(
            mcp_body_error_response(too_large).status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );

        let failed_stream = stream::once(async {
            Err::<Bytes, std::io::Error>(std::io::Error::other("synthetic stream failure"))
        });
        let read_error = collect_mcp_body(Body::from_stream(failed_stream))
            .await
            .expect_err("synthetic stream error should fail");
        assert_eq!(read_error, McpBodyReadError::Read);
        assert_eq!(
            mcp_body_error_response(read_error).status(),
            StatusCode::BAD_REQUEST
        );
    }
}
