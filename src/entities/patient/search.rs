//! `search patient`: typed facts against the operator's FHIR server.
//!
//! Filters are validated here, once, before any request. The capability
//! check reads `metadata` and refuses a parameter the server does not
//! declare, so a filter is never silently dropped from a cohort.

use serde_json::Value;

use crate::entities::patient::text_at;
use crate::error::BioMcpError;
use crate::sources::fhir::{FhirClient, FhirError, PatientId, array};

/// One coded condition filter: a non-empty system and code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionFilter {
    pub(crate) system: String,
    pub(crate) code: String,
}

/// The typed facts a search sends, already validated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PatientSearchFilters {
    pub(crate) gender: Option<&'static str>,
    pub(crate) born_after: Option<String>,
    pub(crate) born_before: Option<String>,
    pub(crate) condition: Option<ConditionFilter>,
}

/// One output row: only the identity facts the command keeps.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct PatientSearchRow {
    /// Present only when the id passes the FHIR id rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
}

/// What one search read: rows, or the count the server reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatientSearchOutcome {
    Rows(Vec<PatientSearchRow>),
    /// `Bundle.total` verbatim; `None` when the server sent no count.
    Count(Option<u64>),
}

const GENDERS: [&str; 4] = ["male", "female", "other", "unknown"];
const ELEMENTS: &str = "id,gender,birthDate";
const COMMA_MESSAGE: &str = "a comma is not allowed in a patient search filter value";
const NO_FILTER_MESSAGE: &str =
    "search patient needs at least one of --gender, --born-after, --born-before, or --condition";
const CONDITION_MESSAGE: &str =
    "--condition takes system|code with a non-empty system and a non-empty code";

/// Checks every filter value and requires one filter. Runs before any
/// request.
pub(crate) fn parse_filters(
    gender: Option<&str>,
    born_after: Option<&str>,
    born_before: Option<&str>,
    condition: Option<&str>,
) -> Result<PatientSearchFilters, BioMcpError> {
    let filters = PatientSearchFilters {
        gender: gender.map(parse_gender).transpose()?,
        born_after: born_after
            .map(|value| parse_fhir_date("--born-after", value))
            .transpose()?,
        born_before: born_before
            .map(|value| parse_fhir_date("--born-before", value))
            .transpose()?,
        condition: condition.map(parse_condition).transpose()?,
    };
    let bare = filters.gender.is_none()
        && filters.born_after.is_none()
        && filters.born_before.is_none()
        && filters.condition.is_none();
    if bare {
        return Err(BioMcpError::InvalidArgument(NO_FILTER_MESSAGE.into()));
    }
    Ok(filters)
}

fn parse_gender(raw: &str) -> Result<&'static str, BioMcpError> {
    let text = raw.trim();
    GENDERS
        .iter()
        .find(|gender| **gender == text)
        .copied()
        .ok_or_else(|| {
            BioMcpError::InvalidArgument(
                "--gender takes one of: male, female, other, unknown".into(),
            )
        })
}

/// A FHIR date with no prefix of its own: `YYYY`, `YYYY-MM`, or
/// `YYYY-MM-DD`, and a real calendar date.
fn parse_fhir_date(flag: &str, raw: &str) -> Result<String, BioMcpError> {
    let text = raw.trim();
    if text.contains(',') {
        return Err(BioMcpError::InvalidArgument(COMMA_MESSAGE.into()));
    }
    let shape_error = || {
        BioMcpError::InvalidArgument(format!(
            "{flag} takes a FHIR date: YYYY, YYYY-MM, or YYYY-MM-DD"
        ))
    };
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() > 3
        || parts.iter().enumerate().any(|(index, part)| {
            part.len() != if index == 0 { 4 } else { 2 }
                || !part.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return Err(shape_error());
    }
    let year: i64 = parts[0].parse().map_err(|_| shape_error())?;
    let month: u32 = if parts.len() > 1 {
        parts[1].parse().map_err(|_| shape_error())?
    } else {
        1
    };
    let day: u32 = if parts.len() > 2 {
        parts[2].parse().map_err(|_| shape_error())?
    } else {
        1
    };
    let month_ok = (1..=12).contains(&month);
    let day_ok = day >= 1 && day <= days_in_month(year, month);
    if !month_ok || !day_ok {
        return Err(shape_error());
    }
    Ok(text.to_string())
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        _ => 28,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn parse_condition(raw: &str) -> Result<ConditionFilter, BioMcpError> {
    let text = raw.trim();
    let Some((system, code)) = text.split_once('|') else {
        return Err(BioMcpError::InvalidArgument(CONDITION_MESSAGE.into()));
    };
    if system.contains(',') || code.contains(',') {
        return Err(BioMcpError::InvalidArgument(COMMA_MESSAGE.into()));
    }
    if system.is_empty() || code.is_empty() || text.matches('|').count() != 1 {
        return Err(BioMcpError::InvalidArgument(CONDITION_MESSAGE.into()));
    }
    Ok(ConditionFilter {
        system: system.to_string(),
        code: code.to_string(),
    })
}

/// Reads `metadata`, refuses any parameter the server does not declare for
/// Patient, then runs one search page under strict handling.
pub async fn search(
    filters: &PatientSearchFilters,
    count: bool,
    limit: usize,
) -> Result<PatientSearchOutcome, BioMcpError> {
    let client = FhirClient::from_env()?;
    search_with_client(&client, filters, count, limit).await
}

pub(crate) async fn search_with_client(
    client: &FhirClient,
    filters: &PatientSearchFilters,
    count: bool,
    limit: usize,
) -> Result<PatientSearchOutcome, BioMcpError> {
    let capability = client.read_metadata().await.map_err(BioMcpError::Fhir)?;
    for name in required_search_params(filters) {
        if !declares_search_param(&capability, name) {
            return Err(BioMcpError::Fhir(FhirError::UnsupportedSearchParam(name)));
        }
    }
    let owned = build_query(filters, count, limit);
    let query: Vec<(&str, &str)> = owned
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    let bundle = client
        .search_patients(&query)
        .await
        .map_err(BioMcpError::Fhir)?;
    Ok(if count {
        PatientSearchOutcome::Count(total_from_bundle(&bundle))
    } else {
        PatientSearchOutcome::Rows(rows_from_bundle(&bundle, limit))
    })
}

/// The search parameters this run would send, in flag order. `_elements`,
/// `_count`, and `_summary` are result controls, not search parameters, so
/// the capability check never looks for them.
fn required_search_params(filters: &PatientSearchFilters) -> Vec<&'static str> {
    let mut names = Vec::new();
    if filters.gender.is_some() {
        names.push("gender");
    }
    if filters.born_after.is_some() || filters.born_before.is_some() {
        names.push("birthdate");
    }
    if filters.condition.is_some() {
        names.push("_has");
    }
    names
}

/// True when the server-mode Patient resource declares `name` in its
/// `searchParam` list.
fn declares_search_param(capability: &Value, name: &str) -> bool {
    array(capability.get("rest"))
        .filter(|rest| rest.get("mode").and_then(Value::as_str) == Some("server"))
        .filter_map(|rest| rest.get("resource").and_then(Value::as_array))
        .flatten()
        .find(|resource| resource.get("type").and_then(Value::as_str) == Some("Patient"))
        .and_then(|resource| resource.get("searchParam").and_then(Value::as_array))
        .into_iter()
        .flatten()
        .any(|param| param.get("name").and_then(Value::as_str) == Some(name))
}

/// The one query string builder: filters in flag order, then the result
/// controls. `gt` and `lt` are the only prefixes sent.
pub(crate) fn build_query(
    filters: &PatientSearchFilters,
    count: bool,
    limit: usize,
) -> Vec<(&'static str, String)> {
    let mut query: Vec<(&'static str, String)> = Vec::new();
    if let Some(gender) = filters.gender {
        query.push(("gender", gender.to_string()));
    }
    if let Some(date) = &filters.born_after {
        query.push(("birthdate", format!("gt{date}")));
    }
    if let Some(date) = &filters.born_before {
        query.push(("birthdate", format!("lt{date}")));
    }
    if let Some(condition) = &filters.condition {
        query.push((
            "_has:Condition:patient:code",
            format!("{}|{}", condition.system, condition.code),
        ));
    }
    if count {
        query.push(("_summary", "count".to_string()));
    } else {
        query.push(("_elements", ELEMENTS.to_string()));
        query.push(("_count", limit.to_string()));
    }
    query
}

/// Keeps Patient entries only, their three identity fields only, and at
/// most `limit` rows even when the server ignored `_count`. An id that
/// fails the FHIR id rule prints no id and no next command.
fn rows_from_bundle(bundle: &Value, limit: usize) -> Vec<PatientSearchRow> {
    array(bundle.get("entry"))
        .filter_map(|entry| entry.get("resource"))
        .filter(|resource| resource.get("resourceType").and_then(Value::as_str) == Some("Patient"))
        .take(limit)
        .map(|resource| PatientSearchRow {
            id: text_at(resource, "id").filter(|id| PatientId::parse(id).is_ok()),
            gender: text_at(resource, "gender"),
            birth_date: text_at(resource, "birthDate"),
        })
        .collect()
}

/// `Bundle.total` verbatim, or `None` when the server sent none.
fn total_from_bundle(bundle: &Value) -> Option<u64> {
    bundle.get("total").and_then(Value::as_u64)
}
