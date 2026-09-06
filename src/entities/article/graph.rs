//! Article citation, reference, and recommendation graph helpers.

use std::collections::HashSet;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
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
    }
}

fn graph_edge_from_reference(edge: SemanticScholarReferenceEdge) -> ArticleGraphEdge {
    ArticleGraphEdge {
        paper: related_paper_from_semantic_scholar(&edge.cited_paper),
        intents: edge.intents,
        contexts: edge.contexts,
        is_influential: edge.is_influential.unwrap_or(false),
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
    Ok(ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_citation)
            .collect(),
        pagination,
        _meta,
    })
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
    Ok(ArticleGraphResult {
        article,
        edges: response
            .data
            .into_iter()
            .map(graph_edge_from_reference)
            .collect(),
        pagination,
        _meta,
    })
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
