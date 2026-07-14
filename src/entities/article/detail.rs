//! Article detail lookup, identifier parsing, and full-text retrieval.

use crate::error::BioMcpError;
use crate::sources::europepmc::{EuropePmcClient, EuropePmcResult, EuropePmcSearchResponse};
use crate::sources::pubmed::{PubMedCitation, PubMedClient};
use crate::sources::pubtator::PubTatorClient;
use crate::sources::semantic_scholar::{SemanticScholarClient, SemanticScholarPaper};
use crate::transform;
use tracing::warn;

use super::{
    ARTICLE_SECTION_ALL, ARTICLE_SECTION_ANNOTATIONS, ARTICLE_SECTION_ASSET,
    ARTICLE_SECTION_ASSETS, ARTICLE_SECTION_FULLTEXT, ARTICLE_SECTION_INDEXING,
    ARTICLE_SECTION_NAMES, ARTICLE_SECTION_TLDR, Article, ArticleAffiliation,
    ArticleAffiliationIdentifier, ArticleGetOptions, ArticleIndexing, ArticleIndexingAuthor,
    ArticleIndexingStatus, ArticleMeshHeading, ArticleMeshTerm, ArticleSemanticScholar,
    ArticleSemanticScholarPdf, ArticleSource, INVALID_ARTICLE_ID_MSG, fulltext,
};

const ARTICLE_INDEXING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) fn is_doi(id: &str) -> bool {
    let id = id.trim();
    if !id.starts_with("10.") {
        return false;
    }
    id.contains('/')
}

pub(super) fn parse_pmid(id: &str) -> Option<u32> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    id.parse::<u32>().ok()
}

pub(super) fn parse_pmcid(id: &str) -> Option<String> {
    let mut id = id.trim();
    if id.len() > 6
        && let Some(prefix) = id.get(..6)
        && prefix.eq_ignore_ascii_case("PMCID:")
        && let Some(rest) = id.get(6..)
    {
        id = rest.trim();
    }
    if id.len() < 4 {
        return None;
    }
    let prefix = id.get(..3)?;
    if !prefix.eq_ignore_ascii_case("PMC") {
        return None;
    }
    let rest = id.get(3..)?.trim();
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("PMC{rest}"))
}

pub(super) fn parse_arxiv_id(id: &str) -> Option<String> {
    let id = id.trim();
    if id.len() <= 6 {
        return None;
    }
    let prefix = id.get(..6)?;
    if !prefix.eq_ignore_ascii_case("arxiv:") {
        return None;
    }
    let rest = id.get(6..)?.trim();
    if rest.is_empty() {
        return None;
    }
    Some(format!("ARXIV:{rest}"))
}

#[derive(Debug, Clone)]
pub(super) enum ArticleIdType {
    Pmc(String),
    Doi(String),
    Pmid(u32),
    Invalid,
}

pub(super) fn parse_article_id(id: &str) -> ArticleIdType {
    let id = id.trim();
    if let Some(pmcid) = parse_pmcid(id) {
        return ArticleIdType::Pmc(pmcid);
    }
    if is_doi(id) {
        return ArticleIdType::Doi(id.to_string());
    }
    if let Some(pmid) = parse_pmid(id) {
        return ArticleIdType::Pmid(pmid);
    }
    ArticleIdType::Invalid
}

#[derive(Debug, Clone, Default)]
pub(super) struct ArticleSections {
    pub(super) include_annotations: bool,
    pub(super) include_fulltext: bool,
    pub(super) include_tldr: bool,
    pub(super) include_indexing: bool,
    pub(super) include_all: bool,
    pub(super) include_assets: bool,
    pub(super) asset_name: Option<String>,
}

pub(super) fn parse_sections(sections: &[String]) -> Result<ArticleSections, BioMcpError> {
    let mut out = ArticleSections::default();
    let mut index = 0;

    while index < sections.len() {
        let raw = &sections[index];
        let section = raw.trim().to_ascii_lowercase();
        index += 1;
        if section.is_empty() {
            continue;
        }
        if section == "--json" || section == "-j" || section == "--pdf" {
            continue;
        }

        match section.as_str() {
            ARTICLE_SECTION_ANNOTATIONS => out.include_annotations = true,
            ARTICLE_SECTION_FULLTEXT => out.include_fulltext = true,
            ARTICLE_SECTION_TLDR => out.include_tldr = true,
            ARTICLE_SECTION_INDEXING => out.include_indexing = true,
            ARTICLE_SECTION_ASSETS => out.include_assets = true,
            ARTICLE_SECTION_ASSET => {
                let Some(name) = sections
                    .get(index)
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                else {
                    return Err(BioMcpError::InvalidArgument(
                        "asset requires a package filename (example: biomcp get article 22663011 asset traces-s1.csv)"
                            .into(),
                    ));
                };
                out.asset_name = Some(name.to_string());
                index += 1;
            }
            ARTICLE_SECTION_ALL => out.include_all = true,
            _ => {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Unknown section \"{section}\" for article. Available: {}",
                    ARTICLE_SECTION_NAMES.join(", ")
                )));
            }
        }
    }

    if out.include_all {
        out.include_annotations = true;
        out.include_fulltext = true;
        out.include_tldr = true;
        out.include_indexing = true;
    }

    Ok(out)
}

pub(super) fn is_section_only_request(sections: &[String], include_all: bool) -> bool {
    if include_all {
        return false;
    }
    sections.iter().any(|s| {
        let value = s.trim().to_ascii_lowercase();
        !value.is_empty() && value != "--json" && value != "-j" && value != "--pdf"
    })
}

pub(super) fn article_not_found(id: &str, suggestion_id: &str) -> BioMcpError {
    BioMcpError::NotFound {
        entity: "article".into(),
        id: id.to_string(),
        suggestion: format!("Try searching: biomcp search article -q \"{suggestion_id}\""),
    }
}

pub(super) fn first_europepmc_hit(search: EuropePmcSearchResponse) -> Option<EuropePmcResult> {
    search
        .result_list
        .and_then(|list| list.result.into_iter().next())
}

fn semantic_scholar_enrichment_from_paper(
    paper: &SemanticScholarPaper,
) -> Option<ArticleSemanticScholar> {
    let open_access_pdf = paper.open_access_pdf.as_ref().and_then(|pdf| {
        let url = pdf
            .url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        Some(ArticleSemanticScholarPdf {
            url: url.to_string(),
            status: pdf
                .status
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            license: pdf
                .license
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    });
    let tldr = paper
        .tldr
        .as_ref()
        .and_then(|value| value.text.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if paper.paper_id.is_none()
        && tldr.is_none()
        && paper.citation_count.is_none()
        && paper.influential_citation_count.is_none()
        && paper.reference_count.is_none()
        && paper.is_open_access.is_none()
        && open_access_pdf.is_none()
    {
        return None;
    }

    Some(ArticleSemanticScholar {
        paper_id: paper.paper_id.clone(),
        tldr,
        citation_count: paper.citation_count,
        influential_citation_count: paper.influential_citation_count,
        reference_count: paper.reference_count,
        is_open_access: paper.is_open_access,
        open_access_pdf,
    })
}

pub(super) fn is_pubtator_lag_error(err: &BioMcpError) -> bool {
    matches!(
        err,
        BioMcpError::Api { api, message }
            if api == "pubtator3" && (message.contains("HTTP 400") || message.contains("HTTP 404"))
    )
}

pub(super) async fn resolve_article_from_pmid(
    pmid: u32,
    not_found_id: &str,
    suggestion_id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
    europe_hint: Option<&EuropePmcResult>,
) -> Result<Article, BioMcpError> {
    match pubtator.export_biocjson(pmid).await {
        Ok(resp) => {
            let doc = resp
                .documents
                .into_iter()
                .next()
                .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?;

            let mut article = transform::article::from_pubtator_document(&doc);
            if let Some(hit) = europe_hint {
                transform::article::merge_europepmc_metadata(&mut article, hit);
            } else if let Ok(search) = europe.search_by_pmid(&pmid.to_string()).await
                && let Some(hit) = first_europepmc_hit(search)
            {
                transform::article::merge_europepmc_metadata(&mut article, &hit);
            }
            article.annotations = transform::article::extract_annotations(&doc);
            Ok(article)
        }
        Err(err) => {
            if !is_pubtator_lag_error(&err) {
                return Err(err);
            }

            let hit = match europe_hint.cloned() {
                Some(hit) => hit,
                None => {
                    let search = europe.search_by_pmid(&pmid.to_string()).await?;
                    first_europepmc_hit(search)
                        .ok_or_else(|| article_not_found(not_found_id, suggestion_id))?
                }
            };
            Ok(article_from_europepmc_fallback(&hit))
        }
    }
}

fn article_from_europepmc_fallback(hit: &EuropePmcResult) -> Article {
    let mut article = transform::article::from_europepmc_result(hit);
    article.pubtator_fallback = true;
    article
}

pub(super) async fn get_article_base_with_clients(
    id: &str,
    pubtator: &PubTatorClient,
    europe: &EuropePmcClient,
) -> Result<Article, BioMcpError> {
    let id = id.trim();
    if id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "ID is required. Example: biomcp get article 22663011".into(),
        ));
    }
    if id.len() > 512 {
        return Err(BioMcpError::InvalidArgument("ID is too long.".into()));
    }

    match parse_article_id(id) {
        ArticleIdType::Pmid(pmid) => {
            resolve_article_from_pmid(pmid, id, id, pubtator, europe, None).await
        }
        ArticleIdType::Doi(doi) => {
            let search = europe.search_by_doi(&doi).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&doi, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&doi, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &doi, id, pubtator, europe, Some(&hit)).await
            } else {
                Ok(transform::article::from_europepmc_result(&hit))
            }
        }
        ArticleIdType::Pmc(pmcid) => {
            let search = europe.search_by_pmcid(&pmcid).await?;
            if search.hit_count.unwrap_or(0) == 0 {
                return Err(article_not_found(&pmcid, id));
            }
            let hit = first_europepmc_hit(search).ok_or_else(|| article_not_found(&pmcid, id))?;

            if let Some(pmid) = hit.pmid.as_deref().and_then(parse_pmid) {
                resolve_article_from_pmid(pmid, &pmcid, id, pubtator, europe, Some(&hit)).await
            } else {
                Ok(transform::article::from_europepmc_result(&hit))
            }
        }
        ArticleIdType::Invalid => Err(BioMcpError::InvalidArgument(INVALID_ARTICLE_ID_MSG.into())),
    }
}

pub(super) async fn get_article_base(id: &str) -> Result<Article, BioMcpError> {
    let pubtator = PubTatorClient::new()?;
    let europe = EuropePmcClient::new()?;
    get_article_base_with_clients(id, &pubtator, &europe).await
}

fn unavailable_indexing() -> ArticleIndexing {
    ArticleIndexing {
        status: ArticleIndexingStatus::Unavailable,
        source: ArticleSource::PubMed,
        authors: Vec::new(),
        mesh_headings: Vec::new(),
    }
}

fn article_indexing_from_citation(citation: PubMedCitation) -> ArticleIndexing {
    ArticleIndexing {
        status: ArticleIndexingStatus::Available,
        source: ArticleSource::PubMed,
        authors: citation
            .authors
            .into_iter()
            .map(|author| ArticleIndexingAuthor {
                name: author.name,
                orcid: author.orcid,
                affiliations: author
                    .affiliations
                    .into_iter()
                    .map(|affiliation| ArticleAffiliation {
                        text: affiliation.text,
                        identifiers: affiliation
                            .identifiers
                            .into_iter()
                            .map(|identifier| ArticleAffiliationIdentifier {
                                source: identifier.source,
                                value: identifier.value,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        mesh_headings: citation
            .mesh_headings
            .into_iter()
            .map(|heading| ArticleMeshHeading {
                descriptor: ArticleMeshTerm {
                    text: heading.descriptor.text,
                    ui: heading.descriptor.ui,
                    major_topic: heading.descriptor.major_topic,
                },
                qualifiers: heading
                    .qualifiers
                    .into_iter()
                    .map(|qualifier| ArticleMeshTerm {
                        text: qualifier.text,
                        ui: qualifier.ui,
                        major_topic: qualifier.major_topic,
                    })
                    .collect(),
            })
            .collect(),
    }
}

async fn citation_with_timeout<F>(
    timeout: std::time::Duration,
    future: F,
) -> Result<PubMedCitation, BioMcpError>
where
    F: std::future::Future<Output = Result<PubMedCitation, BioMcpError>>,
{
    tokio::time::timeout(timeout, future)
        .await
        .map_err(|_| BioMcpError::Api {
            api: "pubmed-eutils".into(),
            message: "PubMed article indexing timed out".into(),
        })?
}

async fn enrich_article_with_indexing(article: &mut Article) {
    let Some(pmid) = article.pmid.as_deref() else {
        article.indexing = Some(unavailable_indexing());
        return;
    };
    let client = match PubMedClient::new() {
        Ok(client) => client,
        Err(_error) => {
            warn!(pmid, "PubMed article indexing client failed");
            article.indexing = Some(unavailable_indexing());
            return;
        }
    };
    match citation_with_timeout(ARTICLE_INDEXING_TIMEOUT, client.citation(pmid)).await {
        Ok(citation) => article.indexing = Some(article_indexing_from_citation(citation)),
        Err(_error) => {
            warn!(pmid, "PubMed article indexing unavailable");
            article.indexing = Some(unavailable_indexing());
        }
    }
}

pub(super) async fn enrich_article_with_semantic_scholar(
    article: &mut Article,
) -> Result<(), BioMcpError> {
    let client = SemanticScholarClient::new()?;

    let lookup_id = article
        .pmid
        .as_deref()
        .map(|pmid| format!("PMID:{pmid}"))
        .or_else(|| article.doi.as_deref().map(|doi| format!("DOI:{doi}")));
    let Some(lookup_id) = lookup_id else {
        return Ok(());
    };

    match client.paper_detail(&lookup_id).await {
        Ok(paper) => article.semantic_scholar = semantic_scholar_enrichment_from_paper(&paper),
        Err(err) => warn!(?err, lookup_id, "Semantic Scholar enrichment failed"),
    }

    Ok(())
}

pub async fn get(
    id: &str,
    sections: &[String],
    options: ArticleGetOptions,
) -> Result<Article, BioMcpError> {
    let id = id.trim();
    let section_flags = parse_sections(sections)?;
    if options.allow_pdf && !section_flags.include_fulltext {
        return Err(BioMcpError::InvalidArgument(
            "--pdf requires the fulltext section (example: biomcp get article 22663011 fulltext --pdf)"
                .into(),
        ));
    }
    let section_only = is_section_only_request(sections, section_flags.include_all);
    let mut article = get_article_base(id).await?;

    if section_flags.include_indexing {
        enrich_article_with_indexing(&mut article).await;
    }

    enrich_article_with_semantic_scholar(&mut article).await?;

    if section_only && !section_flags.include_annotations {
        article.annotations = None;
    }

    if section_flags.include_fulltext {
        fulltext::resolve_fulltext(&mut article, id, options).await?;
        if options.include_asset_summary {
            super::assets::attach_not_included(&mut article, id).await;
        }
    }

    if section_only && !section_flags.include_tldr {
        article.semantic_scholar = None;
    }

    Ok(article)
}

#[cfg(test)]
mod tests;
