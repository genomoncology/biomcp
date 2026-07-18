use crate::sources::RequestBuilderSourceContextExt;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use flate2::read::GzDecoder;
use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;

const SOURCE_NAME: &str = "NCBI Genetic Testing Registry";
const GTR_API: &str = "gtr";
pub(crate) const GTR_TEST_VERSION_URL: &str =
    "https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_version.gz";
pub(crate) const GTR_CONDITION_GENE_URL: &str =
    "https://ftp.ncbi.nlm.nih.gov/pub/GTR/data/test_condition_gene.txt";
pub(crate) const GTR_TEST_VERSION_URL_ENV: &str = "BIOMCP_GTR_TEST_VERSION_URL";
pub(crate) const GTR_CONDITION_GENE_URL_ENV: &str = "BIOMCP_GTR_CONDITION_GENE_URL";
pub(crate) const GTR_TEST_VERSION_FILE: &str = "test_version.gz";
pub(crate) const GTR_CONDITION_GENE_FILE: &str = "test_condition_gene.txt";
pub(crate) const GTR_REQUIRED_FILES: [&str; 2] = [GTR_TEST_VERSION_FILE, GTR_CONDITION_GENE_FILE];
pub(crate) const GTR_STALE_AFTER: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const GTR_TEST_VERSION_MAX_BODY_BYTES: usize = 100 * 1024 * 1024;
const GTR_CONDITION_GENE_MAX_BODY_BYTES: usize = 50 * 1024 * 1024;

const TEST_VERSION_REQUIRED_HEADERS: &[&str] = &[
    "test_accession_ver",
    "now_current",
    "lab_test_name",
    "manufacturer_test_name",
    "name_of_laboratory",
    "name_of_institution",
    "CLIA_number",
    "state_licenses",
    "facility_country",
    "test_currStat",
    "test_pubStat",
    "method_categories",
    "methods",
    "genes",
];

const CONDITION_GENE_REQUIRED_HEADERS: &[&str] = &["#accession_version", "object", "object_name"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GtrSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone)]
pub(crate) struct GtrClient {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GtrRecord {
    pub accession: String,
    pub lab_test_name: String,
    pub manufacturer_test_name: String,
    pub test_type: String,
    pub name_of_laboratory: String,
    pub name_of_institution: String,
    pub clia_number: String,
    pub state_licenses: String,
    pub facility_country: String,
    pub test_curr_stat: String,
    pub test_pub_stat: String,
    pub method_categories: Vec<String>,
    pub methods: Vec<String>,
    pub genes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct GtrIndex {
    pub records_by_id: HashMap<String, GtrRecord>,
    pub genes_by_id: HashMap<String, Vec<String>>,
    pub conditions_by_id: HashMap<String, Vec<String>>,
    pub test_types_by_id: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Fresh,
    Missing,
    Stale,
}

type LinkMap = HashMap<String, Vec<String>>;

impl GtrClient {
    #[cfg(test)]
    pub(crate) fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) async fn ready(mode: GtrSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_gtr_root();
        sync_gtr_root(&root, mode).await?;
        Ok(Self { root })
    }

    pub(crate) async fn sync(mode: GtrSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_gtr_root();
        sync_gtr_root(&root, mode).await
    }

    pub(crate) fn load_index(&self) -> Result<GtrIndex, BioMcpError> {
        self.require_files()?;
        let mut records_by_id = read_test_version_records(&self.root.join(GTR_TEST_VERSION_FILE))
            .map_err(|err| gtr_read_error(&self.root, err.to_string()))?;
        let (genes_by_id, conditions_by_id, test_types_by_id) =
            read_condition_gene_links(&self.root.join(GTR_CONDITION_GENE_FILE))
                .map_err(|err| gtr_read_error(&self.root, err.to_string()))?;
        for (accession, record) in &mut records_by_id {
            if record.test_type.is_empty()
                && let Some(test_type) = test_types_by_id
                    .get(accession)
                    .and_then(|values| values.first())
            {
                record.test_type = test_type.clone();
            }
        }

        Ok(GtrIndex {
            records_by_id,
            genes_by_id,
            conditions_by_id,
            test_types_by_id,
        })
    }

    fn require_files(&self) -> Result<(), BioMcpError> {
        let missing = gtr_missing_files(&self.root);
        if missing.is_empty() {
            return Ok(());
        }
        Err(gtr_read_error(
            &self.root,
            format!("Missing required GTR file(s): {}", missing.join(", ")),
        ))
    }
}

fn gene_symbol(raw: &str) -> Option<&str> {
    let symbol = raw.split_once(':').map_or(raw, |(symbol, _)| symbol).trim();
    (!symbol.is_empty()).then_some(symbol)
}

fn push_merged_gene(out: &mut Vec<String>, seen: &mut HashSet<String>, raw: &str) {
    let Some(symbol) = gene_symbol(raw) else {
        return;
    };
    if seen.insert(symbol.to_ascii_lowercase()) {
        out.push(symbol.to_string());
    }
}

impl GtrIndex {
    pub(crate) fn record(&self, accession: &str) -> Option<&GtrRecord> {
        self.records_by_id.get(accession)
    }

    pub(crate) fn merged_genes(&self, accession: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        if let Some(genes) = self.genes_by_id.get(accession) {
            for gene in genes {
                push_merged_gene(&mut out, &mut seen, gene);
            }
        }
        if let Some(record) = self.records_by_id.get(accession) {
            for gene in &record.genes {
                push_merged_gene(&mut out, &mut seen, gene);
            }
        }
        out
    }

    pub(crate) fn conditions(&self, accession: &str) -> Vec<String> {
        self.conditions_by_id
            .get(accession)
            .cloned()
            .unwrap_or_default()
    }
}

fn normalize_sync_mode(mode: GtrSyncMode) -> GtrSyncMode {
    if matches!(mode, GtrSyncMode::Auto) && crate::sources::is_no_cache_enabled() {
        GtrSyncMode::Force
    } else {
        mode
    }
}

fn sync_state(root: &Path, mode: GtrSyncMode) -> SyncState {
    let missing = gtr_missing_files(root);
    if matches!(normalize_sync_mode(mode), GtrSyncMode::Force) {
        return if missing.len() == GTR_REQUIRED_FILES.len() {
            SyncState::Missing
        } else {
            SyncState::Stale
        };
    }
    if !missing.is_empty() {
        return SyncState::Missing;
    }
    if GTR_REQUIRED_FILES
        .iter()
        .any(|file_name| file_is_stale(&root.join(file_name)))
    {
        SyncState::Stale
    } else {
        SyncState::Fresh
    }
}

fn sync_intro(state: SyncState, mode: GtrSyncMode) -> &'static str {
    if matches!(normalize_sync_mode(mode), GtrSyncMode::Force) {
        return "Refreshing";
    }
    match state {
        SyncState::Fresh => "Checking",
        SyncState::Missing => "Downloading",
        SyncState::Stale => "Refreshing stale",
    }
}

async fn sync_gtr_root(root: &Path, mode: GtrSyncMode) -> Result<(), BioMcpError> {
    let state = sync_state(root, mode);
    if matches!(state, SyncState::Fresh) {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;
    write_stderr_line(&format!("{} GTR data...", sync_intro(state, mode)))?;

    let has_local_pair = has_readable_local_pair(root);
    let downloaded = download_pair(root, state, mode).await;
    match downloaded {
        Ok((test_version_body, condition_gene_body)) => {
            if let Err(err) =
                write_validated_pair(root, &test_version_body, &condition_gene_body).await
            {
                if has_local_pair {
                    write_stderr_line(&format!(
                        "Warning: GTR refresh failed: {err}. Using existing data."
                    ))?;
                    return Ok(());
                }
                return Err(gtr_sync_error(root, err.to_string()));
            }
        }
        Err(err) => {
            if has_local_pair {
                write_stderr_line(&format!(
                    "Warning: GTR refresh failed: {err}. Using existing data."
                ))?;
                return Ok(());
            }
            return Err(gtr_sync_error(root, err.to_string()));
        }
    }

    let missing = gtr_missing_files(root);
    if missing.is_empty() {
        return Ok(());
    }

    Err(gtr_sync_error(
        root,
        format!("Missing required GTR file(s): {}", missing.join(", ")),
    ))
}

async fn download_pair(
    root: &Path,
    state: SyncState,
    mode: GtrSyncMode,
) -> Result<(Vec<u8>, Vec<u8>), BioMcpError> {
    let stale = matches!(state, SyncState::Stale);
    let test_version_body = download_payload(
        root,
        GTR_TEST_VERSION_FILE,
        &gtr_test_version_url(),
        GTR_TEST_VERSION_MAX_BODY_BYTES,
        mode,
        stale,
    )
    .await?;
    let condition_gene_body = download_payload(
        root,
        GTR_CONDITION_GENE_FILE,
        &gtr_condition_gene_url(),
        GTR_CONDITION_GENE_MAX_BODY_BYTES,
        mode,
        stale,
    )
    .await?;
    Ok((test_version_body, condition_gene_body))
}

async fn download_payload(
    root: &Path,
    file_name: &str,
    url: &str,
    max_body_bytes: usize,
    mode: GtrSyncMode,
    stale: bool,
) -> Result<Vec<u8>, BioMcpError> {
    let client = crate::sources::shared_client()?;
    let mut request = client.get(url).with_extension(
        if matches!(normalize_sync_mode(mode), GtrSyncMode::Force) {
            CacheMode::Reload
        } else {
            CacheMode::Default
        },
    );
    if matches!(normalize_sync_mode(mode), GtrSyncMode::Auto) && stale {
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }
    let response = crate::sources::with_response_body_limit(request, max_body_bytes, GTR_API)
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::GTR,
        ))
        .await?;
    let status = response.status();
    let body = crate::sources::read_limited_source_body_with_limit(
        response,
        crate::error::SourceContext::narrow(crate::error::SourceProvider::GTR),
        max_body_bytes,
    )
    .await?;
    if !status.is_success() {
        return Err(gtr_sync_error(
            root,
            format!(
                "{}: HTTP {status}: {}",
                file_name,
                crate::sources::body_excerpt(&body)
            ),
        )
        .with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::GTR,
        )));
    }
    Ok(body)
}

async fn write_validated_pair(
    root: &Path,
    test_version_body: &[u8],
    condition_gene_body: &[u8],
) -> Result<(), BioMcpError> {
    let context = crate::error::SourceContext::retry(crate::error::SourceProvider::GTR);
    validate_test_version_payload(test_version_body)
        .map_err(|error| error.with_source_context(context))?;
    validate_condition_gene_payload(condition_gene_body)
        .map_err(|error| error.with_source_context(context))?;

    let test_version_path = root.join(GTR_TEST_VERSION_FILE);
    let condition_gene_path = root.join(GTR_CONDITION_GENE_FILE);

    let previous_test_version = tokio::fs::read(&test_version_path).await.ok();
    let previous_condition_gene = tokio::fs::read(&condition_gene_path).await.ok();
    let test_version_unchanged = previous_test_version
        .as_deref()
        .is_some_and(|existing| existing == test_version_body);
    let condition_gene_unchanged = previous_condition_gene
        .as_deref()
        .is_some_and(|existing| existing == condition_gene_body);

    if !test_version_unchanged {
        crate::utils::download::write_atomic_bytes(&test_version_path, test_version_body).await?;
    }

    if !condition_gene_unchanged
        && let Err(err) =
            crate::utils::download::write_atomic_bytes(&condition_gene_path, condition_gene_body)
                .await
    {
        if !test_version_unchanged {
            restore_previous_file(&test_version_path, previous_test_version.as_deref()).await?;
        }
        return Err(err);
    }

    if test_version_unchanged {
        touch_file(&test_version_path)?;
    }
    if condition_gene_unchanged {
        touch_file(&condition_gene_path)?;
    }

    Ok(())
}

fn validate_test_version_payload(body: &[u8]) -> Result<(), BioMcpError> {
    parse_test_version_records_from_gzip_bytes(body).map(|_| ())
}

fn validate_condition_gene_payload(body: &[u8]) -> Result<(), BioMcpError> {
    parse_condition_gene_links_bytes(body).map(|_| ())
}

fn read_test_version_records(path: &Path) -> Result<HashMap<String, GtrRecord>, BioMcpError> {
    let file = File::open(path).map_err(|err| BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!("Could not read {}: {err}", path.display()),
        suggestion: "Run `biomcp gtr sync` or preseed a complete GTR bundle.".to_string(),
    })?;
    parse_test_version_records(BufReader::new(file))
}

fn parse_test_version_records<R: Read>(
    reader: R,
) -> Result<HashMap<String, GtrRecord>, BioMcpError> {
    let decoder = GzDecoder::new(reader);
    parse_test_version_tsv(BufReader::new(decoder))
}

fn parse_test_version_records_from_gzip_bytes(
    body: &[u8],
) -> Result<HashMap<String, GtrRecord>, BioMcpError> {
    parse_test_version_records(body)
}

fn parse_test_version_tsv<R: Read>(reader: R) -> Result<HashMap<String, GtrRecord>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(reader);
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: GTR_API.to_string(),
        message: format!("Failed to read {GTR_TEST_VERSION_FILE} headers: {err}"),
    })?;
    let positions = header_positions(headers);
    for required in TEST_VERSION_REQUIRED_HEADERS {
        if !positions.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: GTR_API.to_string(),
                message: format!("{GTR_TEST_VERSION_FILE} is missing required column: {required}"),
            });
        }
    }

    let mut out = HashMap::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: GTR_API.to_string(),
            message: format!("Failed to parse {GTR_TEST_VERSION_FILE}: {err}"),
        })?;
        if field(&record, &positions, "now_current") != "1" {
            continue;
        }
        let accession = field(&record, &positions, "test_accession_ver");
        if accession.is_empty() {
            continue;
        }
        out.insert(
            accession.clone(),
            GtrRecord {
                accession,
                lab_test_name: field(&record, &positions, "lab_test_name"),
                manufacturer_test_name: field(&record, &positions, "manufacturer_test_name"),
                test_type: field(&record, &positions, "test_type"),
                name_of_laboratory: field(&record, &positions, "name_of_laboratory"),
                name_of_institution: field(&record, &positions, "name_of_institution"),
                clia_number: field(&record, &positions, "CLIA_number"),
                state_licenses: field(&record, &positions, "state_licenses"),
                facility_country: field(&record, &positions, "facility_country"),
                test_curr_stat: field(&record, &positions, "test_currStat"),
                test_pub_stat: field(&record, &positions, "test_pubStat"),
                method_categories: split_pipe_list(&field(
                    &record,
                    &positions,
                    "method_categories",
                )),
                methods: split_pipe_list(&field(&record, &positions, "methods")),
                genes: split_pipe_list(&field(&record, &positions, "genes")),
            },
        );
    }

    Ok(out)
}

fn read_condition_gene_links(path: &Path) -> Result<(LinkMap, LinkMap, LinkMap), BioMcpError> {
    let file = File::open(path).map_err(|err| BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!("Could not read {}: {err}", path.display()),
        suggestion: "Run `biomcp gtr sync` or preseed a complete GTR bundle.".to_string(),
    })?;
    parse_condition_gene_links(BufReader::new(file))
}

fn parse_condition_gene_links<R: Read>(
    reader: R,
) -> Result<(LinkMap, LinkMap, LinkMap), BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(reader);
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: GTR_API.to_string(),
        message: format!("Failed to read {GTR_CONDITION_GENE_FILE} headers: {err}"),
    })?;
    let positions = header_positions(headers);
    for required in CONDITION_GENE_REQUIRED_HEADERS {
        if !positions.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: GTR_API.to_string(),
                message: format!(
                    "{GTR_CONDITION_GENE_FILE} is missing required column: {required}"
                ),
            });
        }
    }

    let mut genes_by_id = HashMap::new();
    let mut conditions_by_id = HashMap::new();
    let mut test_types_by_id = HashMap::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: GTR_API.to_string(),
            message: format!("Failed to parse {GTR_CONDITION_GENE_FILE}: {err}"),
        })?;
        let accession = field(&record, &positions, "#accession_version");
        let test_type = if positions.contains_key("test_type") {
            field(&record, &positions, "test_type")
        } else {
            String::new()
        };
        let object = field(&record, &positions, "object").to_ascii_lowercase();
        let object_name = field(&record, &positions, "object_name");
        if accession.is_empty() || object_name.is_empty() {
            continue;
        }
        if !test_type.is_empty() {
            push_unique(&mut test_types_by_id, accession.clone(), test_type);
        }

        match object.as_str() {
            "gene" => push_unique(&mut genes_by_id, accession, object_name),
            "condition" => push_unique(&mut conditions_by_id, accession, object_name),
            _ => {}
        }
    }

    Ok((genes_by_id, conditions_by_id, test_types_by_id))
}

fn parse_condition_gene_links_bytes(
    body: &[u8],
) -> Result<(LinkMap, LinkMap, LinkMap), BioMcpError> {
    parse_condition_gene_links(body)
}

fn header_positions(headers: &csv::StringRecord) -> HashMap<String, usize> {
    headers
        .iter()
        .enumerate()
        .map(|(idx, value)| (value.trim_matches('\u{feff}').trim().to_string(), idx))
        .collect()
}

fn field(record: &csv::StringRecord, positions: &HashMap<String, usize>, header: &str) -> String {
    positions
        .get(header)
        .and_then(|idx| record.get(*idx))
        .and_then(clean_text)
        .unwrap_or_default()
}

fn clean_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn split_pipe_list(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for item in value.split('|').filter_map(clean_text) {
        let key = item.to_ascii_lowercase();
        if seen.insert(key) {
            out.push(item);
        }
    }
    out
}

fn push_unique(map: &mut HashMap<String, Vec<String>>, accession: String, value: String) {
    let entry = map.entry(accession).or_default();
    if entry
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&value))
    {
        return;
    }
    entry.push(value);
}

fn has_readable_local_pair(root: &Path) -> bool {
    gtr_missing_files(root).is_empty()
        && GTR_REQUIRED_FILES.iter().all(|file_name| {
            root.join(file_name).is_file() && File::open(root.join(file_name)).is_ok()
        })
}

async fn restore_previous_file(path: &Path, previous: Option<&[u8]>) -> Result<(), BioMcpError> {
    match previous {
        Some(content) => crate::utils::download::write_atomic_bytes(path, content).await,
        None => match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        },
    }
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
        .is_some_and(|age| age >= GTR_STALE_AFTER)
}

fn write_stderr_line(line: &str) -> Result<(), BioMcpError> {
    let mut stderr = std::io::stderr().lock();
    writeln!(stderr, "{line}")?;
    Ok(())
}

fn gtr_sync_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not prepare GTR data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: format!(
            "Retry with network access or run `biomcp gtr sync`. You can also preseed `{}` and `{}` into {} or set BIOMCP_GTR_DIR.",
            GTR_TEST_VERSION_FILE,
            GTR_CONDITION_GENE_FILE,
            root.display()
        ),
    }
}

fn gtr_read_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not read GTR data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: format!(
            "Run `biomcp gtr sync` or preseed `{}` and `{}` under {}.",
            GTR_TEST_VERSION_FILE,
            GTR_CONDITION_GENE_FILE,
            root.display()
        ),
    }
}

fn gtr_test_version_url() -> String {
    crate::sources::env_base(GTR_TEST_VERSION_URL, GTR_TEST_VERSION_URL_ENV).into_owned()
}

fn gtr_condition_gene_url() -> String {
    crate::sources::env_base(GTR_CONDITION_GENE_URL, GTR_CONDITION_GENE_URL_ENV).into_owned()
}

pub(crate) fn gtr_missing_files(root: &Path) -> Vec<String> {
    GTR_REQUIRED_FILES
        .iter()
        .filter(|file_name| !root.join(file_name).is_file())
        .map(|file_name| (*file_name).to_string())
        .collect()
}

pub(crate) fn resolve_gtr_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_GTR_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("gtr"),
        None => std::env::temp_dir().join("biomcp").join("gtr"),
    }
}

#[cfg(test)]
mod tests;
