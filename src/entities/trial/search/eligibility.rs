//! Trial eligibility, facility-geo, and age post-filter helpers.

use futures::{StreamExt, stream};
use regex::Regex;
use std::sync::OnceLock;
use tracing::warn;

use crate::sources::clinicaltrials::{ClinicalTrialsClient, CtGovLocation, CtGovStudy};

use super::super::{TRIAL_SECTION_ELIGIBILITY, TRIAL_SECTION_LOCATIONS, TrialSearchFilters};
use super::has_boolean_operators;

const DETAIL_VERIFY_CONCURRENCY: usize = 8;

fn normalize_facility_text(value: &str) -> Option<String> {
    let normalized = value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    (!normalized.is_empty()).then_some(normalized)
}

pub(crate) fn haversine_miles(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
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

pub(super) fn ctgov_nct_id(study: &CtGovStudy) -> Option<String> {
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
        "must have no",
        "no prior",
        "no previous",
        "not have received",
        "not have previously",
        "not received",
        "not previously received",
        "have not received",
        "should not have",
        "cannot have",
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

fn keyword_has_negative_inclusion_context(inclusion_text: &str, keyword: &str) -> bool {
    inclusion_text
        .split(['\n', '.', ';'])
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .filter(|segment| contains_keyword_tokens(segment, keyword))
        .any(contains_exclusion_language)
}

fn eligibility_keyword_in_inclusion(
    inclusion_text: &str,
    exclusion_text: &str,
    keyword: &str,
) -> bool {
    let keyword = keyword.trim().to_ascii_lowercase();
    if keyword.is_empty() {
        return true;
    }

    let inclusion_has_keyword = contains_keyword_tokens(inclusion_text, &keyword);

    if !exclusion_text.is_empty() {
        if inclusion_has_keyword && keyword_has_positive_inclusion_context(inclusion_text, &keyword)
        {
            return true;
        }
        if contains_keyword_tokens(exclusion_text, &keyword) {
            return false;
        }
        if inclusion_has_keyword {
            return false;
        }
        return true;
    }

    if !inclusion_has_keyword {
        return true;
    }
    !keyword_has_negative_inclusion_context(inclusion_text, &keyword)
}

pub(super) fn collect_eligibility_keywords(filters: &TrialSearchFilters) -> Vec<String> {
    let mut keywords = Vec::new();

    if let Some(mutation) = filters
        .mutation
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && !has_boolean_operators(mutation)
    {
        keywords.push(mutation.to_string());
    }

    if let Some(criteria) = filters
        .criteria
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && !has_boolean_operators(criteria)
    {
        keywords.push(criteria.to_string());
    }

    if let Some(prior_therapies) = filters
        .prior_therapies
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        keywords.push(prior_therapies.to_string());
    }

    if let Some(progression_on) = filters
        .progression_on
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        keywords.push(progression_on.to_string());
    }

    keywords
}

pub(super) async fn verify_detail_filters(
    client: &ClinicalTrialsClient,
    studies: Vec<CtGovStudy>,
    facility_geo: Option<(&str, f64, f64, u32)>,
    keywords: &[String],
) -> Vec<CtGovStudy> {
    let facility_geo = facility_geo.and_then(|(facility, lat, lon, distance)| {
        normalize_facility_text(facility).map(|facility| (facility, lat, lon, distance))
    });
    if facility_geo.is_none() && keywords.is_empty() {
        return studies;
    }

    let mut sections = Vec::new();
    if facility_geo.is_some() {
        sections.push(TRIAL_SECTION_LOCATIONS.to_string());
    }
    if !keywords.is_empty() {
        sections.push(TRIAL_SECTION_ELIGIBILITY.to_string());
    }

    let keywords = keywords.to_vec();
    let mut verification_stream = stream::iter(studies.into_iter().map(|study| {
        let nct_id = ctgov_nct_id(&study);
        let sections = sections.clone();
        let facility_geo = facility_geo.clone();
        let keywords = keywords.clone();
        async move {
            let Some(nct_id) = nct_id else {
                return Some(study);
            };
            let details = match client.get(&nct_id, &sections).await {
                Ok(details) => details,
                Err(e) => {
                    warn!(nct_id, error = %e, "trial detail fetch failed, keeping study");
                    return Some(study);
                }
            };

            if let Some((facility, lat, lon, distance)) = facility_geo
                && !trial_matches_facility_geo(&details, &facility, lat, lon, distance)
            {
                return None;
            }

            if keywords.is_empty() {
                return Some(study);
            }
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
                .all(|keyword| eligibility_keyword_in_inclusion(&inclusion, &exclusion, keyword))
                .then_some(study)
        }
    }))
    .buffered(DETAIL_VERIFY_CONCURRENCY);

    let mut verified = Vec::new();
    while let Some(maybe_study) = verification_stream.next().await {
        if let Some(study) = maybe_study {
            verified.push(study);
        }
    }
    verified
}

fn parse_age_years(value: &str) -> Option<f64> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<f64>().ok()?;
    let unit = parts.next().map(|token| {
        token
            .trim_matches(|c: char| !c.is_ascii_alphabetic())
            .to_ascii_lowercase()
    });

    match unit.as_deref() {
        None | Some("year") | Some("years") => Some(amount),
        Some("month") | Some("months") => Some(amount / 12.0),
        Some("week") | Some("weeks") => Some(amount / 52.0),
        Some("day") | Some("days") => Some(amount / 365.0),
        _ => None,
    }
}

pub(super) fn verify_age_eligibility(studies: Vec<CtGovStudy>, age: f64) -> Vec<CtGovStudy> {
    studies
        .into_iter()
        .filter(|study| {
            let module = study
                .protocol_section
                .as_ref()
                .and_then(|s| s.eligibility_module.as_ref());
            let min_ok = module
                .and_then(|m| m.minimum_age.as_deref())
                .and_then(parse_age_years)
                .is_none_or(|min| age >= min);
            let max_ok = module
                .and_then(|m| m.maximum_age.as_deref())
                .and_then(parse_age_years)
                .is_none_or(|max| age <= max);
            min_ok && max_ok
        })
        .collect()
}

#[cfg(test)]
mod tests;
