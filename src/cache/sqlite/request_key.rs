//! Request context survives middleware's Parts-only key callback.
use reqwest::Request;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub(crate) struct RequestContext {
    pub(crate) base: String,
    pub(crate) original_url: reqwest::Url,
    pub(crate) canonical_url: reqwest::Url,
    headers: http::HeaderMap,
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
            .filter(|(name, _)| {
                !matches!(
                    name.to_ascii_lowercase().as_str(),
                    "api_key" | "apikey" | "api-key" | "key" | "access_token"
                )
            })
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        query.sort();
        url.set_query(None);
        url.set_fragment(None);
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
