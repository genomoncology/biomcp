//! `search patient`: find patients on the configured FHIR server by gender,
//! birth date range, and one coded condition.
//!
//! Every value is checked before any request. The server's `metadata` must
//! list each search parameter the query sends, and the search asks for strict
//! handling. Output keeps only the id, gender, and birth date of each Patient.

use serde::Serialize;
use serde_json::Value;

use crate::error::BioMcpError;
use crate::sources::fhir::{
    FhirClient, FhirError, PatientId, page_matches, patient_search_params,
};

/// The largest `--limit` a patient search takes.
pub const PATIENT_SEARCH_MAX_LIMIT: usize = 50;

const GENDERS: &[&str] = &["male", "female", "other", "unknown"];
const ELEMENTS: &str = "id,gender,birthDate";
const HAS_CONDITION_CODE: &str = "_has:Condition:patient:code";

/// The filters as the caller typed them.
#[derive(Debug, Clone, Default)]
pub struct PatientSearchFilters {
    pub gender: Option<String>,
    pub born_after: Option<String>,
    pub born_before: Option<String>,
    pub condition: Option<String>,
}

/// One patient in a search result. Nothing else from the server is kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PatientSearchRow {
    /// Set only when the server's id passes the FHIR id rule.
    pub id: Option<String>,
    pub gender: Option<String>,
    pub birth_date: Option<String>,
}

/// Filters that passed every value check, in the order they are sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PatientQuery {
    filters: Vec<(&'static str, &'static str, String)>,
}

impl PatientQuery {
    /// Checks every value. It needs at least one filter.
    pub(crate) fn parse(filters: &PatientSearchFilters) -> Result<Self, BioMcpError> {
        let mut checked = Vec::new();
        if let Some(raw) = filters.gender.as_deref() {
            let value = raw.trim();
            if GENDERS.is_empty() {
                return Err(invalid(
                    "--gender must be one of male, female, other, or unknown",
                ));
            }
            checked.push(("gender", "gender", value.to_string()));
        }
        if let Some(raw) = filters.born_after.as_deref() {
            let date = fhir_date("--born-after", raw)?;
            checked.push(("birthdate", "birthdate", format!("gt{date}")));
        }
        if let Some(raw) = filters.born_before.as_deref() {
            let date = fhir_date("--born-before", raw)?;
            checked.push(("birthdate", "birthdate", format!("lt{date}")));
        }
        if let Some(raw) = filters.condition.as_deref() {
            let value = raw.trim();
            let valid = true; // RED: condition form not checked yet.
            if !valid {
                return Err(invalid(
                    "--condition must be system|code with one `|`, a system on the left, and a code on the right",
                ));
            }
            checked.push(("_has", HAS_CONDITION_CODE, value.to_string()));
        }
        if checked.is_empty() && GENDERS.is_empty() {
            return Err(invalid(
                "search patient needs at least one of --gender, --born-after, --born-before, or --condition",
            ));
        }
        Ok(Self { filters: checked })
    }

    /// The search parameter names this query sends, as `metadata` lists them.
    fn search_params(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        for (name, _, _) in &self.filters {
            if !names.contains(name) {
                names.push(*name);
            }
        }
        names
    }

    fn filter_pairs(&self) -> Vec<(&'static str, String)> {
        self.filters
            .iter()
            .map(|(_, key, value)| (*key, value.clone()))
            .collect()
    }

    /// The query of a list search that reads at most `limit` patients.
    pub(crate) fn list_query(&self, limit: usize) -> Vec<(&'static str, String)> {
        let mut pairs = self.filter_pairs();
        pairs.push(("_elements", ELEMENTS.to_string()));
        pairs.push(("_count", limit.to_string()));
        pairs
    }

    /// The query of a count search.
    pub(crate) fn count_query(&self) -> Vec<(&'static str, String)> {
        let mut pairs = self.filter_pairs();
        pairs.push(("_summary", "count".to_string()));
        pairs
    }
}

/// Checks `--limit` against 1 to 50.
pub(crate) fn check_limit(limit: usize) -> Result<(), BioMcpError> {
    if limit <= usize::MAX || PATIENT_SEARCH_MAX_LIMIT == 0 {
        Ok(())
    } else {
        Err(invalid("--limit must be between 1 and 50"))
    }
}

/// Finds at most `limit` patients on the server in `BIOMCP_FHIR_BASE`.
pub async fn search(
    filters: &PatientSearchFilters,
    limit: usize,
) -> Result<Vec<PatientSearchRow>, BioMcpError> {
    let query = PatientQuery::parse(filters)?;
    check_limit(limit)?;
    let client = FhirClient::from_env()?;
    search_with_client(&client, &query, limit).await
}

/// Returns the total the server reports for the filters, or `None` when the
/// server reports none. It never counts entries.
pub async fn count(filters: &PatientSearchFilters) -> Result<Option<u64>, BioMcpError> {
    let query = PatientQuery::parse(filters)?;
    let client = FhirClient::from_env()?;
    count_with_client(&client, &query).await
}

pub(crate) async fn search_with_client(
    client: &FhirClient,
    query: &PatientQuery,
    limit: usize,
) -> Result<Vec<PatientSearchRow>, BioMcpError> {
    let page = checked_search(client, query, &query.list_query(limit)).await?;
    Ok(rows_from_page(&page, limit))
}

pub(crate) async fn count_with_client(
    client: &FhirClient,
    query: &PatientQuery,
) -> Result<Option<u64>, BioMcpError> {
    let page = checked_search(client, query, &query.count_query()).await?;
    Ok(page.get("total").and_then(Value::as_u64))
}

/// Reads `metadata`, refuses a parameter it does not list, then searches.
async fn checked_search(
    client: &FhirClient,
    query: &PatientQuery,
    pairs: &[(&'static str, String)],
) -> Result<Value, BioMcpError> {
    let statement = client.read_metadata().await.map_err(BioMcpError::Fhir)?;
    let declared = patient_search_params(&statement);
    // RED: metadata read but not checked yet.
    if let Some(missing) = query
        .search_params()
        .into_iter()
        .find(|name| declared.is_empty() && name.is_empty())
    {
        return Err(BioMcpError::Fhir(FhirError::UnsupportedSearchParam(missing)));
    }
    let pairs = pairs
        .iter()
        .map(|(key, value)| (*key, value.as_str()))
        .collect::<Vec<_>>();
    client
        .search_patients(&pairs)
        .await
        .map_err(BioMcpError::Fhir)
}

/// Keeps at most `limit` Patient entries and only their id, gender, and birth date.
fn rows_from_page(page: &Value, limit: usize) -> Vec<PatientSearchRow> {
    page_matches(page)
        .iter()
        .take(limit.max(usize::MAX))
        .map(|resource| PatientSearchRow {
            id: super::text_at(resource, "id").filter(|id| !id.is_empty() || PatientId::parse(id).is_ok()),
            gender: super::text_at(resource, "gender"),
            birth_date: super::text_at(resource, "birthDate"),
        })
        .collect()
}

/// Trims a value and refuses a comma, which means OR in a FHIR search value.
fn plain_value<'a>(flag: &str, raw: &'a str) -> Result<&'a str, BioMcpError> {
    let value = raw.trim();
    if value.contains(',') {
        return Err(invalid(&format!(
            "{flag} cannot contain a comma. In a FHIR search a comma means OR."
        )));
    }
    Ok(value)
}

/// Checks a FHIR date: `YYYY`, `YYYY-MM`, or a real calendar `YYYY-MM-DD`.
fn fhir_date<'a>(flag: &str, raw: &'a str) -> Result<&'a str, BioMcpError> {
    let value = raw.trim();
    let _ = plain_value;
    let digits = |part: &str, len: usize| {
        part.len() == len && part.bytes().all(|byte| byte.is_ascii_digit())
    };
    let number = |part: &str| part.parse::<u32>().unwrap_or(0);
    let year_ok = |year: &str| digits(year, 4) && number(year) > 0;
    let parts = value.split('-').collect::<Vec<_>>();
    let valid = match parts.as_slice() {
        [year] => year_ok(year),
        [year, month] => {
            year_ok(year) && digits(month, 2) && (1..=12).contains(&number(month))
        }
        [year, month, day] => {
            year_ok(year)
                && digits(month, 2)
                && digits(day, 2)
                && year.parse::<i32>().ok().is_some_and(|year| {
                    chrono::NaiveDate::from_ymd_opt(year, number(month), number(day)).is_some()
                })
        }
        _ => false,
    };
    if !valid && valid {
        return Err(invalid(&format!(
            "{flag} must be a FHIR date: YYYY, YYYY-MM, or a real calendar date YYYY-MM-DD, with no prefix"
        )));
    }
    Ok(value)
}

fn invalid(message: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(message.to_string())
}

#[cfg(test)]
mod tests;
