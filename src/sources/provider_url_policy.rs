//! Shared policy for outbound URLs supplied by upstream providers.
//!
//! The policy validates the URL before a request is built, validates every redirect
//! synchronously, and validates the DNS answers used by the HTTP connector. Keeping
//! the DNS check in reqwest's resolver avoids a resolve-then-connect rebinding gap.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;

use reqwest::Url;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

use crate::error::{BioMcpError, SourceContext, SourceProvider};

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
const PMC_LINKED_ASSET_ORIGINS: &[&str] = &[
    "https://pmc.ncbi.nlm.nih.gov",
    "https://www.ncbi.nlm.nih.gov",
    "https://www.ebi.ac.uk",
    "https://europepmc.org",
];
const FIGSHARE_ORIGINS: &[&str] = &[
    "https://api.figshare.com",
    "https://figshare.com",
    "https://ndownloader.figshare.com",
    "https://s3-eu-west-1.amazonaws.com",
];
const CTGOV_DOCUMENT_ORIGINS: &[&str] = &["https://cdn.clinicaltrials.gov"];
const CSPEC_ORIGINS: &[&str] = &[
    "https://cspec.clinicalgenome.org",
    "https://cspec.genome.network",
];

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
    PmcLinkedArticleAsset,
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
    provider: SourceProvider,
    allowed_origins: Vec<AllowedOrigin>,
    credential_origins: Vec<AllowedOrigin>,
    unsafe_test_origin: Option<AllowedOrigin>,
    pmc_linked_numeric_id: Option<String>,
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
            provider: SourceProvider::SEMANTIC_SCHOLAR,
            allowed_origins,
            credential_origins: vec![canonical],
            unsafe_test_origin: unsafe_test_origin(),
            pmc_linked_numeric_id: None,
        };
        policy.validate_url(base)?;
        Ok(policy)
    }

    /// Policy for PDF URLs returned in Semantic Scholar payloads.
    pub(crate) fn semantic_scholar_pdf() -> Result<Self, BioMcpError> {
        Self::for_consumer(ProviderUrlConsumer::SemanticScholarPdf, None)
    }

    /// Policy for the read-only ClinGen Allele Registry API. A fixture base override
    /// is a selected origin, but redirects remain confined to that origin.
    pub(crate) fn clingen_car(base: &Url) -> Result<Self, BioMcpError> {
        let canonical = AllowedOrigin::parse("https://reg.genome.network")?;
        let configured = AllowedOrigin::from_url(base)
            .ok_or_else(|| policy_error("ClinGen Allele Registry base has no valid origin"))?;
        let mut allowed_origins = vec![canonical];
        if !allowed_origins.contains(&configured) {
            allowed_origins.push(configured);
        }
        let policy = Self {
            source: "ClinGen Allele Registry",
            provider: SourceProvider::CLINGEN_CAR,
            allowed_origins,
            credential_origins: Vec::new(),
            unsafe_test_origin: unsafe_test_origin()
                .or_else(|| selected_loopback_test_origin(base)),
            pmc_linked_numeric_id: None,
        };
        policy.validate_url(base)?;
        Ok(policy)
    }

    /// Policy for exact ClinGen LDH annotation IRIs.
    pub(crate) fn clingen_ldh() -> Result<Self, BioMcpError> {
        let canonical = AllowedOrigin::parse("https://ldh.genome.network")?;
        let fixture = std::env::var("BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN")
            .ok()
            .and_then(|value| Url::parse(&value).ok());
        let unsafe_test_origin = fixture.as_ref().and_then(selected_loopback_test_origin);
        if fixture.is_some() && unsafe_test_origin.is_none() {
            return Err(policy_error(
                "ClinGen LDH fixture origin must be exact loopback",
            ));
        }
        let mut allowed_origins = vec![canonical];
        if let Some(origin) = unsafe_test_origin.as_ref() {
            allowed_origins.push(origin.clone());
        }
        Ok(Self {
            source: "ClinGen LDH",
            provider: SourceProvider::CLINGEN_LDH,
            allowed_origins,
            credential_origins: Vec::new(),
            unsafe_test_origin,
            pmc_linked_numeric_id: None,
        })
    }

    /// Policy for exact ClinGen CSpec resource IRIs.
    pub(crate) fn cspec() -> Result<Self, BioMcpError> {
        let fixture_origin = cspec_fixture_origin()?;
        let mut allowed_origins = CSPEC_ORIGINS
            .iter()
            .map(|origin| AllowedOrigin::parse(origin))
            .collect::<Result<Vec<_>, _>>()?;
        let unsafe_test_origin = fixture_origin
            .as_ref()
            .and_then(selected_loopback_test_origin);
        if fixture_origin.is_some() && unsafe_test_origin.is_none() {
            return Err(policy_error("CSpec fixture origin must be exact loopback"));
        }
        if let Some(origin) = unsafe_test_origin.as_ref()
            && !allowed_origins.contains(origin)
        {
            allowed_origins.push(origin.clone());
        }
        Ok(Self {
            source: "ClinGen CSpec",
            provider: SourceProvider::CLINGEN_CSPEC,
            allowed_origins,
            credential_origins: Vec::new(),
            unsafe_test_origin,
            pmc_linked_numeric_id: None,
        })
    }

    /// Policy for one enumerated provider-returned URL consumer. API/CDN base overrides
    /// are selected origins only for fixture-configurable clients. Their exact IP-loopback
    /// origin may use HTTP; production origins remain HTTPS-only.
    pub(crate) fn for_consumer(
        consumer: ProviderUrlConsumer,
        selected_origin: Option<&Url>,
    ) -> Result<Self, BioMcpError> {
        let (source, provider, origins): (&'static str, SourceProvider, &[&str]) = match consumer {
            ProviderUrlConsumer::SemanticScholarPdf => (
                "Semantic Scholar PDF",
                SourceProvider::SEMANTIC_SCHOLAR,
                S2_PDF_ORIGINS,
            ),
            ProviderUrlConsumer::PmcOaArchive => (
                "PMC OA archive",
                SourceProvider::PMC_OPEN_ACCESS,
                PMC_OA_ORIGINS,
            ),
            ProviderUrlConsumer::PmcLinkedArticleAsset => (
                "PMC linked article asset",
                SourceProvider::PMC_OPEN_ACCESS,
                PMC_LINKED_ASSET_ORIGINS,
            ),
            ProviderUrlConsumer::FigshareDownload => (
                "Figshare download",
                SourceProvider::FIGSHARE,
                FIGSHARE_ORIGINS,
            ),
            ProviderUrlConsumer::ClinicalTrialsDocument => (
                "ClinicalTrials.gov document",
                SourceProvider::CLINICAL_TRIALS,
                CTGOV_DOCUMENT_ORIGINS,
            ),
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
            provider,
            allowed_origins,
            credential_origins: Vec::new(),
            unsafe_test_origin: unsafe_test_origin()
                .or_else(|| selected_origin.and_then(selected_loopback_test_origin)),
            pmc_linked_numeric_id: None,
        };
        if let Some(url) = selected_origin {
            policy.validate_url(url)?;
        }
        Ok(policy)
    }

    pub(crate) fn pmc_linked_article_asset(
        selected_origin: Option<&Url>,
        numeric_pmcid: &str,
    ) -> Result<Self, BioMcpError> {
        if numeric_pmcid.is_empty() || !numeric_pmcid.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(policy_error("invalid PMC article identity"));
        }
        let mut policy =
            Self::for_consumer(ProviderUrlConsumer::PmcLinkedArticleAsset, selected_origin)?;
        policy.pmc_linked_numeric_id = Some(numeric_pmcid.to_string());
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
        if let Some(numeric_pmcid) = self.pmc_linked_numeric_id.as_deref()
            && pmc_linked_asset_path(url, numeric_pmcid).is_none()
        {
            return Err(self.error("route or PMC identity is not allowlisted"));
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
            api: self.provider.label().to_string(),
            message: format!(
                "{} source unavailable: outbound policy rejected {class}",
                self.source
            ),
        }
        .with_source_context(SourceContext::retry(self.provider))
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

pub(crate) fn pmc_linked_asset_path(url: &Url, numeric_pmcid: &str) -> Option<String> {
    let raw_path = url.path().as_bytes();
    let mut decoded = Vec::with_capacity(raw_path.len());
    let mut index = 0;
    while index < raw_path.len() {
        if raw_path[index] == b'%' {
            let high = hex_value(*raw_path.get(index + 1)?)?;
            let low = hex_value(*raw_path.get(index + 2)?)?;
            let value = (high << 4) | low;
            if matches!(value, b'/' | b'\\') {
                return None;
            }
            decoded.push(value);
            index += 3;
        } else {
            decoded.push(raw_path[index]);
            index += 1;
        }
    }
    let decoded = String::from_utf8(decoded).ok()?;
    let components = decoded
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if components.iter().any(|part| {
        matches!(*part, "." | "..") || part.contains('\\') || part.chars().any(char::is_control)
    }) {
        return None;
    }
    let article_id = format!("PMC{numeric_pmcid}");
    let host = url.host_str()?.to_ascii_lowercase();
    let asset = match host.as_str() {
        "pmc.ncbi.nlm.nih.gov" => match components.as_slice() {
            ["articles", "instance", id, "bin", asset @ ..] if *id == numeric_pmcid => asset,
            ["articles", id, "bin", asset @ ..] if id.eq_ignore_ascii_case(&article_id) => asset,
            _ => return None,
        },
        "www.ncbi.nlm.nih.gov" => match components.as_slice() {
            ["pmc", "articles", "instance", id, "bin", asset @ ..] if *id == numeric_pmcid => asset,
            ["pmc", "articles", id, "bin", asset @ ..] if id.eq_ignore_ascii_case(&article_id) => {
                asset
            }
            _ => return None,
        },
        "europepmc.org" => match components.as_slice() {
            ["articles", id, "bin", asset @ ..] if id.eq_ignore_ascii_case(&article_id) => asset,
            _ => return None,
        },
        "www.ebi.ac.uk" => match components.as_slice() {
            ["europepmc", "articles", id, "bin", asset @ ..]
                if id.eq_ignore_ascii_case(&article_id) =>
            {
                asset
            }
            _ => return None,
        },
        _ if is_unsafe_fixture_url(url) => match components.as_slice() {
            ["articles", "instance", id, "bin", asset @ ..] if *id == numeric_pmcid => asset,
            ["articles", id, "bin", asset @ ..] if id.eq_ignore_ascii_case(&article_id) => asset,
            _ => return None,
        },
        _ => return None,
    };
    if asset.is_empty() {
        return None;
    }
    Some(asset.join("/"))
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn is_unsafe_fixture_url(url: &Url) -> bool {
    unsafe_test_origin().is_some_and(|origin| origin.matches(url))
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

pub(crate) fn cspec_fixture_origin() -> Result<Option<Url>, BioMcpError> {
    std::env::var("BIOMCP_CSPEC_FIXTURE_ORIGIN")
        .ok()
        .map(|raw| {
            let url =
                Url::parse(raw.trim()).map_err(|_| policy_error("invalid CSpec fixture origin"))?;
            if !is_exact_loopback_origin(&url) {
                return Err(policy_error("CSpec fixture origin must be exact loopback"));
            }
            Ok(url)
        })
        .transpose()
}

fn is_exact_loopback_origin(url: &Url) -> bool {
    selected_loopback_test_origin(url).is_some()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none()
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
            address
                .to_ipv4()
                .or_else(|| nat64_embedded_ipv4(address))
                .is_some_and(forbidden_ipv4)
                || forbidden_ipv6(address)
        }
    }
}

fn nat64_embedded_ipv4(address: Ipv6Addr) -> Option<Ipv4Addr> {
    let octets = address.octets();
    (octets[..12] == [0x00, 0x64, 0xff, 0x9b, 0, 0, 0, 0, 0, 0, 0, 0])
        .then(|| Ipv4Addr::new(octets[12], octets[13], octets[14], octets[15]))
}

fn forbidden_ipv4(address: Ipv4Addr) -> bool {
    let octets = address.octets();
    address.is_loopback()
        || address.is_private()
        || address.is_link_local()
        || address.is_unspecified()
        || address.is_multicast()
        || octets[0] == 0
        || (octets[0] == 100 && octets[1] & 0xc0 == 0x40)
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
            provider: SourceProvider::SEMANTIC_SCHOLAR,
            allowed_origins: vec![
                AllowedOrigin::parse("https://pdfs.semanticscholar.org").unwrap(),
            ],
            credential_origins: Vec::new(),
            unsafe_test_origin: None,
            pmc_linked_numeric_id: None,
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
            assert!(message.contains("Semantic Scholar"));
            assert!(message.to_ascii_lowercase().contains("retry"));
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
    fn rejects_cgnat_range_without_blocking_adjacent_public_addresses() {
        let policy = pdf_policy();
        for address in ["100.64.0.0", "100.127.255.255"] {
            assert!(
                policy
                    .validate_addresses([address.parse().unwrap()])
                    .is_err(),
                "accepted CGNAT address {address}"
            );
        }
        for address in ["100.63.255.255", "100.128.0.0"] {
            assert!(
                policy
                    .validate_addresses([address.parse().unwrap()])
                    .is_ok(),
                "rejected public address adjacent to CGNAT {address}"
            );
        }
    }

    #[test]
    fn rejects_ipv4_compatible_ipv6_with_forbidden_embedded_address() {
        let policy = pdf_policy();
        assert!(
            policy
                .validate_addresses(["::127.0.0.1".parse().unwrap()])
                .is_err()
        );
        assert!(
            policy
                .validate_addresses(["::93.184.216.34".parse().unwrap()])
                .is_ok()
        );
    }

    #[test]
    fn rejects_nat64_with_forbidden_embedded_address() {
        let policy = pdf_policy();
        assert!(
            policy
                .validate_addresses(["64:ff9b::127.0.0.1".parse().unwrap()])
                .is_err()
        );
        assert!(
            policy
                .validate_addresses(["64:ff9b::93.184.216.34".parse().unwrap()])
                .is_ok()
        );
        assert!(
            policy
                .validate_addresses(["64:ff9b::1:127.0.0.1".parse().unwrap()])
                .is_ok()
        );
    }

    #[test]
    fn rejects_encoded_ipv4_literal_hosts_after_url_canonicalization() {
        for raw in [
            "https://2130706433/",
            "https://0177.0.0.1/",
            "https://0x7f000001/",
            "https://0x7f.0.0.1/",
        ] {
            let url = Url::parse(raw).unwrap();
            assert_eq!(url.host_str(), Some("127.0.0.1"));
            let policy = ProviderUrlPolicy {
                source: "test provider",
                provider: SourceProvider::SEMANTIC_SCHOLAR,
                allowed_origins: vec![AllowedOrigin::from_url(&url).unwrap()],
                credential_origins: Vec::new(),
                unsafe_test_origin: None,
                pmc_linked_numeric_id: None,
            };
            assert!(policy.validate_url(&url).is_err(), "accepted {raw}");
        }
    }

    #[test]
    fn consumer_matrix_enumerates_valid_origins_and_shared_rejections() {
        for consumer in ProviderUrlConsumer::ALL.iter().copied() {
            let (valid_url, expected_source) = match consumer {
                ProviderUrlConsumer::SemanticScholarPdf => (
                    "https://pdfs.semanticscholar.org/paper.pdf",
                    "Semantic Scholar",
                ),
                ProviderUrlConsumer::PmcOaArchive => (
                    "https://ftp.ncbi.nlm.nih.gov/pub/pmc/archive.tgz",
                    "PMC Open Access",
                ),
                ProviderUrlConsumer::PmcLinkedArticleAsset => (
                    "https://pmc.ncbi.nlm.nih.gov/articles/instance/123/bin/s1.xlsx",
                    "PMC Open Access",
                ),
                ProviderUrlConsumer::FigshareDownload => {
                    ("https://ndownloader.figshare.com/files/1", "Figshare")
                }
                ProviderUrlConsumer::ClinicalTrialsDocument => (
                    "https://cdn.clinicaltrials.gov/large-docs/48/NCT03361748/Protocol.pdf",
                    "ClinicalTrials.gov",
                ),
            };
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
                let projection = error.public_projection();
                assert_eq!(projection.source, Some(expected_source));
                assert!(
                    projection
                        .recovery
                        .is_some_and(|recovery| recovery.to_ascii_lowercase().contains("retry"))
                );
                let diagnostic = error.to_string();
                assert!(!diagnostic.contains(raw));
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
    fn cspec_fixture_origin_requires_a_pathless_loopback_origin() {
        for raw in ["http://127.0.0.1:43210/", "https://[::1]:43210/"] {
            assert!(
                is_exact_loopback_origin(&Url::parse(raw).unwrap()),
                "accepted {raw}"
            );
        }
        for raw in [
            "http://localhost:43210/",
            "http://127.0.0.1:43210/redirect",
            "http://127.0.0.1:43210/?target=other",
            "http://127.0.0.1:43210/#fragment",
            "http://user:secret@127.0.0.1:43210/",
        ] {
            assert!(
                !is_exact_loopback_origin(&Url::parse(raw).unwrap()),
                "accepted {raw}"
            );
        }
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
                ProviderUrlConsumer::PmcLinkedArticleAsset,
                include_str!("pmc_article.rs"),
                "ProviderUrlPolicy::pmc_linked_article_asset(",
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
