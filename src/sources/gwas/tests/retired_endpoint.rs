//! The retired legacy by-rsID endpoint (ticket 2047): upstream retired the
//! GWAS Catalog REST API in 2026-10 with HTTP 410, so a live
//! `get variant <rsid> all` hard-failed on the dead source. The reader
//! degrades the retired class (410, and a 404 the same way) with the honest
//! gap note instead of failing the whole variant answer.

use super::super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

macro_rules! retired_body {
    () => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gwas/associations_retired_410_body.txt"
        ))
    };
}

struct GwasBaseEnv(Option<std::ffi::OsString>);

impl GwasBaseEnv {
    fn set(&mut self, value: &str) {
        self.0 = std::env::var_os("BIOMCP_GWAS_BASE");
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe { std::env::set_var("BIOMCP_GWAS_BASE", value) };
    }
}

impl Drop for GwasBaseEnv {
    fn drop(&mut self) {
        // SAFETY: this test holds the serial-test process-wide environment lock.
        unsafe {
            if let Some(value) = self.0.take() {
                std::env::set_var("BIOMCP_GWAS_BASE", value);
            } else {
                std::env::remove_var("BIOMCP_GWAS_BASE");
            }
        }
    }
}

async fn retired_endpoint_server(
    status_line: &'static str,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind retired endpoint fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let len = stream.read(&mut request).await.expect("read request");
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                captured
                    .lock()
                    .expect("lock fixture requests")
                    .push(request.clone());
                let body = retired_body!();
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: text/plain\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, requests, task)
}

async fn associations_by_rsid_against(
    status_line: &'static str,
) -> (
    BioMcpError,
    Arc<Mutex<Vec<String>>>,
    tokio::task::JoinHandle<()>,
) {
    let (base, requests, server) = retired_endpoint_server(status_line).await;
    let mut env = GwasBaseEnv(None);
    env.set(&base);
    let client = GwasClient::new().expect("client against fixture");
    let error = client
        .associations_by_rsid("rs121913529", 20)
        .await
        .expect_err("the retired endpoint degrades as an error, not an empty page");
    drop(env);
    (error, requests, server)
}

/// The live shape: 410 Gone names the retired endpoint, so the reader
/// returns the honest retirement note instead of the hard HTTP failure a
/// bare Api error would carry into every `get variant ... all` answer.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn gone_by_rsid_endpoint_degrades_with_the_retirement_note() {
    let (error, requests, server) = associations_by_rsid_against("410 Gone").await;
    server.abort();
    assert_eq!(error.code(), "source_unavailable");
    let detail = format!("{error:?}");
    assert!(
        detail.contains("legacy GWAS Catalog REST endpoint"),
        "{detail}"
    );
    assert!(detail.contains("retired upstream"), "{detail}");
    assert!(detail.contains("HTTP 410"), "{detail}");
    let requests = requests.lock().expect("lock fixture requests").join("\n");
    assert!(
        requests.contains("/singleNucleotidePolymorphisms/rs121913529/associations"),
        "the by-rsID read must run before it degrades: {requests}"
    );
}

/// A gateway that answers the retired endpoint with 404 degrades the same
/// way: the legacy by-rsID route is gone, and that status no longer reads
/// as an empty association page for this endpoint (ticket 2047).
#[tokio::test]
#[serial_test::serial(source_env)]
async fn not_found_by_rsid_endpoint_degrades_with_the_retirement_note() {
    let (error, _requests, server) = associations_by_rsid_against("404 Not Found").await;
    server.abort();
    assert_eq!(error.code(), "source_unavailable");
    let detail = format!("{error:?}");
    assert!(detail.contains("retired upstream"), "{detail}");
}
