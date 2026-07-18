use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use http_cache_reqwest::CacheMode;
use reqwest::{StatusCode, Url, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::provider_url_policy::{ProviderUrlConsumer, ProviderUrlPolicy};
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const FIGSHARE_BASE: &str = "https://api.figshare.com";
const FIGSHARE_BASE_ENV: &str = "BIOMCP_FIGSHARE_BASE";
const FIGSHARE_API: &str = "figshare";
const MAX_FIGSHARE_FILE_BYTES: usize = crate::sources::DEFAULT_MAX_BODY_BYTES;
const FIGSHARE_SEARCH_PAGE_SIZE: usize = 100;
const FIGSHARE_DOWNLOAD_ACCEPTED_RETRIES: usize = 3;
const FIGSHARE_DOWNLOAD_ACCEPTED_BACKOFF_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigshareArticleRef {
    pub article_id: u64,
    pub file_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigshareArticle {
    pub article_id: u64,
    pub title: Option<String>,
    pub doi: Option<String>,
    pub api_url: Option<String>,
    pub public_url: Option<String>,
    pub license: Option<FigshareLicense>,
    pub files: Vec<FigshareFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigshareLicense {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigshareArticleSearchResult {
    pub article_id: u64,
    pub title: Option<String>,
    pub doi: Option<String>,
    pub api_url: Option<String>,
    pub public_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigshareFile {
    pub id: u64,
    pub filename: String,
    pub size: Option<usize>,
    pub md5: Option<String>,
    pub mimetype: Option<String>,
    pub download_url: String,
}

#[derive(Clone)]
pub struct FigshareClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    provider_policy: ProviderUrlPolicy,
}

#[derive(Debug, Deserialize)]
struct FigshareArticleResponse {
    id: Option<u64>,
    title: Option<String>,
    doi: Option<String>,
    url_api: Option<String>,
    url_public_html: Option<String>,
    license: Option<FigshareLicenseResponse>,
    #[serde(default)]
    files: Vec<FigshareFileResponse>,
}

#[derive(Debug, Deserialize)]
struct FigshareLicenseResponse {
    name: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FigshareFileResponse {
    id: Option<u64>,
    name: Option<String>,
    size: Option<usize>,
    md5: Option<String>,
    mimetype: Option<String>,
    download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FigshareArticleSearchResponse {
    id: Option<u64>,
    title: Option<String>,
    doi: Option<String>,
    url_api: Option<String>,
    url_public_html: Option<String>,
}

#[derive(Debug, Serialize)]
struct FigshareArticleSearchRequest<'a> {
    search_for: &'a str,
    page_size: usize,
}

#[derive(Debug)]
enum DownloadResponse {
    Retry,
    Bytes(Vec<u8>),
}

impl FigshareClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(FIGSHARE_BASE, FIGSHARE_BASE_ENV);
        let base_url = Url::parse(base.as_ref()).map_err(|_| BioMcpError::Api {
            api: FIGSHARE_API.to_string(),
            message: "Figshare source unavailable: outbound policy rejected invalid base URL"
                .into(),
        })?;
        let provider_policy = ProviderUrlPolicy::for_consumer(
            ProviderUrlConsumer::FigshareDownload,
            Some(&base_url),
        )?;
        Ok(Self {
            client: crate::sources::provider_url_client(&provider_policy)?,
            base,
            provider_policy,
        })
    }

    fn article_plan(reference: &FigshareArticleRef) -> RequestPlan {
        RequestPlan::get(format!("v2/articles/{}", reference.article_id))
    }

    fn search_articles_plan(search_for: &str) -> Option<RequestPlan> {
        let search_for = search_for.trim();
        if search_for.is_empty() {
            return None;
        }

        let body = FigshareArticleSearchRequest {
            search_for,
            page_size: FIGSHARE_SEARCH_PAGE_SIZE,
        };
        Some(RequestPlan {
            method: crate::sources::HttpMethod::Post,
            path: "v2/articles/search".to_string(),
            query: Vec::new(),
            headers: Vec::new(),
            body: RequestBody::Json(serde_json::to_value(body).expect("search request serializes")),
        })
    }

    fn decode_article_response(
        status: StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
        reference: &FigshareArticleRef,
    ) -> Result<FigshareArticle, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::summarize_http_error_body(content_type, bytes);
            return Err(BioMcpError::Api {
                api: FIGSHARE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        let raw: FigshareArticleResponse =
            serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
                api: FIGSHARE_API.to_string(),
                source,
            })?;
        Ok(normalize_article(raw, reference))
    }

    fn decode_search_response(
        status: StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: &[u8],
    ) -> Result<Vec<FigshareArticleSearchResult>, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::summarize_http_error_body(content_type, bytes);
            return Err(BioMcpError::Api {
                api: FIGSHARE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        let raw: Vec<FigshareArticleSearchResponse> =
            serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
                api: FIGSHARE_API.to_string(),
                source,
            })?;
        Ok(raw
            .into_iter()
            .filter_map(normalize_search_result)
            .collect())
    }

    fn decode_download_response(
        status: StatusCode,
        content_type: Option<&reqwest::header::HeaderValue>,
        bytes: Vec<u8>,
        accepted_retries: usize,
    ) -> Result<DownloadResponse, BioMcpError> {
        if bytes.len() > MAX_FIGSHARE_FILE_BYTES {
            return Err(BioMcpError::Api {
                api: FIGSHARE_API.to_string(),
                message: format!("Figshare file exceeded {MAX_FIGSHARE_FILE_BYTES} bytes"),
            });
        }
        if status == StatusCode::ACCEPTED {
            if accepted_retries >= FIGSHARE_DOWNLOAD_ACCEPTED_RETRIES {
                return Err(BioMcpError::Api {
                    api: FIGSHARE_API.to_string(),
                    message: format!(
                        "HTTP {status}: Figshare file still staging after {accepted_retries} retries"
                    ),
                });
            }
            return Ok(DownloadResponse::Retry);
        }
        if !status.is_success() {
            let excerpt = crate::sources::summarize_http_error_body(content_type, &bytes);
            return Err(BioMcpError::Api {
                api: FIGSHARE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        Ok(DownloadResponse::Bytes(bytes))
    }

    pub async fn article(
        &self,
        reference: &FigshareArticleRef,
    ) -> Result<FigshareArticle, BioMcpError> {
        let plan = Self::article_plan(reference);
        let resp = self
            .client_from_plan(&plan)
            .with_extension(CacheMode::NoStore)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::FIGSHARE,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::FIGSHARE),
        )
        .await?;
        Self::decode_article_response(status, content_type.as_ref(), &bytes, reference).map_err(
            |error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::FIGSHARE,
                ))
            },
        )
    }

    pub async fn search_articles(
        &self,
        search_for: &str,
    ) -> Result<Vec<FigshareArticleSearchResult>, BioMcpError> {
        let Some(plan) = Self::search_articles_plan(search_for) else {
            return Ok(Vec::new());
        };
        let resp = self
            .client_from_plan(&plan)
            .with_extension(CacheMode::NoStore)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::FIGSHARE,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::FIGSHARE),
        )
        .await?;
        Self::decode_search_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::FIGSHARE,
            ))
        })
    }

    pub async fn download_file(&self, file: &FigshareFile) -> Result<Vec<u8>, BioMcpError> {
        let download_url = self.validate_download_url(&file.download_url)?;
        let mut accepted_retries = 0;
        loop {
            let resp = self
                .client
                .get(download_url.clone())
                .with_extension(CacheMode::NoStore)
                .send_with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::FIGSHARE,
                ))
                .await?;
            let status = resp.status();
            let content_type = resp.headers().get(CONTENT_TYPE).cloned();
            let bytes = crate::sources::read_limited_source_body_with_limit(
                resp,
                crate::error::SourceContext::narrow(crate::error::SourceProvider::FIGSHARE),
                MAX_FIGSHARE_FILE_BYTES,
            )
            .await?;
            match Self::decode_download_response(
                status,
                content_type.as_ref(),
                bytes,
                accepted_retries,
            )
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::FIGSHARE,
                ))
            })? {
                DownloadResponse::Retry => {
                    accepted_retries += 1;
                    tokio::time::sleep(Duration::from_millis(
                        FIGSHARE_DOWNLOAD_ACCEPTED_BACKOFF_MS,
                    ))
                    .await;
                    continue;
                }
                DownloadResponse::Bytes(bytes) => return Ok(bytes),
            }
        }
    }

    fn client_from_plan(&self, plan: &RequestPlan) -> reqwest_middleware::RequestBuilder {
        request_from_plan(&self.client, self.base.as_ref(), plan)
    }

    fn validate_download_url(&self, raw: &str) -> Result<Url, BioMcpError> {
        let url = Url::parse(raw.trim()).map_err(|_| BioMcpError::Api {
            api: "provider-url-policy".to_string(),
            message: "Figshare download source unavailable: outbound policy rejected invalid URL"
                .into(),
        })?;
        self.provider_policy.validate_url(&url)?;
        Ok(url)
    }
}

pub fn parse_figshare_article_url(raw: &str) -> Option<FigshareArticleRef> {
    let url = Url::parse(raw.trim()).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    if host != "figshare.com" && !host.ends_with(".figshare.com") {
        return None;
    }

    let segments = url
        .path_segments()
        .map(|segments| segments.collect::<Vec<_>>())
        .unwrap_or_default();
    let file_path_index = segments.iter().position(|segment| *segment == "files");
    let article_id = if host == "api.figshare.com" {
        segments.windows(2).find_map(|window| {
            (window[0] == "articles")
                .then(|| parse_u64(window[1]))
                .flatten()
        })?
    } else if let Some(article_path_index) =
        segments.iter().position(|segment| *segment == "articles")
    {
        let end = file_path_index.unwrap_or(segments.len());
        segments[article_path_index + 1..end]
            .iter()
            .find_map(|segment| parse_u64(segment))?
    } else if let Some(index) = file_path_index {
        segments[..index]
            .iter()
            .rev()
            .find_map(|segment| parse_u64(segment))?
    } else {
        segments
            .iter()
            .rev()
            .find_map(|segment| parse_u64(segment))?
    };
    let file_id = url
        .query_pairs()
        .find_map(|(key, value)| (key == "file").then(|| parse_u64(&value)).flatten())
        .or_else(|| {
            segments.windows(2).find_map(|window| {
                (window[0] == "files")
                    .then(|| parse_u64_prefix(window[1]))
                    .flatten()
            })
        });

    Some(FigshareArticleRef {
        article_id,
        file_id,
    })
}

fn parse_u64(value: &str) -> Option<u64> {
    value.trim().parse::<u64>().ok()
}

fn parse_u64_prefix(value: &str) -> Option<u64> {
    let value = value.trim();
    let end = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    if end == 0 {
        None
    } else {
        value[..end].parse::<u64>().ok()
    }
}

fn normalize_article(
    raw: FigshareArticleResponse,
    reference: &FigshareArticleRef,
) -> FigshareArticle {
    let preferred_file_id = reference.file_id;
    let mut files = raw
        .files
        .into_iter()
        .filter_map(normalize_file)
        .collect::<Vec<_>>();
    files.sort_by(|left, right| {
        let left_preferred = Some(left.id) == preferred_file_id;
        let right_preferred = Some(right.id) == preferred_file_id;
        right_preferred
            .cmp(&left_preferred)
            .then_with(|| left.filename.cmp(&right.filename))
    });

    FigshareArticle {
        article_id: raw.id.unwrap_or(reference.article_id),
        title: clean_string(raw.title),
        doi: clean_string(raw.doi),
        api_url: clean_string(raw.url_api),
        public_url: clean_string(raw.url_public_html),
        license: raw.license.map(|license| FigshareLicense {
            name: clean_string(license.name),
            url: clean_string(license.url),
        }),
        files,
    }
}

fn normalize_search_result(
    raw: FigshareArticleSearchResponse,
) -> Option<FigshareArticleSearchResult> {
    Some(FigshareArticleSearchResult {
        article_id: raw.id?,
        title: clean_string(raw.title),
        doi: clean_string(raw.doi),
        api_url: clean_string(raw.url_api),
        public_url: clean_string(raw.url_public_html),
    })
}

fn normalize_file(raw: FigshareFileResponse) -> Option<FigshareFile> {
    let id = raw.id?;
    let filename = safe_filename(raw.name.as_deref()?)?;
    let download_url = clean_string(raw.download_url)?;
    Some(FigshareFile {
        id,
        filename,
        size: raw.size,
        md5: clean_string(raw.md5),
        mimetype: clean_string(raw.mimetype),
        download_url,
    })
}

fn clean_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn safe_filename(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let normalized = raw.replace('\\', "/");
    let mut out = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            Component::Normal(part) => {
                if part.to_str().is_some_and(|value| value.ends_with(':')) {
                    return None;
                }
                out.push(part);
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => return None,
        }
    }
    out.to_str()
        .map(str::trim)
        .filter(|value| !value.is_empty() && !value.contains('/'))
        .map(str::to_string)
}

#[cfg(test)]
mod tests;
