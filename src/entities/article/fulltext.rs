use std::borrow::Cow;

use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use tracing::warn;

use crate::error::BioMcpError;
use crate::sources::europepmc::EuropePmcClient;
use crate::sources::ncbi_efetch::NcbiEfetchClient;
use crate::sources::ncbi_idconv::NcbiIdConverterClient;
use crate::sources::pmc_oa::{PmcOaArchiveManifest, PmcOaClient};
use crate::transform;
use crate::utils::download;

use super::{
    Article, ArticleFulltextKind, ArticleFulltextManifest, ArticleFulltextManifestKind,
    ArticleFulltextProvenance, ArticleFulltextProvider, ArticleFulltextQuality,
    ArticleFulltextReuse, ArticleFulltextSource, ArticleGetOptions,
};

const FULLTEXT_CACHE_VERSION: &str = "v3";
const ARTICLE_FULLTEXT_API: &str = "article";
const PMC_ARTICLE_BASE: &str = "https://pmc.ncbi.nlm.nih.gov";
const PMC_ARTICLE_BASE_ENV: &str = "BIOMCP_PMC_HTML_BASE";
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
    Resolved(T),
    Miss,
    HardError(BioMcpError),
}

#[derive(Default)]
struct FulltextAttemptState {
    tried_xml: bool,
    tried_html: bool,
    tried_pdf: bool,
    hard_error: Option<BioMcpError>,
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

fn record_hard_error(state: &mut FulltextAttemptState, err: BioMcpError) {
    if state.hard_error.is_none() {
        state.hard_error = Some(err);
    }
}

async fn render_fulltext_xml(xml: String) -> Result<String, BioMcpError> {
    tokio::task::spawn_blocking(move || transform::article::extract_text_from_xml(&xml))
        .await
        .map_err(|err| BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: format!("Full text XML render worker failed: {err}"),
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

fn html_content_type_is_supported(content_type: Option<&reqwest::header::HeaderValue>) -> bool {
    let Some(content_type) = content_type.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default();
    media_type.eq_ignore_ascii_case("text/html")
        || media_type.eq_ignore_ascii_case("application/xhtml+xml")
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

fn body_limit_error(err: &BioMcpError, max_bytes: usize) -> bool {
    matches!(
        err,
        BioMcpError::BodyLimit {
            source_name,
            max_bytes: actual_max,
        } if source_name == ARTICLE_FULLTEXT_API && *actual_max == max_bytes
    )
}

fn pdf_body_signature_matches(body: &[u8]) -> bool {
    body.starts_with(b"%PDF-")
}

fn pmc_article_base() -> Cow<'static, str> {
    crate::sources::env_base(PMC_ARTICLE_BASE, PMC_ARTICLE_BASE_ENV)
}

fn pmc_article_url(pmcid: &str) -> Result<Url, BioMcpError> {
    let base = pmc_article_base();
    let mut url = Url::parse(base.as_ref()).map_err(|err| BioMcpError::Api {
        api: ARTICLE_FULLTEXT_API.to_string(),
        message: format!("invalid PMC HTML base URL: {err}"),
    })?;
    {
        let mut segments = url.path_segments_mut().map_err(|_| BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message: "invalid PMC HTML base URL".to_string(),
        })?;
        segments.push("articles");
        segments.push(pmcid);
        segments.push("");
    }
    Ok(url)
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

fn semantic_scholar_pdf_url(article: &Article, options: ArticleGetOptions) -> Option<&str> {
    if !options.allow_pdf {
        return None;
    }
    article
        .semantic_scholar
        .as_ref()
        .and_then(|value| value.open_access_pdf.as_ref())
        .map(|value| value.url.trim())
        .filter(|value| !value.is_empty())
}

async fn try_resolve_html(pmcid: &str, requested_id: &str) -> FulltextStepOutcome<String> {
    let url = match pmc_article_url(pmcid) {
        Ok(url) => url,
        Err(err) => return FulltextStepOutcome::HardError(err),
    };
    let client = match crate::sources::shared_client() {
        Ok(client) => client,
        Err(err) => return FulltextStepOutcome::HardError(err),
    };
    let response = match crate::sources::apply_cache_mode(client.get(url.clone()))
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return FulltextStepOutcome::HardError(err.into()),
    };
    if !response.status().is_success() {
        return FulltextStepOutcome::Miss;
    }
    if !html_content_type_is_supported(response.headers().get(CONTENT_TYPE)) {
        return FulltextStepOutcome::Miss;
    }

    let bytes = match crate::sources::read_limited_body(response, ARTICLE_FULLTEXT_API).await {
        Ok(bytes) => bytes,
        Err(err) if body_limit_error(&err, crate::sources::DEFAULT_MAX_BODY_BYTES) => {
            warn!(?err, requested_id, pmcid, "PMC HTML body exceeded limit");
            return FulltextStepOutcome::Miss;
        }
        Err(err) => return FulltextStepOutcome::HardError(err),
    };
    let html = String::from_utf8_lossy(&bytes).to_string();
    let markdown = match transform::article::extract_text_from_html(&html, url.as_str()) {
        Ok(markdown) => markdown,
        Err(err) => {
            warn!(?err, requested_id, pmcid, "PMC HTML conversion failed");
            return FulltextStepOutcome::Miss;
        }
    };
    if markdown.trim().is_empty() {
        return FulltextStepOutcome::Miss;
    }

    FulltextStepOutcome::Resolved(markdown)
}

async fn try_resolve_pdf(raw_pdf_url: &str, requested_id: &str) -> FulltextStepOutcome<String> {
    let Some(url) = parse_pdf_url(raw_pdf_url) else {
        return FulltextStepOutcome::HardError(BioMcpError::Api {
            api: ARTICLE_FULLTEXT_API.to_string(),
            message:
                "Semantic Scholar PDF source unavailable: outbound policy rejected invalid URL"
                    .to_string(),
        });
    };
    let policy =
        match crate::sources::provider_url_policy::ProviderUrlPolicy::semantic_scholar_pdf() {
            Ok(policy) => policy,
            Err(err) => return FulltextStepOutcome::HardError(err),
        };
    if let Err(err) = policy.validate_url(&url) {
        return FulltextStepOutcome::HardError(err);
    }
    let client = match crate::sources::provider_url_client(&policy) {
        Ok(client) => client,
        Err(err) => return FulltextStepOutcome::HardError(err),
    };
    let request = crate::sources::apply_no_store(client.get(url.clone()));
    let response = match crate::sources::with_response_body_limit(
        request,
        PDF_MAX_BODY_BYTES,
        ARTICLE_FULLTEXT_API,
    )
    .send()
    .await
    {
        Ok(response) => response,
        Err(_) => {
            warn!(
                requested_id,
                "Semantic Scholar PDF outbound request unavailable"
            );
            return FulltextStepOutcome::HardError(BioMcpError::Api {
                api: ARTICLE_FULLTEXT_API.to_string(),
                message:
                    "Semantic Scholar PDF source unavailable: outbound request rejected or failed"
                        .to_string(),
            });
        }
    };
    if !response.status().is_success() {
        return FulltextStepOutcome::Miss;
    }

    let content_type = response.headers().get(CONTENT_TYPE).cloned();
    let bytes = match crate::sources::read_limited_body_with_limit(
        response,
        ARTICLE_FULLTEXT_API,
        PDF_MAX_BODY_BYTES,
    )
    .await
    {
        Ok(bytes) => bytes,
        Err(err) if body_limit_error(&err, PDF_MAX_BODY_BYTES) => {
            warn!(requested_id, "Semantic Scholar PDF body exceeded limit");
            return FulltextStepOutcome::Miss;
        }
        Err(_) => {
            warn!(requested_id, "Semantic Scholar PDF body read failed");
            return FulltextStepOutcome::Miss;
        }
    };
    if !pdf_content_type_is_supported(content_type.as_ref()) && !pdf_body_signature_matches(&bytes)
    {
        return FulltextStepOutcome::Miss;
    }

    let markdown = match render_fulltext_pdf(bytes, PDF_PAGE_LIMIT).await {
        Ok(markdown) => markdown,
        Err(_) => {
            warn!(requested_id, "Semantic Scholar PDF conversion failed");
            return FulltextStepOutcome::Miss;
        }
    };
    if markdown.trim().is_empty() {
        return FulltextStepOutcome::Miss;
    }

    FulltextStepOutcome::Resolved(markdown)
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
    article.full_text_path = Some(path);
    article.full_text_note = None;
    article.full_text_source = Some(source);
    article.full_text_manifest = Some(manifest);
    Ok(())
}

fn unresolved_fulltext_note(article: &Article, state: &FulltextAttemptState) -> Option<String> {
    if article.pmcid.is_none() && state.hard_error.is_none() {
        return Some("Full text not available: Article not in PubMed Central".into());
    }
    if state.tried_xml && state.tried_html && state.tried_pdf {
        return Some(
            "Full text not available: XML, HTML, and PDF sources did not return full text".into(),
        );
    }
    if state.tried_xml && state.tried_html {
        return Some(
            "Full text not available: XML and HTML sources did not return full text".into(),
        );
    }
    if state.hard_error.is_some() {
        return Some("Full text not available: API error".into());
    }
    None
}

pub(super) async fn resolve_fulltext(
    article: &mut Article,
    requested_id: &str,
    options: ArticleGetOptions,
) -> Result<(), BioMcpError> {
    let europe = EuropePmcClient::new()?;
    let mut state = FulltextAttemptState::default();
    let mut resolved_pmcid = article.pmcid.clone();
    let mut resolved_xml: Option<(String, XmlWaterfallWinner, Option<PmcOaArchiveManifest>)> = None;

    article.full_text_path = None;
    article.full_text_note = None;
    article.full_text_source = None;
    article.full_text_manifest = None;

    if resolved_pmcid.is_none() {
        match NcbiIdConverterClient::new() {
            Ok(ncbi) => {
                if let Some(pmid) = article.pmid.as_deref() {
                    match ncbi.pmid_to_pmcid(pmid).await {
                        Ok(value) => resolved_pmcid = value,
                        Err(err) => record_hard_error(&mut state, err),
                    }
                } else if let Some(doi) = article.doi.as_deref() {
                    match ncbi.doi_to_pmcid(doi).await {
                        Ok(value) => resolved_pmcid = value,
                        Err(err) => record_hard_error(&mut state, err),
                    }
                }
            }
            Err(err) => record_hard_error(&mut state, err),
        }
    }

    if article.pmcid.is_none() {
        article.pmcid = resolved_pmcid.clone();
    }

    for attempt in xml_fulltext_attempts(article, resolved_pmcid.as_deref()) {
        state.tried_xml = true;
        match attempt {
            XmlFulltextAttempt::EuropePmcPmc => {
                let Some(pmcid) = resolved_pmcid.as_deref() else {
                    continue;
                };
                match europe.get_full_text_xml("PMC", pmcid).await {
                    Ok(Some(value)) => resolved_xml = Some((value, attempt.winner(), None)),
                    Ok(None) => {}
                    Err(err) => record_hard_error(&mut state, err),
                }
            }
            XmlFulltextAttempt::NcbiEfetchPmc => {
                let Some(pmcid) = resolved_pmcid.as_deref() else {
                    continue;
                };
                match NcbiEfetchClient::new() {
                    Ok(ncbi_efetch) => match ncbi_efetch.get_full_text_xml(pmcid).await {
                        Ok(Some(value)) => resolved_xml = Some((value, attempt.winner(), None)),
                        Ok(None) => {}
                        Err(err) => record_hard_error(&mut state, err),
                    },
                    Err(err) => record_hard_error(&mut state, err),
                }
            }
            XmlFulltextAttempt::PmcOaArchive => {
                let Some(pmcid) = resolved_pmcid.as_deref() else {
                    continue;
                };
                match PmcOaClient::new() {
                    Ok(pmc_oa) => match pmc_oa.get_full_text_xml_with_manifest(pmcid).await {
                        Ok(Some((value, manifest))) => {
                            resolved_xml = Some((value, attempt.winner(), Some(manifest)));
                        }
                        Ok(None) => {}
                        Err(err) => record_hard_error(&mut state, err),
                    },
                    Err(err) => record_hard_error(&mut state, err),
                }
            }
            XmlFulltextAttempt::EuropePmcMed => {
                let Some(pmid) = article.pmid.as_deref() else {
                    continue;
                };
                match europe.get_full_text_xml("MED", pmid).await {
                    Ok(Some(value)) => resolved_xml = Some((value, attempt.winner(), None)),
                    Ok(None) => {}
                    Err(err) => record_hard_error(&mut state, err),
                }
            }
        }
        if resolved_xml.is_some() {
            break;
        }
    }

    if let Some((xml, winner, oa_manifest)) = resolved_xml {
        let mut quality = transform::article::jats_quality_flags(&xml);
        let text = render_fulltext_xml(xml).await?;
        quality.has_fulltext_signal = !text.trim().is_empty();
        let source = xml_source_metadata(winner);
        let source_identifier = match winner {
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
            quality,
            reuse: manifest_reuse(license),
            provenance: manifest_provenance(article, false, package_url, oa_retracted),
        };
        return save_resolved_fulltext(
            article,
            requested_id,
            ArticleFulltextKind::JatsXml,
            text,
            source,
            manifest,
        )
        .await;
    }

    if let Some(pmcid) = resolved_pmcid.as_deref() {
        state.tried_html = true;
        match try_resolve_html(pmcid, requested_id).await {
            FulltextStepOutcome::Resolved(text) => {
                let source = html_source_metadata();
                let manifest = ArticleFulltextManifest {
                    source_kind: ArticleFulltextManifestKind::PmcHtml,
                    provider: manifest_provider(&source),
                    source_identifier: clean_manifest_string(pmcid)
                        .unwrap_or_else(|| requested_id.trim().to_string()),
                    quality: manifest_quality(!text.trim().is_empty()),
                    reuse: manifest_reuse(article.europepmc_license.clone()),
                    provenance: manifest_provenance(article, false, None, None),
                };
                return save_resolved_fulltext(
                    article,
                    requested_id,
                    ArticleFulltextKind::Html,
                    text,
                    source,
                    manifest,
                )
                .await;
            }
            FulltextStepOutcome::Miss => {}
            FulltextStepOutcome::HardError(err) => record_hard_error(&mut state, err),
        }
    }

    if let Some(pdf_url) = semantic_scholar_pdf_url(article, options) {
        state.tried_pdf = true;
        match try_resolve_pdf(pdf_url, requested_id).await {
            FulltextStepOutcome::Resolved(text) => {
                let source = pdf_source_metadata();
                let license = article
                    .semantic_scholar
                    .as_ref()
                    .and_then(|value| value.open_access_pdf.as_ref())
                    .and_then(|value| value.license.as_deref())
                    .and_then(clean_manifest_string);
                let manifest = ArticleFulltextManifest {
                    source_kind: ArticleFulltextManifestKind::Pdf,
                    provider: manifest_provider(&source),
                    source_identifier: pdf_url.to_string(),
                    quality: manifest_quality(!text.trim().is_empty()),
                    reuse: manifest_reuse(license),
                    provenance: manifest_provenance(article, true, None, None),
                };
                return save_resolved_fulltext(
                    article,
                    requested_id,
                    ArticleFulltextKind::Pdf,
                    text,
                    source,
                    manifest,
                )
                .await;
            }
            FulltextStepOutcome::Miss => {}
            FulltextStepOutcome::HardError(err) => record_hard_error(&mut state, err),
        }
    }

    if let Some(err) = state.hard_error.as_ref()
        && unresolved_fulltext_note(article, &state)
            .as_deref()
            .is_some_and(|note| note == "Full text not available: API error")
    {
        warn!(?err, requested_id, "Full text retrieval failed");
    }

    article.full_text_note = unresolved_fulltext_note(article, &state);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::article::{ArticleSemanticScholar, ArticleSemanticScholarPdf};

    fn article_for_fulltext() -> Article {
        Article {
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
    fn semantic_scholar_pdf_url_requires_opt_in_and_nonblank_url() {
        let mut article = article_for_fulltext();
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

        assert_eq!(
            semantic_scholar_pdf_url(
                &article,
                ArticleGetOptions {
                    allow_pdf: false,
                    ..ArticleGetOptions::default()
                }
            ),
            None
        );
        assert_eq!(
            semantic_scholar_pdf_url(
                &article,
                ArticleGetOptions {
                    allow_pdf: true,
                    ..ArticleGetOptions::default()
                }
            ),
            Some("https://example.test/open.pdf")
        );

        let Some(semantic) = article.semantic_scholar.as_mut() else {
            panic!("semantic scholar metadata");
        };
        let Some(pdf) = semantic.open_access_pdf.as_mut() else {
            panic!("open access pdf");
        };
        pdf.url = "   ".into();
        assert_eq!(
            semantic_scholar_pdf_url(
                &article,
                ArticleGetOptions {
                    allow_pdf: true,
                    ..ArticleGetOptions::default()
                }
            ),
            None
        );
    }

    #[test]
    fn typed_body_limit_error_keeps_fulltext_overflow_as_a_source_miss() {
        let error = BioMcpError::BodyLimit {
            source_name: ARTICLE_FULLTEXT_API.into(),
            max_bytes: PDF_MAX_BODY_BYTES,
        };
        assert!(body_limit_error(&error, PDF_MAX_BODY_BYTES));
    }

    #[tokio::test]
    async fn semantic_scholar_pdf_policy_rejects_unsafe_urls_without_echoing_them() {
        for raw in [
            "http://127.0.0.1:9/private-token.pdf",
            "https://user:secret@pdfs.semanticscholar.org/paper.pdf",
            "https://pdfs.semanticscholar.org:444/paper.pdf",
            "https://example.test/private-token.pdf",
        ] {
            let FulltextStepOutcome::HardError(error) = try_resolve_pdf(raw, "PMID:1").await else {
                panic!("unsafe PDF URL should fail before contact: {raw}");
            };
            let message = error.to_string();
            assert!(message.contains("outbound policy"));
            assert!(!message.contains(raw));
            assert!(!message.contains("secret"));
            assert!(!message.contains("private-token"));
        }
    }

    #[test]
    fn pdf_detection_rejects_non_pdf_payloads() {
        let text_plain = reqwest::header::HeaderValue::from_static("text/plain");

        assert!(!pdf_content_type_is_supported(Some(&text_plain)));
        assert!(!pdf_body_signature_matches(b"not a pdf"));
    }

    #[test]
    fn fulltext_cache_key_is_kind_aware_and_versioned() {
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::JatsXml, "22663011"),
            "article-fulltext-v3:jats_xml:22663011"
        );
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::Html, "10.1000/example"),
            "article-fulltext-v3:html:10.1000/example"
        );
        assert_eq!(
            fulltext_cache_key(ArticleFulltextKind::Pdf, "10.1000/example"),
            "article-fulltext-v3:pdf:10.1000/example"
        );
    }

    #[test]
    fn unresolved_note_reflects_attempted_ladder() {
        let mut article = article_for_fulltext();
        let html_only = FulltextAttemptState {
            tried_xml: true,
            tried_html: true,
            tried_pdf: false,
            hard_error: None,
        };
        assert_eq!(
            unresolved_fulltext_note(&article, &html_only).as_deref(),
            Some("Full text not available: XML and HTML sources did not return full text")
        );

        let pdf = FulltextAttemptState {
            tried_xml: true,
            tried_html: true,
            tried_pdf: true,
            hard_error: None,
        };
        assert_eq!(
            unresolved_fulltext_note(&article, &pdf).as_deref(),
            Some("Full text not available: XML, HTML, and PDF sources did not return full text")
        );

        article.pmcid = None;
        assert_eq!(
            unresolved_fulltext_note(&article, &FulltextAttemptState::default()).as_deref(),
            Some("Full text not available: Article not in PubMed Central")
        );
    }
}
