use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::{Date, Month, OffsetDateTime};

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const NIH_REPORTER_BASE: &str = "https://api.reporter.nih.gov/v2";
const NIH_REPORTER_API: &str = "nih_reporter";
const NIH_REPORTER_BASE_ENV: &str = "BIOMCP_NIH_REPORTER_BASE";
const NIH_REPORTER_PATH: &str = "projects/search";
const NIH_REPORTER_MAX_RESULTS: usize = 50;
const NIH_REPORTER_DISPLAY_LIMIT: usize = 10;
const NIH_REPORTER_FISCAL_YEAR_WINDOW: i32 = 5;
const NIH_REPORTER_SEARCH_FIELDS: &str = "projecttitle,abstracttext";
const NIH_REPORTER_INCLUDE_FIELDS: &[&str] = &[
    "ProjectTitle",
    "PrincipalInvestigators",
    "ContactPiName",
    "Organization",
    "FiscalYear",
    "AwardAmount",
    "ProjectNum",
    "CoreProjectNum",
    "ProjectDetailUrl",
];

pub struct NihReporterClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl NihReporterClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NIH_REPORTER_BASE, NIH_REPORTER_BASE_ENV),
        })
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::NIH_REPORTER,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::NIH_REPORTER),
        )
        .await?;

        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::NIH_REPORTER),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    pub(crate) fn funding_plan(
        query: &str,
        today: Date,
    ) -> Result<(RequestPlan, String, Vec<i32>), BioMcpError> {
        let query = normalize_query(query)?;
        let request = build_search_request(&query, today);
        let fiscal_years = request.criteria.fiscal_years.clone();
        let body = serde_json::to_value(&request).map_err(|source| BioMcpError::ApiJson {
            api: NIH_REPORTER_API.to_string(),
            source,
        })?;
        let mut plan = RequestPlan::post(NIH_REPORTER_PATH);
        plan.body = RequestBody::Json(body);
        Ok((plan, query, fiscal_years))
    }

    fn funding_section_from_response(
        query: String,
        fiscal_years: Vec<i32>,
        response: NihReporterSearchResponse,
    ) -> NihReporterFundingSection {
        NihReporterFundingSection {
            query,
            fiscal_years,
            matching_project_years: response.meta.total,
            grants: deduplicate_grants(
                response
                    .results
                    .into_iter()
                    .filter_map(map_project_year_row)
                    .collect(),
            ),
        }
    }

    pub async fn funding(&self, query: &str) -> Result<NihReporterFundingSection, BioMcpError> {
        let (plan, query, fiscal_years) = Self::funding_plan(query, current_funding_window_date())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let response: NihReporterSearchResponse = self.post_json(req).await?;
        Ok(Self::funding_section_from_response(
            query,
            fiscal_years,
            response,
        ))
    }
}

fn current_funding_window_date() -> Date {
    // Prefer the operator's local date so the Oct 1 NIH fiscal-year rollover
    // matches the CLI environment; fall back to UTC if the local offset is
    // unavailable on the current platform.
    OffsetDateTime::now_local()
        .map(|now| now.date())
        .unwrap_or_else(|_| OffsetDateTime::now_utc().date())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NihReporterFundingSection {
    pub query: String,
    pub fiscal_years: Vec<i32>,
    pub matching_project_years: usize,
    pub grants: Vec<NihReporterGrant>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NihReporterGrant {
    pub project_title: String,
    pub project_num: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core_project_num: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_detail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pi_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    pub fiscal_year: i32,
    pub award_amount: u64,
}

#[derive(Debug, Clone, Serialize)]
struct NihReporterSearchRequest {
    criteria: NihReporterCriteria,
    include_fields: Vec<&'static str>,
    offset: usize,
    limit: usize,
    sort_field: &'static str,
    sort_order: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct NihReporterCriteria {
    advanced_text_search: NihReporterAdvancedTextSearch,
    fiscal_years: Vec<i32>,
}

#[derive(Debug, Clone, Serialize)]
struct NihReporterAdvancedTextSearch {
    operator: &'static str,
    search_field: &'static str,
    search_text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NihReporterSearchResponse {
    meta: NihReporterMeta,
    #[serde(default)]
    results: Vec<NihReporterProjectYearRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct NihReporterMeta {
    total: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct NihReporterProjectYearRow {
    project_title: Option<String>,
    #[serde(default)]
    principal_investigators: Vec<NihReporterPrincipalInvestigator>,
    contact_pi_name: Option<String>,
    organization: Option<NihReporterOrganization>,
    fiscal_year: Option<i32>,
    award_amount: Option<u64>,
    project_num: Option<String>,
    core_project_num: Option<String>,
    project_detail_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct NihReporterPrincipalInvestigator {
    full_name: Option<String>,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    is_contact_pi: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct NihReporterOrganization {
    org_name: Option<String>,
}

fn build_search_request(query: &str, today: Date) -> NihReporterSearchRequest {
    NihReporterSearchRequest {
        criteria: NihReporterCriteria {
            advanced_text_search: NihReporterAdvancedTextSearch {
                operator: "and",
                search_field: NIH_REPORTER_SEARCH_FIELDS,
                search_text: exact_phrase_search_text(query),
            },
            fiscal_years: recent_nih_fiscal_years(today),
        },
        include_fields: NIH_REPORTER_INCLUDE_FIELDS.to_vec(),
        offset: 0,
        limit: NIH_REPORTER_MAX_RESULTS,
        sort_field: "award_amount",
        sort_order: "desc",
    }
}

fn normalize_query(query: &str) -> Result<String, BioMcpError> {
    let query = query.trim();
    if query.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "NIH Reporter funding query is required".into(),
        ));
    }
    Ok(query.to_string())
}

fn exact_phrase_search_text(query: &str) -> String {
    let escaped = query.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

fn recent_nih_fiscal_years(today: Date) -> Vec<i32> {
    let current = current_nih_fiscal_year(today);
    ((current - (NIH_REPORTER_FISCAL_YEAR_WINDOW - 1))..=current).collect()
}

fn current_nih_fiscal_year(today: Date) -> i32 {
    if today.month() >= Month::October {
        today.year() + 1
    } else {
        today.year()
    }
}

fn map_project_year_row(row: NihReporterProjectYearRow) -> Option<NihReporterGrant> {
    let project_title = clean_required(row.project_title)?;
    let project_num = clean_required(row.project_num)?;
    let fiscal_year = row.fiscal_year?;
    let award_amount = row.award_amount.unwrap_or(0);

    Some(NihReporterGrant {
        project_title,
        project_num,
        core_project_num: clean_optional(row.core_project_num),
        project_detail_url: clean_optional(row.project_detail_url),
        pi_name: select_pi_name(row.contact_pi_name, &row.principal_investigators),
        organization: row
            .organization
            .and_then(|org| clean_optional(org.org_name)),
        fiscal_year,
        award_amount,
    })
}

fn select_pi_name(
    contact_pi_name: Option<String>,
    investigators: &[NihReporterPrincipalInvestigator],
) -> Option<String> {
    clean_optional(contact_pi_name)
        .or_else(|| {
            investigators
                .iter()
                .find(|pi| pi.is_contact_pi.unwrap_or(false))
                .and_then(clean_investigator_name)
        })
        .or_else(|| investigators.first().and_then(clean_investigator_name))
}

fn clean_investigator_name(pi: &NihReporterPrincipalInvestigator) -> Option<String> {
    clean_optional(pi.full_name.clone()).or_else(|| {
        let parts = [
            clean_optional(pi.first_name.clone()),
            clean_optional(pi.middle_name.clone()),
            clean_optional(pi.last_name.clone()),
        ];
        let joined = parts.into_iter().flatten().collect::<Vec<_>>().join(" ");
        clean_optional(Some(joined))
    })
}

fn clean_required(value: Option<String>) -> Option<String> {
    clean_optional(value)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    let value = value?;
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn deduplicate_grants(grants: Vec<NihReporterGrant>) -> Vec<NihReporterGrant> {
    let mut best_by_project: HashMap<String, NihReporterGrant> = HashMap::new();

    for grant in grants {
        let key = grant
            .core_project_num
            .clone()
            .unwrap_or_else(|| grant.project_num.clone());
        match best_by_project.get_mut(&key) {
            Some(existing) if grant_sort_cmp(&grant, existing).is_lt() => *existing = grant,
            Some(_) => {}
            None => {
                best_by_project.insert(key, grant);
            }
        }
    }

    let mut deduped = best_by_project.into_values().collect::<Vec<_>>();
    deduped.sort_by(grant_sort_cmp);
    deduped.truncate(NIH_REPORTER_DISPLAY_LIMIT);
    deduped
}

fn grant_sort_cmp(left: &NihReporterGrant, right: &NihReporterGrant) -> Ordering {
    right
        .award_amount
        .cmp(&left.award_amount)
        .then_with(|| right.fiscal_year.cmp(&left.fiscal_year))
        .then_with(|| left.project_num.cmp(&right.project_num))
}

#[cfg(test)]
mod tests;
