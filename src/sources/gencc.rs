//! GenCC dataset transport and durable-store facade.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, ETAG,
    IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, TRANSFER_ENCODING,
};
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

pub(crate) mod model;
#[rustfmt::skip]
pub(crate) mod store;

pub(crate) use model::GenCcAssertion;
use model::GenCcDataset;
use store::{Attempt, PublishMetadata, Snapshot, State, Store, StoreError};

pub(crate) const ENDPOINT: &str =
    "https://thegencc.org/download/action/submissions-export-csv?format=new";
const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;
const FRESH_FOR: i64 = 604_800;
const RETRY_AFTER: i64 = 86_400;

static REFRESH_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenCcFreshness {
    Fresh,
    Stale,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenCcResult {
    Data,
    Empty,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenCcOperation {
    LocalQuery,
    InitialDownload,
    ConditionalRefresh,
    RetrySuppressed,
    RefreshDeferred,
    IdentityMatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenCcStatus {
    pub freshness: GenCcFreshness,
    pub result: GenCcResult,
    pub operation: GenCcOperation,
    pub checked_at: Option<String>,
    pub retrieved_at: Option<String>,
    pub attempted_at: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub upstream_version: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenCcData {
    pub dataset: Option<GenCcDataset>,
    pub status: GenCcStatus,
    pub(crate) lease: Option<Arc<std::fs::File>>,
}

pub(crate) struct GenCcClient {
    client: ClientWithMiddleware,
}

impl GenCcClient {
    pub(crate) fn new() -> Result<Self, ()> {
        let endpoint = endpoint();
        let expected = reqwest::Url::parse(&endpoint).map_err(|_| ())?;
        if !valid_endpoint(&expected, cfg!(debug_assertions)) {
            return Err(());
        }
        let redirect_endpoint = expected.clone();
        let client = crate::sources::ordinary_url_policy::ordinary_middleware_client_for_base(
            &endpoint,
            "BIOMCP_GENCC_BASE",
            move |builder| {
                builder.connect_timeout(Duration::from_secs(10)).redirect(
                    reqwest::redirect::Policy::custom(move |attempt| {
                        if attempt.previous().len() >= 3
                            || !same_endpoint(attempt.url(), &redirect_endpoint)
                        {
                            attempt.error("GenCC redirect target rejected")
                        } else {
                            attempt.follow()
                        }
                    }),
                )
            },
        )
        .map_err(|_| ())?;
        Ok(Self { client })
    }

    pub(crate) async fn health(&self) -> bool {
        let response = match self
            .client
            .head(endpoint())
            .header(ACCEPT, "text/csv")
            .header(ACCEPT_ENCODING, "identity")
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return false,
        };
        response.status() == reqwest::StatusCode::OK
            && reqwest::Url::parse(&endpoint())
                .is_ok_and(|expected| same_endpoint(response.url(), &expected))
            && csv_content_type(&response)
            && identity_encoding(&response)
            && required_header(&response, ETAG).is_some_and(|value| valid_etag(&value))
            && required_header(&response, LAST_MODIFIED)
                .is_some_and(|value| valid_http_date(&value))
    }

    pub(crate) async fn acquire(&self, timeout: Duration) -> GenCcData {
        let deadline = tokio::time::Instant::now() + timeout;
        let store = match Store::open_until(store_deadline(deadline)) {
            Ok(store) => store,
            Err(StoreError::Deadline) => {
                return unavailable(GenCcOperation::RefreshDeferred, State::default());
            }
            Err(_) => return unavailable(GenCcOperation::InitialDownload, State::default()),
        };
        let state = store.load_state().unwrap_or_default();
        let snapshot = store.load().ok().flatten();
        let now = Utc::now();
        if let Some(snapshot) = &snapshot
            && is_fresh(snapshot.state.checked_at.as_deref(), now)
        {
            return from_snapshot(snapshot.clone(), GenCcOperation::LocalQuery, true, None);
        }
        if state.last_attempt == Some(Attempt::Failure)
            && inside_retry_window(state.attempted_at.as_deref(), now)
        {
            return snapshot.map_or_else(
                || unavailable(GenCcOperation::RetrySuppressed, state),
                |snapshot| stale_snapshot(snapshot, GenCcOperation::RetrySuppressed),
            );
        }

        let mutex = REFRESH_MUTEX.get_or_init(|| Mutex::new(()));
        let _guard = match tokio::time::timeout_at(deadline, mutex.lock()).await {
            Ok(guard) => guard,
            Err(_) => return deferred(snapshot, state),
        };
        match lock_refresh_until(&store, deadline).await {
            Ok(true) => {}
            Ok(false) | Err(_) => return deferred(snapshot, state),
        }
        let timeout_operation = if snapshot.is_some() {
            GenCcOperation::ConditionalRefresh
        } else {
            GenCcOperation::InitialDownload
        };
        let result = self
            .refresh(&store, false, deadline, timeout_operation)
            .await;
        store.unlock_refresh();
        result
    }

    pub(crate) async fn sync(&self) -> Result<bool, crate::error::BioMcpError> {
        self.sync_with_timeout(Duration::from_secs(120)).await
    }

    async fn sync_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<bool, crate::error::BioMcpError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let store = Store::open_until(store_deadline(deadline)).map_err(|_| sync_error())?;
        let before = store.load_state().unwrap_or_default().active_generation;
        let mutex = REFRESH_MUTEX.get_or_init(|| Mutex::new(()));
        let _guard = tokio::time::timeout_at(deadline, mutex.lock())
            .await
            .map_err(|_| sync_error())?;
        if !lock_refresh_until(&store, deadline)
            .await
            .map_err(|_| sync_error())?
        {
            return Err(sync_error());
        }
        let operation = if before.is_some() {
            GenCcOperation::ConditionalRefresh
        } else {
            GenCcOperation::InitialDownload
        };
        let result = self.refresh(&store, true, deadline, operation).await;
        store.unlock_refresh();
        if result.status.freshness != GenCcFreshness::Fresh {
            return Err(sync_error());
        }
        let after = store
            .load_state()
            .map_err(|_| sync_error())?
            .active_generation;
        Ok(before != after)
    }

    async fn refresh(
        &self,
        store: &Store,
        force: bool,
        deadline: tokio::time::Instant,
        timeout_operation: GenCcOperation,
    ) -> GenCcData {
        store.cleanup_abandoned();
        let state = store.load_state().unwrap_or_default();
        let snapshot = store.load().ok().flatten();
        if !force {
            let now = Utc::now();
            if let Some(snapshot) = &snapshot
                && is_fresh(snapshot.state.checked_at.as_deref(), now)
            {
                return from_snapshot(snapshot.clone(), GenCcOperation::LocalQuery, true, None);
            }
            if state.last_attempt == Some(Attempt::Failure)
                && inside_retry_window(state.attempted_at.as_deref(), now)
            {
                return snapshot.map_or_else(
                    || unavailable(GenCcOperation::RetrySuppressed, state),
                    |snapshot| stale_snapshot(snapshot, GenCcOperation::RetrySuppressed),
                );
            }
        }
        let operation = if snapshot.is_some() {
            GenCcOperation::ConditionalRefresh
        } else {
            GenCcOperation::InitialDownload
        };
        let endpoint = endpoint();
        let mut request = self
            .client
            .get(&endpoint)
            .header(ACCEPT, "text/csv")
            .header(ACCEPT_ENCODING, "identity");
        if let Some(snapshot) = &snapshot {
            request = request
                .header(IF_NONE_MATCH, &snapshot.manifest.etag)
                .header(IF_MODIFIED_SINCE, &snapshot.manifest.last_modified);
        }
        let mut response = match tokio::time::timeout_at(deadline, request.send()).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => return failed_refresh_now(store, snapshot, state, operation),
            Err(_) => return failed_refresh_now(store, snapshot, state, timeout_operation),
        };

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            let valid = snapshot.as_ref().is_some_and(|snapshot| {
                valid_304(
                    &response,
                    &snapshot.manifest.etag,
                    &snapshot.manifest.last_modified,
                )
            });
            let body_empty = if valid {
                tokio::time::timeout_at(
                    deadline,
                    crate::sources::read_limited_body_with_limit(response, "GenCC", 0),
                )
                .await
                .is_ok_and(|result| result.is_ok_and(|body| body.is_empty()))
            } else {
                false
            };
            if body_empty {
                let mut snapshot = snapshot.expect("validated 304 requires generation");
                let now = timestamp(Utc::now());
                match store.record_304(snapshot.state.clone(), &now) {
                    Ok(state) => {
                        snapshot.state = state;
                        return from_snapshot(snapshot, operation, true, None);
                    }
                    Err(StoreError::PostRenameSync) if !force => {
                        if let Ok(Some(visible)) = store.load() {
                            return from_snapshot(visible, operation, true, None);
                        }
                        return failed_refresh(store, Some(snapshot), state, operation, &now);
                    }
                    Err(StoreError::PostRenameSync) => {
                        return unavailable(operation, store.load_state().unwrap_or(state));
                    }
                    Err(_) => return failed_refresh(store, Some(snapshot), state, operation, &now),
                }
            }
            return failed_refresh_now(store, snapshot, state, operation);
        }
        if response.status() != reqwest::StatusCode::OK
            || !csv_content_type(&response)
            || !identity_encoding(&response)
            || response.headers().contains_key(TRANSFER_ENCODING)
            || content_length(&response).is_some_and(|length| length > MAX_BODY_BYTES as u64)
        {
            return failed_refresh_now(store, snapshot, state, operation);
        }
        let etag = required_header(&response, ETAG).filter(|value| valid_etag(value));
        let last_modified =
            required_header(&response, LAST_MODIFIED).filter(|value| valid_http_date(value));
        let (Some(etag), Some(last_modified)) = (etag, last_modified) else {
            return failed_refresh_now(store, snapshot, state, operation);
        };
        let mut raw = match store.create_raw_temp() {
            Ok(raw) => raw,
            Err(_) => return failed_refresh_now(store, snapshot, state, operation),
        };
        loop {
            let chunk = match tokio::time::timeout_at(deadline, response.chunk()).await {
                Ok(Ok(chunk)) => chunk,
                Ok(Err(_)) => return failed_refresh_now(store, snapshot, state, operation),
                Err(_) => return failed_refresh_now(store, snapshot, state, timeout_operation),
            };
            let Some(chunk) = chunk else { break };
            if raw.write_chunk(&chunk, MAX_BODY_BYTES).is_err() {
                return failed_refresh_now(store, snapshot, state, operation);
            }
        }
        let (body, body_sha256) = match raw.finish() {
            Ok(result) => result,
            Err(_) => return failed_refresh_now(store, snapshot, state, operation),
        };
        let dataset = match parse_with_deadline(body, deadline).await {
            Some(dataset) => dataset,
            None => return failed_refresh_now(store, snapshot, state, timeout_operation),
        };
        if tokio::time::Instant::now() >= deadline {
            return failed_refresh_now(store, snapshot, state, timeout_operation);
        }
        let now = timestamp(Utc::now());
        match store.publish(
            &dataset,
            PublishMetadata {
                now: &now,
                etag: &etag,
                last_modified: &last_modified,
                endpoint: ENDPOINT,
                body_sha256: &body_sha256,
                row_count: dataset.row_count(),
            },
        ) {
            Ok(snapshot) => from_snapshot(snapshot, operation, true, None),
            Err(StoreError::PostRenameSync) if !force => store.load().ok().flatten().map_or_else(
                || failed_refresh(store, snapshot, state, operation, &now),
                |snapshot| from_snapshot(snapshot, operation, true, None),
            ),
            Err(StoreError::PostRenameSync) => {
                unavailable(operation, store.load_state().unwrap_or(state))
            }
            Err(_) => failed_refresh(store, snapshot, state, operation, &now),
        }
    }
}

async fn parse_with_deadline(
    body: Vec<u8>,
    deadline: tokio::time::Instant,
) -> Option<GenCcDataset> {
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_cancelled = Arc::clone(&cancelled);
    let mut worker =
        tokio::task::spawn_blocking(move || GenCcDataset::parse(&body, &worker_cancelled));
    match tokio::time::timeout_at(deadline, &mut worker).await {
        Ok(Ok(Ok(parsed))) => Some(parsed),
        Ok(Ok(Err(_)) | Err(_)) => None,
        Err(_) => {
            cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
            let _ = worker.await;
            None
        }
    }
}

async fn lock_refresh_until(store: &Store, deadline: tokio::time::Instant) -> Result<bool, ()> {
    loop {
        match store.try_lock_refresh() {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(_) => return Err(()),
        }
        if tokio::time::Instant::now() >= deadline {
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

fn store_deadline(deadline: tokio::time::Instant) -> std::time::Instant {
    std::time::Instant::now() + deadline.saturating_duration_since(tokio::time::Instant::now())
}

fn sync_error() -> crate::error::BioMcpError {
    crate::error::BioMcpError::Api {
        api: "GenCC".into(),
        message: "GenCC synchronization failed".into(),
    }
}

fn endpoint() -> String {
    if cfg!(debug_assertions)
        && let Ok(value) = std::env::var("BIOMCP_GENCC_BASE")
    {
        return value;
    }
    ENDPOINT.to_string()
}

fn same_endpoint(actual: &reqwest::Url, expected: &reqwest::Url) -> bool {
    actual.scheme() == expected.scheme()
        && actual.host_str() == expected.host_str()
        && actual.port_or_known_default() == expected.port_or_known_default()
        && actual.path() == expected.path()
        && actual.query() == expected.query()
        && actual.fragment().is_none()
        && actual.username().is_empty()
        && actual.password().is_none()
}

fn valid_endpoint(url: &reqwest::Url, fixture_allowed: bool) -> bool {
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return false;
    }
    if fixture_allowed && url.as_str() != ENDPOINT {
        return matches!(url.scheme(), "http" | "https") && url.has_host();
    }
    url.scheme() == "https"
        && url.host_str() == Some("thegencc.org")
        && url.port().is_none()
        && url.path() == "/download/action/submissions-export-csv"
        && url.query() == Some("format=new")
}

fn required_header(
    response: &reqwest::Response,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    let mut values = response.headers().get_all(name).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return None;
    }
    value.to_str().ok().map(str::to_string)
}

fn valid_etag(value: &str) -> bool {
    let value = value.strip_prefix("W/").unwrap_or(value);
    let Some(inner) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return false;
    };
    inner
        .bytes()
        .all(|byte| byte == 0x21 || (0x23..=0x7e).contains(&byte) || byte >= 0x80)
}

fn valid_http_date(value: &str) -> bool {
    value.ends_with(" GMT") && DateTime::parse_from_rfc2822(value).is_ok()
}

fn content_length(response: &reqwest::Response) -> Option<u64> {
    let mut values = response.headers().get_all(CONTENT_LENGTH).iter();
    let value = values.next()?;
    if values.next().is_some() {
        return Some(u64::MAX);
    }
    value
        .to_str()
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .or(Some(u64::MAX))
}

fn csv_content_type(response: &reqwest::Response) -> bool {
    required_header(response, CONTENT_TYPE)
        .as_deref()
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/csv"))
}

fn identity_encoding(response: &reqwest::Response) -> bool {
    let mut values = response.headers().get_all(CONTENT_ENCODING).iter();
    let first = values.next();
    values.next().is_none()
        && first.is_none_or(|value| value.as_bytes().eq_ignore_ascii_case(b"identity"))
}

fn optional_header_matches(
    response: &reqwest::Response,
    name: reqwest::header::HeaderName,
    expected: &str,
    validate: impl FnOnce(&str) -> bool,
) -> bool {
    let mut values = response.headers().get_all(name).iter();
    let first = values.next();
    if values.next().is_some() {
        return false;
    }
    first.is_none_or(|value| {
        value.as_bytes() == expected.as_bytes() && value.to_str().is_ok_and(validate)
    })
}

fn valid_304(response: &reqwest::Response, etag: &str, modified: &str) -> bool {
    response.headers().get(TRANSFER_ENCODING).is_none()
        && identity_encoding(response)
        && content_length(response).is_none_or(|value| value == 0)
        && optional_header_matches(response, ETAG, etag, valid_etag)
        && optional_header_matches(response, LAST_MODIFIED, modified, valid_http_date)
}

fn timestamp(now: DateTime<Utc>) -> String {
    now.to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn parsed_time(value: Option<&str>) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value?)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn is_fresh(value: Option<&str>, now: DateTime<Utc>) -> bool {
    parsed_time(value).is_some_and(|checked| {
        now >= checked && now.signed_duration_since(checked).num_seconds() < FRESH_FOR
    })
}

fn inside_retry_window(value: Option<&str>, now: DateTime<Utc>) -> bool {
    parsed_time(value).is_some_and(|attempted| {
        now < attempted || now.signed_duration_since(attempted).num_seconds() < RETRY_AFTER
    })
}

fn status_from(
    snapshot: &Snapshot,
    freshness: GenCcFreshness,
    operation: GenCcOperation,
    message: Option<&str>,
) -> GenCcStatus {
    GenCcStatus {
        freshness,
        result: if snapshot.dataset.assertions().is_empty() {
            GenCcResult::Empty
        } else {
            GenCcResult::Data
        },
        operation,
        checked_at: snapshot.state.checked_at.clone(),
        retrieved_at: Some(snapshot.manifest.retrieved_at.clone()),
        attempted_at: snapshot.state.attempted_at.clone(),
        etag: Some(snapshot.manifest.etag.clone()),
        last_modified: Some(snapshot.manifest.last_modified.clone()),
        upstream_version: None,
        message: message.map(str::to_string),
    }
}

fn from_snapshot(
    snapshot: Snapshot,
    operation: GenCcOperation,
    fresh: bool,
    message: Option<&str>,
) -> GenCcData {
    let status = status_from(
        &snapshot,
        if fresh {
            GenCcFreshness::Fresh
        } else {
            GenCcFreshness::Stale
        },
        operation,
        message,
    );
    GenCcData {
        dataset: Some(snapshot.dataset),
        status,
        lease: Some(snapshot.lease),
    }
}

fn stale_snapshot(snapshot: Snapshot, operation: GenCcOperation) -> GenCcData {
    from_snapshot(
        snapshot,
        operation,
        false,
        Some("GenCC refresh failed; results come from the last validated dataset."),
    )
}

fn progress_snapshot(snapshot: Snapshot) -> GenCcData {
    from_snapshot(
        snapshot,
        GenCcOperation::RefreshDeferred,
        false,
        Some("GenCC refresh is still in progress; results come from the last validated dataset."),
    )
}

fn deferred(snapshot: Option<Snapshot>, state: State) -> GenCcData {
    snapshot.map_or_else(
        || unavailable(GenCcOperation::RefreshDeferred, state),
        progress_snapshot,
    )
}

fn unavailable(operation: GenCcOperation, state: State) -> GenCcData {
    GenCcData {
        dataset: None,
        status: GenCcStatus {
            freshness: GenCcFreshness::Unavailable,
            result: GenCcResult::Unknown,
            operation,
            checked_at: state.checked_at,
            retrieved_at: None,
            attempted_at: state.attempted_at,
            etag: None,
            last_modified: None,
            upstream_version: None,
            message: Some(
                if operation == GenCcOperation::RefreshDeferred {
                    "GenCC refresh is still in progress; no GenCC absence can be concluded."
                } else {
                    "GenCC data is unavailable; no GenCC absence can be concluded."
                }
                .to_string(),
            ),
        },
        lease: None,
    }
}

fn failed_refresh(
    store: &Store,
    snapshot: Option<Snapshot>,
    state: State,
    operation: GenCcOperation,
    now: &str,
) -> GenCcData {
    let state = match store.record_failure(state.clone(), now) {
        Ok(state) => state,
        Err(StoreError::PostRenameSync) => store.load_state().unwrap_or(state),
        Err(_) => state,
    };
    match snapshot {
        None => unavailable(operation, state),
        Some(mut snapshot) => {
            snapshot.state = state;
            stale_snapshot(snapshot, operation)
        }
    }
}

fn failed_refresh_now(
    store: &Store,
    snapshot: Option<Snapshot>,
    state: State,
    operation: GenCcOperation,
) -> GenCcData {
    let now = timestamp(Utc::now());
    failed_refresh(store, snapshot, state, operation, &now)
}

#[cfg(test)]
#[tokio::test(flavor = "current_thread")]
#[serial_test::serial(gencc_env)]
async fn explicit_sync_lock_deadline_preserves_state_with_and_without_generation() {
    for seeded in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let root = temp.path().join("gencc");
        let previous = std::env::var_os("BIOMCP_GENCC_DIR");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let store = Store::open().unwrap();
        if seeded {
            let body = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/gencc/submissions-new-odc1.csv"
            ));
            let dataset = GenCcDataset::parse(body, &AtomicBool::new(false)).unwrap();
            store
                .publish(
                    &dataset,
                    PublishMetadata {
                        now: "2026-01-01T00:00:00Z",
                        etag: "\"sync-lock\"",
                        last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                        endpoint: ENDPOINT,
                        body_sha256: &format!("{:x}", Sha256::digest(body)),
                        row_count: dataset.row_count(),
                    },
                )
                .unwrap();
        }
        let before = std::fs::read(root.join("state.json")).ok();
        assert!(store.try_lock_refresh().unwrap());
        assert!(
            GenCcClient::new()
                .unwrap()
                .sync_with_timeout(Duration::from_millis(40))
                .await
                .is_err()
        );
        store.unlock_refresh();
        assert_eq!(std::fs::read(root.join("state.json")).ok(), before);
        match previous {
            Some(value) => unsafe { std::env::set_var("BIOMCP_GENCC_DIR", value) },
            None => unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") },
        }
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests;
