use futures::{StreamExt, stream};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::warn;

use crate::entities::SearchPage;
use crate::error::BioMcpError;
use crate::sources::clinicaltrials::{
    ClinicalTrialsClient, CtGovLocation, CtGovSearchParams, CtGovStudy,
};
use crate::sources::nci_cts::{NciCtsClient, NciSearchParams};
use crate::transform;
use crate::utils::date::validate_since;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    pub nct_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub study_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_range: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eligibility_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<TrialLocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcomes: Option<TrialOutcomes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arms: Option<Vec<TrialArm>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<TrialReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialLocation {
    pub facility: String,
    pub city: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub country: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcomes {
    #[serde(default)]
    pub primary: Vec<TrialOutcome>,
    #[serde(default)]
    pub secondary: Vec<TrialOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialOutcome {
    pub measure: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialArm {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arm_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub interventions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialReference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    pub citation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialSearchResult {
    pub nct_id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TrialSearchFilters {
    pub condition: Option<String>,
    pub intervention: Option<String>,
    pub facility: Option<String>,
    pub status: Option<String>,
    pub phase: Option<String>,
    pub study_type: Option<String>,
    pub age: Option<u32>,
    pub sex: Option<String>,
    pub sponsor: Option<String>,
    pub sponsor_type: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub mutation: Option<String>,
    pub biomarker: Option<String>,
    pub prior_therapies: Option<String>,
    pub progression_on: Option<String>,
    pub line_of_therapy: Option<String>,
    pub results_available: bool,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub distance: Option<u32>,
    pub source: TrialSource,
}

#[derive(Debug, Clone, Default, Copy)]
pub enum TrialSource {
    #[default]
    ClinicalTrialsGov,
    NciCts,
}

impl TrialSource {
    pub fn from_flag(value: &str) -> Result<Self, BioMcpError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "ctgov" | "clinicaltrials" | "clinicaltrials.gov" => Ok(Self::ClinicalTrialsGov),
            "nci" | "nci_cts" | "cts" => Ok(Self::NciCts),
            other => Err(BioMcpError::InvalidArgument(format!(
                "Unknown --source '{other}'. Expected 'ctgov' or 'nci'."
            ))),
        }
    }
}

const TRIAL_SECTION_ELIGIBILITY: &str = "eligibility";
const TRIAL_SECTION_LOCATIONS: &str = "locations";
const TRIAL_SECTION_OUTCOMES: &str = "outcomes";
const TRIAL_SECTION_ARMS: &str = "arms";
const TRIAL_SECTION_REFERENCES: &str = "references";
const TRIAL_SECTION_ALL: &str = "all";

pub const TRIAL_SECTION_NAMES: &[&str] = &[
    TRIAL_SECTION_ELIGIBILITY,
    TRIAL_SECTION_LOCATIONS,
    TRIAL_SECTION_OUTCOMES,
    TRIAL_SECTION_ARMS,
    TRIAL_SECTION_REFERENCES,
    TRIAL_SECTION_ALL,
];

const ELIGIBILITY_MAX_CHARS: usize = 12_000;
const FACILITY_GEO_VERIFY_CONCURRENCY: usize = 8;
const ELIGIBILITY_VERIFY_CONCURRENCY: usize = 8;
const CTGOV_MAX_PAGE_FETCHES: usize = 20;

#[derive(Debug, Clone, Copy, Default)]
struct TrialSections {
    include_eligibility: bool,
    include_locations: bool,
    include_outcomes: bool,
    include_arms: bool,
    include_references: bool,
}

fn parse_sections(sections: &[String]) -> Result<TrialSections, BioMcpError> {
    let mut out = TrialSections::default();
    let mut include_all = false;

    for raw in sections {
        let section = raw.trim().to_ascii_lowercase();
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" {
            continue;
        }
        match section.as_str() {
            TRIAL_SECTION_ELIGIBILITY => out.include_eligibility = true,
            TRIAL_SECTION_LOCATIONS => out.include_locations = true,
            TRIAL_SECTION_OUTCOMES => out.include_outcomes = true,
            TRIAL_SECTION_ARMS => out.include_arms = true,
            TRIAL_SECTION_REFERENCES => out.include_references = true,
            TRIAL_SECTION_ALL => include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for trial. Available: {}",
                    TRIAL_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if include_all {
        out.include_eligibility = true;
        out.include_locations = true;
        out.include_outcomes = true;
        out.include_arms = true;
        out.include_references = true;
    }

    Ok(out)
}

fn essie_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if matches!(
            ch,
            '\\' | '\"'
                | '+'
                | '-'
                | '!'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '^'
                | '~'
                | '*'
                | '?'
                | ':'
                | '/'
                | '|'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn has_essie_filters(filters: &TrialSearchFilters) -> bool {
    filters
        .prior_therapies
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .progression_on
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .line_of_therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
}

fn line_of_therapy_patterns(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_uppercase().as_str() {
        "1L" => Some(
            "\"first line\" OR \"first-line\" OR \"1st line\" OR \"frontline\" OR \"treatment naive\" OR \"previously untreated\"",
        ),
        "2L" => Some(
            "\"second line\" OR \"second-line\" OR \"2nd line\" OR \"one prior line\" OR \"1 prior line\"",
        ),
        "3L+" => Some(
            "\"third line\" OR \"third-line\" OR \"3rd line\" OR \"≥2 prior\" OR \"at least 2 prior\" OR \"heavily pretreated\"",
        ),
        _ => None,
    }
}

fn build_essie_fragments(filters: &TrialSearchFilters) -> Result<Vec<String>, BioMcpError> {
    let mut fragments = Vec::new();

    if let Some(therapy) = filters
        .prior_therapies
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let therapy = essie_escape(therapy);
        fragments.push(format!(
            "AREA[EligibilityCriteria](\"{therapy}\" AND (prior OR previous OR received))"
        ));
    }

    if let Some(drug) = filters
        .progression_on
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let drug = essie_escape(drug);
        fragments.push(format!(
            "AREA[EligibilityCriteria](\"{drug}\" AND (progression OR resistant OR refractory))"
        ));
    }

    if let Some(line) = filters
        .line_of_therapy
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let patterns = line_of_therapy_patterns(line).ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "Invalid --line-of-therapy value. Expected one of: 1L, 2L, 3L+".into(),
            )
        })?;
        fragments.push(format!("AREA[EligibilityCriteria]({patterns})"));
    }

    Ok(fragments)
}

fn normalize_enum_key(value: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = false;
    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
            prev_sep = false;
            continue;
        }
        if matches!(ch, ' ' | ',' | '-' | '_') && !prev_sep {
            out.push('_');
            prev_sep = true;
        }
    }
    out.trim_matches('_').to_string()
}

fn invalid_status_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --status value '{raw}'. Expected one of: \
NOT_YET_RECRUITING, RECRUITING, ENROLLING_BY_INVITATION, ACTIVE_NOT_RECRUITING, \
COMPLETED, SUSPENDED, TERMINATED, WITHDRAWN. Aliases: active, comma/space forms."
    ))
}

fn normalize_status(value: &str) -> Result<String, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--status must not be empty".into(),
        ));
    }

    let key = normalize_enum_key(raw);
    let canonical = match key.as_str() {
        "NOT_YET_RECRUITING" => "NOT_YET_RECRUITING",
        "RECRUITING" => "RECRUITING",
        "ENROLLING_BY_INVITATION" | "ENROLLING" => "ENROLLING_BY_INVITATION",
        "ACTIVE_NOT_RECRUITING" | "ACTIVE" => "ACTIVE_NOT_RECRUITING",
        "COMPLETED" | "COMPLETE" => "COMPLETED",
        "SUSPENDED" => "SUSPENDED",
        "TERMINATED" => "TERMINATED",
        "WITHDRAWN" => "WITHDRAWN",
        _ => return Err(invalid_status_error(raw)),
    };
    Ok(canonical.to_string())
}

fn status_priority(value: &str) -> u8 {
    match normalize_enum_key(value).as_str() {
        "RECRUITING" => 0,
        "ACTIVE_NOT_RECRUITING" => 1,
        "ENROLLING_BY_INVITATION" => 2,
        "NOT_YET_RECRUITING" => 3,
        "COMPLETED" => 4,
        "UNKNOWN" => 5,
        "WITHDRAWN" => 6,
        "TERMINATED" => 7,
        "SUSPENDED" => 8,
        _ => 9,
    }
}

fn sort_trials_by_status_priority(rows: &mut [TrialSearchResult]) {
    rows.sort_by(|a, b| {
        status_priority(&a.status)
            .cmp(&status_priority(&b.status))
            .then_with(|| a.nct_id.cmp(&b.nct_id))
    });
}

fn invalid_phase_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --phase value '{raw}'. Expected one of: NA, EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4. \
Aliases: 1-4, 1/2, early_phase1, early1, n/a."
    ))
}

fn invalid_sex_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --sex value '{raw}'. Expected one of: female, male, all."
    ))
}

fn normalize_sex(value: &str) -> Result<Option<&'static str>, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--sex must not be empty".into(),
        ));
    }
    match normalize_enum_key(raw).as_str() {
        "FEMALE" | "F" => Ok(Some("f")),
        "MALE" | "M" => Ok(Some("m")),
        "ALL" | "ANY" | "BOTH" => Ok(None),
        _ => Err(invalid_sex_error(raw)),
    }
}

fn invalid_sponsor_type_error(raw: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unrecognized --sponsor-type value '{raw}'. Expected one of: nih, industry, fed, other."
    ))
}

fn normalize_sponsor_type(value: &str) -> Result<&'static str, BioMcpError> {
    let raw = value.trim();
    if raw.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--sponsor-type must not be empty".into(),
        ));
    }
    match normalize_enum_key(raw).as_str() {
        "NIH" => Ok("nih"),
        "INDUSTRY" => Ok("industry"),
        "FED" | "FEDERAL" => Ok("fed"),
        "OTHER" => Ok("other"),
        _ => Err(invalid_sponsor_type_error(raw)),
    }
}

fn normalize_phase(value: &str) -> Result<String, BioMcpError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--phase must not be empty".into(),
        ));
    }

    let compact = v
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    if matches!(
        compact.as_str(),
        "1/2" | "EARLY_PHASE1" | "EARLYPHASE1" | "EARLY1"
    ) {
        return Ok("EARLY_PHASE1".to_string());
    }
    if matches!(compact.as_str(), "NA" | "N/A") {
        return Ok("NA".to_string());
    }
    if compact.chars().all(|c| c.is_ascii_digit()) {
        return match compact.as_str() {
            "1" => Ok("PHASE1".to_string()),
            "2" => Ok("PHASE2".to_string()),
            "3" => Ok("PHASE3".to_string()),
            "4" => Ok("PHASE4".to_string()),
            _ => Err(invalid_phase_error(v)),
        };
    }

    let key = normalize_enum_key(v);
    match key.as_str() {
        "PHASE1" => Ok("PHASE1".to_string()),
        "PHASE2" => Ok("PHASE2".to_string()),
        "PHASE3" => Ok("PHASE3".to_string()),
        "PHASE4" => Ok("PHASE4".to_string()),
        "EARLY_PHASE1" | "EARLY1" => Ok("EARLY_PHASE1".to_string()),
        "NA" => Ok("NA".to_string()),
        _ => Err(invalid_phase_error(v)),
    }
}

fn normalized_status_filter(filters: &TrialSearchFilters) -> Result<Option<String>, BioMcpError> {
    filters
        .status
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(normalize_status)
        .transpose()
}

fn normalized_phase_filter(filters: &TrialSearchFilters) -> Result<Option<String>, BioMcpError> {
    filters
        .phase
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(normalize_phase)
        .transpose()
}

fn normalized_facility_filter(filters: &TrialSearchFilters) -> Option<String> {
    filters
        .facility
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn normalize_facility_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

fn haversine_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_MILES: f64 = 3958.7613;
    let to_rad = |deg: f64| deg.to_radians();
    let d_lat = to_rad(lat2 - lat1);
    let d_lon = to_rad(lon2 - lon1);
    let lat1_rad = to_rad(lat1);
    let lat2_rad = to_rad(lat2);

    let a =
        (d_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_MILES * c
}

fn location_matches_facility_geo(
    location: &CtGovLocation,
    facility_needle: &str,
    origin_lat: f64,
    origin_lon: f64,
    max_distance_miles: u32,
) -> bool {
    let Some(location_facility) = location
        .facility
        .as_deref()
        .and_then(normalize_facility_text)
    else {
        return false;
    };
    if !location_facility.contains(facility_needle) {
        return false;
    }
    let Some(geo) = location.geo_point.as_ref() else {
        return false;
    };
    let (Some(lat), Some(lon)) = (geo.lat, geo.lon) else {
        return false;
    };

    haversine_miles(origin_lat, origin_lon, lat, lon) <= max_distance_miles as f64
}

fn ctgov_nct_id(study: &CtGovStudy) -> Option<String> {
    study
        .protocol_section
        .as_ref()
        .and_then(|section| section.identification_module.as_ref())
        .and_then(|id| id.nct_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn trial_matches_facility_geo(
    study: &CtGovStudy,
    facility_needle: &str,
    origin_lat: f64,
    origin_lon: f64,
    max_distance_miles: u32,
) -> bool {
    study
        .protocol_section
        .as_ref()
        .and_then(|section| section.contacts_locations_module.as_ref())
        .map(|module| {
            module.locations.iter().any(|location| {
                location_matches_facility_geo(
                    location,
                    facility_needle,
                    origin_lat,
                    origin_lon,
                    max_distance_miles,
                )
            })
        })
        .unwrap_or(false)
}

fn exclusion_criteria_header_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?mi)^\s*(?:Key\s+)?Exclusion\s+Criteria\s*:?\s*$")
            .expect("exclusion criteria header regex is valid")
    })
}

fn split_eligibility_sections(text: &str) -> (String, String) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (String::new(), String::new());
    }

    let Some(header) = exclusion_criteria_header_re().find(trimmed) else {
        return (trimmed.to_ascii_lowercase(), String::new());
    };

    let inclusion = trimmed[..header.start()].trim().to_ascii_lowercase();
    let exclusion = trimmed[header.end()..].trim().to_ascii_lowercase();
    (inclusion, exclusion)
}

fn contains_keyword_tokens(section_text: &str, keyword: &str) -> bool {
    if section_text.is_empty() {
        return false;
    }

    let token_pattern = keyword
        .split_whitespace()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(regex::escape)
        .collect::<Vec<String>>();

    if token_pattern.is_empty() {
        return false;
    }

    token_pattern.iter().all(|token| {
        let pattern = build_token_pattern(token);
        Regex::new(&pattern)
            .map(|regex| regex.is_match(section_text))
            .unwrap_or(false)
    })
}

fn build_token_pattern(escaped_token: &str) -> String {
    let start = if escaped_token
        .chars()
        .next()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        r"\b"
    } else {
        r"(^|[^\w])"
    };
    let end = if escaped_token
        .chars()
        .last()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        r"\b"
    } else {
        r"($|[^\w])"
    };
    format!("{start}{escaped_token}{end}")
}

fn contains_exclusion_language(text: &str) -> bool {
    [
        "exclude",
        "excluded",
        "exclusion",
        "ineligible",
        "ineligibility",
        "not eligible",
        "not allowed",
        "not permitted",
        "must not",
    ]
    .iter()
    .any(|cue| text.contains(cue))
}

fn keyword_has_positive_inclusion_context(inclusion_text: &str, keyword: &str) -> bool {
    inclusion_text
        .split(['\n', '.', ';'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .filter(|segment| contains_keyword_tokens(segment, keyword))
        .any(|segment| !contains_exclusion_language(segment))
}

fn eligibility_keyword_in_inclusion(
    inclusion_text: &str,
    exclusion_text: &str,
    keyword: &str,
) -> bool {
    let keyword = keyword.trim().to_ascii_lowercase();
    if keyword.is_empty() || exclusion_text.is_empty() {
        return true;
    }

    let inclusion_has_keyword = contains_keyword_tokens(inclusion_text, &keyword);
    if inclusion_has_keyword && keyword_has_positive_inclusion_context(inclusion_text, &keyword) {
        return true;
    }

    if contains_keyword_tokens(exclusion_text, &keyword) {
        return false;
    }

    if inclusion_has_keyword {
        return false;
    }

    true
}

fn collect_eligibility_keywords(filters: &TrialSearchFilters) -> Vec<String> {
    [
        filters.mutation.as_deref(),
        filters.biomarker.as_deref(),
        filters.prior_therapies.as_deref(),
        filters.progression_on.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(str::trim)
    .filter(|value| !value.is_empty())
    .map(str::to_string)
    .collect()
}

async fn verify_facility_geo(
    client: &ClinicalTrialsClient,
    studies: Vec<CtGovStudy>,
    facility_filter: &str,
    origin_lat: f64,
    origin_lon: f64,
    max_distance_miles: u32,
) -> Vec<CtGovStudy> {
    let Some(facility_needle) = normalize_facility_text(facility_filter) else {
        return studies;
    };

    let location_section = vec![TRIAL_SECTION_LOCATIONS.to_string()];
    let mut verification_stream = stream::iter(studies.into_iter().map(|study| {
        let nct_id = ctgov_nct_id(&study);
        let sections = location_section.clone();
        let facility_needle = facility_needle.clone();
        async move {
            let Some(nct_id) = nct_id else {
                return Some(study);
            };
            match client.get(&nct_id, &sections).await {
                Ok(details) => trial_matches_facility_geo(
                    &details,
                    &facility_needle,
                    origin_lat,
                    origin_lon,
                    max_distance_miles,
                )
                .then_some(study),
                Err(e) => {
                    warn!(nct_id, error = %e, "facility-geo detail fetch failed, keeping study");
                    Some(study)
                }
            }
        }
    }))
    .buffered(FACILITY_GEO_VERIFY_CONCURRENCY);

    let mut verified = Vec::new();
    while let Some(maybe_study) = verification_stream.next().await {
        if let Some(study) = maybe_study {
            verified.push(study);
        }
    }
    verified
}

async fn verify_eligibility_criteria(
    client: &ClinicalTrialsClient,
    studies: Vec<CtGovStudy>,
    keywords: &[String],
) -> Vec<CtGovStudy> {
    if keywords.is_empty() {
        return studies;
    }

    let eligibility_section = vec![TRIAL_SECTION_ELIGIBILITY.to_string()];
    let keywords = keywords.to_vec();
    let mut verification_stream = stream::iter(studies.into_iter().map(|study| {
        let nct_id = ctgov_nct_id(&study);
        let sections = eligibility_section.clone();
        let keywords = keywords.clone();
        async move {
            let Some(nct_id) = nct_id else {
                return Some(study);
            };
            match client.get(&nct_id, &sections).await {
                Ok(details) => {
                    let Some(criteria) = details
                        .protocol_section
                        .as_ref()
                        .and_then(|section| section.eligibility_module.as_ref())
                        .and_then(|module| module.eligibility_criteria.as_deref())
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    else {
                        warn!(
                            nct_id,
                            "missing eligibility criteria in detail fetch, keeping study"
                        );
                        return Some(study);
                    };

                    let (inclusion, exclusion) = split_eligibility_sections(criteria);
                    keywords
                        .iter()
                        .all(|keyword| {
                            eligibility_keyword_in_inclusion(&inclusion, &exclusion, keyword)
                        })
                        .then_some(study)
                }
                Err(e) => {
                    warn!(nct_id, error = %e, "eligibility detail fetch failed, keeping study");
                    Some(study)
                }
            }
        }
    }))
    .buffered(ELIGIBILITY_VERIFY_CONCURRENCY);

    let mut verified = Vec::new();
    while let Some(maybe_study) = verification_stream.next().await {
        if let Some(study) = maybe_study {
            verified.push(study);
        }
    }
    verified
}

fn ctgov_agg_filters(filters: &TrialSearchFilters) -> Result<Option<String>, BioMcpError> {
    let mut facets: Vec<String> = Vec::new();

    if let Some(sex) = filters
        .sex
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        && let Some(code) = normalize_sex(sex)?
    {
        facets.push(format!("sex:{code}"));
    }

    if let Some(sponsor_type) = filters
        .sponsor_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        facets.push(format!(
            "funderType:{}",
            normalize_sponsor_type(sponsor_type)?
        ));
    }

    if facets.is_empty() {
        Ok(None)
    } else {
        Ok(Some(facets.join(",")))
    }
}

fn validate_location(filters: &TrialSearchFilters) -> Result<(), BioMcpError> {
    let has_lat = filters.lat.is_some();
    let has_lon = filters.lon.is_some();
    let has_distance = filters.distance.is_some();

    if has_distance && (!has_lat || !has_lon) {
        return Err(BioMcpError::InvalidArgument(
            "--distance requires both --lat and --lon".into(),
        ));
    }
    if (has_lat || has_lon) && !has_distance {
        return Err(BioMcpError::InvalidArgument(
            "--lat/--lon requires --distance".into(),
        ));
    }
    if has_lat != has_lon {
        return Err(BioMcpError::InvalidArgument(
            "--lat and --lon must be provided together".into(),
        ));
    }
    Ok(())
}

fn truncate_inline_text(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }
    let truncated = value.chars().take(max_chars).collect::<String>();
    format!("{truncated}\n\n(truncated, {count} chars total)")
}

fn looks_like_nct_id(value: &str) -> bool {
    let v = value.trim().as_bytes();
    if v.len() != 11 {
        return false;
    }
    if &v[0..3] != b"NCT" {
        return false;
    }
    v[3..].iter().all(|b| b.is_ascii_digit())
}

fn normalize_nct_id(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(prefix) = trimmed.get(..3)
        && prefix.eq_ignore_ascii_case("NCT")
    {
        return format!("NCT{}", &trimmed[3..]);
    }
    trimmed.to_string()
}

fn ctgov_query_term(
    filters: &TrialSearchFilters,
    normalized_phase: Option<&str>,
) -> Result<Option<String>, BioMcpError> {
    let mut terms: Vec<String> = Vec::new();

    if let Some(phase) = normalized_phase {
        terms.push(format!("AREA[Phase]{phase}"));
    }
    if let Some(sponsor) = filters
        .sponsor
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let sponsor = essie_escape(sponsor);
        terms.push(format!("AREA[LeadSponsorName]\"{sponsor}\""));
    }
    if let Some(mutation) = filters
        .mutation
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let mutation = essie_escape(mutation);
        terms.push(format!("AREA[EligibilityCriteria]\"{mutation}\""));
    }
    if let Some(biomarker) = filters
        .biomarker
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let biomarker = essie_escape(biomarker);
        terms.push(format!("AREA[EligibilityCriteria]\"{biomarker}\""));
    }
    if let Some(study_type) = filters
        .study_type
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let study_type = essie_escape(study_type);
        terms.push(format!("AREA[StudyType]\"{study_type}\""));
    }
    terms.extend(build_essie_fragments(filters)?);
    if let Some(date_from) = filters
        .date_from
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let date_from = validate_since(date_from)?;
        let date_to = filters
            .date_to
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(validate_since)
            .transpose()?;
        if let Some(date_to) = date_to.as_deref() {
            if date_from.as_str() > date_to {
                return Err(BioMcpError::InvalidArgument(
                    "--date-from must be <= --date-to".into(),
                ));
            }
            terms.push(format!(
                "AREA[LastUpdatePostDate]RANGE[{date_from},{date_to}]"
            ));
        } else {
            terms.push(format!("AREA[LastUpdatePostDate]RANGE[{date_from},MAX]"));
        }
    } else if let Some(date_to) = filters
        .date_to
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        let date_to = validate_since(date_to)?;
        terms.push(format!("AREA[LastUpdatePostDate]RANGE[MIN,{date_to}]"));
    }
    if filters.results_available {
        terms.push("AREA[ResultsFirstPostDate]RANGE[MIN,MAX]".to_string());
    }
    if let Some(age) = filters.age {
        terms.push(format!("AREA[MinimumAge]RANGE[MIN,{age} years]"));
        terms.push(format!("AREA[MaximumAge]RANGE[{age} years,MAX]"));
    }

    if terms.is_empty() {
        Ok(None)
    } else {
        Ok(Some(terms.join(" AND ")))
    }
}

fn has_any_query(filters: &TrialSearchFilters) -> bool {
    filters
        .condition
        .as_deref()
        .map(str::trim)
        .is_some_and(|v| !v.is_empty())
        || filters
            .intervention
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .facility
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .mutation
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .biomarker
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .prior_therapies
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .progression_on
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .line_of_therapy
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .sponsor
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .status
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .phase
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .study_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.age.is_some()
        || filters
            .sex
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .sponsor_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .date_from
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters
            .date_to
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
        || filters.results_available
        || filters.distance.is_some()
}

pub async fn search(
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
) -> Result<(Vec<TrialSearchResult>, Option<u32>), BioMcpError> {
    let page = search_page(filters, limit, offset, None).await?;
    Ok((page.results, page.total.map(|v| v as u32)))
}

pub async fn search_page(
    filters: &TrialSearchFilters,
    limit: usize,
    offset: usize,
    next_page: Option<String>,
) -> Result<SearchPage<TrialSearchResult>, BioMcpError> {
    const MAX_SEARCH_LIMIT: usize = 50;
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(BioMcpError::InvalidArgument(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    if !has_any_query(filters) {
        return Err(BioMcpError::InvalidArgument(
            "At least one filter is required. Example: biomcp search trial -c melanoma".into(),
        ));
    }
    let normalized_status = normalized_status_filter(filters)?;
    let normalized_phase = normalized_phase_filter(filters)?;
    validate_location(filters)?;
    if matches!(filters.source, TrialSource::NciCts) && has_essie_filters(filters) {
        return Err(BioMcpError::InvalidArgument(
            "--prior-therapies, --progression-on, and --line-of-therapy are only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts) && filters.results_available {
        return Err(BioMcpError::InvalidArgument(
            "--results-available is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts) && filters.age.is_some() {
        return Err(BioMcpError::InvalidArgument(
            "--age is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts)
        && filters
            .sex
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
    {
        return Err(BioMcpError::InvalidArgument(
            "--sex is only supported for --source ctgov".into(),
        ));
    }
    if matches!(filters.source, TrialSource::NciCts)
        && filters
            .sponsor_type
            .as_deref()
            .map(str::trim)
            .is_some_and(|v| !v.is_empty())
    {
        return Err(BioMcpError::InvalidArgument(
            "--sponsor-type is only supported for --source ctgov".into(),
        ));
    }
    if next_page
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
        && offset > 0
    {
        return Err(BioMcpError::InvalidArgument(
            "--next-page cannot be used together with --offset".into(),
        ));
    }

    match filters.source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            let query_term = ctgov_query_term(filters, normalized_phase.as_deref())?;
            let facility = normalized_facility_filter(filters);
            let eligibility_keywords = collect_eligibility_keywords(filters);
            let agg_filters = ctgov_agg_filters(filters)?;
            let has_explicit_status = filters
                .status
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| !v.is_empty());
            let facility_geo_verification = facility
                .as_deref()
                .zip(filters.lat)
                .zip(filters.lon)
                .zip(filters.distance)
                .map(|(((facility_name, lat), lon), distance)| {
                    (facility_name.to_string(), lat, lon, distance)
                });
            let uses_post_filters =
                facility_geo_verification.is_some() || !eligibility_keywords.is_empty();

            let page_size = limit.clamp(1, 100);
            let mut rows: Vec<TrialSearchResult> = Vec::new();
            let mut total: Option<usize> = None;
            let mut verified_total: usize = 0;
            let mut exhausted = false;
            let mut page_token = next_page
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let mut remaining_skip = offset;
            for _ in 0..CTGOV_MAX_PAGE_FETCHES {
                let resp = client
                    .search(&CtGovSearchParams {
                        condition: filters.condition.clone(),
                        intervention: filters.intervention.clone(),
                        facility: facility.clone(),
                        status: normalized_status.clone(),
                        agg_filters: agg_filters.clone(),
                        query_term: query_term.clone(),
                        count_total: true,
                        page_token: page_token.clone(),
                        page_size,
                        lat: filters.lat,
                        lon: filters.lon,
                        distance_miles: filters.distance,
                    })
                    .await?;
                if total.is_none() {
                    total = resp.total_count.map(|v| v as usize);
                }
                let mut studies = resp.studies;
                let next_page_token = resp.next_page_token;

                if studies.is_empty() {
                    exhausted = true;
                    break;
                }

                if let Some((facility_name, lat, lon, distance)) =
                    facility_geo_verification.as_ref()
                {
                    studies =
                        verify_facility_geo(&client, studies, facility_name, *lat, *lon, *distance)
                            .await;
                }
                if !eligibility_keywords.is_empty() {
                    studies =
                        verify_eligibility_criteria(&client, studies, &eligibility_keywords).await;
                }

                if uses_post_filters {
                    verified_total = verified_total.saturating_add(studies.len());
                }

                let page_study_count = studies.len();
                let mut page_consumed = 0;
                for study in studies.drain(..) {
                    page_consumed += 1;
                    if remaining_skip > 0 {
                        remaining_skip -= 1;
                        continue;
                    }
                    if rows.len() < limit {
                        rows.push(transform::trial::from_ctgov_hit(&study));
                    }
                    if rows.len() >= limit {
                        break;
                    }
                }

                if rows.len() >= limit {
                    // If we consumed every study on this page, advance to
                    // the next cursor.  Otherwise we stopped mid-page and
                    // an opaque cursor can't represent the mid-page offset,
                    // so return None (caller should use --offset instead).
                    if page_consumed >= page_study_count {
                        page_token = next_page_token;
                    } else {
                        page_token = None;
                    }
                    break;
                }

                page_token = next_page_token;
                if page_token.is_none() {
                    exhausted = true;
                    break;
                }
            }

            if !has_explicit_status {
                sort_trials_by_status_priority(&mut rows);
            }

            rows.truncate(limit);
            let returned_total = if uses_post_filters {
                if exhausted {
                    Some(verified_total)
                } else {
                    // Conservative lower bound when traversal is capped.
                    Some(
                        verified_total
                            .saturating_add(1)
                            .max(offset.saturating_add(rows.len()).saturating_add(1)),
                    )
                }
            } else {
                total.or_else(|| Some(offset.saturating_add(rows.len())))
            };
            Ok(SearchPage::cursor(rows, returned_total, page_token))
        }
        TrialSource::NciCts => {
            if filters.date_from.is_some() || filters.date_to.is_some() {
                return Err(BioMcpError::InvalidArgument(
                    "--date-from/--date-to is only supported for --source ctgov".into(),
                ));
            }
            if next_page.is_some() {
                return Err(BioMcpError::InvalidArgument(
                    "--next-page is only supported for --source ctgov".into(),
                ));
            }
            let client = NciCtsClient::new()?;

            let params = NciSearchParams {
                diseases: filters.condition.clone(),
                interventions: filters.intervention.clone(),
                sites_org_name: normalized_facility_filter(filters),
                recruitment_status: normalized_status,
                phase: normalized_phase,
                latitude: filters.lat,
                longitude: filters.lon,
                distance: filters.distance,
                biomarkers: filters
                    .biomarker
                    .clone()
                    .or_else(|| filters.mutation.clone()),
                size: limit,
                from: offset,
            };

            let resp = client.search(&params).await?;
            Ok(SearchPage::offset(
                resp.hits()
                    .iter()
                    .map(transform::trial::from_nci_hit)
                    .collect(),
                resp.total,
            ))
        }
    }
}

pub async fn get(
    nct_id: &str,
    sections: &[String],
    source: TrialSource,
) -> Result<Trial, BioMcpError> {
    let nct_id = normalize_nct_id(nct_id);
    let nct_id = nct_id.trim();
    if nct_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "NCT ID is required. Example: biomcp get trial NCT02576665".into(),
        ));
    }
    if nct_id.len() > 64 {
        return Err(BioMcpError::InvalidArgument("NCT ID is too long.".into()));
    }
    if !looks_like_nct_id(nct_id) {
        return Err(BioMcpError::NotFound {
            entity: "trial".into(),
            id: nct_id.to_string(),
            suggestion: format!("Try searching: biomcp search trial -c \"{nct_id}\""),
        });
    }

    let section_flags = parse_sections(sections)?;

    match source {
        TrialSource::ClinicalTrialsGov => {
            let client = ClinicalTrialsClient::new()?;
            let study = client.get(nct_id, sections).await?;
            let mut trial = transform::trial::from_ctgov_study(&study);
            trial.source = Some("ClinicalTrials.gov".into());

            if section_flags.include_eligibility {
                let criteria = study
                    .protocol_section
                    .as_ref()
                    .and_then(|p| p.eligibility_module.as_ref())
                    .and_then(|m| m.eligibility_criteria.as_deref())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());

                if let Some(criteria) = criteria {
                    trial.eligibility_text =
                        Some(truncate_inline_text(criteria, ELIGIBILITY_MAX_CHARS));
                }
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
        TrialSource::NciCts => {
            let client = NciCtsClient::new()?;
            let resp = client.get(nct_id).await?;
            let mut trial = transform::trial::from_nci_trial(&resp);
            trial.source = Some("NCI CTS".into());

            if section_flags.include_eligibility {
                // Best-effort: look for eligibility in common fields.
                let criteria = resp
                    .get("eligibility")
                    .and_then(|v| v.as_str())
                    .map(str::trim)
                    .filter(|s| !s.is_empty());
                if let Some(criteria) = criteria {
                    trial.eligibility_text =
                        Some(truncate_inline_text(criteria, ELIGIBILITY_MAX_CHARS));
                } else {
                    warn!(nct_id, "NCI CTS eligibility criteria not found in response");
                }
            }
            if section_flags.include_references && trial.references.is_none() {
                trial.references = Some(Vec::new());
            }

            Ok(trial)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctgov_study_fixture(locations: serde_json::Value) -> CtGovStudy {
        serde_json::from_value(json!({
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT00000001",
                    "briefTitle": "Fixture Trial",
                    "overallStatus": "RECRUITING"
                },
                "contactsLocationsModule": {
                    "locations": locations
                }
            }
        }))
        .expect("valid CtGovStudy fixture")
    }

    #[test]
    fn split_eligibility_sections_detects_exclusion_header() {
        let text = "Inclusion Criteria:\nMust have MSI-H disease\n\nExclusion Criteria:\nNo active CNS mets";
        let (inclusion, exclusion) = split_eligibility_sections(text);
        assert!(inclusion.contains("must have msi-h disease"));
        assert!(exclusion.contains("no active cns mets"));
    }

    #[test]
    fn split_eligibility_sections_supports_key_exclusion_header() {
        let text =
            "Inclusion:\nBRAF V600E mutation\n\nKey Exclusion Criteria:\nPrior anti-braf therapy";
        let (inclusion, exclusion) = split_eligibility_sections(text);
        assert!(inclusion.contains("braf v600e mutation"));
        assert!(exclusion.contains("prior anti-braf therapy"));
    }

    #[test]
    fn split_eligibility_sections_without_exclusion_keeps_all_in_inclusion() {
        let text = "Inclusion Criteria:\nPathogenic EGFR mutation";
        let (inclusion, exclusion) = split_eligibility_sections(text);
        assert!(inclusion.contains("pathogenic egfr mutation"));
        assert!(exclusion.is_empty());
    }

    #[test]
    fn eligibility_keyword_in_inclusion_keeps_when_inclusion_matches() {
        assert!(eligibility_keyword_in_inclusion(
            "must have msi-h disease",
            "no untreated brain metastases",
            "MSI-H"
        ));
    }

    #[test]
    fn eligibility_keyword_in_inclusion_discards_exclusion_only_match() {
        assert!(!eligibility_keyword_in_inclusion(
            "must have metastatic colorectal cancer",
            "exclusion includes msi-h tumors",
            "MSI-H"
        ));
    }

    #[test]
    fn eligibility_keyword_in_inclusion_keeps_when_in_both_sections() {
        assert!(eligibility_keyword_in_inclusion(
            "inclusion requires braf v600e mutation",
            "exclude prior braf v600e inhibitor exposure",
            "BRAF V600E"
        ));
    }

    #[test]
    fn eligibility_keyword_in_inclusion_discards_negated_inclusion_sentence() {
        assert!(!eligibility_keyword_in_inclusion(
            "patients whose tumors are msi-h are excluded",
            "exclude active infection",
            "MSI-H"
        ));
    }

    #[test]
    fn eligibility_keyword_in_inclusion_fails_open_when_keyword_missing() {
        assert!(eligibility_keyword_in_inclusion(
            "include untreated metastatic disease",
            "exclude uncontrolled infection",
            "MSI-H"
        ));
    }

    #[test]
    fn eligibility_keyword_in_inclusion_fails_open_without_exclusion_section() {
        assert!(eligibility_keyword_in_inclusion(
            "patients with msi-h disease",
            "",
            "MSI-H"
        ));
    }

    #[test]
    fn collect_eligibility_keywords_includes_supported_filters() {
        let filters = TrialSearchFilters {
            mutation: Some("MSI-H".into()),
            biomarker: Some("TMB-high".into()),
            prior_therapies: Some("osimertinib".into()),
            progression_on: Some("pembrolizumab".into()),
            ..Default::default()
        };

        assert_eq!(
            collect_eligibility_keywords(&filters),
            vec!["MSI-H", "TMB-high", "osimertinib", "pembrolizumab"]
        );
    }

    #[test]
    fn collect_eligibility_keywords_omits_blank_values() {
        let filters = TrialSearchFilters {
            mutation: Some("   ".into()),
            biomarker: Some(" MSI-H ".into()),
            prior_therapies: None,
            progression_on: Some("".into()),
            ..Default::default()
        };

        assert_eq!(collect_eligibility_keywords(&filters), vec!["MSI-H"]);
    }

    #[test]
    fn contains_keyword_tokens_matches_plus_suffix_token() {
        assert!(contains_keyword_tokens(
            "HER2+ positive breast cancer",
            "HER2+"
        ));
    }

    #[test]
    fn contains_keyword_tokens_does_not_match_without_plus_suffix() {
        assert!(!contains_keyword_tokens("her2 amplification", "HER2+"));
    }

    #[test]
    fn contains_keyword_tokens_matches_slash_separated_plus_tokens() {
        assert!(contains_keyword_tokens("ER+/PR+ breast cancer", "ER+"));
    }

    #[test]
    fn contains_keyword_tokens_matches_hyphenated_token() {
        assert!(contains_keyword_tokens("PD-L1 expression >=1%", "PD-L1"));
    }

    #[test]
    fn contains_keyword_tokens_matches_word_token() {
        assert!(contains_keyword_tokens("BRAF V600E mutation", "BRAF"));
    }

    #[test]
    fn contains_keyword_tokens_rejects_substring_word_match() {
        assert!(!contains_keyword_tokens("abraf", "BRAF"));
    }

    #[test]
    fn status_priority_prefers_recruiting_over_completed() {
        assert!(status_priority("RECRUITING") < status_priority("COMPLETED"));
        assert!(status_priority("ACTIVE_NOT_RECRUITING") < status_priority("UNKNOWN"));
    }

    #[test]
    fn line_of_therapy_patterns_accepts_supported_values() {
        assert!(line_of_therapy_patterns("1L").is_some());
        assert!(line_of_therapy_patterns("2L").is_some());
        assert!(line_of_therapy_patterns("3L+").is_some());
        assert!(line_of_therapy_patterns("2l").is_some());
    }

    #[test]
    fn line_of_therapy_patterns_rejects_invalid_values() {
        assert!(line_of_therapy_patterns("4L").is_none());
        assert!(line_of_therapy_patterns("frontline").is_none());
    }

    #[test]
    fn normalize_phase_accepts_aliases() {
        assert_eq!(normalize_phase("1").unwrap(), "PHASE1");
        assert_eq!(normalize_phase("PHASE2").unwrap(), "PHASE2");
        assert_eq!(normalize_phase("1/2").unwrap(), "EARLY_PHASE1");
        assert_eq!(normalize_phase("early_phase1").unwrap(), "EARLY_PHASE1");
        assert_eq!(normalize_phase("early1").unwrap(), "EARLY_PHASE1");
        assert_eq!(normalize_phase("n/a").unwrap(), "NA");
    }

    #[test]
    fn normalize_phase_rejects_invalid_value() {
        let err = normalize_phase("5").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unrecognized --phase value"));
        assert!(msg.contains("EARLY_PHASE1"));
    }

    #[test]
    fn normalize_status_accepts_ctgov_wording_and_aliases() {
        assert_eq!(
            normalize_status("active, not recruiting").unwrap(),
            "ACTIVE_NOT_RECRUITING"
        );
        assert_eq!(normalize_status("active").unwrap(), "ACTIVE_NOT_RECRUITING");
        assert_eq!(normalize_status("recruiting").unwrap(), "RECRUITING");
        assert_eq!(
            normalize_status("enrolling_by_invitation").unwrap(),
            "ENROLLING_BY_INVITATION"
        );
    }

    #[test]
    fn normalize_status_rejects_invalid_value() {
        let err = normalize_status("bogus").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Unrecognized --status value"));
        assert!(msg.contains("ENROLLING_BY_INVITATION"));
    }

    #[test]
    fn normalize_sex_accepts_supported_values() {
        assert_eq!(normalize_sex("female").unwrap(), Some("f"));
        assert_eq!(normalize_sex("male").unwrap(), Some("m"));
        assert_eq!(normalize_sex("all").unwrap(), None);
        assert_eq!(normalize_sex("F").unwrap(), Some("f"));
        assert_eq!(normalize_sex("M").unwrap(), Some("m"));
    }

    #[test]
    fn normalize_sponsor_type_accepts_supported_values() {
        assert_eq!(normalize_sponsor_type("nih").unwrap(), "nih");
        assert_eq!(normalize_sponsor_type("industry").unwrap(), "industry");
        assert_eq!(normalize_sponsor_type("fed").unwrap(), "fed");
        assert_eq!(normalize_sponsor_type("federal").unwrap(), "fed");
        assert_eq!(normalize_sponsor_type("other").unwrap(), "other");
    }

    #[test]
    fn normalize_sex_rejects_invalid_value() {
        let err = normalize_sex("unknown").unwrap_err();
        assert!(err.to_string().contains("Unrecognized --sex value"));
    }

    #[test]
    fn normalize_sponsor_type_rejects_invalid_value() {
        let err = normalize_sponsor_type("charity").unwrap_err();
        assert!(
            err.to_string()
                .contains("Unrecognized --sponsor-type value")
        );
    }

    #[test]
    fn normalize_nct_id_uppercases_prefix() {
        assert_eq!(normalize_nct_id("nct06162221"), "NCT06162221");
        assert_eq!(normalize_nct_id("NCT06162221"), "NCT06162221");
    }

    #[tokio::test]
    async fn nci_source_rejects_essie_filters() {
        let filters = TrialSearchFilters {
            source: TrialSource::NciCts,
            prior_therapies: Some("platinum".into()),
            ..Default::default()
        };

        let err = search(&filters, 10, 0).await.expect_err("should fail");
        assert!(
            format!("{err}").contains("--prior-therapies, --progression-on, and --line-of-therapy"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn nci_source_rejects_age_filter() {
        let filters = TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("melanoma".into()),
            age: Some(67),
            ..Default::default()
        };

        let err = search(&filters, 10, 0).await.expect_err("should fail");
        assert!(
            format!("{err}").contains("--age is only supported for --source ctgov"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn nci_source_rejects_sex_filter() {
        let filters = TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("melanoma".into()),
            sex: Some("female".into()),
            ..Default::default()
        };

        let err = search(&filters, 10, 0).await.expect_err("should fail");
        assert!(
            format!("{err}").contains("--sex is only supported for --source ctgov"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn nci_source_rejects_sponsor_type_filter() {
        let filters = TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("melanoma".into()),
            sponsor_type: Some("nih".into()),
            ..Default::default()
        };

        let err = search(&filters, 10, 0).await.expect_err("should fail");
        assert!(
            format!("{err}").contains("--sponsor-type is only supported for --source ctgov"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn facility_geo_discards_mixed_site_false_positive() {
        let study = ctgov_study_fixture(json!([
            {
                "facility": "University Hospitals Cleveland Medical Center",
                "city": "Cleveland",
                "country": "United States",
                "geoPoint": { "lat": 40.7128, "lon": -74.0060 }
            },
            {
                "facility": "Cleveland Clinic Taussig Cancer Center",
                "city": "Cleveland",
                "country": "United States",
                "geoPoint": { "lat": 41.4993, "lon": -81.6944 }
            }
        ]));

        assert!(!trial_matches_facility_geo(
            &study,
            "university hospitals",
            41.4993,
            -81.6944,
            50
        ));
    }

    #[test]
    fn facility_geo_keeps_same_site_match() {
        let study = ctgov_study_fixture(json!([
            {
                "facility": "University Hospitals Cleveland Medical Center",
                "city": "Cleveland",
                "country": "United States",
                "geoPoint": { "lat": 41.5031, "lon": -81.6208 }
            }
        ]));

        assert!(trial_matches_facility_geo(
            &study,
            "university hospitals",
            41.4993,
            -81.6944,
            50
        ));
    }
}
