use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::model::{GenCcDataset, HEADER};
use super::{
    ENDPOINT, inside_retry_window, is_fresh, same_endpoint, valid_endpoint, valid_etag,
    valid_http_date,
};

fn fixture() -> &'static [u8] {
    include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/testdata/sources/gencc/submissions-new-odc1.csv"
    ))
}

fn fixture_records() -> (csv::StringRecord, Vec<csv::StringRecord>) {
    let mut reader = csv::Reader::from_reader(fixture());
    let header = reader.headers().unwrap().clone();
    let rows = reader.records().collect::<Result<Vec<_>, _>>().unwrap();
    (header, rows)
}

fn csv_bytes(header: &csv::StringRecord, rows: &[csv::StringRecord]) -> Vec<u8> {
    let mut writer = csv::WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());
    writer.write_record(header).unwrap();
    for row in rows {
        writer.write_record(row).unwrap();
    }
    writer.into_inner().unwrap()
}

fn one_row() -> (csv::StringRecord, csv::StringRecord) {
    let (header, rows) = fixture_records();
    (header, rows[0].clone())
}

fn set(row: &mut csv::StringRecord, index: usize, value: impl Into<String>) {
    let mut fields = row.iter().map(str::to_string).collect::<Vec<_>>();
    fields[index] = value.into();
    *row = csv::StringRecord::from(fields);
}

#[test]
fn receipt_backed_odc1_rows_remain_separate_and_ordered() {
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let matches = dataset.matching("odc1", "HGNC:8109");
    assert_eq!(matches.len(), 3);
    assert_eq!(matches[0].classification.code, "strong");
    assert_eq!(matches[0].mode_of_inheritance.id, "HP:0000006");
    assert_eq!(
        matches
            .iter()
            .map(|row| row.submitter.label.as_str())
            .collect::<Vec<_>>(),
        [
            "G2P",
            "PanelApp Australia",
            "Labcorp Genetics (formerly Invitae)"
        ]
    );
    assert_eq!(matches[2].id, "SGC-113621.1");
    assert_eq!(
        matches[2]
            .publications
            .iter()
            .map(|publication| publication.pmid.as_str())
            .collect::<Vec<_>>(),
        ["30239107", "30475435"]
    );
}

#[test]
fn exact_header_and_all_classification_pairs_are_closed() {
    let (header, base) = one_row();
    assert_eq!(header.iter().collect::<Vec<_>>(), HEADER);
    for (id, label, code) in [
        ("GENCC:100001", "Definitive", "definitive"),
        ("GENCC:100002", "Strong", "strong"),
        ("GENCC:100003", "Moderate", "moderate"),
        ("GENCC:100004", "Limited", "limited"),
        ("GENCC:100005", "Disputed Evidence", "disputed_evidence"),
        ("GENCC:100006", "Refuted Evidence", "refuted_evidence"),
        ("GENCC:100007", "Animal Model Only", "animal_model_only"),
        (
            "GENCC:100008",
            "No Known Disease Relationship",
            "no_known_disease_relationship",
        ),
        ("GENCC:100009", "Supportive", "supportive"),
    ] {
        let mut row = base.clone();
        set(&mut row, 8, id);
        set(&mut row, 9, label);
        let csv = csv_bytes(&header, &[row]);
        let parsed = GenCcDataset::parse(&csv, &AtomicBool::new(false)).unwrap();
        assert_eq!(parsed.assertions()[0].classification.code, code);
    }
}

#[test]
fn malformed_classification_version_and_pmid_fail_closed() {
    let (header, base) = one_row();
    for (column, value) in [
        (1, "0"),
        (1, "4294967296"),
        (8, "GENCC:999999"),
        (9, "Moderate"),
        (27, "0"),
        (27, "18446744073709551616"),
        (27, "42,,43"),
    ] {
        let mut row = base.clone();
        set(&mut row, column, value);
        let csv = csv_bytes(&header, &[row]);
        assert!(GenCcDataset::parse(&csv, &AtomicBool::new(false)).is_err());
    }
}

#[test]
fn duplicate_comparison_uses_normalized_retained_tuple() {
    let (header, row) = one_row();
    let identical = csv_bytes(&header, &[row.clone(), row.clone()]);
    assert_eq!(
        GenCcDataset::parse(&identical, &AtomicBool::new(false))
            .unwrap()
            .assertions()
            .len(),
        1
    );
    let mut changed = row.clone();
    set(&mut changed, 13, "Different submitter");
    let conflict = csv_bytes(&header, &[row.clone(), changed]);
    assert!(GenCcDataset::parse(&conflict, &AtomicBool::new(false)).is_err());

    let mut excluded_change = row.clone();
    set(&mut excluded_change, 26, "different excluded note");
    let equivalent = csv_bytes(&header, &[row, excluded_change]);
    assert_eq!(
        GenCcDataset::parse(&equivalent, &AtomicBool::new(false))
            .unwrap()
            .assertions()
            .len(),
        1
    );
}

#[test]
fn numeric_date_url_and_pmid_boundaries_are_exact() {
    let (header, base) = one_row();
    let mut row = base.clone();
    set(&mut row, 1, "4294967295");
    set(&mut row, 24, "2026-01-01T00:30:00+01:00");
    set(&mut row, 25, " HTTP://Example.COM:80/a%2fb?b=2&a=1 ");
    set(&mut row, 27, "PMID:00042, 42, 18446744073709551615");
    set(&mut row, 30, "2026-09-06 23:59:59");
    let parsed = GenCcDataset::parse(&csv_bytes(&header, &[row]), &AtomicBool::new(false)).unwrap();
    let assertion = &parsed.assertions()[0];
    assert_eq!(assertion.version, u32::MAX);
    assert_eq!(assertion.evaluated_date.as_deref(), Some("2025-12-31"));
    assert_eq!(assertion.submitted_date.as_deref(), Some("2026-09-06"));
    assert_eq!(
        assertion.public_report_url.as_deref(),
        Some("HTTP://Example.COM:80/a%2fb?b=2&a=1")
    );
    assert_eq!(
        assertion
            .publications
            .iter()
            .map(|publication| publication.pmid.as_str())
            .collect::<Vec<_>>(),
        ["42", "18446744073709551615"]
    );

    for bad_date in [
        "2026-02-29",
        "2026-01-01T00:00:00",
        "2026-01-01 00:00:00Z",
        "2026-01-01 24:00:00",
    ] {
        let mut row = base.clone();
        set(&mut row, 24, bad_date);
        assert!(GenCcDataset::parse(&csv_bytes(&header, &[row]), &AtomicBool::new(false)).is_err());
    }
}

#[test]
fn raw_field_label_and_link_bounds_apply_before_optional_nulling() {
    let (header, base) = one_row();
    for (column, value, accepted) in [
        (26, "a".repeat(16_384), true),
        (26, "é".repeat(8_192), true),
        (26, format!("{}é", "a".repeat(16_383)), false),
        (3, "x".repeat(1_024), true),
        (3, "x".repeat(1_025), false),
        (25, format!("http://x/{}", "a".repeat(2_039)), true),
        (25, format!("http://x/{}", "a".repeat(2_040)), true),
        (25, format!("http://x/{}\u{0007}", "a".repeat(2_040)), false),
    ] {
        let mut row = base.clone();
        set(&mut row, column, value);
        assert_eq!(
            GenCcDataset::parse(&csv_bytes(&header, &[row.clone()]), &AtomicBool::new(false))
                .is_ok(),
            accepted,
            "column {column} boundary"
        );
        if column == 25 && accepted && row.get(column).unwrap().len() == 2_049 {
            let parsed =
                GenCcDataset::parse(&csv_bytes(&header, &[row]), &AtomicBool::new(false)).unwrap();
            assert_eq!(parsed.assertions()[0].public_report_url, None);
        }
    }
}

#[test]
fn current_version_selection_and_cancellation_fail_closed() {
    let (header, mut old) = one_row();
    set(&mut old, 1, "1");
    let mut current = old.clone();
    set(&mut current, 1, "2");
    set(&mut current, 24, "2020-01-01");
    let parsed = GenCcDataset::parse(
        &csv_bytes(&header, &[old, current]),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(parsed.assertions().len(), 1);
    assert_eq!(parsed.assertions()[0].version, 2);

    let cancelled = AtomicBool::new(false);
    cancelled.store(true, Ordering::Relaxed);
    assert!(GenCcDataset::parse(fixture(), &cancelled).is_err());
}

#[test]
fn endpoint_redirect_and_validator_policy_is_closed() {
    let endpoint = reqwest::Url::parse(ENDPOINT).unwrap();
    assert!(valid_endpoint(&endpoint, false));
    assert!(same_endpoint(&endpoint, &endpoint));
    for rejected in [
        "http://thegencc.org/download/action/submissions-export-csv?format=new",
        "https://user@thegencc.org/download/action/submissions-export-csv?format=new",
        "https://thegencc.org/download/action/submissions-export-csv?format=legacy",
        "https://thegencc.org/download/action/submissions-export-csv?format=new#fragment",
        "https://example.org/download/action/submissions-export-csv?format=new",
    ] {
        let rejected = reqwest::Url::parse(rejected).unwrap();
        assert!(!valid_endpoint(&rejected, false));
        assert!(!same_endpoint(&rejected, &endpoint));
    }
    for accepted in ["\"opaque\"", "W/\"weak-tag\"", "\"opaque\\tag\""] {
        assert!(valid_etag(accepted));
    }
    for rejected in ["opaque", "w/\"bad-case\"", "\"bad space\""] {
        assert!(!valid_etag(rejected));
    }
    assert!(valid_http_date("Sun, 06 Sep 2026 06:00:29 GMT"));
    assert!(!valid_http_date("2026-09-06T06:00:29Z"));
}

#[test]
fn freshness_and_retry_boundaries_include_rollback_rules() {
    use chrono::{TimeZone, Utc};

    let checked = "2026-01-01T00:00:00Z";
    let base = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    assert!(is_fresh(
        Some(checked),
        base + chrono::Duration::seconds(604_799)
    ));
    assert!(!is_fresh(
        Some(checked),
        base + chrono::Duration::seconds(604_800)
    ));
    assert!(!is_fresh(
        Some(checked),
        base - chrono::Duration::seconds(1)
    ));

    assert!(inside_retry_window(
        Some(checked),
        base + chrono::Duration::seconds(86_399)
    ));
    assert!(!inside_retry_window(
        Some(checked),
        base + chrono::Duration::seconds(86_400)
    ));
    assert!(inside_retry_window(
        Some(checked),
        base - chrono::Duration::seconds(1)
    ));
}

struct EnvRestore {
    name: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvRestore {
    fn set(name: &'static str, value: &std::ffi::OsStr) -> Self {
        let previous = std::env::var_os(name);
        // SAFETY: GenCC tests serialize mutations of their unique environment keys.
        unsafe { std::env::set_var(name, value) };
        Self { name, previous }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        // SAFETY: GenCC tests serialize mutations of their unique environment keys.
        unsafe {
            match self.previous.take() {
                Some(value) => std::env::set_var(self.name, value),
                None => std::env::remove_var(self.name),
            }
        }
    }
}

#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(gencc_env)]
async fn initial_fresh_and_conditional_304_lifecycle_uses_one_get() {
    use axum::Router;
    use axum::body::Body;
    use axum::extract::State as AxumState;
    use axum::http::{HeaderMap, Method, Response, StatusCode};

    #[derive(Clone)]
    struct FixtureState {
        requests: Arc<Mutex<Vec<(Method, HeaderMap)>>>,
    }

    async fn handler(
        AxumState(state): AxumState<FixtureState>,
        method: Method,
        headers: HeaderMap,
    ) -> Response<Body> {
        state
            .requests
            .lock()
            .unwrap()
            .push((method.clone(), headers));
        if method == Method::GET && state.requests.lock().unwrap().len() == 1 {
            return Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/csv; charset=UTF-8")
                .header("content-encoding", "identity")
                .header("etag", "\"fixture-v1\"")
                .header("last-modified", "Sun, 06 Sep 2026 06:00:29 GMT")
                .body(Body::from(fixture()))
                .unwrap();
        }
        Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header("content-length", "0")
            .header("etag", "\"fixture-v1\"")
            .header("last-modified", "Sun, 06 Sep 2026 06:00:29 GMT")
            .body(Body::empty())
            .unwrap()
    }

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("gencc");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = format!("http://{address}/download/action/submissions-export-csv?format=new");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let server = tokio::spawn(
        axum::serve(
            listener,
            Router::new().fallback(handler).with_state(FixtureState {
                requests: Arc::clone(&requests),
            }),
        )
        .into_future(),
    );
    let _root = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
    let _base = EnvRestore::set("BIOMCP_GENCC_BASE", std::ffi::OsStr::new(&endpoint));

    let client = super::GenCcClient::new().unwrap();
    let initial = client.acquire(Duration::from_secs(2)).await;
    assert_eq!(
        initial.status.operation,
        super::GenCcOperation::InitialDownload
    );
    assert_eq!(initial.status.freshness, super::GenCcFreshness::Fresh);
    let fresh = client.acquire(Duration::from_secs(2)).await;
    assert_eq!(fresh.status.operation, super::GenCcOperation::LocalQuery);
    assert_eq!(requests.lock().unwrap().len(), 1);
    assert!(!client.sync().await.unwrap());
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].0, Method::GET);
    assert_eq!(requests[1].0, Method::GET);
    assert_eq!(requests[1].1["if-none-match"], "\"fixture-v1\"");
    assert_eq!(
        requests[1].1["if-modified-since"],
        "Sun, 06 Sep 2026 06:00:29 GMT"
    );
    drop(requests);
    assert!(std::fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".raw-")
    }));
    let state: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("state.json")).unwrap()).unwrap();
    let generation = state["active_generation"].as_str().unwrap();
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            root.join("generations")
                .join(generation)
                .join("manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        manifest["body_sha256"],
        format!("{:x}", Sha256::digest(fixture()))
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut directories = vec![root.clone()];
        while let Some(directory) = directories.pop() {
            assert_eq!(
                std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
            for entry in std::fs::read_dir(directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    directories.push(path);
                } else {
                    assert_eq!(
                        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                        0o600
                    );
                }
            }
        }
    }
    server.abort();
}

#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(gencc_env)]
async fn failed_initial_attempt_is_durably_suppressed_without_body_leakage() {
    use axum::Router;
    use axum::body::Body;
    use axum::extract::State as AxumState;
    use axum::http::{Response, StatusCode};

    async fn handler(
        AxumState(count): AxumState<Arc<std::sync::atomic::AtomicUsize>>,
    ) -> Response<Body> {
        count.fetch_add(1, Ordering::Relaxed);
        Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .header("content-type", "text/plain")
            .body(Body::from(
                "SENSITIVE-UPSTREAM-DETAIL /private/provider/path",
            ))
            .unwrap()
    }

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("gencc");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let endpoint = format!("http://{address}/download/action/submissions-export-csv?format=new");
    let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server_count = Arc::clone(&count);
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(handler).with_state(server_count),
        )
        .await
        .unwrap();
    });
    let _root = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
    let _base = EnvRestore::set("BIOMCP_GENCC_BASE", std::ffi::OsStr::new(&endpoint));

    let client = super::GenCcClient::new().unwrap();
    let failed = client.acquire(Duration::from_secs(2)).await;
    assert_eq!(
        failed.status.operation,
        super::GenCcOperation::InitialDownload
    );
    assert_eq!(failed.status.freshness, super::GenCcFreshness::Unavailable);
    let attempted = failed.status.attempted_at.clone();
    assert!(attempted.is_some());
    let suppressed = client.acquire(Duration::from_secs(2)).await;
    assert_eq!(
        suppressed.status.operation,
        super::GenCcOperation::RetrySuppressed
    );
    assert_eq!(suppressed.status.attempted_at, attempted);
    assert_eq!(count.load(Ordering::Relaxed), 1);
    let durable = std::fs::read_to_string(root.join("state.json")).unwrap();
    assert!(!durable.contains("SENSITIVE-UPSTREAM-DETAIL"));
    assert!(!durable.contains("/private/provider/path"));
    assert!(std::fs::read_dir(&root).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".raw-")
    }));
    server.abort();
}

#[test]
#[serial_test::serial(gencc_env)]
fn generation_cleanup_retains_an_actively_leased_old_snapshot() {
    use super::store::{PublishMetadata, Store};

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("gencc");
    let _root = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
    let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
    let store = Store::open().unwrap();
    let body_sha256 = format!("{:x}", Sha256::digest(fixture()));
    let publish = |now: &str| {
        store
            .publish(
                &dataset,
                PublishMetadata {
                    now,
                    etag: "\"fixture\"",
                    last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                    endpoint: ENDPOINT,
                    body_sha256: &body_sha256,
                    row_count: dataset.row_count(),
                },
            )
            .unwrap()
    };
    let g1 = publish("2026-01-01T00:00:00Z");
    let g1_name = g1.state.active_generation.as_deref().unwrap();
    let suffix = g1_name.rsplit_once('-').unwrap().1;
    assert_eq!(suffix.len(), 32);
    assert!(suffix.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let same_generation = store.load().unwrap().unwrap();
    assert!(Arc::ptr_eq(&g1.lease, &same_generation.lease));
    drop(same_generation);
    drop(publish("2026-01-02T00:00:00Z"));
    drop(publish("2026-01-03T00:00:00Z"));
    assert_eq!(
        std::fs::read_dir(root.join("generations")).unwrap().count(),
        3
    );
    assert_eq!(g1.dataset.assertions().len(), 3);
    drop(g1);
    let invalid = root.join("generations/invalid-finalized");
    std::fs::create_dir(&invalid).unwrap();
    std::fs::write(invalid.join("lease.lock"), b"").unwrap();
    std::fs::write(invalid.join("manifest.json"), b"invalid").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&invalid, std::fs::Permissions::from_mode(0o700)).unwrap();
        for name in ["lease.lock", "manifest.json"] {
            std::fs::set_permissions(invalid.join(name), std::fs::Permissions::from_mode(0o600))
                .unwrap();
        }
    }
    drop(publish("2026-01-04T00:00:00Z"));
    assert!(!invalid.exists());
    assert_eq!(
        std::fs::read_dir(root.join("generations")).unwrap().count(),
        2
    );
}

#[tokio::test]
async fn parser_work_obeys_an_expired_refresh_deadline() {
    let result = super::parse_with_deadline(
        fixture().to_vec(),
        tokio::time::Instant::now() - Duration::from_millis(1),
    )
    .await;
    assert!(result.is_none());
}

#[test]
#[serial_test::serial(gencc_env)]
fn refresh_leader_removes_only_owned_abandoned_temporaries() {
    use super::store::Store;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("gencc");
    let _root = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
    let store = Store::open().unwrap();
    let raw = root.join(".raw-abandoned.tmp");
    let state = root.join(".state-abandoned.tmp");
    let generation = root.join("generations/.tmp-abandoned");
    std::fs::write(&raw, b"raw").unwrap();
    std::fs::write(&state, b"state").unwrap();
    std::fs::create_dir(&generation).unwrap();
    std::fs::write(root.join("unrelated.tmp"), b"keep").unwrap();

    store.cleanup_abandoned();

    assert!(!raw.exists());
    assert!(!state.exists());
    assert!(!generation.exists());
    assert!(root.join("unrelated.tmp").exists());
}

#[test]
#[serial_test::serial(gencc_env)]
fn bootstrap_and_store_lock_waits_are_bounded_by_the_call_deadline() {
    use fs2::FileExt;

    use super::store::{Store, StoreError};

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("gencc");
    let _root = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
    drop(Store::open().unwrap());

    let anchor_path = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(".biomcp-gencc-root-"))
        })
        .unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(anchor_path)
        .unwrap();
    held.lock_exclusive().unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(60));
        drop(held);
    });
    let started = std::time::Instant::now();
    drop(Store::open_until(started + Duration::from_secs(1)).unwrap());
    assert!(started.elapsed() >= Duration::from_millis(50));
    release.join().unwrap();

    let held = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".store.lock"))
        .unwrap();
    held.lock_exclusive().unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(60));
        drop(held);
    });
    let started = std::time::Instant::now();
    drop(Store::open_until(started + Duration::from_secs(1)).unwrap());
    assert!(started.elapsed() >= Duration::from_millis(50));
    release.join().unwrap();

    let held = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".store.lock"))
        .unwrap();
    held.lock_exclusive().unwrap();
    let result = Store::open_until(std::time::Instant::now() + Duration::from_millis(30));
    assert!(matches!(result, Err(StoreError::Deadline)));
    drop(held);
}

#[test]
#[ignore = "subprocess helper"]
fn gencc_subprocess_client() {
    if let Some(entered) = std::env::var_os("BIOMCP_GENCC_CHILD_HOLD_LEASE") {
        use super::store::Store;
        let snapshot = Store::open().unwrap().load().unwrap().unwrap();
        std::fs::write(entered, b"entered").unwrap();
        let release = std::env::var_os("BIOMCP_GENCC_CHILD_RELEASE").unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !std::path::Path::new(&release).exists() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(snapshot.dataset.assertions().len(), 3);
        return;
    }
    if std::env::var_os("BIOMCP_GENCC_CHILD_CRASH_PUBLISH").is_some() {
        #[cfg(unix)]
        unsafe {
            libc::setrlimit(
                libc::RLIMIT_CORE,
                &libc::rlimit {
                    rlim_cur: 0,
                    rlim_max: 0,
                },
            );
        }
        use super::store::{PublishMetadata, Store};
        let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
        Store::open()
            .unwrap()
            .publish(
                &dataset,
                PublishMetadata {
                    now: "2026-02-01T00:00:00Z",
                    etag: "\"crash-new\"",
                    last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                    endpoint: ENDPOINT,
                    body_sha256: &format!("{:x}", Sha256::digest(fixture())),
                    row_count: dataset.row_count(),
                },
            )
            .unwrap();
        panic!("configured crash point was not reached");
    }
    let Ok(expected) = std::env::var("BIOMCP_GENCC_CHILD_EXPECT") else {
        return;
    };
    let timeout = std::env::var("BIOMCP_GENCC_CHILD_TIMEOUT_MS")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let data = runtime.block_on(async {
        super::GenCcClient::new()
            .unwrap()
            .acquire(Duration::from_millis(timeout))
            .await
    });
    let actual = format!(
        "{:?}/{:?}/{:?}",
        data.status.operation, data.status.freshness, data.status.result
    );
    assert_eq!(actual, expected);
}

#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(gencc_env)]
async fn cross_process_first_use_elects_one_leader_and_settles_followers() {
    use axum::Router;
    use axum::body::Body;
    use axum::extract::State as AxumState;
    use axum::http::{Response, StatusCode};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};

    #[derive(Clone)]
    struct Barrier {
        entered: PathBuf,
        release: PathBuf,
        requests: Arc<std::sync::atomic::AtomicUsize>,
        fail: bool,
        conditional: bool,
    }

    async fn handler(AxumState(state): AxumState<Barrier>) -> Response<Body> {
        state.requests.fetch_add(1, Ordering::Relaxed);
        std::fs::write(&state.entered, b"entered").unwrap();
        while !state.release.exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        if state.fail {
            return Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from("private provider failure"))
                .unwrap();
        }
        if state.conditional {
            return Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header("content-length", "0")
                .header("etag", "\"fixture-cross-process\"")
                .header("last-modified", "Sun, 06 Sep 2026 06:00:29 GMT")
                .body(Body::empty())
                .unwrap();
        }
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/csv")
            .header("content-length", fixture().len())
            .header("etag", "\"fixture-cross-process\"")
            .header("last-modified", "Sun, 06 Sep 2026 06:00:29 GMT")
            .body(Body::from(fixture()))
            .unwrap()
    }

    fn child(root: &Path, endpoint: &str, timeout: u64, expected: &str) -> Child {
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "sources::gencc::tests::gencc_subprocess_client",
            ])
            .env("BIOMCP_GENCC_DIR", root)
            .env("BIOMCP_GENCC_BASE", endpoint)
            .env("BIOMCP_GENCC_CHILD_TIMEOUT_MS", timeout.to_string())
            .env("BIOMCP_GENCC_CHILD_EXPECT", expected)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }

    async fn wait(child: &mut Child) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                return;
            }
            assert!(tokio::time::Instant::now() < deadline, "GenCC child hung");
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    async fn wait_for(path: &Path, child: &mut Child) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while !path.exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "leader exited before HTTP"
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "leader never entered HTTP"
            );
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    fn seed(root: &Path) {
        use super::store::{PublishMetadata, Store};
        let restore = EnvRestore::set("BIOMCP_GENCC_DIR", root.as_os_str());
        let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
        Store::open()
            .unwrap()
            .publish(
                &dataset,
                PublishMetadata {
                    now: "2026-01-01T00:00:00Z",
                    etag: "\"fixture-cross-process\"",
                    last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                    endpoint: ENDPOINT,
                    body_sha256: &format!("{:x}", Sha256::digest(fixture())),
                    row_count: dataset.row_count(),
                },
            )
            .unwrap();
        drop(restore);
    }

    fn status(data: &super::GenCcData) -> String {
        format!(
            "{:?}/{:?}/{:?}",
            data.status.operation, data.status.freshness, data.status.result
        )
    }

    for (conditional, fail) in [(false, false), (false, true), (true, false), (true, true)] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("gencc");
        if conditional {
            seed(&root);
        }
        let entered = temp.path().join("entered");
        let release = temp.path().join("release");
        let requests = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!(
            "http://{}/download/action/submissions-export-csv?format=new",
            listener.local_addr().unwrap()
        );
        let server = tokio::spawn(
            axum::serve(
                listener,
                Router::new().fallback(handler).with_state(Barrier {
                    entered: entered.clone(),
                    release: release.clone(),
                    requests: Arc::clone(&requests),
                    fail,
                    conditional,
                }),
            )
            .into_future(),
        );
        let mut leader = child(
            &root,
            &endpoint,
            3_000,
            if conditional && fail {
                "ConditionalRefresh/Stale/Data"
            } else if conditional {
                "ConditionalRefresh/Fresh/Data"
            } else if fail {
                "InitialDownload/Unavailable/Unknown"
            } else {
                "InitialDownload/Fresh/Data"
            },
        );
        wait_for(&entered, &mut leader).await;
        let mut follower = child(
            &root,
            &endpoint,
            80,
            if conditional {
                "RefreshDeferred/Stale/Data"
            } else {
                "RefreshDeferred/Unavailable/Unknown"
            },
        );
        wait(&mut follower).await;
        assert_eq!(requests.load(Ordering::Relaxed), 1);
        std::fs::write(&release, b"release").unwrap();
        wait(&mut leader).await;
        let mut settled = child(
            &root,
            &endpoint,
            1_000,
            if conditional && fail {
                "RetrySuppressed/Stale/Data"
            } else if conditional {
                "LocalQuery/Fresh/Data"
            } else if fail {
                "RetrySuppressed/Unavailable/Unknown"
            } else {
                "LocalQuery/Fresh/Data"
            },
        );
        wait(&mut settled).await;
        assert_eq!(requests.load(Ordering::Relaxed), 1);

        std::fs::remove_file(&entered).unwrap();
        std::fs::remove_file(&release).unwrap();
        let local_root = temp.path().join("local-gencc");
        if conditional {
            seed(&local_root);
        }
        let _root = EnvRestore::set("BIOMCP_GENCC_DIR", local_root.as_os_str());
        let _base = EnvRestore::set("BIOMCP_GENCC_BASE", std::ffi::OsStr::new(&endpoint));
        let expected_leader = if conditional && fail {
            "ConditionalRefresh/Stale/Data"
        } else if conditional {
            "ConditionalRefresh/Fresh/Data"
        } else if fail {
            "InitialDownload/Unavailable/Unknown"
        } else {
            "InitialDownload/Fresh/Data"
        };
        let expected_follower = if conditional {
            "RefreshDeferred/Stale/Data"
        } else {
            "RefreshDeferred/Unavailable/Unknown"
        };
        let leader = tokio::spawn(async move {
            super::GenCcClient::new()
                .unwrap()
                .acquire(Duration::from_secs(3))
                .await
        });
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while !entered.exists() {
            assert!(!leader.is_finished() && tokio::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert_eq!(
            status(
                &super::GenCcClient::new()
                    .unwrap()
                    .acquire(Duration::from_millis(80))
                    .await
            ),
            expected_follower
        );
        std::fs::write(&release, b"release").unwrap();
        assert_eq!(status(&leader.await.unwrap()), expected_leader);
        assert_eq!(requests.load(Ordering::Relaxed), 2);
        server.abort();
    }
}
