//! Article citation, reference, and recommendation graph helpers.

use std::collections::HashSet;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::semantic_scholar::{
    SemanticScholarCitationEdge, SemanticScholarClient, SemanticScholarPaper,
    SemanticScholarReferenceEdge,
};

use super::detail::{
    article_not_found, first_europepmc_hit, is_doi, parse_arxiv_id, parse_pmcid, parse_pmid,
};
use super::{
    ArticleGraphEdge, ArticleGraphMeta, ArticleGraphPagination, ArticleGraphResult,
    ArticleRecommendationsResult, ArticleRelatedPaper, GraphCoverageStatus,
};
use crate::entities::author::{
    ArticleAuthorRecord, ArticleAuthorsResult, AuthorAssertion, AuthorEvidence, AuthorEvidenceUrl,
    AuthorIdentity, AuthorMeta, AuthorSourceStatus, ProviderStatus, TemporalAnchor, evidence_url,
    nonblank, provider_id, sanitized_provider_error, valid_wire_id,
};

fn is_semantic_scholar_paper_id(id: &str) -> bool {
    id.len() == 40 && id.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn semantic_scholar_invalid_id(id: &str) -> BioMcpError {
    BioMcpError::InvalidArgument(format!(
        "Unsupported identifier format for Semantic Scholar article helpers: '{id}'. Supported: PMID, PMCID, DOI, arXiv, or a Semantic Scholar paper ID."
    ))
}

pub(super) fn semantic_scholar_lookup_id(id: &str) -> Option<String> {
    let id = id.trim();
    if let Some(pmid) = parse_pmid(id) {
        return Some(format!("PMID:{pmid}"));
    }
    if is_doi(id) {
        return Some(format!("DOI:{id}"));
    }
    if let Some(arxiv) = parse_arxiv_id(id) {
        return Some(arxiv);
    }
    if let Some((year_month, sequence)) = id.split_once('.')
        && !year_month.is_empty()
        && year_month.chars().all(|ch| ch.is_ascii_digit())
        && !sequence.is_empty()
        && sequence.chars().all(|ch| ch.is_ascii_digit())
    {
        return Some(format!("ARXIV:{id}"));
    }
    if is_semantic_scholar_paper_id(id) {
        return Some(id.to_string());
    }
    None
}

fn related_paper_from_semantic_scholar(paper: &SemanticScholarPaper) -> ArticleRelatedPaper {
    let external_ids = paper.external_ids.as_ref();
    ArticleRelatedPaper {
        paper_id: paper.paper_id.clone(),
        pmid: external_ids.and_then(|ids| ids.pubmed.clone()),
        doi: external_ids.and_then(|ids| ids.doi.clone()),
        arxiv_id: external_ids.and_then(|ids| ids.arxiv.clone()),
        title: paper.title.clone().unwrap_or_default().trim().to_string(),
        journal: paper
            .venue
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        year: paper.year,
    }
}

async fn resolve_semantic_scholar_input_id(
    id: &str,
    europe: &EuropePmcClient,
) -> Result<String, BioMcpError> {
    if let Some(id) = semantic_scholar_lookup_id(id) {
        return Ok(id);
    }

    if let Some(pmcid) = parse_pmcid(id) {
        let search = europe.search_by_pmcid(&pmcid).await?;
        let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&pmcid, id))?;
        if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
            return Ok(format!("PMID:{pmid}"));
        }
        if let Some(doi) = hit
            .doi
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(format!("DOI:{doi}"));
        }
        return Err(article_not_found(&pmcid, id));
    }

    Err(semantic_scholar_invalid_id(id))
}

async fn resolve_semantic_scholar_seed(
    id: &str,
    client: &SemanticScholarClient,
    europe: &EuropePmcClient,
) -> Result<ArticleRelatedPaper, BioMcpError> {
    let lookup_id = resolve_semantic_scholar_input_id(id, europe).await?;
    let mut rows = client.paper_batch(&[lookup_id]).await?;
    let paper = rows
        .pop()
        .flatten()
        .ok_or_else(|| article_not_found(id, id))?;
    Ok(related_paper_from_semantic_scholar(&paper))
}

fn dedup_related_papers(rows: Vec<ArticleRelatedPaper>) -> Vec<ArticleRelatedPaper> {
    let mut seen: HashSet<String> = HashSet::with_capacity(rows.len());
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let key = row
            .paper_id
            .as_deref()
            .map(str::to_string)
            .or_else(|| row.pmid.as_deref().map(|value| format!("pmid:{value}")))
            .or_else(|| row.doi.as_deref().map(|value| format!("doi:{value}")))
            .or_else(|| {
                row.arxiv_id
                    .as_deref()
                    .map(|value| format!("arxiv:{value}"))
            })
            .unwrap_or_else(|| row.title.clone());
        if seen.insert(key) {
            out.push(row);
        }
    }
    out
}

async fn resolve_semantic_scholar_seeds(
    ids: &[String],
    client: &SemanticScholarClient,
    europe: &EuropePmcClient,
) -> Result<Vec<ArticleRelatedPaper>, BioMcpError> {
    let mut out = Vec::with_capacity(ids.len());
    for id in ids {
        out.push(resolve_semantic_scholar_seed(id, client, europe).await?);
    }
    Ok(dedup_related_papers(out))
}

fn graph_edge_from_citation(edge: SemanticScholarCitationEdge) -> ArticleGraphEdge {
    ArticleGraphEdge {
        paper: related_paper_from_semantic_scholar(&edge.citing_paper),
        intents: edge.intents,
        contexts: edge.contexts,
        is_influential: edge.is_influential.unwrap_or(false),
        _meta: None,
    }
}

fn graph_edge_from_reference(edge: SemanticScholarReferenceEdge) -> ArticleGraphEdge {
    ArticleGraphEdge {
        paper: related_paper_from_semantic_scholar(&edge.cited_paper),
        intents: edge.intents,
        contexts: edge.contexts,
        is_influential: edge.is_influential.unwrap_or(false),
        _meta: None,
    }
}

fn attach_edge_evidence_meta(
    result: &mut ArticleGraphResult,
    direction: GraphDirection,
    caller_id: &str,
) {
    for edge in &mut result.edges {
        edge._meta = graph_edge_evidence_meta(edge, direction, caller_id);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphDirection {
    Citations,
    References,
}

impl GraphDirection {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Citations => "citations",
            Self::References => "references",
        }
    }
}

fn graph_continuation_command(
    caller_id: &str,
    direction: GraphDirection,
    limit: usize,
    next_offset: u64,
) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["article", direction.as_str()])
        .arg(caller_id.trim())
        .args([
            "--limit",
            &limit.to_string(),
            "--offset",
            &next_offset.to_string(),
        ])
        .render_shell()
}

fn graph_page_contract<T>(
    response: &crate::sources::semantic_scholar::SemanticScholarGraphResponse<T>,
    requested_offset: u64,
    limit: usize,
    caller_id: &str,
    direction: GraphDirection,
) -> Result<(ArticleGraphPagination, ArticleGraphMeta), BioMcpError> {
    let provider_offset = response.offset.ok_or_else(|| BioMcpError::Api {
        api: "semantic-scholar".into(),
        message: "graph response omitted its required offset".into(),
    })?;
    if provider_offset != requested_offset {
        return Err(BioMcpError::Api {
            api: "semantic-scholar".into(),
            message: "graph response offset did not match the request".into(),
        });
    }
    if response.next.is_some_and(|next| next <= provider_offset) {
        return Err(BioMcpError::Api {
            api: "semantic-scholar".into(),
            message: "graph response continuation did not advance".into(),
        });
    }
    let next_commands = response
        .next
        .map(|next| graph_continuation_command(caller_id, direction, limit, next))
        .into_iter()
        .collect();
    Ok((
        ArticleGraphPagination {
            offset: provider_offset,
            limit,
            returned: response.data.len(),
            next_offset: response.next,
            coverage_status: if response.next.is_some() {
                GraphCoverageStatus::Continuable
            } else {
                GraphCoverageStatus::Exhausted
            },
        },
        ArticleGraphMeta { next_commands },
    ))
}

fn article_graph_from_citations(
    article: ArticleRelatedPaper,
    response: crate::sources::semantic_scholar::SemanticScholarGraphResponse<
        SemanticScholarCitationEdge,
    >,
    requested_offset: u64,
    limit: usize,
    caller_id: &str,
) -> Result<ArticleGraphResult, BioMcpError> {
    let (pagination, _meta) = graph_page_contract(
        &response,
        requested_offset,
        limit,
        caller_id,
        GraphDirection::Citations,
    )?;
    let mut result = ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_citation)
            .collect(),
        pagination,
        _meta,
    };
    attach_edge_evidence_meta(&mut result, GraphDirection::Citations, caller_id);
    Ok(result)
}

fn article_graph_from_references(
    article: ArticleRelatedPaper,
    response: crate::sources::semantic_scholar::SemanticScholarGraphResponse<
        SemanticScholarReferenceEdge,
    >,
    requested_offset: u64,
    limit: usize,
    caller_id: &str,
) -> Result<ArticleGraphResult, BioMcpError> {
    let (pagination, _meta) = graph_page_contract(
        &response,
        requested_offset,
        limit,
        caller_id,
        GraphDirection::References,
    )?;
    let mut result = ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_reference)
            .collect(),
        pagination,
        _meta,
    };
    attach_edge_evidence_meta(&mut result, GraphDirection::References, caller_id);
    Ok(result)
}

fn article_recommendations_from_response(
    positive_seeds: Vec<ArticleRelatedPaper>,
    negative_seeds: Vec<ArticleRelatedPaper>,
    response: crate::sources::semantic_scholar::SemanticScholarRecommendationsResponse,
) -> ArticleRecommendationsResult {
    ArticleRecommendationsResult {
        positive_seeds,
        negative_seeds,
        recommendations: response
            .recommended_papers
            .into_iter()
            .map(|paper| related_paper_from_semantic_scholar(&paper))
            .collect(),
    }
}

pub async fn authors(id: &str) -> Result<ArticleAuthorsResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let lookup_id = resolve_semantic_scholar_input_id(id, &europe).await?;
    let paper = client
        .paper_authors(&lookup_id)
        .await
        .map_err(sanitized_provider_error)?;
    let article = related_paper_from_semantic_scholar(&paper);
    let observed_at = chrono::Utc::now().to_rfc3339();
    let authors = paper
        .authors
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let author_id = valid_wire_id(row.author_id)?;
            let display_name = nonblank(row.name)?;
            let url = evidence_url(&author_id);
            let affiliations = row
                .affiliations
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| nonblank(Some(value)))
                .map(|value| AuthorAssertion {
                    value,
                    evidence: AuthorEvidence {
                        source: "semantic_scholar",
                        url: url.clone(),
                    },
                    temporal: TemporalAnchor::ObservedAt(observed_at.clone()),
                })
                .collect();
            Some(ArticleAuthorRecord {
                identity: AuthorIdentity::ExactProvider {
                    id: provider_id(author_id),
                },
                display_name,
                affiliations,
            })
        })
        .collect::<Vec<_>>();
    let evidence_urls = authors
        .iter()
        .map(|author| {
            let AuthorIdentity::ExactProvider { id } = &author.identity;
            AuthorEvidenceUrl {
                source: "semantic_scholar",
                url: evidence_url(&id.value),
            }
        })
        .collect();
    let next_commands = authors
        .iter()
        .map(|author| {
            let AuthorIdentity::ExactProvider { id } = &author.identity;
            format!("biomcp get author {id}")
        })
        .collect();
    Ok(ArticleAuthorsResult {
        article,
        authors,
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "semantic_scholar",
                status: ProviderStatus::Available,
            }],
            evidence_urls,
            next_commands,
        },
    })
}

pub async fn citations(
    id: &str,
    limit: usize,
    offset: u64,
) -> Result<ArticleGraphResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let article = resolve_semantic_scholar_seed(id, &client, &europe).await?;
    let graph_id = article
        .paper_id
        .as_deref()
        .map(str::to_string)
        .ok_or_else(|| article_not_found(id, id))?;
    let response = client.paper_citations(&graph_id, limit, offset).await?;

    article_graph_from_citations(article, response, offset, limit, id)
}

pub async fn references(
    id: &str,
    limit: usize,
    offset: u64,
) -> Result<ArticleGraphResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let article = resolve_semantic_scholar_seed(id, &client, &europe).await?;
    let graph_id = article
        .paper_id
        .as_deref()
        .map(str::to_string)
        .ok_or_else(|| article_not_found(id, id))?;
    let response = client.paper_references(&graph_id, limit, offset).await?;

    article_graph_from_references(article, response, offset, limit, id)
}

pub async fn recommendations(
    ids: &[String],
    negative: &[String],
    limit: usize,
) -> Result<ArticleRecommendationsResult, BioMcpError> {
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let positive_seeds = resolve_semantic_scholar_seeds(ids, &client, &europe).await?;
    let negative_seeds = resolve_semantic_scholar_seeds(negative, &client, &europe).await?;
    if positive_seeds.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one positive article seed is required. Example: biomcp article recommendations 22663011".into(),
        ));
    }

    let positive_ids: Vec<String> = positive_seeds
        .iter()
        .filter_map(|paper| paper.paper_id.clone())
        .collect();
    let negative_ids: Vec<String> = negative_seeds
        .iter()
        .filter_map(|paper| paper.paper_id.clone())
        .collect();
    let positive_set: HashSet<&str> = positive_ids.iter().map(String::as_str).collect();
    if let Some(conflict) = negative_ids
        .iter()
        .map(String::as_str)
        .find(|paper_id| positive_set.contains(paper_id))
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "The same paper cannot appear in both positive and negative recommendation seeds ({conflict})"
        )));
    }

    let response = if positive_ids.len() == 1 && negative_ids.is_empty() {
        client
            .recommendations_for_paper(&positive_ids[0], limit)
            .await?
    } else {
        client
            .recommendations(&positive_ids, &negative_ids, limit)
            .await?
    };

    Ok(article_recommendations_from_response(
        positive_seeds,
        negative_seeds,
        response,
    ))
}

#[cfg(test)]
mod tests;

// --- Ticket 1145: directed citation evidence ---

use crate::transform::article::{
    JatsCitationExtraction, JatsCitationTargetIds,
    extract_citation_evidence as real_jats_extraction,
};
use std::time::Duration;

const CITATION_EVIDENCE_PAGE_SIZE: usize = 100;
const CITATION_EVIDENCE_MAX_PAGES: usize = 3;
const CITATION_EVIDENCE_COMMAND_DEADLINE: Duration = Duration::from_secs(22);
const CITATION_EVIDENCE_GRAPH_DEADLINE: Duration = Duration::from_secs(10);

const SEMANTIC_SCHOLAR_SOURCE: &str = "semantic_scholar";
const EUROPE_PMC_JATS_SOURCE: &str = "europe_pmc_jats";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CitationEvidenceStatus {
    ContextFromProvider,
    ContextFromFulltext,
    FulltextUnavailable,
    ReferenceUnresolved,
    CitationMarkerUnlinked,
}

impl CitationEvidenceStatus {
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::ContextFromProvider => {
                "Semantic Scholar supplied citation context for this directed edge."
            }
            Self::ContextFromFulltext => {
                "Open-access JATS linked the cited reference to the returned passage."
            }
            Self::FulltextUnavailable => {
                "Structured open full text was unavailable for the citing paper."
            }
            Self::ReferenceUnresolved => {
                "Structured full text was available, but the cited reference could not be resolved exactly."
            }
            Self::CitationMarkerUnlinked => {
                "The cited reference was resolved, but no unambiguous in-text citation marker linked to it."
            }
        }
    }

    const fn source(self) -> Option<&'static str> {
        match self {
            Self::ContextFromProvider => Some(SEMANTIC_SCHOLAR_SOURCE),
            Self::ContextFromFulltext
            | Self::ReferenceUnresolved
            | Self::CitationMarkerUnlinked => Some(EUROPE_PMC_JATS_SOURCE),
            Self::FulltextUnavailable => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidenceSourceStatus {
    pub source: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidenceUrl {
    pub source: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidenceMeta {
    pub source_status: Vec<CitationEvidenceSourceStatus>,
    pub evidence_urls: Vec<CitationEvidenceUrl>,
    pub next_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidencePassageLocator {
    pub pmcid: String,
    pub ref_id: String,
    pub section_path: Vec<String>,
    pub paragraph: usize,
    pub marker: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidencePassage {
    pub text: String,
    pub locator: CitationEvidencePassageLocator,
    pub evidence_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidenceLocator {
    pub pmcid: String,
    pub evidence_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ArticleCitationEvidenceResult {
    pub citing: super::ArticleRelatedPaper,
    pub cited: super::ArticleRelatedPaper,
    pub status: CitationEvidenceStatus,
    pub message: String,
    pub source: Option<String>,
    pub provider_contexts: Vec<String>,
    pub passages: Vec<CitationEvidencePassage>,
    pub fulltext_locator: Option<CitationEvidenceLocator>,
    pub _meta: CitationEvidenceMeta,
}

/// The shared evidence command: `biomcp article citation-evidence <citing> <cited>`.
pub(crate) fn citation_evidence_command(citing_id: &str, cited_id: &str) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["article", "citation-evidence"])
        .arg(citing_id.trim())
        .arg(cited_id.trim())
        .render_shell()
}

/// The executable edge ID for the evidence command, using the existing
/// related-paper preference: PMID, then DOI, then arXiv, then paper ID.
fn edge_paper_command_id(paper: &super::ArticleRelatedPaper) -> Option<String> {
    paper
        .pmid
        .clone()
        .or_else(|| paper.doi.clone())
        .or_else(|| paper.arxiv_id.clone())
        .or_else(|| paper.paper_id.clone())
        .filter(|value| !value.trim().is_empty())
}

/// Graph edges whose contexts contain no nonblank value carry an edge-local
/// next command for the evidence surface. On `references` the caller anchors
/// the citing side; on `citations` the edge paper cites the caller.
fn graph_edge_evidence_meta(
    edge: &super::ArticleGraphEdge,
    direction: GraphDirection,
    caller_id: &str,
) -> Option<GraphEdgeMeta> {
    if edge.contexts.iter().any(|value| !value.trim().is_empty()) {
        return None;
    }
    let caller_id = caller_id.trim();
    if caller_id.is_empty() {
        return None;
    }
    let edge_id = edge_paper_command_id(&edge.paper)?;
    let command = match direction {
        GraphDirection::References => citation_evidence_command(caller_id, &edge_id),
        GraphDirection::Citations => citation_evidence_command(&edge_id, caller_id),
    };
    Some(GraphEdgeMeta {
        next_commands: vec![command],
    })
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GraphEdgeMeta {
    pub next_commands: Vec<String>,
}

fn provider_decode_error(message: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "semantic-scholar".into(),
        message: message.to_string(),
    }
}

fn bounded_unavailable_error(message: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "semantic-scholar".into(),
        message: message.to_string(),
    }
}

fn command_deadline_error() -> BioMcpError {
    BioMcpError::Api {
        api: "article-citation-evidence".into(),
        message: "citation-evidence command deadline exceeded".into(),
    }
}

fn valid_paper_id(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|id| {
            !id.is_empty() && id.len() == 40 && id.chars().all(|ch| ch.is_ascii_hexdigit())
        })
        .map(str::to_string)
}

async fn resolve_citation_seed(
    id: &str,
    client: &SemanticScholarClient,
    europe: &EuropePmcClient,
) -> Result<(super::ArticleRelatedPaper, SemanticScholarPaper), BioMcpError> {
    let lookup_id = resolve_semantic_scholar_input_id(id, europe).await?;
    let rows = client.paper_batch(&[lookup_id]).await?;
    if rows.len() != 1 {
        return Err(provider_decode_error(
            "paper batch response must contain exactly one row",
        ));
    }
    let paper = rows
        .into_iter()
        .next()
        .flatten()
        .ok_or_else(|| provider_decode_error("paper batch row was null"))?;
    if valid_paper_id(paper.paper_id.as_deref()).is_none() {
        return Err(provider_decode_error(
            "paper batch response omitted a valid 40-character paper ID",
        ));
    }
    Ok((related_paper_from_semantic_scholar(&paper), paper))
}

fn validate_evidence_page<T>(
    response: &crate::sources::semantic_scholar::SemanticScholarGraphResponse<T>,
    requested_offset: u64,
) -> Result<Option<u64>, BioMcpError> {
    let provider_offset = response
        .offset
        .ok_or_else(|| provider_decode_error("graph response omitted its required offset"))?;
    if provider_offset != requested_offset {
        return Err(provider_decode_error(
            "graph response offset did not match the request",
        ));
    }
    if let Some(next) = response.next
        && next <= provider_offset
    {
        return Err(provider_decode_error(
            "graph response continuation did not advance",
        ));
    }
    if response.data.len() > CITATION_EVIDENCE_PAGE_SIZE {
        return Err(provider_decode_error(
            "graph response returned more rows than the page size",
        ));
    }
    Ok(response.next)
}

fn normalized_contexts(
    edges: &[&crate::sources::semantic_scholar::SemanticScholarReferenceEdge],
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for edge in edges {
        for context in &edge.contexts {
            let trimmed = context.trim();
            if trimmed.is_empty() || out.iter().any(|existing| existing == trimmed) {
                continue;
            }
            out.push(trimmed.to_string());
        }
    }
    out
}

fn evidence_deadline() -> tokio::time::Instant {
    tokio::time::Instant::now() + CITATION_EVIDENCE_COMMAND_DEADLINE
}

enum EvidenceGraphOutcome {
    Matched(Vec<String>),
    ExhaustedWithoutMatch,
}

async fn directed_edge_contexts(
    client: &SemanticScholarClient,
    citing_pid: &str,
    cited_pid: &str,
    deadline: tokio::time::Instant,
) -> Result<EvidenceGraphOutcome, BioMcpError> {
    let now = tokio::time::Instant::now();
    let graph_deadline = now + CITATION_EVIDENCE_GRAPH_DEADLINE;
    let graph_deadline = if graph_deadline < deadline {
        graph_deadline
    } else {
        deadline
    };
    let mut offset = 0_u64;
    for _page in 0..CITATION_EVIDENCE_MAX_PAGES {
        if tokio::time::Instant::now() >= graph_deadline {
            return Err(bounded_unavailable_error(
                "Semantic Scholar directed reference traversal exceeded its bounded deadline",
            ));
        }
        let request = client.paper_references(citing_pid, CITATION_EVIDENCE_PAGE_SIZE, offset);
        let response = match tokio::time::timeout_at(graph_deadline, request).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(bounded_unavailable_error(
                    "Semantic Scholar directed reference traversal exceeded its bounded deadline",
                ));
            }
        };
        let next = validate_evidence_page(&response, offset)?;
        let mut matching = Vec::new();
        for edge in &response.data {
            let edge_id =
                valid_paper_id(edge.cited_paper.paper_id.as_deref()).ok_or_else(|| {
                    provider_decode_error("reference edge omitted a valid cited paper ID")
                })?;
            if edge_id.eq_ignore_ascii_case(cited_pid) {
                matching.push(edge);
            }
        }
        if !matching.is_empty() {
            return Ok(EvidenceGraphOutcome::Matched(normalized_contexts(
                &matching,
            )));
        }
        match next {
            Some(next_offset) => offset = next_offset,
            None => return Ok(EvidenceGraphOutcome::ExhaustedWithoutMatch),
        }
    }
    Err(bounded_unavailable_error(
        "Semantic Scholar directed reference traversal exceeded its three-page bound",
    ))
}

fn cited_target_ids(paper: &SemanticScholarPaper) -> JatsCitationTargetIds {
    let external = paper.external_ids.as_ref();
    JatsCitationTargetIds {
        doi: external
            .and_then(|ids| ids.doi.as_deref())
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| value.starts_with("10.") && value.contains('/')),
        pmid: external
            .and_then(|ids| ids.pubmed.as_deref())
            .map(str::trim)
            .filter(|value| value.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|digits| digits.parse::<u64>().ok())
            .map(|decimal| decimal.to_string()),
        pmcid: external
            .and_then(|ids| ids.pmcid.as_deref())
            .map(|value| value.trim().to_ascii_uppercase())
            .and_then(|upper| upper.strip_prefix("PMC").map(str::to_string))
            .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|digits| digits.parse::<u64>().ok())
            .map(|decimal| format!("PMC{decimal}")),
    }
}

type JatsCitationParseFn = fn(&str, &JatsCitationTargetIds) -> Result<JatsCitationExtraction, ()>;

fn jats_citation_parse_fn() -> Option<JatsCitationParseFn> {
    #[cfg(test)]
    {
        if let Some(seam) = jats_citation_test_seam() {
            return Some(seam);
        }
    }
    let real: JatsCitationParseFn = |xml, target| real_jats_extraction(xml, target);
    Some(real)
}

#[cfg(test)]
fn jats_citation_test_seam() -> Option<JatsCitationParseFn> {
    JATS_CITATION_SEAM.with(|cell| *cell.borrow())
}

#[cfg(test)]
thread_local! {
    static JATS_CITATION_SEAM: std::cell::RefCell<Option<JatsCitationParseFn>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn install_jats_citation_seam(seam: Option<JatsCitationParseFn>) {
    JATS_CITATION_SEAM.with(|cell| *cell.borrow_mut() = seam);
}

fn jats_worker_permit() -> &'static tokio::sync::Semaphore {
    static PERMIT: std::sync::OnceLock<tokio::sync::Semaphore> = std::sync::OnceLock::new();
    PERMIT.get_or_init(|| tokio::sync::Semaphore::new(1))
}

#[derive(Debug)]
enum JatsEvidenceOutcome {
    FulltextUnavailable,
    Parsed {
        pmcid: String,
        extraction: JatsCitationExtraction,
    },
}

async fn jats_citation_outcome(
    citing_paper: &SemanticScholarPaper,
    cited_paper: &SemanticScholarPaper,
    deadline: tokio::time::Instant,
) -> Result<JatsEvidenceOutcome, BioMcpError> {
    let Some(pmcid) = citing_pmcid(citing_paper).await? else {
        return Ok(JatsEvidenceOutcome::FulltextUnavailable);
    };
    let Some(xml) = fetch_citing_fulltext(&pmcid, deadline).await? else {
        return Ok(JatsEvidenceOutcome::FulltextUnavailable);
    };
    run_jats_extraction(&pmcid, &xml, &cited_target_ids(cited_paper), deadline).await
}

async fn citing_pmcid(paper: &SemanticScholarPaper) -> Result<Option<String>, BioMcpError> {
    let external = paper.external_ids.as_ref();
    if let Some(pmcid) = external
        .and_then(|ids| ids.pmcid.as_deref())
        .map(|value| value.trim().to_ascii_uppercase())
        .and_then(|upper| upper.strip_prefix("PMC").map(str::to_string))
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|digits| digits.parse::<u64>().ok())
        .map(|decimal| format!("PMC{decimal}"))
    {
        return Ok(Some(pmcid));
    }
    let bridge = NcbiIdConverterClient::new()?;
    let resolved = match (
        external.and_then(|ids| ids.pubmed.as_deref()),
        external.and_then(|ids| ids.doi.as_deref()),
    ) {
        (Some(pmid), _) => bridge.pmid_to_pmcid(pmid.trim()).await,
        (None, Some(doi)) => bridge.doi_to_pmcid(doi.trim()).await,
        (None, None) => Ok(None),
    };
    // A bridge failure or timeout leaves the JATS attempt unavailable
    // without failing the command.
    Ok(resolved.unwrap_or(None))
}

async fn fetch_citing_fulltext(
    pmcid: &str,
    deadline: tokio::time::Instant,
) -> Result<Option<String>, BioMcpError> {
    let europe = EuropePmcClient::new()?;
    let fetched = tokio::time::timeout_at(deadline, europe.get_full_text_xml("PMC", pmcid)).await;
    match fetched {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) | Err(_) => Ok(None),
    }
}

async fn run_jats_extraction(
    pmcid: &str,
    xml: &str,
    target: &JatsCitationTargetIds,
    deadline: tokio::time::Instant,
) -> Result<JatsEvidenceOutcome, BioMcpError> {
    let permit =
        match tokio::time::timeout_at(deadline, async { jats_worker_permit().acquire().await })
            .await
        {
            Ok(Ok(guard)) => guard,
            _ => return Err(command_deadline_error()),
        };
    let xml = xml.to_string();
    let target = target.clone();
    let parse = jats_citation_parse_fn();
    let mut worker = tokio::task::spawn_blocking(move || {
        // The permit is owned by the worker: success, parse failure, panic,
        // and late completion each release it exactly once.
        let _held = permit;
        match parse {
            Some(parse) => parse(&xml, &target),
            None => real_jats_extraction(&xml, &target),
        }
    });
    match tokio::time::timeout_at(deadline, &mut worker).await {
        Ok(Ok(Ok(extraction))) => Ok(JatsEvidenceOutcome::Parsed {
            pmcid: pmcid.to_string(),
            extraction,
        }),
        // Parse failure, an unusable document, and a panicked worker all
        // bound into the public fulltext_unavailable message.
        Ok(Ok(Err(()))) | Ok(Err(_)) => Ok(JatsEvidenceOutcome::FulltextUnavailable),
        Err(_) => Err(command_deadline_error()),
    }
}

fn europepmc_jats_url(pmcid: &str) -> String {
    format!("https://www.ebi.ac.uk/europepmc/webservices/rest/{pmcid}/fullTextXML")
}

fn semantic_scholar_paper_url(paper_id: &str) -> String {
    format!("https://www.semanticscholar.org/paper/{paper_id}")
}

pub async fn citation_evidence(
    citing_id: &str,
    cited_id: &str,
    force_fulltext: bool,
) -> Result<ArticleCitationEvidenceResult, BioMcpError> {
    let deadline = evidence_deadline();
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let (citing, citing_paper) = resolve_citation_seed(citing_id, &client, &europe).await?;
    let (cited, cited_paper) = resolve_citation_seed(cited_id, &client, &europe).await?;
    let citing_pid = valid_paper_id(citing.paper_id.as_deref())
        .ok_or_else(|| provider_decode_error("citing seed lacks a valid paper ID"))?;
    let cited_pid = valid_paper_id(cited.paper_id.as_deref())
        .ok_or_else(|| provider_decode_error("cited seed lacks a valid paper ID"))?;

    let contexts = match directed_edge_contexts(&client, &citing_pid, &cited_pid, deadline).await? {
        EvidenceGraphOutcome::Matched(contexts) => Some(contexts),
        EvidenceGraphOutcome::ExhaustedWithoutMatch => {
            return Err(BioMcpError::NotFound {
                entity: "directed citation".into(),
                id: format!("{} -> {}", citing_id.trim(), cited_id.trim()),
                suggestion: "Semantic Scholar exhausted the directed reference pages without finding this pair.".into(),
            });
        }
    };

    let provider_contexts = contexts.unwrap_or_default();
    let mut jats_urls_appended = false;
    let mut jats: Option<JatsEvidenceOutcome> = None;
    if force_fulltext || provider_contexts.is_empty() {
        jats = Some(jats_citation_outcome(&citing_paper, &cited_paper, deadline).await?);
    }

    let mut status = CitationEvidenceStatus::ContextFromProvider;
    let mut passages: Vec<CitationEvidencePassage> = Vec::new();
    let mut locator: Option<CitationEvidenceLocator> = None;
    let mut europepmc_status = if jats.is_none() {
        "not_requested"
    } else {
        "unavailable"
    };
    if let Some(JatsEvidenceOutcome::Parsed { pmcid, extraction }) = &jats {
        jats_urls_appended = true;
        let url = europepmc_jats_url(pmcid);
        match extraction {
            JatsCitationExtraction::Linked {
                ref_id,
                passages: extracted,
            } => {
                status = CitationEvidenceStatus::ContextFromFulltext;
                europepmc_status = "available";
                locator = Some(CitationEvidenceLocator {
                    pmcid: pmcid.clone(),
                    evidence_url: url.clone(),
                });
                passages = extracted
                    .iter()
                    .map(|passage| CitationEvidencePassage {
                        text: passage.text.clone(),
                        locator: CitationEvidencePassageLocator {
                            pmcid: pmcid.clone(),
                            ref_id: ref_id.clone(),
                            section_path: passage.section_path.clone(),
                            paragraph: passage.paragraph,
                            marker: passage.marker.clone(),
                        },
                        evidence_url: url.clone(),
                    })
                    .collect();
            }
            JatsCitationExtraction::ReferenceUnresolved => {
                status = CitationEvidenceStatus::ReferenceUnresolved;
                locator = Some(CitationEvidenceLocator {
                    pmcid: pmcid.clone(),
                    evidence_url: url.clone(),
                });
            }
            JatsCitationExtraction::MarkerUnlinked { .. } => {
                status = CitationEvidenceStatus::CitationMarkerUnlinked;
                locator = Some(CitationEvidenceLocator {
                    pmcid: pmcid.clone(),
                    evidence_url: url.clone(),
                });
            }
            JatsCitationExtraction::ParsedUnusable => {
                status = CitationEvidenceStatus::FulltextUnavailable;
            }
        }
    }

    let mut evidence_urls = vec![
        CitationEvidenceUrl {
            source: SEMANTIC_SCHOLAR_SOURCE.to_string(),
            url: semantic_scholar_paper_url(&citing_pid),
        },
        CitationEvidenceUrl {
            source: SEMANTIC_SCHOLAR_SOURCE.to_string(),
            url: semantic_scholar_paper_url(&cited_pid),
        },
    ];
    if jats_urls_appended && let Some(JatsEvidenceOutcome::Parsed { pmcid, .. }) = &jats {
        evidence_urls.push(CitationEvidenceUrl {
            source: EUROPE_PMC_JATS_SOURCE.to_string(),
            url: europepmc_jats_url(pmcid),
        });
    }

    Ok(ArticleCitationEvidenceResult {
        citing,
        cited,
        message: status.message().to_string(),
        source: status.source().map(str::to_string),
        provider_contexts,
        passages,
        fulltext_locator: locator,
        status,
        _meta: CitationEvidenceMeta {
            source_status: vec![
                CitationEvidenceSourceStatus {
                    source: SEMANTIC_SCHOLAR_SOURCE.to_string(),
                    status: "available".to_string(),
                },
                CitationEvidenceSourceStatus {
                    source: EUROPE_PMC_JATS_SOURCE.to_string(),
                    status: europepmc_status.to_string(),
                },
            ],
            evidence_urls,
            next_commands: Vec::new(),
        },
    })
}
