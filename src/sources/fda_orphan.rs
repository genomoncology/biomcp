//! Bounded client for FDA's HTML orphan-designation search service.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{StreamExt, stream};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

pub(crate) const FDA_ORPHAN_SOURCE: &str = "FDA Orphan Drug Designations and Approvals";
pub(crate) const FDA_ORPHAN_PARSE_VERSION: u8 = 1;
const BASE: &str = "https://www.accessdata.fda.gov/scripts/opdlisting/oopd/";
const BASE_ENV: &str = "BIOMCP_FDA_ORPHAN_BASE";
const MAX_BODY: usize = 2 * 1024 * 1024;
const MAX_WIRE_ROWS: usize = 500;
const MAX_RECORDS: usize = 100;
const TTL: Duration = Duration::from_secs(24 * 60 * 60);
const DEADLINE: Duration = Duration::from_secs(8);

const HEADERS: [&str; 19] = [
    "Generic Name",
    "Trade Name",
    "Date Designated",
    "Orphan Designation",
    "Orphan Designation Status",
    "Date Designation Withdrawn or Revoked",
    "FDA Orphan Approval Status",
    "Approved Labeled Indication",
    "Marketing Approval Date",
    "Exclusivity End Date",
    "Exclusivity Protected Indication * (Shown on labeling)",
    "Sponsor Company",
    "Sponsor Address 1",
    "Sponsor Address 2",
    "Sponsor City",
    "Sponsor State",
    "Sponsor Zip",
    "Sponsor Country",
    "CF Grid Key",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrphanApproval {
    NotApproved,
    Approved,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FdaOrphanRecord {
    pub record_id: String,
    pub generic_name: String,
    pub trade_name: Option<String>,
    pub designation_date: String,
    pub designation: String,
    pub designation_status: Option<String>,
    pub designation_withdrawn_or_revoked_date: Option<String>,
    pub orphan_approval: OrphanApproval,
    pub orphan_approval_status_text: Option<String>,
    pub approved_labeled_indication: Option<String>,
    pub marketing_approval_date: Option<String>,
    pub exclusivity_end_date: Option<String>,
    pub exclusivity_protected_indication: Option<String>,
    pub sponsor: Option<String>,
    pub source_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FdaOrphanOutcome {
    Data,
    Empty,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FdaOrphanDesignations {
    pub outcome: FdaOrphanOutcome,
    pub sources: Vec<String>,
    pub records: Vec<FdaOrphanRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub total_matching: Option<usize>,
    pub truncated: bool,
}

#[derive(Serialize, Deserialize)]
struct CachedQuery {
    stored_at: u64,
    records: Vec<FdaOrphanRecord>,
}

fn clean(value: &str) -> String {
    value.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

fn optional(value: String) -> Option<String> {
    let value = clean(&value.replace('\u{a0}', " "));
    (!value.is_empty()).then_some(value)
}

fn date(value: Option<String>) -> Result<Option<String>, BioMcpError> {
    value
        .map(|value| {
            chrono::NaiveDate::parse_from_str(&value, "%m/%d/%Y")
                .map(|date| date.format("%Y-%m-%d").to_string())
                .map_err(|_| source_error("invalid date"))
        })
        .transpose()
}

fn source_error(reason: &str) -> BioMcpError {
    BioMcpError::Api {
        api: FDA_ORPHAN_SOURCE.into(),
        message: reason.into(),
    }
}

pub(crate) fn ordered_form(candidate: &str) -> Vec<(String, String)> {
    [
        ("Product_name", candidate),
        ("sponsor_name", ""),
        ("Designation", ""),
        ("Designation_Start_Date", ""),
        ("Designation_End_Date", ""),
        ("Search_param", "DESDATE"),
        ("Output_Format", "Excel"),
        ("Sort_order", "GENERIC_NAME"),
        ("RecordsPerPage", "25"),
        ("newSearch", "Run Search"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect()
}

fn table_rows(html: &[u8]) -> Result<Vec<Vec<String>>, BioMcpError> {
    let text = std::str::from_utf8(html).map_err(|_| source_error("invalid UTF-8"))?;
    let doc = Html::parse_document(text);
    let table_sel = Selector::parse("table").expect("selector");
    let row_sel = Selector::parse("tr").expect("selector");
    let cell_sel = Selector::parse("th, td").expect("selector");
    let tables = doc
        .select(&table_sel)
        .map(|table| direct_rows(table, &row_sel, &cell_sel))
        .collect::<Vec<_>>();
    if tables.len() != 1 {
        return Err(source_error("unexpected table shape"));
    }
    let rows = tables.into_iter().next().expect("one table");
    if !rows
        .first()
        .is_some_and(|row| row.iter().map(String::as_str).eq(HEADERS))
    {
        return Err(source_error("unexpected headers"));
    }
    Ok(rows)
}

fn direct_rows(table: ElementRef<'_>, row_sel: &Selector, cell_sel: &Selector) -> Vec<Vec<String>> {
    table
        .select(row_sel)
        .filter(|row| {
            row.ancestors()
                .filter_map(ElementRef::wrap)
                .find(|node| node.value().name() == "table")
                .is_some_and(|owner| owner.id() == table.id())
        })
        .map(|row| {
            row.select(cell_sel)
                .filter(|cell| {
                    cell.ancestors()
                        .filter_map(ElementRef::wrap)
                        .find(|node| node.value().name() == "tr")
                        .is_some_and(|owner| owner.id() == row.id())
                })
                .map(|cell| clean(&cell.text().collect::<String>().replace('\u{a0}', " ")))
                .collect()
        })
        .collect()
}

#[cfg(test)]
fn parse_response(html: &[u8], candidates: &[String]) -> Result<Vec<FdaOrphanRecord>, BioMcpError> {
    Ok(filter_records(parse_table(html)?, candidates))
}

fn parse_table(html: &[u8]) -> Result<Vec<FdaOrphanRecord>, BioMcpError> {
    if html.len() > MAX_BODY {
        return Err(source_error("response too large"));
    }
    let rows = table_rows(html)?;
    if !rows
        .first()
        .is_some_and(|row| row.iter().map(String::as_str).eq(HEADERS))
    {
        return Err(source_error("unexpected headers"));
    }
    if rows.len().saturating_sub(1) > MAX_WIRE_ROWS {
        return Err(source_error("too many rows"));
    }
    rows.into_iter()
        .skip(1)
        .map(|row| {
            if row.len() != HEADERS.len() {
                return Err(source_error("malformed row"));
            }
            let generic = clean(&row[0]);
            let trade = optional(row[1].clone());
            parse_row(row, generic, trade)
        })
        .collect()
}

fn filter_records(records: Vec<FdaOrphanRecord>, candidates: &[String]) -> Vec<FdaOrphanRecord> {
    let candidates = candidates
        .iter()
        .map(|v| clean(v).to_ascii_lowercase())
        .collect::<HashSet<_>>();
    records
        .into_iter()
        .filter(|row| {
            candidates.contains(&row.generic_name.to_ascii_lowercase())
                || row
                    .trade_name
                    .as_ref()
                    .is_some_and(|v| candidates.contains(&v.to_ascii_lowercase()))
        })
        .collect()
}

fn parse_row(
    row: Vec<String>,
    generic_name: String,
    trade_name: Option<String>,
) -> Result<FdaOrphanRecord, BioMcpError> {
    let designation = clean(&row[3]);
    let record_id = clean(&row[18]);
    if generic_name.is_empty()
        || designation.is_empty()
        || record_id.is_empty()
        || !record_id.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(source_error("missing required row field"));
    }
    let designation_status = optional(row[4].clone());
    let approval_text = optional(row[6].clone());
    let marketing_approval_date = date(optional(row[8].clone()))?;
    let exclusivity_end_date = date(optional(row[9].clone()))?;
    let positive = marketing_approval_date.is_some()
        || designation_status.as_deref().is_some_and(|status| {
            status
                .split('/')
                .any(|part| clean(part).eq_ignore_ascii_case("approved"))
        });
    let negative = approval_text
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("Not FDA Approved for Orphan Indication"));
    if positive && negative {
        return Err(source_error("contradictory approval facts"));
    }
    let orphan_approval = if positive {
        OrphanApproval::Approved
    } else if negative {
        OrphanApproval::NotApproved
    } else {
        OrphanApproval::Unknown
    };
    Ok(FdaOrphanRecord {
        source_url: format!("{BASE}detailedIndex.cfm?cfgridkey={record_id}"),
        record_id,
        generic_name,
        trade_name,
        designation_date: date(Some(row[2].clone()))?
            .ok_or_else(|| source_error("missing designation date"))?,
        designation,
        designation_status,
        designation_withdrawn_or_revoked_date: date(optional(row[5].clone()))?,
        orphan_approval,
        orphan_approval_status_text: approval_text,
        approved_labeled_indication: optional(row[7].clone()),
        marketing_approval_date,
        exclusivity_end_date,
        exclusivity_protected_indication: optional(row[10].clone()),
        sponsor: optional(row[11].clone()),
    })
}

fn cache_key(base: &str, form: &[(String, String)]) -> String {
    let encoded = serde_json::to_string(form).expect("form serializes");
    format!("fda-orphan-v{FDA_ORPHAN_PARSE_VERSION}:{base}:{encoded}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceCacheMode {
    Normal,
    Off,
    Infinite,
}

fn cache_mode() -> SourceCacheMode {
    source_cache_mode(
        crate::sources::cache_is_bypassed(),
        std::env::var("BIOMCP_CACHE_MODE").ok().as_deref(),
    )
}

fn source_cache_mode(no_cache: bool, env: Option<&str>) -> SourceCacheMode {
    if no_cache || env.is_some_and(|v| v.trim().eq_ignore_ascii_case("off")) {
        SourceCacheMode::Off
    } else if env.is_some_and(|v| v.trim().eq_ignore_ascii_case("infinite")) {
        SourceCacheMode::Infinite
    } else {
        SourceCacheMode::Normal
    }
}

fn cache_entry_is_usable(mode: SourceCacheMode, now: u64, stored_at: u64) -> bool {
    mode == SourceCacheMode::Infinite || now.saturating_sub(stored_at) < TTL.as_secs()
}

async fn read_cache(key: &str, mode: SourceCacheMode) -> Option<Vec<FdaOrphanRecord>> {
    if mode == SourceCacheMode::Off {
        return None;
    }
    let config = crate::cache::resolve_cache_config().ok()?;
    let bytes = cacache::read(config.cache_root.join("http"), key)
        .await
        .ok()?;
    let cached: CachedQuery = serde_json::from_slice(&bytes).ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    cache_entry_is_usable(mode, now, cached.stored_at).then_some(cached.records)
}

async fn write_cache(
    key: &str,
    records: &[FdaOrphanRecord],
    mode: SourceCacheMode,
) -> Result<(), BioMcpError> {
    if mode == SourceCacheMode::Off {
        return Ok(());
    }
    let config = crate::cache::resolve_cache_config()?;
    let path = config.cache_root.join("http");
    let _guard = crate::cache::lock_cache_key_async(
        config.cache_root.clone(),
        key.to_string(),
        std::sync::Arc::new(|_| {}),
    )
    .await?;
    crate::cache::prepare_write_paths(&path, key)?;
    let value = CachedQuery {
        stored_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        records: records.to_vec(),
    };
    cacache::write(&path, key, serde_json::to_vec(&value)?)
        .await
        .map_err(|error| source_error(&error.to_string()))?;
    if let Some(meta) = cacache::metadata(&path, key)
        .await
        .map_err(|error| source_error(&error.to_string()))?
    {
        crate::cache::secure_written_content(&path, &meta.integrity)?;
    }
    Ok(())
}

async fn query(
    client: reqwest::Client,
    base: String,
    candidate: String,
    all_candidates: Vec<String>,
    mode: SourceCacheMode,
) -> Result<Vec<FdaOrphanRecord>, BioMcpError> {
    let form = ordered_form(&candidate);
    let key = cache_key(&base, &form);
    if let Some(records) = read_cache(&key, mode).await {
        return Ok(filter_records(records, &all_candidates));
    }
    let url = format!("{}/OOPD_Results.cfm", base.trim_end_matches('/'));
    let response = client
        .post(url)
        .form(&form)
        .send()
        .await
        .map_err(BioMcpError::from)?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(source_error("unexpected HTTP status"));
    }
    let bytes =
        crate::sources::read_limited_body_with_limit(response, FDA_ORPHAN_SOURCE, MAX_BODY).await?;
    let records = parse_table(&bytes)?;
    write_cache(&key, &records, mode).await?;
    Ok(filter_records(records, &all_candidates))
}

pub(crate) async fn fetch(candidates: Vec<String>) -> FdaOrphanDesignations {
    fetch_with_mode(candidates, cache_mode()).await
}

async fn fetch_with_mode(candidates: Vec<String>, mode: SourceCacheMode) -> FdaOrphanDesignations {
    fetch_with_mode_and_deadline(candidates, mode, DEADLINE).await
}

async fn fetch_with_mode_and_deadline(
    candidates: Vec<String>,
    mode: SourceCacheMode,
    deadline: Duration,
) -> FdaOrphanDesignations {
    let candidates = normalize_candidates(candidates);
    let base = crate::sources::env_base(BASE, BASE_ENV).into_owned();
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(deadline)
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(_) => return unavailable(),
    };
    let count = candidates.len();
    let work = stream::iter(candidates.iter().cloned().map(|candidate| {
        query(
            client.clone(),
            base.clone(),
            candidate,
            candidates.clone(),
            mode,
        )
    }))
    .buffer_unordered(2)
    .collect::<Vec<_>>();
    let results = match tokio::time::timeout(deadline, work).await {
        Ok(results) => results,
        Err(_) => return unavailable(),
    };
    let failures = results.iter().filter(|result| result.is_err()).count();
    let successes = results
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    merge(successes, failures, count)
}

fn normalize_candidates(values: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .map(|v| clean(&v))
        .filter(|v| !v.is_empty() && seen.insert(v.to_ascii_lowercase()))
        .take(6)
        .collect()
}

fn merge(
    results: Vec<Vec<FdaOrphanRecord>>,
    failures: usize,
    attempted: usize,
) -> FdaOrphanDesignations {
    if results.is_empty() {
        return unavailable();
    }
    let mut by_key: HashMap<String, Option<FdaOrphanRecord>> = HashMap::new();
    let mut merge_failures = 0;
    for row in results.into_iter().flatten() {
        match by_key.get(&row.record_id) {
            None => {
                by_key.insert(row.record_id.clone(), Some(row));
            }
            Some(Some(existing)) if existing == &row => {}
            Some(Some(_)) => {
                by_key.insert(row.record_id.clone(), None);
                merge_failures += 1;
            }
            Some(None) => {}
        }
    }
    let mut records = by_key.into_values().flatten().collect::<Vec<_>>();
    records.sort_by(|a, b| {
        b.designation_date
            .cmp(&a.designation_date)
            .then_with(|| numeric_key_cmp(&a.record_id, &b.record_id))
    });
    envelope(records, failures + merge_failures, attempted)
}

fn numeric_key_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    let left_number = left.trim_start_matches('0');
    let right_number = right.trim_start_matches('0');
    left_number
        .len()
        .cmp(&right_number.len())
        .then_with(|| left_number.cmp(right_number))
        .then_with(|| left.cmp(right))
}

fn envelope(
    mut records: Vec<FdaOrphanRecord>,
    failures: usize,
    _attempted: usize,
) -> FdaOrphanDesignations {
    let total = records.len();
    let outcome = if failures > 0 {
        FdaOrphanOutcome::Degraded
    } else if total == 0 {
        FdaOrphanOutcome::Empty
    } else {
        FdaOrphanOutcome::Data
    };
    records.truncate(MAX_RECORDS);
    FdaOrphanDesignations {
        outcome,
        sources: vec![FDA_ORPHAN_SOURCE.into()],
        records,
        message: (failures > 0)
            .then(|| "Some FDA orphan-designation aliases were unavailable.".into()),
        total_matching: Some(total),
        truncated: total > MAX_RECORDS,
    }
}

fn unavailable() -> FdaOrphanDesignations {
    FdaOrphanDesignations {
        outcome: FdaOrphanOutcome::Unavailable,
        sources: Vec::new(),
        records: Vec::new(),
        message: Some("FDA orphan-designation data is temporarily unavailable.".into()),
        total_matching: None,
        truncated: false,
    }
}

pub(crate) async fn health_probe(_client: reqwest::Client) -> Result<(), BioMcpError> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(BioMcpError::HttpClientInit)?;
    let base = crate::sources::env_base(BASE, BASE_ENV);
    let url = format!("{}/OOPD_Results.cfm", base.trim_end_matches('/'));
    let response = client
        .post(url)
        .form(&ordered_form("eflornithine hydrochloride"))
        .send()
        .await
        .map_err(BioMcpError::from)?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(source_error("unexpected HTTP status"));
    }
    let bytes =
        crate::sources::read_limited_body_with_limit(response, FDA_ORPHAN_SOURCE, MAX_BODY).await?;
    parse_table(&bytes).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    use axum::{Router, body::Bytes, extract::State, http::StatusCode, routing::post};

    fn fixture(row: &str) -> String {
        format!(
            "<table><tr>{}</tr>{row}</table>",
            HEADERS
                .iter()
                .map(|h| format!("<th>{h}</th>"))
                .collect::<String>()
        )
    }

    #[test]
    fn form_is_exact_and_ordered() {
        assert_eq!(
            ordered_form("x"),
            vec![
                ("Product_name".into(), "x".into()),
                ("sponsor_name".into(), "".into()),
                ("Designation".into(), "".into()),
                ("Designation_Start_Date".into(), "".into()),
                ("Designation_End_Date".into(), "".into()),
                ("Search_param".into(), "DESDATE".into()),
                ("Output_Format".into(), "Excel".into()),
                ("Sort_order".into(), "GENERIC_NAME".into()),
                ("RecordsPerPage".into(), "25".into()),
                ("newSearch".into(), "Run Search".into()),
            ]
        );
    }

    #[test]
    fn cache_modes_and_expiry_are_explicit() {
        assert_eq!(
            source_cache_mode(true, Some("infinite")),
            SourceCacheMode::Off
        );
        assert_eq!(source_cache_mode(false, Some("off")), SourceCacheMode::Off);
        assert_eq!(
            source_cache_mode(false, Some("infinite")),
            SourceCacheMode::Infinite
        );
        assert_eq!(source_cache_mode(false, None), SourceCacheMode::Normal);
        assert!(cache_entry_is_usable(SourceCacheMode::Normal, 100, 99));
        assert!(!cache_entry_is_usable(
            SourceCacheMode::Normal,
            TTL.as_secs() + 1,
            0
        ));
        assert!(cache_entry_is_usable(
            SourceCacheMode::Infinite,
            u64::MAX,
            0
        ));
    }

    struct EnvGuard {
        name: &'static str,
        old: Option<std::ffi::OsString>,
    }
    impl EnvGuard {
        fn set(name: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
            let old = std::env::var_os(name);
            unsafe {
                std::env::set_var(name, value);
            }
            Self { name, old }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.old {
                    Some(value) => std::env::set_var(self.name, value),
                    None => std::env::remove_var(self.name),
                }
            }
        }
    }

    #[derive(Clone)]
    struct ServerState {
        bodies: Arc<Mutex<Vec<String>>>,
        active: Arc<AtomicUsize>,
        peak: Arc<AtomicUsize>,
        status: StatusCode,
        delay: Duration,
    }

    async fn form_handler(
        State(state): State<ServerState>,
        body: Bytes,
    ) -> (StatusCode, &'static str) {
        let active = state.active.fetch_add(1, Ordering::SeqCst) + 1;
        state.peak.fetch_max(active, Ordering::SeqCst);
        state
            .bodies
            .lock()
            .unwrap()
            .push(String::from_utf8(body.to_vec()).unwrap());
        tokio::time::sleep(state.delay).await;
        state.active.fetch_sub(1, Ordering::SeqCst);
        (
            state.status,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/fda_orphan/provider-shaped.html"
            )),
        )
    }

    async fn fixture_server(
        status: StatusCode,
    ) -> (String, ServerState, tokio::task::JoinHandle<()>) {
        fixture_server_with_delay(status, Duration::from_millis(20)).await
    }

    async fn fixture_server_with_delay(
        status: StatusCode,
        delay: Duration,
    ) -> (String, ServerState, tokio::task::JoinHandle<()>) {
        let state = ServerState {
            bodies: Default::default(),
            active: Default::default(),
            peak: Default::default(),
            status,
            delay,
        };
        let app = Router::new()
            .route("/OOPD_Results.cfm", post(form_handler))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), state, task)
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn acquisition_caps_requests_and_concurrency_and_preserves_exact_forms() {
        let (base, state, server) = fixture_server(StatusCode::OK).await;
        let _base = EnvGuard::set(BASE_ENV, &base);
        let cache = tempfile::tempdir().unwrap();
        let _cache = EnvGuard::set("BIOMCP_CACHE_DIR", cache.path());
        let result = fetch_with_mode(
            (0..8).map(|index| format!("candidate {index}")).collect(),
            SourceCacheMode::Off,
        )
        .await;
        server.abort();
        assert_eq!(result.outcome, FdaOrphanOutcome::Empty);
        assert_eq!(state.bodies.lock().unwrap().len(), 6);
        assert!(state.peak.load(Ordering::SeqCst) <= 2);
        let mut bodies = state.bodies.lock().unwrap().clone();
        bodies.sort();
        let mut expected = (0..6).map(|index| format!("Product_name=candidate+{index}&sponsor_name=&Designation=&Designation_Start_Date=&Designation_End_Date=&Search_param=DESDATE&Output_Format=Excel&Sort_order=GENERIC_NAME&RecordsPerPage=25&newSearch=Run+Search")).collect::<Vec<_>>();
        expected.sort();
        assert_eq!(bodies, expected);
        assert!(!cache.path().join("http").exists());
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn one_deadline_cancels_queued_alias_work() {
        let (base, state, server) =
            fixture_server_with_delay(StatusCode::OK, Duration::from_secs(1)).await;
        let _base = EnvGuard::set(BASE_ENV, &base);
        let started = std::time::Instant::now();
        let result = fetch_with_mode_and_deadline(
            (0..6).map(|index| format!("candidate {index}")).collect(),
            SourceCacheMode::Off,
            Duration::from_millis(40),
        )
        .await;
        server.abort();
        assert_eq!(result.outcome, FdaOrphanOutcome::Unavailable);
        assert!(started.elapsed() < Duration::from_millis(500));
        assert!(state.bodies.lock().unwrap().len() <= 2);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn infinite_cache_miss_fetches_stores_and_then_serves_without_network() {
        let (base, state, server) = fixture_server(StatusCode::OK).await;
        let cache = tempfile::tempdir().unwrap();
        let _base = EnvGuard::set(BASE_ENV, &base);
        let _cache = EnvGuard::set("BIOMCP_CACHE_DIR", cache.path());
        let candidates = vec!["eflornithine hydrochloride".into()];
        let first = fetch_with_mode(candidates.clone(), SourceCacheMode::Infinite).await;
        assert_eq!(first.outcome, FdaOrphanOutcome::Data);
        assert_eq!(state.bodies.lock().unwrap().len(), 1);
        server.abort();
        let second = fetch_with_mode(candidates, SourceCacheMode::Infinite).await;
        assert_eq!(second, first);
        assert_eq!(state.bodies.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn health_uses_one_uncached_known_form_and_http_failures_are_unavailable() {
        let (base, state, server) = fixture_server(StatusCode::OK).await;
        let _base = EnvGuard::set(BASE_ENV, &base);
        health_probe(reqwest::Client::new()).await.unwrap();
        server.abort();
        let bodies = state.bodies.lock().unwrap().clone();
        assert_eq!(bodies.len(), 1);
        assert_eq!(
            bodies[0],
            "Product_name=eflornithine+hydrochloride&sponsor_name=&Designation=&Designation_Start_Date=&Designation_End_Date=&Search_param=DESDATE&Output_Format=Excel&Sort_order=GENERIC_NAME&RecordsPerPage=25&newSearch=Run+Search"
        );

        let (base, _state, server) = fixture_server(StatusCode::FOUND).await;
        let _base = EnvGuard::set(BASE_ENV, &base);
        let value = fetch_with_mode(
            vec!["eflornithine hydrochloride".into()],
            SourceCacheMode::Off,
        )
        .await;
        server.abort();
        assert_eq!(value.outcome, FdaOrphanOutcome::Unavailable);
        assert!(value.sources.is_empty());
    }

    #[test]
    fn parser_keeps_exact_alias_and_truth_semantics() {
        let html = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/fda_orphan/provider-shaped.html"
        ));
        let rows = parse_response(html, &["eflornithine hydrochloride".into()]).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].record_id, "992323");
        assert_eq!(rows[0].orphan_approval, OrphanApproval::NotApproved);
        assert_eq!(rows[0].designation_date, "2024-03-11");
    }

    #[test]
    fn parser_rejects_prefix_and_bad_headers() {
        let cells = [
            "Eflornithine hydrochloride",
            "",
            "03/11/2024",
            "Use",
            "Designated",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "1",
        ];
        let html = fixture(&format!(
            "<tr>{}</tr>",
            cells
                .iter()
                .map(|v| format!("<td>{v}</td>"))
                .collect::<String>()
        ));
        assert!(
            parse_response(html.as_bytes(), &["eflornithine".into()])
                .unwrap()
                .is_empty()
        );
        assert!(parse_response(b"<table><tr><th>Wrong</th></tr></table>", &["x".into()]).is_err());
    }

    fn record(id: &str, date: &str, designation: &str) -> FdaOrphanRecord {
        FdaOrphanRecord {
            record_id: id.into(),
            generic_name: "drug".into(),
            trade_name: None,
            designation_date: date.into(),
            designation: designation.into(),
            designation_status: None,
            designation_withdrawn_or_revoked_date: None,
            orphan_approval: OrphanApproval::Unknown,
            orphan_approval_status_text: None,
            approved_labeled_indication: None,
            marketing_approval_date: None,
            exclusivity_end_date: None,
            exclusivity_protected_indication: None,
            sponsor: None,
            source_url: format!("{BASE}detailedIndex.cfm?cfgridkey={id}"),
        }
    }

    #[test]
    fn merge_is_order_independent_and_discards_conflicts() {
        let same = record("2", "2024-01-01", "same");
        let conflict = record("1", "2025-01-01", "left");
        let changed = record("1", "2025-01-01", "right");
        let first = merge(
            vec![
                vec![same.clone(), conflict.clone()],
                vec![same.clone(), changed.clone()],
            ],
            0,
            2,
        );
        let second = merge(
            vec![vec![changed, same.clone()], vec![conflict, same]],
            0,
            2,
        );
        assert_eq!(first, second);
        assert_eq!(first.outcome, FdaOrphanOutcome::Degraded);
        assert_eq!(
            first
                .records
                .iter()
                .map(|r| r.record_id.as_str())
                .collect::<Vec<_>>(),
            vec!["2"]
        );
    }

    #[test]
    fn envelopes_pin_counts_truncation_and_degraded_without_rows() {
        let data = envelope(
            (0..101)
                .map(|id| record(&id.to_string(), "2024-01-01", "use"))
                .collect(),
            0,
            1,
        );
        assert_eq!(
            (
                data.outcome,
                data.total_matching,
                data.records.len(),
                data.truncated
            ),
            (FdaOrphanOutcome::Data, Some(101), 100, true)
        );
        let empty = envelope(Vec::new(), 0, 1);
        assert_eq!(
            (empty.outcome, empty.total_matching, empty.truncated),
            (FdaOrphanOutcome::Empty, Some(0), false)
        );
        let degraded = envelope(Vec::new(), 1, 2);
        assert_eq!(
            (
                degraded.outcome,
                degraded.total_matching,
                degraded.truncated
            ),
            (FdaOrphanOutcome::Degraded, Some(0), false)
        );
        let unavailable = unavailable();
        assert_eq!(
            (
                unavailable.outcome,
                unavailable.total_matching,
                unavailable.truncated
            ),
            (FdaOrphanOutcome::Unavailable, None, false)
        );
    }

    #[test]
    fn approved_and_contradictory_facts_are_distinct() {
        let mut cells = vec![String::new(); 19];
        cells[0] = "drug".into();
        cells[2] = "01/02/2024".into();
        cells[3] = "use".into();
        cells[4] = "Designated / Approved".into();
        cells[18] = "12".into();
        assert_eq!(
            parse_row(cells.clone(), "drug".into(), None)
                .unwrap()
                .orphan_approval,
            OrphanApproval::Approved
        );
        cells[6] = "Not FDA Approved for Orphan Indication".into();
        assert!(parse_row(cells, "drug".into(), None).is_err());
    }

    #[test]
    fn json_schema_keeps_every_record_key_and_required_nulls() {
        let value =
            serde_json::to_value(envelope(vec![record("9", "2024-01-01", "use")], 0, 1)).unwrap();
        let record = &value["records"][0];
        assert_eq!(
            record
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<std::collections::BTreeSet<_>>(),
            [
                "record_id",
                "generic_name",
                "trade_name",
                "designation_date",
                "designation",
                "designation_status",
                "designation_withdrawn_or_revoked_date",
                "orphan_approval",
                "orphan_approval_status_text",
                "approved_labeled_indication",
                "marketing_approval_date",
                "exclusivity_end_date",
                "exclusivity_protected_indication",
                "sponsor",
                "source_url",
            ]
            .into_iter()
            .map(str::to_string)
            .collect()
        );
        assert!(record["trade_name"].is_null());
        assert_eq!(value["outcome"], "data");
        assert!(value.get("message").is_none());
    }
}
