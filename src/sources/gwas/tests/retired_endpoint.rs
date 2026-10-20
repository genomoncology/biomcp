//! The retired legacy by-rsID endpoint: upstream
//! retired the GWAS Catalog REST API in 2026-10 with HTTP 410, so a live
//! `get variant <rsid> all` hard-failed on the dead source. The reader
//! degrades only the retired class (410) with the honest gap note instead
//! of failing the whole variant answer, while a 404 keeps meaning no
//! associations for the rsID the way it always did.

use super::super::*;
use crate::test_support::FixtureEnv;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;

macro_rules! retired_body {
    () => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gwas/associations_retired_410_body.txt"
        ))
    };
}

async fn retired_endpoint_server(
    status_line: &'static str,
) -> (
    String,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let task = crate::test_support::TestHttpServer::spawn(move |mut stream| {
        let captured = captured.clone();
        async move {
            let Some(request) = crate::test_support::read_request(&mut stream).await? else {
                return Ok(());
            };

            let request = String::from_utf8_lossy(&request).into_owned();
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
                .await?;
            Ok(())
        }
    })
    .expect("start HTTP fixture");
    let base = task.base.clone();
    (base, requests, task)
}

async fn associations_by_rsid_against(
    status_line: &'static str,
) -> (
    Result<Vec<GwasAssociation>, BioMcpError>,
    Arc<Mutex<Vec<String>>>,
    crate::test_support::TestHttpServer,
) {
    let (base, requests, server) = retired_endpoint_server(status_line).await;
    let mut env = FixtureEnv::new();
    env.set("BIOMCP_GWAS_BASE", &base);
    let client = GwasClient::new().expect("client against fixture");
    let result = client.associations_by_rsid("rs121913529", 20).await;
    drop(env);
    (result, requests, server)
}

/// The live shape: 410 Gone names the retired endpoint, so the reader
/// returns the honest retirement note instead of the hard HTTP failure a
/// bare Api error would carry into every `get variant ... all` answer.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn gone_by_rsid_endpoint_degrades_with_the_retirement_note() {
    let (result, requests, mut server) = associations_by_rsid_against("410 Gone").await;
    server.finish().unwrap();
    let error = result.expect_err("the retired endpoint degrades as an error");
    assert_eq!(error.code(), "source_unavailable");
    let detail = format!("{error:?}");
    assert!(
        detail.contains("legacy GWAS Catalog by-rsID endpoint"),
        "{detail}"
    );
    assert!(detail.contains("retired upstream"), "{detail}");
    assert!(detail.contains("HTTP 410"), "{detail}");
    let requests = requests.lock().expect("lock fixture requests").join("\n");
    assert!(
        requests.contains("/singleNucleotidePolymorphisms/rs121913529/associations"),
        "the by-rsID read must run before it degrades: {requests}"
    );
    assert!(
        crate::sources::gwas::is_retired_endpoint_error(&error),
        "the retired degrade marks itself permanent so no retry is offered"
    );
}

/// A 404 keeps meaning no associations for the rsID: it always did,
/// and the retired-endpoint degrade must not
/// swallow it, because a missing rsID is an answer, not a retired route.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn not_found_by_rsid_endpoint_means_no_associations() {
    let (result, _requests, mut server) = associations_by_rsid_against("404 Not Found").await;
    server.finish().unwrap();
    let associations = result.expect("a 404 settles as no associations");
    assert!(
        associations.is_empty(),
        "a 404 carries no association rows: {associations:?}"
    );
}
