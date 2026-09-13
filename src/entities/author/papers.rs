use super::*;
use crate::error::BioMcpError;
use crate::next_command::NextCommand;
use crate::sources::semantic_scholar::{
    SemanticScholarAuthorPaper, SemanticScholarAuthorPaperAuthor, SemanticScholarClient,
    SemanticScholarOpenAccessPdf,
};
use serde::Serialize;
use std::time::Duration;

const AUTHOR_PAPERS_COMMAND_DEADLINE: Duration = Duration::from_secs(35);
/// Ticket 1142: the ORCID works slice is local, and `--offset` beyond the
/// maximum retained group count is a static invalid argument.
const ORCID_MAX_OFFSET: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPapersPagination {
    pub offset: u64,
    pub limit: usize,
    pub next: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPapersResult {
    pub author: AuthorIdentity,
    pub papers: Vec<AuthorPaper>,
    pub pagination: AuthorPapersPagination,
    pub _meta: AuthorMeta,
}

/// Author-owned compact paper: the shared article shape first, then the
/// ORCID-only fields, so Semantic Scholar JSON stays byte-for-byte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPaper {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub identifiers: Vec<AuthorPaperIdentifier>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPaperIdentifier {
    #[serde(rename = "type")]
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPaperOpenAccessPdf {
    pub url: Option<String>,
    pub status: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPaperFullAuthor {
    pub identity: Option<AuthorIdentity>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPaperFull {
    pub paper_id: String,
    pub corpus_id: Option<u64>,
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub journal: Option<String>,
    pub year: Option<u32>,
    pub publication_date: Option<String>,
    pub citation_count: Option<u64>,
    pub reference_count: Option<u64>,
    pub influential_citation_count: Option<u64>,
    pub is_open_access: Option<bool>,
    pub open_access_pdf: Option<AuthorPaperOpenAccessPdf>,
    pub fields_of_study: Option<Vec<String>>,
    pub publication_types: Option<Vec<String>>,
    pub authors: Option<Vec<AuthorPaperFullAuthor>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorPapersFullResult {
    pub author: AuthorIdentity,
    pub papers: Vec<AuthorPaperFull>,
    pub pagination: AuthorPapersPagination,
    pub _meta: AuthorMeta,
}

pub async fn papers(
    raw_id: &str,
    offset: usize,
    limit: usize,
) -> Result<AuthorPapersResult, BioMcpError> {
    let requested: ProviderAuthorId = raw_id.parse()?;
    if requested.provider == AuthorIdProvider::Orcid {
        // Static bound: a caller offset beyond the maximum possible retained
        // group count is rejected before any request is planned or sent.
        if offset > ORCID_MAX_OFFSET {
            return Err(BioMcpError::InvalidArgument(
                "--offset must be at most 10000 for orcid: author claimed works".to_string(),
            ));
        }
        return orcid_papers(&requested, offset, limit).await;
    }
    let mut page = fetch_author_papers_page(&requested, offset, limit, false).await?;
    let mut next_commands = Vec::new();
    let mut evidence_urls = Vec::new();
    let data = std::mem::take(&mut page.data);
    let papers = data
        .into_iter()
        .filter_map(|paper| map_paper(paper, &mut next_commands, &mut evidence_urls))
        .collect();
    finish_compact(
        requested,
        offset,
        limit,
        page,
        papers,
        next_commands,
        evidence_urls,
    )
}

pub async fn papers_full(
    raw_id: &str,
    offset: usize,
    limit: usize,
) -> Result<AuthorPapersFullResult, BioMcpError> {
    let requested: ProviderAuthorId = raw_id.parse()?;
    if requested.provider == AuthorIdProvider::Orcid {
        return Err(BioMcpError::InvalidArgument(
            "--full is available only for semanticscholar: author IDs; omit --full for ORCID claimed works".to_string(),
        ));
    }
    let page = fetch_author_papers_page(&requested, offset, limit, true).await?;
    let mut next_commands = Vec::new();
    let mut evidence_urls = Vec::new();
    let papers: Vec<AuthorPaperFull> = page
        .data
        .iter()
        .filter(|paper| admitted(paper))
        .map(|paper| map_paper_full(paper, &mut next_commands, &mut evidence_urls))
        .collect::<Result<Vec<_>, _>>()?;
    let pagination = AuthorPapersPagination {
        offset: page.offset.unwrap_or(offset as u64),
        limit,
        next: page.next,
        total: None,
        truncated: None,
    };
    if let Some(next) = page.next {
        next_commands.push(format!(
            "biomcp author papers {requested} --full --limit {limit} --offset {next}"
        ));
    }
    Ok(AuthorPapersFullResult {
        author: AuthorIdentity::ExactProvider { id: requested },
        papers,
        pagination,
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

/// Ticket 1142: one logical `/works` request, group selection and stable
/// deduplication upstream, then a bounded local slice.
async fn orcid_papers(
    requested: &ProviderAuthorId,
    offset: usize,
    limit: usize,
) -> Result<AuthorPapersResult, BioMcpError> {
    let deadline = tokio::time::Instant::now() + command_deadline_budget();
    let works = crate::sources::orcid::OrcidClient::new()?
        .works(&requested.value, deadline)
        .await?;
    let selected = works.selected_works()?;
    let retained = retain_deduplicated(&selected);
    let (start, end, next, total) = orcid_page_bounds(retained.len(), offset, limit);
    let mut next_commands = Vec::new();
    let mut evidence_urls = Vec::new();
    let papers: Vec<AuthorPaper> = retained[start..end]
        .iter()
        .map(|work| orcid_paper(requested, work, &mut next_commands, &mut evidence_urls))
        .collect();
    if let Some(next) = next {
        next_commands.push(format!(
            "biomcp author papers {requested} --limit {limit} --offset {next}"
        ));
    }
    Ok(AuthorPapersResult {
        author: AuthorIdentity::ExactProvider {
            id: requested.clone(),
        },
        pagination: AuthorPapersPagination {
            offset: offset as u64,
            limit,
            next,
            total: Some(total),
            truncated: Some(false),
        },
        papers,
        _meta: AuthorMeta {
            source_status: vec![AuthorSourceStatus {
                source: "orcid",
                status: crate::entities::author::ProviderStatus::Available,
            }],
            evidence_urls,
            next_commands,
        },
    })
}

/// Ticket 1142: stable cross-group deduplication on recognized canonical
/// identifiers only. Preserved unrecognized types never drop a group,
/// including groups whose only identifiers are unrecognized or absent.
fn retain_deduplicated(
    selected: &[crate::sources::orcid::OrcidSelectedWork],
) -> Vec<&crate::sources::orcid::OrcidSelectedWork> {
    let mut seen: Vec<(String, String)> = Vec::new();
    let mut retained: Vec<&crate::sources::orcid::OrcidSelectedWork> = Vec::new();
    for work in selected {
        let ids = canonical_identifiers(work);
        // Deduplicate on recognized canonical kinds only: preserved
        // unrecognized types never drop a group, including groups whose only
        // identifiers are unrecognized or absent.
        let recognized: Vec<(String, String)> = ids
            .iter()
            .filter(|(kind, _)| is_recognized_kind(kind))
            .cloned()
            .collect();
        if recognized
            .iter()
            .any(|(kind, value)| seen.contains(&(kind.clone(), value.clone())))
        {
            continue;
        }
        seen.extend(recognized);
        retained.push(work);
    }
    retained
}

/// Ticket 1142 local pagination: slice the retained works by the caller's
/// offset and limit, with `next` exactly when more retained works remain.
#[allow(clippy::type_complexity)]
fn orcid_page_bounds(
    retained: usize,
    offset: usize,
    limit: usize,
) -> (usize, usize, Option<u64>, u64) {
    let total = retained as u64;
    let start = offset.min(retained);
    let end = start.saturating_add(limit).min(retained);
    let returned = (start + (end - start)) as u64;
    let next = (returned < total).then_some(returned);
    (start, end, next, total)
}

/// Canonicalize the recognized identifier kinds and keep the full normalized
/// list (recognized and preserved) in first-occurrence order.
fn canonical_identifiers(work: &crate::sources::orcid::OrcidSelectedWork) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (kind, value) in &work.external_ids {
        let Some((kind, value)) = canonical_identifier(kind, value) else {
            if is_preserved_type(kind) && !out.iter().any(|(k, v)| k == kind && v == value) {
                out.push((kind.clone(), value.clone()));
            }
            continue;
        };
        if !out.iter().any(|(k, v)| k == &kind && v == &value) {
            out.push((kind, value));
        }
    }
    out
}

fn canonical_identifier(kind: &str, value: &str) -> Option<(String, String)> {
    match kind {
        "doi" => {
            let mut value = value.trim();
            for prefix in ["doi:", "https://doi.org/"] {
                if let Some(rest) = value.strip_prefix(prefix) {
                    value = rest.trim_start();
                }
            }
            let value = value.trim().to_ascii_lowercase();
            (value.len() >= 3 && value.len() <= 255 && value.contains('/'))
                .then(|| ("doi".into(), value))
        }
        "pmid" => {
            let digits = value.trim();
            (!digits.is_empty()
                && digits.len() <= 12
                && digits.bytes().all(|b| b.is_ascii_digit())
                && digits.parse::<u64>().is_ok_and(|n| n > 0))
            .then(|| ("pmid".into(), digits.parse::<u64>().unwrap().to_string()))
        }
        "pmc" | "pmcid" => {
            let upper = value.trim().to_ascii_uppercase();
            let digits = upper.strip_prefix("PMC").filter(|digits| {
                !digits.is_empty()
                    && digits.len() <= 12
                    && digits.bytes().all(|b| b.is_ascii_digit())
                    && digits.parse::<u64>().is_ok_and(|n| n > 0)
            })?;
            Some((
                "pmcid".into(),
                format!("PMC{}", digits.parse::<u64>().unwrap()),
            ))
        }
        "arxiv" => {
            let trimmed = value.trim();
            // Remove one case-insensitive `arxiv:` prefix so `ARXIV:2103.0001`
            // normalizes instead of being dropped.
            let value = trimmed
                .strip_prefix_insensitive("arxiv:")
                .map(str::trim)
                .unwrap_or(trimmed);
            (!value.is_empty()
                && value.len() <= 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'/' | b'-')))
            .then(|| ("arxiv".into(), value.to_string()))
        }
        _ => None,
    }
}

/// The recognized canonical identifier kinds that participate in flattening
/// and cross-group deduplication.
fn is_recognized_kind(kind: &str) -> bool {
    matches!(kind, "doi" | "pmid" | "pmcid" | "arxiv")
}

/// Strip one prefix matching `prefix` ASCII-case-insensitively.
trait StripPrefixInsensitive {
    fn strip_prefix_insensitive(&self, prefix: &str) -> Option<&str>;
}

impl StripPrefixInsensitive for str {
    fn strip_prefix_insensitive(&self, prefix: &str) -> Option<&str> {
        if self.len() >= prefix.len()
            && self.is_char_boundary(prefix.len())
            && self[..prefix.len()].eq_ignore_ascii_case(prefix)
        {
            Some(&self[prefix.len()..])
        } else {
            None
        }
    }
}

/// Unrecognized-but-well-formed types are preserved verbatim.
fn is_preserved_type(kind: &str) -> bool {
    !matches!(kind, "doi" | "pmid" | "pmc" | "pmcid" | "arxiv")
}

fn orcid_paper(
    requested: &ProviderAuthorId,
    work: &crate::sources::orcid::OrcidSelectedWork,
    next_commands: &mut Vec<String>,
    evidence_urls: &mut Vec<AuthorEvidenceUrl>,
) -> AuthorPaper {
    let ids = canonical_identifiers(work);
    // Flatten the lexically first normalized value of each recognized kind,
    // not the first occurrence in provider order.
    let first = |kind: &str| {
        ids.iter()
            .filter(|(k, _)| k == kind)
            .map(|(_, v)| v.clone())
            .min()
    };
    let pmid = first("pmid");
    let pmcid = first("pmcid");
    let doi = first("doi");
    let arxiv_id = first("arxiv");
    let evidence_url = format!(
        "https://orcid.org/{}/work/{}",
        requested.value, work.put_code
    );
    evidence_urls.push(AuthorEvidenceUrl {
        source: "orcid",
        url: evidence_url,
    });
    if let Some(id) = pmid
        .as_deref()
        .or(pmcid.as_deref())
        .or(doi.as_deref())
        .or(arxiv_id.as_deref())
    {
        next_commands.push(
            NextCommand::biomcp()
                .args(["get", "article"])
                .arg(id)
                .render_shell(),
        );
    }
    AuthorPaper {
        paper_id: None,
        pmid: pmid.clone(),
        doi: doi.clone(),
        arxiv_id: arxiv_id.clone(),
        title: work.title.clone(),
        journal: work.journal.clone(),
        year: work.year,
        work_id: Some(format!("orcid:{}/work:{}", requested.value, work.put_code)),
        pmcid: pmcid.clone(),
        identifiers: ids
            .into_iter()
            .map(|(kind, value)| AuthorPaperIdentifier { kind, value })
            .collect(),
    }
}

#[allow(clippy::too_many_arguments)]
fn finish_compact(
    requested: ProviderAuthorId,
    offset: usize,
    limit: usize,
    page: crate::sources::semantic_scholar::SemanticScholarAuthorPapersResponse,
    papers: Vec<AuthorPaper>,
    mut next_commands: Vec<String>,
    evidence_urls: Vec<AuthorEvidenceUrl>,
) -> Result<AuthorPapersResult, BioMcpError> {
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
            total: None,
            truncated: None,
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

fn command_deadline_budget() -> Duration {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("BIOMCP_TEST_AUTHOR_PAPERS_DEADLINE_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    AUTHOR_PAPERS_COMMAND_DEADLINE
}

async fn fetch_author_papers_page(
    requested: &ProviderAuthorId,
    offset: usize,
    limit: usize,
    full: bool,
) -> Result<crate::sources::semantic_scholar::SemanticScholarAuthorPapersResponse, BioMcpError> {
    let deadline = tokio::time::Instant::now() + command_deadline_budget();
    let request = async {
        SemanticScholarClient::new()?
            .author_papers(&requested.value, offset, limit, full)
            .await
            .map_err(sanitized_provider_error)
    };
    tokio::time::timeout_at(deadline, request)
        .await
        // The carrier is discarded; the helper supplies the pinned sanitized
        // unavailable message for the deadline arm, matching the request arm.
        .map_err(|_| {
            sanitized_provider_error(BioMcpError::Api {
                api: "".to_string(),
                message: "".into(),
            })
        })?
}

fn admitted(paper: &SemanticScholarAuthorPaper) -> bool {
    nonblank(paper.paper_id.clone()).is_some() && nonblank(paper.title.clone()).is_some()
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

/// WHATWG special-URL path-segment encoding for one evidence-URL segment.
/// C0 controls, space, and `"#<>?`{}%\\` are uppercase `%HH`; every other
/// ASCII byte stays literal; non-ASCII scalars become their UTF-8 bytes.
/// A segment that is exactly `.` or `..` encodes each dot so URL dot-segment
/// normalization can never consume provider data.
pub(crate) fn encode_paper_url_segment(value: &str) -> String {
    let mut out = String::new();
    let dotted = value == "." || value == "..";
    for ch in value.chars() {
        let mut buffer = [0u8; 4];
        for byte in ch.encode_utf8(&mut buffer).as_bytes() {
            let byte = *byte;
            let needs_encoding = !byte.is_ascii()
                || byte < 0x21
                || byte == 0x7f
                || matches!(
                    byte,
                    b'"' | b'#' | b'<' | b'>' | b'?' | b'`' | b'{' | b'}' | b'/' | b'%' | b'\\'
                )
                || (dotted && byte == b'.');
            if needs_encoding {
                out.push_str(&format!("%{byte:02X}"));
            } else {
                out.push(byte as char);
            }
        }
    }
    out
}

pub(crate) fn paper_evidence_url(paper_id: &str) -> String {
    format!(
        "https://www.semanticscholar.org/paper/{}",
        encode_paper_url_segment(paper_id)
    )
}

fn article_follow_up_command(
    pmid: &Option<String>,
    doi: &Option<String>,
    arxiv_id: &Option<String>,
    paper_id: &str,
) -> Option<String> {
    let id = pmid
        .as_deref()
        .or(doi.as_deref())
        .or(arxiv_id.as_deref())
        .or_else(|| {
            (paper_id.len() == 40 && paper_id.bytes().all(|b| b.is_ascii_hexdigit()))
                .then_some(paper_id)
        })?;
    let id = if pmid.is_none() && doi.is_none() && arxiv_id.as_deref() == Some(id) {
        format!("arXiv:{id}")
    } else {
        id.to_string()
    };
    Some(
        NextCommand::biomcp()
            .args(["get", "article"])
            .arg(id)
            .render_shell(),
    )
}

fn map_paper(
    paper: SemanticScholarAuthorPaper,
    next_commands: &mut Vec<String>,
    evidence_urls: &mut Vec<AuthorEvidenceUrl>,
) -> Option<AuthorPaper> {
    let pmid = external_id(&paper, "PubMed");
    let doi = external_id(&paper, "DOI");
    let arxiv_id = external_id(&paper, "ArXiv");
    let paper_id = nonblank(paper.paper_id)?;
    let title = nonblank(paper.title)?;
    evidence_urls.push(AuthorEvidenceUrl {
        source: "semantic_scholar",
        url: format!("https://www.semanticscholar.org/paper/{paper_id}"),
    });
    if let Some(command) = article_follow_up_command(&pmid, &doi, &arxiv_id, &paper_id) {
        next_commands.push(command);
    }
    Some(AuthorPaper {
        paper_id: Some(paper_id),
        pmid,
        doi,
        arxiv_id,
        title,
        journal: nonblank(paper.venue),
        year: paper.year,
        work_id: None,
        pmcid: None,
        identifiers: Vec::new(),
    })
}

fn rich_external_id(
    paper: &SemanticScholarAuthorPaper,
    key: &str,
) -> Result<Option<String>, BioMcpError> {
    match paper.external_ids.as_ref().and_then(|ids| ids.get(key)) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value.trim().to_string())),
        Some(_) => Err(BioMcpError::Api {
            api: "semantic-scholar".into(),
            message: format!("author paper external ID {key} had an unexpected type"),
        }),
    }
}

fn trim_only(value: Option<String>) -> Option<String> {
    Some(value?.trim().to_string())
}

fn trim_list(value: Option<Vec<String>>) -> Option<Vec<String>> {
    value.map(|list| list.iter().map(|item| item.trim().to_string()).collect())
}

fn map_open_access_pdf(
    value: Option<SemanticScholarOpenAccessPdf>,
) -> Option<AuthorPaperOpenAccessPdf> {
    value.map(|pdf| AuthorPaperOpenAccessPdf {
        url: trim_only(pdf.url),
        status: trim_only(pdf.status),
        license: trim_only(pdf.license),
    })
}

fn map_full_author(
    author: &SemanticScholarAuthorPaperAuthor,
) -> Result<AuthorPaperFullAuthor, BioMcpError> {
    let identity =
        valid_wire_id(author.author_id.clone()).map(|value| AuthorIdentity::ExactProvider {
            id: ProviderAuthorId {
                provider: AuthorIdProvider::SemanticScholar,
                value,
            },
        });
    Ok(AuthorPaperFullAuthor {
        identity,
        display_name: trim_only(author.name.clone()),
    })
}

fn map_paper_full(
    paper: &SemanticScholarAuthorPaper,
    next_commands: &mut Vec<String>,
    evidence_urls: &mut Vec<AuthorEvidenceUrl>,
) -> Result<AuthorPaperFull, BioMcpError> {
    let paper_id = nonblank(paper.paper_id.clone()).expect("admitted row");
    let pmid = rich_external_id(paper, "PubMed")?;
    let pmcid = rich_external_id(paper, "PubMedCentral")?;
    let doi = rich_external_id(paper, "DOI")?;
    let arxiv_id = rich_external_id(paper, "ArXiv")?;
    evidence_urls.push(AuthorEvidenceUrl {
        source: "semantic_scholar",
        url: paper_evidence_url(&paper_id),
    });
    if let Some(command) = article_follow_up_command(&pmid, &doi, &arxiv_id, &paper_id) {
        next_commands.push(command);
    }
    Ok(AuthorPaperFull {
        paper_id,
        corpus_id: paper.corpus_id,
        pmid,
        pmcid,
        doi,
        arxiv_id,
        title: nonblank(paper.title.clone()).expect("admitted row"),
        abstract_text: trim_only(paper.abstract_text.clone()),
        journal: trim_only(paper.venue.clone()),
        year: paper.year,
        publication_date: trim_only(paper.publication_date.clone()),
        citation_count: paper.citation_count,
        reference_count: paper.reference_count,
        influential_citation_count: paper.influential_citation_count,
        is_open_access: paper.is_open_access,
        open_access_pdf: map_open_access_pdf(paper.open_access_pdf.clone()),
        fields_of_study: trim_list(paper.fields_of_study.clone()),
        publication_types: trim_list(paper.publication_types.clone()),
        authors: paper
            .authors
            .as_ref()
            .map(|byline| {
                byline
                    .iter()
                    .map(map_full_author)
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?,
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

#[cfg(test)]
mod full_tests {
    use super::*;
    use serde_json::json;

    fn rich_row() -> serde_json::Value {
        json!({
            "paperId": "0123456789abcdef0123456789abcdef01234567",
            "corpusId": 277710284,
            "externalIds": {"PubMed": "40215974", "PubMedCentral": null, "DOI": "10.1016/j.fixture.2024.01.001", "ArXiv": null, "ORCID": "0000-0002-7433-2740"},
            "title": "  A rich author paper fixture  ",
            "abstract": "Source abstract.",
            "venue": "Fixture Medicine",
            "year": 2024,
            "publicationDate": "2024-01-31",
            "citationCount": 17,
            "referenceCount": 23,
            "influentialCitationCount": 2,
            "isOpenAccess": false,
            "openAccessPdf": {"url": "https://example.invalid/paper.pdf", "status": "HYBRID", "license": null},
            "fieldsOfStudy": ["Medicine", "Medicine"],
            "publicationTypes": ["JournalArticle"],
            "authors": [
                {"authorId": "2059910739", "name": "First Author"},
                {"authorId": "not-numeric", "name": null},
                {"authorId": null, "name": "  Third Author  "}
            ]
        })
    }

    #[test]
    fn frozen_rich_object_has_exactly_the_contract_keys_and_values() {
        let wire: SemanticScholarAuthorPaper = serde_json::from_value(rich_row()).unwrap();
        let mut commands = Vec::new();
        let mut evidence = Vec::new();
        let rich = map_paper_full(&wire, &mut commands, &mut evidence).unwrap();
        let rendered = serde_json::to_value(&rich).unwrap();
        assert_eq!(
            rendered,
            json!({
                "paper_id": "0123456789abcdef0123456789abcdef01234567",
                "corpus_id": 277710284,
                "pmid": "40215974",
                "pmcid": null,
                "doi": "10.1016/j.fixture.2024.01.001",
                "arxiv_id": null,
                "title": "A rich author paper fixture",
                "abstract": "Source abstract.",
                "journal": "Fixture Medicine",
                "year": 2024,
                "publication_date": "2024-01-31",
                "citation_count": 17,
                "reference_count": 23,
                "influential_citation_count": 2,
                "is_open_access": false,
                "open_access_pdf": {"url": "https://example.invalid/paper.pdf", "status": "HYBRID", "license": null},
                "fields_of_study": ["Medicine", "Medicine"],
                "publication_types": ["JournalArticle"],
                "authors": [
                    {"identity": {"kind": "exact_provider", "id": "semanticscholar:2059910739"}, "display_name": "First Author"},
                    {"identity": null, "display_name": null},
                    {"identity": null, "display_name": "Third Author"}
                ]
            })
        );
        assert_eq!(
            commands,
            ["biomcp get article 40215974"],
            "PMID wins the article follow-up preference"
        );
    }

    #[test]
    fn nullability_matrix_preserves_empty_false_zero_and_list_distinctions() {
        let wire: SemanticScholarAuthorPaper = serde_json::from_value(json!({
            "paperId": "opaque-id",
            "corpusId": 0,
            "externalIds": {"PubMed": "  ", "PubMedCentral": "", "ArXiv": null},
            "title": "Nullability fixture",
            "abstract": null,
            "venue": "  ",
            "year": null,
            "publicationDate": "",
            "citationCount": 0,
            "referenceCount": null,
            "influentialCitationCount": 0,
            "isOpenAccess": false,
            "openAccessPdf": {"url": null, "status": null, "license": null},
            "fieldsOfStudy": [],
            "publicationTypes": null,
            "authors": []
        }))
        .unwrap();
        let rich = map_paper_full(&wire, &mut Vec::new(), &mut Vec::new()).unwrap();
        let rendered = serde_json::to_value(&rich).unwrap();
        assert_eq!(
            rendered["pmid"],
            json!(""),
            "present-blank trims to empty string"
        );
        assert_eq!(rendered["pmcid"], json!(""));
        assert_eq!(rendered["doi"], json!(null), "absent key stays null");
        assert_eq!(rendered["arxiv_id"], json!(null), "null value stays null");
        assert_eq!(rendered["corpus_id"], json!(0));
        assert_eq!(rendered["citation_count"], json!(0));
        assert_eq!(rendered["reference_count"], json!(null));
        assert_eq!(rendered["influential_citation_count"], json!(0));
        assert_eq!(rendered["is_open_access"], json!(false));
        assert_eq!(rendered["abstract"], json!(null));
        assert_eq!(
            rendered["journal"],
            json!(""),
            "present venue trims to empty string"
        );
        assert_eq!(rendered["publication_date"], json!(""));
        assert_eq!(
            rendered["open_access_pdf"],
            json!({"url": null, "status": null, "license": null}),
            "all-null PDF object stays an object"
        );
        assert_eq!(rendered["fields_of_study"], json!([]));
        assert_eq!(rendered["publication_types"], json!(null));
        assert_eq!(rendered["authors"], json!([]));
    }

    #[test]
    fn wrong_typed_external_ids_fail_the_complete_command() {
        let wire: SemanticScholarAuthorPaper = serde_json::from_value(json!({
            "paperId": "opaque",
            "title": "T",
            "externalIds": {"PubMed": {"nested": true}}
        }))
        .unwrap();
        let error = map_paper_full(&wire, &mut Vec::new(), &mut Vec::new()).unwrap_err();
        assert!(format!("{error:?}").contains("unexpected type"));
    }

    #[test]
    fn admission_is_shared_between_compact_and_rich_views() {
        let mut wire = rich_row();
        wire["paperId"] = json!("  ");
        let parsed: SemanticScholarAuthorPaper = serde_json::from_value(wire).unwrap();
        assert!(!admitted(&parsed));
        let mut wire = rich_row();
        wire["title"] = json!("");
        let parsed: SemanticScholarAuthorPaper = serde_json::from_value(wire).unwrap();
        assert!(!admitted(&parsed));
        let mut wire = rich_row();
        wire["isOpenAccess"] = json!("maybe");
        assert!(serde_json::from_value::<SemanticScholarAuthorPaper>(wire).is_err());
    }

    #[test]
    fn evidence_url_encoder_matches_the_frozen_url_contract() {
        assert_eq!(
            paper_evidence_url("A/?#% \n雪"),
            "https://www.semanticscholar.org/paper/A%2F%3F%23%25%20%0A%E9%9B%AA"
        );
        assert_eq!(
            paper_evidence_url("$&;+, :=@"),
            "https://www.semanticscholar.org/paper/$&;+,%20:=@"
        );
        assert_eq!(
            paper_evidence_url("."),
            "https://www.semanticscholar.org/paper/%2E"
        );
        assert_eq!(
            paper_evidence_url(".."),
            "https://www.semanticscholar.org/paper/%2E%2E"
        );
        assert_eq!(
            paper_evidence_url("0123456789abcdef0123456789abcdef01234567"),
            "https://www.semanticscholar.org/paper/0123456789abcdef0123456789abcdef01234567"
        );
    }

    #[test]
    fn opaque_ids_get_urls_but_never_article_follow_ups() {
        let wire: SemanticScholarAuthorPaper = serde_json::from_value(json!({
            "paperId": "A/?#% \n雪",
            "title": "Hostile identifier fixture",
            "externalIds": {}
        }))
        .unwrap();
        let mut commands = Vec::new();
        let mut evidence = Vec::new();
        let rich = map_paper_full(&wire, &mut commands, &mut evidence).unwrap();
        assert!(commands.is_empty(), "no follow-up for an opaque ID");
        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].url,
            "https://www.semanticscholar.org/paper/A%2F%3F%23%25%20%0A%E9%9B%AA"
        );
        assert_eq!(rich.paper_id, "A/?#% \n雪", "JSON preserves the raw ID");
    }

    #[test]
    fn rich_follow_ups_prefer_pmid_doi_arxiv_then_hex_paper_id() {
        for (external, expected) in [
            (json!({"PubMed": "3170"}), "biomcp get article 3170"),
            (json!({"DOI": "10.5555/x"}), "biomcp get article 10.5555/x"),
            (
                json!({"ArXiv": "2110.01406"}),
                "biomcp get article arXiv:2110.01406",
            ),
        ] {
            let wire: SemanticScholarAuthorPaper = serde_json::from_value(json!({
                "paperId": "0123456789abcdef0123456789abcdef01234567",
                "title": "Preference fixture",
                "externalIds": external
            }))
            .unwrap();
            let mut commands = Vec::new();
            map_paper_full(&wire, &mut commands, &mut Vec::new()).unwrap();
            assert_eq!(commands, [expected.to_string()]);
        }
        let wire: SemanticScholarAuthorPaper = serde_json::from_value(json!({
            "paperId": "0123456789abcdef0123456789abcdef01234567",
            "title": "Hex fallback fixture",
            "externalIds": {}
        }))
        .unwrap();
        let mut commands = Vec::new();
        map_paper_full(&wire, &mut commands, &mut Vec::new()).unwrap();
        assert_eq!(
            commands,
            ["biomcp get article 0123456789abcdef0123456789abcdef01234567"]
        );
    }
}

#[cfg(test)]
mod wire_tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct AuthorEnv {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl AuthorEnv {
        fn new() -> Self {
            Self {
                previous: Vec::new(),
            }
        }
        fn set(&mut self, key: &'static str, value: impl AsRef<std::ffi::OsStr>) {
            self.previous.push((key, std::env::var_os(key)));
            // SAFETY: author tests that mutate provider variables use the
            // same serial-test key as the article test environment.
            unsafe { std::env::set_var(key, value) };
        }
    }

    impl Drop for AuthorEnv {
        fn drop(&mut self) {
            for (key, previous) in self.previous.drain(..).rev() {
                // SAFETY: restoring under the same serial guard.
                unsafe {
                    if let Some(value) = previous {
                        std::env::set_var(key, value);
                    } else {
                        std::env::remove_var(key);
                    }
                }
            }
        }
    }

    async fn spawn_papers_fixture(status: &str, body: String) -> (String, Arc<Mutex<Vec<String>>>) {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let requests = Arc::new(Mutex::new(Vec::new()));
        let logged = requests.clone();
        let status = status.to_string();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind author fixture");
        let address = listener.local_addr().expect("author fixture address");
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let logged = logged.clone();
                let body = body.clone();
                let status = status.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    if let Some(target) = request.split_whitespace().nth(1) {
                        logged.lock().unwrap().push(target.to_string());
                    }
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        (format!("http://{address}"), requests)
    }

    fn page_body(offset: u64, next: Option<u64>, rows: &[serde_json::Value]) -> String {
        serde_json::json!({"offset": offset, "next": next, "data": rows}).to_string()
    }

    async fn fixture_client(base: &str) -> SemanticScholarClient {
        SemanticScholarClient::new_with_cache_observers(base, |_, _| {}, |_, _| {}).unwrap()
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn compact_and_rich_request_exactly_one_page_and_never_prefetch_next() {
        let rows = vec![
            serde_json::json!({"paperId": "alpha", "title": "Alpha", "corpusId": 1, "abstract": "a"}),
            serde_json::json!({"paperId": "", "title": "Dropped", "paperIdIsBlank": true}),
            serde_json::json!({"paperId": "beta", "title": "Beta"}),
            serde_json::json!({"paperId": "alpha", "title": "Alpha duplicate", "abstract": "dup"}),
        ];
        let (base, requests) = spawn_papers_fixture("200 OK", page_body(0, Some(1), &rows)).await;
        for (full, fields) in [
            (
                false,
                "paperId%2CcorpusId%2CexternalIds%2Ctitle%2Cvenue%2Cyear%2C",
            ),
            (
                true,
                "paperId%2CcorpusId%2CexternalIds%2Ctitle%2Cabstract%2C",
            ),
        ] {
            let mut env = AuthorEnv::new();
            let cache = crate::test_support::TempDirGuard::new("author-papers-one-page");
            env.set("BIOMCP_CACHE_DIR", cache.path());
            env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
            let client = fixture_client(&base).await;
            let before = requests.lock().unwrap().len();
            if full {
                let result = crate::sources::semantic_scholar::with_test_client(
                    client,
                    papers_full("semanticscholar:1716151", 0, 10),
                )
                .await
                .unwrap();
                assert_eq!(result.papers.len(), 3);
                assert_eq!(result.papers[0].paper_id, "alpha");
                assert_eq!(result.papers[2].paper_id, "alpha");
                assert_eq!(result.papers[2].title, "Alpha duplicate");
                assert_eq!(
                    result._meta.next_commands.last().unwrap(),
                    "biomcp author papers semanticscholar:1716151 --full --limit 10 --offset 1"
                );
            } else {
                let result = crate::sources::semantic_scholar::with_test_client(
                    client,
                    papers("semanticscholar:1716151", 0, 10),
                )
                .await
                .unwrap();
                assert_eq!(result.papers.len(), 3);
                assert_eq!(
                    result
                        .papers
                        .iter()
                        .map(|paper| paper.paper_id.clone())
                        .collect::<Vec<_>>(),
                    vec![
                        Some("alpha".into()),
                        Some("beta".into()),
                        Some("alpha".into())
                    ],
                    "compact and rich retain identical admitted identity and order"
                );
                assert_eq!(
                    result._meta.next_commands.last().unwrap(),
                    "biomcp author papers semanticscholar:1716151 --limit 10 --offset 1"
                );
            }
            let after = requests.lock().unwrap().len();
            assert_eq!(after, before + 1, "exactly one page per call");
            let request = requests.lock().unwrap()[before].clone();
            assert!(
                request.starts_with("/graph/v1/author/1716151/papers?"),
                "{request}"
            );
            assert!(request.contains(&format!("fields={fields}")), "{request}");
            assert!(request.contains("offset=0"), "{request}");
            assert!(request.contains("limit=10"), "{request}");
        }
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn malformed_pages_fail_the_complete_command_with_no_partial_output() {
        for body in [
            page_body(
                7,
                None,
                &[serde_json::json!({"paperId": "x", "title": "T"})],
            ),
            serde_json::json!({"next": null, "data": []}).to_string(),
            page_body(
                0,
                Some(0),
                &[serde_json::json!({"paperId": "x", "title": "T"})],
            ),
            "{\"offset\":0,\"next\":null,\"data\":[{\"paperId\":\"x\",\"title\":\"T\",\"corpusId\":\"not-a-number\"}]}".to_string(),
        ] {
            let (base, _requests) = spawn_papers_fixture("200 OK", body).await;
            let mut env = AuthorEnv::new();
            let cache = crate::test_support::TempDirGuard::new("author-papers-malformed");
            env.set("BIOMCP_CACHE_DIR", cache.path());
            env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
            let client = fixture_client(&base).await;
            let error = crate::sources::semantic_scholar::with_test_client(
                client,
                papers_full("semanticscholar:1716151", 0, 10),
            )
            .await
            .unwrap_err();
            let message = format!("{error:?}");
            assert!(
                message.contains("semantic_scholar") || message.contains("Semantic Scholar"),
                "{message}"
            );
        }
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn sanitized_errors_do_not_leak_provider_bodies_urls_or_credentials() {
        let (base, _requests) =
            spawn_papers_fixture("503 Service Unavailable", "secret-provider-body".into()).await;
        let mut env = AuthorEnv::new();
        let cache = crate::test_support::TempDirGuard::new("author-papers-unavailable");
        env.set("BIOMCP_CACHE_DIR", cache.path());
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        let client = fixture_client(&base).await;
        let error = crate::sources::semantic_scholar::with_test_client(
            client,
            papers_full("semanticscholar:1716151", 0, 10),
        )
        .await
        .unwrap_err();
        let message = format!("{error:?}");
        assert!(!message.contains("secret-provider-body"), "{message}");
        assert!(!message.contains(&base), "{message}");
    }

    #[test]
    fn command_deadline_is_thirty_five_seconds() {
        assert_eq!(AUTHOR_PAPERS_COMMAND_DEADLINE, Duration::from_secs(35));
    }

    #[tokio::test(start_paused = true)]
    #[serial_test::serial(source_env)]
    async fn stalled_request_future_fails_at_the_absolute_deadline_without_late_output() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let stalled = tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut sink = vec![0_u8; 1024];
                use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
                let _ = stream.read(&mut sink).await;
                let _ = stream.write_all(&[]).await;
                std::future::pending::<()>().await;
            }
        });
        let mut env = AuthorEnv::new();
        let cache = crate::test_support::TempDirGuard::new("author-papers-stalled");
        env.set("BIOMCP_CACHE_DIR", cache.path());
        let stalled_base = format!("http://{address}");
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &stalled_base);
        env.set("BIOMCP_TEST_AUTHOR_PAPERS_DEADLINE_MS", "300");
        let client =
            SemanticScholarClient::new_with_cache_observers(&stalled_base, |_, _| {}, |_, _| {})
                .unwrap();
        let started = tokio::time::Instant::now();
        let error = crate::sources::semantic_scholar::with_test_client(
            client,
            papers_full("semanticscholar:1716151", 0, 10),
        )
        .await
        .unwrap_err();
        assert!(
            tokio::time::Instant::now() - started >= Duration::from_millis(300),
            "the real request future is held across the boundary"
        );
        let message = format!("{error:?}");
        assert!(
            message.contains("unavailable") || message.contains("deadline"),
            "{message}"
        );
        assert!(!message.contains("http://"), "{message}");
        stalled.abort();
    }
}

#[cfg(test)]
mod orcid_works_tests {
    use super::*;

    fn work(put_code: u64, ids: &[(&str, &str)]) -> crate::sources::orcid::OrcidSelectedWork {
        crate::sources::orcid::OrcidSelectedWork {
            put_code,
            title: "A claimed work".into(),
            journal: Some("A Journal".into()),
            year: Some(2024),
            external_ids: ids
                .iter()
                .map(|(kind, value)| (kind.to_string(), value.to_string()))
                .collect(),
        }
    }

    #[test]
    fn canonical_identifier_normalizes_every_recognized_kind() {
        assert_eq!(
            canonical_identifier("doi", "https://doi.org/10.1/Example"),
            Some(("doi".into(), "10.1/example".into()))
        );
        assert_eq!(
            canonical_identifier("pmid", "000123"),
            Some(("pmid".into(), "123".into()))
        );
        assert_eq!(canonical_identifier("pmid", "0"), None);
        assert_eq!(
            canonical_identifier("pmc", "pmc000456"),
            Some(("pmcid".into(), "PMC456".into()))
        );
        assert_eq!(
            canonical_identifier("arxiv", "arxiv:2401.00001"),
            Some(("arxiv".into(), "2401.00001".into()))
        );
        assert_eq!(canonical_identifier("doi", "no-slash"), None);
        assert_eq!(canonical_identifier("unknown", "keep"), None);
    }

    #[test]
    fn identifier_lists_preserve_unrecognized_types_in_first_occurrence_order() {
        // Wire types arrive lowercased from the source validation layer.
        let ids = canonical_identifiers(&work(
            1,
            &[("doi", "10.1/a"), ("unknown", "keep-me"), ("doi", "10.1/a")],
        ));
        assert_eq!(
            ids,
            vec![
                ("doi".to_string(), "10.1/a".into()),
                ("unknown".to_string(), "keep-me".into()),
            ]
        );
    }

    /// Ticket 1142: identifiers that exist only in excluded locations —
    /// group level, a PRIVATE summary, or a nonselected PUBLIC summary —
    /// never reach the projected paper, its commands, or its evidence.
    #[test]
    fn excluded_location_identifiers_never_reach_the_projected_paper() {
        let hostile = serde_json::json!({
            "path": "/0000-0002-1825-0097/works",
            "group": [{
                "external-ids": {"external-id": [
                    {"external-id-type": "pmid", "external-id-value": "9991", "external-id-relationship": "SELF"}
                ]},
                "work-summary": [
                    {"visibility": "PRIVATE", "put-code": 91, "display-index": "50",
                     "title": {"title": {"value": "Private summary"}},
                     "external-ids": {"external-id": [
                        {"external-id-type": "pmid", "external-id-value": "9992", "external-id-relationship": "SELF"}
                     ]}},
                    {"visibility": "PUBLIC", "put-code": 92, "display-index": "1",
                     "title": {"title": {"value": "Public but not selected"}},
                     "external-ids": {"external-id": [
                        {"external-id-type": "pmid", "external-id-value": "9993", "external-id-relationship": "SELF"}
                     ]}},
                    {"visibility": "PUBLIC", "put-code": 42, "display-index": "2",
                     "title": {"title": {"value": "Selected representative"}},
                     "external-ids": {"external-id": [
                        {"external-id-type": "pmid", "external-id-value": "42", "external-id-relationship": "SELF"}
                     ]}}
                ]
            }]
        })
        .to_string();
        let response: crate::sources::orcid::OrcidWorksResponse =
            serde_json::from_str(&hostile).expect("valid wire body");
        let selected = response.selected_works().expect("one representative");
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].put_code, 42);
        let requested: ProviderAuthorId = "orcid:0000-0002-1825-0097".parse().unwrap();
        let mut commands = Vec::new();
        let mut evidence = Vec::new();
        let paper = orcid_paper(&requested, &selected[0], &mut commands, &mut evidence);
        assert_eq!(paper.pmid.as_deref(), Some("42"));
        let rendered = serde_json::to_string(&paper).unwrap();
        for forbidden in ["9991", "9992", "9993"] {
            assert!(
                !rendered.contains(forbidden),
                "{forbidden} leaked: {rendered}"
            );
        }
        assert_eq!(commands, ["biomcp get article 42".to_string()]);
    }

    /// Ticket 1142: `--full` on an ORCID ID is rejected statically, before
    /// any client construction, with zero provider requests of any kind.
    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn full_mode_on_an_orcid_id_is_rejected_with_zero_requests() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let logged = requests.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind author orcid fixture");
        let address = listener.local_addr().expect("fixture address");
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let logged = logged.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    if let Some(target) = request.split_whitespace().nth(1) {
                        logged.lock().unwrap().push(target.to_string());
                    }
                    let response =
                        "HTTP/1.1 500 X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
        });
        let base = format!("http://{address}");
        struct EnvRestore {
            previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
        }
        impl EnvRestore {
            fn set(&mut self, key: &'static str, value: &str) {
                self.previous.push((key, std::env::var_os(key)));
                // SAFETY: serial-guarded test environment mutation.
                unsafe { std::env::set_var(key, value) };
            }
        }
        impl Drop for EnvRestore {
            fn drop(&mut self) {
                for (key, value) in self.previous.drain(..) {
                    // SAFETY: restoring the serial-guarded environment.
                    unsafe {
                        match value {
                            Some(value) => std::env::set_var(key, value),
                            None => std::env::remove_var(key),
                        }
                    }
                }
            }
        }
        let mut env = EnvRestore {
            previous: Vec::new(),
        };
        env.set("BIOMCP_S2_BASE", &base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set("BIOMCP_ORCID_BASE", &base);
        env.set("ORCID_ACCESS_TOKEN", "fixture-public-read-token");
        let error = papers_full("orcid:0000-0002-1825-0097", 0, 10)
            .await
            .expect_err("static rejection");
        match error {
            BioMcpError::InvalidArgument(message) => assert_eq!(
                message,
                "--full is available only for semanticscholar: author IDs; omit --full for ORCID claimed works"
            ),
            other => panic!("wrong error: {other:?}"),
        }
        assert!(
            requests.lock().unwrap().is_empty(),
            "no provider request may be made"
        );
    }

    #[test]
    fn cross_group_dedupe_drops_only_on_recognized_identity() {
        let selected = vec![
            work(1, &[("doi", "10.1/a"), ("grant", "g1")]),
            // Shares only the preserved unrecognized grant with work 1.
            work(2, &[("pmid", "100"), ("grant", "g1")]),
            // Shares the recognized DOI with work 1: dropped.
            work(3, &[("doi", "10.1/a")]),
            // No recognized IDs at all: never dropped.
            work(4, &[("grant", "g2")]),
            // No identifiers at all: never dropped.
            work(5, &[]),
        ];
        let retained = retain_deduplicated(&selected);
        assert_eq!(
            retained.iter().map(|w| w.put_code).collect::<Vec<_>>(),
            [1, 2, 4, 5],
            "only recognized canonical identity may drop a group"
        );
    }

    #[test]
    fn lexical_first_normalized_value_wins_the_flattened_slot() {
        // Provider order carries 10.1/z first, but the flattened slot takes
        // the lexically first normalized value.
        let requested: ProviderAuthorId = "orcid:0000-0002-1825-0097".parse().unwrap();
        let selected = work(7, &[("doi", "10.1/z"), ("doi", "10.1/a")]);
        let mut commands = Vec::new();
        let mut evidence = Vec::new();
        let paper = orcid_paper(&requested, &selected, &mut commands, &mut evidence);
        assert_eq!(paper.doi.as_deref(), Some("10.1/a"));
    }

    #[test]
    fn orcid_page_bounds_pin_first_middle_terminal_and_beyond_total() {
        // 25 retained works, limit 10.
        assert_eq!(orcid_page_bounds(25, 0, 10), (0, 10, Some(10), 25));
        assert_eq!(orcid_page_bounds(25, 10, 10), (10, 20, Some(20), 25));
        // Terminal page: 5 returned, 25 is not below total, so no next.
        assert_eq!(orcid_page_bounds(25, 20, 10), (20, 25, None, 25));
        // Offset exactly at total: a successful empty page.
        assert_eq!(orcid_page_bounds(25, 25, 10), (25, 25, None, 25));
        // Offset beyond total but within the static cap: still empty.
        assert_eq!(orcid_page_bounds(25, 30, 10), (25, 25, None, 25));
        // A single-work terminal page has no next.
        assert_eq!(orcid_page_bounds(1, 0, 10), (0, 1, None, 1));
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn an_orcid_offset_above_ten_thousand_is_a_static_rejection() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let logged = requests.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind offset fixture");
        let address = listener.local_addr().expect("offset fixture address");
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let logged = logged.clone();
                tokio::spawn(async move {
                    let mut request = vec![0_u8; 16 * 1024];
                    let length = stream.read(&mut request).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&request[..length]);
                    if let Some(target) = request.split_whitespace().nth(1) {
                        logged.lock().unwrap().push(target.to_string());
                    }
                    let _ = stream
                        .write_all(
                            b"HTTP/1.1 500 X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        )
                        .await;
                });
            }
        });
        let base = format!("http://{address}");
        struct EnvRestore {
            previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
        }
        impl EnvRestore {
            fn set(&mut self, key: &'static str, value: &str) {
                self.previous.push((key, std::env::var_os(key)));
                // SAFETY: serial-guarded test environment mutation.
                unsafe { std::env::set_var(key, value) };
            }
        }
        impl Drop for EnvRestore {
            fn drop(&mut self) {
                for (key, value) in self.previous.drain(..) {
                    // SAFETY: restoring the serial-guarded environment.
                    unsafe {
                        match value {
                            Some(value) => std::env::set_var(key, value),
                            None => std::env::remove_var(key),
                        }
                    }
                }
            }
        }
        let mut env = EnvRestore {
            previous: Vec::new(),
        };
        env.set("BIOMCP_ORCID_BASE", &base);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
        env.set("ORCID_ACCESS_TOKEN", "fixture-public-read-token");
        let error = papers("orcid:0000-0002-1825-0097", 10_001, 10)
            .await
            .expect_err("static rejection");
        match error {
            BioMcpError::InvalidArgument(message) => assert_eq!(
                message,
                "--offset must be at most 10000 for orcid: author claimed works"
            ),
            other => panic!("wrong error: {other:?}"),
        }
        assert!(
            requests.lock().unwrap().is_empty(),
            "no provider request may be planned or sent"
        );
    }

    #[test]
    fn orcid_paper_projects_the_frozen_shape_with_row_command_priority() {
        let requested: ProviderAuthorId = "orcid:0000-0002-1825-0097".parse().unwrap();
        let mut commands = Vec::new();
        let mut evidence = Vec::new();
        let paper = orcid_paper(
            &requested,
            &work(
                42,
                &[("pmid", "123"), ("doi", "10.1/example"), ("pmc", "PMC456")],
            ),
            &mut commands,
            &mut evidence,
        );
        assert_eq!(paper.paper_id, None);
        assert_eq!(paper.pmid.as_deref(), Some("123"));
        assert_eq!(paper.pmcid.as_deref(), Some("PMC456"));
        assert_eq!(paper.doi.as_deref(), Some("10.1/example"));
        assert_eq!(
            paper.work_id.as_deref(),
            Some("orcid:0000-0002-1825-0097/work:42")
        );
        assert_eq!(
            commands,
            ["biomcp get article 123".to_string()],
            "PMID wins the follow-up priority"
        );
        assert_eq!(
            evidence[0].url,
            "https://orcid.org/0000-0002-1825-0097/work/42"
        );
        let json = serde_json::to_value(&paper).unwrap();
        assert_eq!(
            json["identifiers"][0],
            serde_json::json!({"type":"pmid","value":"123"})
        );
        assert!(json.get("paper_id").is_none());
    }
}
