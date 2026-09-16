//! Directed citation-evidence traversal for ticket 1145.

use super::{
    GraphDirection, related_paper_from_semantic_scholar, resolve_semantic_scholar_input_id,
};
use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::opencitations::{
    OPENCITATIONS_BASE, OpenCitationsClient, OpenCitationsEdge, normalize_doi, valid_creation,
    valid_oci,
};
use crate::sources::semantic_scholar::{SemanticScholarClient, SemanticScholarPaper};

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
const OPENCITATIONS_SOURCE: &str = "opencitations";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CitationEvidenceStatus {
    ContextFromProvider,
    ContextFromFulltext,
    FulltextUnavailable,
    ReferenceConfirmedWithoutPassage,
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
            Self::ReferenceConfirmedWithoutPassage => {
                "An open citation index confirmed this directed edge, but no open passage is available."
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
            Self::ReferenceConfirmedWithoutPassage => Some(OPENCITATIONS_SOURCE),
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

/// The confirmed directed edge an open citation index supplies when no open
/// passage is available. The `citing` and `cited` members are locally
/// constructed from the command's own normalized DOIs, and the `oci` and
/// `creation` members are validated provider values, never raw echoes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CitationEvidenceConfirmation {
    pub source: String,
    pub oci: String,
    pub citing: String,
    pub cited: String,
    pub creation: Option<String>,
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
    pub confirmation: Option<CitationEvidenceConfirmation>,
    pub _meta: CitationEvidenceMeta,
}

/// The shared evidence command: `biomcp article citation-evidence <citing> <cited>`.
pub(super) fn citation_evidence_command(citing_id: &str, cited_id: &str) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["article", "citation-evidence"])
        .arg(citing_id.trim())
        .arg(cited_id.trim())
        .render_shell()
}

/// The executable edge ID for the evidence command, using the existing
/// related-paper preference: PMID, then DOI, then arXiv, then paper ID.
pub(super) fn edge_paper_command_id(paper: &super::ArticleRelatedPaper) -> Option<String> {
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
pub(super) fn graph_edge_evidence_meta(
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
        message: "invocation deadline exceeded".into(),
    }
}

/// The sidecar is best-effort: an unresolvable cache configuration leaves the
/// command running exactly as before the sidecar existed.
fn citation_evidence_cache_root() -> Option<std::path::PathBuf> {
    crate::cache::resolve_cache_config()
        .ok()
        .map(|config| config.cache_root)
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
    tokio::time::Instant::now() + command_deadline_budget()
}

fn command_deadline_budget() -> Duration {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("BIOMCP_TEST_CITATION_COMMAND_DEADLINE_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    CITATION_EVIDENCE_COMMAND_DEADLINE
}

fn graph_deadline_budget() -> Duration {
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("BIOMCP_TEST_CITATION_GRAPH_DEADLINE_MS")
        && let Ok(millis) = value.trim().parse::<u64>()
    {
        return Duration::from_millis(millis);
    }
    CITATION_EVIDENCE_GRAPH_DEADLINE
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
    let graph_deadline = now + graph_deadline_budget();
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
pub(super) enum JatsEvidenceOutcome {
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
    let Some(pmcid) = citing_pmcid(citing_paper, deadline).await? else {
        return Ok(JatsEvidenceOutcome::FulltextUnavailable);
    };
    let Some(xml) = fetch_citing_fulltext(&pmcid, deadline).await? else {
        return Ok(JatsEvidenceOutcome::FulltextUnavailable);
    };
    run_jats_extraction(&pmcid, &xml, &cited_target_ids(cited_paper), deadline).await
}

async fn citing_pmcid(
    paper: &SemanticScholarPaper,
    deadline: tokio::time::Instant,
) -> Result<Option<String>, BioMcpError> {
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
    let (pmid, doi) = (
        external.and_then(|ids| ids.pubmed.as_deref()),
        external.and_then(|ids| ids.doi.as_deref()),
    );
    let resolved = match (pmid, doi) {
        (Some(pmid), _) => {
            tokio::time::timeout_at(deadline, bridge.pmid_to_pmcid(pmid.trim())).await
        }
        (None, Some(doi)) => {
            tokio::time::timeout_at(deadline, bridge.doi_to_pmcid(doi.trim())).await
        }
        (None, None) => Ok(Ok(None)),
    };
    match resolved {
        // A bridge failure leaves the JATS attempt unavailable without
        // failing the command; deadline expiry is the bounded command error.
        Ok(outcome) => Ok(outcome.unwrap_or(None)),
        Err(_) => Err(command_deadline_error()),
    }
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

pub(super) async fn run_jats_extraction(
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

/// The citing or cited paper's DOI under the JATS reference-extractor rules.
fn external_doi(paper: &SemanticScholarPaper) -> Option<String> {
    paper
        .external_ids
        .as_ref()
        .and_then(|ids| ids.doi.as_deref())
        .and_then(normalize_doi)
}

/// The production canonical URL of one reference list. The fixture base is
/// never part of the output.
fn opencitations_references_url(citing_doi: &str) -> String {
    format!("{OPENCITATIONS_BASE}/references/doi:{citing_doi}")
}

/// The first row whose `cited` token list names the normalized cited DOI.
/// Only the `doi:` token form matches; `omid:`, `openalex:`, and `pmid:`
/// tokens are never consulted.
fn matching_opencitations_edge<'a>(
    edges: &'a [OpenCitationsEdge],
    cited_doi: &str,
) -> Option<&'a OpenCitationsEdge> {
    let wanted = format!("doi:{cited_doi}");
    edges.iter().find(|edge| {
        edge.cited
            .split_ascii_whitespace()
            .any(|token| token.eq_ignore_ascii_case(&wanted))
    })
}

/// Builds the frozen confirmation member. A matching row with an OCI or a
/// creation value outside the accepted shapes fails closed, so the caller
/// keeps the bounded outcome instead of publishing provider text.
fn confirmation_from_edge(
    edge: &OpenCitationsEdge,
    citing_doi: &str,
    cited_doi: &str,
    evidence_url: &str,
) -> Option<CitationEvidenceConfirmation> {
    if !valid_oci(&edge.oci) {
        return None;
    }
    Some(CitationEvidenceConfirmation {
        source: OPENCITATIONS_SOURCE.to_string(),
        oci: edge.oci.clone(),
        citing: format!("doi:{citing_doi}"),
        cited: format!("doi:{cited_doi}"),
        creation: edge
            .creation
            .as_deref()
            .filter(|value| valid_creation(value))
            .map(str::to_string),
        evidence_url: evidence_url.to_string(),
    })
}

/// The confirmation phase: at most one request, placed after the full-text
/// attempt and under the remaining absolute command deadline. Every failure
/// is bounded unavailability, never a command error, and a deadline with no
/// room left for the request is unavailability too.
async fn opencitations_edges(
    citing_doi: &str,
    deadline: tokio::time::Instant,
) -> Result<Vec<OpenCitationsEdge>, ()> {
    if tokio::time::Instant::now() >= deadline {
        return Err(());
    }
    let Ok(client) = OpenCitationsClient::new() else {
        return Err(());
    };
    match tokio::time::timeout_at(deadline, client.references_by_doi(citing_doi)).await {
        Ok(Ok(edges)) => Ok(edges),
        Ok(Err(_)) | Err(_) => Err(()),
    }
}

pub async fn citation_evidence(
    citing_id: &str,
    cited_id: &str,
    force_fulltext: bool,
) -> Result<ArticleCitationEvidenceResult, BioMcpError> {
    let deadline = evidence_deadline();
    let client = SemanticScholarClient::new()?;
    let europe = EuropePmcClient::new()?;
    let (citing, citing_paper) =
        match tokio::time::timeout_at(deadline, resolve_citation_seed(citing_id, &client, &europe))
            .await
        {
            Ok(result) => result?,
            Err(_) => return Err(command_deadline_error()),
        };
    let (cited, cited_paper) =
        match tokio::time::timeout_at(deadline, resolve_citation_seed(cited_id, &client, &europe))
            .await
        {
            Ok(result) => result?,
            Err(_) => return Err(command_deadline_error()),
        };
    let citing_pid = valid_paper_id(citing.paper_id.as_deref())
        .ok_or_else(|| provider_decode_error("citing seed lacks a valid paper ID"))?;
    let cited_pid = valid_paper_id(cited.paper_id.as_deref())
        .ok_or_else(|| provider_decode_error("cited seed lacks a valid paper ID"))?;

    // The read follows seed resolution so every hit is paired with a live
    // resolution of the caller's spelling. A miss is any absent, expired,
    // mismatched, or non-evidence record, and the pipeline below runs
    // unchanged.
    let cache_root = citation_evidence_cache_root();
    if let Some(cache_root) = &cache_root
        && let Some(cached) = crate::cache::read_citation_evidence(
            cache_root,
            &citing_pid,
            &cited_pid,
            force_fulltext,
        )
    {
        return Ok(cached);
    }

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
    } else if jats.is_some() {
        // A forced or contextless attempt that could not produce parsed JATS
        // reports the bounded unavailability; it never falls back to the
        // provider-context state, and the provider contexts stay retained.
        status = CitationEvidenceStatus::FulltextUnavailable;
    }

    // The confirmation phase runs exactly where the assembled outcome would
    // otherwise be the bounded full-text dead end, and both papers carry a
    // normalizable DOI. Every other state already holds evidence or a
    // resolved reference and is left untouched.
    let mut opencitations_status = "not_requested";
    let mut opencitations_url: Option<String> = None;
    let mut confirmation: Option<CitationEvidenceConfirmation> = None;
    if status == CitationEvidenceStatus::FulltextUnavailable
        && let (Some(citing_doi), Some(cited_doi)) =
            (external_doi(&citing_paper), external_doi(&cited_paper))
    {
        match opencitations_edges(&citing_doi, deadline).await {
            Ok(edges) => {
                let url = opencitations_references_url(&citing_doi);
                match matching_opencitations_edge(&edges, &cited_doi) {
                    Some(edge) => {
                        match confirmation_from_edge(edge, &citing_doi, &cited_doi, &url) {
                            Some(value) => {
                                status = CitationEvidenceStatus::ReferenceConfirmedWithoutPassage;
                                confirmation = Some(value);
                                opencitations_status = "available";
                                opencitations_url = Some(url);
                            }
                            // A matching row with a shape-invalid OCI fails closed.
                            None => opencitations_status = "unavailable",
                        }
                    }
                    // A parsed reference list without the edge keeps the
                    // bounded dead end and still contributes its URL.
                    None => {
                        opencitations_status = "available";
                        opencitations_url = Some(url);
                    }
                }
            }
            Err(()) => opencitations_status = "unavailable",
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
    if let Some(url) = opencitations_url {
        evidence_urls.push(CitationEvidenceUrl {
            source: OPENCITATIONS_SOURCE.to_string(),
            url,
        });
    }

    let result = ArticleCitationEvidenceResult {
        citing,
        cited,
        message: status.message().to_string(),
        source: status.source().map(str::to_string),
        provider_contexts,
        passages,
        fulltext_locator: locator,
        confirmation,
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
                CitationEvidenceSourceStatus {
                    source: OPENCITATIONS_SOURCE.to_string(),
                    status: opencitations_status.to_string(),
                },
            ],
            evidence_urls,
            next_commands: Vec::new(),
        },
    };
    if let Some(cache_root) = &cache_root {
        crate::cache::write_citation_evidence(cache_root, &citing_pid, &cited_pid, &result);
    }
    Ok(result)
}
