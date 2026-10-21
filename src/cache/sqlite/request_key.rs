//! Request context survives middleware's Parts-only key callback.
use reqwest::Request;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(crate) struct RequestContext {
    pub(crate) base: String,
    pub(crate) original_url: reqwest::Url,
    pub(crate) canonical_url: reqwest::Url,
    headers: http::HeaderMap,
    credentials: Vec<String>,
}

tokio::task_local! {
    pub(crate) static REQUEST: RequestContext;
}

impl RequestContext {
    pub(crate) fn from_request(request: &Request) -> Option<Self> {
        let body = match request.body() {
            Some(body) => body.as_bytes()?,
            None => &[],
        };
        let mut url = request.url().clone();
        let mut query = url
            .query_pairs()
            .filter(|(name, _)| !credential_parameter(name))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        query.sort();
        url.set_query(None);
        url.set_fragment(None);
        let _ = url.set_username("");
        let _ = url.set_password(None);
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        // Url already lowercases schemes and domain names. Length-delimited JSON
        // prevents separators inside a URL or method aliasing another request.
        let bytes = serde_json::to_vec(&(
            1,
            request.method().as_str(),
            url.as_str(),
            format!("{:x}", Sha256::digest(body)),
        ))
        .ok()?;
        Some(Self {
            base: format!("v1:{:x}", Sha256::digest(bytes)),
            original_url: request.url().clone(),
            canonical_url: url,
            headers: request.headers().clone(),
            credentials: request
                .url()
                .query_pairs()
                .filter(|(name, _)| credential_parameter(name))
                .filter(|(_, value)| !value.is_empty())
                .flat_map(|(_, value)| {
                    let mut encoded_url = request.url().clone();
                    encoded_url.set_query(None);
                    encoded_url.query_pairs_mut().append_pair("", &value);
                    let encoded = encoded_url
                        .query()
                        .unwrap_or_default()
                        .trim_start_matches('=')
                        .to_owned();
                    [value.into_owned(), encoded]
                })
                .collect(),
        })
    }

    /// Never publish a provider echo of a credential, including large blobs.
    pub(crate) fn contains_credential(&self, bytes: &[u8]) -> bool {
        self.credentials.iter().any(|value| {
            bytes
                .windows(value.len())
                .any(|window| window == value.as_bytes())
        })
    }

    pub(crate) fn variant(&self, vary: &str) -> Option<String> {
        let mut names = vary
            .split(',')
            .map(|v| v.trim().to_ascii_lowercase())
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        if names.iter().any(|name| {
            http::header::HeaderName::from_bytes(name.as_bytes()).is_err() || name == "*"
        }) {
            return None;
        }
        let values = names
            .iter()
            .map(|name| {
                (
                    name,
                    self.headers
                        .get_all(name)
                        .iter()
                        .map(|v| v.as_bytes())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let bytes = serde_json::to_vec(&(&self.base, values)).ok()?;
        Some(format!("v1:{:x}", Sha256::digest(bytes)))
    }
}

pub(crate) fn current() -> Option<RequestContext> {
    REQUEST.try_with(Clone::clone).ok()
}

/// One spelling rule covers canonical keys and authentication header bypass.
pub(crate) fn credential_parameter(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase().replace(['-', '_'], "");
    normalized.ends_with("token")
        || normalized.ends_with("apikey")
        || normalized.ends_with("secret")
        || normalized.ends_with("signature")
        || matches!(
            normalized.as_str(),
            "key"
                | "authkey"
                | "secretkey"
                | "authorization"
                | "auth"
                | "password"
                | "credential"
                | "credentials"
        )
}

pub(crate) fn credential_header(name: &str) -> bool {
    credential_parameter(name)
        || matches!(
            name.to_ascii_lowercase().as_str(),
            "authorization"
                | "proxy-authorization"
                | "cookie"
                | "x-api-key"
                | "x-auth-token"
                | "x-goog-api-key"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authentication_spellings_never_enter_cache_identity_or_header_admission() {
        let client = reqwest::Client::new();
        let public = client.get("https://example.org/?q=BRAF").build().unwrap();
        let expected = RequestContext::from_request(&public).unwrap();
        for name in [
            "api_key",
            "API-KEY",
            "apikey",
            "key",
            "token",
            "access_token",
            "auth-token",
            "oauth_token",
            "api_token",
            "refresh_token",
            "authorization",
            "secret_key",
            "client_secret",
            "password",
            "secret",
            "signature",
        ] {
            let request = client
                .get(public.url().clone())
                .query(&[(name, "fake-canary")])
                .build()
                .unwrap();
            let context = RequestContext::from_request(&request).unwrap();
            assert_eq!(context.base, expected.base, "{name}");
            assert!(!context.canonical_url.as_str().contains("fake-canary"));
            assert!(context.contains_credential(b"echo fake-canary"));
        }
        for name in [
            "authorization",
            "proxy-authorization",
            "cookie",
            "x-api-key",
            "api-key",
            "x-auth-token",
            "x-goog-api-key",
        ] {
            let request = client
                .get(public.url().clone())
                .header(name, "fake-canary")
                .build()
                .unwrap();
            assert!(
                crate::cache::http::bypass_request(&request, &http::Extensions::new()),
                "{name}"
            );
        }
    }
}
