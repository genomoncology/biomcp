//! Operator-supplied certificate authorities for ordinary outbound TLS.

use std::path::{Path, PathBuf};

use tracing::warn;

use crate::error::BioMcpError;

/// Operator variable for an extra PEM trust bundle.
const CA_BUNDLE_ENV: &str = "BIOMCP_CA_BUNDLE";

/// Conventional fallback variable, consulted only when [`CA_BUNDLE_ENV`] is unset.
const CA_BUNDLE_FALLBACK_ENV: &str = "SSL_CERT_FILE";

/// A bundle that was read, parsed, and validated, and whose certificates were
/// added to a client builder. It carries the path so a later client-build
/// failure still names the bundle the operator configured.
#[derive(Clone, Debug)]
pub(crate) struct CaBundle {
    path: PathBuf,
}

/// Adds the operator's certificate bundle, if one is configured, to `builder`.
/// The bundled webpki roots stay in place: an operator bundle only adds trust.
pub(crate) fn configure(
    builder: reqwest::ClientBuilder,
) -> Result<(reqwest::ClientBuilder, Option<CaBundle>), BioMcpError> {
    let Some(loaded) = load()? else {
        return Ok((builder, None));
    };
    let mut builder = builder;
    for certificate in loaded.certificates {
        builder = builder.add_root_certificate(certificate);
    }
    Ok((builder, Some(loaded.bundle)))
}

/// Applies the operator's bundle and builds the client in one step.
pub(crate) fn build_client(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::Client, BioMcpError> {
    let (builder, bundle) = configure(builder)?;
    build(builder, bundle)
}

/// Builds a client whose builder already carries the operator's bundle.
pub(crate) fn build(
    builder: reqwest::ClientBuilder,
    bundle: Option<CaBundle>,
) -> Result<reqwest::Client, BioMcpError> {
    builder.build().map_err(|error| match bundle {
        Some(bundle) => bundle.client_build_error(error),
        None => BioMcpError::HttpClientInit(error),
    })
}

impl CaBundle {
    fn client_build_error(&self, error: reqwest::Error) -> BioMcpError {
        bundle_error(
            &self.path,
            format!("the TLS client could not be built: {error}"),
        )
    }
}

/// The certificates an operator bundle contributes, plus its path.
struct LoadedBundle {
    bundle: CaBundle,
    certificates: Vec<reqwest::Certificate>,
}

/// Where the operator named a bundle, and whether failing to read it is fatal.
enum BundleSource {
    /// `BIOMCP_CA_BUNDLE`: every problem is fatal and names the path.
    Explicit(PathBuf),
    /// `SSL_CERT_FILE`: an unreadable or blank value warns and continues.
    Fallback(PathBuf),
}

/// Resolves and loads the configured bundle, if one is configured.
fn load() -> Result<Option<LoadedBundle>, BioMcpError> {
    let Some(source) = configured_source() else {
        return Ok(None);
    };
    let (path, required) = match source {
        BundleSource::Explicit(path) => (path, true),
        BundleSource::Fallback(path) => (path, false),
    };
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if !required => {
            warn!(
                path = %path.display(),
                %error,
                "SSL_CERT_FILE could not be read; continuing with the bundled TLS roots"
            );
            return Ok(None);
        }
        Err(error) => {
            return Err(bundle_error(
                &path,
                format!("the file could not be read: {error}"),
            ));
        }
    };
    let certificates = parse_certificates(&path, &bytes)?;
    Ok(Some(LoadedBundle {
        bundle: CaBundle { path },
        certificates,
    }))
}

fn configured_source() -> Option<BundleSource> {
    // A blank or whitespace-only BIOMCP_CA_BUNDLE counts as unset, following
    // the provider base override reads.
    if let Some(value) = std::env::var(CA_BUNDLE_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return Some(BundleSource::Explicit(PathBuf::from(value)));
    }
    let value = std::env::var_os(CA_BUNDLE_FALLBACK_ENV)?;
    let value = value.to_string_lossy();
    let value = value.trim();
    if value.is_empty() {
        warn!("SSL_CERT_FILE is blank; continuing with the bundled TLS roots");
        return None;
    }
    Some(BundleSource::Fallback(PathBuf::from(value)))
}

/// Parses the bundle and validates every certificate's DER before any client
/// build, which is what lets a bad bundle fail with the path named.
fn parse_certificates(path: &Path, bytes: &[u8]) -> Result<Vec<reqwest::Certificate>, BioMcpError> {
    let mut trusted = rustls::RootCertStore::empty();
    let mut certificates = Vec::new();
    let mut reader = bytes;
    for item in rustls_pemfile::certs(&mut reader) {
        let certificate = item
            .map_err(|error| bundle_error(path, format!("the PEM content is invalid: {error}")))?;
        trusted
            .add(certificate.clone())
            .map_err(|error| bundle_error(path, format!("a certificate is invalid: {error}")))?;
        certificates.push(
            reqwest::Certificate::from_der(certificate.as_ref()).map_err(|error| {
                bundle_error(path, format!("a certificate is invalid: {error}"))
            })?,
        );
    }
    if certificates.is_empty() {
        return Err(bundle_error(path, "no certificates were found".to_string()));
    }
    Ok(certificates)
}

fn bundle_error(path: &Path, reason: impl Into<String>) -> BioMcpError {
    BioMcpError::CaBundle {
        path: path.display().to_string(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard(Vec<(&'static str, Option<std::ffi::OsString>)>);

    impl EnvGuard {
        fn set(pairs: &[(&'static str, Option<&str>)]) -> Self {
            let mut guard = Self(Vec::new());
            for (name, value) in pairs {
                guard.0.push((name, std::env::var_os(name)));
                // SAFETY: the test holds the serial-test process-wide environment lock.
                unsafe {
                    match value {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
            guard
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, previous) in self.0.drain(..).rev() {
                // SAFETY: the test holds the serial-test process-wide environment lock.
                unsafe {
                    match previous {
                        Some(value) => std::env::set_var(name, value),
                        None => std::env::remove_var(name),
                    }
                }
            }
        }
    }

    fn write_bundle(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, contents).expect("write bundle fixture");
        path
    }

    fn valid_bundle(dir: &Path, name: &str) -> PathBuf {
        let key = rcgen::KeyPair::generate().expect("CA key");
        let mut params = rcgen::CertificateParams::new(Vec::<String>::new()).expect("CA params");
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let certificate = params.self_signed(&key).expect("self-signed CA");
        write_bundle(dir, name, certificate.pem().as_bytes())
    }

    fn assert_bundle_error(error: &BioMcpError, path: &Path, needle: &str) {
        match error {
            BioMcpError::CaBundle {
                path: actual,
                reason,
            } => {
                assert_eq!(Path::new(actual.as_str()), path);
                assert!(reason.contains(needle), "reason was {reason:?}");
            }
            other => panic!("expected a CA bundle error, found {other:?}"),
        }
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn explicit_bundle_wins_over_the_ssl_cert_file_fallback() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let bundle = valid_bundle(dir.path(), "ca.pem");
        let _env = EnvGuard::set(&[
            ("BIOMCP_CA_BUNDLE", Some(bundle.to_str().unwrap())),
            ("SSL_CERT_FILE", Some("/nonexistent/fallback.pem")),
        ]);
        let loaded = load()
            .expect("explicit bundle loads")
            .expect("bundle present");
        assert_eq!(loaded.bundle.path, bundle);
        assert_eq!(loaded.certificates.len(), 1);
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn blank_explicit_bundle_falls_back_to_a_readable_ssl_cert_file() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let bundle = valid_bundle(dir.path(), "fallback.pem");
        let _env = EnvGuard::set(&[
            ("BIOMCP_CA_BUNDLE", Some("   ")),
            ("SSL_CERT_FILE", Some(bundle.to_str().unwrap())),
        ]);
        let loaded = load()
            .expect("fallback bundle loads")
            .expect("bundle present");
        assert_eq!(loaded.bundle.path, bundle);
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn blank_or_unreadable_ssl_cert_file_continues_with_bundled_roots() {
        let dir = tempfile::tempdir().expect("bundle directory");
        {
            let _env = EnvGuard::set(&[("BIOMCP_CA_BUNDLE", None), ("SSL_CERT_FILE", Some("   "))]);
            assert!(load().expect("blank fallback continues").is_none());
        }
        let _env = EnvGuard::set(&[
            ("BIOMCP_CA_BUNDLE", None),
            ("SSL_CERT_FILE", Some(dir.path().to_str().unwrap())),
        ]);
        assert!(load().expect("unreadable fallback continues").is_none());
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn missing_explicit_bundle_fails_with_its_path() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let missing = dir.path().join("absent.pem");
        let _env = EnvGuard::set(&[("BIOMCP_CA_BUNDLE", Some(missing.to_str().unwrap()))]);
        let error = load().err().expect("missing bundle fails");
        assert_bundle_error(&error, &missing, "could not be read");
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn malformed_pem_fails_with_its_path() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let path = write_bundle(
            dir.path(),
            "ca.pem",
            b"-----BEGIN CERTIFICATE-----\nnot base64\n-----END CERTIFICATE-----\n",
        );
        let _env = EnvGuard::set(&[("BIOMCP_CA_BUNDLE", Some(path.to_str().unwrap()))]);
        let error = load().err().expect("malformed PEM fails");
        assert_bundle_error(&error, &path, "PEM content is invalid");
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn certificate_less_bundle_fails_with_its_path() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let path = write_bundle(dir.path(), "ca.pem", b"");
        let _env = EnvGuard::set(&[("BIOMCP_CA_BUNDLE", Some(path.to_str().unwrap()))]);
        let error = load().err().expect("certificate-less bundle fails");
        assert_bundle_error(&error, &path, "no certificates were found");
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn invalid_der_fails_with_its_path() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let path = write_bundle(
            dir.path(),
            "ca.pem",
            b"-----BEGIN CERTIFICATE-----\naGVsbG8=\n-----END CERTIFICATE-----\n",
        );
        let _env = EnvGuard::set(&[("BIOMCP_CA_BUNDLE", Some(path.to_str().unwrap()))]);
        let error = load().err().expect("invalid DER fails");
        assert_bundle_error(&error, &path, "a certificate is invalid");
    }

    #[test]
    #[serial_test::serial(source_env)]
    fn malformed_ssl_cert_file_fails_with_its_path() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let path = write_bundle(
            dir.path(),
            "ca.pem",
            b"-----BEGIN CERTIFICATE-----\nnot base64\n-----END CERTIFICATE-----\n",
        );
        let _env = EnvGuard::set(&[
            ("BIOMCP_CA_BUNDLE", None),
            ("SSL_CERT_FILE", Some(path.to_str().unwrap())),
        ]);
        let error = load().err().expect("malformed fallback fails");
        assert_bundle_error(&error, &path, "PEM content is invalid");
    }
}
