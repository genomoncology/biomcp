use std::borrow::Cow;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use http::Extensions;
use reqwest::Url;
use reqwest_middleware::{Middleware, Next};
use tokio::sync::Mutex;
use tokio::time::{Instant, sleep_until};

#[derive(Clone, Debug)]
pub(crate) struct RateLimitPolicy {
    pub key: &'static str,
    pub prefix: Cow<'static, str>,
    pub min_interval: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UnpacedOrigin {
    scheme: String,
    host: IpAddr,
    port: u16,
}

impl UnpacedOrigin {
    fn parse_signal(raw: &str) -> Option<Self> {
        if raw.contains('\\')
            || raw
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return None;
        }
        let (_, authority_and_suffix) = raw.split_once("://")?;
        let suffix_start = authority_and_suffix
            .find(['/', '?', '#'])
            .unwrap_or(authority_and_suffix.len());
        let (authority, suffix) = authority_and_suffix.split_at(suffix_start);
        if authority.contains('@') || !matches!(suffix, "" | "/") {
            return None;
        }

        let url = Url::parse(raw).ok()?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return None;
        }

        let host = url.host_str()?.trim_matches(['[', ']']).parse().ok()?;
        if !IpAddr::is_loopback(&host) {
            return None;
        }

        Some(Self {
            scheme: url.scheme().to_string(),
            host,
            port: url.port_or_known_default()?,
        })
    }

    fn matches(&self, url: &Url) -> bool {
        let Some(host) = url.host_str().and_then(|host| {
            host.trim_matches(['[', ']'].as_ref())
                .parse::<IpAddr>()
                .ok()
        }) else {
            return false;
        };
        self.scheme == url.scheme()
            && self.host == host
            && Some(self.port) == url.port_or_known_default()
    }
}

#[derive(Debug)]
pub(crate) struct RateLimiter {
    policies: Vec<RateLimitPolicy>,
    default_min_interval: Duration,
    unpaced_origin: Option<UnpacedOrigin>,
    last_seen: Mutex<HashMap<String, Instant>>,
}

impl RateLimiter {
    pub(crate) fn from_env() -> Self {
        // NCBI_API_KEY enables the higher PubTator request budget (10 req/sec).
        let has_ncbi_api_key = crate::sources::ncbi_api_key().is_some();
        let has_s2_api_key = crate::sources::s2_api_key().is_some();
        let policies = vec![
            policy(
                "pubtator",
                "BIOMCP_PUBTATOR_BASE",
                "https://www.ncbi.nlm.nih.gov/research/pubtator3-api",
                pubtator_min_interval(has_ncbi_api_key),
            ),
            policy(
                "pmc-oa",
                "BIOMCP_PMC_OA_BASE",
                "https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi",
                Duration::from_millis(334),
            ),
            policy(
                "pubmed-eutils",
                "BIOMCP_PUBMED_BASE",
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils",
                pubmed_eutils_min_interval(has_ncbi_api_key),
            ),
            policy(
                "litsense2",
                "BIOMCP_LITSENSE2_BASE",
                "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api",
                Duration::from_secs(1),
            ),
            policy(
                "ncbi-idconv",
                "BIOMCP_NCBI_IDCONV_BASE",
                "https://pmc.ncbi.nlm.nih.gov/tools/idconv/api/v1/articles",
                Duration::from_millis(334),
            ),
            policy(
                "nih-reporter",
                "BIOMCP_NIH_REPORTER_BASE",
                "https://api.reporter.nih.gov/v2",
                Duration::from_secs(1),
            ),
            policy(
                "opentargets",
                "BIOMCP_OPENTARGETS_BASE",
                "https://api.platform.opentargets.org/api/v4",
                Duration::from_millis(500),
            ),
            policy(
                "civic",
                "BIOMCP_CIVIC_BASE",
                "https://civicdb.org/api",
                Duration::from_millis(334),
            ),
            policy(
                "cpic",
                "BIOMCP_CPIC_BASE",
                "https://api.cpicpgx.org/v1",
                Duration::from_millis(250),
            ),
            policy(
                "pharmgkb",
                "BIOMCP_PHARMGKB_BASE",
                "https://api.pharmgkb.org/v1",
                Duration::from_millis(500),
            ),
            policy(
                "semantic-scholar",
                "BIOMCP_S2_BASE",
                "https://api.semanticscholar.org",
                s2_min_interval(has_s2_api_key),
            ),
            policy(
                "kegg",
                "BIOMCP_KEGG_BASE",
                "https://rest.kegg.jp",
                Duration::from_millis(334),
            ),
        ];
        let unpaced_origin = std::env::var("BIOMCP_TEST_UNPACED_ORIGIN")
            .ok()
            .and_then(|raw| UnpacedOrigin::parse_signal(&raw));
        Self::new(policies, Duration::from_millis(100), unpaced_origin)
    }

    fn new(
        policies: Vec<RateLimitPolicy>,
        default_min_interval: Duration,
        unpaced_origin: Option<UnpacedOrigin>,
    ) -> Self {
        Self {
            policies,
            default_min_interval,
            unpaced_origin,
            last_seen: Mutex::new(HashMap::new()),
        }
    }

    fn resolve_key_and_interval(&self, url: &Url) -> (String, Duration) {
        let full = url.as_str();

        if let Some(policy) = self
            .policies
            .iter()
            .filter(|p| full.starts_with(p.prefix.as_ref()))
            .max_by_key(|p| p.prefix.len())
        {
            return (format!("policy:{}", policy.key), policy.min_interval);
        }

        let origin = format!(
            "{}://{}",
            url.scheme(),
            url.host_str().unwrap_or("unknown-host")
        );
        (format!("default:{origin}"), self.default_min_interval)
    }

    pub(crate) async fn wait_for_url(&self, url: &Url) {
        if self
            .unpaced_origin
            .as_ref()
            .is_some_and(|origin| origin.matches(url))
        {
            return;
        }

        let (key, min_interval) = self.resolve_key_and_interval(url);
        loop {
            let now = Instant::now();
            let mut map = self.last_seen.lock().await;
            let wait_until = map.get(&key).map(|last| *last + min_interval);

            match wait_until {
                Some(target) if target > now => {
                    drop(map);
                    sleep_until(target).await;
                }
                _ => {
                    map.insert(key, now);
                    return;
                }
            }
        }
    }

    #[cfg(test)]
    fn resolve_key_for_str(&self, raw: &str) -> Option<String> {
        let url = Url::parse(raw).ok()?;
        Some(self.resolve_key_and_interval(&url).0)
    }
}

fn pubtator_min_interval(has_ncbi_api_key: bool) -> Duration {
    if has_ncbi_api_key {
        Duration::from_millis(100)
    } else {
        Duration::from_millis(334)
    }
}

fn pubmed_eutils_min_interval(has_ncbi_api_key: bool) -> Duration {
    if has_ncbi_api_key {
        Duration::from_millis(100)
    } else {
        Duration::from_millis(334)
    }
}

fn s2_min_interval(has_s2_api_key: bool) -> Duration {
    if has_s2_api_key {
        Duration::from_secs(1)
    } else {
        Duration::from_secs(2)
    }
}

fn policy(
    key: &'static str,
    env_var: &'static str,
    default_prefix: &'static str,
    min_interval: Duration,
) -> RateLimitPolicy {
    RateLimitPolicy {
        key,
        prefix: crate::sources::env_base(default_prefix, env_var),
        min_interval,
    }
}

static GLOBAL_RATE_LIMITER: OnceLock<Arc<RateLimiter>> = OnceLock::new();

pub(crate) fn global_limiter() -> Arc<RateLimiter> {
    GLOBAL_RATE_LIMITER
        .get_or_init(|| Arc::new(RateLimiter::from_env()))
        .clone()
}

#[derive(Clone, Debug)]
pub(crate) struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub(crate) fn new() -> Self {
        Self {
            limiter: global_limiter(),
        }
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn handle(
        &self,
        req: reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<reqwest::Response> {
        self.limiter.wait_for_url(req.url()).await;
        next.run(req, extensions).await
    }
}

pub(crate) async fn wait_for_url_str(raw: &str) {
    if let Ok(url) = Url::parse(raw) {
        global_limiter().wait_for_url(&url).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy(key: &'static str, prefix: &str, ms: u64) -> RateLimitPolicy {
        RateLimitPolicy {
            key,
            prefix: Cow::Owned(prefix.to_string()),
            min_interval: Duration::from_millis(ms),
        }
    }

    #[test]
    fn unpaced_origin_accepts_only_complete_ip_loopback_origins() {
        assert!(UnpacedOrigin::parse_signal("http://127.0.0.1:8123").is_some());
        assert!(UnpacedOrigin::parse_signal("http://[::1]:8123").is_some());

        for raw in [
            "not a url",
            r"http:\127.0.0.1:8123",
            "http://localhost:8123",
            "http://192.0.2.1:8123",
            "http://@127.0.0.1:8123",
            "http://user@127.0.0.1:8123",
            "http://user:pass@127.0.0.1:8123",
            "http://127.0.0.1:8123/path",
            "http://127.0.0.1:8123/foo/..",
            "http://127.0.0.1:8123/%2e%2e",
            "http://127.0.0.1:8123?query=yes",
            "http://127.0.0.1:8123#fragment",
        ] {
            assert!(
                UnpacedOrigin::parse_signal(raw).is_none(),
                "signal should fail closed: {raw}"
            );
        }
    }

    #[tokio::test]
    async fn exact_unpaced_origin_skips_waiting_and_state_updates() {
        let limiter = RateLimiter::new(
            Vec::new(),
            Duration::from_millis(100),
            UnpacedOrigin::parse_signal("http://127.0.0.1:8123"),
        );
        let url = Url::parse("http://127.0.0.1:8123/resource").unwrap();

        limiter.wait_for_url(&url).await;
        limiter.wait_for_url(&url).await;

        assert!(limiter.last_seen.lock().await.is_empty());
    }

    #[tokio::test]
    async fn unpaced_origin_keeps_other_ports_paced() {
        let limiter = RateLimiter::new(
            Vec::new(),
            Duration::from_millis(80),
            UnpacedOrigin::parse_signal("http://127.0.0.1:8123"),
        );
        let url = Url::parse("http://127.0.0.1:8124/resource").unwrap();
        let start = Instant::now();

        limiter.wait_for_url(&url).await;
        limiter.wait_for_url(&url).await;

        assert!(start.elapsed() >= Duration::from_millis(65));
    }

    #[tokio::test]
    async fn unsignaled_loopback_origin_remains_paced() {
        let limiter = RateLimiter::new(Vec::new(), Duration::from_millis(80), None);
        let url = Url::parse("http://127.0.0.1:8123/resource").unwrap();
        let start = Instant::now();

        limiter.wait_for_url(&url).await;
        limiter.wait_for_url(&url).await;

        assert!(start.elapsed() >= Duration::from_millis(65));
    }

    #[tokio::test]
    async fn rate_limit_blocks_second_request_for_same_prefix() {
        let limiter = RateLimiter::new(
            vec![test_policy("strict", "https://api.example.org/strict", 120)],
            Duration::from_millis(1),
            None,
        );

        let url = Url::parse("https://api.example.org/strict/resource").unwrap();
        let start = Instant::now();
        limiter.wait_for_url(&url).await;
        limiter.wait_for_url(&url).await;

        assert!(
            start.elapsed() >= Duration::from_millis(100),
            "second request should be throttled for strict prefix"
        );
    }

    #[tokio::test]
    async fn rate_limit_keeps_same_host_prefixes_independent() {
        let limiter = RateLimiter::new(
            vec![
                test_policy("a", "https://www.ebi.ac.uk/europepmc/webservices/rest", 100),
                test_policy("b", "https://www.ebi.ac.uk/chembl/api/data", 100),
            ],
            Duration::from_millis(1),
            None,
        );

        let url_a = Url::parse("https://www.ebi.ac.uk/europepmc/webservices/rest/search").unwrap();
        let url_b = Url::parse("https://www.ebi.ac.uk/chembl/api/data/molecule").unwrap();

        let start = Instant::now();
        limiter.wait_for_url(&url_a).await;
        limiter.wait_for_url(&url_b).await;

        assert!(
            start.elapsed() < Duration::from_millis(80),
            "same host, different prefixes should not block each other"
        );
    }

    #[tokio::test]
    async fn rate_limit_uses_default_policy_for_unknown_prefix() {
        let limiter = RateLimiter::new(Vec::new(), Duration::from_millis(80), None);
        let url = Url::parse("https://unknown.example.org/path").unwrap();

        let start = Instant::now();
        limiter.wait_for_url(&url).await;
        limiter.wait_for_url(&url).await;

        assert!(
            start.elapsed() >= Duration::from_millis(65),
            "default policy should throttle unknown prefixes"
        );
    }

    #[test]
    fn rate_limit_uses_longest_matching_prefix() {
        let limiter = RateLimiter::new(
            vec![
                test_policy("short", "https://example.org/api", 10),
                test_policy("long", "https://example.org/api/v1", 10),
            ],
            Duration::from_millis(1),
            None,
        );

        let key = limiter
            .resolve_key_for_str("https://example.org/api/v1/resource")
            .unwrap();
        assert_eq!(key, "policy:long");
    }

    #[test]
    fn pubtator_interval_uses_key_aware_values() {
        assert_eq!(pubtator_min_interval(false), Duration::from_millis(334));
        assert_eq!(pubtator_min_interval(true), Duration::from_millis(100));
    }

    #[test]
    fn pubmed_eutils_interval_uses_key_aware_values() {
        assert_eq!(
            pubmed_eutils_min_interval(false),
            Duration::from_millis(334)
        );
        assert_eq!(pubmed_eutils_min_interval(true), Duration::from_millis(100));
    }

    #[test]
    fn semantic_scholar_interval_uses_key_aware_values() {
        assert_eq!(s2_min_interval(false), Duration::from_secs(2));
        assert_eq!(s2_min_interval(true), Duration::from_secs(1));
    }

    #[test]
    fn semantic_scholar_urls_resolve_to_semantic_scholar_policy() {
        let limiter = RateLimiter::from_env();
        let key = limiter
            .resolve_key_for_str("https://api.semanticscholar.org/graph/v1/paper/PMID%3A22663011")
            .expect("semantic scholar URL should parse");
        assert_eq!(key, "policy:semantic-scholar");
    }

    #[test]
    fn kegg_urls_resolve_to_kegg_policy() {
        let limiter = RateLimiter::from_env();
        let key = limiter
            .resolve_key_for_str("https://rest.kegg.jp/find/pathway/MAPK")
            .expect("kegg URL should parse");
        assert_eq!(key, "policy:kegg");
    }

    #[test]
    fn pubmed_eutils_urls_resolve_to_pubmed_policy() {
        let limiter = RateLimiter::from_env();
        let key = limiter
            .resolve_key_for_str(
                "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term=BRAF",
            )
            .expect("pubmed E-utilities URL should parse");
        assert_eq!(key, "policy:pubmed-eutils");
    }

    #[test]
    fn nih_reporter_policy_uses_one_second_interval() {
        let limiter = RateLimiter::from_env();
        let policy = limiter
            .policies
            .iter()
            .find(|policy| policy.key == "nih-reporter")
            .expect("nih-reporter policy should be registered");
        assert_eq!(policy.min_interval, Duration::from_secs(1));
        assert_eq!(policy.prefix.as_ref(), "https://api.reporter.nih.gov/v2");
    }

    #[test]
    fn nih_reporter_urls_resolve_to_nih_reporter_policy() {
        let limiter = RateLimiter::from_env();
        let key = limiter
            .resolve_key_for_str("https://api.reporter.nih.gov/v2/projects/search")
            .expect("NIH Reporter URL should parse");
        assert_eq!(key, "policy:nih-reporter");
    }

    #[test]
    fn litsense2_policy_uses_one_second_interval() {
        let limiter = RateLimiter::from_env();
        let policy = limiter
            .policies
            .iter()
            .find(|policy| policy.key == "litsense2")
            .expect("litsense2 policy should be registered");
        assert_eq!(policy.min_interval, Duration::from_secs(1));
        assert_eq!(
            policy.prefix.as_ref(),
            "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api"
        );
    }

    #[test]
    fn litsense2_urls_resolve_to_litsense2_policy() {
        let limiter = RateLimiter::from_env();
        let key = limiter
            .resolve_key_for_str(
                "https://www.ncbi.nlm.nih.gov/research/litsense2-api/api/sentences/?query=test&rerank=true",
            )
            .expect("litsense2 URL should parse");
        assert_eq!(key, "policy:litsense2");
    }
}
