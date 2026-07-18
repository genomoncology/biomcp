use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CLINGEN_BASE: &str = "https://search.clinicalgenome.org";
const CLINGEN_API: &str = "clingen";
const CLINGEN_BASE_ENV: &str = "BIOMCP_CLINGEN_BASE";
const CLINGEN_VALIDITY_PATH: &str = "kb/gene-validity/download";
const CLINGEN_DOSAGE_PATH: &str = "kb/gene-dosage/download";

pub struct ClinGenClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ClinGenClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CLINGEN_BASE, CLINGEN_BASE_ENV),
        })
    }

    pub(crate) fn gene_lookup_plan(gene_symbol: &str) -> Result<RequestPlan, BioMcpError> {
        let symbol = normalize_gene_symbol(gene_symbol)?;
        Ok(RequestPlan::get(format!("api/genes/look/{symbol}")))
    }

    pub(crate) fn validity_download_plan() -> RequestPlan {
        RequestPlan::get(CLINGEN_VALIDITY_PATH)
    }

    pub(crate) fn dosage_download_plan() -> RequestPlan {
        RequestPlan::get(CLINGEN_DOSAGE_PATH)
    }

    pub(crate) fn decode_text_response(
        api: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<String, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: api.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        api: &str,
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: api.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        // ClinGen's gene lookup endpoint can return JSON with a text/html content type.
        // Accept JSON-shaped payloads in that specific mismatch case.
        let allow_mislabeled_json = content_type
            .is_some_and(|header| is_html_content_type(header) && looks_like_json(bytes));
        if !allow_mislabeled_json {
            crate::sources::ensure_json_content_type(
                crate::error::SourceContext::retry(crate::error::SourceProvider::CLINGEN),
                content_type,
                bytes,
            )?;
        }

        serde_json::from_slice(bytes)
            .map_err(|source| BioMcpError::ApiJson {
                api: api.to_string(),
                source,
            })
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::CLINGEN,
                ))
            })
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
        api: &str,
    ) -> Result<String, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINGEN,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CLINGEN),
        )
        .await?;
        Self::decode_text_response(api, status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINGEN,
            ))
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        api: &str,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINGEN,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CLINGEN),
        )
        .await?;
        Self::decode_json_response(api, status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CLINGEN,
            ))
        })
    }

    pub async fn gene_context(&self, gene_symbol: &str) -> Result<GeneClinGen, BioMcpError> {
        let symbol = normalize_gene_symbol(gene_symbol)?;
        let hgnc_fut = self.lookup_hgnc_id(&symbol);
        let validity_fut = self.get_text(
            request_from_plan(
                &self.client,
                self.base.as_ref(),
                &Self::validity_download_plan(),
            ),
            CLINGEN_API,
        );
        let dosage_fut = self.get_text(
            request_from_plan(
                &self.client,
                self.base.as_ref(),
                &Self::dosage_download_plan(),
            ),
            CLINGEN_API,
        );
        let (hgnc_result, validity_csv, dosage_csv) =
            tokio::join!(hgnc_fut, validity_fut, dosage_fut);
        let hgnc_id = hgnc_result.unwrap_or_else(|err| {
            warn!(symbol = %symbol, "ClinGen gene lookup failed, falling back to symbol matching: {err}");
            None
        });
        let validity_csv = validity_csv?;
        let dosage_csv = dosage_csv?;
        let validity = parse_validity_csv(&validity_csv, &symbol, hgnc_id.as_deref())?;
        let (haploinsufficiency, triplosensitivity) =
            parse_dosage_csv(&dosage_csv, &symbol, hgnc_id.as_deref())?;

        Ok(GeneClinGen {
            validity,
            haploinsufficiency,
            triplosensitivity,
        })
    }

    async fn lookup_hgnc_id(&self, gene_symbol: &str) -> Result<Option<String>, BioMcpError> {
        let plan = Self::gene_lookup_plan(gene_symbol)?;
        let rows: Vec<ClinGenLookupGeneRow> = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                CLINGEN_API,
            )
            .await?;
        Ok(hgnc_id_from_lookup_rows(gene_symbol, &rows))
    }
}

fn hgnc_id_from_lookup_rows(gene_symbol: &str, rows: &[ClinGenLookupGeneRow]) -> Option<String> {
    if rows.is_empty() {
        return None;
    }

    let is_exact = |row: &ClinGenLookupGeneRow| {
        row.label
            .as_deref()
            .map(str::trim)
            .is_some_and(|label| label.eq_ignore_ascii_case(gene_symbol))
    };

    let pick = rows
        .iter()
        .find(|row| row.curated.unwrap_or(false) && is_exact(row))
        .or_else(|| rows.iter().find(|row| is_exact(row)))
        .or_else(|| rows.iter().find(|row| row.curated.unwrap_or(false)))
        .or_else(|| rows.first());

    pick.and_then(|row| clean_optional(row.hgnc.clone()))
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneClinGen {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validity: Vec<ClinGenValidity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub haploinsufficiency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub triplosensitivity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinGenValidity {
    pub disease: String,
    pub classification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moi: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ClinGenLookupGeneRow {
    label: Option<String>,
    hgnc: Option<String>,
    curated: Option<bool>,
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required for ClinGen".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn is_html_content_type(header: &reqwest::header::HeaderValue) -> bool {
    let Ok(raw) = header.to_str() else {
        return false;
    };
    let media_type = raw
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(media_type.as_str(), "text/html" | "application/xhtml+xml")
}

fn looks_like_json(body: &[u8]) -> bool {
    body.iter()
        .find(|b| !b.is_ascii_whitespace())
        .is_some_and(|b| matches!(*b, b'{' | b'['))
}

fn clean_field(record: &csv::StringRecord, headers: &HashMap<String, usize>, name: &str) -> String {
    headers
        .get(name)
        .and_then(|idx| record.get(*idx))
        .map(str::trim)
        .unwrap_or("")
        .to_string()
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

fn is_separator_row(record: &csv::StringRecord) -> bool {
    record
        .iter()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .all(|value| value.chars().all(|ch| ch == '+'))
}

fn header_map(record: &csv::StringRecord) -> HashMap<String, usize> {
    record
        .iter()
        .enumerate()
        .map(|(idx, col)| (normalize_header(col), idx))
        .collect()
}

fn matches_gene(symbol: &str, hgnc_id: Option<&str>, row_symbol: &str, row_hgnc: &str) -> bool {
    if let Some(hgnc_id) = hgnc_id
        && !hgnc_id.trim().is_empty()
        && row_hgnc.eq_ignore_ascii_case(hgnc_id.trim())
    {
        return true;
    }
    row_symbol.eq_ignore_ascii_case(symbol)
}

fn normalize_review_date(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() < 10 {
        return None;
    }
    let prefix = &value[..10];
    let bytes = prefix.as_bytes();
    let valid = bytes.len() == 10
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(|b| b.is_ascii_digit());
    valid.then(|| prefix.to_string())
}

fn parse_validity_csv(
    csv_payload: &str,
    symbol: &str,
    hgnc_id: Option<&str>,
) -> Result<Vec<ClinGenValidity>, BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_payload.as_bytes());

    let mut headers: Option<HashMap<String, usize>> = None;
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for row in reader.records() {
        let row = row.map_err(|err| BioMcpError::Api {
            api: CLINGEN_API.to_string(),
            message: format!("Failed to parse gene validity CSV: {err}"),
        })?;

        if row.iter().all(|value| value.trim().is_empty()) || is_separator_row(&row) {
            continue;
        }

        if headers.is_none() {
            let map = header_map(&row);
            if map.contains_key("GENE SYMBOL")
                && map.contains_key("DISEASE LABEL")
                && map.contains_key("CLASSIFICATION")
            {
                headers = Some(map);
            }
            continue;
        }

        let headers = headers.as_ref().expect("header map initialized");
        let row_symbol = clean_field(&row, headers, "GENE SYMBOL");
        let row_hgnc = clean_field(&row, headers, "GENE ID (HGNC)");
        if !matches_gene(symbol, hgnc_id, &row_symbol, &row_hgnc) {
            continue;
        }

        let disease = clean_field(&row, headers, "DISEASE LABEL");
        let classification = clean_field(&row, headers, "CLASSIFICATION");
        if disease.is_empty() || classification.is_empty() {
            continue;
        }

        let review_date = normalize_review_date(&clean_field(&row, headers, "CLASSIFICATION DATE"));
        let moi = clean_optional(Some(clean_field(&row, headers, "MOI")));
        let unique_key = format!(
            "{disease}|{classification}|{}",
            review_date.as_deref().unwrap_or("")
        );
        if !seen.insert(unique_key) {
            continue;
        }

        out.push(ClinGenValidity {
            disease,
            classification,
            review_date,
            moi,
        });
    }

    out.sort_by(|a, b| {
        b.review_date
            .cmp(&a.review_date)
            .then_with(|| a.disease.cmp(&b.disease))
            .then_with(|| a.classification.cmp(&b.classification))
    });
    out.truncate(5);
    Ok(out)
}

fn parse_dosage_csv(
    csv_payload: &str,
    symbol: &str,
    hgnc_id: Option<&str>,
) -> Result<(Option<String>, Option<String>), BioMcpError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(csv_payload.as_bytes());

    let mut headers: Option<HashMap<String, usize>> = None;
    let mut best: Option<(Option<String>, Option<String>, Option<String>)> = None;

    for row in reader.records() {
        let row = row.map_err(|err| BioMcpError::Api {
            api: CLINGEN_API.to_string(),
            message: format!("Failed to parse dosage CSV: {err}"),
        })?;

        if row.iter().all(|value| value.trim().is_empty()) || is_separator_row(&row) {
            continue;
        }

        if headers.is_none() {
            let map = header_map(&row);
            if map.contains_key("GENE SYMBOL")
                && map.contains_key("HGNC ID")
                && map.contains_key("HAPLOINSUFFICIENCY")
                && map.contains_key("TRIPLOSENSITIVITY")
            {
                headers = Some(map);
            }
            continue;
        }

        let headers = headers.as_ref().expect("header map initialized");
        let row_symbol = clean_field(&row, headers, "GENE SYMBOL");
        let row_hgnc = clean_field(&row, headers, "HGNC ID");
        if !matches_gene(symbol, hgnc_id, &row_symbol, &row_hgnc) {
            continue;
        }

        let haplo = clean_optional(Some(clean_field(&row, headers, "HAPLOINSUFFICIENCY")));
        let triplo = clean_optional(Some(clean_field(&row, headers, "TRIPLOSENSITIVITY")));
        if haplo.is_none() && triplo.is_none() {
            continue;
        }
        let date = normalize_review_date(&clean_field(&row, headers, "DATE"));

        let replace = match &best {
            None => true,
            Some((_, _, current_date)) => match (date.as_ref(), current_date.as_ref()) {
                (Some(new), Some(current)) => new.cmp(current) == Ordering::Greater,
                (Some(_), None) => true,
                _ => false,
            },
        };

        if replace {
            best = Some((haplo, triplo, date));
        }
    }

    Ok(best
        .map(|(haplo, triplo, _)| (haplo, triplo))
        .unwrap_or((None, None)))
}

#[cfg(test)]
mod tests;
