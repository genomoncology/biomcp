use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::BioMcpError;
use crate::sources::RequestBuilderSourceContextExt;
use crate::sources::provider_url_policy::ProviderUrlPolicy;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const SEMANTIC_SCHOLAR_BASE: &str = "https://api.semanticscholar.org";
const SEMANTIC_SCHOLAR_API: &str = "semantic_scholar";
const SEMANTIC_SCHOLAR_BASE_ENV: &str = "BIOMCP_S2_BASE";
const SEMANTIC_SCHOLAR_DOCS_URL: &str = "https://www.semanticscholar.org/product/api";
const GRAPH_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year,tldr,citationCount,influentialCitationCount,referenceCount,isOpenAccess,openAccessPdf,authors.authorId,authors.name,authors.affiliations,authors.paperCount,authors.citationCount,authors.hIndex";
const BATCH_PAPER_FIELDS: &str = "paperId,externalIds,title,venue,year";
const BATCH_PAPER_COMPACT_FIELDS: &str =
    "paperId,externalIds,title,venue,year,tldr,citationCount,influentialCitationCount";
const BATCH_PAPER_SEARCH_ENRICHMENT_FIELDS: &str =
    "paperId,externalIds,citationCount,influentialCitationCount,abstract";
const SEARCH_PAPER_FIELDS: &str =
    "paperId,externalIds,title,venue,year,citationCount,influentialCitationCount,abstract";
const CITATION_EDGE_FIELDS: &str = "contexts,intents,isInfluential,citingPaper.paperId,citingPaper.externalIds,citingPaper.title,citingPaper.venue,citingPaper.year";
const REFERENCE_EDGE_FIELDS: &str = "contexts,intents,isInfluential,citedPaper.paperId,citedPaper.externalIds,citedPaper.title,citedPaper.venue,citedPaper.year";
const RECOMMENDATION_FIELDS: &str = "paperId,externalIds,title,venue,year";
// dead-code reason: semantic_scholar::AUTHOR_FIELDS preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
const AUTHOR_FIELDS: &str =
    "authorId,name,affiliations,externalIds,paperCount,citationCount,hIndex";
// dead-code reason: semantic_scholar::AUTHOR_PAPER_FIELDS preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
const AUTHOR_PAPER_FIELDS: &str =
    "paperId,corpusId,externalIds,title,venue,year,authors.authorId,authors.name";
// dead-code reason: semantic_scholar::SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
const SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX: usize = 100;
// dead-code reason: semantic_scholar::SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
const SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX: usize = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticScholarAuthMode {
    Authenticated,
    SharedPool,
}

// dead-code reason: semantic_scholar::SemanticScholarPaperSearchRequestPlan preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
pub struct SemanticScholarPaperSearchRequestPlan {
    pub method: &'static str,
    pub path: &'static str,
    pub query_params: Vec<(&'static str, String)>,
    pub cache_mode: &'static str,
    pub status_expectation: &'static str,
    pub content_type_expectation: &'static str,
    pub auth_mode: SemanticScholarAuthMode,
}

#[derive(Clone)]
pub struct SemanticScholarClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl SemanticScholarClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(SEMANTIC_SCHOLAR_BASE, SEMANTIC_SCHOLAR_BASE_ENV);
        let base_url = reqwest::Url::parse(base.as_ref()).map_err(|_| BioMcpError::Api {
            api: SEMANTIC_SCHOLAR_API.to_string(),
            message:
                "Semantic Scholar source unavailable: outbound policy rejected invalid base URL"
                    .into(),
        })?;
        let policy = ProviderUrlPolicy::semantic_scholar_api(&base_url)?;
        let api_key = effective_api_key(&policy, &base_url, crate::sources::s2_api_key());
        let client = crate::sources::semantic_scholar_provider_client(&policy, api_key.is_some())?;
        Ok(Self {
            client,
            base,
            api_key,
        })
    }

    pub fn auth_mode(&self) -> SemanticScholarAuthMode {
        match self.api_key.as_ref() {
            Some(_) => SemanticScholarAuthMode::Authenticated,
            None => SemanticScholarAuthMode::SharedPool,
        }
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let context =
            crate::error::SourceContext::retry(crate::error::SourceProvider::SEMANTIC_SCHOLAR);
        let resp = match crate::sources::apply_cache_mode_with_auth(req, self.api_key.is_some())
            .send_with_source_context(context)
            .await
        {
            Ok(resp) => resp,
            Err(BioMcpError::WithSourceContext { source, .. })
                if matches!(
                    source.as_ref(),
                    BioMcpError::HttpMiddleware(err)
                        if crate::sources::is_semantic_scholar_shared_pool_rate_limit_error(err)
                ) =>
            {
                return Err(BioMcpError::Api {
                    api: SEMANTIC_SCHOLAR_API.to_string(),
                    message: format!(
                        "Rate limited by Semantic Scholar. Set S2_API_KEY for a dedicated rate limit. See {SEMANTIC_SCHOLAR_DOCS_URL}"
                    ),
                }
                .with_source_context(crate::error::SourceContext::new(
                    crate::error::SourceProvider::SEMANTIC_SCHOLAR,
                    crate::error::RecoveryAction::ReviewSourceConfiguration,
                )));
            }
            Err(_) => {
                return Err(BioMcpError::Api {
                    api: SEMANTIC_SCHOLAR_API.to_string(),
                    message:
                        "Semantic Scholar source unavailable: outbound request rejected or failed"
                            .to_string(),
                }
                .with_source_context(context));
            }
        };
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(resp, context)
            .await
            .map_err(|error| {
                let error_context = match error {
                    BioMcpError::WithSourceContext { context, .. } => context,
                    _ => context,
                };
                BioMcpError::Api {
                    api: SEMANTIC_SCHOLAR_API.to_string(),
                    message: "Semantic Scholar source unavailable: response body could not be read"
                        .to_string(),
                }
                .with_source_context(error_context)
            })?;
        Self::decode_json_response(status, &bytes, self.api_key.is_none())
            .map_err(|error| error.with_source_context(context))
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: reqwest::StatusCode,
        bytes: &[u8],
        shared_pool: bool,
    ) -> Result<T, BioMcpError> {
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS && shared_pool {
            return Err(BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: format!(
                    "Rate limited by Semantic Scholar. Set S2_API_KEY for a dedicated rate limit. See {SEMANTIC_SCHOLAR_DOCS_URL}"
                ),
            });
        }
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: SEMANTIC_SCHOLAR_API.to_string(),
                message: format!("Semantic Scholar source unavailable: upstream HTTP {status}"),
            });
        }
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::SEMANTIC_SCHOLAR),
            status,
            None,
            bytes,
            false,
        )
    }
}

// dead-code reason: semantic_scholar::SemanticScholarClient preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
impl SemanticScholarClient {
    pub(crate) fn author_search_plan(
        query: &str,
        offset: usize,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar author search query is required".into(),
            ));
        }
        let limit = validate_author_page_limit(limit)?;
        Ok(with_s2_api_key(
            RequestPlan::get("graph/v1/author/search")
                .query("query", query)
                .query("fields", AUTHOR_FIELDS)
                .query("offset", offset.to_string())
                .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub async fn author_search(
        &self,
        query: &str,
        offset: usize,
        limit: usize,
    ) -> Result<SemanticScholarAuthorSearchResponse, BioMcpError> {
        let plan = Self::author_search_plan(query, offset, limit, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn author_detail_plan(
        author_id: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let author_id = validate_author_id(author_id)?;
        Ok(with_s2_api_key(
            RequestPlan::get(format!(
                "graph/v1/author/{}",
                encode_path_segment(author_id)
            ))
            .query("fields", AUTHOR_FIELDS),
            api_key,
        ))
    }

    pub async fn author_detail(
        &self,
        author_id: &str,
    ) -> Result<SemanticScholarAuthor, BioMcpError> {
        let plan = Self::author_detail_plan(author_id, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn author_batch_plan(
        author_ids: &[String],
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        if author_ids.is_empty() || author_ids.len() > SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX {
            return Err(BioMcpError::InvalidArgument(format!(
                "Semantic Scholar author batch lookup requires 1-{SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX} author IDs"
            )));
        }
        let author_ids = author_ids
            .iter()
            .map(|author_id| validate_author_id(author_id))
            .collect::<Result<Vec<_>, _>>()?;
        let mut plan = RequestPlan::post("graph/v1/author/batch").query("fields", AUTHOR_FIELDS);
        plan.body = RequestBody::Json(json!({ "ids": author_ids }));
        Ok(with_s2_api_key(plan, api_key))
    }

    pub async fn author_batch(
        &self,
        author_ids: &[String],
    ) -> Result<Vec<Option<SemanticScholarAuthor>>, BioMcpError> {
        let plan = Self::author_batch_plan(author_ids, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn author_papers_plan(
        author_id: &str,
        offset: usize,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let author_id = validate_author_id(author_id)?;
        let limit = validate_author_page_limit(limit)?;
        Ok(with_s2_api_key(
            RequestPlan::get(format!(
                "graph/v1/author/{}/papers",
                encode_path_segment(author_id)
            ))
            .query("fields", AUTHOR_PAPER_FIELDS)
            .query("offset", offset.to_string())
            .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub async fn author_papers(
        &self,
        author_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<SemanticScholarAuthorPapersResponse, BioMcpError> {
        let plan = Self::author_papers_plan(author_id, offset, limit, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }
}

impl SemanticScholarClient {
    pub(crate) fn paper_detail_plan(
        id: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let id = validate_paper_id(id)?;
        Ok(with_s2_api_key(
            RequestPlan::get(format!("graph/v1/paper/{}", encode_path_segment(id)))
                .query("fields", GRAPH_PAPER_FIELDS),
            api_key,
        ))
    }

    pub async fn paper_detail(&self, id: &str) -> Result<SemanticScholarPaper, BioMcpError> {
        let plan = Self::paper_detail_plan(id, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub async fn paper_batch(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<SemanticScholarPaper>>, BioMcpError> {
        self.paper_batch_with_fields(ids, BATCH_PAPER_FIELDS).await
    }

    pub async fn paper_batch_compact(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<SemanticScholarPaper>>, BioMcpError> {
        self.paper_batch_with_fields(ids, BATCH_PAPER_COMPACT_FIELDS)
            .await
    }

    pub async fn paper_batch_search_enrichment(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<SemanticScholarPaper>>, BioMcpError> {
        self.paper_batch_with_fields(ids, BATCH_PAPER_SEARCH_ENRICHMENT_FIELDS)
            .await
    }

    async fn paper_batch_with_fields(
        &self,
        ids: &[String],
        fields: &str,
    ) -> Result<Vec<Option<SemanticScholarPaper>>, BioMcpError> {
        let plan = Self::paper_batch_plan(ids, fields, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn paper_batch_plan(
        ids: &[String],
        fields: &str,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        if ids.is_empty() || ids.len() > 500 {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar batch lookup requires 1-500 paper IDs".into(),
            ));
        }
        let mut plan = RequestPlan::post("graph/v1/paper/batch").query("fields", fields);
        plan.body = RequestBody::Json(json!({ "ids": ids }));
        Ok(with_s2_api_key(plan, api_key))
    }

    pub(crate) fn paper_search_plan(
        query: &str,
        limit: usize,
        year_filter: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar paper search query is required".into(),
            ));
        }
        let limit = validate_limit(limit)?;
        let mut query_params = vec![
            ("query", query.to_string()),
            ("fields", SEARCH_PAPER_FIELDS.to_string()),
            ("limit", limit.to_string()),
        ];
        if let Some(year_filter) = year_filter {
            query_params.push(("year", year_filter.to_string()));
        }
        let mut plan = RequestPlan::get("graph/v1/paper/search");
        for (key, value) in query_params {
            plan = plan.query(key, value);
        }
        Ok(with_s2_api_key(plan, api_key))
    }

    // dead-code reason: semantic_scholar::paper_search_request_plan preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub fn paper_search_request_plan(
        &self,
        query: &str,
        limit: usize,
        year_filter: Option<&str>,
    ) -> Result<SemanticScholarPaperSearchRequestPlan, BioMcpError> {
        let plan = Self::paper_search_plan(query, limit, year_filter, self.api_key.as_deref())?;
        Ok(SemanticScholarPaperSearchRequestPlan {
            method: "GET",
            path: "graph/v1/paper/search",
            query_params: plan
                .query
                .into_iter()
                .map(|(key, value)| (semantic_scholar_query_key(&key), value))
                .collect(),
            cache_mode: if self.api_key.is_some() {
                "auth"
            } else {
                "shared_pool"
            },
            status_expectation: "429 shared_pool => unavailable guidance; non-2xx => Api",
            content_type_expectation: "json",
            auth_mode: self.auth_mode(),
        })
    }

    pub async fn paper_search(
        &self,
        query: &str,
        limit: usize,
        year_filter: Option<&str>,
    ) -> Result<SemanticScholarSearchResponse, BioMcpError> {
        let plan = Self::paper_search_plan(query, limit, year_filter, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn paper_search_bulk_plan(
        query: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let mut plan = Self::paper_search_plan(query, limit, None, api_key)?;
        plan.path = "graph/v1/paper/search/bulk".into();
        Ok(plan)
    }

    pub async fn paper_search_bulk(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<SemanticScholarSearchResponse, BioMcpError> {
        let plan = Self::paper_search_bulk_plan(query, limit, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub async fn paper_citations(
        &self,
        id: &str,
        limit: usize,
    ) -> Result<SemanticScholarGraphResponse<SemanticScholarCitationEdge>, BioMcpError> {
        let plan = Self::paper_subresource_plan(
            id,
            "citations",
            CITATION_EDGE_FIELDS,
            limit,
            self.api_key.as_deref(),
        )?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub async fn paper_references(
        &self,
        id: &str,
        limit: usize,
    ) -> Result<SemanticScholarGraphResponse<SemanticScholarReferenceEdge>, BioMcpError> {
        let plan = Self::paper_subresource_plan(
            id,
            "references",
            REFERENCE_EDGE_FIELDS,
            limit,
            self.api_key.as_deref(),
        )?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn paper_subresource_plan(
        id: &str,
        subresource: &str,
        fields: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let id = validate_paper_id(id)?;
        let limit = validate_limit(limit)?;
        Ok(with_s2_api_key(
            RequestPlan::get(format!(
                "graph/v1/paper/{}/{}",
                encode_path_segment(id),
                subresource
            ))
            .query("fields", fields)
            .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub async fn recommendations_for_paper(
        &self,
        paper_id: &str,
        limit: usize,
    ) -> Result<SemanticScholarRecommendationsResponse, BioMcpError> {
        let plan = Self::recommendations_for_paper_plan(paper_id, limit, self.api_key.as_deref())?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn recommendations_for_paper_plan(
        paper_id: &str,
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let paper_id = validate_paper_id(paper_id)?;
        let limit = validate_limit(limit)?;
        Ok(with_s2_api_key(
            RequestPlan::get(format!(
                "recommendations/v1/papers/forpaper/{}",
                encode_path_segment(paper_id)
            ))
            .query("fields", RECOMMENDATION_FIELDS)
            .query("limit", limit.to_string()),
            api_key,
        ))
    }

    pub async fn recommendations(
        &self,
        positive_paper_ids: &[String],
        negative_paper_ids: &[String],
        limit: usize,
    ) -> Result<SemanticScholarRecommendationsResponse, BioMcpError> {
        let plan = Self::recommendations_plan(
            positive_paper_ids,
            negative_paper_ids,
            limit,
            self.api_key.as_deref(),
        )?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.send_json(req).await
    }

    pub(crate) fn recommendations_plan(
        positive_paper_ids: &[String],
        negative_paper_ids: &[String],
        limit: usize,
        api_key: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        if positive_paper_ids.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Semantic Scholar recommendations require at least one positive paper".into(),
            ));
        }
        let limit = validate_limit(limit)?;
        let mut plan = RequestPlan::post("recommendations/v1/papers/")
            .query("fields", RECOMMENDATION_FIELDS)
            .query("limit", limit.to_string());
        plan.body = RequestBody::Json(json!({
            "positivePaperIds": positive_paper_ids,
            "negativePaperIds": negative_paper_ids,
        }));
        Ok(with_s2_api_key(plan, api_key))
    }
}

fn effective_api_key(
    policy: &ProviderUrlPolicy,
    base: &reqwest::Url,
    candidate: Option<String>,
) -> Option<String> {
    policy
        .is_credential_origin(base)
        .then_some(candidate)
        .flatten()
}

fn clean_api_key(api_key: Option<&str>) -> Option<&str> {
    api_key.map(str::trim).filter(|key| !key.is_empty())
}

fn with_s2_api_key(mut plan: RequestPlan, api_key: Option<&str>) -> RequestPlan {
    if let Some(key) = clean_api_key(api_key) {
        plan = plan.header("x-api-key", key);
    }
    plan
}

// dead-code reason: semantic_scholar::semantic_scholar_query_key preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
fn semantic_scholar_query_key(key: &str) -> &'static str {
    match key {
        "query" => "query",
        "fields" => "fields",
        "limit" => "limit",
        "year" => "year",
        _ => unreachable!("unexpected Semantic Scholar query key: {key}"),
    }
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b':' | b'@' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

// dead-code reason: semantic_scholar::validate_author_id preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
fn validate_author_id(author_id: &str) -> Result<&str, BioMcpError> {
    let author_id = author_id.trim();
    if author_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar author ID is required".into(),
        ));
    }
    if matches!(author_id, "." | "..") {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar author ID cannot be a path dot segment".into(),
        ));
    }
    if author_id.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar author ID is too long".into(),
        ));
    }
    Ok(author_id)
}

// dead-code reason: semantic_scholar::validate_author_page_limit preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
fn validate_author_page_limit(limit: usize) -> Result<usize, BioMcpError> {
    if limit == 0 || limit > SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX {
        return Err(BioMcpError::InvalidArgument(format!(
            "Semantic Scholar author page limit must be between 1 and {SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX}"
        )));
    }
    Ok(limit)
}

fn validate_paper_id(id: &str) -> Result<&str, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar paper ID is required".into(),
        ));
    }
    if id.len() > 512 {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar paper ID is too long".into(),
        ));
    }
    Ok(id)
}

fn validate_limit(limit: usize) -> Result<usize, BioMcpError> {
    if limit == 0 || limit > 100 {
        return Err(BioMcpError::InvalidArgument(
            "Semantic Scholar --limit must be between 1 and 100".into(),
        ));
    }
    Ok(limit)
}

// dead-code reason: semantic_scholar::SemanticScholarAuthor preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarAuthor {
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    pub name: Option<String>,
    pub affiliations: Option<Vec<String>>,
    #[serde(rename = "externalIds")]
    pub external_ids: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(rename = "paperCount")]
    pub paper_count: Option<u64>,
    #[serde(rename = "citationCount")]
    pub citation_count: Option<u64>,
    #[serde(rename = "hIndex")]
    pub h_index: Option<u64>,
}

// dead-code reason: semantic_scholar::SemanticScholarAuthorPaper preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarAuthorPaper {
    #[serde(rename = "paperId")]
    pub paper_id: Option<String>,
    #[serde(rename = "corpusId")]
    pub corpus_id: Option<u64>,
    #[serde(rename = "externalIds")]
    pub external_ids: Option<serde_json::Map<String, serde_json::Value>>,
    pub title: Option<String>,
    pub venue: Option<String>,
    pub year: Option<u32>,
    pub authors: Option<Vec<SemanticScholarAuthorPaperAuthor>>,
}

// dead-code reason: semantic_scholar::SemanticScholarAuthorPaperAuthor preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarAuthorPaperAuthor {
    #[serde(rename = "authorId")]
    pub author_id: Option<String>,
    pub name: Option<String>,
}

// dead-code reason: semantic_scholar::SemanticScholarAuthorSearchResponse preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarAuthorSearchResponse {
    pub total: Option<u64>,
    pub offset: Option<u64>,
    pub next: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<SemanticScholarAuthor>,
}

// dead-code reason: semantic_scholar::SemanticScholarAuthorPapersResponse preserves the provider shape used by source contract fixtures
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarAuthorPapersResponse {
    pub offset: Option<u64>,
    pub next: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<SemanticScholarAuthorPaper>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarPaper {
    #[serde(rename = "paperId")]
    pub paper_id: Option<String>,
    #[serde(rename = "externalIds")]
    pub external_ids: Option<SemanticScholarExternalIds>,
    pub title: Option<String>,
    pub venue: Option<String>,
    pub year: Option<u32>,
    #[serde(rename = "citationCount")]
    pub citation_count: Option<u64>,
    #[serde(rename = "influentialCitationCount")]
    pub influential_citation_count: Option<u64>,
    #[serde(rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(rename = "referenceCount")]
    pub reference_count: Option<u64>,
    #[serde(rename = "isOpenAccess")]
    pub is_open_access: Option<bool>,
    #[serde(rename = "openAccessPdf")]
    pub open_access_pdf: Option<SemanticScholarOpenAccessPdf>,
    pub authors: Option<Vec<SemanticScholarAuthor>>,
    pub tldr: Option<SemanticScholarTldr>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarSearchResponse {
    pub total: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<SemanticScholarPaper>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarExternalIds {
    #[serde(rename = "PubMed")]
    pub pubmed: Option<String>,
    #[serde(rename = "PubMedCentral")]
    pub pmcid: Option<String>,
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    #[serde(rename = "ArXiv")]
    pub arxiv: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarOpenAccessPdf {
    pub url: Option<String>,
    pub status: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarTldr {
    pub text: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct SemanticScholarGraphResponse<T> {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<T>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarCitationEdge {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub contexts: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub intents: Vec<String>,
    #[serde(rename = "isInfluential")]
    pub is_influential: Option<bool>,
    #[serde(rename = "citingPaper")]
    pub citing_paper: SemanticScholarPaper,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SemanticScholarReferenceEdge {
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub contexts: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub intents: Vec<String>,
    #[serde(rename = "isInfluential")]
    pub is_influential: Option<bool>,
    #[serde(rename = "citedPaper")]
    pub cited_paper: SemanticScholarPaper,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SemanticScholarRecommendationsResponse {
    #[serde(rename = "recommendedPapers", default)]
    pub recommended_papers: Vec<SemanticScholarPaper>,
}

#[cfg(test)]
mod tests;
