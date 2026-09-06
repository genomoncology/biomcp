//! GenCC dataset transport and durable-store facade.

use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::header::{
    CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH,
    LAST_MODIFIED, TRANSFER_ENCODING,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub(crate) mod model;
pub(crate) mod store;

pub(crate) use model::GenCcAssertion;
use model::GenCcDataset;
use store::{Attempt, PublishMetadata, Snapshot, State, Store};

const ENDPOINT: &str = "https://thegencc.org/download/action/submissions-export-csv?format=new";
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
}

pub(crate) struct GenCcClient {
    client: reqwest::Client,
}

impl GenCcClient {
    pub(crate) fn new() -> Result<Self, ()> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::limited(3))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| ())?;
        Ok(Self { client })
    }

    pub(crate) async fn acquire(&self) -> GenCcData {
        let store = match Store::open() {
            Ok(store) => store,
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
        let _guard = mutex.lock().await;
        if store.lock_refresh().is_err() {
            return snapshot.map_or_else(
                || unavailable(GenCcOperation::RefreshDeferred, state),
                progress_snapshot,
            );
        }
        let result = self.refresh(&store).await;
        store.unlock_refresh();
        result
    }

    pub(crate) async fn sync(&self) -> Result<bool, crate::error::BioMcpError> {
        let store = Store::open().map_err(|_| sync_error())?;
        let before = store.load_state().unwrap_or_default().active_generation;
        let mutex = REFRESH_MUTEX.get_or_init(|| Mutex::new(()));
        let _guard = mutex.lock().await;
        store.lock_refresh().map_err(|_| sync_error())?;
        let result = self.refresh(&store).await;
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

    async fn refresh(&self, store: &Store) -> GenCcData {
        let state = store.load_state().unwrap_or_default();
        let snapshot = store.load().ok().flatten();
        let operation = if snapshot.is_some() {
            GenCcOperation::ConditionalRefresh
        } else {
            GenCcOperation::InitialDownload
        };
        let endpoint = endpoint();
        let mut request = self.client.get(&endpoint);
        if let Some(snapshot) = &snapshot {
            request = request
                .header(IF_NONE_MATCH, &snapshot.manifest.etag)
                .header(IF_MODIFIED_SINCE, &snapshot.manifest.last_modified);
        }
        let now = timestamp(Utc::now());
        let response = match request.send().await {
            Ok(response) => response,
            Err(_) => return failed_refresh(store, snapshot, state, operation, &now),
        };

        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            let valid = snapshot.as_ref().is_some_and(|snapshot| {
                valid_304(
                    &response,
                    &snapshot.manifest.etag,
                    &snapshot.manifest.last_modified,
                )
            });
            if valid {
                let mut snapshot = snapshot.expect("validated 304 requires generation");
                match store.record_304(snapshot.state.clone(), &now) {
                    Ok(state) => {
                        snapshot.state = state;
                        return from_snapshot(snapshot, operation, true, None);
                    }
                    Err(_) => return failed_refresh(store, Some(snapshot), state, operation, &now),
                }
            }
            return failed_refresh(store, snapshot, state, operation, &now);
        }
        if response.status() != reqwest::StatusCode::OK
            || !csv_content_type(&response)
            || !identity_encoding(&response)
            || response
                .content_length()
                .is_some_and(|length| length > MAX_BODY_BYTES as u64)
        {
            return failed_refresh(store, snapshot, state, operation, &now);
        }
        let etag = required_header(&response, ETAG);
        let last_modified = required_header(&response, LAST_MODIFIED);
        let (Some(etag), Some(last_modified)) = (etag, last_modified) else {
            return failed_refresh(store, snapshot, state, operation, &now);
        };
        let body =
            match crate::sources::read_limited_body_with_limit(response, "GenCC", MAX_BODY_BYTES)
                .await
            {
                Ok(body) => body,
                Err(_) => return failed_refresh(store, snapshot, state, operation, &now),
            };
        let rows = csv_data_rows(&body);
        let cancelled = AtomicBool::new(false);
        let dataset = match GenCcDataset::parse(&body, &cancelled) {
            Ok(dataset) => dataset,
            Err(_) => return failed_refresh(store, snapshot, state, operation, &now),
        };
        match store.publish(
            &dataset,
            &body,
            PublishMetadata {
                now: &now,
                etag: &etag,
                last_modified: &last_modified,
                endpoint: ENDPOINT,
                row_count: rows,
            },
        ) {
            Ok(snapshot) => from_snapshot(snapshot, operation, true, None),
            Err(_) => failed_refresh(store, snapshot, state, operation, &now),
        }
    }
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

fn required_header(
    response: &reqwest::Response,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    response
        .headers()
        .get(name)?
        .to_str()
        .ok()
        .map(str::to_string)
}

fn csv_content_type(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("text/csv"))
}

fn identity_encoding(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get(CONTENT_ENCODING)
        .is_none_or(|value| value.as_bytes().eq_ignore_ascii_case(b"identity"))
}

fn valid_304(response: &reqwest::Response, etag: &str, modified: &str) -> bool {
    response.headers().get(TRANSFER_ENCODING).is_none()
        && identity_encoding(response)
        && response
            .headers()
            .get(CONTENT_LENGTH)
            .is_none_or(|value| value.as_bytes() == b"0")
        && response
            .headers()
            .get(ETAG)
            .is_none_or(|value| value.as_bytes() == etag.as_bytes())
        && response
            .headers()
            .get(LAST_MODIFIED)
            .is_none_or(|value| value.as_bytes() == modified.as_bytes())
}

fn csv_data_rows(body: &[u8]) -> usize {
    body.iter()
        .filter(|byte| **byte == b'\n')
        .count()
        .saturating_sub(1)
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
    }
}

fn failed_refresh(
    store: &Store,
    snapshot: Option<Snapshot>,
    state: State,
    operation: GenCcOperation,
    now: &str,
) -> GenCcData {
    let state = store.record_failure(state, now).unwrap_or_default();
    match snapshot {
        None => unavailable(operation, state),
        Some(mut snapshot) => {
            snapshot.state = state;
            stale_snapshot(snapshot, operation)
        }
    }
}

#[cfg(test)]
mod tests;
