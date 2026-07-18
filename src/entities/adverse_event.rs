use std::collections::{HashMap, HashSet};

use crate::entities::SearchPage;
use crate::entities::drug::resolve_trial_aliases;
use crate::entities::section_outcome::{SectionOutcome, SectionOutcomes};
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::{
    CTGOV_ADVERSE_EVENT_SEARCH_FIELDS, ClinicalTrialsClient, CtGovAdverseEvent, CtGovSearchParams,
    CtGovStudy,
};
use crate::sources::cvx::{CvxClient, CvxSyncMode, CvxVaccineCandidate};
use crate::sources::openfda::{FaersEventResult, OpenFdaClient, OpenFdaResponse};
use crate::sources::vaers::VaersClient;
use crate::transform;
use crate::utils::date::validate_since;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub enum FaersSearchStatus {
    NotFound,
    Results(AdverseEventSearchResponse),
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialAdverseEventTerm {
    pub term: String,
    pub trial_count: usize,
}

#[derive(Debug, Clone)]
pub enum TrialAdverseEventOutcome {
    Found(Vec<TrialAdverseEventTerm>),
    Empty,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdverseEventSourceFilter {
    Faers,
    Vaers,
    All,
}

impl AdverseEventSourceFilter {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "all" => Ok(Self::All),
            "faers" => Ok(Self::Faers),
            "vaers" => Ok(Self::Vaers),
            other => Err(BioMcpError::InvalidArgument(format!(
                "--source must be one of: faers, vaers, all (got {other})"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaersMatchedVaccine {
    pub display_name: String,
    pub wonder_code: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cvx_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaersReactionCount {
    pub reaction: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaersAgeBucket {
    pub age_bucket: String,
    pub reports: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaersSearchSummary {
    pub total_reports: usize,
    pub serious_reports: usize,
    pub non_serious_reports: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub age_distribution: Vec<VaersAgeBucket>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub top_reactions: Vec<VaersReactionCount>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaersSearchStatus {
    Ok,
    Empty,
    QueryNotVaccine,
    UnsupportedFilters,
    UnmappedVaccine,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaersSearchPayload {
    pub status: VaersSearchStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_vaccine: Option<VaersMatchedVaccine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<VaersSearchSummary>,
}

#[derive(Debug, Clone)]
pub struct AdverseEventSourceSearch {
    pub section_outcomes: SectionOutcomes,
    pub source: AdverseEventSourceFilter,
    pub faers: Option<FaersSearchStatus>,
    pub vaers: Option<VaersSearchPayload>,
}

const ADVERSE_EVENT_OUTCOME_KEYS: &[&str] = &["faers", "vaers"];
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

const TRIAL_ADVERSE_EVENT_LIMIT: usize = 20;
const CTGOV_ADVERSE_EVENT_PAGE_SIZE: usize = 100;
const CTGOV_ADVERSE_EVENT_PAGE_CAP: usize = 20;
const VAERS_TOP_REACTION_LIMIT: usize = 10;

struct VaersBridgeEntry {
    display_name: &'static str,
    wonder_code: &'static str,
    cvx_codes: &'static [&'static str],
    query_terms: &'static [&'static str],
}

const VAERS_BRIDGE: &[VaersBridgeEntry] = &[
    VaersBridgeEntry {
        display_name: "MMR",
        wonder_code: "MMR",
        cvx_codes: &["03", "94"],
        query_terms: &[
            "mmr",
            "mmr vaccine",
            "measles mumps rubella",
            "measles mumps rubella vaccine",
            "measles mumps and rubella",
            "measles mumps and rubella vaccine",
        ],
    },
    VaersBridgeEntry {
        display_name: "COVID-19 vaccine",
        wonder_code: "COVID19",
        cvx_codes: &["208", "217"],
        query_terms: &[
            "covid 19 vaccine",
            "covid19 vaccine",
            "covid vaccine",
            "sars cov 2 vaccine",
            "comirnaty",
        ],
    },
    VaersBridgeEntry {
        display_name: "Influenza vaccine",
        wonder_code: "FLU",
        cvx_codes: &["140", "141"],
        query_terms: &[
            "flu",
            "flu shot",
            "flu vaccine",
            "influenza",
            "influenza shot",
            "influenza vaccine",
            "influenza vaccination",
        ],
    },
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

// dead-code reason: binary CLI dispatch uses this page-aware entity seam
#[allow(dead_code)]
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

pub fn summarize_count_buckets(
    total_reports: usize,
    returned_report_count: usize,
    buckets: &[AdverseEventCountBucket],
) -> AdverseEventSearchSummary {
    let denom = total_reports.max(1) as f64;
    let top_reactions = buckets
        .iter()
        .take(10)
        .map(|bucket| AdverseEventReactionSummary {
            reaction: bucket.value.clone(),
            count: bucket.count,
            percentage: round_one_decimal((bucket.count as f64 * 100.0) / denom),
        })
        .collect::<Vec<_>>();

    AdverseEventSearchSummary {
        total_reports,
        returned_report_count,
        top_reactions,
    }
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

fn empty_search_summary() -> AdverseEventSearchSummary {
    AdverseEventSearchSummary {
        total_reports: 0,
        returned_report_count: 0,
        top_reactions: Vec::new(),
    }
}

pub(crate) fn empty_search_response() -> AdverseEventSearchResponse {
    AdverseEventSearchResponse {
        summary: empty_search_summary(),
        results: Vec::new(),
    }
}

impl VaersSearchPayload {
    fn status_only(status: VaersSearchStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: Some(message.into()),
            matched_vaccine: None,
            summary: None,
        }
    }

    fn ok(matched_vaccine: VaersMatchedVaccine, summary: VaersSearchSummary) -> Self {
        Self {
            status: VaersSearchStatus::Ok,
            message: None,
            matched_vaccine: Some(matched_vaccine),
            summary: Some(summary),
        }
    }
}

enum ResolvedVaersVaccine {
    Matched(VaersMatchedVaccine),
    QueryNotVaccine(String),
    Unmapped(String),
}

#[derive(Debug, Clone, Copy)]
enum CvxLookupMode {
    AutoSync,
    LocalOnly,
}

fn unsupported_vaers_filter_names(filters: &AdverseEventSearchFilters) -> Vec<&'static str> {
    let mut out = Vec::new();
    if filters
        .reaction
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--reaction");
    }
    if filters
        .outcome
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--outcome");
    }
    if filters
        .serious
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--serious");
    }
    if filters
        .since
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--date-from");
    }
    if filters
        .date_to
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--date-to");
    }
    if filters.suspect_only {
        out.push("--suspect-only");
    }
    if filters
        .sex
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--sex");
    }
    if filters.age_min.is_some() {
        out.push("--age-min");
    }
    if filters.age_max.is_some() {
        out.push("--age-max");
    }
    if filters
        .reporter
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
    {
        out.push("--reporter");
    }
    out
}

fn normalize_vaccine_match_key(value: &str) -> Option<String> {
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
                normalized.push(mapped);
            }
            last_was_space = true;
        } else {
            normalized.push(mapped);
            last_was_space = false;
        }
    }
    let trimmed = normalized.trim().to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}

fn query_looks_like_vaccine(normalized_query: &str, candidates: &[CvxVaccineCandidate]) -> bool {
    !candidates.is_empty()
        || normalized_query == "mmr"
        || normalized_query.contains("vaccine")
        || normalized_query.contains("vaccination")
        || normalized_query.contains("booster")
        || normalized_query.contains("shot")
}

fn dedupe_cvx_codes(codes: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for code in codes {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn entry_matches_query(entry: &VaersBridgeEntry, normalized_query: &str) -> bool {
    entry.query_terms.contains(&normalized_query)
}

fn matched_vaccine_from_bridge(
    normalized_query: &str,
    candidates: &[CvxVaccineCandidate],
) -> Option<VaersMatchedVaccine> {
    for entry in VAERS_BRIDGE {
        let candidate_codes = candidates
            .iter()
            .filter(|candidate| entry.cvx_codes.contains(&candidate.cvx_code.as_str()))
            .map(|candidate| candidate.cvx_code.clone())
            .collect::<Vec<_>>();

        if entry_matches_query(entry, normalized_query) || !candidate_codes.is_empty() {
            let cvx_codes = if candidate_codes.is_empty() {
                entry
                    .cvx_codes
                    .iter()
                    .map(|code| (*code).to_string())
                    .collect::<Vec<_>>()
            } else {
                dedupe_cvx_codes(candidate_codes)
            };
            return Some(VaersMatchedVaccine {
                display_name: entry.display_name.to_string(),
                wonder_code: entry.wonder_code.to_string(),
                cvx_codes,
            });
        }
    }
    None
}

async fn lookup_vaccine_candidates(
    query: &str,
    mode: CvxLookupMode,
) -> Result<Vec<CvxVaccineCandidate>, BioMcpError> {
    match mode {
        CvxLookupMode::LocalOnly => CvxClient::new().lookup_vaccine_candidates(query),
        CvxLookupMode::AutoSync => CvxClient::ready(CvxSyncMode::Auto)
            .await?
            .lookup_vaccine_candidates(query),
    }
}

async fn resolve_vaers_vaccine(
    query: &str,
    cvx_lookup_mode: CvxLookupMode,
) -> Result<ResolvedVaersVaccine, BioMcpError> {
    let normalized_query = normalize_vaccine_match_key(query).unwrap_or_default();
    let candidates = lookup_vaccine_candidates(query, cvx_lookup_mode).await?;

    if let Some(matched) = matched_vaccine_from_bridge(&normalized_query, &candidates) {
        return Ok(ResolvedVaersVaccine::Matched(matched));
    }

    if query_looks_like_vaccine(&normalized_query, &candidates) {
        Ok(ResolvedVaersVaccine::Unmapped(
            "VAERS aggregate search does not yet map this vaccine family to a CDC WONDER code."
                .to_string(),
        ))
    } else {
        Ok(ResolvedVaersVaccine::QueryNotVaccine(
            "VAERS is vaccine-only; this query did not resolve to a vaccine identity.".to_string(),
        ))
    }
}

fn vaers_summary_from_tables(
    matched_vaccine: VaersMatchedVaccine,
    tables: crate::sources::vaers::VaersSummaryTables,
) -> VaersSearchPayload {
    let mut top_reactions = tables
        .reactions
        .into_iter()
        .map(|row| VaersReactionCount {
            reaction: row.label,
            count: row.count,
            percentage: row.percentage,
        })
        .collect::<Vec<_>>();
    top_reactions.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.reaction.cmp(&b.reaction))
    });
    top_reactions.truncate(VAERS_TOP_REACTION_LIMIT);

    let age_distribution = tables
        .age_distribution
        .into_iter()
        .map(|row| VaersAgeBucket {
            age_bucket: row.label,
            reports: row.count,
            percentage: row.percentage,
        })
        .collect::<Vec<_>>();

    let summary = VaersSearchSummary {
        total_reports: tables.total_reports,
        serious_reports: tables.serious_reports,
        non_serious_reports: tables.non_serious_reports,
        age_distribution,
        top_reactions,
    };

    if summary.total_reports == 0 {
        VaersSearchPayload {
            status: VaersSearchStatus::Empty,
            message: Some("CDC VAERS returned no aggregate reports for this vaccine query.".into()),
            matched_vaccine: Some(matched_vaccine),
            summary: Some(summary),
        }
    } else {
        VaersSearchPayload::ok(matched_vaccine, summary)
    }
}

async fn fetch_vaers_payload(
    query: &str,
    cvx_lookup_mode: CvxLookupMode,
) -> Result<VaersSearchPayload, BioMcpError> {
    match resolve_vaers_vaccine(query, cvx_lookup_mode).await? {
        ResolvedVaersVaccine::Matched(matched_vaccine) => {
            let client = VaersClient::new()?;
            let tables = client.summary(&matched_vaccine.wonder_code).await?;
            Ok(vaers_summary_from_tables(matched_vaccine, tables))
        }
        ResolvedVaersVaccine::QueryNotVaccine(message) => Ok(VaersSearchPayload::status_only(
            VaersSearchStatus::QueryNotVaccine,
            message,
        )),
        ResolvedVaersVaccine::Unmapped(message) => Ok(VaersSearchPayload::status_only(
            VaersSearchStatus::UnmappedVaccine,
            message,
        )),
    }
}

fn explicit_vaers_filter_error(unsupported: &[&str]) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "--source vaers only supports the vaccine query text in this ticket; unsupported flags: {}",
        unsupported.join(", ")
    ))
}

fn validate_explicit_vaers_source(
    filters: &AdverseEventSearchFilters,
    offset: usize,
) -> Result<(), BioMcpError> {
    if offset > 0 {
        return Err(BioMcpError::InvalidArgument(
            "--source vaers does not support --offset > 0".into(),
        ));
    }

    let unsupported = unsupported_vaers_filter_names(filters);
    if !unsupported.is_empty() {
        return Err(explicit_vaers_filter_error(&unsupported));
    }

    Ok(())
}

fn optional_faers_status(
    result: Result<FaersSearchStatus, BioMcpError>,
) -> Result<FaersSearchStatus, BioMcpError> {
    match result {
        Ok(status) => Ok(status),
        Err(err @ BioMcpError::InvalidArgument(_)) => Err(err),
        Err(_) => Ok(FaersSearchStatus::Unavailable),
    }
}

fn faers_section_outcome(status: &FaersSearchStatus) -> SectionOutcome {
    match status {
        FaersSearchStatus::Results(response) if !response.results.is_empty() => {
            SectionOutcome::data("OpenFDA FAERS")
        }
        FaersSearchStatus::NotFound | FaersSearchStatus::Results(_) => {
            SectionOutcome::empty("OpenFDA FAERS")
        }
        FaersSearchStatus::Unavailable => {
            SectionOutcome::unavailable("OpenFDA FAERS adverse events are unavailable.")
        }
    }
}

fn vaers_section_outcome(payload: &VaersSearchPayload) -> SectionOutcome {
    match payload.status {
        VaersSearchStatus::Ok => SectionOutcome::data_sources(["CDC CVX", "CDC VAERS"]),
        VaersSearchStatus::Empty => SectionOutcome::empty_sources(["CDC CVX", "CDC VAERS"]),
        VaersSearchStatus::QueryNotVaccine | VaersSearchStatus::UnmappedVaccine => {
            SectionOutcome::empty("CDC CVX")
        }
        VaersSearchStatus::UnsupportedFilters | VaersSearchStatus::Unavailable => {
            SectionOutcome::unavailable("CDC CVX/VAERS adverse events are unavailable.")
        }
    }
}

fn source_search(
    source: AdverseEventSourceFilter,
    faers: Option<FaersSearchStatus>,
    vaers: Option<VaersSearchPayload>,
) -> AdverseEventSourceSearch {
    let mut section_outcomes = SectionOutcomes::with_keys(ADVERSE_EVENT_OUTCOME_KEYS);
    if let Some(status) = &faers {
        section_outcomes.complete("faers", faers_section_outcome(status));
    }
    if let Some(payload) = &vaers
        && payload.status != VaersSearchStatus::UnsupportedFilters
    {
        section_outcomes.complete("vaers", vaers_section_outcome(payload));
    }
    AdverseEventSourceSearch {
        section_outcomes,
        source,
        faers,
        vaers,
    }
}

fn all_source_search_with_unsupported_vaers_filters(
    faers: FaersSearchStatus,
    unsupported: &[&str],
) -> AdverseEventSourceSearch {
    source_search(
        AdverseEventSourceFilter::All,
        Some(faers),
        Some(VaersSearchPayload::status_only(
            VaersSearchStatus::UnsupportedFilters,
            format!(
                "CDC VAERS skipped because these filters are unsupported for aggregate vaccine search: {}",
                unsupported.join(", ")
            ),
        )),
    )
}

fn all_source_search_with_vaers_payload(
    faers: FaersSearchStatus,
    vaers: VaersSearchPayload,
) -> AdverseEventSourceSearch {
    source_search(AdverseEventSourceFilter::All, Some(faers), Some(vaers))
}

pub async fn search_with_source(
    filters: &AdverseEventSearchFilters,
    source: AdverseEventSourceFilter,
    limit: usize,
    offset: usize,
) -> Result<AdverseEventSourceSearch, BioMcpError> {
    let query = filters
        .drug
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "drug name is required. Example: biomcp search adverse-event \"MMR vaccine\""
                    .into(),
            )
        })?;
    let unsupported = unsupported_vaers_filter_names(filters);

    match source {
        AdverseEventSourceFilter::Faers => Ok(source_search(
            source,
            Some(optional_faers_status(
                search_with_status(filters, limit, offset).await,
            )?),
            None,
        )),
        AdverseEventSourceFilter::Vaers => {
            validate_explicit_vaers_source(filters, offset)?;

            let vaers = fetch_vaers_payload(query, CvxLookupMode::AutoSync)
                .await
                .unwrap_or_else(|_| {
                    VaersSearchPayload::status_only(
                        VaersSearchStatus::Unavailable,
                        "CDC CVX/VAERS adverse events are unavailable.".to_string(),
                    )
                });
            Ok(source_search(source, None, Some(vaers)))
        }
        AdverseEventSourceFilter::All => {
            if unsupported.is_empty() {
                let (faers_result, vaers_result) = tokio::join!(
                    search_with_status(filters, limit, offset),
                    fetch_vaers_payload(query, CvxLookupMode::LocalOnly)
                );
                let vaers = match vaers_result {
                    Ok(payload) => payload,
                    Err(_) => VaersSearchPayload::status_only(
                        VaersSearchStatus::Unavailable,
                        "CDC CVX/VAERS adverse events are unavailable.".to_string(),
                    ),
                };
                Ok(all_source_search_with_vaers_payload(
                    optional_faers_status(faers_result)?,
                    vaers,
                ))
            } else {
                Ok(all_source_search_with_unsupported_vaers_filters(
                    optional_faers_status(search_with_status(filters, limit, offset).await)?,
                    &unsupported,
                ))
            }
        }
    }
}

async fn search_with_status_client(
    client: &OpenFdaClient,
    filters: &AdverseEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<FaersSearchStatus, BioMcpError> {
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
    let resp = client.faers_search(&q, limit, offset).await?;
    faers_status_from_openfda_response(resp, filters, requested_drug)
}

fn faers_status_from_openfda_response(
    resp: Option<OpenFdaResponse<FaersEventResult>>,
    filters: &AdverseEventSearchFilters,
    requested_drug: &str,
) -> Result<FaersSearchStatus, BioMcpError> {
    let Some(resp) = resp else {
        return Ok(FaersSearchStatus::NotFound);
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

    Ok(FaersSearchStatus::Results(AdverseEventSearchResponse {
        summary: summarize_search_results(total_reports, &results),
        results,
    }))
}

pub async fn search_with_status(
    filters: &AdverseEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<FaersSearchStatus, BioMcpError> {
    let client = OpenFdaClient::new()?;
    search_with_status_client(&client, filters, limit, offset).await
}

// dead-code reason: page-aware adverse-event search preserves report summary metadata
#[allow(dead_code)]
pub async fn search_with_summary(
    filters: &AdverseEventSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<AdverseEventSearchResponse, BioMcpError> {
    match search_with_status(filters, limit, offset).await? {
        FaersSearchStatus::NotFound => Ok(empty_search_response()),
        FaersSearchStatus::Results(response) => Ok(response),
        FaersSearchStatus::Unavailable => Err(BioMcpError::SourceUnavailable {
            source_name: "OpenFDA FAERS".into(),
            reason: "adverse-event search is unavailable".into(),
            suggestion: "Try again later.".into(),
        }),
    }
}

fn study_nct_id(study: &CtGovStudy) -> Option<&str> {
    study
        .protocol_section
        .as_ref()?
        .identification_module
        .as_ref()?
        .nct_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn ctgov_event_counts_for_study(event: &CtGovAdverseEvent) -> bool {
    if event.stats.is_empty() {
        return true;
    }
    event
        .stats
        .iter()
        .any(|stat| stat.num_affected.unwrap_or(0) > 0)
}

fn study_trial_adverse_event_term_count(study: &CtGovStudy) -> usize {
    let Some(module) = study
        .results_section
        .as_ref()
        .and_then(|section| section.adverse_events_module.as_ref())
    else {
        return 0;
    };

    let mut seen_terms = HashSet::new();
    for event in module
        .serious_events
        .iter()
        .chain(module.other_events.iter())
    {
        let Some(term) = event
            .term
            .as_deref()
            .map(str::trim)
            .filter(|term| !term.is_empty())
        else {
            continue;
        };
        if ctgov_event_counts_for_study(event) {
            seen_terms.insert(term.to_ascii_lowercase());
        }
    }
    seen_terms.len()
}

fn add_trial_terms(
    counts: &mut HashMap<String, (String, usize)>,
    seen_in_study: &mut HashSet<String>,
    events: &[CtGovAdverseEvent],
) {
    for event in events {
        let Some(term) = event
            .term
            .as_deref()
            .map(str::trim)
            .filter(|term| !term.is_empty())
        else {
            continue;
        };
        if !ctgov_event_counts_for_study(event) {
            continue;
        }
        let key = term.to_ascii_lowercase();
        if !seen_in_study.insert(key.clone()) {
            continue;
        }
        let entry = counts
            .entry(key)
            .or_insert_with(|| (term.to_string(), 0usize));
        entry.1 += 1;
    }
}

fn aggregate_trial_adverse_event_terms(
    studies: impl Iterator<Item = CtGovStudy>,
) -> Vec<TrialAdverseEventTerm> {
    let mut counts: HashMap<String, (String, usize)> = HashMap::new();

    for study in studies {
        let Some(module) = study
            .results_section
            .as_ref()
            .and_then(|section| section.adverse_events_module.as_ref())
        else {
            continue;
        };

        let mut seen_in_study = HashSet::new();
        add_trial_terms(&mut counts, &mut seen_in_study, &module.serious_events);
        add_trial_terms(&mut counts, &mut seen_in_study, &module.other_events);
    }

    let mut rows = counts
        .into_values()
        .map(|(term, trial_count)| TrialAdverseEventTerm { term, trial_count })
        .collect::<Vec<_>>();
    rows.sort_by(|a, b| {
        b.trial_count
            .cmp(&a.trial_count)
            .then_with(|| a.term.cmp(&b.term))
    });
    rows.truncate(TRIAL_ADVERSE_EVENT_LIMIT);
    rows
}

async fn fetch_ctgov_studies_for_alias(
    client: &ClinicalTrialsClient,
    alias: &str,
) -> Result<Vec<CtGovStudy>, BioMcpError> {
    let mut studies = Vec::new();
    let mut page_token = None;

    for _ in 0..CTGOV_ADVERSE_EVENT_PAGE_CAP {
        let response = client
            .search(&CtGovSearchParams {
                condition: None,
                intervention: Some(alias.to_string()),
                facility: None,
                status: None,
                agg_filters: Some("results:with".into()),
                query_term: None,
                fields_override: Some(CTGOV_ADVERSE_EVENT_SEARCH_FIELDS.into()),
                count_total: false,
                page_token: page_token.clone(),
                page_size: CTGOV_ADVERSE_EVENT_PAGE_SIZE,
                lat: None,
                lon: None,
                distance_miles: None,
            })
            .await?;

        studies.extend(response.studies);
        page_token = response.next_page_token;
        if page_token.is_none() {
            break;
        }
    }

    Ok(studies)
}

async fn trial_adverse_events_with_aliases(
    client: &ClinicalTrialsClient,
    aliases: &[String],
) -> Result<TrialAdverseEventOutcome, BioMcpError> {
    let mut batches = Vec::new();
    for alias in aliases
        .iter()
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        batches.push(fetch_ctgov_studies_for_alias(client, alias).await?);
    }

    Ok(trial_adverse_events_from_study_batches(batches))
}

fn trial_adverse_events_from_study_batches(
    batches: impl IntoIterator<Item = Vec<CtGovStudy>>,
) -> TrialAdverseEventOutcome {
    let mut studies_by_nct: HashMap<String, CtGovStudy> = HashMap::new();

    for studies in batches {
        for study in studies {
            let Some(nct_id) = study_nct_id(&study).map(str::to_string) else {
                continue;
            };
            let new_term_count = study_trial_adverse_event_term_count(&study);
            match studies_by_nct.entry(nct_id) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    if new_term_count > study_trial_adverse_event_term_count(entry.get()) {
                        entry.insert(study);
                    }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(study);
                }
            }
        }
    }

    let rows = aggregate_trial_adverse_event_terms(studies_by_nct.into_values());
    if rows.is_empty() {
        TrialAdverseEventOutcome::Empty
    } else {
        TrialAdverseEventOutcome::Found(rows)
    }
}

pub async fn trial_adverse_events(
    drug_name: &str,
) -> Result<TrialAdverseEventOutcome, BioMcpError> {
    let requested_name = drug_name.trim();
    if requested_name.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is required. Example: biomcp drug adverse-events pembrolizumab".into(),
        ));
    }

    let aliases = resolve_trial_aliases(requested_name).await?;
    let client = ClinicalTrialsClient::new()?;
    trial_adverse_events_with_aliases(&client, &aliases).await
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
    validate_count_field_for_openfda(count_field)?;

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

fn validate_count_field_for_openfda(count_field: &str) -> Result<(), BioMcpError> {
    let field = count_field.trim();
    let lower = field.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "total" | "meta.total" | "results.total" | "meta.results.total"
    ) {
        return Err(BioMcpError::InvalidArgument(
            "--count total is not a count field; pass an openFDA field key such as patient.reaction.reactionmeddrapt. The overall report total is meta.results.total, not a --count field.".into(),
        ));
    }
    if !is_plausible_openfda_count_field(field) {
        return Err(BioMcpError::InvalidArgument(
            "--count must be an openFDA field key such as patient.reaction.reactionmeddrapt".into(),
        ));
    }
    Ok(())
}

fn is_plausible_openfda_count_field(field: &str) -> bool {
    !field.starts_with('.')
        && !field.ends_with('.')
        && field
            .split('.')
            .all(|segment| !segment.is_empty() && segment.chars().all(is_openfda_field_char))
}

fn is_openfda_field_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
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

    fn ctgov_study(value: serde_json::Value) -> CtGovStudy {
        serde_json::from_value(value).expect("valid CTGov study")
    }

    fn vaers_summary_tables_fixture() -> crate::sources::vaers::VaersSummaryTables {
        use crate::sources::vaers::{VaersAggregateRow, VaersSummaryTables};

        VaersSummaryTables {
            total_reports: 83_359,
            serious_reports: 5_795,
            non_serious_reports: 77_564,
            reactions: vec![
                VaersAggregateRow {
                    label: "Injection site reaction".into(),
                    count: 12_000,
                    percentage: 14.4,
                },
                VaersAggregateRow {
                    label: "Fever".into(),
                    count: 10_500,
                    percentage: 12.6,
                },
            ],
            age_distribution: vec![VaersAggregateRow {
                label: "1-5 years".into(),
                count: 20_000,
                percentage: 24.0,
            }],
        }
    }

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
    fn source_filter_rejects_unknown_flag() {
        let err = AdverseEventSourceFilter::from_flag("foo").unwrap_err();
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

    #[test]
    fn validate_count_field_rejects_total_affordances() {
        for field in ["total", "results.total", "meta.results.total"] {
            let err = validate_count_field_for_openfda(field).expect_err("total is not countable");
            let message = err.to_string();
            assert!(message.contains("not a count field"));
            assert!(message.contains("patient.reaction.reactionmeddrapt"));
        }
    }

    #[tokio::test]
    async fn search_count_rejects_empty_count_field() {
        let filters = AdverseEventSearchFilters {
            drug: Some("pembrolizumab".into()),
            ..Default::default()
        };

        let err = search_count(&filters, "", 10)
            .await
            .expect_err("empty count field should be invalid");
        assert!(err.to_string().contains("--count requires a field name"));
    }

    #[test]
    fn validate_count_field_preserves_real_field_paths_and_aliases() {
        for field in [
            "reaction",
            "patient.reaction.reactionmeddrapt",
            "patient.reaction.reactionmeddrapt.exact",
            "patient.drug.medicinalproduct",
        ] {
            validate_count_field_for_openfda(field).expect("field should remain countable");
        }
    }

    #[test]
    fn validate_count_field_rejects_non_field_syntax() {
        let err =
            validate_count_field_for_openfda("patient reaction").expect_err("not a field key");
        assert!(err.to_string().contains("openFDA field key"));
    }

    #[test]
    fn search_with_status_preserves_openfda_not_found() {
        let filters = AdverseEventSearchFilters {
            drug: Some("daraxonrasib".into()),
            ..Default::default()
        };

        let status = faers_status_from_openfda_response(None, &filters, "daraxonrasib").unwrap();
        assert!(matches!(status, FaersSearchStatus::NotFound));
    }

    #[test]
    fn search_with_status_preserves_openfda_empty_results() {
        let filters = AdverseEventSearchFilters {
            drug: Some("faers-empty".into()),
            ..Default::default()
        };
        let resp: OpenFdaResponse<FaersEventResult> = serde_json::from_value(serde_json::json!({
            "meta": {"results": {"skip": 0, "limit": 5, "total": 0}},
            "results": []
        }))
        .expect("valid empty OpenFDA response");

        let status =
            faers_status_from_openfda_response(Some(resp), &filters, "faers-empty").unwrap();
        match status {
            FaersSearchStatus::NotFound => panic!("expected empty results, got not-found"),
            FaersSearchStatus::Results(response) => {
                assert_eq!(response.summary.total_reports, 0);
                assert!(response.results.is_empty());
            }
            FaersSearchStatus::Unavailable => panic!("expected empty results, got unavailable"),
        }
    }

    #[test]
    fn search_with_source_vaers_rejects_offset() {
        let filters = AdverseEventSearchFilters {
            drug: Some("MMR vaccine".into()),
            ..Default::default()
        };

        let err = validate_explicit_vaers_source(&filters, 1)
            .expect_err("vaers search should reject offset");
        assert!(
            err.to_string()
                .contains("--source vaers does not support --offset > 0")
        );
    }

    #[test]
    fn search_with_source_vaers_rejects_unsupported_filters() {
        let filters = AdverseEventSearchFilters {
            drug: Some("MMR vaccine".into()),
            reaction: Some("fever".into()),
            ..Default::default()
        };

        let err = validate_explicit_vaers_source(&filters, 0)
            .expect_err("vaers search should reject unsupported filters");
        assert!(err.to_string().contains("--reaction"));
    }

    #[tokio::test]
    async fn vaers_resolver_returns_query_not_vaccine_without_upstream_call() {
        let resolved = resolve_vaers_vaccine("ibuprofen", CvxLookupMode::LocalOnly).await;

        match resolved {
            Ok(ResolvedVaersVaccine::QueryNotVaccine(message)) => {
                assert!(message.contains("vaccine-only"));
            }
            _ => panic!("expected non-vaccine query"),
        }
    }

    #[test]
    fn optional_faers_status_preserves_invalid_arguments() {
        let result = optional_faers_status(Err(BioMcpError::InvalidArgument(
            "--limit must be between 1 and 50".into(),
        )));

        assert!(matches!(result, Err(BioMcpError::InvalidArgument(_))));
    }

    #[test]
    fn optional_faers_status_degrades_provider_failures() {
        let result = optional_faers_status(Err(BioMcpError::Api {
            api: "OpenFDA".into(),
            message: "service unavailable".into(),
        }))
        .expect("provider failure should become an in-band outcome");

        assert!(matches!(result, FaersSearchStatus::Unavailable));
    }

    #[test]
    fn successful_vaers_outcomes_credit_identity_and_evidence_sources() {
        for (status, expected) in [
            (
                VaersSearchStatus::Ok,
                crate::entities::section_outcome::SectionOutcomeState::Data,
            ),
            (
                VaersSearchStatus::Empty,
                crate::entities::section_outcome::SectionOutcomeState::Empty,
            ),
        ] {
            let outcome = vaers_section_outcome(&VaersSearchPayload::status_only(
                status,
                "fixture".to_string(),
            ));
            assert_eq!(outcome.outcome(), expected);
            assert!(outcome.sources().iter().any(|source| source == "CDC CVX"));
            assert!(outcome.sources().iter().any(|source| source == "CDC VAERS"));
        }
    }

    #[test]
    fn search_with_source_all_skips_vaers_for_unsupported_filters() {
        let response = all_source_search_with_unsupported_vaers_filters(
            FaersSearchStatus::NotFound,
            &["--reaction"],
        );
        let vaers = response.vaers.expect("vaers payload");

        assert_eq!(response.source, AdverseEventSourceFilter::All);
        assert!(matches!(response.faers, Some(FaersSearchStatus::NotFound)));
        assert_eq!(vaers.status, VaersSearchStatus::UnsupportedFilters);
        assert!(vaers.message.unwrap_or_default().contains("--reaction"));
        assert_eq!(
            response
                .section_outcomes
                .get("vaers")
                .expect("vaers outcome")
                .outcome(),
            crate::entities::section_outcome::SectionOutcomeState::NotRequested
        );
    }

    #[tokio::test]
    async fn search_with_source_all_non_vaccine_uses_local_only_vaers_result() {
        let vaers = match resolve_vaers_vaccine("ibuprofen", CvxLookupMode::LocalOnly).await {
            Ok(ResolvedVaersVaccine::QueryNotVaccine(message)) => {
                VaersSearchPayload::status_only(VaersSearchStatus::QueryNotVaccine, message)
            }
            _ => panic!("expected non-vaccine query"),
        };
        let response = all_source_search_with_vaers_payload(FaersSearchStatus::NotFound, vaers);
        let vaers = response.vaers.expect("vaers payload");

        assert_eq!(response.source, AdverseEventSourceFilter::All);
        assert!(matches!(response.faers, Some(FaersSearchStatus::NotFound)));
        assert_eq!(vaers.status, VaersSearchStatus::QueryNotVaccine);
        assert!(vaers.message.unwrap_or_default().contains("vaccine-only"));
    }

    #[tokio::test]
    async fn vaers_resolver_matches_influenza_family_queries() {
        let matched =
            match resolve_vaers_vaccine("influenza vaccine", CvxLookupMode::LocalOnly).await {
                Ok(ResolvedVaersVaccine::Matched(matched)) => matched,
                _ => panic!("expected influenza vaccine match"),
            };

        assert_eq!(matched.display_name, "Influenza vaccine");
        assert_eq!(matched.wonder_code, "FLU");
        assert_eq!(
            matched.cvx_codes,
            vec!["140".to_string(), "141".to_string()]
        );
    }

    #[test]
    fn vaers_summary_payload_maps_mmr_tables() {
        let vaers = vaers_summary_from_tables(
            VaersMatchedVaccine {
                display_name: "MMR".into(),
                wonder_code: "MMR".into(),
                cvx_codes: vec!["03".into(), "94".into()],
            },
            vaers_summary_tables_fixture(),
        );
        let summary = vaers.summary.expect("vaers summary");
        let matched = vaers.matched_vaccine.expect("matched vaccine");

        assert_eq!(vaers.status, VaersSearchStatus::Ok);
        assert_eq!(matched.display_name, "MMR");
        assert_eq!(matched.wonder_code, "MMR");
        assert_eq!(matched.cvx_codes, vec!["03".to_string(), "94".to_string()]);
        assert_eq!(summary.total_reports, 83_359);
        assert_eq!(summary.serious_reports, 5_795);
        assert_eq!(summary.non_serious_reports, 77_564);
        assert!(!summary.age_distribution.is_empty());
        assert!(!summary.top_reactions.is_empty());
        assert!(summary.top_reactions.iter().all(|row| row.count > 0));
        assert!(summary.top_reactions.len() <= 10);
    }

    #[test]
    fn trial_adverse_events_dedupe_studies_across_aliases() {
        let study = ctgov_study(serde_json::json!({
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT05379985",
                    "briefTitle": "Daraxonrasib first-in-human study"
                }
            },
            "hasResults": true,
            "resultsSection": {
                "adverseEventsModule": {
                    "seriousEvents": [{
                        "term": "Rash",
                        "stats": [{"groupId": "g1", "numAffected": 2, "numAtRisk": 10}]
                    }],
                    "otherEvents": [{
                        "term": "Fatigue",
                        "stats": [{"groupId": "g1", "numAffected": 1, "numAtRisk": 10}]
                    }]
                }
            }
        }));

        let outcome =
            trial_adverse_events_from_study_batches(vec![vec![study.clone()], vec![study]]);

        match outcome {
            TrialAdverseEventOutcome::Empty => panic!("expected trial adverse events"),
            TrialAdverseEventOutcome::Found(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].term, "Fatigue");
                assert_eq!(rows[0].trial_count, 1);
                assert_eq!(rows[1].term, "Rash");
                assert_eq!(rows[1].trial_count, 1);
            }
        }
    }

    #[test]
    fn trial_adverse_events_count_each_term_once_per_study() {
        let outcome = trial_adverse_events_from_study_batches(vec![vec![
            ctgov_study(serde_json::json!({
                "protocolSection": {
                    "identificationModule": {
                        "nctId": "NCT05379985",
                        "briefTitle": "Daraxonrasib first-in-human study"
                    }
                },
                "hasResults": true,
                "resultsSection": {
                    "adverseEventsModule": {
                        "seriousEvents": [{
                            "term": "Rash",
                            "stats": [{"groupId": "g1", "numAffected": 2, "numAtRisk": 10}]
                        }],
                        "otherEvents": [{
                            "term": "Rash",
                            "stats": [{"groupId": "g2", "numAffected": 4, "numAtRisk": 10}]
                        }]
                    }
                }
            })),
            ctgov_study(serde_json::json!({
                "protocolSection": {
                    "identificationModule": {
                        "nctId": "NCT00000002",
                        "briefTitle": "Daraxonrasib expansion cohort"
                    }
                },
                "hasResults": true,
                "resultsSection": {
                    "adverseEventsModule": {
                        "seriousEvents": [],
                        "otherEvents": [{
                            "term": "Rash",
                            "stats": [{"groupId": "g3", "numAffected": 1, "numAtRisk": 12}]
                        }]
                    }
                }
            })),
        ]]);

        match outcome {
            TrialAdverseEventOutcome::Empty => panic!("expected trial adverse events"),
            TrialAdverseEventOutcome::Found(rows) => {
                assert_eq!(rows[0].term, "Rash");
                assert_eq!(rows[0].trial_count, 2);
            }
        }
    }

    #[test]
    fn trial_adverse_events_prefers_alias_copy_with_counted_terms() {
        let outcome = trial_adverse_events_from_study_batches(vec![
            vec![ctgov_study(serde_json::json!({
                "protocolSection": {
                    "identificationModule": {
                        "nctId": "NCT05379985",
                        "briefTitle": "Daraxonrasib first-in-human study"
                    }
                },
                "hasResults": true,
                "resultsSection": {
                    "adverseEventsModule": {
                        "seriousEvents": [{
                            "term": "Rash",
                            "stats": [{"groupId": "g1", "numAffected": 0, "numAtRisk": 10}]
                        }],
                        "otherEvents": []
                    }
                }
            }))],
            vec![ctgov_study(serde_json::json!({
                "protocolSection": {
                    "identificationModule": {
                        "nctId": "NCT05379985",
                        "briefTitle": "Daraxonrasib first-in-human study"
                    }
                },
                "hasResults": true,
                "resultsSection": {
                    "adverseEventsModule": {
                        "seriousEvents": [{
                            "term": "Rash",
                            "stats": [{"groupId": "g1", "numAffected": 2, "numAtRisk": 10}]
                        }],
                        "otherEvents": [{
                            "term": "Fatigue",
                            "stats": [{"groupId": "g1", "numAffected": 1, "numAtRisk": 10}]
                        }]
                    }
                }
            }))],
        ]);

        match outcome {
            TrialAdverseEventOutcome::Empty => panic!("expected trial adverse events"),
            TrialAdverseEventOutcome::Found(rows) => {
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].term, "Fatigue");
                assert_eq!(rows[0].trial_count, 1);
                assert_eq!(rows[1].term, "Rash");
                assert_eq!(rows[1].trial_count, 1);
            }
        }
    }

    fn injected_cvx_vaers_failure(kind: &str) -> BioMcpError {
        match kind {
            "connection-refused" => BioMcpError::Api {
                api: "VAERS".to_string(),
                message: "connection refused".to_string(),
            },
            "timeout" => BioMcpError::SourceUnavailable {
                source_name: "CDC CVX/MVX".to_string(),
                reason: "request timed out".to_string(),
                suggestion: "retry".to_string(),
            },
            "malformed-body" => BioMcpError::ApiJson {
                api: "VAERS".to_string(),
                source: serde_json::from_str::<serde_json::Value>("{")
                    .expect_err("fixture body must be malformed"),
            },
            other => panic!("unknown injected failure: {other}"),
        }
    }

    fn matched_mmr() -> VaersMatchedVaccine {
        VaersMatchedVaccine {
            display_name: "MMR".to_string(),
            wonder_code: "MMR".to_string(),
            cvx_codes: vec!["03".to_string(), "94".to_string()],
        }
    }

    fn assert_cvx_vaers_outcome(
        payload: &VaersSearchPayload,
        expected: crate::entities::section_outcome::SectionOutcomeState,
    ) {
        use crate::entities::section_outcome::SectionOutcomeState;

        let outcome = vaers_section_outcome(payload);
        assert_eq!(outcome.outcome(), expected);
        if expected == SectionOutcomeState::Unavailable {
            assert_eq!(payload.status, VaersSearchStatus::Unavailable);
            assert!(outcome.sources().is_empty());
        } else {
            assert_eq!(outcome.sources(), &["CDC CVX", "CDC VAERS"]);
        }
    }

    #[test]
    fn cvx_vaers_failure_state_matrix() {
        use crate::entities::section_outcome::SectionOutcomeState;

        for failure in ["connection-refused", "timeout", "malformed-body"] {
            let result: Result<VaersSearchPayload, BioMcpError> =
                Err(injected_cvx_vaers_failure(failure));
            let payload = vaers_payload_from_result(result);
            assert_cvx_vaers_outcome(&payload, SectionOutcomeState::Unavailable);
        }

        let mut empty_tables = vaers_summary_tables_fixture();
        empty_tables.total_reports = 0;
        empty_tables.serious_reports = 0;
        empty_tables.non_serious_reports = 0;
        empty_tables.reactions.clear();
        empty_tables.age_distribution.clear();
        let empty =
            vaers_payload_from_result(Ok(vaers_summary_from_tables(matched_mmr(), empty_tables)));
        assert_eq!(empty.status, VaersSearchStatus::Empty);
        assert_cvx_vaers_outcome(&empty, SectionOutcomeState::Empty);

        let data = vaers_payload_from_result(Ok(vaers_summary_from_tables(
            matched_mmr(),
            vaers_summary_tables_fixture(),
        )));
        assert_eq!(data.status, VaersSearchStatus::Ok);
        assert!(
            data.summary
                .as_ref()
                .is_some_and(|summary| summary.total_reports > 0)
        );
        assert_cvx_vaers_outcome(&data, SectionOutcomeState::Data);
    }
}
