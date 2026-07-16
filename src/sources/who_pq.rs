use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use http_cache_reqwest::CacheMode;
use regex::Regex;

use crate::entities::SearchPage;
use crate::entities::drug::{
    WhoPrequalificationEntry, WhoPrequalificationKind, WhoPrequalificationSearchResult,
};
use crate::error::BioMcpError;

const SOURCE_NAME: &str = "WHO Prequalification";
const WHO_PQ_API: &str = "who-prequalification";
pub(crate) const WHO_PQ_EXPORT_URL: &str = "https://extranet.who.int/prequal/medicines/prequalified/finished-pharmaceutical-products/export?page&_format=csv";
pub(crate) const WHO_PQ_EXPORT_URL_ENV: &str = "BIOMCP_WHO_PQ_URL";
pub(crate) const WHO_PQ_CSV_FILE: &str = "who_pq.csv";
pub(crate) const WHO_PQ_API_EXPORT_URL: &str = "https://extranet.who.int/prequal/medicines/prequalified/active-pharmaceutical-ingredients/export?page&_format=csv";
pub(crate) const WHO_PQ_API_EXPORT_URL_ENV: &str = "BIOMCP_WHO_PQ_API_URL";
pub(crate) const WHO_PQ_API_CSV_FILE: &str = "who_api.csv";
pub(crate) const WHO_VACCINES_EXPORT_URL: &str =
    "https://extranet.who.int/prequal/vaccines/prequalified/export";
pub(crate) const WHO_VACCINES_EXPORT_URL_ENV: &str = "BIOMCP_WHO_VACCINES_URL";
pub(crate) const WHO_VACCINES_CSV_FILE: &str = "who_vaccines.csv";
pub(crate) const WHO_PQ_REQUIRED_FILES: &[&str] =
    &[WHO_PQ_CSV_FILE, WHO_PQ_API_CSV_FILE, WHO_VACCINES_CSV_FILE];
pub(crate) const WHO_PQ_STALE_AFTER: Duration = Duration::from_secs(72 * 60 * 60);
pub(crate) const WHO_PQ_SIZE_HINT: &str = "~134 KB";
pub(crate) const WHO_PQ_API_SIZE_HINT: &str = "~22 KB";
pub(crate) const WHO_VACCINES_SIZE_HINT: &str = "~47 KB";
const WHO_PQ_MAX_BODY_BYTES: usize = 4 * 1024 * 1024;
const WHO_PQ_API_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;
const WHO_VACCINES_MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

const REQUIRED_HEADERS: &[&str] = &[
    "WHO REFERENCE NUMBER",
    "INN, DOSAGE FORM AND STRENGTH",
    "PRODUCT TYPE",
    "THERAPEUTIC AREA",
    "APPLICANT",
    "DOSAGE FORM",
    "BASIS OF LISTING",
    "BASIS OF ALTERNATIVE LISTING",
    "DATE OF PREQUALIFICATION",
];

const API_REQUIRED_HEADERS: &[&str] = &[
    "WHO PRODUCT ID",
    "INN",
    "GRADE",
    "THERAPEUTIC AREA",
    "APPLICANT ORGANIZATION",
    "DATE OF PREQUALIFICATION",
    "CONFIRMATION OF PREQUALIFICATION DOCUMENT DATE",
];

const VACCINE_REQUIRED_HEADERS: &[&str] = &[
    "DATE OF PREQUALIFICATION",
    "VACCINE TYPE",
    "COMMERCIAL NAME",
    "PRESENTATION",
    "NO. OF DOSES",
    "MANUFACTURER",
    "RESPONSIBLE NRA",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhoPqSyncMode {
    Auto,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum WhoProductTypeFilter {
    #[default]
    Both,
    FinishedPharma,
    Api,
    Vaccine,
}

#[derive(Debug, Clone)]
pub(crate) struct WhoPqIdentity {
    terms: Vec<String>,
}

impl WhoPqIdentity {
    pub(crate) fn new(primary: &str) -> Self {
        Self::from_terms(vec![primary.to_string()])
    }

    pub(crate) fn with_aliases(primary: &str, canonical: Option<&str>, aliases: &[String]) -> Self {
        let mut terms = vec![primary.to_string()];
        if let Some(canonical) = canonical {
            terms.push(canonical.to_string());
        }
        terms.extend(aliases.iter().cloned());
        Self::from_terms(terms)
    }

    fn from_terms(terms: Vec<String>) -> Self {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for term in terms {
            let Some(term) = normalize_match_key(&term) else {
                continue;
            };
            if seen.insert(term.clone()) {
                out.push(term);
            }
        }
        Self { terms: out }
    }

    fn term_set(&self) -> HashSet<String> {
        self.terms.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct WhoPqClient {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncState {
    Fresh,
    Missing,
    Stale,
}

#[derive(Debug, Default)]
struct RawWhoPqRow {
    who_reference_number: String,
    presentation: String,
    product_type: String,
    therapeutic_area: String,
    applicant: String,
    dosage_form: String,
    listing_basis: String,
    alternative_listing_basis: String,
    prequalification_date: String,
}

#[derive(Debug, Default)]
struct RawWhoApiRow {
    who_product_id: String,
    inn: String,
    grade: String,
    therapeutic_area: String,
    applicant: String,
    prequalification_date: String,
    confirmation_document_date: String,
}

#[derive(Debug, Default)]
struct RawWhoVaccineRow {
    prequalification_date: String,
    vaccine_type: String,
    commercial_name: String,
    presentation: String,
    dose_count: String,
    manufacturer: String,
    responsible_nra: String,
}

impl WhoPqClient {
    #[allow(dead_code)]
    pub(crate) fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub(crate) async fn ready(mode: WhoPqSyncMode) -> Result<Self, BioMcpError> {
        let root = resolve_who_pq_root();
        sync_who_pq_root(&root, mode).await?;
        Ok(Self { root })
    }

    pub(crate) async fn sync(mode: WhoPqSyncMode) -> Result<(), BioMcpError> {
        let root = resolve_who_pq_root();
        sync_who_pq_root(&root, mode).await
    }

    pub(crate) fn regulatory(
        &self,
        identity: &WhoPqIdentity,
        product_type: WhoProductTypeFilter,
    ) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
        let rows = filter_rows_by_product_type(&self.read_rows()?, product_type);
        Ok(filter_regulatory_rows(&rows, identity))
    }

    pub(crate) fn search(
        &self,
        identity: &WhoPqIdentity,
        limit: usize,
        offset: usize,
        product_type: WhoProductTypeFilter,
    ) -> Result<SearchPage<WhoPrequalificationSearchResult>, BioMcpError> {
        const MAX_SEARCH_LIMIT: usize = 50;
        if limit == 0 || limit > MAX_SEARCH_LIMIT {
            return Err(BioMcpError::InvalidArgument(format!(
                "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
            )));
        }

        let rows = self.regulatory(identity, product_type)?;
        let total = rows.len();
        let results = rows
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|row| WhoPrequalificationSearchResult {
                kind: row.kind,
                inn: row.inn,
                product_type: row.product_type,
                therapeutic_area: row.therapeutic_area,
                presentation: row.presentation,
                dosage_form: row.dosage_form,
                applicant: row.applicant,
                who_reference_number: row.who_reference_number,
                who_product_id: row.who_product_id,
                listing_basis: row.listing_basis,
                prequalification_date: row.prequalification_date,
                vaccine_type: row.vaccine_type,
                commercial_name: row.commercial_name,
                dose_count: row.dose_count,
                manufacturer: row.manufacturer,
                responsible_nra: row.responsible_nra,
            })
            .collect();
        Ok(SearchPage::offset(results, Some(total)))
    }

    pub(crate) fn read_rows(&self) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
        let mut out = self.read_export_rows(WHO_PQ_CSV_FILE, parse_who_pq_csv)?;
        out.extend(self.read_export_rows(WHO_PQ_API_CSV_FILE, parse_who_api_csv)?);
        out.extend(self.read_export_rows(WHO_VACCINES_CSV_FILE, parse_who_vaccines_csv)?);
        Ok(out)
    }

    fn read_export_rows(
        &self,
        file_name: &str,
        parser: fn(&str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError>,
    ) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
        let path = self.root.join(file_name);
        let payload =
            std::fs::read_to_string(&path).map_err(|err| BioMcpError::SourceUnavailable {
                source_name: SOURCE_NAME.to_string(),
                reason: format!(
                    "Could not read WHO Prequalification CSV at {}: {err}",
                    path.display()
                ),
                suggestion: who_preseed_suggestion(&self.root),
            })?;
        parser(&payload)
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
        .to_ascii_uppercase()
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

fn clean_optional(value: &str) -> Option<String> {
    clean_text(value)
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

fn parse_who_pq_csv(payload: &str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(payload.as_bytes());
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: WHO_PQ_API.to_string(),
        message: format!("Failed to read WHO Prequalification headers: {err}"),
    })?;
    let header_map = header_map(headers);
    for required in REQUIRED_HEADERS {
        if !header_map.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: WHO_PQ_API.to_string(),
                message: format!("WHO Prequalification CSV is missing required column: {required}"),
            });
        }
    }

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: WHO_PQ_API.to_string(),
            message: format!("Failed to parse WHO Prequalification CSV: {err}"),
        })?;

        let row = RawWhoPqRow {
            who_reference_number: clean_csv_field(&record, &header_map, "WHO REFERENCE NUMBER"),
            presentation: clean_csv_field(&record, &header_map, "INN, DOSAGE FORM AND STRENGTH"),
            product_type: clean_csv_field(&record, &header_map, "PRODUCT TYPE"),
            therapeutic_area: clean_csv_field(&record, &header_map, "THERAPEUTIC AREA"),
            applicant: clean_csv_field(&record, &header_map, "APPLICANT"),
            dosage_form: clean_csv_field(&record, &header_map, "DOSAGE FORM"),
            listing_basis: clean_csv_field(&record, &header_map, "BASIS OF LISTING"),
            alternative_listing_basis: clean_csv_field(
                &record,
                &header_map,
                "BASIS OF ALTERNATIVE LISTING",
            ),
            prequalification_date: clean_csv_field(
                &record,
                &header_map,
                "DATE OF PREQUALIFICATION",
            ),
        };

        if row.who_reference_number.is_empty()
            || row.presentation.is_empty()
            || row.product_type.is_empty()
            || row.therapeutic_area.is_empty()
            || row.applicant.is_empty()
            || row.dosage_form.is_empty()
            || row.listing_basis.is_empty()
        {
            continue;
        }
        if !seen.insert(row.who_reference_number.clone()) {
            continue;
        }

        let inn = derive_inn(&row.presentation, &row.dosage_form);
        out.push(WhoPrequalificationEntry {
            kind: WhoPrequalificationKind::FinishedPharma,
            who_reference_number: Some(row.who_reference_number),
            inn,
            presentation: Some(row.presentation),
            dosage_form: Some(row.dosage_form),
            product_type: row.product_type,
            therapeutic_area: row.therapeutic_area,
            applicant: row.applicant,
            listing_basis: Some(row.listing_basis),
            alternative_listing_basis: clean_optional(&row.alternative_listing_basis),
            prequalification_date: normalize_who_date(&row.prequalification_date),
            who_product_id: None,
            grade: None,
            confirmation_document_date: None,
            vaccine_type: None,
            commercial_name: None,
            dose_count: None,
            manufacturer: None,
            responsible_nra: None,
        });
    }

    Ok(out)
}

fn parse_who_api_csv(payload: &str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(payload.as_bytes());
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: WHO_PQ_API.to_string(),
        message: format!("Failed to read WHO API headers: {err}"),
    })?;
    let header_map = header_map(headers);
    for required in API_REQUIRED_HEADERS {
        if !header_map.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: WHO_PQ_API.to_string(),
                message: format!("WHO API CSV is missing required column: {required}"),
            });
        }
    }

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: WHO_PQ_API.to_string(),
            message: format!("Failed to parse WHO API CSV: {err}"),
        })?;

        let row = RawWhoApiRow {
            who_product_id: clean_csv_field(&record, &header_map, "WHO PRODUCT ID"),
            inn: clean_csv_field(&record, &header_map, "INN"),
            grade: clean_csv_field(&record, &header_map, "GRADE"),
            therapeutic_area: clean_csv_field(&record, &header_map, "THERAPEUTIC AREA"),
            applicant: clean_csv_field(&record, &header_map, "APPLICANT ORGANIZATION"),
            prequalification_date: clean_csv_field(
                &record,
                &header_map,
                "DATE OF PREQUALIFICATION",
            ),
            confirmation_document_date: clean_csv_field(
                &record,
                &header_map,
                "CONFIRMATION OF PREQUALIFICATION DOCUMENT DATE",
            ),
        };

        if row.who_product_id.is_empty()
            || row.inn.is_empty()
            || row.therapeutic_area.is_empty()
            || row.applicant.is_empty()
        {
            continue;
        }
        if !seen.insert(row.who_product_id.clone()) {
            continue;
        }

        out.push(WhoPrequalificationEntry {
            kind: WhoPrequalificationKind::Api,
            who_reference_number: None,
            inn: row.inn,
            presentation: None,
            dosage_form: None,
            product_type: "Active Pharmaceutical Ingredient".to_string(),
            therapeutic_area: row.therapeutic_area,
            applicant: row.applicant,
            listing_basis: None,
            alternative_listing_basis: None,
            prequalification_date: normalize_who_date(&row.prequalification_date),
            who_product_id: Some(row.who_product_id),
            grade: clean_optional(&row.grade),
            confirmation_document_date: normalize_who_date(&row.confirmation_document_date),
            vaccine_type: None,
            commercial_name: None,
            dose_count: None,
            manufacturer: None,
            responsible_nra: None,
        });
    }

    Ok(out)
}

fn parse_who_vaccines_csv(payload: &str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(payload.as_bytes());
    let headers = reader.headers().map_err(|err| BioMcpError::Api {
        api: WHO_PQ_API.to_string(),
        message: format!("Failed to read WHO vaccine headers: {err}"),
    })?;
    let header_map = header_map(headers);
    for required in VACCINE_REQUIRED_HEADERS {
        if !header_map.contains_key(*required) {
            return Err(BioMcpError::Api {
                api: WHO_PQ_API.to_string(),
                message: format!("WHO vaccine CSV is missing required column: {required}"),
            });
        }
    }

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for record in reader.records() {
        let record = record.map_err(|err| BioMcpError::Api {
            api: WHO_PQ_API.to_string(),
            message: format!("Failed to parse WHO vaccine CSV: {err}"),
        })?;

        let row = RawWhoVaccineRow {
            prequalification_date: clean_csv_field(
                &record,
                &header_map,
                "DATE OF PREQUALIFICATION",
            ),
            vaccine_type: clean_csv_field(&record, &header_map, "VACCINE TYPE"),
            commercial_name: clean_csv_field(&record, &header_map, "COMMERCIAL NAME"),
            presentation: clean_csv_field(&record, &header_map, "PRESENTATION"),
            dose_count: clean_csv_field(&record, &header_map, "NO. OF DOSES"),
            manufacturer: clean_csv_field(&record, &header_map, "MANUFACTURER"),
            responsible_nra: clean_csv_field(&record, &header_map, "RESPONSIBLE NRA"),
        };

        if row.vaccine_type.is_empty() || row.manufacturer.is_empty() {
            continue;
        }

        let entry = WhoPrequalificationEntry {
            kind: WhoPrequalificationKind::Vaccine,
            who_reference_number: None,
            inn: row.vaccine_type.clone(),
            presentation: clean_optional(&row.presentation),
            dosage_form: None,
            product_type: "Vaccine".to_string(),
            therapeutic_area: "Vaccine".to_string(),
            applicant: row.manufacturer.clone(),
            listing_basis: None,
            alternative_listing_basis: None,
            prequalification_date: normalize_vaccine_date(&row.prequalification_date),
            who_product_id: None,
            grade: None,
            confirmation_document_date: None,
            vaccine_type: Some(row.vaccine_type),
            commercial_name: clean_optional(&row.commercial_name),
            dose_count: clean_optional(&row.dose_count),
            manufacturer: Some(row.manufacturer),
            responsible_nra: clean_optional(&row.responsible_nra),
        };

        if !seen.insert(entry.stable_identifier_key()) {
            continue;
        }

        out.push(entry);
    }

    Ok(out)
}

fn derive_inn(presentation: &str, dosage_form: &str) -> String {
    let presentation = clean_text(presentation).unwrap_or_default();
    let dosage_form = clean_text(dosage_form).unwrap_or_default();
    if presentation.is_empty() {
        return presentation;
    }
    if dosage_form.is_empty() {
        return presentation;
    }

    let presentation_lower = presentation.to_ascii_lowercase();
    let dosage_lower = dosage_form.to_ascii_lowercase();
    if let Some(idx) = presentation_lower.find(&dosage_lower) {
        let prefix = presentation[..idx]
            .trim()
            .trim_end_matches([',', '+', '/', ';', '-'])
            .trim();
        if !prefix.is_empty() {
            return prefix.to_string();
        }
    }

    presentation
}

fn normalize_who_date(value: &str) -> Option<String> {
    let value = clean_text(value)?;
    let parts = value
        .replace(',', " ")
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<u32>().ok()?;
    let month = match parts[1].to_ascii_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    let year = parts[2].parse::<u32>().ok()?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn normalize_vaccine_date(value: &str) -> Option<String> {
    let value = clean_text(value)?;
    let parts = value.split('/').map(str::trim).collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    let day = parts[0].parse::<u32>().ok()?;
    let month = parts[1].parse::<u32>().ok()?;
    let year = parts[2].parse::<u32>().ok()?;
    if day == 0 || day > 31 || month == 0 || month > 12 {
        return None;
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

fn normalize_match_key(value: &str) -> Option<String> {
    let mut value = clean_text(value)?.to_ascii_lowercase();
    value = parenthetical_salt_regex()
        .replace_all(&value, " ")
        .into_owned();
    value = punctuation_regex().replace_all(&value, " ").into_owned();
    value = slash_plus_regex().replace_all(&value, " + ").into_owned();
    let normalized = value
        .split(" + ")
        .filter_map(normalize_match_segment)
        .collect::<Vec<_>>();
    (!normalized.is_empty()).then(|| normalized.join(" + "))
}

fn normalize_match_segment(value: &str) -> Option<String> {
    let mut value = clean_text(value)?.to_ascii_lowercase();
    for suffix in salt_suffixes() {
        if value == *suffix {
            return None;
        }
        if let Some(stripped) = value.strip_suffix(&format!(" {suffix}")) {
            value = stripped.trim().to_string();
        }
    }
    clean_text(&value)
}

fn salt_suffixes() -> &'static [&'static str] {
    &[
        "acetate",
        "besylate",
        "diphosphate",
        "hydrochloride",
        "maleate",
        "mesylate",
        "phosphate",
        "sodium",
        "sulfate",
    ]
}

fn parenthetical_salt_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\((acetate|besylate|diphosphate|hydrochloride|maleate|mesylate|phosphate|sodium|sulfate)\)")
            .expect("regex should compile")
    })
}

fn punctuation_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"[,;]").expect("regex should compile"))
}

fn slash_plus_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s*(/|\+| and )\s*").expect("regex should compile"))
}

fn contains_boundary_phrase(field: &str, term: &str) -> bool {
    if field.is_empty() || term.is_empty() {
        return false;
    }

    let field_bytes = field.as_bytes();
    let mut search_from = 0usize;
    while let Some(pos) = field[search_from..].find(term) {
        let start = search_from + pos;
        let end = start + term.len();
        let before_ok = start == 0 || !field_bytes[start - 1].is_ascii_alphanumeric();
        let after_ok = end == field_bytes.len() || !field_bytes[end].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        search_from = start + 1;
    }
    false
}

fn entry_match_keys(row: &WhoPrequalificationEntry) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let values = if row.is_vaccine() {
        let mut values = vec![
            row.vaccine_type.as_deref(),
            vaccine_family_key(row),
            row.commercial_name.as_deref(),
            row.presentation.as_deref(),
        ];
        values.retain(|value| value.is_some());
        values
    } else {
        vec![Some(row.inn.as_str()), row.presentation.as_deref()]
    };
    for value in values.into_iter().flatten() {
        if let Some(value) = normalize_match_key(value)
            && seen.insert(value.clone())
        {
            out.push(value);
        }
    }
    out
}

fn vaccine_family_key(row: &WhoPrequalificationEntry) -> Option<&str> {
    row.vaccine_type
        .as_deref()
        .and_then(|value| value.split_once('(').map(|(prefix, _)| prefix.trim()))
        .filter(|value| !value.is_empty())
}

fn row_matches_identity(row: &WhoPrequalificationEntry, identity: &WhoPqIdentity) -> bool {
    let terms = identity.term_set();
    if terms.is_empty() {
        return false;
    }

    let match_keys = entry_match_keys(row);
    if match_keys.iter().any(|value| terms.contains(value)) {
        return true;
    }

    match_keys.iter().any(|field| {
        terms.iter().any(|term| {
            contains_boundary_phrase(field, term)
                || (row.is_vaccine() && contains_boundary_phrase(term, field))
        })
    })
}

pub(crate) fn filter_regulatory_rows(
    rows: &[WhoPrequalificationEntry],
    identity: &WhoPqIdentity,
) -> Vec<WhoPrequalificationEntry> {
    rows.iter()
        .filter(|row| row_matches_identity(row, identity))
        .cloned()
        .collect()
}

pub(crate) fn filter_rows_by_product_type(
    rows: &[WhoPrequalificationEntry],
    product_type: WhoProductTypeFilter,
) -> Vec<WhoPrequalificationEntry> {
    rows.iter()
        .filter(|row| match product_type {
            WhoProductTypeFilter::Both => matches!(
                row.kind,
                WhoPrequalificationKind::FinishedPharma | WhoPrequalificationKind::Api
            ),
            WhoProductTypeFilter::FinishedPharma => {
                matches!(row.kind, WhoPrequalificationKind::FinishedPharma)
            }
            WhoProductTypeFilter::Api => matches!(row.kind, WhoPrequalificationKind::Api),
            WhoProductTypeFilter::Vaccine => matches!(row.kind, WhoPrequalificationKind::Vaccine),
        })
        .cloned()
        .collect()
}

fn who_pq_export_url() -> String {
    std::env::var(WHO_PQ_EXPORT_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| WHO_PQ_EXPORT_URL.to_string())
}

fn who_pq_api_export_url() -> String {
    std::env::var(WHO_PQ_API_EXPORT_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| WHO_PQ_API_EXPORT_URL.to_string())
}

fn who_vaccines_export_url() -> String {
    std::env::var(WHO_VACCINES_EXPORT_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| WHO_VACCINES_EXPORT_URL.to_string())
}

fn who_preseed_suggestion(root: &Path) -> String {
    format!(
        "Run `biomcp who sync`, place {} from {}, {} from {}, and {} from {} in {}, or set BIOMCP_WHO_DIR.",
        WHO_PQ_CSV_FILE,
        who_pq_export_url(),
        WHO_PQ_API_CSV_FILE,
        who_pq_api_export_url(),
        WHO_VACCINES_CSV_FILE,
        who_vaccines_export_url(),
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
        .is_some_and(|age| age >= WHO_PQ_STALE_AFTER)
}

fn sync_state(root: &Path, mode: WhoPqSyncMode) -> SyncState {
    let missing = who_pq_missing_files(root, WHO_PQ_REQUIRED_FILES);
    if matches!(mode, WhoPqSyncMode::Force) {
        return if missing.len() == WHO_PQ_REQUIRED_FILES.len() {
            SyncState::Missing
        } else {
            SyncState::Stale
        };
    }
    if !missing.is_empty() {
        return SyncState::Missing;
    }
    if WHO_PQ_REQUIRED_FILES
        .iter()
        .any(|file_name| file_is_stale(&root.join(file_name)))
    {
        SyncState::Stale
    } else {
        SyncState::Fresh
    }
}

fn sync_intro(state: SyncState, mode: WhoPqSyncMode) -> &'static str {
    if matches!(mode, WhoPqSyncMode::Force) {
        return "Refreshing";
    }
    match state {
        SyncState::Fresh => "Checking",
        SyncState::Missing => "Downloading",
        SyncState::Stale => "Refreshing stale",
    }
}

fn who_pq_sync_error(root: &Path, detail: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SOURCE_NAME.to_string(),
        reason: format!(
            "Could not prepare WHO Prequalification data under {}. {}",
            root.display(),
            detail.into()
        ),
        suggestion: who_preseed_suggestion(root),
    }
}

async fn sync_who_pq_root(root: &Path, mode: WhoPqSyncMode) -> Result<(), BioMcpError> {
    let state = sync_state(root, mode);
    if matches!(state, SyncState::Fresh) {
        return Ok(());
    }

    tokio::fs::create_dir_all(root).await?;
    write_stderr_line(&format!(
        "{} WHO Prequalification data ({WHO_PQ_SIZE_HINT} + {WHO_PQ_API_SIZE_HINT} + {WHO_VACCINES_SIZE_HINT})...",
        sync_intro(state, mode)
    ))?;

    for (file_name, export_url, max_body_bytes, parser) in [
        (
            WHO_PQ_CSV_FILE,
            who_pq_export_url(),
            WHO_PQ_MAX_BODY_BYTES,
            parse_who_pq_csv as fn(&str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError>,
        ),
        (
            WHO_PQ_API_CSV_FILE,
            who_pq_api_export_url(),
            WHO_PQ_API_MAX_BODY_BYTES,
            parse_who_api_csv as fn(&str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError>,
        ),
        (
            WHO_VACCINES_CSV_FILE,
            who_vaccines_export_url(),
            WHO_VACCINES_MAX_BODY_BYTES,
            parse_who_vaccines_csv
                as fn(&str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError>,
        ),
    ] {
        let path = root.join(file_name);
        if let Err(err) =
            sync_export(root, file_name, &export_url, max_body_bytes, mode, parser).await
        {
            if has_readable_local_file(&path) {
                write_stderr_line(&format!(
                    "Warning: WHO Prequalification refresh failed for {}: {err}. Using existing data.",
                    file_name
                ))?;
            } else {
                return Err(who_pq_sync_error(root, err.to_string()));
            }
        }
    }

    let missing = who_pq_missing_files(root, WHO_PQ_REQUIRED_FILES);
    if missing.is_empty() {
        return Ok(());
    }

    Err(who_pq_sync_error(
        root,
        format!(
            "Missing required WHO Prequalification file(s): {}",
            missing.join(", ")
        ),
    ))
}

async fn sync_export(
    root: &Path,
    file_name: &str,
    export_url: &str,
    max_body_bytes: usize,
    mode: WhoPqSyncMode,
    parser: fn(&str) -> Result<Vec<WhoPrequalificationEntry>, BioMcpError>,
) -> Result<(), BioMcpError> {
    let client = crate::sources::shared_client()?;
    let mut request = client.get(export_url).with_extension(CacheMode::NoStore);
    if matches!(mode, WhoPqSyncMode::Force) {
        request = request.header(reqwest::header::CACHE_CONTROL, "no-cache");
    }
    let response = crate::sources::with_response_body_limit(request, max_body_bytes, WHO_PQ_API)
        .send()
        .await?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let body =
        crate::sources::read_limited_body_with_limit(response, WHO_PQ_API, max_body_bytes).await?;

    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: WHO_PQ_API.to_string(),
            message: format!(
                "{}: HTTP {status}: {}",
                file_name,
                crate::sources::body_excerpt(&body)
            ),
        });
    }

    ensure_csv_content_type(content_type.as_ref(), &body)?;
    let payload = std::str::from_utf8(&body).map_err(|source| BioMcpError::Api {
        api: WHO_PQ_API.to_string(),
        message: format!("{file_name} was not valid UTF-8: {source}"),
    })?;
    parser(payload)?;

    let path = root.join(file_name);
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
            api: WHO_PQ_API.to_string(),
            message: format!(
                "Unexpected HTML response (content-type: {raw}): {}",
                crate::sources::summarize_http_error_body(Some(header), body)
            ),
        });
    }
    Ok(())
}

pub(crate) fn who_pq_missing_files<'a>(root: &Path, files: &[&'a str]) -> Vec<&'a str> {
    files
        .iter()
        .filter(|file| !root.join(file).is_file())
        .copied()
        .collect()
}

pub(crate) fn resolve_who_pq_root() -> PathBuf {
    if let Some(path) = std::env::var("BIOMCP_WHO_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    match dirs::data_dir() {
        Some(path) => path.join("biomcp").join("who-pq"),
        None => std::env::temp_dir().join("biomcp").join("who-pq"),
    }
}

#[cfg(test)]
mod tests;
