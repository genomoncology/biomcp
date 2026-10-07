//! Tier 2 - sync outcome coverage over a local export fixture server.
//! Proves the sync loop attempts every export file, reports the true
//! per-file outcome, and fails with the failing file named when required
//! files are missing after the run (ticket 1304). No external network.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::*;
use crate::test_support::TempDirGuard;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

/// SAFETY: these tests own the `who_pq_sync_env` serial-test key, so the
/// process-global WHO Prequalification export URL variables are exclusive
/// while they run.
struct EnvRestore(Vec<(&'static str, Option<std::ffi::OsString>)>);
impl EnvRestore {
    fn set(&mut self, name: &'static str, value: impl AsRef<std::ffi::OsStr>) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: this test owns the who_pq_sync_env serial-test key.
        unsafe { std::env::set_var(name, value) };
    }
}
impl Drop for EnvRestore {
    fn drop(&mut self) {
        for (name, value) in self.0.drain(..).rev() {
            // SAFETY: this test owns the who_pq_sync_env serial-test key.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
    }
}

async fn serve_exports_once(
    fpp_body: String,
    api_body: String,
    vaccines_body: String,
    requests: Arc<AtomicUsize>,
) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind WHO PQ fixture server");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut request = [0_u8; 4096];
            let Ok(read) = stream.read(&mut request).await else {
                continue;
            };
            let request = String::from_utf8_lossy(&request[..read]).to_string();
            let body = if request.starts_with("GET /fpp.csv") {
                fpp_body.clone()
            } else if request.starts_with("GET /api.csv") {
                api_body.clone()
            } else if request.starts_with("GET /vaccines.csv") {
                vaccines_body.clone()
            } else {
                String::new()
            };
            requests.fetch_add(1, Ordering::SeqCst);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/csv\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    base
}

fn point_export_urls_at(env: &mut EnvRestore, base: &str) {
    env.set(WHO_PQ_EXPORT_URL_ENV, format!("{base}/fpp.csv"));
    env.set(WHO_PQ_API_EXPORT_URL_ENV, format!("{base}/api.csv"));
    env.set(WHO_VACCINES_EXPORT_URL_ENV, format!("{base}/vaccines.csv"));
    let cache = TempDirGuard::new("who-pq-sync-test-cache");
    env.set("BIOMCP_CACHE_DIR", cache.path());
}

#[tokio::test]
#[serial_test::serial(who_pq_sync_env)]
async fn sync_replays_the_recorded_captures_and_reports_every_file_refreshed() {
    let requests = Arc::new(AtomicUsize::new(0));
    let base = serve_exports_once(
        super::fixture_csv(),
        super::fixture_api_csv(),
        super::fixture_vaccine_csv(),
        Arc::clone(&requests),
    )
    .await;
    let root = TempDirGuard::new("who-pq-sync-replay");
    let mut env = EnvRestore(Vec::new());
    point_export_urls_at(&mut env, &base);

    let report = sync_who_pq_root(root.path(), WhoPqSyncMode::Force)
        .await
        .expect("recorded captures should sync");

    assert_eq!(report.refreshed, WHO_PQ_REQUIRED_FILES.to_vec());
    assert!(report.failed.is_empty());
    assert!(report.changed);
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    let rows = WhoPqClient::from_root(root.path())
        .read_rows()
        .expect("refreshed files should parse");
    assert!(!rows.is_empty());
}

#[tokio::test]
#[serial_test::serial(who_pq_sync_env)]
async fn sync_attempts_every_file_and_names_the_failing_file_when_required_files_are_missing() {
    let requests = Arc::new(AtomicUsize::new(0));
    let base = serve_exports_once(
        "wrong,header\n1,2\n".to_string(),
        super::fixture_api_csv(),
        super::fixture_vaccine_csv(),
        Arc::clone(&requests),
    )
    .await;
    let root = TempDirGuard::new("who-pq-sync-failure");
    let mut env = EnvRestore(Vec::new());
    point_export_urls_at(&mut env, &base);

    let err = sync_who_pq_root(root.path(), WhoPqSyncMode::Force)
        .await
        .expect_err("a missing required file must fail the sync");

    // The loop attempted every export instead of aborting at the first
    // failure, and the summary names the failing file.
    assert_eq!(requests.load(Ordering::SeqCst), 3);
    let detail = format!("{err:?}");
    assert!(detail.contains(WHO_PQ_CSV_FILE));
    assert!(detail.contains("Missing required WHO Prequalification file(s)"));
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    assert_eq!(err.exit_code(), 1);
    assert!(
        root.path().join(WHO_PQ_API_CSV_FILE).is_file()
            && root.path().join(WHO_VACCINES_CSV_FILE).is_file(),
        "later exports must still land when an earlier refresh fails"
    );
}

/// The recorded finished-pharma export with one column dropped, the shape a
/// real WHO export change produces. Validation must fail naming the file
/// and the missing column (ticket 2021).
fn fixture_csv_missing_basis_of_listing() -> String {
    let payload = super::fixture_csv();
    let (header, rest) = payload
        .split_once('\n')
        .expect("recorded fixture should carry a header row");
    let trimmed = header
        .replace(r#""Basis of Listing","#, "")
        .replace(r#""Basis of Listing""#, "");
    assert_ne!(
        trimmed, header,
        "the recorded fixture should still carry the Basis of Listing column"
    );
    format!("{trimmed}\n{rest}")
}

#[tokio::test]
#[serial_test::serial(who_pq_sync_env)]
async fn sync_names_the_failing_file_and_column_in_its_final_error() {
    let requests = Arc::new(AtomicUsize::new(0));
    let base = serve_exports_once(
        fixture_csv_missing_basis_of_listing(),
        super::fixture_api_csv(),
        super::fixture_vaccine_csv(),
        Arc::clone(&requests),
    )
    .await;
    let root = TempDirGuard::new("who-pq-sync-column");
    let mut env = EnvRestore(Vec::new());
    point_export_urls_at(&mut env, &base);

    let err = sync_who_pq_root(root.path(), WhoPqSyncMode::Force)
        .await
        .expect_err("an export missing a required column must fail the sync");

    assert_eq!(requests.load(Ordering::SeqCst), 3);
    assert!(matches!(err, BioMcpError::SourceUnavailable { .. }));
    assert_eq!(err.exit_code(), 1);
    // The final error carries its reason through the public surface
    // instead of the generic source-down line (ticket 2021).
    let projection = err.public_projection();
    assert!(
        projection
            .message
            .starts_with(WHO_PQ_SYNC_FAILURE_REASON_PREFIX),
        "projection: {}",
        projection.message
    );
    assert!(
        projection.message.contains("Refresh failed for who_pq.csv"),
        "projection: {}",
        projection.message
    );
    assert!(
        projection
            .message
            .contains("missing required column BASIS OF LISTING"),
        "projection: {}",
        projection.message
    );
    assert!(
        projection
            .message
            .contains("Missing required WHO Prequalification file(s)"),
        "projection: {}",
        projection.message
    );
    assert!(
        projection
            .recovery
            .is_some_and(|recovery| recovery.contains("network access")),
        "recovery: {:?}",
        projection.recovery
    );
    // The rendered error names the file and column once, with no doubled
    // period from the recovery suffix (the slip finding 12 reported).
    let rendered = err.to_string();
    assert!(rendered.contains("who_pq.csv"));
    assert!(!rendered.contains(".."));
}

#[tokio::test]
#[serial_test::serial(who_pq_sync_env)]
async fn sync_partial_run_keeps_older_data_and_reports_the_failure_reason_per_file() {
    let requests = Arc::new(AtomicUsize::new(0));
    let base = serve_exports_once(
        fixture_csv_missing_basis_of_listing(),
        super::fixture_api_csv(),
        super::fixture_vaccine_csv(),
        Arc::clone(&requests),
    )
    .await;
    let root = TempDirGuard::new("who-pq-sync-partial");
    // An older good copy keeps the run partial instead of failing it.
    std::fs::write(root.path().join(WHO_PQ_CSV_FILE), super::fixture_csv())
        .expect("seed older WHO export");
    let mut env = EnvRestore(Vec::new());
    point_export_urls_at(&mut env, &base);

    let report = sync_who_pq_root(root.path(), WhoPqSyncMode::Force)
        .await
        .expect("a partial refresh with local data must not fail the sync");

    assert_eq!(
        report.refreshed,
        vec![WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE]
    );
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].file, WHO_PQ_CSV_FILE);
    assert!(
        report.failed[0]
            .reason
            .contains("missing required column BASIS OF LISTING"),
        "reason: {}",
        report.failed[0].reason
    );
    assert!(
        report
            .failure_sentences()
            .contains("Refresh failed for who_pq.csv: WHO Prequalification export headers did not match: who_pq.csv: missing required column BASIS OF LISTING."),
        "sentences: {}",
        report.failure_sentences()
    );
    // The older copy survives the failed refresh.
    assert!(root.path().join(WHO_PQ_CSV_FILE).is_file());
}

#[test]
fn summary_line_names_refreshed_and_failed_files() {
    let report = WhoPqSyncReport {
        refreshed: vec![WHO_PQ_CSV_FILE, WHO_PQ_API_CSV_FILE],
        failed: vec![WhoPqSyncFileFailure {
            file: WHO_VACCINES_CSV_FILE,
            reason: "API request to who-prequalification failed".to_string(),
        }],
        changed: true,
    };
    assert_eq!(
        report.summary_line(),
        "WHO Prequalification sync outcome: refreshed who_pq.csv, who_api.csv; failed who_vaccines.csv."
    );
}
