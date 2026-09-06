//! Bounded parsing and normalization for the GenCC new-format CSV export.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

const MAX_ROWS: usize = 100_000;
const MAX_FIELD_BYTES: usize = 16_384;
const MAX_LABEL_CHARS: usize = 1_024;
const MAX_URL_BYTES: usize = 2_048;
const MAX_PMIDS: usize = 128;

pub(crate) const HEADER: [&str; 31] = [
    "sgc_id",
    "version_number",
    "gene_curie",
    "gene_symbol",
    "disease_curie",
    "disease_title",
    "disease_original_curie",
    "disease_original_title",
    "classification_curie",
    "classification_title",
    "moi_curie",
    "moi_title",
    "submitter_curie",
    "submitter_title",
    "submitted_as_hgnc_id",
    "submitted_as_hgnc_symbol",
    "submitted_as_disease_id",
    "submitted_as_disease_name",
    "submitted_as_moi_id",
    "submitted_as_moi_name",
    "submitted_as_submitter_id",
    "submitted_as_submitter_name",
    "submitted_as_classification_id",
    "submitted_as_classification_name",
    "submitted_as_date",
    "submitted_as_public_report_url",
    "submitted_as_notes",
    "submitted_as_pmids",
    "submitted_as_assertion_criteria_url",
    "submitted_as_submission_id",
    "submitted_run_date",
];

#[derive(Debug, thiserror::Error)]
pub(crate) enum ParseError {
    #[error("invalid GenCC CSV")]
    Invalid,
    #[error("GenCC CSV row limit exceeded")]
    RowLimit,
    #[error("GenCC parsing cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenCcTerm {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenCcClassification {
    pub id: String,
    pub label: String,
    pub code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenCcPublication {
    pub pmid: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenCcAssertion {
    pub id: String,
    pub sgc_id: String,
    pub version: u32,
    pub gene: GenCcTerm,
    pub disease: GenCcTerm,
    pub classification: GenCcClassification,
    pub mode_of_inheritance: GenCcTerm,
    pub submitter: GenCcTerm,
    pub evaluated_date: Option<String>,
    pub submitted_date: Option<String>,
    pub source_record_url: String,
    pub public_report_url: Option<String>,
    pub assertion_criteria_url: Option<String>,
    pub publications: Vec<GenCcPublication>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GenCcDataset {
    assertions: Vec<GenCcAssertion>,
}

impl GenCcDataset {
    pub(crate) fn parse(bytes: &[u8], cancelled: &AtomicBool) -> Result<Self, ParseError> {
        let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(false)
            .from_reader(bytes);
        let headers = reader.headers().map_err(|_| ParseError::Invalid)?;
        if headers.len() != HEADER.len()
            || !headers
                .iter()
                .zip(HEADER)
                .all(|(actual, expected)| actual == expected)
        {
            return Err(ParseError::Invalid);
        }

        let mut rows = Vec::new();
        let mut duplicates: HashMap<(String, u32), usize> = HashMap::new();
        for (row_index, record) in reader.records().enumerate() {
            if cancelled.load(AtomicOrdering::Relaxed) {
                return Err(ParseError::Cancelled);
            }
            if row_index >= MAX_ROWS {
                return Err(ParseError::RowLimit);
            }
            let record = record.map_err(|_| ParseError::Invalid)?;
            if record.len() != HEADER.len() {
                return Err(ParseError::Invalid);
            }
            for field in &record {
                if field.len() > MAX_FIELD_BYTES || field.chars().any(char::is_control) {
                    return Err(ParseError::Invalid);
                }
            }
            let parsed = parse_record(&record)?;
            let key = (parsed.sgc_id.clone(), parsed.version);
            if let Some(existing) = duplicates.get(&key).copied() {
                if rows[existing] != parsed {
                    return Err(ParseError::Invalid);
                }
                continue;
            }
            duplicates.insert(key, rows.len());
            rows.push(parsed);
        }

        let greatest = rows
            .iter()
            .fold(HashMap::<String, u32>::new(), |mut versions, row| {
                versions
                    .entry(row.sgc_id.clone())
                    .and_modify(|version| *version = (*version).max(row.version))
                    .or_insert(row.version);
                versions
            });
        rows.retain(|row| greatest.get(row.sgc_id.as_str()) == Some(&row.version));
        rows.sort_by(assertion_order);
        Ok(Self { assertions: rows })
    }

    pub(crate) fn assertions(&self) -> &[GenCcAssertion] {
        &self.assertions
    }

    pub(crate) fn symbol_hgnc_ids(&self, symbol: &str) -> Vec<String> {
        let mut values = Vec::new();
        for row in &self.assertions {
            if row.gene.label.eq_ignore_ascii_case(symbol) && !values.contains(&row.gene.id) {
                values.push(row.gene.id.clone());
            }
        }
        values
    }

    pub(crate) fn matching(&self, symbol: &str, hgnc: &str) -> Vec<GenCcAssertion> {
        self.assertions
            .iter()
            .filter(|row| row.gene.label.eq_ignore_ascii_case(symbol) && row.gene.id == hgnc)
            .cloned()
            .collect()
    }
}

fn parse_record(row: &csv::StringRecord) -> Result<GenCcAssertion, ParseError> {
    let required = |index: usize| -> Result<&str, ParseError> {
        let value = row.get(index).ok_or(ParseError::Invalid)?.trim();
        if value.is_empty() || value.chars().count() > MAX_LABEL_CHARS {
            return Err(ParseError::Invalid);
        }
        Ok(value)
    };
    let sgc_id = required(0)?;
    if !sgc_id
        .strip_prefix("SGC-")
        .is_some_and(valid_positive_ascii_decimal)
    {
        return Err(ParseError::Invalid);
    }
    let version = parse_positive_u32(required(1)?)?;
    let gene_id = required(2)?;
    validate_curie(gene_id, "HGNC:", None)?;
    let gene_label = required(3)?;
    let disease_id = required(4)?;
    validate_curie(disease_id, "MONDO:", Some(7))?;
    let disease_label = required(5)?;
    let classification_id = required(8)?;
    let classification_label = required(9)?;
    let classification_code = classification(classification_id, classification_label)?;
    let inheritance_id = required(10)?;
    validate_curie(inheritance_id, "HP:", Some(7))?;
    let inheritance_label = required(11)?;
    let submitter_id = required(12)?;
    validate_curie(submitter_id, "GENCC:", Some(6))?;
    let submitter_label = required(13)?;
    let evaluated_date = parse_optional_date(row.get(24).ok_or(ParseError::Invalid)?)?;
    let public_report_url = parse_optional_url(row.get(25).ok_or(ParseError::Invalid)?);
    let publications = parse_pmids(row.get(27).ok_or(ParseError::Invalid)?)?;
    let assertion_criteria_url = parse_optional_url(row.get(28).ok_or(ParseError::Invalid)?);
    let submitted_date = parse_optional_date(row.get(30).ok_or(ParseError::Invalid)?)?;

    Ok(GenCcAssertion {
        id: format!("{sgc_id}.{version}"),
        sgc_id: sgc_id.to_string(),
        version,
        gene: GenCcTerm {
            id: gene_id.to_string(),
            label: gene_label.to_string(),
        },
        disease: GenCcTerm {
            id: disease_id.to_string(),
            label: disease_label.to_string(),
        },
        classification: GenCcClassification {
            id: classification_id.to_string(),
            label: classification_label.to_string(),
            code: classification_code.to_string(),
        },
        mode_of_inheritance: GenCcTerm {
            id: inheritance_id.to_string(),
            label: inheritance_label.to_string(),
        },
        submitter: GenCcTerm {
            id: submitter_id.to_string(),
            label: submitter_label.to_string(),
        },
        evaluated_date,
        submitted_date,
        source_record_url: format!("https://thegencc.org/submissions/{sgc_id}.{version}"),
        public_report_url,
        assertion_criteria_url,
        publications,
    })
}

fn valid_positive_ascii_decimal(value: &str) -> bool {
    !value.is_empty()
        && value.as_bytes()[0] != b'0'
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn parse_positive_u32(value: &str) -> Result<u32, ParseError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParseError::Invalid);
    }
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ParseError::Invalid)
}

fn validate_curie(value: &str, prefix: &str, width: Option<usize>) -> Result<(), ParseError> {
    let digits = value.strip_prefix(prefix).ok_or(ParseError::Invalid)?;
    if digits.is_empty()
        || width.is_some_and(|width| digits.len() != width)
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (prefix == "HGNC:" && !valid_positive_ascii_decimal(digits))
    {
        return Err(ParseError::Invalid);
    }
    Ok(())
}

fn classification(id: &str, label: &str) -> Result<&'static str, ParseError> {
    match (id, label) {
        ("GENCC:100001", "Definitive") => Ok("definitive"),
        ("GENCC:100002", "Strong") => Ok("strong"),
        ("GENCC:100003", "Moderate") => Ok("moderate"),
        ("GENCC:100004", "Limited") => Ok("limited"),
        ("GENCC:100005", "Disputed Evidence") => Ok("disputed_evidence"),
        ("GENCC:100006", "Refuted Evidence") => Ok("refuted_evidence"),
        ("GENCC:100007", "Animal Model Only") => Ok("animal_model_only"),
        ("GENCC:100008", "No Known Disease Relationship") => Ok("no_known_disease_relationship"),
        ("GENCC:100009", "Supportive") => Ok("supportive"),
        _ => Err(ParseError::Invalid),
    }
}

fn parse_optional_date(value: &str) -> Result<Option<String>, ParseError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    let date = if value.len() == 10 && value.is_ascii() {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| ParseError::Invalid)?
    } else if value.len() == 19 && value.is_ascii() && value.as_bytes()[10] == b' ' {
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .map_err(|_| ParseError::Invalid)?
            .date()
    } else {
        DateTime::parse_from_rfc3339(value)
            .map_err(|_| ParseError::Invalid)?
            .with_timezone(&Utc)
            .date_naive()
    };
    Ok(Some(date.format("%Y-%m-%d").to_string()))
}

fn parse_optional_url(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > MAX_URL_BYTES {
        return None;
    }
    let url = reqwest::Url::parse(value).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.has_host()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    Some(value.to_string())
}

fn parse_pmids(value: &str) -> Result<Vec<GenCcPublication>, ParseError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for token in value.split(',') {
        let token = token.trim();
        let digits = token
            .get(..5)
            .filter(|prefix| prefix.eq_ignore_ascii_case("PMID:"))
            .and_then(|_| token.get(5..))
            .unwrap_or(token);
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(ParseError::Invalid);
        }
        let pmid = digits
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or(ParseError::Invalid)?
            .to_string();
        if seen.insert(pmid.clone()) {
            if result.len() == MAX_PMIDS {
                return Err(ParseError::Invalid);
            }
            result.push(GenCcPublication {
                url: format!("https://pubmed.ncbi.nlm.nih.gov/{pmid}/"),
                pmid,
            });
        }
    }
    Ok(result)
}

fn assertion_order(left: &GenCcAssertion, right: &GenCcAssertion) -> Ordering {
    right
        .evaluated_date
        .cmp(&left.evaluated_date)
        .then_with(|| right.submitted_date.cmp(&left.submitted_date))
        .then_with(|| {
            left.submitter
                .label
                .to_ascii_lowercase()
                .cmp(&right.submitter.label.to_ascii_lowercase())
        })
        .then_with(|| left.submitter.label.cmp(&right.submitter.label))
        .then_with(|| left.disease.id.cmp(&right.disease.id))
        .then_with(|| {
            left.mode_of_inheritance
                .id
                .cmp(&right.mode_of_inheritance.id)
        })
        .then_with(|| left.sgc_id.cmp(&right.sgc_id))
        .then_with(|| right.version.cmp(&left.version))
}
