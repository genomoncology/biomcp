use crate::sources::RequestBuilderSourceContextExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;

const SOURCE_NAME: &str = "CDC CVX/MVX";
const CVX_API: &str = "cdc-cvx";
pub(crate) const CVX_URL: &str = "https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/cvx.txt";
pub(crate) const CVX_URL_ENV: &str = "BIOMCP_CVX_URL";
pub(crate) const TRADENAME_URL: &str =
    "https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/TRADENAME.txt";
pub(crate) const TRADENAME_URL_ENV: &str = "BIOMCP_CVX_TRADENAME_URL";
pub(crate) const MVX_URL: &str = "https://www2.cdc.gov/vaccines/iis/iisstandards/downloads/mvx.txt";
pub(crate) const MVX_URL_ENV: &str = "BIOMCP_MVX_URL";
pub(crate) const CVX_FILE: &str = "cvx.txt";
pub(crate) const TRADENAME_FILE: &str = "TRADENAME.txt";
pub(crate) const MVX_FILE: &str = "mvx.txt";
pub(crate) const CVX_REQUIRED_FILES: &[&str] = &[CVX_FILE, TRADENAME_FILE, MVX_FILE];
pub(crate) const CVX_STALE_AFTER: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const CVX_MAX_BODY_BYTES: usize = 256 * 1024;
const TRADENAME_MAX_BODY_BYTES: usize = 256 * 1024;
const MVX_MAX_BODY_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CvxSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone)]
pub(crate) struct CvxClient {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Fresh,
    Missing,
    Stale,
}

#[derive(Debug, Clone)]
struct CvxCodeRow {
    cvx_code: String,
    short_description: String,
    full_vaccine_name: String,
    status: String,
    non_vaccine: bool,
}

#[derive(Debug, Clone)]
struct CvxProductRow {
    product_name: String,
    normalized_product_name: String,
    cvx_code: String,
    product_name_status: String,
    mvx_code: Option<String>,
}

#[derive(Debug, Clone)]
struct MvxRow {
    mvx_code: String,
    manufacturer_name: String,
}

// dead-code reason: cvx::CvxAliasRecord preserves the provider shape used by source contract fixtures
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone)]
struct CvxAliasRecord {
    cvx_code: String,
    product_name: String,
    normalized_product_name: String,
    product_name_status: String,
    cvx_status: String,
    cvx_short_description: String,
    cvx_full_vaccine_name: String,
    cvx_non_vaccine: bool,
    mvx_code: Option<String>,
    mvx_manufacturer_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CvxVaccineCandidate {
    pub cvx_code: String,
    pub product_name: String,
    pub short_description: String,
    pub full_vaccine_name: String,
}

impl CvxClient {
    // dead-code reason: binary adverse-event dispatch constructs the local CVX client
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self {
            root: resolve_cvx_root(),
        }
    }

    #[cfg(test)]
    fn from_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) async fn ready(mode: CvxSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_cvx_root();
        sync_cvx_root(&root, mode).await?;
        Ok(Self { root })
    }

    pub(crate) async fn sync(mode: CvxSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_cvx_root();
        sync_cvx_root(&root, mode).await
    }

    pub(crate) fn lookup_brand_aliases(&self, query: &str) -> Result<Vec<String>, BioMcpError> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for record in self.lookup_vaccine_candidates(query)? {
            for term in [&record.short_description, &record.full_vaccine_name] {
                let Some(dedupe_key) = normalize_match_key(term) else {
                    continue;
                };
                let value = clean_text(term).unwrap_or_else(|| term.trim().to_string());
                if seen.insert(dedupe_key) {
                    out.push(value);
                }
            }
        }

        Ok(out)
    }

    pub(crate) fn lookup_vaccine_candidates(
        &self,
        query: &str,
    ) -> Result<Vec<CvxVaccineCandidate>, BioMcpError> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for record in self.matching_alias_records(query)? {
            if seen.insert(record.cvx_code.clone()) {
                out.push(CvxVaccineCandidate {
                    cvx_code: record.cvx_code,
                    product_name: record.product_name,
                    short_description: record.cvx_short_description,
                    full_vaccine_name: record.cvx_full_vaccine_name,
                });
            }
        }
        Ok(out)
    }

    fn matching_alias_records(&self, query: &str) -> Result<Vec<CvxAliasRecord>, BioMcpError> {
        let Some(normalized_query) = normalize_match_key(query) else {
            return Ok(Vec::new());
        };

        let mut matches = self
            .read_alias_records()?
            .into_iter()
            .filter_map(|record| {
                if record.cvx_non_vaccine {
                    return None;
                }
                let match_rank = record_match_rank(&record, &normalized_query)?;
                Some((match_rank, record))
            })
            .collect::<Vec<_>>();

        matches.sort_by(|(rank_a, a), (rank_b, b)| {
            rank_a
                .cmp(rank_b)
                .then_with(|| {
                    active_sort_key(&a.product_name_status)
                        .cmp(&active_sort_key(&b.product_name_status))
                })
                .then_with(|| active_sort_key(&a.cvx_status).cmp(&active_sort_key(&b.cvx_status)))
                .then_with(|| a.normalized_product_name.cmp(&b.normalized_product_name))
                .then_with(|| a.product_name.cmp(&b.product_name))
        });

        Ok(matches.into_iter().map(|(_, record)| record).collect())
    }

    fn read_alias_records(&self) -> Result<Vec<CvxAliasRecord>, BioMcpError> {
        self.require_files(CVX_REQUIRED_FILES)?;

        let cvx_path = self.root.join(CVX_FILE);
        let tradename_path = self.root.join(TRADENAME_FILE);
        let mvx_path = self.root.join(MVX_FILE);

        let codes = parse_cvx_codes(
            &cvx_path,
            &std::fs::read_to_string(&cvx_path).map_err(|err| {
                cvx_read_error(
                    &self.root,
                    format!("Could not read {}: {err}", cvx_path.display()),
                )
            })?,
        )
        .map_err(|err| cvx_read_error(&self.root, err.to_string()))?;
        let products = parse_cvx_products(
            &tradename_path,
            &std::fs::read_to_string(&tradename_path).map_err(|err| {
                cvx_read_error(
                    &self.root,
                    format!("Could not read {}: {err}", tradename_path.display()),
                )
            })?,
        )
        .map_err(|err| cvx_read_error(&self.root, err.to_string()))?;
        let mvx_rows = parse_mvx_rows(
            &mvx_path,
            &std::fs::read_to_string(&mvx_path).map_err(|err| {
                cvx_read_error(
                    &self.root,
                    format!("Could not read {}: {err}", mvx_path.display()),
                )
            })?,
        )
        .map_err(|err| cvx_read_error(&self.root, err.to_string()))?;

        let code_map = codes
            .into_iter()
            .map(|row| (row.cvx_code.clone(), row))
            .collect::<HashMap<_, _>>();
        let mvx_map = mvx_rows
            .into_iter()
            .map(|row| (row.mvx_code.clone(), row))
            .collect::<HashMap<_, _>>();

        let mut out = Vec::new();
        for product in products {
            let Some(code) = code_map.get(&product.cvx_code) else {
                return Err(cvx_read_error(
                    &self.root,
                    format!(
                        "{} references unknown cvx_code {} from {}",
                        tradename_path.display(),
                        product.cvx_code,
                        cvx_path.display()
                    ),
                ));
            };
            let mvx = product
                .mvx_code
                .as_deref()
                .and_then(|mvx_code| mvx_map.get(mvx_code));
            out.push(CvxAliasRecord {
                cvx_code: code.cvx_code.clone(),
                product_name: product.product_name,
                normalized_product_name: product.normalized_product_name,
                product_name_status: product.product_name_status,
                cvx_status: code.status.clone(),
                cvx_short_description: code.short_description.clone(),
                cvx_full_vaccine_name: code.full_vaccine_name.clone(),
                cvx_non_vaccine: code.non_vaccine,
                mvx_code: product.mvx_code,
                mvx_manufacturer_name: mvx.map(|row| row.manufacturer_name.clone()),
            });
        }

        Ok(out)
    }

    fn require_files(&self, files: &[&str]) -> Result<(), BioMcpError> {
        let missing = cvx_missing_files(&self.root, files);
        if missing.is_empty() {
            return Ok(());
        }

        Err(cvx_read_error(
            &self.root,
            format!(
                "Missing required CDC CVX/MVX file(s): {}",
                missing.join(", ")
            ),
        ))
    }
}

fn active_sort_key(status: &str) -> u8 {
    if status.trim().eq_ignore_ascii_case("active") {
        0
    } else {
        1
    }
}

fn match_kind(normalized_product_name: &str, normalized_query: &str) -> Option<u8> {
    if normalized_product_name == normalized_query {
        Some(0)
    } else if normalized_product_name.starts_with(normalized_query)
        && normalized_product_name
            .as_bytes()
            .get(normalized_query.len())
            .is_some_and(|byte| *byte == b' ' || byte.is_ascii_digit())
    {
        Some(1)
    } else {
        None
    }
}

fn record_match_rank(record: &CvxAliasRecord, normalized_query: &str) -> Option<u8> {
    let mut best = match_kind(&record.normalized_product_name, normalized_query);
    for candidate in [&record.cvx_short_description, &record.cvx_full_vaccine_name] {
        let Some(normalized_candidate) = normalize_match_key(candidate) else {
            continue;
        };
        let Some(rank) = match_kind(&normalized_candidate, normalized_query) else {
            continue;
        };
        let adjusted_rank = rank.saturating_add(2);
        best = Some(best.map_or(adjusted_rank, |current| current.min(adjusted_rank)));
    }
    best
}

fn clean_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn optional_field(record: &csv::StringRecord, index: usize) -> Option<String> {
    record.get(index).and_then(clean_text)
}

fn required_field(
    record: &csv::StringRecord,
    index: usize,
    field_name: &str,
    source: &Path,
    row_no: usize,
) -> Result<String, BioMcpError> {
    optional_field(record, index)
        .ok_or_else(|| parse_error(source, row_no, format!("missing {field_name}")))
}

fn parse_error(source: &Path, row_no: usize, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::Api {
        api: CVX_API.to_string(),
        message: format!("{} row {}: {}", source.display(), row_no, detail.into()),
    }
}

fn parse_non_vaccine(source: &Path, row_no: usize, value: &str) -> Result<bool, BioMcpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(parse_error(
            source,
            row_no,
            format!("invalid non_vaccine boolean {other:?}"),
        )),
    }
}

fn parse_cvx_codes(source: &Path, payload: &str) -> Result<Vec<CvxCodeRow>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .flexible(true)
        .from_reader(payload.as_bytes());
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for (idx, result) in reader.records().enumerate() {
        let row_no = idx + 1;
        let record = result
            .map_err(|err| parse_error(source, row_no, format!("failed to parse row: {err}")))?;
        if record.len() < 7 {
            return Err(parse_error(
                source,
                row_no,
                format!("expected at least 7 fields, found {}", record.len()),
            ));
        }

        let cvx_code = required_field(&record, 0, "cvx_code", source, row_no)?;
        let short_description = required_field(&record, 1, "short_description", source, row_no)?;
        let full_vaccine_name = required_field(&record, 2, "full_vaccine_name", source, row_no)?;
        let status = required_field(&record, 4, "status", source, row_no)?;
        let non_vaccine = parse_non_vaccine(
            source,
            row_no,
            &required_field(&record, 5, "non_vaccine", source, row_no)?,
        )?;

        if seen.insert(cvx_code.clone()) {
            out.push(CvxCodeRow {
                cvx_code,
                short_description,
                full_vaccine_name,
                status,
                non_vaccine,
            });
        }
    }

    Ok(out)
}

fn parse_cvx_products(source: &Path, payload: &str) -> Result<Vec<CvxProductRow>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .flexible(true)
        .from_reader(payload.as_bytes());
    let mut out = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        let row_no = idx + 1;
        let record = result
            .map_err(|err| parse_error(source, row_no, format!("failed to parse row: {err}")))?;
        if record.len() < 8 {
            return Err(parse_error(
                source,
                row_no,
                format!("expected at least 8 fields, found {}", record.len()),
            ));
        }

        let product_name = required_field(&record, 0, "product_name", source, row_no)?;
        let normalized_product_name = normalize_match_key(&product_name)
            .ok_or_else(|| parse_error(source, row_no, "product_name normalized to empty"))?;
        let cvx_code = required_field(&record, 2, "cvx_code", source, row_no)?;
        let product_name_status =
            required_field(&record, 6, "product_name_status", source, row_no)?;

        out.push(CvxProductRow {
            product_name,
            normalized_product_name,
            cvx_code,
            product_name_status,
            mvx_code: optional_field(&record, 4),
        });
    }

    Ok(out)
}

fn parse_mvx_rows(source: &Path, payload: &str) -> Result<Vec<MvxRow>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .flexible(true)
        .from_reader(payload.as_bytes());
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for (idx, result) in reader.records().enumerate() {
        let row_no = idx + 1;
        let record = result
            .map_err(|err| parse_error(source, row_no, format!("failed to parse row: {err}")))?;
        if record.len() < 5 {
            return Err(parse_error(
                source,
                row_no,
                format!("expected at least 5 fields, found {}", record.len()),
            ));
        }

        let mvx_code = required_field(&record, 0, "mvx_code", source, row_no)?;
        let manufacturer_name = required_field(&record, 1, "manufacturer_name", source, row_no)?;
        let _status = required_field(&record, 3, "status", source, row_no)?;

        if seen.insert(mvx_code.clone()) {
            out.push(MvxRow {
                mvx_code,
                manufacturer_name,
            });
        }
    }

    Ok(out)
}

fn normalize_match_key(value: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut last_was_space = true;
    for ch in value.trim().chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            ' '
        };
        if mapped == ' ' {
            if !last_was_space {
                normalized.push(' ');
            }
            last_was_space = true;
        } else {
            normalized.push(mapped);
            last_was_space = false;
        }
    }

    let normalized = normalized.trim().to_string();
    (!normalized.is_empty()).then_some(normalized)
}

fn cvx_url() -> String {
    crate::sources::env_base(CVX_URL, CVX_URL_ENV).into_owned()
}

fn tradename_url() -> String {
    crate::sources::env_base(TRADENAME_URL, TRADENAME_URL_ENV).into_owned()
}

fn mvx_url() -> String {
    crate::sources::env_base(MVX_URL, MVX_URL_ENV).into_owned()
}

fn cvx_preseed_suggestion(root: &Path) -> String {
    format!(
        "Run `biomcp cvx sync`, place {} from {}, {} from {}, and {} from {} in {}, or set BIOMCP_CVX_DIR.",
        CVX_FILE,
        cvx_url(),
        TRADENAME_FILE,
        tradename_url(),
        MVX_FILE,
        mvx_url(),
        root.display()
    )
}

fn cvx_read_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not prepare CDC CVX/MVX data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: cvx_preseed_suggestion(root),
    }
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
        .is_some_and(|age| age >= CVX_STALE_AFTER)
}

fn normalize_sync_mode(mode: CvxSyncMode) -> CvxSyncMode {
    if matches!(mode, CvxSyncMode::Auto) && crate::sources::is_no_cache_enabled() {
        CvxSyncMode::Force
    } else {
        mode
    }
}

fn sync_state(root: &Path, mode: CvxSyncMode) -> SyncState {
    let missing = cvx_missing_files(root, CVX_REQUIRED_FILES);
    if matches!(normalize_sync_mode(mode), CvxSyncMode::Force) {
        return if missing.len() == CVX_REQUIRED_FILES.len() {
            SyncState::Missing
        } else {
            SyncState::Stale
        };
    }
    if !missing.is_empty() {
        return SyncState::Missing;
    }
    if CVX_REQUIRED_FILES
        .iter()
        .any(|file_name| file_is_stale(&root.join(file_name)))
    {
        SyncState::Stale
    } else {
        SyncState::Fresh
    }
}

fn sync_intro(state: SyncState, mode: CvxSyncMode) -> &'static str {
    if matches!(normalize_sync_mode(mode), CvxSyncMode::Force) {
        return "Refreshing";
    }
    match state {
        SyncState::Fresh => "Checking",
        SyncState::Missing => "Downloading",
        SyncState::Stale => "Refreshing stale",
    }
}

async fn sync_export(
    root: &Path,
    file_name: &str,
    url: &str,
    max_body_bytes: usize,
    mode: CvxSyncMode,
    parser: fn(&Path, &str) -> Result<(), BioMcpError>,
) -> Result<(), BioMcpError> {
    let client = crate::sources::shared_client()?;
    let path = root.join(file_name);
    let mut request = client.get(url).with_extension(
        if matches!(normalize_sync_mode(mode), CvxSyncMode::Force) {
            CacheMode::Reload
        } else {
            CacheMode::Default
        },
    );
    if matches!(normalize_sync_mode(mode), CvxSyncMode::Auto) && file_is_stale(&path) {
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }

    let response = crate::sources::with_response_body_limit(request, max_body_bytes, CVX_API)
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::CVX,
        ))
        .await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body = crate::sources::read_limited_source_body_with_limit(
        response,
        crate::error::SourceContext::narrow(crate::error::SourceProvider::CVX),
        max_body_bytes,
    )
    .await?;
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: CVX_API.to_string(),
            message: format!(
                "{}: HTTP {status}: {}",
                file_name,
                crate::sources::body_excerpt(&body)
            ),
        });
    }

    ensure_csv_content_type(content_type.as_ref(), &body)?;
    let payload = std::str::from_utf8(&body).map_err(|source| BioMcpError::Api {
        api: CVX_API.to_string(),
        message: format!("{file_name} was not valid UTF-8: {source}"),
    })?;
    parser(&path, payload)?;

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
            api: CVX_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::summarize_http_error_body(Some(header), body)
            ),
        });
    }
    Ok(())
}

async fn sync_cvx_root(root: &Path, mode: CvxSyncMode) -> Result<(), BioMcpError> {
    let state = sync_state(root, mode);
    if matches!(state, SyncState::Fresh) {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;
    write_stderr_line(&format!("{} CDC CVX/MVX data...", sync_intro(state, mode)))?;

    for (file_name, result) in [
        (
            CVX_FILE,
            sync_export(
                root,
                CVX_FILE,
                &cvx_url(),
                CVX_MAX_BODY_BYTES,
                mode,
                |path, payload| parse_cvx_codes(path, payload).map(|_| ()),
            )
            .await,
        ),
        (
            TRADENAME_FILE,
            sync_export(
                root,
                TRADENAME_FILE,
                &tradename_url(),
                TRADENAME_MAX_BODY_BYTES,
                mode,
                |path, payload| parse_cvx_products(path, payload).map(|_| ()),
            )
            .await,
        ),
        (
            MVX_FILE,
            sync_export(
                root,
                MVX_FILE,
                &mvx_url(),
                MVX_MAX_BODY_BYTES,
                mode,
                |path, payload| parse_mvx_rows(path, payload).map(|_| ()),
            )
            .await,
        ),
    ] {
        if let Err(err) = result {
            let path = root.join(file_name);
            if has_readable_local_file(&path) {
                write_stderr_line(&format!(
                    "Warning: CDC CVX/MVX refresh failed for {}: {err}. Using existing data.",
                    file_name
                ))?;
                continue;
            }
            return Err(cvx_read_error(root, err.to_string()));
        }
    }

    let missing = cvx_missing_files(root, CVX_REQUIRED_FILES);
    if missing.is_empty() {
        return Ok(());
    }

    Err(cvx_read_error(
        root,
        format!(
            "Missing required CDC CVX/MVX file(s): {}",
            missing.join(", ")
        ),
    ))
}

pub(crate) fn cvx_missing_files<'a>(root: &Path, files: &[&'a str]) -> Vec<&'a str> {
    files
        .iter()
        .filter(|file| !root.join(file).is_file())
        .copied()
        .collect()
}

pub(crate) fn resolve_cvx_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_CVX_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("cvx"),
        None => std::env::temp_dir().join("biomcp").join("cvx"),
    }
}

#[cfg(test)]
mod tests;
