use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;

use http_cache_reqwest::CacheMode;
use regex::Regex;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

// PubMed Central Open Access (OA) service
// Docs: https://www.ncbi.nlm.nih.gov/pmc/tools/oa/
const PMC_OA_BASE: &str = "https://www.ncbi.nlm.nih.gov/pmc/utils/oa/oa.fcgi";
const PMC_OA_API: &str = "pmc-oa";
const PMC_OA_BASE_ENV: &str = "BIOMCP_PMC_OA_BASE";
const MAX_TGZ_BYTES: usize = 64 * 1024 * 1024;
const MAX_ARCHIVE_ENTRY_BYTES: u64 = 8 * 1024 * 1024;

static TGZ_HREF_RE: OnceLock<Regex> = OnceLock::new();
static LICENSE_ATTR_RE: OnceLock<Regex> = OnceLock::new();
static RETRACTED_ATTR_RE: OnceLock<Regex> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PmcOaArchiveManifest {
    pub tgz_url: String,
    pub package_url: String,
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
}

impl PmcOaClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PMC_OA_BASE, PMC_OA_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = req.with_extension(CacheMode::NoStore).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, PMC_OA_API).await?;
        decode_text(status, &bytes)
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

        let mut plan = RequestPlan::get("").query("id", pmcid);
        if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
            plan = plan.query("api_key", key);
        }
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
        let xml = self.get_text(req).await?;
        parse_archive_manifest_xml(&xml)
    }

    pub async fn get_full_text_xml_with_manifest(
        &self,
        pmcid: &str,
    ) -> Result<Option<(String, PmcOaArchiveManifest)>, BioMcpError> {
        let Some(manifest) = self.oa_archive_manifest(pmcid).await? else {
            return Ok(None);
        };

        let bytes = self.archive_bytes(&manifest).await?;
        let xml = tokio::task::spawn_blocking(move || extract_first_nxml(&bytes))
            .await
            .map_err(|err| BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: format!("Task join error: {err}"),
            })??;

        Ok(xml.map(|xml| (xml, manifest)))
    }

    pub(crate) async fn archive_package(
        &self,
        manifest: PmcOaArchiveManifest,
    ) -> Result<PmcOaArchivePackage, BioMcpError> {
        let bytes = self.archive_bytes(&manifest).await?;
        let entries = tokio::task::spawn_blocking(move || extract_archive_entries(&bytes))
            .await
            .map_err(|err| BioMcpError::Api {
                api: PMC_OA_API.to_string(),
                message: format!("Task join error: {err}"),
            })??;
        Ok(PmcOaArchivePackage { manifest, entries })
    }
}

fn decode_text(status: reqwest::StatusCode, bytes: &[u8]) -> Result<String, BioMcpError> {
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(bytes);
        return Err(BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }
    Ok(String::from_utf8_lossy(bytes).to_string())
}

fn parse_archive_manifest_xml(xml: &str) -> Result<Option<PmcOaArchiveManifest>, BioMcpError> {
    let document = roxmltree::Document::parse(xml).map_err(|_| BioMcpError::Api {
        api: PMC_OA_API.to_string(),
        message: "Invalid PMC OA manifest XML".to_string(),
    })?;
    let records = document
        .descendants()
        .any(|node| node.is_element() && node.tag_name().name() == "records");
    if !records {
        let root = document.root_element();
        let not_open_access = root.tag_name().name() == "OA"
            && root.children().any(|node| {
                node.is_element()
                    && node.tag_name().name() == "error"
                    && node.attribute("code") == Some("idIsNotOpenAccess")
            });
        if not_open_access {
            return Ok(None);
        }
        return Err(BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: "Unexpected PMC OA manifest XML".to_string(),
        });
    }

    let re = TGZ_HREF_RE.get_or_init(|| {
        Regex::new(r#"<link[^>]*format="tgz"[^>]*href="([^"]+)""#).expect("valid tgz href regex")
    });

    let Some(caps) = re.captures(xml) else {
        return Ok(None);
    };
    let Some(raw_href) = caps
        .get(1)
        .map(|m| m.as_str().trim())
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };

    let href = if raw_href.starts_with("ftp://ftp.ncbi.nlm.nih.gov/") {
        raw_href.replacen(
            "ftp://ftp.ncbi.nlm.nih.gov/",
            "https://ftp.ncbi.nlm.nih.gov/",
            1,
        )
    } else if raw_href.starts_with("ftp://") {
        raw_href.replacen("ftp://", "https://", 1)
    } else {
        raw_href.to_string()
    };

    let license = LICENSE_ATTR_RE
        .get_or_init(|| Regex::new(r#"\blicense="([^"]+)""#).expect("valid license regex"))
        .captures(xml)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty());
    let retracted = RETRACTED_ATTR_RE
        .get_or_init(|| Regex::new(r#"\bretracted="([^"]+)""#).expect("valid retracted regex"))
        .captures(xml)
        .and_then(|caps| caps.get(1))
        .and_then(|value| parse_boolish(value.as_str()));

    Ok(Some(PmcOaArchiveManifest {
        tgz_url: href.clone(),
        package_url: href,
        license,
        retracted,
    }))
}

impl PmcOaClient {
    async fn archive_bytes(&self, manifest: &PmcOaArchiveManifest) -> Result<Vec<u8>, BioMcpError> {
        let resp = self
            .client
            .get(&manifest.tgz_url)
            .with_extension(CacheMode::NoStore)
            .send()
            .await?;
        let status = resp.status();
        let bytes =
            crate::sources::read_limited_body_with_limit(resp, PMC_OA_API, MAX_TGZ_BYTES).await?;
        decode_archive_bytes(status, &bytes)
    }
}

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

fn parse_boolish(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => Some(true),
        "n" | "no" | "false" | "0" => Some(false),
        _ => None,
    }
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

fn extract_archive_entries(tgz_bytes: &[u8]) -> Result<Vec<PmcOaArchiveEntry>, BioMcpError> {
    use std::io::Read;

    if tgz_bytes.len() > MAX_TGZ_BYTES {
        return Err(BioMcpError::Api {
            api: PMC_OA_API.to_string(),
            message: format!("PMC OA archive exceeded {MAX_TGZ_BYTES} bytes"),
        });
    }

    let gz = flate2::read::GzDecoder::new(tgz_bytes);
    let mut archive = tar::Archive::new(gz);
    let entries = archive.entries()?;
    let mut out = Vec::new();

    for entry in entries {
        let entry = entry?;
        if !entry.header().entry_type().is_file() || entry.size() > MAX_ARCHIVE_ENTRY_BYTES {
            continue;
        }
        let path = entry.path()?;
        let Some(filename) = safe_archive_name(&path) else {
            continue;
        };

        let mut bytes = Vec::new();
        let mut reader = entry.take(MAX_ARCHIVE_ENTRY_BYTES + 1);
        reader.read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_ARCHIVE_ENTRY_BYTES || bytes.is_empty() {
            continue;
        }
        let is_xml = is_xml_name(&filename);
        out.push(PmcOaArchiveEntry {
            filename,
            bytes,
            is_xml,
        });
    }

    Ok(out)
}

fn extract_first_nxml(tgz_bytes: &[u8]) -> Result<Option<String>, BioMcpError> {
    for entry in extract_archive_entries(tgz_bytes)? {
        if entry.is_xml {
            return Ok(Some(String::from_utf8_lossy(&entry.bytes).to_string()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
