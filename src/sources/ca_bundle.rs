//! Operator-supplied certificate authorities for ordinary outbound TLS.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rustls::pki_types::pem::PemObject;
use tracing::warn;

use crate::error::BioMcpError;

const CA_BUNDLE_ENV: &str = "BIOMCP_CA_BUNDLE";
const CA_BUNDLE_FALLBACK_ENV: &str = "SSL_CERT_FILE";

static RESOLVED_BUNDLE: OnceLock<Result<Option<LoadedBundle>, CachedBundleError>> = OnceLock::new();

#[cfg(test)]
static PARSE_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub(crate) struct CaBundle {
    path: PathBuf,
}

#[derive(Clone)]
struct LoadedBundle {
    bundle: CaBundle,
    certificates: Vec<reqwest::Certificate>,
}

#[derive(Clone)]
struct CachedBundleError {
    path: PathBuf,
    reason: String,
}

impl CachedBundleError {
    fn to_error(&self) -> BioMcpError {
        bundle_error(&self.path, self.reason.clone())
    }
}

enum BundleSource {
    Explicit(PathBuf),
    Fallback(PathBuf),
}

/// Resolves the process-wide bundle snapshot before a server accepts work.
pub(crate) fn validate() -> Result<(), BioMcpError> {
    load().map(|_| ())
}

/// Adds the operator's certificate bundle, if configured, to `builder`.
/// Reqwest's bundled Mozilla roots remain enabled; this only adds trust.
pub(crate) fn configure(
    builder: reqwest::ClientBuilder,
) -> Result<(reqwest::ClientBuilder, Option<CaBundle>), BioMcpError> {
    let Some(loaded) = load()? else {
        return Ok((builder, None));
    };
    let mut builder = builder;
    for certificate in &loaded.certificates {
        builder = builder.add_root_certificate(certificate.clone());
    }
    Ok((builder, Some(loaded.bundle.clone())))
}

pub(crate) fn build_client(
    builder: reqwest::ClientBuilder,
) -> Result<reqwest::Client, BioMcpError> {
    let (builder, bundle) = configure(builder)?;
    build(builder, bundle)
}

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

fn load() -> Result<Option<&'static LoadedBundle>, BioMcpError> {
    match RESOLVED_BUNDLE.get_or_init(resolve) {
        Ok(bundle) => Ok(bundle.as_ref()),
        Err(error) => Err(error.to_error()),
    }
}

fn resolve() -> Result<Option<LoadedBundle>, CachedBundleError> {
    resolve_inner().map_err(|error| match error {
        BioMcpError::CaBundle { path, reason } => CachedBundleError {
            path: PathBuf::from(path),
            reason,
        },
        _ => unreachable!("CA bundle resolution only returns CA bundle errors"),
    })
}

fn resolve_inner() -> Result<Option<LoadedBundle>, BioMcpError> {
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
            warn!(path = %path.display(), %error, "SSL_CERT_FILE could not be read; continuing with the bundled TLS roots; set BIOMCP_CA_BUNDLE to require a usable custom bundle");
            return Ok(None);
        }
        Err(error) => {
            return Err(bundle_error(
                &path,
                format!("the file could not be read: {error}"),
            ));
        }
    };
    let certificates = match parse_certificates(&path, &bytes) {
        Ok(certificates) => certificates,
        Err(_error) if !required => {
            warn!(path = %path.display(), "SSL_CERT_FILE is not a usable certificate bundle; continuing with the bundled TLS roots; set BIOMCP_CA_BUNDLE to require a usable custom bundle");
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    Ok(Some(LoadedBundle {
        bundle: CaBundle { path },
        certificates,
    }))
}

fn configured_path(value: std::ffi::OsString) -> Option<PathBuf> {
    match value.to_str() {
        Some(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| PathBuf::from(value))
        }
        None => Some(PathBuf::from(value)),
    }
}

fn configured_source() -> Option<BundleSource> {
    if let Some(path) = std::env::var_os(CA_BUNDLE_ENV).and_then(configured_path) {
        return Some(BundleSource::Explicit(path));
    }
    let value = std::env::var_os(CA_BUNDLE_FALLBACK_ENV)?;
    let Some(path) = configured_path(value) else {
        warn!(
            "SSL_CERT_FILE is blank; continuing with the bundled TLS roots; set BIOMCP_CA_BUNDLE to require a usable custom bundle"
        );
        return None;
    };
    Some(BundleSource::Fallback(path))
}

fn parse_certificates(path: &Path, bytes: &[u8]) -> Result<Vec<reqwest::Certificate>, BioMcpError> {
    #[cfg(test)]
    PARSE_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    let mut trusted = rustls::RootCertStore::empty();
    let mut certificates = Vec::new();
    for (offset, item) in rustls::pki_types::CertificateDer::pem_slice_iter(bytes).enumerate() {
        let index = offset + 1;
        let certificate: rustls::pki_types::CertificateDer<'static> = item
            .map_err(|_| {
                bundle_error(
                    path,
                    format!("certificate {index} has invalid PEM encoding"),
                )
            })?
            .into_owned();
        trusted.add(certificate.clone()).map_err(|_| {
            bundle_error(
                path,
                format!("certificate {index} has invalid DER encoding"),
            )
        })?;
        certificates.push(
            reqwest::Certificate::from_der(certificate.as_ref()).map_err(|_| {
                bundle_error(
                    path,
                    format!("certificate {index} has invalid DER encoding"),
                )
            })?,
        );
    }
    if certificates.is_empty() {
        return Err(bundle_error(path, "no certificates were found"));
    }
    Ok(certificates)
}

fn bundle_error(path: &Path, reason: impl Into<String>) -> BioMcpError {
    BioMcpError::CaBundle {
        path: path.to_string_lossy().into_owned(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_bundle(path: &Path) {
        let key = rcgen::KeyPair::generate().expect("CA key");
        let mut params = rcgen::CertificateParams::new(Vec::<String>::new()).expect("CA params");
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let certificate = params.self_signed(&key).expect("self-signed CA");
        std::fs::write(path, certificate.pem()).expect("write CA");
    }

    #[test]
    fn loader_cases_run_in_fresh_processes() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let valid = dir.path().join("valid.pem");
        let malformed = dir.path().join("malformed.pem");
        let empty = dir.path().join("empty.pem");
        let missing = dir.path().join("missing.pem");
        let directory = dir.path().join("directory");
        valid_bundle(&valid);
        std::fs::write(&malformed, b"not a certificate").expect("write malformed bundle");
        std::fs::write(&empty, b"").expect("write empty bundle");
        std::fs::create_dir(&directory).expect("create directory");
        let cases = [
            (Some(valid.as_os_str()), None, "some"),
            (
                Some(std::ffi::OsStr::new("   ")),
                Some(valid.as_os_str()),
                "some",
            ),
            (Some(missing.as_os_str()), Some(valid.as_os_str()), "error"),
            (Some(malformed.as_os_str()), None, "error"),
            (Some(empty.as_os_str()), None, "error"),
            (None, Some(missing.as_os_str()), "none"),
            (None, Some(directory.as_os_str()), "none"),
            (None, Some(std::ffi::OsStr::new("   ")), "none"),
        ];
        for (explicit, fallback, expected) in cases {
            let mut command =
                std::process::Command::new(std::env::current_exe().expect("test executable"));
            command
                .args([
                    "--ignored",
                    "--exact",
                    "sources::ca_bundle::tests::bundle_loader_child",
                ])
                .env_remove(CA_BUNDLE_ENV)
                .env_remove(CA_BUNDLE_FALLBACK_ENV)
                .env("BIOMCP_CA_TEST_EXPECT", expected)
                .env("RUST_LOG", "warn");
            if let Some(value) = explicit {
                command.env(CA_BUNDLE_ENV, value);
            }
            if let Some(value) = fallback {
                command.env(CA_BUNDLE_FALLBACK_ENV, value);
            }
            let output = command.output().expect("run loader child");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    #[test]
    #[ignore = "reentered by loader_cases_run_in_fresh_processes"]
    fn bundle_loader_child() {
        let result = resolve_inner();
        match std::env::var("BIOMCP_CA_TEST_EXPECT").as_deref() {
            Ok("some") => assert!(matches!(result, Ok(Some(_)))),
            Ok("none") => assert!(matches!(result, Ok(None))),
            Ok("error") => assert!(matches!(result, Err(BioMcpError::CaBundle { .. }))),
            other => panic!("unexpected loader child expectation: {other:?}"),
        }
    }

    #[test]
    fn bundle_is_parsed_once_across_builders_in_a_fresh_process() {
        let dir = tempfile::tempdir().expect("bundle directory");
        let path = dir.path().join("broken.pem");
        std::fs::write(
            &path,
            b"-----BEGIN CERTIFICATE-----\nnot base64\n-----END CERTIFICATE-----\n",
        )
        .expect("write fallback");
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--ignored",
                "--exact",
                "sources::ca_bundle::tests::bundle_parse_child",
            ])
            .env_remove(CA_BUNDLE_ENV)
            .env(CA_BUNDLE_FALLBACK_ENV, &path)
            .env("RUST_LOG", "warn")
            .output()
            .expect("run parse child");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            stderr
                .matches("SSL_CERT_FILE is not a usable certificate bundle")
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "reentered by bundle_is_parsed_once_across_builders_in_a_fresh_process"]
    fn bundle_parse_child() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("warn")
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .try_init();
        let _ = configure(reqwest::Client::builder()).expect("shared builder fallback");
        let _ = build_client(reqwest::Client::builder().timeout(std::time::Duration::from_secs(1)))
            .expect("health builder fallback");
        let _ = configure(reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()))
            .expect("dedicated builder fallback");
        assert_eq!(PARSE_COUNT.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_configured_path_keeps_its_raw_os_bytes() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let raw = std::ffi::OsString::from_vec(b"ca-\xff.pem".to_vec());
        let path = configured_path(raw).expect("nonblank path");
        assert_eq!(path.as_os_str().as_bytes(), b"ca-\xff.pem");
    }

    #[test]
    fn parse_error_is_indexed_and_does_not_echo_bundle_content() {
        let path = Path::new("bundle.pem");
        let secret = b"-----BEGIN CERTIFICATE-----\ntoken=hunter2\n-----END CERTIFICATE-----\n";
        let error = parse_certificates(path, secret).expect_err("invalid PEM");
        let message = error.to_string();
        assert!(message.contains("certificate 1 has invalid PEM encoding"));
        assert!(!message.contains("hunter2"));
    }

    #[test]
    fn root_configuration_stays_additive() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let cargo = std::fs::read_to_string(root.join("Cargo.toml")).expect("read Cargo.toml");
        assert!(cargo.contains("\"rustls-tls\""));
        fn inspect(directory: &Path) {
            for entry in std::fs::read_dir(directory).expect("read source directory") {
                let path = entry.expect("source entry").path();
                if path.is_dir() {
                    inspect(&path);
                } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                    let source = std::fs::read_to_string(&path).expect("read Rust source");
                    for forbidden in [
                        ["tls_built_in_", "root_certs", "(false)"].concat(),
                        ["danger_accept_", "invalid_certs", "(true)"].concat(),
                        ["danger_accept_", "invalid_hostnames", "(true)"].concat(),
                    ] {
                        assert!(
                            !source.contains(&forbidden),
                            "{} contains {forbidden}",
                            path.display()
                        );
                    }
                }
            }
        }
        inspect(&root.join("src"));
    }
}
