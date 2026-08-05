use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;
#[cfg(test)]
use crate::sources::archive_budget::{ArchiveBudget, ArchiveEntry, ArchiveLimits};
use crate::sources::provider_url_policy::{ProviderUrlConsumer, ProviderUrlPolicy};
use crate::sources::{RequestPlan, request_from_plan};

// PubMed Central Open Access (OA) service
// Docs: https://www.ncbi.nlm.nih.gov/pmc/tools/oa/
const PMC_OA_BASE: &str = "https://pmc-oa-opendata.s3.amazonaws.com";
const PMC_OA_API: &str = "pmc-oa";
const PMC_OA_BASE_ENV: &str = "BIOMCP_PMC_OA_BASE";
#[cfg(test)]
const MAX_TGZ_BYTES: usize = 64 * 1024 * 1024;
const MAX_ARCHIVE_ENTRY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: u64 = 256;
const MAX_ARCHIVE_EXPANDED_BYTES: u64 = 64 * 1024 * 1024;
#[cfg(test)]
const MAX_ARCHIVE_METADATA_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmcOaArchiveManifest {
    /// Metadata-declared XML object URL. Kept as `tgz_url` for the existing article interface.
    pub tgz_url: String,
    /// Durable versioned metadata URL used as public provenance.
    pub package_url: String,
    pub media_urls: Vec<String>,
    pub license: Option<String>,
    pub retracted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmcOaArchiveEntry {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub is_xml: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmcOaArchivePackage {
    pub manifest: PmcOaArchiveManifest,
    pub entries: Vec<PmcOaArchiveEntry>,
}

#[derive(Clone)]
pub struct PmcOaClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
    provider_policy: ProviderUrlPolicy,
}

impl PmcOaClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(PMC_OA_BASE, PMC_OA_BASE_ENV);
        let base_url = reqwest::Url::parse(base.as_ref()).map_err(|_| BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: "PMC OA source unavailable: outbound policy rejected invalid base URL".into(),
        })?;
        let provider_policy =
            ProviderUrlPolicy::for_consumer(ProviderUrlConsumer::PmcOaArchive, Some(&base_url))?;
        Ok(Self {
            client: crate::sources::provider_url_client(&provider_policy)?,
            base,
            api_key: crate::sources::ncbi_api_key(),
            provider_policy,
        })
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = crate::sources::with_response_body_limit(
            req.with_extension(CacheMode::NoStore),
            MAX_ARCHIVE_ENTRY_BYTES as usize,
            PMC_OA_API,
        )
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::PMC_OPEN_ACCESS,
        ))
        .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PMC_OPEN_ACCESS),
        )
        .await?;
        decode_text(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PMC_OPEN_ACCESS,
            ))
        })
    }

    pub(crate) fn oa_archive_manifest_plan(
        pmcid: &str,
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let pmcid = pmcid.trim();
        if pmcid.is_empty() {
            return Ok(None);
        }
        if pmcid.len() > 64 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        let plan = RequestPlan::get("")
            .query("list-type", "2")
            .query("prefix", format!("{pmcid}."))
            .query("delimiter", "/");
        let _ = api_key;
        Ok(Some(plan))
    }

    pub(crate) async fn oa_archive_manifest(
        &self,
        pmcid: &str,
    ) -> Result<Option<PmcOaArchiveManifest>, BioMcpError> {
        let Some(plan) = Self::oa_archive_manifest_plan(pmcid, self.api_key.as_deref())? else {
            return Ok(None);
        };
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let listing = self.get_text(req).await?;
        let Some(metadata_manifest) = parse_archive_manifest_xml(&listing)? else {
            return Ok(None);
        };
        if !manifest_matches_pmcid(&metadata_manifest, pmcid) {
            return Err(route_error("S3 listing named a different PMCID"));
        }
        let metadata_url = self.object_url(&metadata_manifest.package_url)?;
        let metadata = self
            .get_text(
                self.client
                    .get(metadata_url)
                    .with_extension(CacheMode::NoStore),
            )
            .await?;
        let manifest = parse_archive_manifest_xml(&metadata).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::PMC_OPEN_ACCESS,
            ))
        })?;
        if manifest
            .as_ref()
            .is_some_and(|manifest| manifest.package_url != metadata_manifest.package_url)
        {
            return Err(route_error(
                "metadata identity did not match its S3 listing",
            ));
        }
        Ok(manifest)
    }

    pub async fn get_full_text_xml_with_manifest(
        &self,
        pmcid: &str,
    ) -> Result<Option<(String, PmcOaArchiveManifest)>, BioMcpError> {
        let Some(manifest) = self.oa_archive_manifest(pmcid).await? else {
            return Ok(None);
        };

        let bytes = self.object_bytes(&manifest.tgz_url).await?;
        let xml = std::str::from_utf8(&bytes)
            .map(str::to_string)
            .map_err(|_| route_error("resolved XML object was not valid UTF-8"))?;

        Ok(Some((xml, manifest)))
    }

    pub(crate) async fn archive_package(
        &self,
        manifest: PmcOaArchiveManifest,
    ) -> Result<PmcOaArchivePackage, BioMcpError> {
        if manifest.media_urls.len() as u64 > MAX_ARCHIVE_ENTRIES {
            return Err(route_error("metadata named too many media objects"));
        }
        let mut total = 0_u64;
        let mut entries = Vec::new();
        for media_url in std::iter::once(&manifest.tgz_url).chain(&manifest.media_urls) {
            let bytes = self.object_bytes(media_url).await?;
            total = total.saturating_add(bytes.len() as u64);
            if total > MAX_ARCHIVE_EXPANDED_BYTES {
                return Err(route_error(
                    "media objects exceeded the aggregate resource limit",
                ));
            }
            let filename = reqwest::Url::parse(media_url)
                .ok()
                .and_then(|url| {
                    Path::new(url.path())
                        .file_name()?
                        .to_str()
                        .map(str::to_string)
                })
                .and_then(|name| safe_archive_name(Path::new(&name)))
                .ok_or_else(|| route_error("metadata named an unsafe media object"))?;
            entries.push(PmcOaArchiveEntry {
                is_xml: is_xml_name(&filename),
                filename,
                bytes,
            });
        }
        Ok(PmcOaArchivePackage { manifest, entries })
    }
}

fn decode_text(status: reqwest::StatusCode, bytes: &[u8]) -> Result<String, BioMcpError> {
    if !status.is_success() {
        return Err(route_error(&format!("route returned HTTP {status}")));
    }
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: "PMC OA response was not valid UTF-8".to_string(),
        })
}

fn parse_archive_manifest_xml(body: &str) -> Result<Option<PmcOaArchiveManifest>, BioMcpError> {
    if body.trim_start().starts_with('<') {
        let document = roxmltree::Document::parse(body)
            .map_err(|_| route_error("invalid S3 version listing"))?;
        if document.root_element().tag_name().name() != "ListBucketResult" {
            return Err(route_error("invalid S3 version listing"));
        }
        let prefixes = document
            .descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == "Prefix")
            .filter_map(|node| node.text())
            .map(str::trim)
            .filter(|prefix| {
                prefix.starts_with("PMC")
                    && prefix.ends_with('/')
                    && prefix[3..prefix.len() - 1]
                        .split_once('.')
                        .is_some_and(|(id, version)| {
                            !id.is_empty()
                                && id.bytes().all(|byte| byte.is_ascii_digit())
                                && !version.is_empty()
                                && version.bytes().all(|byte| byte.is_ascii_digit())
                        })
            })
            .collect::<Vec<_>>();
        let Some(prefix) = prefixes.into_iter().max_by_key(|prefix| {
            prefix[3..prefix.len() - 1]
                .rsplit_once('.')
                .expect("validated version prefix")
                .1
                .parse::<u64>()
                .expect("validated numeric version")
        }) else {
            return Ok(None);
        };
        return Ok(Some(PmcOaArchiveManifest {
            tgz_url: format!(
                "https://pmc-oa-opendata.s3.amazonaws.com/{0}/{0}.json",
                &prefix[..prefix.len() - 1]
            ),
            package_url: format!(
                "https://pmc-oa-opendata.s3.amazonaws.com/{0}/{0}.json",
                &prefix[..prefix.len() - 1]
            ),
            media_urls: Vec::new(),
            license: None,
            retracted: None,
        }));
    }

    let metadata: serde_json::Value =
        serde_json::from_str(body).map_err(|_| route_error("invalid S3 metadata object"))?;
    let object_url = |raw: &str| -> Result<String, BioMcpError> {
        let url = reqwest::Url::parse(raw)
            .map_err(|_| route_error("metadata named a malformed object URL"))?;
        if url.scheme() != "s3" || url.host_str() != Some("pmc-oa-opendata") {
            return Err(route_error(
                "metadata named an object outside the PMC OA bucket",
            ));
        }
        let path = url.path().trim_start_matches('/');
        if path.is_empty()
            || path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(route_error("metadata named a malformed object path"));
        }
        let mut https = reqwest::Url::parse("https://pmc-oa-opendata.s3.amazonaws.com/")
            .expect("constant URL is valid");
        https.set_path(path);
        https.set_query(url.query());
        Ok(https.to_string())
    };
    let xml_url = metadata["xml_url"]
        .as_str()
        .ok_or_else(|| route_error("metadata omitted a required object URL"))
        .and_then(object_url)?;
    let media_urls = metadata["media_urls"]
        .as_array()
        .ok_or_else(|| route_error("metadata omitted media object URLs"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| route_error("metadata named a malformed media object URL"))
                .and_then(object_url)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let pmcid = metadata["pmcid"]
        .as_str()
        .ok_or_else(|| route_error("metadata omitted PMCID"))?;
    let version = metadata["version"]
        .as_u64()
        .ok_or_else(|| route_error("metadata omitted numeric version"))?;
    let package_url = format!(
        "https://pmc-oa-opendata.s3.amazonaws.com/{pmcid}.{version}/{pmcid}.{version}.json"
    );
    Ok(Some(PmcOaArchiveManifest {
        tgz_url: xml_url,
        package_url,
        media_urls,
        license: metadata["license_code"].as_str().map(str::to_string),
        retracted: metadata["is_retracted"].as_bool(),
    }))
}

impl PmcOaClient {
    fn object_url(&self, raw: &str) -> Result<reqwest::Url, BioMcpError> {
        let canonical = reqwest::Url::parse(raw)
            .map_err(|_| route_error("metadata named an invalid object URL"))?;
        self.provider_policy
            .validate_url(&canonical)
            .map_err(|_| route_error("metadata named a URL rejected by provider policy"))?;
        if self.base.as_ref() == PMC_OA_BASE {
            return Ok(canonical);
        }
        let mut fixture = reqwest::Url::parse(self.base.as_ref())
            .map_err(|_| route_error("configured PMC OA base was invalid"))?;
        fixture.set_path(canonical.path());
        fixture.set_query(canonical.query());
        self.provider_policy
            .validate_url(&fixture)
            .map_err(|_| route_error("configured fixture URL was rejected by provider policy"))?;
        Ok(fixture)
    }

    async fn object_bytes(&self, raw: &str) -> Result<Vec<u8>, BioMcpError> {
        let object_url = self.object_url(raw)?;
        let request = self
            .client
            .get(object_url)
            .with_extension(CacheMode::NoStore);
        let resp = crate::sources::with_response_body_limit(
            request,
            MAX_ARCHIVE_ENTRY_BYTES as usize,
            PMC_OA_API,
        )
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::PMC_OPEN_ACCESS,
        ))
        .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body_with_limit(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::PMC_OPEN_ACCESS),
            MAX_ARCHIVE_ENTRY_BYTES as usize,
        )
        .await?;
        if !status.is_success() {
            return Err(route_error(&format!(
                "resolved object returned HTTP {status}"
            )));
        }
        Ok(bytes)
    }
}

fn manifest_matches_pmcid(manifest: &PmcOaArchiveManifest, pmcid: &str) -> bool {
    manifest
        .package_url
        .starts_with(&format!("{PMC_OA_BASE}/{}.", pmcid.trim()))
}

fn route_error(reason: &str) -> BioMcpError {
    BioMcpError::Api {
        api: PMC_OA_API.to_string(),
        message: format!("PMC OA package-route resolution failed: {reason}"),
    }
}

#[cfg(test)]
fn decode_archive_bytes(status: reqwest::StatusCode, bytes: &[u8]) -> Result<Vec<u8>, BioMcpError> {
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(bytes);
        return Err(BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }
    Ok(bytes.to_vec())
}

fn is_xml_name(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    lower.ends_with(".nxml") || lower.ends_with(".xml")
}

fn safe_archive_name(path: &Path) -> Option<String> {
    let raw = path.to_str()?.trim();
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
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
fn extract_archive_entries(tgz_bytes: &[u8]) -> Result<Vec<PmcOaArchiveEntry>, BioMcpError> {
    use std::io::Read;

    if tgz_bytes.len() > MAX_TGZ_BYTES {
        return Err(BioMcpError::SourceUnavailable {
            source_name: PMC_OA_API.to_string(),
            reason: "PMC OA archive failed its resource limit or metadata policy.".to_string(),
            suggestion: "Try another full-text source or retry later.".to_string(),
        });
    }

    let gz = flate2::read::GzDecoder::new(tgz_bytes);
    let mut archive = tar::Archive::new(gz);
    let entries = archive.entries()?.raw(true);
    let limits = ArchiveLimits {
        max_entries: MAX_ARCHIVE_ENTRIES,
        max_member_bytes: MAX_ARCHIVE_ENTRY_BYTES,
        max_total_bytes: MAX_ARCHIVE_EXPANDED_BYTES,
        max_metadata_bytes: MAX_ARCHIVE_METADATA_BYTES,
    };
    let mut budget = ArchiveBudget::new(limits);
    let mut out = Vec::new();

    for entry in entries {
        let mut entry = entry.map_err(|_| BioMcpError::SourceUnavailable {
            source_name: PMC_OA_API.to_string(),
            reason: "PMC OA archive failed its resource limit or metadata policy.".to_string(),
            suggestion: "Try another full-text source or retry later.".to_string(),
        })?;
        let accounted = budget
            .account(&mut entry)
            .map_err(|_| BioMcpError::SourceUnavailable {
                source_name: PMC_OA_API.to_string(),
                reason: "PMC OA archive failed its resource limit or metadata policy.".to_string(),
                suggestion: "Try another full-text source or retry later.".to_string(),
            })?;
        let ArchiveEntry::Regular(path) = accounted else {
            continue;
        };
        let Some(filename) = safe_archive_name(&path) else {
            continue;
        };

        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        if bytes.is_empty() {
            continue;
        }
        let is_xml = is_xml_name(&filename);
        out.push(PmcOaArchiveEntry {
            filename,
            bytes,
            is_xml,
        });
    }

    budget
        .finish()
        .map_err(|_| BioMcpError::SourceUnavailable {
            source_name: PMC_OA_API.to_string(),
            reason: "PMC OA archive failed its resource limit or metadata policy.".to_string(),
            suggestion: "Try another full-text source or retry later.".to_string(),
        })?;
    Ok(out)
}

#[cfg(test)]
fn extract_first_nxml(tgz_bytes: &[u8]) -> Result<Option<String>, BioMcpError> {
    for entry in extract_archive_entries(tgz_bytes)? {
        if entry.is_xml {
            let xml = std::str::from_utf8(&entry.bytes).map_err(|_| BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: "PMC OA full text XML was not valid UTF-8".to_string(),
            })?;
            return Ok(Some(xml.to_string()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
