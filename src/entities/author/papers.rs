use super::*;
use crate::entities::article::ArticleRelatedPaper;
use crate::error::BioMcpError;
use crate::next_command::NextCommand;
use crate::sources::semantic_scholar::SemanticScholarAuthorPaper;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPapersPagination {
    pub offset: u64,
    pub limit: usize,
    pub next: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPapersResult {
    pub author: AuthorIdentity,
    pub papers: Vec<ArticleRelatedPaper>,
    pub pagination: AuthorPapersPagination,
    pub _meta: AuthorMeta,
}

pub async fn papers(
    raw_id: &str,
    offset: usize,
    limit: usize,
) -> Result<AuthorPapersResult, BioMcpError> {
    let requested: ProviderAuthorId = raw_id.parse()?;
    let page = crate::sources::semantic_scholar::SemanticScholarClient::new()?
        .author_papers(&requested.value, offset, limit)
        .await
        .map_err(sanitized_provider_error)?;

    let mut next_commands = Vec::new();
    let mut evidence_urls = Vec::new();
    let papers = page
        .data
        .into_iter()
        .filter_map(|paper| map_paper(paper, &mut next_commands, &mut evidence_urls))
        .collect();
    if let Some(next) = page.next {
        next_commands.push(format!(
            "biomcp author papers {requested} --limit {limit} --offset {next}"
        ));
    }

    Ok(AuthorPapersResult {
        author: AuthorIdentity::ExactProvider { id: requested },
        papers,
        pagination: AuthorPapersPagination {
            offset: page.offset.unwrap_or(offset as u64),
            limit,
            next: page.next,
        },
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

fn external_id(paper: &SemanticScholarAuthorPaper, key: &str) -> Option<String> {
    paper
        .external_ids
        .as_ref()?
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn map_paper(
    paper: SemanticScholarAuthorPaper,
    next_commands: &mut Vec<String>,
    evidence_urls: &mut Vec<AuthorEvidenceUrl>,
) -> Option<ArticleRelatedPaper> {
    let pmid = external_id(&paper, "PubMed");
    let doi = external_id(&paper, "DOI");
    let arxiv_id = external_id(&paper, "ArXiv");
    let paper_id = nonblank(paper.paper_id)?;
    let title = nonblank(paper.title)?;
    evidence_urls.push(AuthorEvidenceUrl {
        source: "semantic_scholar",
        url: format!("https://www.semanticscholar.org/paper/{paper_id}"),
    });
    if let Some(id) = pmid
        .as_deref()
        .or(doi.as_deref())
        .or(arxiv_id.as_deref())
        .or_else(|| {
            (paper_id.len() == 40 && paper_id.bytes().all(|b| b.is_ascii_hexdigit()))
                .then_some(paper_id.as_str())
        })
    {
        let id = if pmid.is_none() && doi.is_none() && arxiv_id.as_deref() == Some(id) {
            format!("arXiv:{id}")
        } else {
            id.to_string()
        };
        next_commands.push(
            NextCommand::biomcp()
                .args(["get", "article"])
                .arg(id)
                .render_shell(),
        );
    }
    Some(ArticleRelatedPaper {
        paper_id: Some(paper_id),
        pmid,
        doi,
        arxiv_id,
        title,
        journal: nonblank(paper.venue),
        year: paper.year,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_article_ids_are_shell_quoted_in_next_commands() {
        let paper = SemanticScholarAuthorPaper {
            paper_id: Some("0123456789abcdef0123456789abcdef01234567".into()),
            external_ids: Some(serde_json::Map::from_iter([(
                "DOI".into(),
                serde_json::json!("10/example;echo unsafe"),
            )])),
            title: Some("Safe title".into()),
            ..Default::default()
        };
        let mut commands = Vec::new();
        let mut evidence = Vec::new();

        map_paper(paper, &mut commands, &mut evidence).expect("valid paper");

        assert_eq!(commands, ["biomcp get article \"10/example;echo unsafe\""]);
    }
}
