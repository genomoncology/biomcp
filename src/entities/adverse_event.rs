use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::openfda::OpenFdaClient;
use crate::transform;
use crate::utils::date::validate_since;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEvent {
    pub report_id: String,
    pub drug: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patient: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concomitant_medications: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reporter_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indication: Option<String>,
    pub serious: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventSearchResult {
    pub report_id: String,
    pub drug: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<String>,
    pub serious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventReactionSummary {
    pub reaction: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventSearchSummary {
    pub total_reports: usize,
    pub returned_report_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_reactions: Vec<AdverseEventReactionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventSearchResponse {
    pub summary: AdverseEventSearchSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<AdverseEventSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    pub report_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_number: Option<String>,
    pub device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEventSearchResult {
    pub report_id: String,
    pub device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "kebab-case")]
pub enum AdverseEventReport {
    Faers(AdverseEvent),
    Device(DeviceEvent),
}

#[derive(Debug, Clone, Copy)]
pub enum AdverseEventQueryType {
    Faers,
    Recall,
    Device,
}

impl AdverseEventQueryType {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "faers" => Ok(Self::Faers),
            "recall" | "recalls" | "enforcement" => Ok(Self::Recall),
            "device" | "devices" | "maude" => Ok(Self::Device),
            _ => Err(BioMcpError::InvalidArgument(
                "--type must be one of: faers, recall, device".into(),
            )),
        }
    }
}

const ADVERSE_EVENT_SECTION_REACTIONS: &str = "reactions";
const ADVERSE_EVENT_SECTION_OUTCOMES: &str = "outcomes";
const ADVERSE_EVENT_SECTION_CONCOMITANT: &str = "concomitant";
const ADVERSE_EVENT_SECTION_GUIDANCE: &str = "guidance";
const ADVERSE_EVENT_SECTION_ALL: &str = "all";

pub const ADVERSE_EVENT_SECTION_NAMES: &[&str] = &[
    ADVERSE_EVENT_SECTION_REACTIONS,
    ADVERSE_EVENT_SECTION_OUTCOMES,
    ADVERSE_EVENT_SECTION_CONCOMITANT,
    ADVERSE_EVENT_SECTION_GUIDANCE,
    ADVERSE_EVENT_SECTION_ALL,
];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AdverseEventSections {
    pub include_reactions: bool,
    pub include_outcomes: bool,
    pub include_concomitant: bool,
    pub include_guidance: bool,
}

pub fn parse_sections(sections: &[String]) -> Result<AdverseEventSections, BioMcpError> {
    let mut out = AdverseEventSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() || section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            ADVERSE_EVENT_SECTION_REACTIONS => out.include_reactions = true,
            ADVERSE_EVENT_SECTION_OUTCOMES => out.include_outcomes = true,
            ADVERSE_EVENT_SECTION_CONCOMITANT => out.include_concomitant = true,
            ADVERSE_EVENT_SECTION_GUIDANCE => out.include_guidance = true,
            ADVERSE_EVENT_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for adverse-event. Available: {}",
                    ADVERSE_EVENT_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_reactions = true;
        out.include_outcomes = true;
        out.include_concomitant = true;
        out.include_guidance = true;
    }

    Ok(out)
}

#[derive(Debug, Clone, Default)]
pub struct AdverseEventSearchFilters {
    pub drug: Option<String>,
    pub reaction: Option<String>,
    pub outcome: Option<String>,
    pub serious: Option<String>,
    pub since: Option<String>,
    pub date_to: Option<String>,
    pub suspect_only: bool,
    pub sex: Option<String>,
    pub age_min: Option<u32>,
    pub age_max: Option<u32>,
    pub reporter: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RecallSearchFilters {
    pub drug: Option<String>,
    pub classification: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DeviceEventSearchFilters {
    pub device: Option<String>,
    pub manufacturer: Option<String>,
    pub product_code: Option<String>,
    pub serious: bool,
    pub since: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventCountBucket {
    pub value: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventCountResponse {
    pub count_field: String,
    #[serde(default)]
    pub buckets: Vec<AdverseEventCountBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallSearchResult {
    pub recall_number: String,
    pub classification: String,
    pub product_description: String,
    pub reason_for_recall: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution_pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recall_initiation_date: Option<String>,
}

fn yyyymmdd_from_date(value: &str, end_of_year: bool) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.len() == 4 && raw.chars().all(|c| c.is_ascii_digit()) {
        return Ok(if end_of_year {
            format!("{raw}1231")
        } else {
            format!("{raw}0101")
        });
    }

    let normalized = validate_since(raw)?;
    Ok(normalized.replace('-', ""))
}

fn serious_filter_term(raw: &str) -> Result<String, BioMcpError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "any" | "serious" => Ok("serious:1".to_string()),
        "death" => Ok("seriousnessdeath:1".to_string()),
        "hospitalization" | "hospitalisation" => Ok("seriousnesshospitalization:1".to_string()),
        "lifethreatening" | "life-threatening" => Ok("seriousnesslifethreatening:1".to_string()),
        "disability" | "disabling" => Ok("seriousnessdisabling:1".to_string()),
        "congenital" | "congenital_anomaly" => Ok("seriousnesscongenitalanomali:1".to_string()),
        "other" => Ok("seriousnessother:1".to_string()),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown --serious value '{other}'. Expected one of: death, hospitalization, lifethreatening, disability, congenital, other"
        ))),
    }
}

fn normalized_sex_filter(value: &str) -> Result<&'static str, BioMcpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "m" | "male" => Ok("1"),
        "f" | "female" => Ok("2"),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown --sex '{other}'. Expected one of: m, f"
        ))),
    }
}

fn normalized_reporter_filter(value: &str) -> Result<&'static str, BioMcpError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "physician" | "doctor" => Ok("1"),
        "pharmacist" => Ok("2"),
        "other_health_professional" | "health-professional" => Ok("3"),
        "lawyer" => Ok("4"),
        "consumer" | "patient" => Ok("5"),
        other => Err(BioMcpError::InvalidArgument(format!(
            "Unknown --reporter '{other}'. Expected one of: physician, pharmacist, other_health_professional, lawyer, consumer"
        ))),
    }
}

fn build_openfda_query(filters: &AdverseEventSearchFilters) -> Result<String, BioMcpError> {
    let drug = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "drug name is required. Example: biomcp search adverse-event -d pembrolizumab"
                    .into(),
            )
        })?;

    let mut terms: Vec<String> = Vec::new();
    let escaped_drug = OpenFdaClient::escape_query_value(drug);
    terms.push(format!(
        "(patient.drug.openfda.generic_name:\"{escaped_drug}\" OR patient.drug.openfda.brand_name:\"{escaped_drug}\" OR patient.drug.medicinalproduct:\"{escaped_drug}\")"
    ));
    if filters.suspect_only {
        terms.push("patient.drug.drugcharacterization:1".to_string());
    }

    if let Some(reaction) = filters
        .reaction
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(format!(
            "patient.reaction.reactionmeddrapt:\"{}\"",
            OpenFdaClient::escape_query_value(reaction)
        ));
    }

    if let Some(outcome) = filters
        .outcome
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let code = match outcome.to_ascii_lowercase().as_str() {
            "death" | "fatal" => "5",
            "hospitalization" | "hospitalisation" => "1",
            "disability" => "3",
            other => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown --outcome '{other}'. Expected one of: death, hospitalization, disability"
                )));
            }
        };
        terms.push(format!("patient.reaction.reactionoutcome:{code}"));
    }

    if let Some(serious) = filters
        .serious
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        terms.push(serious_filter_term(serious)?);
    }

    if let Some(since) = filters
        .since
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let yyyymmdd = yyyymmdd_from_date(since, false)?;
        terms.push(format!("receivedate:[{yyyymmdd} TO *]"));
    }
    if let Some(date_to) = filters
        .date_to
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let yyyymmdd = yyyymmdd_from_date(date_to, true)?;
        terms.push(format!("receivedate:[* TO {yyyymmdd}]"));
    }
    if let (Some(since), Some(date_to)) = (
        filters
            .since
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
        filters
            .date_to
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
    ) {
        let from = yyyymmdd_from_date(since, false)?;
        let to = yyyymmdd_from_date(date_to, true)?;
        if from > to {
            return Err(BioMcpError::InvalidArgument(
                "--date-from must be <= --date-to".into(),
            ));
        }
    }

    if let Some(sex) = filters
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let sex_code = normalized_sex_filter(sex)?;
        terms.push(format!("patient.patientsex:{sex_code}"));
    }

    if let Some(age_min) = filters.age_min {
        terms.push(format!("patient.patientonsetage:[{age_min} TO *]"));
    }
    if let Some(age_max) = filters.age_max {
        terms.push(format!("patient.patientonsetage:[* TO {age_max}]"));
    }
    if let (Some(age_min), Some(age_max)) = (filters.age_min, filters.age_max)
        && age_min > age_max
    {
        return Err(BioMcpError::InvalidArgument(
            "--age-min must be <= --age-max".into(),
        ));
    }

    if let Some(reporter) = filters
        .reporter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let reporter_code = normalized_reporter_filter(reporter)?;
        terms.push(format!("primarysource.qualification:{reporter_code}"));
    }

    Ok(terms.join(" AND "))
}

#[allow(dead_code)]
pub async fn search(
    filters: &AdverseEventSearchFilters,
    limit: usize,
) -> Result<Vec<AdverseEventSearchResult>, BioMcpError> {
    Ok(search_page(filters, limit, 0).await?.results)
}

pub async fn search_page(
    filters: &AdverseEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<AdverseEventSearchResult>, BioMcpError> {
    let response = search_with_summary(filters, limit, offset).await?;
    Ok(SearchPage::offset(
        response.results,
        Some(response.summary.total_reports),
    ))
}

fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub fn summarize_search_results(
    total_reports: usize,
    results: &[AdverseEventSearchResult],
) -> AdverseEventSearchSummary {
    let mut reaction_counts: HashMap<String, (String, usize)> = HashMap::new();
    for row in results {
        let mut seen_in_report: HashSet<String> = HashSet::new();
        for reaction in &row.reactions {
            let reaction = reaction.trim();
            if reaction.is_empty() {
                continue;
            }
            let key = reaction.to_ascii_lowercase();
            if !seen_in_report.insert(key.clone()) {
                continue;
            }
            let entry = reaction_counts
                .entry(key)
                .or_insert_with(|| (reaction.to_string(), 0usize));
            entry.1 += 1;
        }
    }

    let mut top_reactions = reaction_counts.into_values().collect::<Vec<_>>();
    top_reactions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    top_reactions.truncate(10);

    let returned_report_count = results.len();
    let denom = returned_report_count.max(1) as f64;
    let top_reactions = top_reactions
        .into_iter()
        .map(|(reaction, count)| AdverseEventReactionSummary {
            reaction,
            count,
            percentage: round_one_decimal((count as f64 * 100.0) / denom),
        })
        .collect::<Vec<_>>();

    AdverseEventSearchSummary {
        total_reports,
        returned_report_count,
        top_reactions,
    }
}

pub async fn search_with_summary(
    filters: &AdverseEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<AdverseEventSearchResponse, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let requested_drug = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "drug name is required. Example: biomcp search adverse-event -d pembrolizumab"
                    .into(),
            )
        })?;

    let q = build_openfda_query(filters)?;

    let client = OpenFdaClient::new()?;
    let resp = client.faers_search(&q, limit, offset).await?;
    let Some(resp) = resp else {
        return Ok(AdverseEventSearchResponse {
            summary: AdverseEventSearchSummary {
                total_reports: 0,
                returned_report_count: 0,
                top_reactions: Vec::new(),
            },
            results: Vec::new(),
        });
    };

    let total_reports = resp.meta.results.total;
    let results = resp
        .results
        .iter()
        .filter(|r| {
            if filters.suspect_only {
                transform::adverse_event::faers_report_matches_suspect_drug_query(r, requested_drug)
            } else {
                true
            }
        })
        .map(|r| {
            transform::adverse_event::from_openfda_faers_search_result(r, Some(requested_drug))
        })
        .collect::<Vec<_>>();

    Ok(AdverseEventSearchResponse {
        summary: summarize_search_results(total_reports, &results),
        results,
    })
}

pub async fn search_count(
    filters: &AdverseEventSearchFilters,
    count_field: &str,
    limit: usize,
) -> Result<AdverseEventCountResponse, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    let count_field = count_field.trim();
    if count_field.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--count requires a field name (example: --count patient.reaction.reactionmeddrapt)"
                .into(),
        ));
    }
    if count_field.len() > 128 {
        return Err(BioMcpError::InvalidArgument(
            "--count field is too long".into(),
        ));
    }

    let q = build_openfda_query(filters)?;
    let openfda_count_field = normalize_count_field_for_openfda(count_field);
    let client = OpenFdaClient::new()?;
    let resp = client.faers_count(&q, &openfda_count_field, limit).await?;
    let buckets = resp
        .map(|value| value.results)
        .unwrap_or_default()
        .into_iter()
        .map(|row| AdverseEventCountBucket {
            value: row.term,
            count: row.count,
        })
        .collect::<Vec<_>>();
    Ok(AdverseEventCountResponse {
        count_field: count_field.to_string(),
        buckets,
    })
}

fn normalize_count_field_for_openfda(count_field: &str) -> String {
    let field = count_field.trim();
    if field.eq_ignore_ascii_case("reaction")
        || field.eq_ignore_ascii_case("reactionmeddrapt")
        || field.eq_ignore_ascii_case("patient.reaction.reactionmeddrapt")
    {
        return "patient.reaction.reactionmeddrapt.exact".to_string();
    }
    field.to_string()
}

fn build_device_query(filters: &DeviceEventSearchFilters) -> Result<String, BioMcpError> {
    let device = filters
        .device
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let manufacturer = filters
        .manufacturer
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let product_code = filters
        .product_code
        .as_deref()
        .and_then(normalize_product_code);

    if device.is_none() && manufacturer.is_none() && product_code.is_none() {
        return Err(BioMcpError::InvalidArgument(
            "At least one device filter is required (--device, --manufacturer, or --product-code)."
                .into(),
        ));
    }

    let mut terms: Vec<String> = Vec::new();
    if let Some(device) = device {
        let escaped = OpenFdaClient::escape_query_value(device);
        let name_query = if device.chars().any(|c| c.is_whitespace()) {
            format!("device.brand_name:\"{escaped}\" OR device.generic_name:\"{escaped}\"")
        } else {
            format!("device.brand_name:*{escaped}* OR device.generic_name:*{escaped}*")
        };
        terms.push(format!("({name_query})"));
    }

    if let Some(manufacturer) = manufacturer {
        let escaped = OpenFdaClient::escape_query_value(manufacturer);
        let manufacturer_query = if manufacturer.chars().any(|c| c.is_whitespace()) {
            format!("manufacturer_name:\"{escaped}\" OR device.manufacturer_d_name:\"{escaped}\"")
        } else {
            format!("manufacturer_name:*{escaped}* OR device.manufacturer_d_name:*{escaped}*")
        };
        terms.push(format!("({manufacturer_query})"));
    }

    if let Some(product_code) = product_code {
        terms.push(format!(
            "device.device_report_product_code:\"{}\"",
            OpenFdaClient::escape_query_value(&product_code)
        ));
    }

    if filters.serious {
        terms.push("(event_type:\"Death\" OR event_type:\"Injury\")".to_string());
    }

    if let Some(since) = filters.since.as_deref() {
        let yyyymmdd = yyyymmdd_from_date(since, false)?;
        terms.push(format!("date_received:[{yyyymmdd} TO *]"));
    }

    Ok(terms.join(" AND "))
}

fn normalize_product_code(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

#[allow(dead_code)]
pub async fn search_device(
    filters: &DeviceEventSearchFilters,
    limit: usize,
) -> Result<Vec<DeviceEventSearchResult>, BioMcpError> {
    Ok(search_device_page(filters, limit, 0).await?.results)
}

pub async fn search_device_page(
    filters: &DeviceEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<DeviceEventSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let q = build_device_query(filters)?;

    let client = OpenFdaClient::new()?;
    let resp = client.device_event_search(&q, limit, offset).await?;
    let Some(resp) = resp else {
        return Ok(SearchPage::offset(Vec::new(), Some(0)));
    };

    Ok(SearchPage::offset(
        resp.results
            .iter()
            .map(transform::adverse_event::from_openfda_device_search_result)
            .collect(),
        Some(resp.meta.results.total),
    ))
}

fn normalize_classification(value: &str) -> Result<String, BioMcpError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--classification must be Class I, Class II, or Class III".into(),
        ));
    }

    let up = v.to_ascii_uppercase();
    let cleaned = up.strip_prefix("CLASS").unwrap_or(&up).trim();
    let cleaned = cleaned.trim_matches(|c: char| c == ':' || c.is_whitespace());
    match cleaned {
        "I" | "1" => Ok("Class I".into()),
        "II" | "2" => Ok("Class II".into()),
        "III" | "3" => Ok("Class III".into()),
        _ => Err(BioMcpError::InvalidArgument(
            "--classification must be Class I, Class II, or Class III".into(),
        )),
    }
}

fn build_enforcement_query(filters: &RecallSearchFilters) -> Result<String, BioMcpError> {
    let mut terms: Vec<String> = Vec::new();

    if let Some(drug) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let escaped = OpenFdaClient::escape_query_value(drug);
        if drug.chars().any(|c| c.is_whitespace()) {
            terms.push(format!("product_description:\"{escaped}\""));
        } else {
            terms.push(format!("product_description:*{escaped}*"));
        }
    }

    if let Some(classification) = filters
        .classification
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let norm = normalize_classification(classification)?;
        terms.push(format!("classification:\"{norm}\""));
    }

    if terms.is_empty() {
        // OpenFDA enforcement endpoint requires a non-empty search query.
        terms.push("recall_initiation_date:[20000101 TO *]".into());
    }

    Ok(terms.join(" AND "))
}

#[allow(dead_code)]
pub async fn search_recalls(
    filters: &RecallSearchFilters,
    limit: usize,
) -> Result<Vec<RecallSearchResult>, BioMcpError> {
    Ok(search_recalls_page(filters, limit, 0).await?.results)
}

pub async fn search_recalls_page(
    filters: &RecallSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<SearchPage<RecallSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }

    let q = build_enforcement_query(filters)?;
    let client = OpenFdaClient::new()?;
    let resp = client.enforcement_search(&q, limit, offset).await?;
    let Some(resp) = resp else {
        return Ok(SearchPage::offset(Vec::new(), Some(0)));
    };

    Ok(SearchPage::offset(
        resp.results
            .iter()
            .map(transform::adverse_event::from_openfda_enforcement_result)
            .collect(),
        Some(resp.meta.results.total),
    ))
}

async fn get_faers(report_id: &str) -> Result<Option<AdverseEvent>, BioMcpError> {
    let report_id = report_id.trim();
    if report_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Report ID is required. Example: biomcp get adverse-event 10222779".into(),
        ));
    }
    if report_id.len() > 64 || !report_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "Report ID must be numeric (FAERS safetyreportid).".into(),
        ));
    }

    let q = format!("safetyreportid:{report_id}");
    let client = OpenFdaClient::new()?;
    let resp = client.faers_search(&q, 1, 0).await?;
    let Some(resp) = resp else {
        return Ok(None);
    };

    let Some(first) = resp.results.into_iter().next() else {
        return Ok(None);
    };

    Ok(Some(
        transform::adverse_event::from_openfda_faers_get_result(&first),
    ))
}

async fn get_device(report_id: &str) -> Result<Option<DeviceEvent>, BioMcpError> {
    let report_id = report_id.trim();
    if report_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Report ID is required. Example: biomcp get adverse-event 10000000".into(),
        ));
    }
    if report_id.len() > 64 || !report_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "Report ID must be numeric (MAUDE mdr_report_key).".into(),
        ));
    }

    let q = format!("mdr_report_key:{report_id}");
    let client = OpenFdaClient::new()?;
    let resp = client.device_event_search(&q, 1, 0).await?;
    let Some(resp) = resp else {
        return Ok(None);
    };

    let Some(first) = resp.results.into_iter().next() else {
        return Ok(None);
    };

    Ok(Some(
        transform::adverse_event::from_openfda_device_get_result(&first),
    ))
}

pub async fn get(report_id: &str) -> Result<AdverseEventReport, BioMcpError> {
    let report_id = report_id.trim();
    if let Some(event) = get_faers(report_id).await? {
        return Ok(AdverseEventReport::Faers(event));
    }
    if let Some(event) = get_device(report_id).await? {
        return Ok(AdverseEventReport::Device(event));
    }
    Err(BioMcpError::NotFound {
        entity: "adverse-event".into(),
        id: report_id.to_string(),
        suggestion: format!(
            "Report ID {report_id} was not found. Try searching by drug or reaction: biomcp search adverse-event -d \"<drug-name>\""
        ),
    })
}

pub fn search_query_summary(filters: &AdverseEventSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(d) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("drug={d}"));
    }
    if let Some(r) = filters
        .reaction
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("reaction={r}"));
    }
    if let Some(v) = filters
        .outcome
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("outcome={v}"));
    }
    if let Some(serious) = filters
        .serious
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if serious.eq_ignore_ascii_case("any") {
            parts.push("serious=true".into());
        } else {
            parts.push(format!("serious={serious}"));
        }
    }
    if let Some(s) = filters
        .since
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("date_from={s}"));
    }
    if let Some(s) = filters
        .date_to
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("date_to={s}"));
    }
    if filters.suspect_only {
        parts.push("suspect_only=true".into());
    }
    if let Some(v) = filters
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("sex={v}"));
    }
    if let Some(v) = filters.age_min {
        parts.push(format!("age_min={v}"));
    }
    if let Some(v) = filters.age_max {
        parts.push(format!("age_max={v}"));
    }
    if let Some(v) = filters
        .reporter
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("reporter={v}"));
    }
    parts.join(", ")
}

pub fn device_query_summary(filters: &DeviceEventSearchFilters) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(d) = filters
        .device
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("device={d}"));
    }
    if let Some(m) = filters
        .manufacturer
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("manufacturer={m}"));
    }
    if let Some(code) = filters
        .product_code
        .as_deref()
        .and_then(normalize_product_code)
    {
        parts.push(format!("product_code={code}"));
    }
    if filters.serious {
        parts.push("serious=true".into());
    }
    if let Some(s) = filters
        .since
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("since={s}"));
    }
    parts.join(", ")
}

pub fn recall_query_summary(filters: &RecallSearchFilters) -> String {
    let mut parts: Vec<String> = vec!["Recalls".into()];
    if let Some(d) = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("drug={d}"));
    }
    if let Some(c) = filters
        .classification
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        parts.push(format!("classification={c}"));
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_openfda_query_requires_drug_name() {
        let err = build_openfda_query(&AdverseEventSearchFilters::default()).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn build_openfda_query_encodes_outcome_and_since() {
        let filters = AdverseEventSearchFilters {
            drug: Some("pembrolizumab".into()),
            reaction: Some("rash".into()),
            outcome: Some("death".into()),
            serious: Some("any".into()),
            since: Some("2024-01-01".into()),
            date_to: None,
            suspect_only: true,
            sex: None,
            age_min: None,
            age_max: None,
            reporter: None,
        };
        let q = build_openfda_query(&filters).unwrap();
        assert!(q.contains("generic_name"));
        assert!(q.contains("reactionoutcome:5"));
        assert!(q.contains("serious:1"));
        assert!(q.contains("receivedate:[20240101 TO *]"));
        assert!(q.contains("drugcharacterization:1"));
    }

    #[test]
    fn build_device_query_requires_any_device_filter() {
        let err = build_device_query(&DeviceEventSearchFilters::default()).unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn build_device_query_supports_manufacturer_and_product_code_filters() {
        let q = build_device_query(&DeviceEventSearchFilters {
            device: None,
            manufacturer: Some("Medtronic".into()),
            product_code: Some("pqp".into()),
            serious: true,
            since: Some("2024-01-01".into()),
        })
        .unwrap();
        assert!(q.contains("manufacturer_name"));
        assert!(q.contains("device.manufacturer_d_name"));
        assert!(q.contains("device.device_report_product_code:\"PQP\""));
        assert!(q.contains("(event_type:\"Death\" OR event_type:\"Injury\")"));
        assert!(q.contains("date_received:[20240101 TO *]"));
    }

    #[test]
    fn device_query_summary_includes_new_filters() {
        let summary = device_query_summary(&DeviceEventSearchFilters {
            device: None,
            manufacturer: Some("Medtronic".into()),
            product_code: Some("pqp".into()),
            serious: false,
            since: None,
        });
        assert_eq!(summary, "manufacturer=Medtronic, product_code=PQP");
    }

    #[test]
    fn normalize_classification_accepts_common_forms() {
        assert_eq!(normalize_classification("Class II").unwrap(), "Class II");
        assert_eq!(normalize_classification("2").unwrap(), "Class II");
        assert!(normalize_classification("Class IV").is_err());
    }

    #[test]
    fn build_enforcement_query_has_default_when_filters_empty() {
        let q = build_enforcement_query(&RecallSearchFilters::default()).unwrap();
        assert_eq!(q, "recall_initiation_date:[20000101 TO *]");
    }

    #[test]
    fn query_type_rejects_unknown_flag() {
        let err = AdverseEventQueryType::from_flag("foo").unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }

    #[test]
    fn summarize_search_results_computes_top_reactions() {
        let results = vec![
            AdverseEventSearchResult {
                report_id: "1".into(),
                drug: "pembrolizumab".into(),
                reactions: vec!["Rash".into(), "Fatigue".into()],
                serious: true,
            },
            AdverseEventSearchResult {
                report_id: "2".into(),
                drug: "pembrolizumab".into(),
                reactions: vec!["Rash".into()],
                serious: false,
            },
        ];

        let summary = summarize_search_results(200, &results);
        assert_eq!(summary.total_reports, 200);
        assert_eq!(summary.returned_report_count, 2);
        assert_eq!(
            summary.top_reactions.first().map(|v| v.reaction.as_str()),
            Some("Rash")
        );
        assert_eq!(
            summary.top_reactions.first().map(|v| v.percentage),
            Some(100.0)
        );
    }

    #[test]
    fn normalize_count_field_maps_reaction_alias_to_exact_keyword_field() {
        assert_eq!(
            normalize_count_field_for_openfda("patient.reaction.reactionmeddrapt"),
            "patient.reaction.reactionmeddrapt.exact"
        );
        assert_eq!(
            normalize_count_field_for_openfda("reaction"),
            "patient.reaction.reactionmeddrapt.exact"
        );
        assert_eq!(
            normalize_count_field_for_openfda("patient.drug.medicinalproduct"),
            "patient.drug.medicinalproduct"
        );
    }
}
