//! OpenCitations Index reference client for the citation-evidence
//! confirmation phase.
//!
//! One bounded request per command: the provider returns the citing paper's
//! full known reference list in a single response, and the phase that consumes
//! it only distinguishes a confirmed directed edge from an unconfirmed one.
//! The client therefore returns the provider shape, and the phase turns every
//! failure into bounded unavailability instead of a command error.

use std::borrow::Cow;

use serde::Deserialize;

use crate::error::{BioMcpError, SourceContext, SourceProvider};
use crate::sources::{
    RequestBuilderSourceContextExt, apply_cache_mode, env_base, read_limited_source_body,
    shared_client,
};

/// The production OpenCitations Index base. The confirmation phase builds its
/// canonical evidence URL from this base, so a fixture base never reaches
/// output.
pub(crate) const OPENCITATIONS_BASE: &str = "https://api.opencitations.net/index/v2";
const OPENCITATIONS_BASE_ENV: &str = "BIOMCP_OPENCITATIONS_BASE";

/// DOI prefixes stripped by [`normalize_doi`]. The loop repeats, so a value
/// that carries more than one prefix (`doi:https://doi.org/10.1/x`) still
/// normalizes.
const DOI_PREFIXES: [&str; 5] = [
    "doi:",
    "https://doi.org/",
    "http://doi.org/",
    "https://dx.doi.org/",
    "http://dx.doi.org/",
];

/// One citation row from a reference list. The wire shape carries more
/// members (`timespan`, `journal_sc`, `author_sc`); only the evidence-bearing
/// members are decoded, and each one is validated before it can reach output.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct OpenCitationsEdge {
    pub oci: String,
    pub citing: String,
    pub cited: String,
    #[serde(default)]
    pub creation: Option<String>,
}

/// Normalizes a DOI the way the JATS reference extractor does: Unicode-trim,
/// strip the known DOI prefixes once or repeatedly, ASCII-lowercase, then
/// require the `10.` registrant prefix and a slash.
pub(crate) fn normalize_doi(value: &str) -> Option<String> {
    let mut current = value.trim().to_ascii_lowercase();
    loop {
        let Some(rest) = DOI_PREFIXES
            .iter()
            .find_map(|prefix| current.strip_prefix(prefix))
        else {
            break;
        };
        current = rest.trim().to_string();
    }
    (current.starts_with("10.") && current.contains('/')).then_some(current)
}

/// The OpenCitations identifier shape, exactly `^[0-9]{1,32}-[0-9]{1,32}$`.
/// A matching row with any other shape fails the attempt closed rather than
/// publishing an unvalidated provider string.
pub(crate) fn valid_oci(value: &str) -> bool {
    let mut parts = value.split('-');
    let (Some(left), Some(right), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    [left, right].iter().all(|part| {
        !part.is_empty() && part.len() <= 32 && part.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// The creation shape, exactly `^[0-9]{4}-[0-9]{2}(-[0-9]{2})?$`. Anything
/// else becomes null rather than an echoed provider string.
pub(crate) fn valid_creation(value: &str) -> bool {
    let widths = [4_usize, 2, 2];
    let parts: Vec<&str> = value.split('-').collect();
    (parts.len() == 2 || parts.len() == 3)
        && parts.iter().zip(widths).all(|(part, width)| {
            part.len() == width && part.bytes().all(|byte| byte.is_ascii_digit())
        })
}

#[derive(Clone)]
pub(crate) struct OpenCitationsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl OpenCitationsClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: shared_client()?,
            base: env_base(OPENCITATIONS_BASE, OPENCITATIONS_BASE_ENV),
        })
    }

    /// The citing paper's full known reference list. The provider answers an
    /// unknown DOI with an empty array, so an absent row is an empty list,
    /// never an error.
    pub(crate) async fn references_by_doi(
        &self,
        citing_doi: &str,
    ) -> Result<Vec<OpenCitationsEdge>, BioMcpError> {
        let context = SourceContext::retry(SourceProvider::OPEN_CITATIONS);
        let url = format!(
            "{}/references/doi:{citing_doi}",
            self.base.trim_end_matches('/')
        );
        let response = apply_cache_mode(self.client.get(url))
            .send_with_source_context(context)
            .await?;
        let status = response.status();
        let bytes = read_limited_source_body(response, context).await?;
        crate::sources::decode_json(context, status, None, &bytes, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDirGuard;
    use axum::{
        Router,
        body::Body,
        extract::{Path as AxumPath, State},
        http::Response,
        routing::get,
    };
    use std::sync::{Arc, Mutex};

    struct EnvGuard(Vec<(&'static str, Option<std::ffi::OsString>)>);

    impl EnvGuard {
        fn set(values: &[(&'static str, Option<&str>)]) -> Self {
            let mut previous = Vec::new();
            for (key, value) in values {
                previous.push((*key, std::env::var_os(key)));
                // SAFETY: these tests share one serial-test group, and the guard
                // restores every value before releasing it.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
            Self(previous)
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..).rev() {
                // SAFETY: see `EnvGuard::set`.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(key, value),
                        None => std::env::remove_var(key),
                    }
                }
            }
        }
    }

    #[derive(Clone)]
    struct OriginState {
        status: u16,
        body: Vec<u8>,
        paths: Arc<Mutex<Vec<String>>>,
    }

    async fn references_route(
        State(state): State<OriginState>,
        AxumPath(rest): AxumPath<String>,
    ) -> Response<Body> {
        state
            .paths
            .lock()
            .unwrap()
            .push(format!("/references/{rest}"));
        Response::builder()
            .status(state.status)
            .header("content-type", "application/json")
            .body(Body::from(state.body.clone()))
            .unwrap()
    }

    async fn origin(status: u16, body: Vec<u8>) -> (String, OriginState) {
        let state = OriginState {
            status,
            body,
            paths: Arc::new(Mutex::new(Vec::new())),
        };
        let app = Router::new()
            .route("/references/{*rest}", get(references_route))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{address}"), state)
    }

    fn references_body(rows: &str) -> Vec<u8> {
        format!("[{rows}]").into_bytes()
    }

    #[test]
    fn doi_normalization_matches_the_jats_reference_rules() {
        for (input, expected) in [
            ("10.1000/x", Some("10.1000/x")),
            (" 10.1000/x ", Some("10.1000/x")),
            ("DOI:10.1000/X", Some("10.1000/x")),
            ("doi:10.1000/x", Some("10.1000/x")),
            ("https://doi.org/10.1000/X", Some("10.1000/x")),
            ("http://doi.org/10.1000/x", Some("10.1000/x")),
            ("https://dx.doi.org/10.1000/x", Some("10.1000/x")),
            ("http://dx.doi.org/10.1000/x", Some("10.1000/x")),
            ("doi:https://doi.org/10.1000/x", Some("10.1000/x")),
            ("10.1000/0001.2", Some("10.1000/0001.2")),
            ("10.1000", None),
            ("not-a-doi", None),
            ("https://example.org/10.1000/x", None),
            ("", None),
            ("  ", None),
        ] {
            assert_eq!(normalize_doi(input).as_deref(), expected, "{input:?}");
        }
    }

    #[test]
    fn oci_and_creation_validation_fails_closed() {
        for valid in ["061502131318-062102119315", "1-2"] {
            assert!(valid_oci(valid), "{valid:?}");
        }
        let widest = format!("{0}-{0}", "9".repeat(32));
        assert!(valid_oci(&widest), "{widest:?}");
        for invalid in [
            "",
            "1",
            "1-",
            "-2",
            "1-2-3",
            "1-2 ",
            " 1-2",
            "quote\"-1",
            "1|2",
            "`1`-2",
            "$1-2",
            "1;2",
            "1&2",
            "α-1",
        ] {
            assert!(!valid_oci(invalid), "{invalid:?}");
        }
        let too_wide = format!("{}-2", "9".repeat(33));
        assert!(!valid_oci(&too_wide), "{too_wide:?}");
        for valid in ["2012-07-12", "2011-03"] {
            assert!(valid_creation(valid), "{valid:?}");
        }
        for invalid in [
            "",
            "2012",
            "2012-7",
            "2012-07-1",
            "2012-07-12-01",
            "2012/07",
            "2012-07-12|",
            "2012-07-12;",
            "2012-07-12&",
            "2012-07-12\"",
            "2012-07-12`",
            "2012-07-12$",
            "αβγδ-07",
        ] {
            assert!(!valid_creation(invalid), "{invalid:?}");
        }
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn references_request_uses_the_normalized_doi_path() {
        let (base, state) = origin(
            200,
            references_body(
                r#"{"oci":"061502131318-062102119315","citing":"doi:10.1000/citing","cited":"omid:br/1 doi:10.1000/cited pmid:1","creation":"2012-07-12","timespan":"P1Y","journal_sc":"no","author_sc":"no"}"#,
            ),
        )
        .await;
        let cache = TempDirGuard::new("opencitations-path");
        let _env = EnvGuard::set(&[
            ("BIOMCP_OPENCITATIONS_BASE", Some(&base)),
            ("BIOMCP_CACHE_DIR", Some(cache.path().to_str().unwrap())),
        ]);

        let client = OpenCitationsClient::new().unwrap();
        let edges = client.references_by_doi("10.1000/citing").await.unwrap();
        assert_eq!(
            edges,
            vec![OpenCitationsEdge {
                oci: "061502131318-062102119315".into(),
                citing: "doi:10.1000/citing".into(),
                cited: "omid:br/1 doi:10.1000/cited pmid:1".into(),
                creation: Some("2012-07-12".into()),
            }]
        );
        assert_eq!(
            state.paths.lock().unwrap().as_slice(),
            ["/references/doi:10.1000/citing"]
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn references_reject_malformed_bodies_and_statuses() {
        for (label, status, body) in [
            ("non-array", 200, b"{}".to_vec()),
            (
                "missing-oci",
                200,
                references_body(r#"{"citing":"c","cited":"d"}"#),
            ),
            (
                "null-oci",
                200,
                references_body(r#"{"oci":null,"citing":"c","cited":"d"}"#),
            ),
            (
                "non-string-cited",
                200,
                references_body(r#"{"oci":"1-2","citing":"c","cited":7}"#),
            ),
            ("not-json", 200, b"<html>".to_vec()),
            (
                "server-error",
                500,
                references_body(r#"{"oci":"1-2","citing":"c","cited":"d","creation":"2012-07"}"#),
            ),
            ("not-found", 404, b"[]".to_vec()),
        ] {
            let (base, _state) = origin(status, body).await;
            let cache = TempDirGuard::new("opencitations-malformed");
            let _env = EnvGuard::set(&[
                ("BIOMCP_OPENCITATIONS_BASE", Some(&base)),
                ("BIOMCP_CACHE_DIR", Some(cache.path().to_str().unwrap())),
            ]);
            let client = OpenCitationsClient::new().unwrap();
            let result = client.references_by_doi("10.1000/citing").await;
            assert!(result.is_err(), "{label} must fail the attempt");
        }
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn references_accept_an_empty_list_as_no_known_edge() {
        let (base, _state) = origin(200, b"[]".to_vec()).await;
        let cache = TempDirGuard::new("opencitations-empty");
        let _env = EnvGuard::set(&[
            ("BIOMCP_OPENCITATIONS_BASE", Some(&base)),
            ("BIOMCP_CACHE_DIR", Some(cache.path().to_str().unwrap())),
        ]);
        let client = OpenCitationsClient::new().unwrap();
        assert_eq!(
            client.references_by_doi("10.1000/citing").await.unwrap(),
            Vec::new()
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn references_reject_an_oversize_body() {
        let oversize = vec![b'x'; crate::sources::DEFAULT_MAX_BODY_BYTES + 1];
        let (base, _state) = origin(200, oversize).await;
        let cache = TempDirGuard::new("opencitations-oversize");
        let _env = EnvGuard::set(&[
            ("BIOMCP_OPENCITATIONS_BASE", Some(&base)),
            ("BIOMCP_CACHE_DIR", Some(cache.path().to_str().unwrap())),
        ]);
        let client = OpenCitationsClient::new().unwrap();
        assert!(
            client.references_by_doi("10.1000/citing").await.is_err(),
            "an oversize body must fail the attempt"
        );
    }
}
