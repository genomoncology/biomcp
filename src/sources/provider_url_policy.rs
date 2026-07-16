//! Shared policy for outbound URLs supplied by upstream providers.
//!
//! The policy validates the URL before a request is built, validates every redirect
//! synchronously, and validates the DNS answers used by the HTTP connector. Keeping
//! the DNS check in reqwest's resolver avoids a resolve-then-connect rebinding gap.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::error::BioMcpError;

const MAX_REDIRECTS: usize = 10;
const S2_API_ORIGIN: &str = "https://api.semanticscholar.org";
const S2_PDF_ORIGINS: &[&str] = &[
    "https://pdfs.semanticscholar.org",
    "https://www.semanticscholar.org",
];
const PMC_OA_ORIGINS: &[&str] = &[
    "https://www.ncbi.nlm.nih.gov",
    "https://ftp.ncbi.nlm.nih.gov",
];
const FIGSHARE_ORIGINS: &[&str] = &[
    "https://api.figshare.com",
    "https://figshare.com",
    "https://ndownloader.figshare.com",
    "https://s3-eu-west-1.amazonaws.com",
];
const CTGOV_DOCUMENT_ORIGINS: &[&str] = &["https://cdn.clinicaltrials.gov"];

macro_rules! provider_url_consumers {
    ($($consumer:ident),+ $(,)?) => {
        /// Exhaustive inventory of provider-returned URL fetch consumers.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) enum ProviderUrlConsumer {
            $($consumer),+
        }

        impl ProviderUrlConsumer {
            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$consumer),+];
        }
    };
}

provider_url_consumers!(
    SemanticScholarPdf,
    PmcOaArchive,
    FigshareDownload,
    ClinicalTrialsDocument,
);

#[derive(Clone, Debug, Eq, PartialEq)]
struct AllowedOrigin {
    scheme: String,
    host: String,
    port: u16,
}

impl AllowedOrigin {
    fn parse(raw: &str) -> Result<Self, BioMcpError> {
        let url = Url::parse(raw).map_err(|_| policy_error("invalid configured origin"))?;
        Self::from_url(&url).ok_or_else(|| policy_error("invalid configured origin"))
    }

    fn from_url(url: &Url) -> Option<Self> {
        Some(Self {
            scheme: url.scheme().to_ascii_lowercase(),
            host: url.host_str()?.to_ascii_lowercase(),
            port: url.port_or_known_default()?,
        })
    }

    fn matches(&self, url: &Url) -> bool {
        Self::from_url(url).as_ref() == Some(self)
    }
}

/// One reusable owner for provider URL, redirect, DNS, and credential-origin policy.
#[derive(Clone, Debug)]
pub(crate) struct ProviderUrlPolicy {
    source: &'static str,
    allowed_origins: Vec<AllowedOrigin>,
    credential_origins: Vec<AllowedOrigin>,
    unsafe_test_origin: Option<AllowedOrigin>,
}

impl ProviderUrlPolicy {
    /// Policy for Semantic Scholar API calls. A base override is an explicitly selected
    /// request origin, but it is never a credential origin. Local HTTP overrides require
    /// the exact fixture-only `BIOMCP_TEST_UNPACED_ORIGIN` signal.
    pub(crate) fn semantic_scholar_api(base: &Url) -> Result<Self, BioMcpError> {
        let canonical = AllowedOrigin::parse(S2_API_ORIGIN)?;
        let configured = AllowedOrigin::from_url(base)
            .ok_or_else(|| policy_error("Semantic Scholar base has no valid origin"))?;
        let mut allowed_origins = vec![canonical.clone()];
        if !allowed_origins.contains(&configured) {
            allowed_origins.push(configured);
        }
        let policy = Self {
            source: "Semantic Scholar",
            allowed_origins,
            credential_origins: vec![canonical],
            unsafe_test_origin: unsafe_test_origin(),
        };
        policy.validate_url(base)?;
        Ok(policy)
    }

    /// Policy for PDF URLs returned in Semantic Scholar payloads.
    pub(crate) fn semantic_scholar_pdf() -> Result<Self, BioMcpError> {
        Self::for_consumer(ProviderUrlConsumer::SemanticScholarPdf, None)
    }

    /// Policy for one enumerated provider-returned URL consumer. API/CDN base overrides
    /// are selected origins only for fixture-configurable clients. Their exact IP-loopback
    /// origin may use HTTP; production origins remain HTTPS-only.
    pub(crate) fn for_consumer(
        consumer: ProviderUrlConsumer,
        selected_origin: Option<&Url>,
    ) -> Result<Self, BioMcpError> {
        let (source, origins): (&'static str, &[&str]) = match consumer {
            ProviderUrlConsumer::SemanticScholarPdf => ("Semantic Scholar PDF", S2_PDF_ORIGINS),
            ProviderUrlConsumer::PmcOaArchive => ("PMC OA archive", PMC_OA_ORIGINS),
            ProviderUrlConsumer::FigshareDownload => ("Figshare download", FIGSHARE_ORIGINS),
            ProviderUrlConsumer::ClinicalTrialsDocument => {
                ("ClinicalTrials.gov document", CTGOV_DOCUMENT_ORIGINS)
            }
        };
        let mut allowed_origins = origins
            .iter()
            .map(|origin| AllowedOrigin::parse(origin))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(url) = selected_origin {
            let selected = AllowedOrigin::from_url(url)
                .ok_or_else(|| policy_error("selected URL has no valid origin"))?;
            if !allowed_origins.contains(&selected) {
                allowed_origins.push(selected);
            }
        }
        let policy = Self {
            source,
            allowed_origins,
            credential_origins: Vec::new(),
            unsafe_test_origin: unsafe_test_origin()
                .or_else(|| selected_origin.and_then(selected_loopback_test_origin)),
        };
        if let Some(url) = selected_origin {
            policy.validate_url(url)?;
        }
        Ok(policy)
    }

    #[cfg(test)]
    pub(crate) fn test_fixture(
        consumer: ProviderUrlConsumer,
        origin: &Url,
    ) -> Result<Self, BioMcpError> {
        let mut policy = Self::for_consumer(consumer, None)?;
        let fixture = AllowedOrigin::from_url(origin)
            .ok_or_else(|| policy_error("fixture URL has no valid origin"))?;
        if !policy.allowed_origins.contains(&fixture) {
            policy.allowed_origins.push(fixture.clone());
        }
        policy.unsafe_test_origin = Some(fixture);
        Ok(policy)
    }

    pub(crate) fn validate_url(&self, url: &Url) -> Result<(), BioMcpError> {
        if !url.username().is_empty() || url.password().is_some() {
            return Err(self.error("credentials in URL are forbidden"));
        }
        let origin = AllowedOrigin::from_url(url)
            .ok_or_else(|| self.error("URL has no valid host or port"))?;
        let unsafe_fixture = self.unsafe_test_origin.as_ref() == Some(&origin);
        if url.scheme() != "https" && !unsafe_fixture {
            return Err(self.error("non-HTTPS scheme is forbidden"));
        }
        if !self.allowed_origins.contains(&origin) && !unsafe_fixture {
            return Err(self.error("origin is not allowlisted"));
        }
        if !unsafe_fixture && let Some(ip) = url.host_str().and_then(parse_ip_literal) {
            self.validate_addresses(std::iter::once(ip))?;
        }
        Ok(())
    }

    pub(crate) fn is_credential_origin(&self, url: &Url) -> bool {
        self.credential_origins
            .iter()
            .any(|origin| origin.matches(url))
            || self
                .unsafe_test_origin
                .as_ref()
                .is_some_and(|origin| origin.matches(url))
    }

    pub(crate) fn redirect_policy(&self) -> reqwest::redirect::Policy {
        let policy = self.clone();
        reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error("provider URL policy rejected too many redirects");
            }
            match policy.validate_url(attempt.url()) {
                Ok(()) => attempt.follow(),
                Err(_) => attempt.error("provider URL policy rejected redirect target"),
            }
        })
    }

    pub(crate) fn dns_resolver(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    fn validate_addresses(
        &self,
        addresses: impl IntoIterator<Item = IpAddr>,
    ) -> Result<(), BioMcpError> {
        let mut found = false;
        for address in addresses {
            found = true;
            if is_forbidden_address(address) {
                return Err(self.error("DNS resolved to a forbidden address class"));
            }
        }
        if !found {
            return Err(self.error("DNS returned no addresses"));
        }
        Ok(())
    }

    fn error(&self, class: &str) -> BioMcpError {
        BioMcpError::Api {
            api: "provider-url-policy".to_string(),
            message: format!(
                "{} source unavailable: outbound policy rejected {class}",
                self.source
            ),
        }
    }
}

impl Resolve for ProviderUrlPolicy {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        let policy = self.clone();
        Box::pin(async move {
            let resolved = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|_| resolver_error())?
                .collect::<Vec<_>>();
            let unsafe_fixture_host = policy
                .unsafe_test_origin
                .as_ref()
                .is_some_and(|origin| origin.host == host.to_ascii_lowercase());
            if !unsafe_fixture_host {
                policy
                    .validate_addresses(resolved.iter().map(|address| address.ip()))
                    .map_err(|_| resolver_error())?;
            }
            let addrs: Addrs = Box::new(resolved.into_iter());
            Ok(addrs)
        })
    }
}

fn policy_error(class: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "provider-url-policy".to_string(),
        message: format!("provider source unavailable: outbound policy rejected {class}"),
    }
}

fn resolver_error() -> Box<dyn std::error::Error + Send + Sync> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "provider URL DNS policy rejected destination",
    ))
}

fn unsafe_test_origin() -> Option<AllowedOrigin> {
    let raw = std::env::var("BIOMCP_TEST_UNPACED_ORIGIN").ok()?;
    let url = Url::parse(raw.trim()).ok()?;
    let origin = AllowedOrigin::from_url(&url)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url
            .host_str()
            .and_then(parse_ip_literal)
            .is_some_and(|address| address.is_loopback())
    {
        return None;
    }
    Some(origin)
}

fn selected_loopback_test_origin(url: &Url) -> Option<AllowedOrigin> {
    let origin = AllowedOrigin::from_url(url)?;
    (matches!(url.scheme(), "http" | "https")
        && url.username().is_empty()
        && url.password().is_none()
        && url
            .host_str()
            .and_then(parse_ip_literal)
            .is_some_and(|address| address.is_loopback()))
    .then_some(origin)
}

fn parse_ip_literal(host: &str) -> Option<IpAddr> {
    host.trim_matches(['[', ']']).parse().ok()
}

fn is_forbidden_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => forbidden_ipv4(address),
        IpAddr::V6(address) => {
            address.to_ipv4_mapped().is_some_and(forbidden_ipv4) || forbidden_ipv6(address)
        }
    }
}

fn forbidden_ipv4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    address.is_loopback()
        || address.is_private()
        || address.is_link_local()
        || address.is_unspecified()
        || address.is_multicast()
        || octets[0] == 0
        || octets == [100, 100, 100, 200]
}

fn forbidden_ipv6(address: Ipv6Addr) -> bool {
    let first = address.segments()[0];
    address.is_loopback()
        || address.is_unspecified()
        || address.is_multicast()
        || (first & 0xfe00) == 0xfc00
        || (first & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf_policy() -> ProviderUrlPolicy {
        ProviderUrlPolicy {
            source: "test provider",
            allowed_origins: vec![
                AllowedOrigin::parse("https://pdfs.semanticscholar.org").unwrap(),
            ],
            credential_origins: Vec::new(),
            unsafe_test_origin: None,
        }
    }

    #[test]
    fn rejects_scheme_credentials_port_and_off_origin_without_echoing_url() {
        let policy = pdf_policy();
        for raw in [
            "http://pdfs.semanticscholar.org/paper.pdf",
            "https://user:secret@pdfs.semanticscholar.org/paper.pdf",
            "https://pdfs.semanticscholar.org:444/paper.pdf",
            "https://example.test/private-token/paper.pdf",
        ] {
            let error = policy.validate_url(&Url::parse(raw).unwrap()).unwrap_err();
            let message = error.to_string();
            assert!(message.contains("outbound policy"));
            assert!(!message.contains(raw));
            assert!(!message.contains("secret"));
            assert!(!message.contains("private-token"));
        }
    }

    #[test]
    fn rejects_loopback_private_link_local_metadata_and_mapped_addresses() {
        let policy = pdf_policy();
        for address in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.0.1",
            "169.254.169.254",
            "100.100.100.200",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
        ] {
            let parsed: IpAddr = address.parse().unwrap();
            assert!(
                policy.validate_addresses([parsed]).is_err(),
                "accepted {address}"
            );
        }
    }

    #[test]
    fn consumer_matrix_enumerates_valid_origins_and_shared_rejections() {
        let valid = [
            "https://pdfs.semanticscholar.org/paper.pdf",
            "https://ftp.ncbi.nlm.nih.gov/pub/pmc/archive.tgz",
            "https://ndownloader.figshare.com/files/1",
            "https://cdn.clinicaltrials.gov/large-docs/48/NCT03361748/Protocol.pdf",
        ];
        assert_eq!(ProviderUrlConsumer::ALL.len(), valid.len());

        for (consumer, valid_url) in ProviderUrlConsumer::ALL.iter().copied().zip(valid) {
            let policy = ProviderUrlPolicy::for_consumer(consumer, None).unwrap();
            assert!(policy.validate_url(&Url::parse(valid_url).unwrap()).is_ok());
            for raw in [
                "http://example.com/private",
                "https://127.0.0.1/private",
                "https://10.0.0.1/private",
                "https://169.254.169.254/latest/meta-data",
                "https://example.com:444/private",
            ] {
                let error = policy.validate_url(&Url::parse(raw).unwrap()).unwrap_err();
                let message = error.to_string();
                assert!(message.contains("outbound policy"));
                assert!(!message.contains(raw));
            }
            assert!(
                policy
                    .validate_addresses([
                        "93.184.216.34".parse().unwrap(),
                        "127.0.0.1".parse().unwrap(),
                    ])
                    .is_err()
            );
        }
    }

    #[test]
    fn selected_fixture_origin_allows_only_exact_ip_loopback() {
        let fixture = Url::parse("http://127.0.0.1:43210/api").unwrap();
        let policy = ProviderUrlPolicy::for_consumer(
            ProviderUrlConsumer::ClinicalTrialsDocument,
            Some(&fixture),
        )
        .unwrap();
        assert!(
            policy
                .validate_url(&fixture.join("document.pdf").unwrap())
                .is_ok()
        );
        assert!(
            ProviderUrlPolicy::for_consumer(
                ProviderUrlConsumer::ClinicalTrialsDocument,
                Some(&Url::parse("http://localhost:43210/api").unwrap()),
            )
            .is_err()
        );
    }

    #[test]
    fn provider_url_consumer_ownership_ratchet_names_every_fetch_site() {
        let sites = [
            (
                ProviderUrlConsumer::SemanticScholarPdf,
                include_str!("../entities/article/fulltext.rs"),
                "ProviderUrlPolicy::semantic_scholar_pdf()",
            ),
            (
                ProviderUrlConsumer::PmcOaArchive,
                include_str!("pmc_oa.rs"),
                "ProviderUrlConsumer::PmcOaArchive",
            ),
            (
                ProviderUrlConsumer::FigshareDownload,
                include_str!("figshare.rs"),
                "ProviderUrlConsumer::FigshareDownload",
            ),
            (
                ProviderUrlConsumer::ClinicalTrialsDocument,
                include_str!("../entities/trial/documents.rs"),
                "ProviderUrlConsumer::ClinicalTrialsDocument",
            ),
        ];
        assert_eq!(sites.len(), ProviderUrlConsumer::ALL.len());
        for (consumer, source, policy_marker) in sites {
            assert!(ProviderUrlConsumer::ALL.contains(&consumer));
            assert!(source.contains(policy_marker));
            assert!(source.contains(".get("));
        }
    }

    #[test]
    fn dns_answer_set_is_rejected_if_any_answer_is_forbidden() {
        let policy = pdf_policy();
        assert!(
            policy
                .validate_addresses([
                    "93.184.216.34".parse().unwrap(),
                    "127.0.0.1".parse().unwrap(),
                ])
                .is_err()
        );
        assert!(
            policy
                .validate_addresses(["93.184.216.34".parse().unwrap()])
                .is_ok()
        );
    }

    #[tokio::test]
    async fn redirect_target_is_revalidated_before_target_contact() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let target = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_url = format!("http://{}/stolen", target.local_addr().unwrap());
        let redirect = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect_url =
            Url::parse(&format!("http://{}/start", redirect.local_addr().unwrap())).unwrap();
        let server = tokio::spawn(async move {
            for _ in ProviderUrlConsumer::ALL {
                let (mut stream, _) = redirect.accept().await.unwrap();
                let mut request = [0_u8; 1024];
                let _ = stream.read(&mut request).await.unwrap();
                let response = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {target_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        for consumer in ProviderUrlConsumer::ALL.iter().copied() {
            let policy = ProviderUrlPolicy::test_fixture(consumer, &redirect_url).unwrap();
            let client = reqwest::Client::builder()
                .no_proxy()
                .dns_resolver(policy.dns_resolver())
                .redirect(policy.redirect_policy())
                .build()
                .unwrap();
            let error = client
                .get(redirect_url.clone())
                .send()
                .await
                .expect_err("off-origin redirect must fail");
            assert!(error.is_redirect());
        }
        server.await.unwrap();
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(100), target.accept())
                .await
                .is_err(),
            "redirect target was contacted"
        );
    }
}
