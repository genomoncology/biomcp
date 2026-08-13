//! Direct-network policy for ordinary provider URLs owned by BioMCP.

use std::net::IpAddr;
use std::sync::Arc;

use http::Extensions;
use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use reqwest_middleware::{ClientBuilder, Middleware, Next};

use super::provider_url_policy::{ProviderUrlPolicy, is_forbidden_address};
use crate::error::BioMcpError;

const MAX_REDIRECTS: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Origin {
    scheme: String,
    host: String,
    port: u16,
}

impl Origin {
    fn from_url(url: &Url) -> Option<Self> {
        Some(Self {
            scheme: url.scheme().to_ascii_lowercase(),
            host: url.host_str()?.trim_end_matches('.').to_ascii_lowercase(),
            port: url.port_or_known_default()?,
        })
    }
}

/// Built-in endpoints must use public HTTPS. Explicit BioMCP provider-base
/// settings deliberately trust their exact origin for fixtures and on-prem use.
#[derive(Clone, Debug, Default)]
pub(crate) struct OrdinaryProviderPolicy {
    bound_origin: Option<Origin>,
    trusted_private_origin: Option<Origin>,
}

tokio::task_local! {
    static REQUEST_PRIVATE_HOST: Option<String>;
}

impl OrdinaryProviderPolicy {
    fn validate_url(&self, url: &Url) -> Result<Option<String>, BioMcpError> {
        if let Some(bound) = self.bound_origin.as_ref()
            && Origin::from_url(url).as_ref() != Some(bound)
        {
            return Err(policy_error("origin differs from the bound provider base"));
        }
        if self
            .trusted_private_origin
            .as_ref()
            .is_some_and(|origin| Origin::from_url(url).as_ref() == Some(origin))
        {
            if !url.username().is_empty() || url.password().is_some() {
                return Err(policy_error("credentials in URL are forbidden"));
            }
            return Ok(url.host_str().map(str::to_ascii_lowercase));
        }
        self.validate_url_with_overrides(url, configured_provider_override_values())
    }

    fn validate_url_with_overrides<S: AsRef<str>>(
        &self,
        url: &Url,
        overrides: impl IntoIterator<Item = S>,
    ) -> Result<Option<String>, BioMcpError> {
        if !url.username().is_empty() || url.password().is_some() {
            return Err(policy_error("credentials in URL are forbidden"));
        }
        if !matches!(url.scheme(), "http" | "https") {
            return Err(policy_error("unsupported URL scheme"));
        }

        let selected =
            Origin::from_url(url).ok_or_else(|| policy_error("URL has no valid host or port"))?;
        let explicitly_overridden = overrides
            .into_iter()
            .filter_map(|raw| configured_override_origin(raw.as_ref()))
            .any(|configured| configured == selected);
        if explicitly_overridden {
            return Ok(Some(selected.host));
        }
        if url.scheme() != "https" {
            return Err(policy_error("non-HTTPS scheme is forbidden"));
        }
        if url
            .host_str()
            .and_then(parse_ip_literal)
            .is_some_and(is_forbidden_address)
        {
            return Err(policy_error("destination address is forbidden"));
        }
        Ok(None)
    }

    fn redirect_policy(&self) -> reqwest::redirect::Policy {
        let policy = self.clone();
        reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error("ordinary provider policy rejected too many redirects");
            }
            if policy.validate_url(attempt.url()).is_err()
                || !redirect_target_is_allowed(attempt.url(), attempt.previous())
            {
                return attempt.error("ordinary provider policy rejected redirect target");
            }
            attempt.follow()
        })
    }

    fn dns_resolver(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }
}

impl Resolve for OrdinaryProviderPolicy {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().trim_end_matches('.').to_ascii_lowercase();
        let bound_trusts_host = self
            .trusted_private_origin
            .as_ref()
            .is_some_and(|origin| origin.host == host);
        Box::pin(async move {
            let resolved = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|_| resolver_error())?
                .collect::<Vec<_>>();
            if resolved.is_empty() {
                return Err(resolver_error());
            }
            let request_trusts_host = REQUEST_PRIVATE_HOST
                .try_with(|trusted| trusted.as_deref() == Some(host.as_str()))
                .unwrap_or(false);
            if !request_trusts_host
                && !bound_trusts_host
                && resolved
                    .iter()
                    .any(|address| is_forbidden_address(address.ip()))
            {
                return Err(resolver_error());
            }
            let addrs: Addrs = Box::new(resolved.into_iter());
            Ok(addrs)
        })
    }
}

#[derive(Clone, Debug, Default)]
struct OrdinaryProviderPolicyMiddleware;

#[async_trait::async_trait]
impl Middleware for OrdinaryProviderPolicyMiddleware {
    async fn handle(
        &self,
        request: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        let trusted_host = OrdinaryProviderPolicy::default()
            .validate_url(request.url())
            .map_err(reqwest_middleware::Error::middleware)?;
        REQUEST_PRIVATE_HOST
            .scope(trusted_host, next.run(request, extensions))
            .await
    }
}

pub(crate) fn with_initial_policy(
    builder: ClientBuilder,
    provider_policy: Option<&ProviderUrlPolicy>,
) -> ClientBuilder {
    if provider_policy.is_none() {
        builder.with(OrdinaryProviderPolicyMiddleware)
    } else {
        builder
    }
}

pub(crate) fn http_client_builder(
    provider_policy: Option<&ProviderUrlPolicy>,
) -> reqwest::ClientBuilder {
    match provider_policy {
        Some(policy) => provider_policy_client_builder(policy),
        None => ordinary_http_client_builder(),
    }
}

pub(crate) fn ordinary_http_client_builder() -> reqwest::ClientBuilder {
    let policy = OrdinaryProviderPolicy::default();
    reqwest::Client::builder()
        .no_proxy()
        .dns_resolver(policy.dns_resolver())
        .redirect(policy.redirect_policy())
}

pub(crate) fn ordinary_middleware_client_for_base<F>(
    base: &str,
    env_var: &str,
    configure: F,
) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError>
where
    F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
{
    let explicitly_configured = std::env::var(env_var)
        .ok()
        .filter(|value| !value.trim().is_empty());
    middleware_client_for_base(base, explicitly_configured.as_deref(), configure)
}

#[cfg(test)]
pub(crate) fn test_middleware_client_for_base<F>(
    base: &str,
    configure: F,
) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError>
where
    F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
{
    middleware_client_for_base(base, Some(base), configure)
}

fn middleware_client_for_base<F>(
    base: &str,
    explicitly_configured: Option<&str>,
    configure: F,
) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError>
where
    F: FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
{
    let url = Url::parse(base).map_err(|_| policy_error("provider base URL is invalid"))?;
    let origin = Origin::from_url(&url)
        .ok_or_else(|| policy_error("provider base URL has no valid origin"))?;
    let trusted_private_origin = explicitly_configured
        .and_then(configured_override_origin)
        .filter(|configured| configured == &origin);
    let policy = OrdinaryProviderPolicy {
        bound_origin: Some(origin),
        trusted_private_origin,
    };
    if explicitly_configured.is_some() {
        policy.validate_url_with_overrides(&url, [base])?;
    } else {
        policy.validate_url(&url)?;
    }

    let client = configure(
        reqwest::Client::builder()
            .no_proxy()
            .dns_resolver(policy.dns_resolver())
            .redirect(policy.redirect_policy()),
    )
    .build()
    .map_err(BioMcpError::HttpClientInit)?;
    Ok(ClientBuilder::new(client)
        .with(BoundProviderPolicyMiddleware(policy))
        .build())
}

#[derive(Clone, Debug)]
struct BoundProviderPolicyMiddleware(OrdinaryProviderPolicy);

#[async_trait::async_trait]
impl Middleware for BoundProviderPolicyMiddleware {
    async fn handle(
        &self,
        request: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        self.0
            .validate_url(request.url())
            .map_err(reqwest_middleware::Error::middleware)?;
        next.run(request, extensions).await
    }
}

pub(crate) fn provider_policy_client_builder(policy: &ProviderUrlPolicy) -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .no_proxy()
        .dns_resolver(policy.dns_resolver())
        .redirect(policy.redirect_policy())
}

fn redirect_target_is_allowed(target: &Url, previous: &[Url]) -> bool {
    let Some(previous) = previous.last() else {
        return false;
    };
    Origin::from_url(target) == Origin::from_url(previous)
}

fn configured_provider_override_values() -> impl Iterator<Item = String> {
    std::env::vars().filter_map(|(name, value)| is_provider_override_name(&name).then_some(value))
}

fn is_provider_override_name(name: &str) -> bool {
    name.starts_with("BIOMCP_")
        && (name.ends_with("_BASE")
            || name.ends_with("_BASE_URL")
            || name.ends_with("_FIXTURE_ORIGIN")
            || name.ends_with("_URL")
            || name == "BIOMCP_TEST_UNPACED_ORIGIN")
}

fn configured_override_origin(raw: &str) -> Option<Origin> {
    let url = Url::parse(raw.trim()).ok()?;
    (matches!(url.scheme(), "http" | "https")
        && url.username().is_empty()
        && url.password().is_none())
    .then(|| Origin::from_url(&url))?
}

fn parse_ip_literal(host: &str) -> Option<IpAddr> {
    host.trim_matches(['[', ']']).parse().ok()
}

fn policy_error(class: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "provider-network-policy".to_string(),
        message: format!("provider source unavailable: outbound policy rejected {class}"),
    }
}

fn resolver_error() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "provider URL DNS policy rejected destination",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_public_https_unless_origin_is_explicitly_overridden() {
        let policy = OrdinaryProviderPolicy::default();
        assert!(
            policy
                .validate_url_with_overrides(
                    &Url::parse("https://api.example.test/data").unwrap(),
                    std::iter::empty::<&str>(),
                )
                .is_ok()
        );
        for raw in [
            "http://api.example.test/data",
            "https://127.0.0.1/private",
            "https://10.0.0.1/private",
            "https://169.254.169.254/latest/meta-data",
            "https://user:secret@api.example.test/data",
        ] {
            assert!(
                policy
                    .validate_url_with_overrides(
                        &Url::parse(raw).unwrap(),
                        std::iter::empty::<&str>(),
                    )
                    .is_err(),
                "accepted {raw}",
            );
        }

        let local = Url::parse("http://127.0.0.1:8123/v1/query").unwrap();
        assert!(
            policy
                .validate_url_with_overrides(&local, ["http://127.0.0.1:8123/base"])
                .is_ok()
        );
        assert!(
            policy
                .validate_url_with_overrides(&local, ["http://127.0.0.1:8124/base"])
                .is_err()
        );
    }

    #[test]
    fn confines_redirects_to_the_request_origin() {
        let start = Url::parse("https://api.example.test/start").unwrap();
        let same = Url::parse("https://api.example.test/final").unwrap();
        let other = Url::parse("https://cdn.example.test/final").unwrap();
        assert!(redirect_target_is_allowed(
            &same,
            std::slice::from_ref(&start)
        ));
        assert!(!redirect_target_is_allowed(&other, &[start]));
    }

    #[test]
    fn override_origin_requires_valid_http_without_credentials() {
        assert_eq!(
            configured_override_origin("http://fixture.internal:8123/base"),
            Some(Origin {
                scheme: "http".into(),
                host: "fixture.internal".into(),
                port: 8123,
            })
        );
        assert!(configured_override_origin("ftp://fixture.internal/private").is_none());
        assert!(
            configured_override_origin("http://user:secret@fixture.internal/private").is_none()
        );
    }

    #[test]
    fn recognizes_base_and_full_url_provider_overrides() {
        for name in [
            "BIOMCP_GPROFILER_BASE",
            "BIOMCP_MUTALYZER_BASE_URL",
            "BIOMCP_CSPEC_FIXTURE_ORIGIN",
            "BIOMCP_CVX_URL",
            "BIOMCP_GTR_TEST_VERSION_URL",
            "BIOMCP_TEST_UNPACED_ORIGIN",
        ] {
            assert!(is_provider_override_name(name), "missed {name}");
        }
        assert!(!is_provider_override_name("HTTP_PROXY"));
        assert!(!is_provider_override_name("BIOMCP_CACHE_MODE"));
    }
}
