#![allow(dead_code)]

use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, LOCATION, RETRY_AFTER};
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, apply_no_store, request_from_plan};

pub(crate) const ORCID_BASE: &str = "https://pub.orcid.org/v3.0";
const ORCID_BASE_ENV: &str = "BIOMCP_ORCID_BASE";
const ORCID_API: &str = "orcid";
const ORCID_MEDIA_TYPE: &str = "application/vnd.orcid+json";
const RETRY_AFTER_MAX_CHARS: usize = 128;
const MAX_REDIRECTS: usize = 10;

#[derive(Clone)]
pub(crate) struct OrcidClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl OrcidClient {
    pub(crate) fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::orcid_shared_client()?,
            base: crate::sources::env_base(ORCID_BASE, ORCID_BASE_ENV),
        })
    }

    pub(crate) fn record_plan(orcid: &str) -> Result<RequestPlan, BioMcpError> {
        let orcid = validate_orcid(orcid)?;
        Ok(RequestPlan::get(format!("{orcid}/record")).header("Accept", ORCID_MEDIA_TYPE))
    }

    pub(crate) fn works_plan(orcid: &str) -> Result<RequestPlan, BioMcpError> {
        let orcid = validate_orcid(orcid)?;
        Ok(RequestPlan::get(format!("{orcid}/works")).header("Accept", ORCID_MEDIA_TYPE))
    }

    pub(crate) async fn record(
        &self,
        orcid: &str,
    ) -> Result<OrcidFetchOutcome<OrcidRecord>, BioMcpError> {
        let plan = Self::record_plan(orcid)?;
        let response = self.send(&plan).await?;
        self.decode_record_response(orcid, response).await
    }

    pub(crate) async fn works(
        &self,
        orcid: &str,
    ) -> Result<OrcidFetchOutcome<OrcidWorks>, BioMcpError> {
        let plan = Self::works_plan(orcid)?;
        let response = self.send(&plan).await?;
        self.decode_works_response(orcid, response).await
    }

    async fn send(&self, plan: &RequestPlan) -> Result<reqwest::Response, BioMcpError> {
        let mut response = apply_no_store(request_from_plan(&self.client, &self.base, plan))
            .send()
            .await?;
        let origin = response.url().clone();
        let mut redirects = 0;

        while follows_redirect(response.status()) {
            let Some(location) = response.headers().get(LOCATION) else {
                break;
            };
            if redirects == MAX_REDIRECTS {
                return Err(malformed("ORCID redirect limit exceeded"));
            }
            let location = location
                .to_str()
                .map_err(|_| malformed("ORCID redirect Location was invalid"))?;
            let target = response
                .url()
                .join(location)
                .map_err(|_| malformed("ORCID redirect Location was invalid"))?;
            if !same_origin(&origin, &target) {
                return Err(malformed("ORCID redirect cannot leave the original origin"));
            }

            drop(response);
            response = apply_no_store(self.client.get(target).header("Accept", ORCID_MEDIA_TYPE))
                .send()
                .await?;
            redirects += 1;
        }

        Ok(response)
    }

    async fn decode_record_response(
        &self,
        requested_orcid: &str,
        response: reqwest::Response,
    ) -> Result<OrcidFetchOutcome<OrcidRecord>, BioMcpError> {
        let status = response.status();
        let final_url = response.url().clone();
        let headers = response.headers().clone();
        if let Some(outcome) = classify_status(requested_orcid, status, &headers)? {
            return Ok(outcome);
        }
        let bytes = match crate::sources::read_limited_body(response, ORCID_API).await {
            Ok(bytes) => bytes,
            Err(BioMcpError::BodyLimit { max_bytes, .. }) => {
                return Ok(OrcidFetchOutcome::Unavailable {
                    requested_orcid: requested_orcid.to_string(),
                    reason: OrcidUnavailableReason::BodyLimit { max_bytes },
                });
            }
            Err(error) => return Err(error),
        };
        decode_record_with_base(
            requested_orcid,
            final_url.as_str(),
            self.base.as_ref(),
            status,
            &headers,
            &bytes,
        )
    }

    async fn decode_works_response(
        &self,
        requested_orcid: &str,
        response: reqwest::Response,
    ) -> Result<OrcidFetchOutcome<OrcidWorks>, BioMcpError> {
        let status = response.status();
        let final_url = response.url().clone();
        let headers = response.headers().clone();
        if let Some(outcome) = classify_status(requested_orcid, status, &headers)? {
            return Ok(outcome);
        }
        let bytes = match crate::sources::read_limited_body(response, ORCID_API).await {
            Ok(bytes) => bytes,
            Err(BioMcpError::BodyLimit { max_bytes, .. }) => {
                return Ok(OrcidFetchOutcome::Unavailable {
                    requested_orcid: requested_orcid.to_string(),
                    reason: OrcidUnavailableReason::BodyLimit { max_bytes },
                });
            }
            Err(error) => return Err(error),
        };
        decode_works_with_base(
            requested_orcid,
            final_url.as_str(),
            self.base.as_ref(),
            status,
            &headers,
            &bytes,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum OrcidFetchOutcome<T> {
    Available {
        requested_orcid: String,
        canonical_orcid: String,
        data: T,
    },
    Redirected {
        requested_orcid: String,
        canonical_orcid: String,
        data: T,
    },
    NotFound {
        requested_orcid: String,
    },
    RateLimited {
        requested_orcid: String,
        retry_after: Option<String>,
    },
    Unavailable {
        requested_orcid: String,
        reason: OrcidUnavailableReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum OrcidUnavailableReason {
    ServerStatus { status: u16 },
    BodyLimit { max_bytes: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidRecord {
    pub modified_date: Option<i64>,
    pub names: Vec<OrcidName>,
    pub employments: Vec<OrcidEmployment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidName {
    pub given_names: Option<String>,
    pub family_name: Option<String>,
    pub credit_name: Option<String>,
    pub visibility: String,
    pub source: Option<OrcidSource>,
    pub created_date: Option<i64>,
    pub modified_date: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidEmployment {
    pub organization: OrcidOrganization,
    pub department_name: Option<String>,
    pub role_title: Option<String>,
    pub start_date: Option<OrcidPartialDate>,
    pub end_date: Option<OrcidPartialDate>,
    pub put_code: Option<i64>,
    pub visibility: String,
    pub source: Option<OrcidSource>,
    pub created_date: Option<i64>,
    pub modified_date: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidOrganization {
    pub name: String,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub disambiguated_identifier: Option<String>,
    pub disambiguation_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidPartialDate {
    pub year: Option<String>,
    pub month: Option<String>,
    pub day: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidSource {
    pub source_orcid: Option<String>,
    pub source_name: Option<String>,
    pub assertion_origin_orcid: Option<String>,
    pub assertion_origin_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidWorks {
    pub groups: Vec<OrcidWorkGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidWorkGroup {
    pub external_ids: Vec<OrcidExternalId>,
    pub summaries: Vec<OrcidWorkSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidWorkSummary {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub translated_title: Option<String>,
    pub translated_title_language: Option<String>,
    pub work_type: Option<String>,
    pub external_ids: Vec<OrcidExternalId>,
    pub put_code: Option<i64>,
    pub visibility: String,
    pub source: Option<OrcidSource>,
    pub created_date: Option<i64>,
    pub modified_date: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OrcidExternalId {
    pub external_id_type: Option<String>,
    pub external_id_value: Option<String>,
    pub external_id_relationship: Option<String>,
    pub normalized_value: Option<String>,
    pub normalized_url: Option<String>,
}

fn follows_redirect(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}

fn same_origin(origin: &reqwest::Url, target: &reqwest::Url) -> bool {
    origin.scheme() == target.scheme()
        && origin.host_str() == target.host_str()
        && origin.port_or_known_default() == target.port_or_known_default()
}

fn validate_orcid(orcid: &str) -> Result<&str, BioMcpError> {
    let bytes = orcid.as_bytes();
    let shape = bytes.len() == 19
        && bytes[4] == b'-'
        && bytes[9] == b'-'
        && bytes[14] == b'-'
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 9 | 14 => true,
            18 => byte.is_ascii_digit() || *byte == b'X',
            _ => byte.is_ascii_digit(),
        });
    if !shape {
        return Err(BioMcpError::InvalidArgument(
            "ORCID iD must use checksummed 0000-0000-0000-0000 format".into(),
        ));
    }

    let digits = bytes.iter().copied().filter(u8::is_ascii_digit).take(15);
    let total = digits.fold(0_u32, |total, digit| (total + u32::from(digit - b'0')) * 2);
    let result = (12 - (total % 11)) % 11;
    let expected = if result == 10 {
        b'X'
    } else {
        b'0' + result as u8
    };
    if bytes[18] != expected {
        return Err(BioMcpError::InvalidArgument(
            "ORCID iD checksum is invalid".into(),
        ));
    }
    Ok(orcid)
}

#[cfg(test)]
fn decode_record(
    requested_orcid: &str,
    final_url: &str,
    status: StatusCode,
    headers: &HeaderMap,
    bytes: &[u8],
) -> Result<OrcidFetchOutcome<OrcidRecord>, BioMcpError> {
    decode_record_with_base(
        requested_orcid,
        final_url,
        ORCID_BASE,
        status,
        headers,
        bytes,
    )
}

fn decode_record_with_base(
    requested_orcid: &str,
    final_url: &str,
    base: &str,
    status: StatusCode,
    headers: &HeaderMap,
    bytes: &[u8],
) -> Result<OrcidFetchOutcome<OrcidRecord>, BioMcpError> {
    if let Some(outcome) = classify_status(requested_orcid, status, headers)? {
        return Ok(outcome);
    }
    ensure_orcid_media_type(headers)?;
    let wire: RecordWire =
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: ORCID_API.into(),
            source,
        })?;
    let canonical_orcid = final_orcid(final_url, base, "record")?;
    let decoded_orcid = wire
        .orcid_identifier
        .path
        .as_deref()
        .ok_or_else(|| malformed("record omitted orcid-identifier.path"))?;
    validate_orcid(decoded_orcid)?;
    if decoded_orcid != canonical_orcid {
        return Err(malformed("final URL and decoded record ORCID disagree"));
    }
    let data = map_record(wire);
    Ok(success_outcome(requested_orcid, canonical_orcid, data))
}

#[cfg(test)]
fn decode_works(
    requested_orcid: &str,
    final_url: &str,
    status: StatusCode,
    headers: &HeaderMap,
    bytes: &[u8],
) -> Result<OrcidFetchOutcome<OrcidWorks>, BioMcpError> {
    decode_works_with_base(
        requested_orcid,
        final_url,
        ORCID_BASE,
        status,
        headers,
        bytes,
    )
}

fn decode_works_with_base(
    requested_orcid: &str,
    final_url: &str,
    base: &str,
    status: StatusCode,
    headers: &HeaderMap,
    bytes: &[u8],
) -> Result<OrcidFetchOutcome<OrcidWorks>, BioMcpError> {
    if let Some(outcome) = classify_status(requested_orcid, status, headers)? {
        return Ok(outcome);
    }
    ensure_orcid_media_type(headers)?;
    let wire: WorksWire = serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: ORCID_API.into(),
        source,
    })?;
    let canonical_orcid = final_orcid(final_url, base, "works")?;
    let data = map_works(wire);
    Ok(success_outcome(requested_orcid, canonical_orcid, data))
}

fn classify_status<T>(
    requested_orcid: &str,
    status: StatusCode,
    headers: &HeaderMap,
) -> Result<Option<OrcidFetchOutcome<T>>, BioMcpError> {
    if status == StatusCode::NOT_FOUND {
        return Ok(Some(OrcidFetchOutcome::NotFound {
            requested_orcid: requested_orcid.into(),
        }));
    }
    if status == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = headers
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(RETRY_AFTER_MAX_CHARS).collect());
        return Ok(Some(OrcidFetchOutcome::RateLimited {
            requested_orcid: requested_orcid.into(),
            retry_after,
        }));
    }
    if status.is_server_error() {
        return Ok(Some(OrcidFetchOutcome::Unavailable {
            requested_orcid: requested_orcid.into(),
            reason: OrcidUnavailableReason::ServerStatus {
                status: status.as_u16(),
            },
        }));
    }
    if !status.is_success() {
        return Err(malformed(&format!("unexpected HTTP status {status}")));
    }
    Ok(None)
}

fn success_outcome<T>(requested: &str, canonical: String, data: T) -> OrcidFetchOutcome<T> {
    if requested == canonical {
        OrcidFetchOutcome::Available {
            requested_orcid: requested.into(),
            canonical_orcid: canonical,
            data,
        }
    } else {
        OrcidFetchOutcome::Redirected {
            requested_orcid: requested.into(),
            canonical_orcid: canonical,
            data,
        }
    }
}

fn ensure_orcid_media_type(headers: &HeaderMap) -> Result<(), BioMcpError> {
    let raw = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| malformed("successful response omitted Content-Type"))?;
    let media_type = raw.split(';').next().unwrap_or_default().trim();
    if !media_type.eq_ignore_ascii_case(ORCID_MEDIA_TYPE) {
        return Err(malformed(
            "successful response was not application/vnd.orcid+json",
        ));
    }
    Ok(())
}

fn final_orcid(final_url: &str, base: &str, operation: &str) -> Result<String, BioMcpError> {
    let url =
        reqwest::Url::parse(final_url).map_err(|_| malformed("final response URL was invalid"))?;
    let base = reqwest::Url::parse(base).map_err(|_| malformed("ORCID base URL was invalid"))?;
    let segments = url
        .path_segments()
        .ok_or_else(|| malformed("final response URL had no path"))?
        .collect::<Vec<_>>();
    let base_segments = base
        .path_segments()
        .ok_or_else(|| malformed("ORCID base URL had no path"))?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() != base_segments.len() + 2
        || segments[..base_segments.len()] != base_segments
        || segments.last() != Some(&operation)
    {
        return Err(malformed(
            "final response URL did not match the requested ORCID base and operation",
        ));
    }
    let orcid = segments[base_segments.len()];
    validate_orcid(orcid)?;
    Ok(orcid.to_string())
}

fn malformed(message: &str) -> BioMcpError {
    BioMcpError::Api {
        api: ORCID_API.into(),
        message: message.into(),
    }
}

fn public(visibility: Option<&str>) -> bool {
    visibility == Some("PUBLIC")
}

fn map_record(wire: RecordWire) -> OrcidRecord {
    let names = wire
        .person
        .and_then(|person| person.name)
        .filter(|name| public(name.visibility.as_deref()))
        .map(|name| {
            vec![OrcidName {
                given_names: value(name.given_names),
                family_name: value(name.family_name),
                credit_name: value(name.credit_name),
                visibility: name.visibility.unwrap_or_default(),
                source: name.source.map(Into::into),
                created_date: date_value(name.created_date),
                modified_date: date_value(name.last_modified_date),
            }]
        })
        .unwrap_or_default();
    let employments = wire
        .activities_summary
        .and_then(|activities| activities.employments)
        .map(|employments| employments.affiliation_group)
        .unwrap_or_default()
        .into_iter()
        .flat_map(|group| group.summaries)
        .filter_map(|summary| summary.employment_summary)
        .filter(|employment| public(employment.visibility.as_deref()))
        .filter_map(map_employment)
        .collect();
    OrcidRecord {
        modified_date: wire
            .history
            .and_then(|history| date_value(history.last_modified_date)),
        names,
        employments,
    }
}

fn map_employment(wire: EmploymentWire) -> Option<OrcidEmployment> {
    let organization = wire.organization?;
    Some(OrcidEmployment {
        organization: OrcidOrganization {
            name: organization.name?,
            city: organization
                .address
                .as_ref()
                .and_then(|value| value.city.clone()),
            region: organization
                .address
                .as_ref()
                .and_then(|value| value.region.clone()),
            country: organization.address.and_then(|value| value.country),
            disambiguated_identifier: organization
                .disambiguated_organization
                .as_ref()
                .and_then(|value| value.disambiguated_organization_identifier.clone()),
            disambiguation_source: organization
                .disambiguated_organization
                .and_then(|value| value.disambiguation_source),
        },
        department_name: wire.department_name,
        role_title: wire.role_title,
        start_date: wire.start_date.map(Into::into),
        end_date: wire.end_date.map(Into::into),
        put_code: wire.put_code,
        visibility: wire.visibility.unwrap_or_default(),
        source: wire.source.map(Into::into),
        created_date: date_value(wire.created_date),
        modified_date: date_value(wire.last_modified_date),
    })
}

fn map_works(wire: WorksWire) -> OrcidWorks {
    let groups = wire
        .group
        .into_iter()
        .filter_map(|group| {
            let summaries = group
                .work_summary
                .into_iter()
                .filter(|summary| public(summary.visibility.as_deref()))
                .map(|summary| OrcidWorkSummary {
                    title: summary
                        .title
                        .as_ref()
                        .and_then(|title| value(title.title.clone())),
                    subtitle: summary
                        .title
                        .as_ref()
                        .and_then(|title| value(title.subtitle.clone())),
                    translated_title: summary
                        .title
                        .as_ref()
                        .and_then(|title| title.translated_title.as_ref())
                        .and_then(|title| title.value.clone()),
                    translated_title_language: summary
                        .title
                        .as_ref()
                        .and_then(|title| title.translated_title.as_ref())
                        .and_then(|title| title.language_code.clone()),
                    work_type: summary.work_type,
                    external_ids: summary
                        .external_ids
                        .map(map_external_ids)
                        .unwrap_or_default(),
                    put_code: summary.put_code,
                    visibility: summary.visibility.unwrap_or_default(),
                    source: summary.source.map(Into::into),
                    created_date: date_value(summary.created_date),
                    modified_date: date_value(summary.last_modified_date),
                })
                .collect::<Vec<_>>();
            if summaries.is_empty() {
                return None;
            }
            Some(OrcidWorkGroup {
                external_ids: group.external_ids.map(map_external_ids).unwrap_or_default(),
                summaries,
            })
        })
        .collect();
    OrcidWorks {
        groups,
        continuation: None,
    }
}

fn map_external_ids(ids: ExternalIdsWire) -> Vec<OrcidExternalId> {
    ids.external_id
        .into_iter()
        .map(|id| OrcidExternalId {
            external_id_type: id.external_id_type,
            external_id_value: id.external_id_value,
            external_id_relationship: id.external_id_relationship,
            normalized_value: id.external_id_normalized.and_then(|value| value.value),
            normalized_url: value(id.external_id_url),
        })
        .collect()
}

fn value(wire: Option<ValueWire>) -> Option<String> {
    wire.and_then(|value| value.value)
}

fn date_value(wire: Option<DateValueWire>) -> Option<i64> {
    wire.and_then(|value| value.value)
}

impl From<PartialDateWire> for OrcidPartialDate {
    fn from(value: PartialDateWire) -> Self {
        Self {
            year: value.year.and_then(|value| value.value),
            month: value.month.and_then(|value| value.value),
            day: value.day.and_then(|value| value.value),
        }
    }
}

impl From<SourceWire> for OrcidSource {
    fn from(value: SourceWire) -> Self {
        Self {
            source_orcid: value.source_orcid.and_then(|value| value.path),
            source_name: value.source_name.and_then(|value| value.value),
            assertion_origin_orcid: value.assertion_origin_orcid.and_then(|value| value.path),
            assertion_origin_name: value.assertion_origin_name.and_then(|value| value.value),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RecordWire {
    #[serde(rename = "orcid-identifier")]
    orcid_identifier: OrcidIdentifierWire,
    history: Option<HistoryWire>,
    person: Option<PersonWire>,
    #[serde(rename = "activities-summary")]
    activities_summary: Option<ActivitiesWire>,
}

#[derive(Debug, Deserialize)]
struct OrcidIdentifierWire {
    path: Option<String>,
}
#[derive(Debug, Deserialize)]
struct HistoryWire {
    #[serde(rename = "last-modified-date")]
    last_modified_date: Option<DateValueWire>,
}
#[derive(Debug, Deserialize)]
struct PersonWire {
    name: Option<NameWire>,
}
#[derive(Debug, Deserialize)]
struct NameWire {
    #[serde(rename = "given-names")]
    given_names: Option<ValueWire>,
    #[serde(rename = "family-name")]
    family_name: Option<ValueWire>,
    #[serde(rename = "credit-name")]
    credit_name: Option<ValueWire>,
    visibility: Option<String>,
    source: Option<SourceWire>,
    #[serde(rename = "created-date")]
    created_date: Option<DateValueWire>,
    #[serde(rename = "last-modified-date")]
    last_modified_date: Option<DateValueWire>,
}
#[derive(Debug, Deserialize)]
struct ActivitiesWire {
    employments: Option<EmploymentsWire>,
}
#[derive(Debug, Deserialize)]
struct EmploymentsWire {
    #[serde(rename = "affiliation-group", default)]
    affiliation_group: Vec<AffiliationGroupWire>,
}
#[derive(Debug, Deserialize)]
struct AffiliationGroupWire {
    #[serde(default)]
    summaries: Vec<EmploymentSummaryWire>,
}
#[derive(Debug, Deserialize)]
struct EmploymentSummaryWire {
    #[serde(rename = "employment-summary")]
    employment_summary: Option<EmploymentWire>,
}
#[derive(Debug, Deserialize)]
struct EmploymentWire {
    #[serde(rename = "department-name")]
    department_name: Option<String>,
    #[serde(rename = "role-title")]
    role_title: Option<String>,
    #[serde(rename = "start-date")]
    start_date: Option<PartialDateWire>,
    #[serde(rename = "end-date")]
    end_date: Option<PartialDateWire>,
    organization: Option<OrganizationWire>,
    source: Option<SourceWire>,
    #[serde(rename = "put-code")]
    put_code: Option<i64>,
    visibility: Option<String>,
    #[serde(rename = "created-date")]
    created_date: Option<DateValueWire>,
    #[serde(rename = "last-modified-date")]
    last_modified_date: Option<DateValueWire>,
}
#[derive(Debug, Deserialize)]
struct OrganizationWire {
    name: Option<String>,
    address: Option<AddressWire>,
    #[serde(rename = "disambiguated-organization")]
    disambiguated_organization: Option<DisambiguatedOrganizationWire>,
}
#[derive(Debug, Deserialize)]
struct AddressWire {
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
}
#[derive(Debug, Deserialize)]
struct DisambiguatedOrganizationWire {
    #[serde(rename = "disambiguated-organization-identifier")]
    disambiguated_organization_identifier: Option<String>,
    #[serde(rename = "disambiguation-source")]
    disambiguation_source: Option<String>,
}
#[derive(Debug, Deserialize)]
struct WorksWire {
    #[serde(default)]
    group: Vec<WorkGroupWire>,
}
#[derive(Debug, Deserialize)]
struct WorkGroupWire {
    #[serde(rename = "external-ids")]
    external_ids: Option<ExternalIdsWire>,
    #[serde(rename = "work-summary", default)]
    work_summary: Vec<WorkSummaryWire>,
}
#[derive(Debug, Deserialize)]
struct WorkSummaryWire {
    title: Option<WorkTitleWire>,
    #[serde(rename = "type")]
    work_type: Option<String>,
    #[serde(rename = "external-ids")]
    external_ids: Option<ExternalIdsWire>,
    source: Option<SourceWire>,
    #[serde(rename = "put-code")]
    put_code: Option<i64>,
    visibility: Option<String>,
    #[serde(rename = "created-date")]
    created_date: Option<DateValueWire>,
    #[serde(rename = "last-modified-date")]
    last_modified_date: Option<DateValueWire>,
}
#[derive(Debug, Deserialize)]
struct WorkTitleWire {
    title: Option<ValueWire>,
    subtitle: Option<ValueWire>,
    #[serde(rename = "translated-title")]
    translated_title: Option<TranslatedTitleWire>,
}
#[derive(Debug, Deserialize)]
struct TranslatedTitleWire {
    value: Option<String>,
    #[serde(rename = "language-code")]
    language_code: Option<String>,
}
#[derive(Debug, Deserialize)]
struct ExternalIdsWire {
    #[serde(rename = "external-id", default)]
    external_id: Vec<ExternalIdWire>,
}
#[derive(Debug, Deserialize)]
struct ExternalIdWire {
    #[serde(rename = "external-id-type")]
    external_id_type: Option<String>,
    #[serde(rename = "external-id-value")]
    external_id_value: Option<String>,
    #[serde(rename = "external-id-url")]
    external_id_url: Option<ValueWire>,
    #[serde(rename = "external-id-relationship")]
    external_id_relationship: Option<String>,
    #[serde(rename = "external-id-normalized")]
    external_id_normalized: Option<NormalizedExternalIdWire>,
}
#[derive(Debug, Deserialize)]
struct NormalizedExternalIdWire {
    value: Option<String>,
}
#[derive(Debug, Deserialize)]
struct SourceWire {
    #[serde(rename = "source-orcid")]
    source_orcid: Option<SourceOrcidWire>,
    #[serde(rename = "source-name")]
    source_name: Option<ValueWire>,
    #[serde(rename = "assertion-origin-orcid")]
    assertion_origin_orcid: Option<SourceOrcidWire>,
    #[serde(rename = "assertion-origin-name")]
    assertion_origin_name: Option<ValueWire>,
}
#[derive(Debug, Deserialize)]
struct SourceOrcidWire {
    path: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
struct ValueWire {
    value: Option<String>,
}
#[derive(Debug, Deserialize)]
struct DateValueWire {
    value: Option<i64>,
}
#[derive(Debug, Deserialize)]
struct PartialDateWire {
    year: Option<ValueWire>,
    month: Option<ValueWire>,
    day: Option<ValueWire>,
}

#[cfg(test)]
mod tests;
