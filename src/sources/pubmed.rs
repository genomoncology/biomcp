// dead-code reason: PubMed keeps fixture-only request planners beside live citation execution
#![allow(dead_code)]

use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashSet;

use reqwest_middleware::ClientWithMiddleware;
use serde::Deserialize;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::xml::parse_external_xml;

const PUBMED_EUTILS_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const PUBMED_EUTILS_BASE_ENV: &str = "BIOMCP_PUBMED_BASE";
const PUBMED_EUTILS_API: &str = "pubmed-eutils";
pub(crate) const PUBMED_CITATION_NODE_LIMIT: u32 = 100_000;

#[derive(Clone)]
pub struct PubMedClient {
    client: ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedESearchParams {
    pub term: String,
    pub retstart: usize,
    pub retmax: usize,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

// dead-code reason: pubmed::PubMedESearchRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubMedESearchRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

// dead-code reason: pubmed::PubMedESummaryRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubMedESummaryRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

// dead-code reason: pubmed::PubMedCitationRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct PubMedCitationRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PubMedCitationErrorKind {
    Network,
    Http,
    RateLimited,
    InvalidResponse,
    ResponseTooLarge,
    Parse,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedCitation {
    pub authors: Vec<PubMedCitationAuthor>,
    pub mesh_headings: Vec<PubMedMeshHeading>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedCitationAuthor {
    pub name: String,
    pub orcid: Option<String>,
    pub affiliations: Vec<PubMedAffiliation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedAffiliation {
    pub text: String,
    pub identifiers: Vec<PubMedAffiliationIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedAffiliationIdentifier {
    pub source: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedMeshHeading {
    pub descriptor: PubMedMeshTerm,
    pub qualifiers: Vec<PubMedMeshTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedMeshTerm {
    pub text: String,
    pub ui: Option<String>,
    pub major_topic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PubMedESearchResponse {
    pub count: u64,
    pub idlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ESummaryEntry {
    pub uid: String,
    pub title: String,
    pub sortpubdate: Option<String>,
    pub pubdate: Option<String>,
    pub edat: Option<String>,
    pub lr: Option<String>,
    pub fulljournalname: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ESearchEnvelope {
    esearchresult: ESearchInner,
}

#[derive(Debug, Deserialize)]
struct ESearchInner {
    count: String,
    #[serde(default)]
    idlist: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ESummaryEnvelope {
    result: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct HistoryEntry {
    pubstatus: String,
    date: String,
}

#[derive(Debug, Deserialize)]
struct ESummaryEntryRaw {
    uid: Option<String>,
    title: Option<String>,
    sortpubdate: Option<String>,
    pubdate: Option<String>,
    #[serde(default)]
    history: Vec<HistoryEntry>,
    fulljournalname: Option<String>,
    source: Option<String>,
}

fn format_pubmed_date(value: &str) -> String {
    value.trim().replace('-', "/")
}

impl PubMedClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PUBMED_EUTILS_BASE, PUBMED_EUTILS_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    async fn send(
        &self,
        req: reqwest_middleware::RequestBuilder,
        authenticated: bool,
    ) -> Result<
        (
            reqwest::StatusCode,
            Option<reqwest::header::HeaderValue>,
            Vec<u8>,
        ),
        BioMcpError,
    > {
        let resp = crate::sources::apply_cache_mode_with_auth(req, authenticated)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PUBMED,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PUBMED),
        )
        .await?;
        Ok((status, content_type, bytes))
    }

    pub(crate) fn citation_plan(
        pmid: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let pmid = pmid.trim();
        if pmid.is_empty() || !pmid.chars().all(|character| character.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PubMed citation PMID must be numeric".into(),
            ));
        }

        let mut plan = RequestPlan::get("efetch.fcgi")
            .query("db", "pubmed")
            .query("retmode", "xml")
            .query("id", pmid);
        if let Some(key) = clean_api_key(api_key) {
            plan = plan.query("api_key", key);
        }
        Ok(plan)
    }

    // dead-code reason: pubmed::citation_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn citation_request_plan(
        &self,
        pmid: &str,
    ) -> Result<PubMedCitationRequestPlan, BioMcpError> {
        let plan = Self::citation_plan(pmid, self.api_key.as_deref())?;
        Ok(PubMedCitationRequestPlan {
            method: "GET",
            path: "/efetch.fcgi",
            query_params: plan
                .query
                .into_iter()
                .filter(|(key, _)| key != "api_key")
                .map(|(key, value)| (pubmed_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "xml",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        })
    }

    pub(crate) async fn citation(
        &self,
        pmid: &str,
    ) -> Result<PubMedCitation, PubMedCitationErrorKind> {
        let authenticated = self.api_key.is_some();
        let plan = Self::citation_plan(pmid, self.api_key.as_deref())
            .map_err(|_| PubMedCitationErrorKind::InvalidResponse)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let response = crate::sources::apply_cache_mode_with_auth(req, authenticated)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PUBMED,
            ))
            .await
            .map_err(Self::citation_request_error)?;
        let status = response.status();
        Self::validate_citation_status(status)?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let bytes = crate::sources::read_limited_source_body(
            response,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PUBMED),
        )
        .await
        .map_err(Self::citation_request_error)?;
        let xml = Self::decode_citation_response(status, content_type.as_ref(), bytes)?;
        let pmid = pmid.trim().to_string();
        tokio::task::spawn_blocking(move || parse_citation_xml(&pmid, &xml))
            .await
            .map_err(|_| PubMedCitationErrorKind::Parse)?
    }

    fn citation_request_error(error: BioMcpError) -> PubMedCitationErrorKind {
        match error {
            BioMcpError::WithSourceContext { source, .. } => Self::citation_request_error(*source),
            BioMcpError::Http(_) | BioMcpError::HttpMiddleware(_) => {
                PubMedCitationErrorKind::Network
            }
            BioMcpError::BodyLimit { .. } => PubMedCitationErrorKind::ResponseTooLarge,
            _ => PubMedCitationErrorKind::InvalidResponse,
        }
    }

    fn validate_citation_status(
        status: reqwest::StatusCode,
    ) -> Result<(), PubMedCitationErrorKind> {
        match status {
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(PubMedCitationErrorKind::RateLimited),
            reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::GONE => {
                Err(PubMedCitationErrorKind::NotFound)
            }
            status if !status.is_success() => Err(PubMedCitationErrorKind::Http),
            _ => Ok(()),
        }
    }

    fn decode_citation_response(
        status: reqwest::StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: Vec<u8>,
    ) -> Result<String, PubMedCitationErrorKind> {
        Self::validate_citation_status(status)?;
        let media_type = content_type
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(';').next())
            .map(str::trim)
            .map(str::to_ascii_lowercase);
        if !matches!(media_type.as_deref(), Some("application/xml" | "text/xml")) {
            return Err(PubMedCitationErrorKind::InvalidResponse);
        }
        String::from_utf8(bytes).map_err(|_| PubMedCitationErrorKind::InvalidResponse)
    }

    pub(crate) fn esearch_plan(
        params: &PubMedESearchParams,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let term = params.term.trim();
        if term.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "PubMed ESearch term is required".into(),
            ));
        }
        if term.len() > 4096 {
            return Err(BioMcpError::InvalidArgument(
                "PubMed ESearch term is too long".into(),
            ));
        }
        if params.retmax == 0 || params.retmax > 10_000 {
            return Err(BioMcpError::InvalidArgument(
                "PubMed ESearch retmax must be between 1 and 10000".into(),
            ));
        }

        let mut query_params = vec![
            ("db", "pubmed".to_string()),
            ("retmode", "json".to_string()),
            ("term", term.to_string()),
            ("retstart", params.retstart.to_string()),
            ("retmax", params.retmax.to_string()),
        ];
        if params.date_from.is_some() || params.date_to.is_some() {
            query_params.push(("datetype", "pdat".to_string()));
        }
        if let Some(date_from) = params
            .date_from
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query_params.push(("mindate", format_pubmed_date(date_from)));
        }
        if let Some(date_to) = params
            .date_to
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query_params.push(("maxdate", format_pubmed_date(date_to)));
        }
        if let Some(key) = clean_api_key(api_key) {
            query_params.push(("api_key", key.to_string()));
        }

        let mut plan = RequestPlan::get("esearch.fcgi");
        for (key, value) in query_params {
            plan = plan.query(key, value);
        }
        Ok(plan)
    }

    // dead-code reason: pubmed::esearch_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn esearch_request_plan(
        &self,
        params: &PubMedESearchParams,
    ) -> Result<PubMedESearchRequestPlan, BioMcpError> {
        let plan = Self::esearch_plan(params, self.api_key.as_deref())?;
        Ok(PubMedESearchRequestPlan {
            method: "GET",
            path: "/esearch.fcgi",
            query_params: plan
                .query
                .into_iter()
                .filter(|(key, _)| key != "api_key")
                .map(|(key, value)| (pubmed_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        })
    }

    pub async fn esearch(
        &self,
        params: &PubMedESearchParams,
    ) -> Result<PubMedESearchResponse, BioMcpError> {
        let authenticated = self.api_key.is_some();
        let plan = Self::esearch_plan(params, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, content_type, bytes) = self.send(req, authenticated).await?;
        Self::decode_esearch_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PUBMED,
            ))
        })
    }

    pub(crate) fn decode_esearch_response(
        status: reqwest::StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
    ) -> Result<PubMedESearchResponse, BioMcpError> {
        let response: ESearchEnvelope = crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::PUBMED),
            status,
            content_type,
            bytes,
            true,
        )?;
        let count = response
            .esearchresult
            .count
            .trim()
            .parse::<u64>()
            .map_err(|_| BioMcpError::Api {
                api: PUBMED_EUTILS_API.to_string(),
                message: format!(
                    "Invalid ESearch count value {:?} from upstream contract",
                    response.esearchresult.count
                ),
            })?;

        Ok(PubMedESearchResponse {
            count,
            idlist: response.esearchresult.idlist,
        })
    }

    pub(crate) fn esummary_plan(
        ids: &[String],
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        if ids.is_empty() {
            return Ok(None);
        }

        let requested_ids = ids.iter().map(|id| id.trim()).collect::<Vec<_>>();
        if let Some(blank) = requested_ids.iter().find(|id| id.is_empty()) {
            return Err(BioMcpError::InvalidArgument(format!(
                "PubMed ESummary ids must be nonblank (got {:?})",
                blank
            )));
        }

        let mut plan = RequestPlan::get("esummary.fcgi")
            .query("db", "pubmed")
            .query("retmode", "json")
            .query("id", requested_ids.join(","));
        if let Some(key) = clean_api_key(api_key) {
            plan = plan.query("api_key", key);
        }
        Ok(Some(plan))
    }

    // dead-code reason: pubmed::esummary_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn esummary_request_plan(
        &self,
        ids: &[String],
    ) -> Result<Option<PubMedESummaryRequestPlan>, BioMcpError> {
        let Some(plan) = Self::esummary_plan(ids, self.api_key.as_deref())? else {
            return Ok(None);
        };

        Ok(Some(PubMedESummaryRequestPlan {
            method: "GET",
            path: "/esummary.fcgi",
            query_params: plan
                .query
                .into_iter()
                .filter(|(key, _)| key != "api_key")
                .map(|(key, value)| (pubmed_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "default"
            },
            status_expectation: "non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: if self.api_key.is_some() {
                "authenticated"
            } else {
                "keyless"
            },
        }))
    }

    pub async fn esummary(&self, ids: &[String]) -> Result<Vec<ESummaryEntry>, BioMcpError> {
        let Some(plan) = Self::esummary_plan(ids, self.api_key.as_deref())? else {
            return Ok(Vec::new());
        };

        let authenticated = self.api_key.is_some();
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let (status, content_type, bytes) = self.send(req, authenticated).await?;
        Self::decode_esummary_response(ids, status, content_type.as_ref(), &bytes).map_err(
            |error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::PUBMED,
                ))
            },
        )
    }

    pub(crate) fn decode_esummary_response(
        ids: &[String],
        status: reqwest::StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
    ) -> Result<Vec<ESummaryEntry>, BioMcpError> {
        let requested_ids = ids.iter().map(|id| id.trim()).collect::<Vec<_>>();
        let requested_set = requested_ids.iter().copied().collect::<HashSet<_>>();
        let response: ESummaryEnvelope = crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::PUBMED),
            status,
            content_type,
            bytes,
            true,
        )?;

        let uids = response
            .result
            .get("uids")
            .and_then(|value| value.as_array())
            .ok_or_else(|| BioMcpError::Api {
                api: PUBMED_EUTILS_API.to_string(),
                message: "ESummary response missing uids array".into(),
            })?;

        let mut upstream_ids = Vec::with_capacity(uids.len());
        let mut upstream_seen = HashSet::with_capacity(uids.len());
        for value in uids {
            let uid = value
                .as_str()
                .map(str::trim)
                .filter(|uid| !uid.is_empty())
                .ok_or_else(|| BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: "ESummary uids must be a string array of nonblank PMIDs".into(),
                })?;
            if !upstream_seen.insert(uid) {
                return Err(BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!("ESummary response contains duplicate uid {uid}"),
                });
            }
            upstream_ids.push(uid);
        }

        for requested_id in &requested_ids {
            if !upstream_seen.contains(requested_id) {
                return Err(BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!(
                        "ESummary response missing requested PMID {requested_id} in uids"
                    ),
                });
            }
        }
        for upstream_id in &upstream_ids {
            if !requested_set.contains(upstream_id) {
                return Err(BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!("ESummary response contains unexpected PMID {upstream_id}"),
                });
            }
        }

        let mut entries = Vec::with_capacity(requested_ids.len());
        for requested_id in requested_ids {
            let raw_value = response
                .result
                .get(requested_id)
                .ok_or_else(|| BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!(
                        "ESummary response missing entry for requested PMID {requested_id}"
                    ),
                })?;
            let raw = serde_json::from_value::<ESummaryEntryRaw>(raw_value.clone()).map_err(
                |source| BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!(
                        "ESummary entry for PMID {requested_id} failed to parse: {source}"
                    ),
                },
            )?;
            if raw
                .uid
                .as_deref()
                .map(str::trim)
                .filter(|uid| !uid.is_empty())
                .is_some_and(|uid| uid != requested_id)
            {
                return Err(BioMcpError::Api {
                    api: PUBMED_EUTILS_API.to_string(),
                    message: format!(
                        "ESummary entry for PMID {requested_id} had conflicting inner uid {:?}",
                        raw.uid
                    ),
                });
            }
            let edat = raw
                .history
                .iter()
                .find(|h| h.pubstatus == "entrez")
                .or_else(|| raw.history.iter().find(|h| h.pubstatus == "pubmed"))
                .map(|h| h.date.clone());
            let lr = raw
                .history
                .iter()
                .find(|h| h.pubstatus == "medline")
                .map(|h| h.date.clone());
            entries.push(ESummaryEntry {
                uid: requested_id.to_string(),
                title: raw.title.unwrap_or_default(),
                sortpubdate: raw.sortpubdate,
                pubdate: raw.pubdate,
                edat,
                lr,
                fulljournalname: raw.fulljournalname,
                source: raw.source,
            });
        }

        Ok(entries)
    }
}

fn pubmed_api_error(message: impl Into<String>) -> BioMcpError {
    BioMcpError::Api {
        api: PUBMED_EUTILS_API.to_string(),
        message: message.into(),
    }
}

fn element_children<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    name: &str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'input>> {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

fn child_text<'a, 'input>(node: roxmltree::Node<'a, 'input>, name: &str) -> Option<&'a str> {
    element_children(node, name)
        .next()
        .and_then(|child| child.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn required_text<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    field: &str,
) -> Result<&'a str, BioMcpError> {
    node.text()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| pubmed_api_error(format!("PubMed citation has blank {field}")))
}

fn parse_major_topic(node: roxmltree::Node<'_, '_>) -> Result<bool, BioMcpError> {
    match node.attribute("MajorTopicYN") {
        Some("Y") => Ok(true),
        Some("N") => Ok(false),
        _ => Err(pubmed_api_error(
            "PubMed citation has invalid or missing MajorTopicYN",
        )),
    }
}

fn parse_mesh_term(node: roxmltree::Node<'_, '_>) -> Result<PubMedMeshTerm, BioMcpError> {
    Ok(PubMedMeshTerm {
        text: required_text(node, node.tag_name().name())?.to_string(),
        ui: node
            .attribute("UI")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        major_topic: parse_major_topic(node)?,
    })
}

fn parse_affiliation(node: roxmltree::Node<'_, '_>) -> Result<PubMedAffiliation, BioMcpError> {
    let text = child_text(node, "Affiliation")
        .ok_or_else(|| pubmed_api_error("PubMed citation has blank affiliation"))?;
    let identifiers = element_children(node, "Identifier")
        .map(|identifier| {
            let source = identifier
                .attribute("Source")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    pubmed_api_error("PubMed affiliation identifier has blank source")
                })?;
            let value = required_text(identifier, "affiliation identifier")?;
            Ok(PubMedAffiliationIdentifier {
                source: source.to_string(),
                value: value.to_string(),
            })
        })
        .collect::<Result<Vec<_>, BioMcpError>>()?;
    Ok(PubMedAffiliation {
        text: text.to_string(),
        identifiers,
    })
}

fn author_name(node: roxmltree::Node<'_, '_>) -> Option<String> {
    if let Some(collective) = child_text(node, "CollectiveName") {
        return Some(collective.to_string());
    }
    let last = child_text(node, "LastName")?;
    if let Some(fore) = child_text(node, "ForeName") {
        return Some(format!("{fore} {last}"));
    }
    if let Some(initials) = child_text(node, "Initials") {
        return Some(format!("{initials} {last}"));
    }
    Some(last.to_string())
}

fn normalize_orcid(value: &str) -> String {
    let value = value.trim();
    for prefix in ["https://orcid.org/", "http://orcid.org/"] {
        if value.len() >= prefix.len()
            && value
                .get(..prefix.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
        {
            return value[prefix.len()..].to_string();
        }
    }
    value.to_string()
}

fn parse_author(node: roxmltree::Node<'_, '_>) -> Result<PubMedCitationAuthor, BioMcpError> {
    let name = author_name(node)
        .ok_or_else(|| pubmed_api_error("PubMed citation author is missing a usable name"))?;
    let orcid = element_children(node, "Identifier")
        .find(|identifier| {
            identifier
                .attribute("Source")
                .is_some_and(|source| source.trim().eq_ignore_ascii_case("ORCID"))
                && identifier
                    .text()
                    .is_some_and(|value| !value.trim().is_empty())
        })
        .and_then(|identifier| identifier.text())
        .map(normalize_orcid);
    let affiliations = element_children(node, "AffiliationInfo")
        .map(parse_affiliation)
        .collect::<Result<Vec<_>, BioMcpError>>()?;
    Ok(PubMedCitationAuthor {
        name,
        orcid,
        affiliations,
    })
}

fn parse_citation_xml(pmid: &str, xml: &str) -> Result<PubMedCitation, PubMedCitationErrorKind> {
    let document = parse_external_xml(xml, PUBMED_CITATION_NODE_LIMIT)
        .map_err(|_| PubMedCitationErrorKind::Parse)?;
    let citation = document
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "PubmedArticle")
        .filter_map(|article| element_children(article, "MedlineCitation").next())
        .find(|citation| child_text(*citation, "PMID") == Some(pmid))
        .ok_or(PubMedCitationErrorKind::NotFound)?;

    let authors = element_children(citation, "Article")
        .next()
        .and_then(|article| element_children(article, "AuthorList").next())
        .map(|list| {
            element_children(list, "Author")
                .map(parse_author)
                .collect::<Result<Vec<_>, BioMcpError>>()
        })
        .transpose()
        .map_err(|_| PubMedCitationErrorKind::Parse)?
        .unwrap_or_default();

    let mesh_headings = element_children(citation, "MeshHeadingList")
        .next()
        .map(|list| {
            element_children(list, "MeshHeading")
                .map(|heading| {
                    let mut descriptors = element_children(heading, "DescriptorName");
                    let descriptor = descriptors.next().ok_or_else(|| {
                        pubmed_api_error("PubMed MeSH heading is missing a descriptor")
                    })?;
                    if descriptors.next().is_some() {
                        return Err(pubmed_api_error(
                            "PubMed MeSH heading has multiple descriptors",
                        ));
                    }
                    Ok(PubMedMeshHeading {
                        descriptor: parse_mesh_term(descriptor)?,
                        qualifiers: element_children(heading, "QualifierName")
                            .map(parse_mesh_term)
                            .collect::<Result<Vec<_>, BioMcpError>>()?,
                    })
                })
                .collect::<Result<Vec<_>, BioMcpError>>()
        })
        .transpose()
        .map_err(|_| PubMedCitationErrorKind::Parse)?
        .unwrap_or_default();

    Ok(PubMedCitation {
        authors,
        mesh_headings,
    })
}

fn clean_api_key(api_key: Option<&str>) -> Option<&str> {
    api_key.map(str::trim).filter(|key| !key.is_empty())
}

// dead-code reason: pubmed::pubmed_query_key preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
fn pubmed_query_key(key: &str) -> &'static str {
    match key {
        "db" => "db",
        "retmode" => "retmode",
        "term" => "term",
        "retstart" => "retstart",
        "retmax" => "retmax",
        "datetype" => "datetype",
        "mindate" => "mindate",
        "maxdate" => "maxdate",
        "id" => "id",
        "api_key" => "api_key",
        _ => unreachable!("unexpected PubMed query key: {key}"),
    }
}

#[cfg(test)]
mod tests;
