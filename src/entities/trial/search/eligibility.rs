//! Trial eligibility, facility-geo, and age post-filter helpers.

use futures::{StreamExt, stream};
use regex::Regex;
use std::sync::OnceLock;
use tracing::warn;

use crate::sources::clinicaltrials::ClinicalTrialsClient;

use super::super::{TRIAL_SECTION_ELIGIBILITY, TRIAL_SECTION_LOCATIONS};

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
    location: &biodata::ClinicalTrialSite,
    facility_needle: &str,
    origin_lat: f64,
    origin_lon: f64,
    max_distance_miles: u32,
) -> bool {
    let Some(location_facility) = location.facility().and_then(normalize_facility_text) else {
        return false;
    };
    if !location_facility.contains(facility_needle) {
        return false;
    }
    let Some(geo) = location.coordinates() else {
        return false;
    };
    haversine_miles(origin_lat, origin_lon, geo.latitude(), geo.longitude())
        <= max_distance_miles as f64
}

pub(super) fn ctgov_nct_id(study: &biodata::ClinicalTrialsGovApiV2SearchResult) -> Option<String> {
    study
        .projection()
        .value()
        .identities()
        .first()
        .map(|identity| identity.identifier().to_owned())
}

fn trial_matches_facility_geo(
    response: &biodata::ClinicalTrialsGovApiV2Response,
    facility_needle: &str,
    origin_lat: f64,
    origin_lon: f64,
    max_distance_miles: u32,
) -> bool {
    match response.site_directory() {
        biodata::ClinicalTrialSection::Present(directory) => directory
            .sites()
            .unwrap_or_default()
            .iter()
            .any(|location| {
                location_matches_facility_geo(
                    location,
                    facility_needle,
                    origin_lat,
                    origin_lon,
                    max_distance_miles,
                )
            }),
        _ => false,
    }
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

pub(super) fn collect_eligibility_keywords(
    filters: &biodata::ClinicalTrialSearchFilters,
) -> Vec<String> {
    let mut keywords = Vec::new();

    if let Some(mutation) = filters.mutation()
        && !filters.mutation_has_boolean_operators()
    {
        keywords.push(mutation.to_string());
    }

    if let Some(criteria) = filters.criteria()
        && !filters.criteria_has_boolean_operators()
    {
        keywords.push(criteria.to_string());
    }

    if let Some(prior_therapies) = filters.prior_therapies() {
        keywords.push(prior_therapies.to_string());
    }

    if let Some(progression_on) = filters.progression_on() {
        keywords.push(progression_on.to_string());
    }

    keywords
}

pub(super) async fn verify_detail_filters(
    client: &ClinicalTrialsClient,
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    facility_geo: Option<(&str, f64, f64, u32)>,
    keywords: &[String],
) -> DetailFilterOutcome {
    let facility_geo = facility_geo.and_then(|(facility, lat, lon, distance)| {
        normalize_facility_text(facility).map(|facility| (facility, lat, lon, distance))
    });
    if facility_geo.is_none() && keywords.is_empty() {
        return DetailFilterOutcome {
            studies,
            incomplete: false,
        };
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
                return (Some(study), true);
            };
            let details = match client.get_biodata_detail(&nct_id, &sections).await {
                Ok(details) => details,
                Err(e) => {
                    warn!(nct_id, error = %e, "trial detail fetch failed, keeping study");
                    return (Some(study), true);
                }
            };

            if let Some((facility, lat, lon, distance)) = facility_geo {
                if !matches!(
                    details.locations_state(),
                    biodata::ClinicalTrialSection::Present(())
                ) {
                    warn!(
                        nct_id,
                        "missing location evidence in detail fetch, keeping study"
                    );
                    return (Some(study), true);
                }
                if !trial_matches_facility_geo(&details, &facility, lat, lon, distance) {
                    return (None, false);
                }
            }

            if keywords.is_empty() {
                return (Some(study), false);
            }
            let Some(criteria) = (match details.eligibility() {
                biodata::ClinicalTrialSection::Present(value) => value.registry_text(),
                _ => None,
            })
            .map(str::trim)
            .filter(|value| !value.is_empty()) else {
                warn!(
                    nct_id,
                    "missing eligibility criteria in detail fetch, keeping study"
                );
                return (Some(study), true);
            };

            let (inclusion, exclusion) = split_eligibility_sections(criteria);
            let keep = keywords
                .iter()
                .all(|keyword| eligibility_keyword_in_inclusion(&inclusion, &exclusion, keyword))
                .then_some(study);
            (keep, false)
        }
    }))
    .buffered(DETAIL_VERIFY_CONCURRENCY);

    let mut verified = Vec::new();
    let mut incomplete = false;
    while let Some((maybe_study, decision_incomplete)) = verification_stream.next().await {
        incomplete |= decision_incomplete;
        if let Some(study) = maybe_study {
            verified.push(study);
        }
    }
    DetailFilterOutcome {
        studies: verified,
        incomplete,
    }
}

pub(super) struct DetailFilterOutcome {
    pub(super) studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    pub(super) incomplete: bool,
}

pub(super) fn verify_age_eligibility(
    studies: Vec<biodata::ClinicalTrialsGovApiV2SearchResult>,
    age: f64,
) -> Vec<biodata::ClinicalTrialsGovApiV2SearchResult> {
    studies
        .into_iter()
        .filter(|study| {
            let age_range = study.projection().value().age_range();
            let min_ok = age_range
                .and_then(|value| value.minimum())
                .and_then(comparable_years)
                .is_none_or(|min| age >= min);
            let max_ok = age_range
                .and_then(|value| value.maximum())
                .filter(|bound| bound.form() == biodata::ClinicalTrialAgeBoundForm::Limited)
                .and_then(comparable_years)
                .is_none_or(|max| age <= max);
            min_ok && max_ok
        })
        .collect()
}

fn comparable_years(bound: &biodata::ClinicalTrialAgeBound) -> Option<f64> {
    let quantity = bound.source().source_quantity().parse::<f64>().ok()?;
    match bound.source().source_unit() {
        biodata::DurationUnit::Years => Some(quantity),
        biodata::DurationUnit::Months => Some(quantity / 12.0),
        biodata::DurationUnit::Weeks => Some(quantity / 52.0),
        biodata::DurationUnit::Days => Some(quantity / 365.0),
    }
}

#[cfg(test)]
mod tests;
