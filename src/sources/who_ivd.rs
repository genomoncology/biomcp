use crate::sources::RequestBuilderSourceContextExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;

const SOURCE_NAME: &str = "WHO Prequalified IVD";
const WHO_IVD_API: &str = "who-ivd";
pub(crate) const WHO_IVD_EXPORT_URL: &str = "https://extranet.who.int/prequal/vitro-diagnostics/prequalified/in-vitro-diagnostics/export?page&_format=csv";
pub(crate) const WHO_IVD_EXPORT_URL_ENV: &str = "BIOMCP_WHO_IVD_URL";
pub(crate) const WHO_IVD_CSV_FILE: &str = "who_ivd.csv";
pub(crate) const WHO_IVD_REQUIRED_FILES: &[&str] = &[WHO_IVD_CSV_FILE];
pub(crate) const WHO_IVD_STALE_AFTER: Duration = Duration::from_secs(72 * 60 * 60);
const WHO_IVD_SIZE_HINT: &str = "~16 KB";
const WHO_IVD_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

const REQUIRED_HEADERS: &[&str] = &[
    "product code",
    "product name",
    "pathogen/disease/marker",
    "manufacturer name",
    "assay format",
    "regulatory version",
    "year prequalification",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhoIvdSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhoIvdRecord {
    pub product_code: String,
    pub product_name: String,
    pub target_marker: String,
    pub manufacturer_name: String,
    pub assay_format: String,
    pub regulatory_version: String,
    pub prequalification_year: String,
}

#[derive(Debug, Clone)]
pub(crate) struct WhoIvdClient {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Fresh,
    Missing,
    Stale,
}

impl WhoIvdClient {
    #[cfg(test)]
    pub(crate) fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) async fn ready(mode: WhoIvdSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_who_ivd_root();
        sync_who_ivd_root(&root, mode).await?;
        Ok(Self { root })
    }

    pub(crate) async fn sync(mode: WhoIvdSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_who_ivd_root();
        sync_who_ivd_root(&root, mode).await
    }

    pub(crate) fn read_rows(&self) -> Result<Vec<WhoIvdRecord>, BioMcpError> {
        let path = self.root.join(WHO_IVD_CSV_FILE);
        let payload =
            std::fs::read_to_string(&path).map_err(|err| BioMcpError::SourceUnavailable {
                source_name: SOURCE_NAME.to_string(),
                reason: format!("Could not read WHO IVD CSV at {}: {err}", path.display()),
                suggestion: who_ivd_preseed_suggestion(&self.root),
            })?;
        parse_who_ivd_csv(&payload)
    }

    #[cfg(test)]
    pub(crate) fn get(&self, product_code: &str) -> Result<Option<WhoIvdRecord>, BioMcpError> {
        let target = product_code.trim();
        if target.is_empty() {
            return Ok(None);
        }
        Ok(self
            .read_rows()?
            .into_iter()
            .find(|row| row.product_code == target))
    }
}

fn header_map(record: &csv::StringRecord) -> HashMap<String, usize> {
    record
        .iter()
        .enumerate()
        .map(|(idx, value)| (normalize_header(value), idx))
        .collect()
}

fn normalize_header(value: &str) -> String {
    value
        .trim_matches('\u{feff}')
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn clean_csv_field(
    record: &csv::StringRecord,
    headers: &HashMap<String, usize>,
    header: &str,
) -> String {
    headers
        .get(header)
        .and_then(|idx| record.get(*idx))
        .and_then(clean_text)
        .unwrap_or_default()
}

fn parse_who_ivd_csv(payload: &str) -> Result<Vec<WhoIvdRecord>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(payload.as_bytes());
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: WHO_IVD_API.to_string(),
        message: format!("Failed to read WHO IVD headers: {err}"),
    })?;
    let header_map = header_map(headers);
    for required in REQUIRED_HEADERS {
        if !header_map.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: WHO_IVD_API.to_string(),
                message: format!("{WHO_IVD_CSV_FILE} is missing required column: {required}"),
            });
        }
    }

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: WHO_IVD_API.to_string(),
            message: format!("Failed to parse {WHO_IVD_CSV_FILE}: {err}"),
        })?;
        let product_code = clean_csv_field(&record, &header_map, "product code");
        if product_code.is_empty() || !seen.insert(product_code.clone()) {
            continue;
        }
        out.push(WhoIvdRecord {
            product_code,
            product_name: clean_csv_field(&record, &header_map, "product name"),
            target_marker: clean_csv_field(&record, &header_map, "pathogen/disease/marker"),
            manufacturer_name: clean_csv_field(&record, &header_map, "manufacturer name"),
            assay_format: clean_csv_field(&record, &header_map, "assay format"),
            regulatory_version: clean_csv_field(&record, &header_map, "regulatory version"),
            prequalification_year: clean_csv_field(&record, &header_map, "year prequalification"),
        });
    }
    Ok(out)
}

fn who_ivd_export_url() -> String {
    crate::sources::env_base(WHO_IVD_EXPORT_URL, WHO_IVD_EXPORT_URL_ENV).into_owned()
}

fn who_ivd_preseed_suggestion(root: &Path) -> String {
    format!(
        "Run `biomcp who-ivd sync`, place {} from {} in {}, or set BIOMCP_WHO_IVD_DIR.",
        WHO_IVD_CSV_FILE,
        who_ivd_export_url(),
        root.display()
    )
}

fn write_stderr_line(line: &str) -> Result<(), BioMcpError> {
    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "{line}")?;
    Ok(())
}

fn has_readable_local_file(path: &Path) -> bool {
    path.is_file() && File::open(path).is_ok()
}

fn touch_file(path: &Path) -> Result<(), BioMcpError> {
    let file = std::fs::OpenOptions::new().write(true).open(path)?;
    file.set_modified(SystemTime::now())?;
    Ok(())
}

fn file_is_stale(path: &Path) -> bool {
    let modified = path
        .metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok());
    modified
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= WHO_IVD_STALE_AFTER)
}

fn sync_state(root: &Path, mode: WhoIvdSyncMode) -> SyncState {
    let missing = who_ivd_missing_files(root, WHO_IVD_REQUIRED_FILES);
    if matches!(mode, WhoIvdSyncMode::Force) {
        return if missing.len() == WHO_IVD_REQUIRED_FILES.len() {
            SyncState::Missing
        } else {
            SyncState::Stale
        };
    }
    if !missing.is_empty() {
        return SyncState::Missing;
    }
    if WHO_IVD_REQUIRED_FILES
        .iter()
        .any(|file_name| file_is_stale(&root.join(file_name)))
    {
        SyncState::Stale
    } else {
        SyncState::Fresh
    }
}

fn sync_intro(state: SyncState, mode: WhoIvdSyncMode) -> &'static str {
    if matches!(mode, WhoIvdSyncMode::Force) {
        return "Refreshing";
    }
    match state {
        SyncState::Fresh => "Checking",
        SyncState::Missing => "Downloading",
        SyncState::Stale => "Refreshing stale",
    }
}

fn who_ivd_sync_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not prepare WHO IVD data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: who_ivd_preseed_suggestion(root),
    }
}

async fn sync_who_ivd_root(root: &Path, mode: WhoIvdSyncMode) -> Result<(), BioMcpError> {
    let state = sync_state(root, mode);
    if matches!(state, SyncState::Fresh) {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;
    write_stderr_line(&format!(
        "{} WHO IVD data ({WHO_IVD_SIZE_HINT})...",
        sync_intro(state, mode)
    ))?;

    let path = root.join(WHO_IVD_CSV_FILE);
    if let Err(err) = sync_export(root, mode).await {
        if has_readable_local_file(&path) {
            write_stderr_line(&format!(
                "Warning: WHO IVD refresh failed: {err}. Using existing data."
            ))?;
        } else {
            return Err(who_ivd_sync_error(root, err.to_string()));
        }
    }

    let missing = who_ivd_missing_files(root, WHO_IVD_REQUIRED_FILES);
    if missing.is_empty() {
        return Ok(());
    }

    Err(who_ivd_sync_error(
        root,
        format!("Missing required WHO IVD file(s): {}", missing.join(", ")),
    ))
}

async fn sync_export(root: &Path, mode: WhoIvdSyncMode) -> Result<(), BioMcpError> {
    let client = crate::sources::shared_client()?;
    let mut request = client
        .get(who_ivd_export_url())
        .with_extension(CacheMode::NoStore);
    if matches!(mode, WhoIvdSyncMode::Force) {
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }
    let response =
        crate::sources::with_response_body_limit(request, WHO_IVD_MAX_BODY_BYTES, WHO_IVD_API)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::WHO_IVD,
            ))
            .await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body = crate::sources::read_limited_source_body_with_limit(
        response,
        crate::error::SourceContext::narrow(crate::error::SourceProvider::WHO_IVD),
        WHO_IVD_MAX_BODY_BYTES,
    )
    .await?;

    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::WHO_IVD);
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: WHO_IVD_API.to_string(),
            message: format!(
                "{}: HTTP {status}: {}",
                WHO_IVD_CSV_FILE,
                crate::sources::body_excerpt(&body)
            ),
        }
        .with_source_context(context));
    }

    ensure_csv_content_type(content_type.as_ref(), &body)
        .map_err(|error| error.with_source_context(context))?;
    let payload = std::str::from_utf8(&body)
        .map_err(|source| BioMcpError::Api {
            api: WHO_IVD_API.to_string(),
            message: format!("{WHO_IVD_CSV_FILE} was not valid UTF-8: {source}"),
        })
        .map_err(|error| error.with_source_context(context))?;
    parse_who_ivd_csv(payload).map_err(|error| error.with_source_context(context))?;

    let path = root.join(WHO_IVD_CSV_FILE);
    if let Ok(existing) = tokio::fs::read(&path).await
        && existing == body
    {
        touch_file(&path)?;
        return Ok(());
    }

    crate::utils::download::write_atomic_bytes(&path, &body).await
}

fn ensure_csv_content_type(
    header: Option<&reqwest::header::HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    let Some(header) = header else {
        return Ok(());
    };
    let Ok(raw) = header.to_str() else {
        return Ok(());
    };
    let media_type = raw
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if matches!(media_type.as_str(), "text/html" | "application/xhtml+xml") {
        return Err(BioMcpError::Api {
            api: WHO_IVD_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::summarize_http_error_body(Some(header), body)
            ),
        });
    }
    Ok(())
}

pub(crate) fn who_ivd_missing_files<'a>(root: &Path, files: &[&'a str]) -> Vec<&'a str> {
    files
        .iter()
        .filter(|file| !root.join(file).is_file())
        .copied()
        .collect()
}

pub(crate) fn resolve_who_ivd_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_WHO_IVD_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("who-ivd"),
        None => std::env::temp_dir().join("biomcp").join("who-ivd"),
    }
}

#[cfg(test)]
mod tests;
