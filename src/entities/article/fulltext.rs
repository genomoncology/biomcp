use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use tracing::debug;

use super::{
    Article, ArticleFulltextAttempt, ArticleFulltextAttemptCoverage, ArticleFulltextAttemptOutcome,
    ArticleFulltextAttemptReason, ArticleFulltextAttemptSourceKind, ArticleFulltextCacheState,
    ArticleFulltextCoverage, ArticleFulltextCoverageKind, ArticleFulltextKind,
    ArticleFulltextManifest, ArticleFulltextManifestKind, ArticleFulltextProvenance,
    ArticleFulltextProvider, ArticleFulltextQuality, ArticleFulltextReuse, ArticleFulltextSource,
};
use crate::entities::section_outcome::SectionOutcome;
use crate::error::BioMcpError;
use crate::sources::RequestBuilderSourceContextExt;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::ncbi_efetch::NcbiEfetchClient;
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_article::{PmcHtmlCacheState, PmcHtmlFetchOutcome};
use crate::sources::pmc_oa::PmcOaClient;
use crate::transform;
use crate::transform::article::{ArticleDocumentCoverage, ClassifiedArticleDocument};
use crate::utils::download;

const FULLTEXT_CACHE_VERSION: &str = "v4";
const ARTICLE_FULLTEXT_API: &str = "article";
const PDF_MAX_BODY_BYTES: usize = 20 * 1024 * 1024;
const PDF_PAGE_LIMIT: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XmlWaterfallWinner {
    EuropePmcPmc,
    NcbiEfetchPmc,
    PmcOaArchive,
    EuropePmcMed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XmlFulltextAttempt {
    EuropePmcPmc,
    NcbiEfetchPmc,
    PmcOaArchive,
    EuropePmcMed,
}

impl XmlFulltextAttempt {
    fn winner(self) -> XmlWaterfallWinner {
        match self {
            Self::EuropePmcPmc => XmlWaterfallWinner::EuropePmcPmc,
            Self::NcbiEfetchPmc => XmlWaterfallWinner::NcbiEfetchPmc,
            Self::PmcOaArchive => XmlWaterfallWinner::PmcOaArchive,
            Self::EuropePmcMed => XmlWaterfallWinner::EuropePmcMed,
        }
    }
}

enum FulltextStepOutcome<T> {
    Data(T),
    Empty,
    Unusable(BioMcpError),
    Failed(BioMcpError),
}

pub(super) enum PdfDiscoveryAttempt {
    Ineligible,
    Data(String),
    Empty,
    Failed(BioMcpError),
}

#[derive(Default)]
struct FulltextAttemptState {
    healthy_sources: Vec<String>,
    failure: Option<BioMcpError>,
    best_partial: Option<ArticleFulltextCoverageKind>,
    attempts: Vec<ArticleFulltextAttempt>,
}

impl FulltextAttemptState {
    fn record_empty(&mut self, source: &str) {
        if !self.healthy_sources.iter().any(|value| value == source) {
            self.healthy_sources.push(source.to_string());
        }
    }

    fn record_failure(&mut self, err: BioMcpError) {
        if self.failure.is_none() {
            self.failure = Some(err);
        }
    }

    fn record_attempt(
        &mut self,
        source: &ArticleFulltextSource,
        source_kind: ArticleFulltextAttemptSourceKind,
        coverage: ArticleFulltextAttemptCoverage,
        outcome: ArticleFulltextAttemptOutcome,
        cache_state: ArticleFulltextCacheState,
        reason: ArticleFulltextAttemptReason,
    ) {
        self.attempts.push(ArticleFulltextAttempt {
            provider: manifest_provider(source),
            source_kind,
            coverage,
            outcome,
            cache_state,
            reason,
        });
    }

    fn observe_partial(&mut self, coverage: ArticleDocumentCoverage) {
        let observed = match coverage {
            ArticleDocumentCoverage::AbstractOnly => ArticleFulltextCoverageKind::AbstractOnly,
            ArticleDocumentCoverage::MetadataOnly => ArticleFulltextCoverageKind::MetadataOnly,
            ArticleDocumentCoverage::FullText => return,
        };
        if self.best_partial != Some(ArticleFulltextCoverageKind::AbstractOnly) {
            self.best_partial = Some(observed);
        }
    }

    fn final_coverage(&self) -> ArticleFulltextCoverageKind {
        self.best_partial.unwrap_or_else(|| {
            if self.failure.is_some() {
                ArticleFulltextCoverageKind::Unavailable
            } else {
                ArticleFulltextCoverageKind::None
            }
        })
    }
}

struct HtmlResolution {
    outcome: FulltextStepOutcome<ClassifiedArticleDocument>,
    cache_state: ArticleFulltextCacheState,
}

fn cache_kind_name(kind: ArticleFulltextKind) -> &'static str {
    match kind {
        ArticleFulltextKind::JatsXml => "jats_xml",
        ArticleFulltextKind::Html => "html",
        ArticleFulltextKind::Pdf => "pdf",
    }
}

fn xml_source_metadata(winner: XmlWaterfallWinner) -> ArticleFulltextSource {
    let (label, source) = match winner {
        XmlWaterfallWinner::EuropePmcPmc => ("Europe PMC XML", "Europe PMC"),
        XmlWaterfallWinner::NcbiEfetchPmc => ("NCBI EFetch PMC XML", "NCBI EFetch"),
        XmlWaterfallWinner::PmcOaArchive => ("PMC OA Archive XML", "PMC OA"),
        XmlWaterfallWinner::EuropePmcMed => ("Europe PMC MED XML", "Europe PMC"),
    };
    ArticleFulltextSource {
        kind: ArticleFulltextKind::JatsXml,
        label: label.to_string(),
        source: source.to_string(),
    }
}

fn html_source_metadata() -> ArticleFulltextSource {
    ArticleFulltextSource {
        kind: ArticleFulltextKind::Html,
        label: "PMC HTML".to_string(),
        source: "PMC".to_string(),
    }
}

fn pdf_source_metadata() -> ArticleFulltextSource {
    ArticleFulltextSource {
        kind: ArticleFulltextKind::Pdf,
        label: "Semantic Scholar PDF".to_string(),
        source: "Semantic Scholar".to_string(),
    }
}

fn manifest_provider(source: &ArticleFulltextSource) -> ArticleFulltextProvider {
    ArticleFulltextProvider {
        label: source.label.clone(),
        source: source.source.clone(),
    }
}

fn clean_manifest_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn manifest_reuse(license: Option<String>) -> ArticleFulltextReuse {
    match license {
        Some(license) => ArticleFulltextReuse {
            license_present: true,
            license: Some(license),
            license_source: None,
            reuse_warning: None,
        },
        None => ArticleFulltextReuse {
            license_present: false,
            license: None,
            license_source: None,
            reuse_warning: Some(
                "License/reuse status is unknown; verify rights before reuse.".to_string(),
            ),
        },
    }
}

fn manifest_provenance(
    article: &Article,
    pdf_fallback_used: bool,
    package_url: Option<String>,
    retracted: Option<bool>,
) -> ArticleFulltextProvenance {
    ArticleFulltextProvenance {
        open_access: article.open_access,
        retracted: retracted.or(article.europepmc_retracted),
        package_url,
        pdf_fallback_used,
    }
}

fn manifest_quality(fulltext_signal: bool) -> ArticleFulltextQuality {
    ArticleFulltextQuality {
        has_fulltext_signal: fulltext_signal,
        ..ArticleFulltextQuality::default()
    }
}

pub(super) fn fulltext_cache_key(kind: ArticleFulltextKind, id: &str) -> String {
    format!(
        "article-fulltext-{FULLTEXT_CACHE_VERSION}:{}:{id}",
        cache_kind_name(kind)
    )
}

fn first_cache_identifier<'a>(article: &'a Article, requested_id: &'a str) -> &'a str {
    article
        .pmid
        .as_deref()
        .or(article.doi.as_deref())
        .or(article.pmcid.as_deref())
        .unwrap_or(requested_id)
}

async fn classify_fulltext_xml(xml: String) -> Result<ClassifiedArticleDocument, BioMcpError> {
    tokio::task::spawn_blocking(move || transform::article::classify_jats_document(&xml))
        .await
        .map_err(|err| BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: format!("Full text XML classification worker failed: {err}"),
        })?
        .map_err(|_| BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: "Full text XML was unusable".to_string(),
        })
}

async fn render_fulltext_pdf(bytes: Vec<u8>, page_limit: usize) -> Result<String, BioMcpError> {
    tokio::task::spawn_blocking(move || {
        transform::article::extract_text_from_pdf(&bytes, page_limit)
    })
    .await
    .map_err(|err| BioMcpError::Api {
        api: ARTICLE_FULLTEXT_API.to_string(),
        message: format!("Full text PDF render worker failed: {err}"),
    })?
}

fn pdf_content_type_is_supported(content_type: Option<&reqwest::header::HeaderValue>) -> bool {
    let Some(content_type) = content_type.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default();
    media_type.eq_ignore_ascii_case("application/pdf")
}

fn pdf_body_signature_matches(body: &[u8]) -> bool {
    body.starts_with(b"%PDF-")
}

fn documented_fulltext_absence(status: reqwest::StatusCode) -> bool {
    matches!(
        status,
        reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::NO_CONTENT
    )
}

fn parse_pdf_url(raw_url: &str) -> Option<Url> {
    Url::parse(raw_url.trim()).ok()
}

fn xml_fulltext_attempts(
    article: &Article,
    resolved_pmcid: Option<&str>,
) -> Vec<XmlFulltextAttempt> {
    let mut attempts = Vec::new();
    if resolved_pmcid.is_some() {
        attempts.push(XmlFulltextAttempt::EuropePmcPmc);
        attempts.push(XmlFulltextAttempt::NcbiEfetchPmc);
        attempts.push(XmlFulltextAttempt::PmcOaArchive);
    }
    if article.pmid.as_deref().is_some() {
        attempts.push(XmlFulltextAttempt::EuropePmcMed);
    }
    attempts
}

pub(super) fn pdf_discovery_attempt(
    article: &Article,
    allow_pdf: bool,
    enrichment: Result<(), BioMcpError>,
) -> PdfDiscoveryAttempt {
    if !allow_pdf {
        return PdfDiscoveryAttempt::Ineligible;
    }
    if let Err(err) = enrichment {
        return PdfDiscoveryAttempt::Failed(err);
    }
    article
        .semantic_scholar
        .as_ref()
        .and_then(|value| value.open_access_pdf.as_ref())
        .map(|value| value.url.trim())
        .filter(|value| !value.is_empty())
        .map(|value| PdfDiscoveryAttempt::Data(value.to_string()))
        .unwrap_or(PdfDiscoveryAttempt::Empty)
}

async fn try_resolve_html(pmcid: &str, requested_id: &str) -> HtmlResolution {
    let fetched = crate::sources::pmc_article::fetch_html(pmcid, requested_id).await;
    let cache_state = match fetched.cache_state {
        PmcHtmlCacheState::Hit => ArticleFulltextCacheState::Hit,
        PmcHtmlCacheState::Miss => ArticleFulltextCacheState::Miss,
        PmcHtmlCacheState::Bypass => ArticleFulltextCacheState::Bypass,
    };
    let outcome = match fetched.outcome {
        PmcHtmlFetchOutcome::Data { html, url } => {
            match transform::article::classify_html_document(&html, url.as_str()) {
                Ok(classified) => FulltextStepOutcome::Data(classified),
                Err(err) => {
                    debug!(?err, requested_id, pmcid, "PMC HTML classification failed");
                    FulltextStepOutcome::Unusable(BioMcpError::Api {
                        api: ARTICLE_FULLTEXT_API.to_string(),
                        message: "PMC HTML content was unusable".to_string(),
                    })
                }
            }
        }
        PmcHtmlFetchOutcome::Empty => FulltextStepOutcome::Empty,
        PmcHtmlFetchOutcome::Unusable(err) => FulltextStepOutcome::Unusable(err),
        PmcHtmlFetchOutcome::Failed(err) => FulltextStepOutcome::Failed(err),
    };
    HtmlResolution {
        outcome,
        cache_state,
    }
}

async fn try_resolve_pdf(raw_pdf_url: &str, requested_id: &str) -> FulltextStepOutcome<String> {
    let Some(url) = parse_pdf_url(raw_pdf_url) else {
        return FulltextStepOutcome::Failed(BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message:
                "Semantic Scholar PDF source unavailable: outbound policy rejected invalid URL"
                    .to_string(),
        });
    };
    let policy =
        match crate::sources::provider_url_policy::ProviderUrlPolicy::semantic_scholar_pdf() {
            Ok(policy) => policy,
            Err(err) => return FulltextStepOutcome::Failed(err),
        };
    if let Err(err) = policy.validate_url(&url) {
        return FulltextStepOutcome::Failed(err);
    }
    let client = match crate::sources::provider_url_client(&policy) {
        Ok(client) => client,
        Err(err) => return FulltextStepOutcome::Failed(err),
    };
    let request = crate::sources::apply_no_store(client.get(url.clone()));
    let response = match crate::sources::with_response_body_limit(
        request,
        PDF_MAX_BODY_BYTES,
        ARTICLE_FULLTEXT_API,
    )
    .send_with_source_context(crate::error::SourceContext::retry(
        crate::error::SourceProvider::SEMANTIC_SCHOLAR,
    ))
    .await
    {
        Ok(response) => response,
        Err(err) => return FulltextStepOutcome::Failed(err),
    };
    if documented_fulltext_absence(response.status()) {
        return FulltextStepOutcome::Empty;
    }
    if !response.status().is_success() {
        return FulltextStepOutcome::Failed(
            BioMcpError::Api {
                api: ARTICLE_FULLTEXT_API.to_string(),
                message: format!("Semantic Scholar PDF returned HTTP {}", response.status()),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEMANTIC_SCHOLAR,
            )),
        );
    }

    let content_type = response.headers().get(CONTENT_TYPE).cloned();
    let bytes = match crate::sources::read_limited_source_body_with_limit(
        response,
        crate::error::SourceContext::narrow(crate::error::SourceProvider::SEMANTIC_SCHOLAR),
        PDF_MAX_BODY_BYTES,
    )
    .await
    {
        Ok(bytes) => bytes,
        Err(err) => {
            debug!(?err, requested_id, "Semantic Scholar PDF body read failed");
            return FulltextStepOutcome::Failed(err);
        }
    };
    if !pdf_content_type_is_supported(content_type.as_ref()) && !pdf_body_signature_matches(&bytes)
    {
        return FulltextStepOutcome::Unusable(
            BioMcpError::Api {
                api: ARTICLE_FULLTEXT_API.to_string(),
                message: "Semantic Scholar PDF returned unsupported content".to_string(),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::SEMANTIC_SCHOLAR,
            )),
        );
    }

    let markdown = match render_fulltext_pdf(bytes, PDF_PAGE_LIMIT).await {
        Ok(markdown) => markdown,
        Err(err) => {
            debug!(?err, requested_id, "Semantic Scholar PDF conversion failed");
            return FulltextStepOutcome::Unusable(err);
        }
    };
    if markdown.trim().is_empty() {
        return FulltextStepOutcome::Unusable(BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: "Semantic Scholar PDF conversion returned empty text".to_string(),
        });
    }

    FulltextStepOutcome::Data(markdown)
}

async fn save_resolved_fulltext(
    article: &mut Article,
    requested_id: &str,
    kind: ArticleFulltextKind,
    text: String,
    source: ArticleFulltextSource,
    manifest: ArticleFulltextManifest,
) -> Result<(), BioMcpError> {
    let path = download::save_atomic(
        &fulltext_cache_key(kind, first_cache_identifier(article, requested_id)),
        &text,
    )
    .await?;
    let outcome_source = source.source.clone();
    article.full_text_path = Some(path);
    article.full_text_note = None;
    article.full_text_source = Some(source);
    article.full_text_manifest = Some(manifest);
    article
        .section_outcomes
        .complete("fulltext", SectionOutcome::data(outcome_source));
    Ok(())
}

fn merge_source_abstract(article: &mut Article, source_abstract: Option<String>) {
    if article
        .abstract_text
        .as_deref()
        .is_none_or(|abstract_text| abstract_text.trim().is_empty())
        && let Some(source_abstract) = source_abstract
    {
        article.abstract_text = Some(source_abstract);
    }
}

fn unavailable_fulltext_note() -> String {
    "Full text unavailable: one or more consulted sources could not be retrieved.".to_string()
}

fn partial_fulltext_note(
    coverage: ArticleFulltextCoverageKind,
    source_failure: bool,
) -> Option<String> {
    match (coverage, source_failure) {
        (ArticleFulltextCoverageKind::AbstractOnly, false) => {
            Some("Abstract found; article body not available.".to_string())
        }
        (ArticleFulltextCoverageKind::MetadataOnly, false) => {
            Some("Article metadata found; article body not available.".to_string())
        }
        (ArticleFulltextCoverageKind::AbstractOnly, true) => {
            Some("Abstract found, but complete article-body retrieval was unavailable.".to_string())
        }
        (ArticleFulltextCoverageKind::MetadataOnly, true) => Some(
            "Article metadata found, but complete article-body retrieval was unavailable."
                .to_string(),
        ),
        _ => None,
    }
}

fn empty_fulltext_note(sources: &[String]) -> String {
    if sources.iter().any(|source| source == "Semantic Scholar") {
        return "Full text not available: XML, HTML, and PDF sources did not return full text"
            .to_string();
    }
    if sources.iter().any(|source| source == "PMC") {
        return "Full text not available: XML and HTML sources did not return full text"
            .to_string();
    }
    "Full text not available: Article not in PubMed Central".to_string()
}

fn final_fulltext_outcome(state: &FulltextAttemptState) -> SectionOutcome {
    if state.failure.is_some() {
        SectionOutcome::unavailable(
            "Full text is unavailable because one or more consulted sources failed.",
        )
    } else {
        SectionOutcome::empty_sources(state.healthy_sources.clone())
    }
}

pub(super) async fn resolve_fulltext(
    article: &mut Article,
    requested_id: &str,
    pdf_discovery: PdfDiscoveryAttempt,
) -> Result<(), BioMcpError> {
    let mut state = FulltextAttemptState::default();
    let europe = match EuropePmcClient::new() {
        Ok(client) => Some(client),
        Err(err) => {
            state.record_failure(err);
            None
        }
    };
    let mut resolved_pmcid = article.pmcid.clone();
    let mut identity_bridge_was_healthy = false;

    article.full_text_path = None;
    article.full_text_note = None;
    article.full_text_source = None;
    article.full_text_manifest = None;
    article.full_text_coverage = None;

    if resolved_pmcid.is_none() {
        match NcbiIdConverterClient::new() {
            Ok(ncbi) => {
                let result = if let Some(pmid) = article.pmid.as_deref() {
                    ncbi.pmid_to_pmcid(pmid).await
                } else if let Some(doi) = article.doi.as_deref() {
                    ncbi.doi_to_pmcid(doi).await
                } else {
                    Ok(None)
                };
                match result {
                    Ok(value) => {
                        identity_bridge_was_healthy = true;
                        resolved_pmcid = value;
                    }
                    Err(err) => state.record_failure(err),
                }
            }
            Err(err) => state.record_failure(err),
        }
    }

    if article.pmcid.is_none() {
        article.pmcid = resolved_pmcid.clone();
    }

    for attempt in xml_fulltext_attempts(article, resolved_pmcid.as_deref()) {
        let source = xml_source_metadata(attempt.winner());
        let fetched = match attempt {
            XmlFulltextAttempt::EuropePmcPmc => match europe.as_ref() {
                Some(client) => client
                    .get_full_text_xml("PMC", resolved_pmcid.as_deref().expect("PMC attempt"))
                    .await
                    .map(|value| value.map(|xml| (xml, None))),
                None => {
                    state.record_attempt(
                        &source,
                        ArticleFulltextAttemptSourceKind::JatsXml,
                        ArticleFulltextAttemptCoverage::Unavailable,
                        ArticleFulltextAttemptOutcome::Unavailable,
                        ArticleFulltextCacheState::Bypass,
                        ArticleFulltextAttemptReason::SourceUnavailable,
                    );
                    continue;
                }
            },
            XmlFulltextAttempt::NcbiEfetchPmc => match NcbiEfetchClient::new() {
                Ok(client) => client
                    .get_full_text_xml(resolved_pmcid.as_deref().expect("PMC attempt"))
                    .await
                    .map(|value| value.map(|xml| (xml, None))),
                Err(err) => Err(err),
            },
            XmlFulltextAttempt::PmcOaArchive => match PmcOaClient::new() {
                Ok(client) => client
                    .get_full_text_xml_with_manifest(
                        resolved_pmcid.as_deref().expect("PMC attempt"),
                    )
                    .await
                    .map(|value| value.map(|(xml, manifest)| (xml, Some(manifest)))),
                Err(err) => Err(err),
            },
            XmlFulltextAttempt::EuropePmcMed => match europe.as_ref() {
                Some(client) => client
                    .get_full_text_xml("MED", article.pmid.as_deref().expect("MED attempt"))
                    .await
                    .map(|value| value.map(|xml| (xml, None))),
                None => {
                    state.record_attempt(
                        &source,
                        ArticleFulltextAttemptSourceKind::JatsXml,
                        ArticleFulltextAttemptCoverage::Unavailable,
                        ArticleFulltextAttemptOutcome::Unavailable,
                        ArticleFulltextCacheState::Bypass,
                        ArticleFulltextAttemptReason::SourceUnavailable,
                    );
                    continue;
                }
            },
        };

        let Some((xml, oa_manifest)) = (match fetched {
            Ok(value) => value,
            Err(err) => {
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::JatsXml,
                    ArticleFulltextAttemptCoverage::Unavailable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::SourceUnavailable,
                );
                state.record_failure(err);
                continue;
            }
        }) else {
            state.record_attempt(
                &source,
                ArticleFulltextAttemptSourceKind::JatsXml,
                ArticleFulltextAttemptCoverage::None,
                ArticleFulltextAttemptOutcome::Empty,
                ArticleFulltextCacheState::Bypass,
                ArticleFulltextAttemptReason::NoContent,
            );
            state.record_empty(&source.source);
            continue;
        };
        let classified = match classify_fulltext_xml(xml).await {
            Ok(classified) => classified,
            Err(err) => {
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::JatsXml,
                    ArticleFulltextAttemptCoverage::Unusable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::UnusableContent,
                );
                state.record_failure(err);
                continue;
            }
        };
        merge_source_abstract(article, classified.abstract_text.clone());
        match classified.coverage {
            ArticleDocumentCoverage::AbstractOnly | ArticleDocumentCoverage::MetadataOnly => {
                state.observe_partial(classified.coverage);
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::JatsXml,
                    if classified.coverage == ArticleDocumentCoverage::AbstractOnly {
                        ArticleFulltextAttemptCoverage::AbstractOnly
                    } else {
                        ArticleFulltextAttemptCoverage::MetadataOnly
                    },
                    ArticleFulltextAttemptOutcome::Empty,
                    ArticleFulltextCacheState::Bypass,
                    if classified.coverage == ArticleDocumentCoverage::AbstractOnly {
                        ArticleFulltextAttemptReason::AbstractWithoutBody
                    } else {
                        ArticleFulltextAttemptReason::MetadataWithoutBody
                    },
                );
                state.record_empty(&source.source);
                continue;
            }
            ArticleDocumentCoverage::FullText => {}
        }
        state.record_attempt(
            &source,
            ArticleFulltextAttemptSourceKind::JatsXml,
            ArticleFulltextAttemptCoverage::FullText,
            ArticleFulltextAttemptOutcome::Data,
            ArticleFulltextCacheState::Bypass,
            ArticleFulltextAttemptReason::BodyDetected,
        );
        let source_identifier = match attempt.winner() {
            XmlWaterfallWinner::EuropePmcMed => article.pmid.as_deref(),
            _ => resolved_pmcid.as_deref(),
        }
        .and_then(clean_manifest_string)
        .unwrap_or_else(|| requested_id.trim().to_string());
        let package_url = oa_manifest
            .as_ref()
            .map(|manifest| manifest.package_url.clone());
        let oa_retracted = oa_manifest.as_ref().and_then(|manifest| manifest.retracted);
        let license = article.europepmc_license.clone().or_else(|| {
            oa_manifest
                .as_ref()
                .and_then(|manifest| manifest.license.clone())
        });
        let manifest = ArticleFulltextManifest {
            source_kind: ArticleFulltextManifestKind::JatsXml,
            provider: manifest_provider(&source),
            source_identifier,
            quality: classified.quality,
            reuse: manifest_reuse(license),
            provenance: manifest_provenance(article, false, package_url, oa_retracted),
        };
        article.full_text_coverage = Some(ArticleFulltextCoverage {
            coverage: ArticleFulltextCoverageKind::FullText,
            attempts: std::mem::take(&mut state.attempts),
        });
        return save_resolved_fulltext(
            article,
            requested_id,
            ArticleFulltextKind::JatsXml,
            classified
                .markdown
                .expect("full-text JATS has rendered body"),
            source,
            manifest,
        )
        .await;
    }

    if let Some(pmcid) = resolved_pmcid.as_deref() {
        let source = html_source_metadata();
        let resolution = try_resolve_html(pmcid, requested_id).await;
        match resolution.outcome {
            FulltextStepOutcome::Data(classified) => {
                merge_source_abstract(article, classified.abstract_text.clone());
                match classified.coverage {
                    ArticleDocumentCoverage::FullText => {
                        state.record_attempt(
                            &source,
                            ArticleFulltextAttemptSourceKind::PmcHtml,
                            ArticleFulltextAttemptCoverage::FullText,
                            ArticleFulltextAttemptOutcome::Data,
                            resolution.cache_state,
                            ArticleFulltextAttemptReason::BodyDetected,
                        );
                        let manifest = ArticleFulltextManifest {
                            source_kind: ArticleFulltextManifestKind::PmcHtml,
                            provider: manifest_provider(&source),
                            source_identifier: clean_manifest_string(pmcid)
                                .unwrap_or_else(|| requested_id.trim().to_string()),
                            quality: classified.quality,
                            reuse: manifest_reuse(article.europepmc_license.clone()),
                            provenance: manifest_provenance(article, false, None, None),
                        };
                        article.full_text_coverage = Some(ArticleFulltextCoverage {
                            coverage: ArticleFulltextCoverageKind::FullText,
                            attempts: std::mem::take(&mut state.attempts),
                        });
                        return save_resolved_fulltext(
                            article,
                            requested_id,
                            ArticleFulltextKind::Html,
                            classified
                                .markdown
                                .expect("full-text HTML has rendered body"),
                            source,
                            manifest,
                        )
                        .await;
                    }
                    ArticleDocumentCoverage::AbstractOnly
                    | ArticleDocumentCoverage::MetadataOnly => {
                        state.observe_partial(classified.coverage);
                        state.record_attempt(
                            &source,
                            ArticleFulltextAttemptSourceKind::PmcHtml,
                            if classified.coverage == ArticleDocumentCoverage::AbstractOnly {
                                ArticleFulltextAttemptCoverage::AbstractOnly
                            } else {
                                ArticleFulltextAttemptCoverage::MetadataOnly
                            },
                            ArticleFulltextAttemptOutcome::Empty,
                            resolution.cache_state,
                            if classified.coverage == ArticleDocumentCoverage::AbstractOnly {
                                ArticleFulltextAttemptReason::AbstractWithoutBody
                            } else {
                                ArticleFulltextAttemptReason::MetadataWithoutBody
                            },
                        );
                        state.record_empty("PMC");
                    }
                }
            }
            FulltextStepOutcome::Empty => {
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::PmcHtml,
                    ArticleFulltextAttemptCoverage::None,
                    ArticleFulltextAttemptOutcome::Empty,
                    resolution.cache_state,
                    ArticleFulltextAttemptReason::NoContent,
                );
                state.record_empty("PMC");
            }
            FulltextStepOutcome::Unusable(err) => {
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::PmcHtml,
                    ArticleFulltextAttemptCoverage::Unusable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    resolution.cache_state,
                    ArticleFulltextAttemptReason::UnusableContent,
                );
                state.record_failure(err);
            }
            FulltextStepOutcome::Failed(err) => {
                state.record_attempt(
                    &source,
                    ArticleFulltextAttemptSourceKind::PmcHtml,
                    ArticleFulltextAttemptCoverage::Unavailable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    resolution.cache_state,
                    ArticleFulltextAttemptReason::SourceUnavailable,
                );
                state.record_failure(err);
            }
        }
    }

    let pdf_source = pdf_source_metadata();
    match pdf_discovery {
        PdfDiscoveryAttempt::Ineligible => {}
        PdfDiscoveryAttempt::Empty => {
            state.record_attempt(
                &pdf_source,
                ArticleFulltextAttemptSourceKind::Pdf,
                ArticleFulltextAttemptCoverage::None,
                ArticleFulltextAttemptOutcome::Empty,
                ArticleFulltextCacheState::Bypass,
                ArticleFulltextAttemptReason::NoContent,
            );
            state.record_empty("Semantic Scholar");
        }
        PdfDiscoveryAttempt::Failed(err) => {
            state.record_attempt(
                &pdf_source,
                ArticleFulltextAttemptSourceKind::Pdf,
                ArticleFulltextAttemptCoverage::Unavailable,
                ArticleFulltextAttemptOutcome::Unavailable,
                ArticleFulltextCacheState::Bypass,
                ArticleFulltextAttemptReason::SourceUnavailable,
            );
            state.record_failure(err);
        }
        PdfDiscoveryAttempt::Data(pdf_url) => match try_resolve_pdf(&pdf_url, requested_id).await {
            FulltextStepOutcome::Data(text) => {
                state.record_attempt(
                    &pdf_source,
                    ArticleFulltextAttemptSourceKind::Pdf,
                    ArticleFulltextAttemptCoverage::FullText,
                    ArticleFulltextAttemptOutcome::Data,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::BodyDetected,
                );
                let mut source_identifier =
                    parse_pdf_url(&pdf_url).expect("successful PDF resolution validates the URL");
                source_identifier.set_query(None);
                source_identifier.set_fragment(None);
                let license = article
                    .semantic_scholar
                    .as_ref()
                    .and_then(|value| value.open_access_pdf.as_ref())
                    .and_then(|value| value.license.as_deref())
                    .and_then(clean_manifest_string);
                let manifest = ArticleFulltextManifest {
                    source_kind: ArticleFulltextManifestKind::Pdf,
                    provider: manifest_provider(&pdf_source),
                    source_identifier: source_identifier.to_string(),
                    quality: manifest_quality(true),
                    reuse: manifest_reuse(license),
                    provenance: manifest_provenance(article, true, None, None),
                };
                article.full_text_coverage = Some(ArticleFulltextCoverage {
                    coverage: ArticleFulltextCoverageKind::FullText,
                    attempts: std::mem::take(&mut state.attempts),
                });
                return save_resolved_fulltext(
                    article,
                    requested_id,
                    ArticleFulltextKind::Pdf,
                    text,
                    pdf_source,
                    manifest,
                )
                .await;
            }
            FulltextStepOutcome::Empty => {
                state.record_attempt(
                    &pdf_source,
                    ArticleFulltextAttemptSourceKind::Pdf,
                    ArticleFulltextAttemptCoverage::None,
                    ArticleFulltextAttemptOutcome::Empty,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::NoContent,
                );
                state.record_empty("Semantic Scholar");
            }
            FulltextStepOutcome::Unusable(err) => {
                state.record_attempt(
                    &pdf_source,
                    ArticleFulltextAttemptSourceKind::Pdf,
                    ArticleFulltextAttemptCoverage::Unusable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::UnusableContent,
                );
                state.record_failure(err);
            }
            FulltextStepOutcome::Failed(err) => {
                state.record_attempt(
                    &pdf_source,
                    ArticleFulltextAttemptSourceKind::Pdf,
                    ArticleFulltextAttemptCoverage::Unavailable,
                    ArticleFulltextAttemptOutcome::Unavailable,
                    ArticleFulltextCacheState::Bypass,
                    ArticleFulltextAttemptReason::SourceUnavailable,
                );
                state.record_failure(err);
            }
        },
    }

    if state.failure.is_none() && state.healthy_sources.is_empty() && identity_bridge_was_healthy {
        state.record_empty("NCBI ID Converter");
    }
    let final_coverage = state.final_coverage();
    article.full_text_note = partial_fulltext_note(final_coverage, state.failure.is_some())
        .or_else(|| {
            if state.failure.is_some() {
                Some(unavailable_fulltext_note())
            } else {
                Some(empty_fulltext_note(&state.healthy_sources))
            }
        });
    article
        .section_outcomes
        .complete("fulltext", final_fulltext_outcome(&state));
    article.full_text_coverage = Some(ArticleFulltextCoverage {
        coverage: final_coverage,
        attempts: state.attempts,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::article::test_support::{
        TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
    };
    use crate::entities::article::{ArticleSemanticScholar, ArticleSemanticScholarPdf};
    use crate::test_support::TempDirGuard;

    fn configure_attempt_env(env: &mut TestEnv, fixture: &TestHttpFixture) {
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set(
            crate::sources::pmc_article::PMC_ARTICLE_BASE_ENV,
            &fixture.base,
        );
    }

    fn failed<T>(outcome: FulltextStepOutcome<T>) -> BioMcpError {
        match outcome {
            FulltextStepOutcome::Failed(err) | FulltextStepOutcome::Unusable(err) => err,
            FulltextStepOutcome::Data(_) | FulltextStepOutcome::Empty => {
                panic!("attempt should be classified as failure")
            }
        }
    }

    fn failed_html(resolution: HtmlResolution) -> BioMcpError {
        failed(resolution.outcome)
    }

    fn article_for_fulltext() -> Article {
        Article {
            section_outcomes: crate::entities::section_outcome::SectionOutcomes::with_keys(
                crate::entities::article::ARTICLE_OUTCOME_KEYS,
            ),
            pmid: Some("22663011".into()),
            pmcid: Some("PMC123456".into()),
            doi: Some("10.1000/example".into()),
            title: "title".into(),
            authors: Vec::new(),
            author_count: 0,
            author_completeness: crate::entities::article::ArticleAuthorCompleteness::Unavailable,
            author_source: crate::entities::article::ArticleSource::PubTator,
            journal: None,
            date: None,
            citation_count: None,
            publication_type: None,
            open_access: None,
            abstract_text: None,
            full_text_path: None,
            full_text_note: None,
            full_text_source: None,
            full_text_manifest: None,
            full_text_coverage: None,
            not_included: None,
            europepmc_license: None,
            europepmc_retracted: None,
            annotations: None,
            indexing: None,
            semantic_scholar: None,
            pubtator_fallback: false,
        }
    }

    #[test]
    fn xml_source_metadata_is_truthful() {
        let cases = [
            (
                XmlWaterfallWinner::EuropePmcPmc,
                ArticleFulltextSource {
                    kind: ArticleFulltextKind::JatsXml,
                    label: "Europe PMC XML".to_string(),
                    source: "Europe PMC".to_string(),
                },
            ),
            (
                XmlWaterfallWinner::NcbiEfetchPmc,
                ArticleFulltextSource {
                    kind: ArticleFulltextKind::JatsXml,
                    label: "NCBI EFetch PMC XML".to_string(),
                    source: "NCBI EFetch".to_string(),
                },
            ),
            (
                XmlWaterfallWinner::PmcOaArchive,
                ArticleFulltextSource {
                    kind: ArticleFulltextKind::JatsXml,
                    label: "PMC OA Archive XML".to_string(),
                    source: "PMC OA".to_string(),
                },
            ),
            (
                XmlWaterfallWinner::EuropePmcMed,
                ArticleFulltextSource {
                    kind: ArticleFulltextKind::JatsXml,
                    label: "Europe PMC MED XML".to_string(),
                    source: "Europe PMC".to_string(),
                },
            ),
        ];

        for (winner, expected) in cases {
            assert_eq!(xml_source_metadata(winner), expected);
        }
    }

    #[test]
    fn xml_fulltext_attempts_try_pmc_sources_before_med_fallback() {
        let article = article_for_fulltext();

        assert_eq!(
            xml_fulltext_attempts(&article, article.pmcid.as_deref()),
            vec![
                XmlFulltextAttempt::EuropePmcPmc,
                XmlFulltextAttempt::NcbiEfetchPmc,
                XmlFulltextAttempt::PmcOaArchive,
                XmlFulltextAttempt::EuropePmcMed,
            ]
        );
    }

    #[test]
    fn xml_fulltext_attempts_use_med_when_only_pmid_is_available() {
        let mut article = article_for_fulltext();
        article.pmcid = None;

        assert_eq!(
            xml_fulltext_attempts(&article, None),
            vec![XmlFulltextAttempt::EuropePmcMed]
        );
    }

    #[test]
    fn pdf_discovery_preserves_ineligible_empty_data_and_failure() {
        let mut article = article_for_fulltext();
        assert!(matches!(
            pdf_discovery_attempt(
                &article,
                false,
                Err(BioMcpError::Api {
                    api: "test".into(),
                    message: "failed".into(),
                })
            ),
            PdfDiscoveryAttempt::Ineligible
        ));
        assert!(matches!(
            pdf_discovery_attempt(&article, true, Ok(())),
            PdfDiscoveryAttempt::Empty
        ));

        article.semantic_scholar = Some(ArticleSemanticScholar {
            paper_id: Some("paper-1".into()),
            tldr: None,
            citation_count: None,
            influential_citation_count: None,
            reference_count: None,
            is_open_access: Some(true),
            open_access_pdf: Some(ArticleSemanticScholarPdf {
                url: "  https://example.test/open.pdf  ".into(),
                status: Some("GREEN".into()),
                license: Some("CC BY".into()),
            }),
        });
        let PdfDiscoveryAttempt::Data(url) = pdf_discovery_attempt(&article, true, Ok(())) else {
            panic!("supported PDF should be retained");
        };
        assert_eq!(url, "https://example.test/open.pdf");
        assert!(matches!(
            pdf_discovery_attempt(
                &article,
                true,
                Err(BioMcpError::Api {
                    api: "test".into(),
                    message: "failed".into(),
                })
            ),
            PdfDiscoveryAttempt::Failed(_)
        ));
    }

    #[tokio::test]
    async fn semantic_scholar_pdf_policy_rejects_unsafe_urls_without_echoing_them() {
        for raw in [
            "http://127.0.0.1:9/private-token.pdf",
            "https://user:secret@pdfs.semanticscholar.org/paper.pdf",
            "https://pdfs.semanticscholar.org:444/paper.pdf",
            "https://example.test/private-token.pdf",
        ] {
            let FulltextStepOutcome::Failed(error) = try_resolve_pdf(raw, "PMID:1").await else {
                panic!("unsafe PDF URL should fail before contact: {raw}");
            };
            let message = error.to_string();
            assert!(message.contains("Semantic Scholar"));
            assert!(message.to_ascii_lowercase().contains("retry"));
            assert!(!message.contains(raw));
            assert!(!message.contains("secret"));
            assert!(!message.contains("private-token"));
        }
    }

    #[test]
    fn only_documented_absence_statuses_are_healthy_empty() {
        assert!(documented_fulltext_absence(reqwest::StatusCode::NOT_FOUND));
        assert!(documented_fulltext_absence(reqwest::StatusCode::NO_CONTENT));
        for status in [
            reqwest::StatusCode::UNAUTHORIZED,
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            assert!(!documented_fulltext_absence(status));
        }
    }

    #[tokio::test]
    async fn xml_conversion_rejects_malformed_unsupported_and_accepts_usable_text() {
        assert!(
            classify_fulltext_xml("<article>".to_string())
                .await
                .is_err()
        );
        assert!(
            classify_fulltext_xml("<metadata><title>not JATS</title></metadata>".to_string())
                .await
                .is_err()
        );
        let classified =
            classify_fulltext_xml("<article><body><p>usable text</p></body></article>".to_string())
                .await
                .expect("valid article XML");
        assert_eq!(classified.coverage, ArticleDocumentCoverage::FullText);
    }

    #[test]
    fn pdf_detection_rejects_non_pdf_payloads() {
        let text_plain = reqwest::header::HeaderValue::from_static("text/plain");

        assert!(!pdf_content_type_is_supported(Some(&text_plain)));
        assert!(!pdf_body_signature_matches(b"not a pdf"));
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn html_and_pdf_attempts_classify_transport_and_conversion_failures() {
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-fulltext-attempt-matrix");
        env.set("BIOMCP_CACHE_DIR", cache.path());

        let not_found = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "404 Not Found",
                "text/plain",
                b"not found",
            ))
        })
        .await;
        configure_attempt_env(&mut env, &not_found);
        assert!(matches!(
            try_resolve_html("PMC1", "1").await.outcome,
            FulltextStepOutcome::Empty
        ));
        assert!(matches!(
            try_resolve_pdf(&format!("{}/missing.pdf", not_found.base), "1").await,
            FulltextStepOutcome::Empty
        ));
        drop(not_found);

        let status_failure = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "500 Internal Server Error",
                "text/plain",
                b"failed",
            ))
        })
        .await;
        configure_attempt_env(&mut env, &status_failure);
        failed_html(try_resolve_html("PMC1", "1").await);
        failed(try_resolve_pdf(&format!("{}/failed.pdf", status_failure.base), "1").await);
        drop(status_failure);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind refused connection fixture");
        let refused = format!("http://{}", listener.local_addr().expect("fixture address"));
        drop(listener);
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &refused);
        env.set(crate::sources::pmc_article::PMC_ARTICLE_BASE_ENV, &refused);
        failed_html(try_resolve_html("PMC1", "1").await);
        failed(try_resolve_pdf(&format!("{refused}/missing.pdf"), "1").await);

        let oversized = TestHttpFixture::spawn(|request| {
            TestHttpReply::Bytes(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    if request.contains("oversized.pdf") {
                        "application/pdf"
                    } else {
                        "text/html"
                    },
                    if request.contains("oversized.pdf") {
                        PDF_MAX_BODY_BYTES + 1
                    } else {
                        crate::sources::DEFAULT_MAX_BODY_BYTES + 1
                    }
                )
                .into_bytes(),
            )
        })
        .await;
        configure_attempt_env(&mut env, &oversized);
        let html_error = failed_html(try_resolve_html("PMC1", "1").await);
        assert_eq!(html_error.code(), "api");
        assert!(format!("{html_error:?}").contains("BodyLimit"));
        let pdf_error =
            failed(try_resolve_pdf(&format!("{}/oversized.pdf", oversized.base), "1").await);
        assert_eq!(pdf_error.code(), "api");
        assert!(format!("{pdf_error:?}").contains("BodyLimit"));
        drop(oversized);

        let unsupported = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response("200 OK", "text/plain", b"plain text"))
        })
        .await;
        configure_attempt_env(&mut env, &unsupported);
        failed_html(try_resolve_html("PMC1", "1").await);
        failed(try_resolve_pdf(&format!("{}/unsupported.pdf", unsupported.base), "1").await);
        drop(unsupported);

        let invalid_utf8 = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response("200 OK", "text/html", &[0xff, 0xfe]))
        })
        .await;
        configure_attempt_env(&mut env, &invalid_utf8);
        failed_html(try_resolve_html("PMC1", "1").await);
        drop(invalid_utf8);

        let empty_conversion = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "text/html",
                b"<html><body></body></html>",
            ))
        })
        .await;
        configure_attempt_env(&mut env, &empty_conversion);
        failed_html(try_resolve_html("PMC1", "1").await);
        drop(empty_conversion);

        let invalid_pdf = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/pdf",
                b"%PDF-invalid",
            ))
        })
        .await;
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &invalid_pdf.base);
        failed(try_resolve_pdf(&format!("{}/invalid.pdf", invalid_pdf.base), "1").await);
        drop(invalid_pdf);
    }

    #[test]
    fn abstract_merge_preserves_nonblank_base_and_fills_missing_or_blank_values() {
        let mut article = article_for_fulltext();
        merge_source_abstract(&mut article, Some("source abstract".into()));
        assert_eq!(article.abstract_text.as_deref(), Some("source abstract"));

        article.abstract_text = Some("   ".into());
        merge_source_abstract(&mut article, Some("replacement abstract".into()));
        assert_eq!(
            article.abstract_text.as_deref(),
            Some("replacement abstract")
        );

        article.abstract_text = Some("base abstract".into());
        merge_source_abstract(&mut article, Some("different source abstract".into()));
        assert_eq!(article.abstract_text.as_deref(), Some("base abstract"));
    }

    #[test]
    fn partial_coverage_and_source_health_fold_independently_in_any_order() {
        for partial_first in [true, false] {
            let mut state = FulltextAttemptState::default();
            if partial_first {
                state.observe_partial(ArticleDocumentCoverage::MetadataOnly);
            }
            state.record_failure(BioMcpError::Api {
                api: "test".into(),
                message: "failed".into(),
            });
            if !partial_first {
                state.observe_partial(ArticleDocumentCoverage::MetadataOnly);
            }
            assert_eq!(
                state.final_coverage(),
                ArticleFulltextCoverageKind::MetadataOnly
            );
            assert_eq!(
                final_fulltext_outcome(&state).outcome(),
                crate::entities::section_outcome::SectionOutcomeState::Unavailable
            );
        }

        let mut healthy = FulltextAttemptState::default();
        healthy.record_empty("PMC");
        healthy.observe_partial(ArticleDocumentCoverage::MetadataOnly);
        healthy.observe_partial(ArticleDocumentCoverage::AbstractOnly);
        assert_eq!(
            healthy.final_coverage(),
            ArticleFulltextCoverageKind::AbstractOnly
        );
        assert_eq!(
            final_fulltext_outcome(&healthy).outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Empty
        );
    }

    #[test]
    fn typed_timeout_failure_survives_later_healthy_miss_in_final_fold() {
        let mut state = FulltextAttemptState::default();
        state.record_failure(BioMcpError::HttpMiddleware(
            reqwest_middleware::Error::Middleware(anyhow::anyhow!("operation timed out")),
        ));
        state.record_empty("PMC");

        assert_eq!(
            final_fulltext_outcome(&state).outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn resolver_continues_after_bad_xml_and_later_data_overrides_failures() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls_for_server = calls.clone();
        let fixture = TestHttpFixture::spawn(move |_| {
            let call = calls_for_server.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let body = if call == 0 {
                b"<article>".as_slice()
            } else {
                b"<article><body><p>NCBI later winner</p></body></article>".as_slice()
            };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/xml", body))
        })
        .await;
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-fulltext-xml-continuation");
        for key in [
            "BIOMCP_TEST_UNPACED_ORIGIN",
            "BIOMCP_EUROPEPMC_BASE",
            "BIOMCP_PUBMED_BASE",
            "BIOMCP_PMC_OA_BASE",
            crate::sources::pmc_article::PMC_ARTICLE_BASE_ENV,
        ] {
            env.set(key, &fixture.base);
        }
        env.set("BIOMCP_CACHE_DIR", cache.path());

        let mut article = article_for_fulltext();
        resolve_fulltext(&mut article, "22663011", PdfDiscoveryAttempt::Ineligible)
            .await
            .expect("later XML winner");

        assert_eq!(
            article
                .section_outcomes
                .get("fulltext")
                .expect("fulltext outcome")
                .outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Data
        );
        assert_eq!(
            article
                .full_text_source
                .as_ref()
                .map(|source| source.source.as_str()),
            Some("NCBI EFetch")
        );
        assert!(calls.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn abstract_only_xml_continues_to_a_later_body_winner() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let calls_for_server = calls.clone();
        let fixture = TestHttpFixture::spawn(move |_| {
            let call = calls_for_server.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let body = if call == 0 {
                b"<article><front><abstract><p>source abstract</p></abstract></front></article>"
                    .as_slice()
            } else {
                b"<article><body><p>later body winner</p></body></article>".as_slice()
            };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/xml", body))
        })
        .await;
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-fulltext-partial-continuation");
        for key in [
            "BIOMCP_TEST_UNPACED_ORIGIN",
            "BIOMCP_EUROPEPMC_BASE",
            "BIOMCP_PUBMED_BASE",
            "BIOMCP_PMC_OA_BASE",
            crate::sources::pmc_article::PMC_ARTICLE_BASE_ENV,
        ] {
            env.set(key, &fixture.base);
        }
        env.set("BIOMCP_CACHE_DIR", cache.path());

        let mut article = article_for_fulltext();
        resolve_fulltext(&mut article, "22663011", PdfDiscoveryAttempt::Ineligible)
            .await
            .expect("later body winner");

        assert_eq!(article.abstract_text.as_deref(), Some("source abstract"));
        let coverage = article.full_text_coverage.as_ref().expect("coverage");
        assert_eq!(coverage.coverage, ArticleFulltextCoverageKind::FullText);
        assert_eq!(
            coverage.attempts[0].coverage,
            ArticleFulltextAttemptCoverage::AbstractOnly
        );
        assert_eq!(
            coverage.attempts[1].coverage,
            ArticleFulltextAttemptCoverage::FullText
        );
        assert!(
            article
                .full_text_manifest
                .as_ref()
                .expect("winner manifest")
                .quality
                .has_fulltext_signal
        );
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn stale_v3_partial_artifact_cannot_become_the_current_winner() {
        let fixture = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/xml",
                b"<article><body><p>current body winner</p></body></article>",
            ))
        })
        .await;
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-fulltext-v3-regression");
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
        env.set("BIOMCP_CACHE_DIR", cache.path());
        let stale_path = download::save_atomic(
            "article-fulltext-v3:jats_xml:22663011",
            "abstract without article body",
        )
        .await
        .expect("plant stale v3 artifact");

        let mut article = article_for_fulltext();
        resolve_fulltext(&mut article, "22663011", PdfDiscoveryAttempt::Ineligible)
            .await
            .expect("current body winner");
        let current_path = article.full_text_path.as_ref().expect("current path");

        assert_ne!(current_path, &stale_path);
        assert!(
            tokio::fs::read_to_string(current_path)
                .await
                .expect("current artifact")
                .contains("current body winner")
        );
    }

    #[serial_test::serial(article_resolver_env)]
    #[tokio::test]
    async fn earlier_fulltext_winner_overrides_failed_pdf_discovery() {
        let fixture = TestHttpFixture::spawn(|_| {
            TestHttpReply::Bytes(test_http_response(
                "200 OK",
                "application/xml",
                b"<article><body><p>earlier XML winner</p></body></article>",
            ))
        })
        .await;
        let mut env = TestEnv::new();
        let cache = TempDirGuard::new("article-fulltext-pdf-precedence");
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
        env.set("BIOMCP_CACHE_DIR", cache.path());

        let mut article = article_for_fulltext();
        resolve_fulltext(
            &mut article,
            "22663011",
            PdfDiscoveryAttempt::Failed(BioMcpError::Api {
                api: "semantic-scholar".into(),
                message: "discovery failed".into(),
            }),
        )
        .await
        .expect("earlier winner");

        assert_eq!(
            article
                .section_outcomes
                .get("fulltext")
                .expect("fulltext outcome")
                .outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Data
        );
        assert_eq!(
            article
                .full_text_source
                .as_ref()
                .map(|source| source.source.as_str()),
            Some("Europe PMC")
        );
    }

    #[test]
    fn fulltext_cache_key_is_kind_aware_and_versioned() {
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::JatsXml, "22663011"),
            "article-fulltext-v4:jats_xml:22663011"
        );
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::Html, "10.1000/example"),
            "article-fulltext-v4:html:10.1000/example"
        );
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::Pdf, "10.1000/example"),
            "article-fulltext-v4:pdf:10.1000/example"
        );
    }

    #[test]
    fn attempt_state_orders_and_deduplicates_healthy_sources_without_erasing_failure() {
        let mut state = FulltextAttemptState::default();
        state.record_empty("Europe PMC");
        state.record_failure(BioMcpError::Api {
            api: "test".into(),
            message: "failed".into(),
        });
        state.record_empty("PMC");
        state.record_empty("Europe PMC");

        assert_eq!(state.healthy_sources, ["Europe PMC", "PMC"]);
        let outcome = final_fulltext_outcome(&state);
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
        assert!(outcome.sources().is_empty());

        let mut healthy = FulltextAttemptState::default();
        healthy.record_empty("Europe PMC");
        healthy.record_empty("PMC");
        let outcome = final_fulltext_outcome(&healthy);
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Empty
        );
        assert_eq!(outcome.sources(), ["Europe PMC", "PMC"]);
    }
}
