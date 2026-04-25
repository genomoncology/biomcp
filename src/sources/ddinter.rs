use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use csv::ReaderBuilder;
use http_cache_reqwest::CacheMode;
use serde::Deserialize;

use crate::error::BioMcpError;

const SOURCE_NAME: &str = "DDInter";
const DDINTER_API: &str = "ddinter";
pub(crate) const DDINTER_STALE_AFTER: Duration = Duration::from_secs(72 * 60 * 60);

const DDINTER_BUNDLE: [(&str, &str); 8] = [
    (
        "ddinter_downloads_code_A.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_A.csv",
    ),
    (
        "ddinter_downloads_code_B.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_B.csv",
    ),
    (
        "ddinter_downloads_code_D.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_D.csv",
    ),
    (
        "ddinter_downloads_code_H.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_H.csv",
    ),
    (
        "ddinter_downloads_code_L.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_L.csv",
    ),
    (
        "ddinter_downloads_code_P.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_P.csv",
    ),
    (
        "ddinter_downloads_code_R.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_R.csv",
    ),
    (
        "ddinter_downloads_code_V.csv",
        "https://ddinter.scbdd.com/static/media/download/ddinter_downloads_code_V.csv",
    ),
];

pub(crate) const DDINTER_REQUIRED_FILES: &[&str] = &[
    DDINTER_BUNDLE[0].0,
    DDINTER_BUNDLE[1].0,
    DDINTER_BUNDLE[2].0,
    DDINTER_BUNDLE[3].0,
    DDINTER_BUNDLE[4].0,
    DDINTER_BUNDLE[5].0,
    DDINTER_BUNDLE[6].0,
    DDINTER_BUNDLE[7].0,
];

const DDINTER_MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DdinterSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone)]
pub(crate) struct DdinterIdentity {
    terms: Vec<String>,
}

impl DdinterIdentity {
    pub(crate) fn with_aliases(primary: &str, canonical: Option<&str>, aliases: &[String]) -> Self {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for candidate in std::iter::once(primary)
            .chain(canonical.into_iter())
            .chain(aliases.iter().map(String::as_str))
        {
            let Some(key) = normalize_name_key(candidate) else {
                continue;
            };
            if seen.insert(key.clone()) {
                out.push(key);
            }
        }
        Self { terms: out }
    }

    pub(crate) fn terms(&self) -> &[String] {
        &self.terms
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DdinterInteractionRow {
    pub drug_a_id: String,
    pub drug_a: String,
    pub drug_b_id: String,
    pub drug_b: String,
    pub level: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DdinterClient {
    root: PathBuf,
    index: Arc<DdinterIndex>,
}

#[derive(Debug, Default)]
struct DdinterIndex {
    rows: Vec<DdinterInteractionRow>,
    by_name: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Deserialize)]
struct DdinterCsvRow {
    #[serde(rename = "DDInterID_A")]
    ddinter_id_a: String,
    #[serde(rename = "Drug_A")]
    drug_a: String,
    #[serde(rename = "DDInterID_B")]
    ddinter_id_b: String,
    #[serde(rename = "Drug_B")]
    drug_b: String,
    #[serde(rename = "Level")]
    level: String,
}

impl DdinterClient {
    pub(crate) async fn ready(mode: DdinterSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_ddinter_root();
        sync_ddinter_root(&root, mode).await?;
        evict_cached_index(&root);
        let index = cached_index_for_root(&root)?;
        Ok(Self { root, index })
    }

    pub(crate) async fn sync(mode: DdinterSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_ddinter_root();
        sync_ddinter_root(&root, mode).await?;
        evict_cached_index(&root);
        Ok(())
    }

    pub(crate) fn interactions(&self, identity: &DdinterIdentity) -> Vec<DdinterInteractionRow> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for term in identity.terms() {
            let Some(indices) = self.index.by_name.get(term) else {
                continue;
            };
            for &idx in indices {
                if seen.insert(idx) {
                    out.push(self.index.rows[idx].clone());
                }
            }
        }
        out
    }

    #[allow(dead_code)]
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
}

fn cached_index_map() -> &'static Mutex<HashMap<PathBuf, Arc<DdinterIndex>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<DdinterIndex>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cached_index_for_root(root: &Path) -> Result<Arc<DdinterIndex>, BioMcpError> {
    if let Ok(cache) = cached_index_map().lock()
        && let Some(index) = cache.get(root)
    {
        return Ok(index.clone());
    }

    let parsed = Arc::new(load_index(root)?);
    let mut cache = cached_index_map().lock().map_err(|_| BioMcpError::Api {
        api: DDINTER_API.to_string(),
        message: "DDInter index cache lock poisoned".into(),
    })?;
    Ok(cache
        .entry(root.to_path_buf())
        .or_insert_with(|| parsed.clone())
        .clone())
}

fn evict_cached_index(root: &Path) {
    if let Ok(mut cache) = cached_index_map().lock() {
        cache.remove(root);
    }
}

fn load_index(root: &Path) -> Result<DdinterIndex, BioMcpError> {
    let missing = ddinter_missing_files(root, DDINTER_REQUIRED_FILES);
    if !missing.is_empty() {
        return Err(ddinter_read_error(
            root,
            format!("Missing required DDInter file(s): {}", missing.join(", ")),
        ));
    }

    let mut rows = Vec::new();
    let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
    for file_name in DDINTER_REQUIRED_FILES {
        let path = root.join(file_name);
        let body = std::fs::read(&path).map_err(|err| ddinter_read_error(root, err.to_string()))?;
        let file_rows = parse_csv_rows(file_name, &body)?;
        for row in file_rows {
            let idx = rows.len();
            if let Some(key) = normalize_name_key(&row.drug_a) {
                by_name.entry(key).or_default().push(idx);
            }
            if let Some(key) = normalize_name_key(&row.drug_b) {
                by_name.entry(key).or_default().push(idx);
            }
            rows.push(row);
        }
    }
    Ok(DdinterIndex { rows, by_name })
}

fn parse_csv_rows(file_name: &str, body: &[u8]) -> Result<Vec<DdinterInteractionRow>, BioMcpError> {
    let mut reader = ReaderBuilder::new().trim(csv::Trim::All).from_reader(body);
    let mut out = Vec::new();
    for row in reader.deserialize::<DdinterCsvRow>() {
        let row = row.map_err(|source| BioMcpError::Api {
            api: DDINTER_API.to_string(),
            message: format!("{file_name} could not be parsed: {source}"),
        })?;
        if row.ddinter_id_a.trim().is_empty()
            || row.drug_a.trim().is_empty()
            || row.ddinter_id_b.trim().is_empty()
            || row.drug_b.trim().is_empty()
        {
            return Err(BioMcpError::Api {
                api: DDINTER_API.to_string(),
                message: format!("{file_name} contained an incomplete interaction row"),
            });
        }
        out.push(DdinterInteractionRow {
            drug_a_id: row.ddinter_id_a.trim().to_string(),
            drug_a: row.drug_a.trim().to_string(),
            drug_b_id: row.ddinter_id_b.trim().to_string(),
            drug_b: row.drug_b.trim().to_string(),
            level: (!row.level.trim().is_empty()).then(|| row.level.trim().to_string()),
        });
    }
    Ok(out)
}

fn file_is_stale(path: &Path, stale_after: Duration) -> bool {
    path.metadata()
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| std::time::SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age >= stale_after)
}

fn sync_plan(root: &Path, mode: DdinterSyncMode) -> Vec<(&'static str, &'static str)> {
    DDINTER_BUNDLE
        .iter()
        .copied()
        .filter(|(file_name, _)| {
            matches!(mode, DdinterSyncMode::Force)
                || !root.join(file_name).is_file()
                || file_is_stale(&root.join(file_name), DDINTER_STALE_AFTER)
        })
        .collect()
}

fn sync_intro(plan_len: usize, mode: DdinterSyncMode) -> &'static str {
    if matches!(mode, DdinterSyncMode::Force) {
        "Refreshing"
    } else if plan_len == DDINTER_BUNDLE.len() {
        "Downloading"
    } else {
        "Updating"
    }
}

fn write_stderr_line(line: &str) -> Result<(), BioMcpError> {
    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "{line}")?;
    Ok(())
}

async fn sync_ddinter_root(root: &Path, mode: DdinterSyncMode) -> Result<(), BioMcpError> {
    let plan = sync_plan(root, mode);
    if plan.is_empty() {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;
    write_stderr_line(&format!(
        "{} DDInter data (eight DDInter CSV files)...",
        sync_intro(plan.len(), mode)
    ))?;

    let mut fatal_errors = Vec::new();
    for (file_name, url) in plan {
        if let Err(err) = sync_export(root, file_name, url, mode).await {
            let path = root.join(file_name);
            if path.is_file() {
                write_stderr_line(&format!(
                    "Warning: DDInter refresh failed for {file_name}: {err}. Using existing data."
                ))?;
                continue;
            }
            fatal_errors.push(format!("{file_name}: {err}"));
        }
    }

    let missing = ddinter_missing_files(root, DDINTER_REQUIRED_FILES);
    if missing.is_empty() {
        return Ok(());
    }

    let detail = if fatal_errors.is_empty() {
        format!("Missing required DDInter file(s): {}", missing.join(", "))
    } else {
        format!(
            "{} Missing required DDInter file(s): {}",
            fatal_errors.join("; "),
            missing.join(", ")
        )
    };
    Err(ddinter_sync_error(root, detail))
}

async fn sync_export(
    root: &Path,
    file_name: &str,
    url: &str,
    mode: DdinterSyncMode,
) -> Result<(), BioMcpError> {
    let client = crate::sources::shared_client()?;
    let mut request = client.get(url).with_extension(CacheMode::NoStore);
    if matches!(mode, DdinterSyncMode::Force) {
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }

    let response = request.send().await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body =
        crate::sources::read_limited_body_with_limit(response, DDINTER_API, DDINTER_MAX_BODY_BYTES)
            .await?;

    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: DDINTER_API.to_string(),
            message: format!(
                "{file_name}: HTTP {status}: {}",
                crate::sources::body_excerpt(&body)
            ),
        });
    }

    ensure_csv_content_type(content_type.as_ref(), &body)?;
    parse_csv_rows(file_name, &body)?;
    crate::utils::download::write_atomic_bytes(&root.join(file_name), &body).await
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
            api: DDINTER_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::body_excerpt(body)
            ),
        });
    }
    Ok(())
}

fn ddinter_sync_error(root: &Path, detail: String) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: detail,
        suggestion: format!(
            "Retry with network access or run `biomcp ddinter sync`. You can also preseed the DDInter CSV bundle into {} or set BIOMCP_DDINTER_DIR.",
            root.display()
        ),
    }
}

fn ddinter_read_error(root: &Path, detail: String) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: detail,
        suggestion: format!(
            "Run `biomcp ddinter sync` or preseed the DDInter CSV bundle into {} or set BIOMCP_DDINTER_DIR.",
            root.display()
        ),
    }
}

pub(crate) fn ddinter_missing_files<'a>(root: &Path, files: &[&'a str]) -> Vec<&'a str> {
    files
        .iter()
        .filter(|file| !root.join(file).is_file())
        .copied()
        .collect()
}

pub(crate) fn resolve_ddinter_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_DDINTER_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("ddinter"),
        None => std::env::temp_dir().join("biomcp").join("ddinter"),
    }
}

pub(crate) fn normalize_name_key(value: &str) -> Option<String> {
    let normalized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::{
        DDINTER_REQUIRED_FILES, DdinterIdentity, DdinterIndex, ddinter_missing_files,
        normalize_name_key, parse_csv_rows,
    };

    #[test]
    fn normalize_name_key_collapses_spacing_and_case() {
        assert_eq!(
            normalize_name_key("Asparaginase Escherichia coli"),
            Some("asparaginase escherichia coli".to_string())
        );
        assert_eq!(
            normalize_name_key("  Warfarin Sodium "),
            Some("warfarin sodium".to_string())
        );
    }

    #[test]
    fn ddinter_identity_dedupes_alias_terms() {
        let aliases = vec!["Coumadin".to_string(), "WARFARIN".to_string()];
        let identity = DdinterIdentity::with_aliases("warfarin", Some("Warfarin"), &aliases);
        assert_eq!(
            identity.terms(),
            &["warfarin".to_string(), "coumadin".to_string()]
        );
    }

    #[test]
    fn parse_csv_rows_reads_expected_shape() {
        let payload = b"DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level\nDDInter1,Abacavir,DDInter2,Warfarin,Moderate\n";
        let rows = parse_csv_rows("fixture.csv", payload).expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].drug_a, "Abacavir");
        assert_eq!(rows[0].drug_b, "Warfarin");
        assert_eq!(rows[0].level.as_deref(), Some("Moderate"));
    }

    #[test]
    fn ddinter_missing_files_reports_incomplete_bundle() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::write(root.path().join(DDINTER_REQUIRED_FILES[0]), b"ok").expect("write");
        let missing = ddinter_missing_files(root.path(), DDINTER_REQUIRED_FILES);
        assert_eq!(missing.len(), DDINTER_REQUIRED_FILES.len() - 1);
    }

    #[test]
    fn ddinter_index_lookup_matches_both_sides() {
        let rows = parse_csv_rows(
            "fixture.csv",
            b"DDInterID_A,Drug_A,DDInterID_B,Drug_B,Level\nDDInter1,Abacavir,DDInter2,Warfarin,Moderate\nDDInter2,Warfarin,DDInter3,Aspirin,Major\n",
        )
        .expect("rows");
        let mut index = DdinterIndex::default();
        for row in rows {
            let idx = index.rows.len();
            if let Some(key) = normalize_name_key(&row.drug_a) {
                index.by_name.entry(key).or_default().push(idx);
            }
            if let Some(key) = normalize_name_key(&row.drug_b) {
                index.by_name.entry(key).or_default().push(idx);
            }
            index.rows.push(row);
        }
        let identity = DdinterIdentity::with_aliases("Warfarin", None, &[]);
        let mut seen = std::collections::HashSet::new();
        let matches = identity
            .terms()
            .iter()
            .flat_map(|term| index.by_name.get(term).into_iter().flatten().copied())
            .filter(|idx| seen.insert(*idx))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 2);
    }
}
