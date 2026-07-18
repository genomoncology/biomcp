use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::entities::disease::Disease;
use crate::error::BioMcpError;
use crate::sources::RequestBuilderSourceContextExt;
use crate::sources::{RequestPlan, request_from_plan};

const SEER_BASE: &str =
    "https://seer.cancer.gov/statistics-network/explorer/source/content_writers";
#[cfg(test)]
const SEER_API: &str = "seer";
const SEER_BASE_ENV: &str = "BIOMCP_SEER_BASE";
const SEER_SOURCE_NAME: &str = "SEER Explorer";
const SEER_SURVIVAL_SUGGESTION: &str = "Retry later: biomcp get disease <name_or_id> survival";
const ALL_RACES_CODE: u16 = 1;
const ALL_AGES_CODE: u16 = 1;
const RELATIVE_SURVIVAL_INTERVAL: u16 = 5;
const EXPECTED_KEY_ORDER: &[&str] = &[
    "relative_survival_interval",
    "sex",
    "race",
    "age_range",
    "site",
];
const EXPECTED_DATA_FIELDS: &[&str] = &[
    "year",
    "rel_rate",
    "rel_rate_se",
    "rel_rate_lower_ci",
    "rel_rate_upper_ci",
    "modeled_rel_rate",
    "count",
];
const CURATED_SITE_ALIASES: &[(&str, u16)] = &[
    ("cml", 97),
    ("chronic myeloid leukemia", 97),
    ("chronic myelogenous leukemia", 97),
    ("chronic myelocytic leukemia", 97),
    // "breast cancer" is the user-facing query term; MyDisease resolves it to
    // "breast carcinoma" (and synonyms below), so all three forms are listed.
    ("breast cancer", 55),
    ("breast carcinoma", 55),
    ("carcinoma of breast", 55),
    ("carcinoma of the breast", 55),
    ("hodgkin disease", 83),
];

#[derive(Clone)]
pub struct SeerClient {
    client: ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl SeerClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(SEER_BASE, SEER_BASE_ENV),
        })
    }

    pub(crate) fn site_catalog_plan() -> RequestPlan {
        RequestPlan::get("get_var_formats.php")
    }

    pub(crate) fn survival_plan(site_code: u16) -> RequestPlan {
        RequestPlan::get("render_region_5.php")
            .query("site", site_code.to_string())
            .query("data_type", "4")
            .query("graph_type", "1")
            .query("compareBy", "sex")
            .query(
                "relative_survival_interval",
                RELATIVE_SURVIVAL_INTERVAL.to_string(),
            )
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            return Err(seer_unavailable(format!(
                "SEER Explorer returned HTTP {status}."
            )));
        }

        crate::sources::ensure_json_content_type(
            crate::error::SourceContext::retry(crate::error::SourceProvider::SEER),
            content_type,
            bytes,
        )
        .map_err(|_| seer_unavailable("SEER Explorer returned an unexpected content type."))?;

        serde_json::from_slice(bytes)
            .map_err(|_| seer_unavailable("SEER Explorer returned data BioMCP could not decode."))
    }

    #[cfg(test)]
    pub(crate) fn decode_site_catalog_response(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<SeerSiteCatalog, BioMcpError> {
        let raw: RawSiteCatalog = Self::decode_json_response(status, content_type, bytes)?;
        SeerSiteCatalog::try_from(raw)
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEER,
            ))
            .await
            .map_err(remap_seer_error)?;
        let status = resp.status();
        let content_type = resp.headers().get(CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::SEER),
        )
        .await
        .map_err(remap_seer_error)?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEER,
            ))
        })
    }

    pub async fn site_catalog(&self) -> Result<SeerSiteCatalog, BioMcpError> {
        let plan = Self::site_catalog_plan();
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let raw: RawSiteCatalog = self.send_json(req).await?;
        SeerSiteCatalog::try_from(raw)
    }

    pub async fn fetch_survival(
        &self,
        site_code: u16,
        catalog: &SeerSiteCatalog,
    ) -> Result<SeerSurvivalPayload, BioMcpError> {
        if !catalog.is_active_site(site_code) {
            return Err(seer_unavailable(format!(
                "SEER Explorer site code {site_code} is not active in the live catalog."
            )));
        }

        let plan = Self::survival_plan(site_code);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let outer_json: String = self.send_json(req).await?;
        Self::decode_survival_payload(site_code, catalog, &outer_json).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEER,
            ))
        })
    }

    pub(crate) fn decode_survival_payload(
        site_code: u16,
        catalog: &SeerSiteCatalog,
        outer_json: &str,
    ) -> Result<SeerSurvivalPayload, BioMcpError> {
        let inner: RawSurvivalResponse = serde_json::from_str(outer_json).map_err(|_| {
            seer_unavailable("SEER Explorer returned data BioMCP could not double-decode.")
        })?;

        if inner.info.key_order.len() != EXPECTED_KEY_ORDER.len()
            || !inner
                .info
                .key_order
                .iter()
                .zip(EXPECTED_KEY_ORDER)
                .all(|(actual, expected)| actual == expected)
            || inner.info.data_fields.len() != EXPECTED_DATA_FIELDS.len()
            || !inner
                .info
                .data_fields
                .iter()
                .zip(EXPECTED_DATA_FIELDS)
                .all(|(actual, expected)| actual == expected)
        {
            return Err(seer_unavailable(
                "SEER Explorer returned an unexpected survival payload layout.",
            ));
        }

        let site_label = catalog
            .site_label(site_code)
            .ok_or_else(|| {
                seer_unavailable(format!(
                    "SEER Explorer site code {site_code} is missing from the live catalog."
                ))
            })?
            .to_string();

        let mut saw_requested_site = false;
        let mut by_sex: BTreeMap<u16, SeerSurvivalSeries> = BTreeMap::new();
        for (composite_key, bucket) in inner.data {
            let key = parse_composite_key(&composite_key)?;
            if key.site != site_code {
                continue;
            }
            saw_requested_site = true;
            if key.relative_survival_interval != RELATIVE_SURVIVAL_INTERVAL
                || key.race != ALL_RACES_CODE
                || key.age_range != ALL_AGES_CODE
            {
                continue;
            }

            let mut points = bucket
                .data_series
                .into_iter()
                .map(parse_data_series_row)
                .collect::<Result<Vec<_>, _>>()?;
            points.sort_by_key(|point| point.year);
            if points.is_empty() {
                continue;
            }

            let sex_label = catalog
                .sex_label(key.sex)
                .map(str::to_string)
                .unwrap_or_else(|| format!("Sex {}", key.sex));

            if by_sex
                .insert(
                    key.sex,
                    SeerSurvivalSeries {
                        sex_code: key.sex,
                        sex_label,
                        points,
                    },
                )
                .is_some()
            {
                return Err(seer_unavailable(
                    "SEER Explorer returned duplicate sex series for the requested site.",
                ));
            }
        }

        if !saw_requested_site {
            return Err(seer_unavailable(
                "SEER Explorer returned data for a different site than was requested.",
            ));
        }
        if by_sex.is_empty() {
            return Err(seer_unavailable(
                "SEER Explorer did not return all-ages and all-races survival rows for the requested site.",
            ));
        }

        Ok(SeerSurvivalPayload {
            site_code,
            site_label,
            series: by_sex.into_values().collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeerSiteCatalog {
    site_labels: HashMap<u16, String>,
    sex_labels: HashMap<u16, String>,
    race_labels: HashMap<u16, String>,
    age_range_labels: HashMap<u16, String>,
    active_sites: HashSet<u16>,
}

impl SeerSiteCatalog {
    pub fn is_active_site(&self, site_code: u16) -> bool {
        self.active_sites.contains(&site_code)
    }

    pub fn site_label(&self, site_code: u16) -> Option<&str> {
        self.site_labels.get(&site_code).map(String::as_str)
    }

    pub fn sex_label(&self, sex_code: u16) -> Option<&str> {
        self.sex_labels.get(&sex_code).map(String::as_str)
    }

    // dead-code reason: seer::race_label preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn race_label(&self, race_code: u16) -> Option<&str> {
        self.race_labels.get(&race_code).map(String::as_str)
    }

    // dead-code reason: seer::age_range_label preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn age_range_label(&self, age_range_code: u16) -> Option<&str> {
        self.age_range_labels
            .get(&age_range_code)
            .map(String::as_str)
    }
}

impl TryFrom<RawSiteCatalog> for SeerSiteCatalog {
    type Error = BioMcpError;

    fn try_from(value: RawSiteCatalog) -> Result<Self, Self::Error> {
        let site_labels = parse_labeled_code_map(value.variable_formats.site, "site")?;
        let sex_labels = parse_labeled_code_map(value.variable_formats.sex, "sex")?;
        let race_labels = parse_labeled_code_map(value.variable_formats.race, "race")?;
        let age_range_labels =
            parse_labeled_code_map(value.variable_formats.age_range, "age_range")?;
        let active_sites = value
            .cancer_sites
            .into_iter()
            .filter(|site| site.active)
            .map(|site| site.value)
            .collect::<HashSet<_>>();

        if site_labels.is_empty()
            || sex_labels.is_empty()
            || race_labels.is_empty()
            || age_range_labels.is_empty()
            || active_sites.is_empty()
        {
            return Err(seer_unavailable(
                "SEER Explorer returned an incomplete live site catalog.",
            ));
        }

        Ok(Self {
            site_labels,
            sex_labels,
            race_labels,
            age_range_labels,
            active_sites,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSeerSite {
    pub site_code: u16,
    pub site_label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeerSurvivalPayload {
    pub site_code: u16,
    pub site_label: String,
    pub series: Vec<SeerSurvivalSeries>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeerSurvivalSeries {
    pub sex_code: u16,
    pub sex_label: String,
    pub points: Vec<SeerSurvivalPoint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeerSurvivalPoint {
    pub year: u16,
    pub relative_survival_rate: Option<f64>,
    pub standard_error: Option<f64>,
    pub lower_ci: Option<f64>,
    pub upper_ci: Option<f64>,
    pub modeled_relative_survival_rate: Option<f64>,
    pub case_count: Option<u32>,
}

pub fn resolve_site(disease: &Disease, catalog: &SeerSiteCatalog) -> Option<ResolvedSeerSite> {
    let match_index = build_match_index(catalog);
    let mut matched_codes = HashSet::new();
    for candidate in disease_match_candidates(disease) {
        if let Some(codes) = match_index.get(candidate.as_str()) {
            matched_codes.extend(codes.iter().copied());
        }
    }

    if matched_codes.len() != 1 {
        return None;
    }

    let site_code = *matched_codes.iter().next()?;
    Some(ResolvedSeerSite {
        site_code,
        site_label: catalog.site_label(site_code)?.to_string(),
    })
}

fn disease_match_candidates(disease: &Disease) -> HashSet<String> {
    let mut out = HashSet::new();
    for raw in
        std::iter::once(disease.name.as_str()).chain(disease.synonyms.iter().map(|s| s.as_str()))
    {
        let normalized = normalize_match_term(raw);
        if !normalized.is_empty() {
            out.insert(normalized);
        }
    }
    out
}

fn build_match_index(catalog: &SeerSiteCatalog) -> HashMap<String, HashSet<u16>> {
    let mut out: HashMap<String, HashSet<u16>> = HashMap::new();

    for (&site_code, label) in &catalog.site_labels {
        if !catalog.is_active_site(site_code) {
            continue;
        }
        let normalized = normalize_match_term(label);
        if normalized.is_empty() {
            continue;
        }
        out.entry(normalized).or_default().insert(site_code);
    }

    for &(alias, site_code) in CURATED_SITE_ALIASES {
        if !catalog.is_active_site(site_code) {
            continue;
        }
        let normalized = normalize_match_term(alias);
        if normalized.is_empty() {
            continue;
        }
        out.entry(normalized).or_default().insert(site_code);
    }

    out
}

fn normalize_match_term(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_labeled_code_map(
    raw: HashMap<String, String>,
    field_name: &str,
) -> Result<HashMap<u16, String>, BioMcpError> {
    let mut out = HashMap::new();
    for (key, label) in raw {
        let code = key.parse::<u16>().map_err(|_| {
            seer_unavailable(format!(
                "SEER Explorer returned an invalid {field_name} code in the live catalog."
            ))
        })?;
        let label = label.trim();
        if label.is_empty() {
            return Err(seer_unavailable(format!(
                "SEER Explorer returned an empty {field_name} label in the live catalog."
            )));
        }
        out.insert(code, label.to_string());
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompositeKey {
    relative_survival_interval: u16,
    sex: u16,
    race: u16,
    age_range: u16,
    site: u16,
}

fn parse_composite_key(raw: &str) -> Result<CompositeKey, BioMcpError> {
    let mut parts = raw.split('_');
    let parse_part = |value: Option<&str>| -> Result<u16, BioMcpError> {
        value
            .ok_or_else(|| {
                seer_unavailable("SEER Explorer returned a malformed survival series key.")
            })?
            .parse::<u16>()
            .map_err(|_| {
                seer_unavailable("SEER Explorer returned a malformed survival series key.")
            })
    };

    let key = CompositeKey {
        relative_survival_interval: parse_part(parts.next())?,
        sex: parse_part(parts.next())?,
        race: parse_part(parts.next())?,
        age_range: parse_part(parts.next())?,
        site: parse_part(parts.next())?,
    };
    if parts.next().is_some() {
        return Err(seer_unavailable(
            "SEER Explorer returned a malformed survival series key.",
        ));
    }
    Ok(key)
}

fn parse_data_series_row(raw: Vec<Value>) -> Result<SeerSurvivalPoint, BioMcpError> {
    if raw.len() != EXPECTED_DATA_FIELDS.len() {
        return Err(seer_unavailable(
            "SEER Explorer returned a survival row with an unexpected field count.",
        ));
    }

    Ok(SeerSurvivalPoint {
        year: parse_required_u16(&raw[0], "year")?,
        relative_survival_rate: parse_optional_f64(&raw[1], "rel_rate")?,
        standard_error: parse_optional_f64(&raw[2], "rel_rate_se")?,
        lower_ci: parse_optional_f64(&raw[3], "rel_rate_lower_ci")?,
        upper_ci: parse_optional_f64(&raw[4], "rel_rate_upper_ci")?,
        modeled_relative_survival_rate: parse_optional_f64(&raw[5], "modeled_rel_rate")?,
        case_count: parse_optional_u32(&raw[6], "count")?,
    })
}

fn parse_required_u16(value: &Value, field_name: &str) -> Result<u16, BioMcpError> {
    parse_optional_u16(value, field_name)?.ok_or_else(|| {
        seer_unavailable(format!(
            "SEER Explorer returned a null {field_name} value in a survival row."
        ))
    })
}

fn parse_optional_u16(value: &Value, field_name: &str) -> Result<Option<u16>, BioMcpError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| {
                seer_unavailable(format!(
                    "SEER Explorer returned an invalid {field_name} value in a survival row."
                ))
            }),
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                return Ok(None);
            }
            text.parse::<u16>().map(Some).map_err(|_| {
                seer_unavailable(format!(
                    "SEER Explorer returned an invalid {field_name} value in a survival row."
                ))
            })
        }
        _ => Err(seer_unavailable(format!(
            "SEER Explorer returned an invalid {field_name} value in a survival row."
        ))),
    }
}

fn parse_optional_u32(value: &Value, field_name: &str) -> Result<Option<u32>, BioMcpError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .map(Some)
            .ok_or_else(|| {
                seer_unavailable(format!(
                    "SEER Explorer returned an invalid {field_name} value in a survival row."
                ))
            }),
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                return Ok(None);
            }
            text.parse::<u32>().map(Some).map_err(|_| {
                seer_unavailable(format!(
                    "SEER Explorer returned an invalid {field_name} value in a survival row."
                ))
            })
        }
        _ => Err(seer_unavailable(format!(
            "SEER Explorer returned an invalid {field_name} value in a survival row."
        ))),
    }
}

fn parse_optional_f64(value: &Value, field_name: &str) -> Result<Option<f64>, BioMcpError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => number.as_f64().map(Some).ok_or_else(|| {
            seer_unavailable(format!(
                "SEER Explorer returned an invalid {field_name} value in a survival row."
            ))
        }),
        Value::String(text) => {
            let text = text.trim();
            if text.is_empty() {
                return Ok(None);
            }
            text.parse::<f64>().map(Some).map_err(|_| {
                seer_unavailable(format!(
                    "SEER Explorer returned an invalid {field_name} value in a survival row."
                ))
            })
        }
        _ => Err(seer_unavailable(format!(
            "SEER Explorer returned an invalid {field_name} value in a survival row."
        ))),
    }
}

fn seer_unavailable(reason: impl Into<String>) -> BioMcpError {
    BioMcpError::SourceUnavailable {
        source_name: SEER_SOURCE_NAME.to_string(),
        reason: reason.into(),
        suggestion: SEER_SURVIVAL_SUGGESTION.to_string(),
    }
}

fn remap_seer_error(err: BioMcpError) -> BioMcpError {
    match err {
        BioMcpError::WithSourceContext { context, source } => {
            remap_seer_error(*source).with_source_context(context)
        }
        BioMcpError::SourceUnavailable { .. } => err,
        BioMcpError::Http(source) if source.is_timeout() || source.is_connect() => {
            seer_unavailable("SEER Explorer is temporarily unavailable.")
        }
        BioMcpError::Http(_) | BioMcpError::HttpMiddleware(_) => {
            seer_unavailable("SEER Explorer is temporarily unavailable.")
        }
        BioMcpError::Api { .. }
        | BioMcpError::ApiJson { .. }
        | BioMcpError::BodyLimit { .. }
        | BioMcpError::Json(_) => {
            seer_unavailable("SEER Explorer returned data BioMCP could not decode.")
        }
        other => other,
    }
}

#[derive(Debug, Deserialize)]
struct RawSiteCatalog {
    #[serde(rename = "VariableFormats")]
    variable_formats: RawVariableFormats,
    #[serde(rename = "CancerSites")]
    cancer_sites: Vec<RawCancerSite>,
}

#[derive(Debug, Deserialize)]
struct RawVariableFormats {
    site: HashMap<String, String>,
    sex: HashMap<String, String>,
    race: HashMap<String, String>,
    age_range: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawCancerSite {
    value: u16,
    active: bool,
}

#[derive(Debug, Deserialize)]
struct RawSurvivalResponse {
    info: RawSurvivalInfo,
    data: HashMap<String, RawSurvivalBucket>,
}

#[derive(Debug, Deserialize)]
struct RawSurvivalInfo {
    #[serde(rename = "key-order")]
    key_order: Vec<String>,
    #[serde(rename = "data-fields")]
    data_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawSurvivalBucket {
    data_series: Vec<Vec<Value>>,
}

#[cfg(test)]
mod tests;
