//! Proves ordinary provider clients reach a private-CA endpoint only when the
//! operator CA bundle names that certificate authority.

use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

const FAERS_REPORT_PAGE: &str = r#"{
  "meta":{"results":{"skip":0,"limit":1,"total":1}},
  "results":[{
    "safetyreportid":"1001",
    "serious":"1",
    "receivedate":"20250101",
    "patient":{
      "reaction":[{"reactionmeddrapt":"Rash"}],
      "drug":[
        {"medicinalproduct":"DRUG NAME","drugcharacterization":"1","drugindication":"LUNG CANCER"},
        {"medicinalproduct":"OTHER DRUG","drugcharacterization":"2"}
      ]
    }
  }]
}"#;

const CTGOV_DOCUMENT_STUDY: &str = r#"{
  "protocolSection":{"identificationModule":{"nctId":"NCT00000001"}},
  "documentSection":{"largeDocumentModule":{"largeDocs":[
    {"typeAbbrev":"Prot","filename":"protocol.pdf","size":17}
  ]}}
}"#;

const MYCHEM_IMATINIB: &str =
    include_str!("../testdata/sources/mychem/query_imatinib_get_20260811.json");

struct TlsFixture {
    origin: String,
    bundle: PathBuf,
    connections: Arc<AtomicUsize>,
    sessions: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
    server: tokio::task::JoinHandle<()>,
    _dir: tempfile::TempDir,
}

impl TlsFixture {
    async fn start() -> Self {
        Self::start_with_body(FAERS_REPORT_PAGE).await
    }

    async fn start_with_body(body: &'static str) -> Self {
        let dir = tempfile::tempdir().expect("fixture directory");
        let material = tls_material();
        let bundle = dir.path().join("ca.pem");
        std::fs::write(&bundle, material.ca_pem.as_bytes()).expect("write CA bundle");

        let certificates = vec![rustls::pki_types::CertificateDer::from(material.leaf_der)];
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(
            rustls::pki_types::PrivatePkcs8KeyDer::from(material.leaf_key_der),
        );
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certificates, key)
            .expect("fixture server config");

        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind TLS fixture");
        let address = listener.local_addr().expect("fixture address");
        let connections = Arc::new(AtomicUsize::new(0));
        let sessions = Arc::new(AtomicUsize::new(0));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let server = tokio::spawn(serve(
            listener,
            acceptor,
            Arc::clone(&connections),
            Arc::clone(&sessions),
            Arc::clone(&requests),
            body,
        ));
        Self {
            origin: format!("https://{address}"),
            bundle,
            connections,
            sessions,
            requests,
            server,
            _dir: dir,
        }
    }

    async fn run_command(&self, env: &[(&str, &str)], args: &[&str]) -> Output {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_biomcp"));
        command
            .env_remove("BIOMCP_CA_BUNDLE")
            .env_remove("SSL_CERT_FILE")
            .env("BIOMCP_CA_BUNDLE", &self.bundle)
            .env("BIOMCP_TEST_UNPACED_ORIGIN", &self.origin)
            .env("RUST_LOG", "warn")
            .env("NO_PROXY", "*")
            .env("no_proxy", "*")
            .args(args);
        for (name, value) in env {
            command.env(name, value);
        }
        command.output().await.expect("run biomcp")
    }

    async fn run(&self, explicit: Option<&Path>, fallback: Option<&Path>, json: bool) -> Output {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_biomcp"));
        command
            .env_remove("BIOMCP_CA_BUNDLE")
            .env_remove("SSL_CERT_FILE")
            .env("BIOMCP_OPENFDA_BASE", &self.origin)
            .env("RUST_LOG", "warn")
            .env("NO_PROXY", "*")
            .env("no_proxy", "*");
        if let Some(bundle) = explicit {
            command.env("BIOMCP_CA_BUNDLE", bundle);
        }
        if let Some(bundle) = fallback {
            command.env("SSL_CERT_FILE", bundle);
        }
        if json {
            command.arg("--json");
        }
        command.args(["--no-cache", "get", "adverse-event", "1001", "reactions"]);
        command.output().await.expect("run biomcp")
    }
}

impl Drop for TlsFixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn serve(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    connections: Arc<AtomicUsize>,
    sessions: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
    body: &'static str,
) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        connections.fetch_add(1, Ordering::SeqCst);
        let acceptor = acceptor.clone();
        let sessions = Arc::clone(&sessions);
        let requests = Arc::clone(&requests);
        tokio::spawn(async move {
            let Ok(mut stream) = acceptor.accept(stream).await else {
                return;
            };
            sessions.fetch_add(1, Ordering::SeqCst);
            let mut request = vec![0_u8; 8192];
            if let Ok(size) = stream.read(&mut request).await
                && let Some(line) = String::from_utf8_lossy(&request[..size]).lines().next()
            {
                requests.lock().expect("request log").push(line.to_string());
            }
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.shutdown().await;
        });
    }
}

struct TlsMaterial {
    ca_pem: String,
    leaf_der: Vec<u8>,
    leaf_key_der: Vec<u8>,
}

fn tls_material() -> TlsMaterial {
    let ca_key = rcgen::KeyPair::generate().expect("CA key");
    let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new()).expect("CA parameters");
    ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
    let ca_cert = ca_params.self_signed(&ca_key).expect("CA certificate");

    let leaf_key = rcgen::KeyPair::generate().expect("leaf key");
    let leaf_params =
        rcgen::CertificateParams::new(vec!["127.0.0.1".to_string()]).expect("leaf parameters");
    let leaf_cert = leaf_params
        .signed_by(&leaf_key, &ca_cert, &ca_key)
        .expect("leaf certificate");

    TlsMaterial {
        ca_pem: ca_cert.pem(),
        leaf_der: leaf_cert.der().to_vec(),
        leaf_key_der: leaf_key.serialize_der(),
    }
}

fn assert_named_path(output: &Output, bundle: &Path) {
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(bundle.to_str().expect("bundle path")),
        "stderr did not name the bundle: {stderr}"
    );
}

/// Writes the five confirmed-unparseable bundle inputs: an empty file, one
/// bad certificate among good ones, a leading byte-order mark, an OpenSSL
/// `BEGIN TRUSTED CERTIFICATE` file, and a raw DER file.
fn write_broken_bundles(dir: &Path, good_pem: &[u8], der: &[u8]) -> Vec<PathBuf> {
    let empty = dir.join("empty.pem");
    std::fs::write(&empty, b"").expect("write empty bundle");

    let mixed = dir.join("mixed.pem");
    let mut contents = good_pem.to_vec();
    contents
        .extend_from_slice(b"-----BEGIN CERTIFICATE-----\nnot base64\n-----END CERTIFICATE-----\n");
    std::fs::write(&mixed, contents).expect("write mixed bundle");

    let bom = dir.join("bom.pem");
    let mut contents = b"\xEF\xBB\xBF".to_vec();
    contents.extend_from_slice(good_pem);
    std::fs::write(&bom, contents).expect("write BOM bundle");

    let trusted = dir.join("trusted.pem");
    std::fs::write(
        &trusted,
        b"-----BEGIN TRUSTED CERTIFICATE-----\nb3RoZXJjZXJ0\n-----END TRUSTED CERTIFICATE-----\n",
    )
    .expect("write trusted bundle");

    let der_file = dir.join("der.pem");
    std::fs::write(&der_file, der).expect("write DER bundle");

    vec![empty, mixed, bom, trusted, der_file]
}

#[tokio::test]
async fn configured_bundle_reaches_the_private_ca_fixture() {
    let fixture = TlsFixture::start().await;
    let output = fixture.run(Some(&fixture.bundle), None, false).await;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rash"), "stdout={stdout}");
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn health_probe_reaches_a_private_ca_provider_through_the_orphan_client() {
    // The health runner probes the orphan endpoint through its own client
    // construction (src/sources/fda_orphan.rs:667-672), not the shared
    // health HTTP client, so this pins the probe path rather than
    // health_http_client's transport; that client has no endpoint
    // override and stays covered by the startup and policy tests.
    let fixture = TlsFixture::start().await;
    let output = fixture
        .run_command(
            &[("BIOMCP_FDA_ORPHAN_BASE", &fixture.origin)],
            &["health", "--api", "FDA Orphan Drug Designations"],
        )
        .await;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn fda_orphan_client_reaches_a_private_ca_provider() {
    let fixture = TlsFixture::start_with_body(MYCHEM_IMATINIB).await;
    let output = fixture
        .run_command(
            &[
                ("BIOMCP_MYCHEM_BASE", &fixture.origin),
                ("BIOMCP_OPENFDA_BASE", &fixture.origin),
                ("BIOMCP_FDA_ORPHAN_BASE", &fixture.origin),
            ],
            &["--no-cache", "get", "drug", "imatinib", "regulatory"],
        )
        .await;
    let requests = fixture.requests.lock().expect("request log");
    assert!(
        requests
            .iter()
            .any(|request| request.contains("OOPD_Results.cfm")),
        "FDA orphan request missing; status={:?}, stderr={}, requests={requests:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn trial_document_client_reaches_a_private_ca_provider() {
    let fixture = TlsFixture::start_with_body(CTGOV_DOCUMENT_STUDY).await;
    let output = fixture
        .run_command(
            &[
                ("BIOMCP_CTGOV_BASE", &fixture.origin),
                ("BIOMCP_CTGOV_CDN_BASE", &fixture.origin),
            ],
            &[
                "--no-cache",
                "get",
                "trial",
                "NCT00000001",
                "document",
                "protocol.pdf",
            ],
        )
        .await;
    let requests = fixture.requests.lock().expect("request log");
    assert!(
        requests
            .iter()
            .any(|request| request.contains("protocol.pdf")),
        "document CDN request missing; status={:?}, stderr={}, requests={requests:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn orcid_client_reaches_a_private_ca_provider() {
    let fixture = TlsFixture::start().await;
    let output = fixture
        .run_command(
            &[
                ("BIOMCP_ORCID_BASE", &fixture.origin),
                ("ORCID_ACCESS_TOKEN", "fixture-token"),
            ],
            &["--no-cache", "get", "author", "orcid:0000-0002-1825-0097"],
        )
        .await;
    let requests = fixture.requests.lock().expect("request log");
    assert!(
        requests.iter().any(|request| request.contains("/person")),
        "ORCID request missing; status={:?}, stderr={}, requests={requests:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn clingen_cspec_client_reaches_a_private_ca_provider() {
    let fixture = TlsFixture::start().await;
    let output = fixture
        .run_command(
            &[("BIOMCP_CSPEC_FIXTURE_ORIGIN", &fixture.origin)],
            &["--no-cache", "gene", "cspec", "ATM"],
        )
        .await;
    let requests = fixture.requests.lock().expect("request log");
    assert!(
        requests
            .iter()
            .any(|request| request.contains("/cspec/Gene/id/ATM/")),
        "CSpec request missing; status={:?}, stderr={}, requests={requests:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn missing_bundle_fails_before_a_completed_handshake() {
    let fixture = TlsFixture::start().await;
    let output = fixture.run(None, None, false).await;
    assert_eq!(output.status.code(), Some(1));
    assert!(
        fixture.connections.load(Ordering::SeqCst) >= 1,
        "the untrusted case must reach the fixture's TLS handshake"
    );
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn broken_bundles_fail_before_any_connection() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("absent.pem");
    let good_pem = std::fs::read(&fixture.bundle).expect("read good bundle");
    let der = tls_material().leaf_der;
    let broken = write_broken_bundles(dir.path(), &good_pem, &der);

    for bundle in std::iter::once(&missing).chain(broken.iter()) {
        let output = fixture.run(Some(bundle), None, false).await;
        assert_named_path(&output, bundle);
    }
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn unreadable_bundle_fails_before_any_connection() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let unreadable = dir.path().join("unreadable.pem");
    std::fs::write(&unreadable, b"unused").expect("write unreadable bundle");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000))
        .expect("make bundle unreadable");
    if std::fs::read(&unreadable).is_ok() {
        // A privileged runner ignores the mode, so this case proves nothing.
        return;
    }

    let output = fixture.run(Some(&unreadable), None, false).await;
    assert_named_path(&output, &unreadable);
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn broken_bundle_json_names_the_path() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("absent.pem");
    let output = fixture.run(Some(&missing), None, true).await;
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON error");
    assert_eq!(value["error"]["code"], "ca_bundle");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains(missing.to_str().expect("bundle path"))),
        "message did not name the bundle: {}",
        value["error"]["message"]
    );
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn invalid_explicit_bundle_fails_closed_even_with_a_valid_fallback() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let invalid = dir.path().join("invalid.pem");
    std::fs::write(&invalid, b"not a certificate").expect("write invalid bundle");
    let output = fixture
        .run(Some(&invalid), Some(&fixture.bundle), false)
        .await;
    assert_named_path(&output, &invalid);
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn fallback_bundle_parses_and_reaches_the_private_ca_fixture() {
    let fixture = TlsFixture::start().await;
    let output = fixture.run(None, Some(&fixture.bundle), false).await;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rash"), "stdout={stdout}");
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn broken_fallback_bundles_warn_and_continue() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let good_pem = std::fs::read(&fixture.bundle).expect("read good bundle");
    let der = tls_material().leaf_der;
    let mut broken = write_broken_bundles(dir.path(), &good_pem, &der);
    broken.push(dir.path().join("missing.pem"));
    broken.push(dir.path().join("directory"));
    std::fs::create_dir(broken.last().expect("directory path")).expect("create directory");
    broken.push(PathBuf::from("   "));

    for bundle in &broken {
        let before = fixture.connections.load(Ordering::SeqCst);
        let output = fixture.run(None, Some(bundle), false).await;
        let stderr = String::from_utf8_lossy(&output.stderr);
        // The dropped fallback degrades to an ordinary untrusted-connection
        // failure: the client builds with the bundled roots and attempts the
        // handshake (connections advance, no session completes). A fail-closed
        // bundle error aborts before any connection, and a silent drop would
        // leave no warning, so both the per-case connection delta and the
        // degrade warning text are asserted.
        assert_eq!(output.status.code(), Some(1), "stderr={stderr}");
        assert!(
            stderr.contains("SSL_CERT_FILE is not a usable certificate bundle")
                || stderr.contains("SSL_CERT_FILE could not be read")
                || stderr.contains("SSL_CERT_FILE is blank"),
            "expected the degrade warning on stderr, got: {stderr}"
        );
        assert!(
            stderr.contains("continuing with the bundled TLS roots"),
            "expected the continue note on stderr, got: {stderr}"
        );
        assert!(
            fixture.connections.load(Ordering::SeqCst) > before,
            "each dropped fallback must still attempt its connections"
        );
        assert_eq!(fixture.sessions.load(Ordering::SeqCst), 0);
    }
}

fn server_command(
    args: &[&str],
    explicit: Option<&Path>,
    fallback: Option<&Path>,
) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command
        .args(args)
        .env_remove("BIOMCP_CA_BUNDLE")
        .env_remove("SSL_CERT_FILE")
        .env("RUST_LOG", "warn")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(path) = explicit {
        command.env("BIOMCP_CA_BUNDLE", path);
    }
    if let Some(path) = fallback {
        command.env("SSL_CERT_FILE", path);
    }
    command
}

async fn stop_and_stderr(child: &mut tokio::process::Child) -> String {
    let _ = tokio::time::timeout(Duration::from_secs(2), child.kill()).await;
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;
    let mut stderr = String::new();
    if let Some(mut pipe) = child.stderr.take() {
        let _ =
            tokio::time::timeout(Duration::from_secs(2), pipe.read_to_string(&mut stderr)).await;
    }
    stderr
}

#[tokio::test]
async fn stdio_rejects_invalid_explicit_bundle_before_session_acceptance() {
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("missing.pem");
    let mut child = server_command(&["serve"], Some(&missing), None)
        .spawn()
        .expect("spawn stdio");
    let status = tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .expect("stdio exit deadline")
        .expect("stdio exit");
    assert!(!status.success());
    let stderr = stop_and_stderr(&mut child).await;
    assert!(
        stderr.contains("CA bundle") && stderr.contains("could not be read"),
        "{stderr}"
    );
}

#[tokio::test]
async fn stdio_bad_fallback_starts_and_warns_once() {
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("missing.pem");
    let mut child = server_command(&["serve"], None, Some(&missing))
        .spawn()
        .expect("spawn stdio");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(child.try_wait().expect("poll stdio").is_none());
    let stderr = stop_and_stderr(&mut child).await;
    assert_eq!(
        stderr.matches("SSL_CERT_FILE could not be read").count(),
        1,
        "{stderr}"
    );
}

fn unused_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .expect("reserve port")
        .local_addr()
        .expect("port")
        .port()
}

#[tokio::test]
async fn http_rejects_invalid_explicit_bundle_before_bind() {
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("missing.pem");
    let port = unused_port();
    let port_text = port.to_string();
    let mut child = server_command(
        &["serve-http", "--host", "127.0.0.1", "--port", &port_text],
        Some(&missing),
        None,
    )
    .spawn()
    .expect("spawn HTTP");
    let status = tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .expect("HTTP exit deadline")
        .expect("HTTP exit");
    assert!(!status.success());
    assert!(
        tokio::time::timeout(
            Duration::from_secs(1),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .expect("pre-bind connect deadline")
        .is_err()
    );
    let stderr = stop_and_stderr(&mut child).await;
    assert!(
        stderr.contains("CA bundle") && stderr.contains("could not be read"),
        "{stderr}"
    );
}

#[tokio::test]
async fn http_bad_fallback_binds_and_warns_once() {
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("missing.pem");
    let port = unused_port();
    let port_text = port.to_string();
    let mut child = server_command(
        &["serve-http", "--host", "127.0.0.1", "--port", &port_text],
        None,
        Some(&missing),
    )
    .spawn()
    .expect("spawn HTTP");
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if tokio::time::timeout(
            Duration::from_secs(1),
            tokio::net::TcpStream::connect(("127.0.0.1", port)),
        )
        .await
        .expect("readiness connect deadline")
        .is_ok()
        {
            break;
        }
        assert!(Instant::now() < deadline, "HTTP readiness deadline");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    let stderr = stop_and_stderr(&mut child).await;
    assert_eq!(
        stderr.matches("SSL_CERT_FILE could not be read").count(),
        1,
        "{stderr}"
    );
}

#[tokio::test]
async fn broken_bundle_json_parse_failure_names_the_path() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let der_file = dir.path().join("der.pem");
    std::fs::write(&der_file, tls_material().leaf_der).expect("write DER bundle");
    let output = fixture.run(Some(&der_file), None, true).await;
    assert_eq!(output.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON error");
    assert_eq!(value["error"]["code"], "ca_bundle");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains(der_file.to_str().expect("bundle path"))),
        "message did not name the bundle: {}",
        value["error"]["message"]
    );
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
}
