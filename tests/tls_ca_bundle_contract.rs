//! Proves ordinary provider clients reach a private-CA endpoint only when the
//! operator CA bundle names that certificate authority.

use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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

struct TlsFixture {
    origin: String,
    bundle: PathBuf,
    connections: Arc<AtomicUsize>,
    sessions: Arc<AtomicUsize>,
    server: tokio::task::JoinHandle<()>,
    _dir: tempfile::TempDir,
}

impl TlsFixture {
    async fn start() -> Self {
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
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let server = tokio::spawn(serve(
            listener,
            acceptor,
            Arc::clone(&connections),
            Arc::clone(&sessions),
        ));
        Self {
            origin: format!("https://{address}"),
            bundle,
            connections,
            sessions,
            server,
            _dir: dir,
        }
    }

    async fn run(&self, bundle: Option<&Path>, json: bool) -> Output {
        let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_biomcp"));
        command
            .env_remove("BIOMCP_CA_BUNDLE")
            .env_remove("SSL_CERT_FILE")
            .env("BIOMCP_OPENFDA_BASE", &self.origin)
            .env("NO_PROXY", "*")
            .env("no_proxy", "*");
        if let Some(bundle) = bundle {
            command.env("BIOMCP_CA_BUNDLE", bundle);
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
) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        connections.fetch_add(1, Ordering::SeqCst);
        let acceptor = acceptor.clone();
        let sessions = Arc::clone(&sessions);
        tokio::spawn(async move {
            let Ok(mut stream) = acceptor.accept(stream).await else {
                return;
            };
            sessions.fetch_add(1, Ordering::SeqCst);
            let mut request = vec![0_u8; 8192];
            let _ = stream.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                FAERS_REPORT_PAGE.len(),
                FAERS_REPORT_PAGE
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

#[tokio::test]
async fn configured_bundle_reaches_the_private_ca_fixture() {
    let fixture = TlsFixture::start().await;
    let output = fixture.run(Some(&fixture.bundle), false).await;
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
async fn missing_bundle_fails_before_a_completed_handshake() {
    let fixture = TlsFixture::start().await;
    let output = fixture.run(None, false).await;
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
    let malformed = dir.path().join("malformed.pem");
    std::fs::write(
        &malformed,
        b"-----BEGIN CERTIFICATE-----\nnot base64\n-----END CERTIFICATE-----\n",
    )
    .expect("write malformed bundle");
    let empty = dir.path().join("empty.pem");
    std::fs::write(&empty, b"").expect("write certificate-less bundle");

    for bundle in [&missing, &malformed, &empty] {
        let output = fixture.run(Some(bundle), false).await;
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

    let output = fixture.run(Some(&unreadable), false).await;
    assert_named_path(&output, &unreadable);
    assert_eq!(fixture.connections.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.sessions.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn broken_bundle_json_names_the_path() {
    let fixture = TlsFixture::start().await;
    let dir = tempfile::tempdir().expect("bundle directory");
    let missing = dir.path().join("absent.pem");
    let output = fixture.run(Some(&missing), true).await;
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
