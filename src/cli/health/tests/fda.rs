//! FDA orphan-designation probe tests for `biomcp health`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::HealthStatus;
use super::super::catalog::SourceDescriptor;
use super::super::runner::{probe_source, report_from_outcomes};

struct FdaBaseGuard(Option<std::ffi::OsString>);

impl FdaBaseGuard {
    fn set(value: &str) -> Self {
        let old = std::env::var_os("BIOMCP_FDA_ORPHAN_BASE");
        unsafe { std::env::set_var("BIOMCP_FDA_ORPHAN_BASE", value) };
        Self(old)
    }
}

impl Drop for FdaBaseGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.0 {
                Some(value) => std::env::set_var("BIOMCP_FDA_ORPHAN_BASE", value),
                None => std::env::remove_var("BIOMCP_FDA_ORPHAN_BASE"),
            }
        }
    }
}

async fn health_fda_server(
    status: axum::http::StatusCode,
    body: String,
) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let requests = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&requests);
    let response_body = Arc::new(body);
    let app = axum::Router::new().route(
        "/OOPD_Results.cfm",
        axum::routing::post(move |request_body: axum::body::Bytes| {
            let seen = Arc::clone(&seen);
            let response_body = Arc::clone(&response_body);
            async move {
                seen.fetch_add(1, Ordering::SeqCst);
                assert_eq!(
                    String::from_utf8_lossy(&request_body),
                    "Product_name=eflornithine+hydrochloride&sponsor_name=&Designation=&Designation_Start_Date=&Designation_End_Date=&Search_param=DESDATE&Output_Format=Excel&Sort_order=GENERIC_NAME&RecordsPerPage=25&newSearch=Run+Search"
                );
                (
                    status,
                    response_body.as_str().to_owned(),
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), requests, task)
}

/// The FDA site answers scripted clients with a redirect: the probe does
/// not follow it, and the health row names the redirect it got.
async fn health_fda_redirect_server() -> (String, tokio::task::JoinHandle<()>) {
    let app = axum::Router::new().route(
        "/OOPD_Results.cfm",
        axum::routing::post(|| async {
            (
                axum::http::StatusCode::FOUND,
                [(axum::http::header::LOCATION, "/blocked/abuse-detection")],
                String::new(),
            )
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), task)
}

fn fda_orphan_source() -> &'static SourceDescriptor {
    super::super::catalog::HEALTH_SOURCES
        .iter()
        .find(|source| source.api == "FDA Orphan Drug Designations")
        .unwrap()
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_redirect_answer_names_the_block_in_the_row() {
    let source = fda_orphan_source();
    let (base, server) = health_fda_redirect_server().await;
    let _base = FdaBaseGuard::set(&base);
    let report = report_from_outcomes(vec![probe_source(reqwest::Client::new(), source).await]);
    server.abort();
    let row = &report.rows[0];
    assert_eq!((report.healthy, report.error, report.total), (0, 1, 1));
    assert_eq!(row.status, HealthStatus::Error);
    assert!(
        row.latency
            .contains("(HTTP 302, blocked: the FDA site redirects scripted clients)"),
        "the row names the redirect it got: {}",
        row.latency
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn fda_orphan_fixture_probe_reconciles_rows_counts_and_fail_policy() {
    let source = fda_orphan_source();
    let fixture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/fda_orphan/provider-shaped.html"
    ));
    let (base, requests, server) =
        health_fda_server(axum::http::StatusCode::OK, fixture.into()).await;
    let _base = FdaBaseGuard::set(&base);
    let good = probe_source(reqwest::Client::new(), source).await;
    server.abort();
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    let report = report_from_outcomes(vec![good]);
    assert_eq!((report.healthy, report.error, report.total), (1, 0, 1));
    assert_eq!(report.rows[0].api, "FDA Orphan Drug Designations");
    assert!(report.rows[0].latency.ends_with("ms"));
    let json: serde_json::Value = serde_json::from_str(&report.to_json(true).unwrap()).unwrap();
    assert_eq!(
        (json["ok"].as_bool(), json["exit_policy"].as_str()),
        (Some(true), Some("fail_on_error"))
    );

    let (base, _, server) = health_fda_server(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        fixture.into(),
    )
    .await;
    let _base = FdaBaseGuard::set(&base);
    let bad = report_from_outcomes(vec![probe_source(reqwest::Client::new(), source).await]);
    server.abort();
    assert_eq!((bad.healthy, bad.error, bad.total), (0, 1, 1));
    assert!(
        bad.to_markdown_with_policy(true)
            .contains("Exit policy: fail_on_error; result: errors present")
    );
    let json: serde_json::Value = serde_json::from_str(&bad.to_json(true).unwrap()).unwrap();
    assert_eq!(json["ok"], false);

    let malformed = fixture.replace("03/11/2024", "not-a-date");
    let (base, _, server) = health_fda_server(axum::http::StatusCode::OK, malformed).await;
    let _base = FdaBaseGuard::set(&base);
    let malformed = report_from_outcomes(vec![probe_source(reqwest::Client::new(), source).await]);
    server.abort();
    assert_eq!(
        (malformed.healthy, malformed.error, malformed.total),
        (0, 1, 1)
    );
}
