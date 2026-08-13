use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use tokio::io::AsyncWriteExt;

use crate::error::BioMcpError;
use crate::sources::archive_budget::{ArchiveBudget, ArchiveEntry, ArchiveLimits};
use crate::sources::{RequestPlan, request_from_plan};

const DATAHUB_BASE: &str = "https://datahub.assets.cbioportal.org";
const DATAHUB_API: &str = "cbioportal-datahub";
const DATAHUB_BASE_ENV: &str = "BIOMCP_CBIOPORTAL_DATAHUB_BASE";
const DATAHUB_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DATAHUB_ARCHIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_ARCHIVE_DOWNLOAD_BYTES: usize = 2 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: u64 = 100_000;
const MAX_ARCHIVE_MEMBER_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_ARCHIVE_EXPANDED_BYTES: u64 = 8 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_METADATA_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct StudyInstallResult {
    pub study_id: String,
    pub path: PathBuf,
    pub downloaded: bool,
}

pub struct CBioPortalDownloadClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    download_idle_timeout: Duration,
    max_archive_download_bytes: usize,
}

impl CBioPortalDownloadClient {
    pub fn new() -> Result<Self, BioMcpError> {
        let base = crate::sources::env_base(DATAHUB_BASE, DATAHUB_BASE_ENV);
        Ok(Self {
            client: datahub_client(base.as_ref(), DATAHUB_CONNECT_TIMEOUT, None)?,
            base,
            download_idle_timeout: DATAHUB_ARCHIVE_IDLE_TIMEOUT,
            max_archive_download_bytes: MAX_ARCHIVE_DOWNLOAD_BYTES,
        })
    }

    pub(crate) fn study_list_plan() -> RequestPlan {
        RequestPlan::get("study_list.json")
    }

    pub(crate) fn study_archive_plan(study_id: &str) -> Result<RequestPlan, BioMcpError> {
        let study_id = validate_study_id(study_id)?;
        Ok(RequestPlan::get(format!("{study_id}.tar.gz")))
    }

    pub(crate) fn decode_study_list_response(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        body: &[u8],
    ) -> Result<Vec<String>, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CBIOPORTAL_DATAHUB),
            status,
            content_type,
            body,
            true,
        )
    }

    pub(crate) fn decode_archive_status(
        study_id: &str,
        status: StatusCode,
        body: &[u8],
    ) -> Result<(), BioMcpError> {
        if matches!(status, StatusCode::FORBIDDEN | StatusCode::NOT_FOUND) {
            return Err(BioMcpError::NotFound {
                entity: "Study".to_string(),
                id: study_id.to_string(),
                suggestion: "Run `biomcp study download --list` to see available study IDs."
                    .to_string(),
            });
        }
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: DATAHUB_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(body)),
            });
        }
        Ok(())
    }

    async fn download_study_archive_to_path(
        &self,
        study_id: &str,
        dest: &Path,
    ) -> Result<(), BioMcpError> {
        let plan = Self::study_archive_plan(study_id)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let mut resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
            ))
            .await?;
        let status = resp.status();
        if !status.is_success() {
            let body = crate::sources::read_limited_source_body(
                resp,
                crate::error::SourceContext::narrow(
                    crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
                ),
            )
            .await?;
            return Self::decode_archive_status(study_id, status, &body).map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
                ))
            });
        }
        if resp
            .content_length()
            .is_some_and(|length| length > self.max_archive_download_bytes as u64)
        {
            return Err(BioMcpError::SourceUnavailable {
                source_name: "cBioPortal DataHub".to_string(),
                reason: "Archive download exceeded its resource limit.".to_string(),
                suggestion: "Retry the study download later.".to_string(),
            }
            .with_source_context(crate::error::SourceContext::narrow(
                crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
            )));
        }
        let mut file = tokio::fs::File::create(dest).await?;
        let mut downloaded = 0;
        loop {
            match tokio::time::timeout(self.download_idle_timeout, resp.chunk()).await {
                Ok(Ok(Some(chunk))) => {
                    downloaded = account_download_bytes(
                        downloaded,
                        chunk.len(),
                        self.max_archive_download_bytes,
                    )
                    .map_err(|error| {
                        error.with_source_context(crate::error::SourceContext::narrow(
                            crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
                        ))
                    })?;
                    file.write_all(&chunk).await?;
                }
                Ok(Ok(None)) => break,
                Ok(Err(err)) => {
                    return Err(BioMcpError::from(err).with_source_context(
                        crate::error::SourceContext::retry(
                            crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
                        ),
                    ));
                }
                Err(_) => {
                    return Err(BioMcpError::SourceUnavailable {
                        source_name: "cBioPortal DataHub".to_string(),
                        reason: format!(
                            "Archive download stalled because no bytes or progress arrived within {:?}.",
                            self.download_idle_timeout
                        ),
                        suggestion: "Retry the study download later.".to_string(),
                    }
                    .with_source_context(crate::error::SourceContext::retry(
                        crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
                    )));
                }
            }
        }
        file.flush().await?;
        Ok(())
    }

    pub async fn list_study_ids(&self) -> Result<Vec<String>, BioMcpError> {
        let plan = Self::study_list_plan();
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let body = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CBIOPORTAL_DATAHUB),
        )
        .await?;
        Self::decode_study_list_response(status, content_type.as_ref(), &body).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CBIOPORTAL_DATAHUB,
            ))
        })
    }

    pub async fn download_study(
        &self,
        study_id: &str,
        root: &Path,
    ) -> Result<StudyInstallResult, BioMcpError> {
        let study_id = validate_study_id(study_id)?;

        fs::create_dir_all(root)?;
        let target = root.join(study_id);
        if target.exists() {
            if is_valid_installed_study(root, study_id, &target)? {
                return Ok(StudyInstallResult {
                    study_id: study_id.to_string(),
                    path: target,
                    downloaded: false,
                });
            }
            return Err(BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: format!(
                    "Target directory already exists but is not a valid study: {}",
                    target.display()
                ),
                suggestion: "Remove the incomplete study directory and retry.".to_string(),
            });
        }

        let archive_path = unique_temp_path(root, &format!(".{study_id}.download"))?;
        let download_result = self
            .download_study_archive_to_path(study_id, &archive_path)
            .await;
        if let Err(err) = download_result {
            let _ = fs::remove_file(&archive_path);
            return Err(err);
        }

        let root = root.to_path_buf();
        let study_id = study_id.to_string();
        let archive_path_for_install = archive_path.clone();
        let install_result = tokio::task::spawn_blocking(move || {
            install_study_archive(&root, &study_id, &archive_path_for_install)
        })
        .await;
        let _ = fs::remove_file(&archive_path);
        install_result.map_err(|err| BioMcpError::Api {
            api: DATAHUB_API.to_string(),
            message: format!("Study install worker failed: {err}"),
        })?
    }
}

fn account_download_bytes(
    downloaded: usize,
    chunk_bytes: usize,
    max_archive_download_bytes: usize,
) -> Result<usize, BioMcpError> {
    let downloaded =
        downloaded
            .checked_add(chunk_bytes)
            .ok_or_else(|| BioMcpError::SourceUnavailable {
                source_name: "cBioPortal DataHub".to_string(),
                reason: "Archive download exceeded its resource limit.".to_string(),
                suggestion: "Retry the study download later.".to_string(),
            })?;
    if downloaded > max_archive_download_bytes {
        return Err(BioMcpError::SourceUnavailable {
            source_name: "cBioPortal DataHub".to_string(),
            reason: "Archive download exceeded its resource limit.".to_string(),
            suggestion: "Retry the study download later.".to_string(),
        });
    }
    Ok(downloaded)
}

fn datahub_client(
    base: &str,
    connect_timeout: Duration,
    total_timeout: Option<Duration>,
) -> Result<reqwest_middleware::ClientWithMiddleware, BioMcpError> {
    crate::sources::ordinary_middleware_client_for_base(base, DATAHUB_BASE_ENV, |builder| {
        let builder = builder
            .connect_timeout(connect_timeout)
            .user_agent(concat!("biomcp-cli/", env!("CARGO_PKG_VERSION")));
        if let Some(timeout) = total_timeout {
            builder.timeout(timeout)
        } else {
            builder
        }
    })
}

fn unique_temp_path(parent: &Path, prefix: &str) -> Result<PathBuf, BioMcpError> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    for attempt in 0..32_u32 {
        let candidate = parent.join(format!(
            "{prefix}-{}-{}-{}",
            std::process::id(),
            seed,
            attempt
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "Unable to allocate temporary study-install path",
    )))
}

fn archive_relative_path(study_id: &str, path: &Path) -> Result<Option<PathBuf>, BioMcpError> {
    let mut components = path.components();
    match components.next() {
        Some(Component::Normal(component)) if component == study_id => {}
        _ => {
            return Err(BioMcpError::Api {
                api: DATAHUB_API.to_string(),
                message: format!(
                    "Archive entry is outside the expected top-level study directory: {}",
                    path.display()
                ),
            });
        }
    }

    let mut relative = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(segment) => relative.push(segment),
            Component::CurDir => {}
            _ => {
                return Err(BioMcpError::Api {
                    api: DATAHUB_API.to_string(),
                    message: format!("Unsafe archive entry path: {}", path.display()),
                });
            }
        }
    }

    if relative.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(relative))
    }
}

fn validate_study_id(study_id: &str) -> Result<&str, BioMcpError> {
    let study_id = study_id.trim();
    if study_id.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Study ID is required.".to_string(),
        ));
    }
    let mut components = Path::new(study_id).components();
    let is_single_segment = matches!(
        (components.next(), components.next()),
        (Some(Component::Normal(_)), None)
    );
    if !is_single_segment
        || study_id.contains('\\')
        || study_id
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid study ID '{study_id}'. Expected a single identifier such as 'msk_impact_2017'."
        )));
    }
    Ok(study_id)
}

fn extract_archive_into(
    root: &Path,
    study_id: &str,
    archive_path: &Path,
) -> Result<PathBuf, BioMcpError> {
    let staging_root = unique_temp_path(root, &format!(".{study_id}.extract"))?;
    fs::create_dir_all(&staging_root)?;
    let staging_dir = staging_root.join(study_id);
    fs::create_dir_all(&staging_dir)?;

    let extract_result = (|| -> Result<(), BioMcpError> {
        let file = File::open(archive_path)?;
        let gz = GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);
        let limits = ArchiveLimits {
            max_entries: MAX_ARCHIVE_ENTRIES,
            max_member_bytes: MAX_ARCHIVE_MEMBER_BYTES,
            max_total_bytes: MAX_ARCHIVE_EXPANDED_BYTES,
            max_metadata_bytes: MAX_ARCHIVE_METADATA_BYTES,
        };
        let mut budget = ArchiveBudget::new(limits);
        for entry in archive.entries()?.raw(true) {
            let mut entry = entry.map_err(|_| BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: "Downloaded study archive failed its resource limit or metadata policy."
                    .to_string(),
                suggestion: "Retry the download or choose a different study.".to_string(),
            })?;
            let accounted =
                budget
                    .account(&mut entry)
                    .map_err(|_| BioMcpError::SourceUnavailable {
                        source_name: DATAHUB_API.to_string(),
                        reason:
                            "Downloaded study archive failed its resource limit or metadata policy."
                                .to_string(),
                        suggestion: "Retry the download or choose a different study.".to_string(),
                    })?;
            let entry_path = match accounted {
                ArchiveEntry::Metadata => continue,
                ArchiveEntry::Regular(path) | ArchiveEntry::Directory(path) => path,
            };
            let Some(relative) = archive_relative_path(study_id, &entry_path)? else {
                continue;
            };
            let dest = staging_dir.join(&relative);
            if !dest.starts_with(&staging_dir) {
                return Err(BioMcpError::Api {
                    api: DATAHUB_API.to_string(),
                    message: format!(
                        "Archive entry escaped staging directory: {}",
                        entry_path.display()
                    ),
                });
            }

            match entry.header().entry_type() {
                tar::EntryType::Directory => {
                    fs::create_dir_all(&dest)?;
                }
                tar::EntryType::Regular => {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut out = File::create(&dest)?;
                    std::io::copy(&mut entry, &mut out)?;
                }
                _ => {
                    return Err(BioMcpError::Api {
                        api: DATAHUB_API.to_string(),
                        message: format!(
                            "Unsupported archive entry type for {}",
                            entry_path.display()
                        ),
                    });
                }
            }
        }
        budget
            .finish()
            .map_err(|_| BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: "Downloaded study archive failed its resource limit or metadata policy."
                    .to_string(),
                suggestion: "Retry the download or choose a different study.".to_string(),
            })?;

        if !staging_dir.join("meta_study.txt").is_file() {
            return Err(BioMcpError::SourceUnavailable {
                source_name: DATAHUB_API.to_string(),
                reason: format!(
                    "Downloaded study archive for '{study_id}' is missing meta_study.txt"
                ),
                suggestion: "Retry the download or choose a different study.".to_string(),
            });
        }

        Ok(())
    })();

    match extract_result {
        Ok(()) => Ok(staging_root),
        Err(err) => {
            let _ = fs::remove_dir_all(&staging_root);
            Err(err)
        }
    }
}

fn is_valid_installed_study(
    root: &Path,
    study_id: &str,
    target: &Path,
) -> Result<bool, BioMcpError> {
    if !target.is_dir() || !target.join("meta_study.txt").is_file() {
        return Ok(false);
    }

    let studies = crate::sources::cbioportal_study::list_studies(root)?;
    Ok(studies
        .into_iter()
        .any(|study| study.study_id.eq_ignore_ascii_case(study_id) && study.path == target))
}

fn install_study_archive(
    root: &Path,
    study_id: &str,
    archive_path: &Path,
) -> Result<StudyInstallResult, BioMcpError> {
    fs::create_dir_all(root)?;
    let target = root.join(study_id);
    if target.exists() {
        if is_valid_installed_study(root, study_id, &target)? {
            return Ok(StudyInstallResult {
                study_id: study_id.to_string(),
                path: target,
                downloaded: false,
            });
        }
        return Err(BioMcpError::SourceUnavailable {
            source_name: DATAHUB_API.to_string(),
            reason: format!(
                "Target directory already exists but is not a valid study: {}",
                target.display()
            ),
            suggestion: "Remove the incomplete study directory and retry.".to_string(),
        });
    }

    let staging_root = extract_archive_into(root, study_id, archive_path)?;
    let staging_dir = staging_root.join(study_id);
    match fs::rename(&staging_dir, &target) {
        Ok(()) => {}
        Err(err) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(err.into());
        }
    }
    let _ = fs::remove_dir_all(&staging_root);

    if !is_valid_installed_study(root, study_id, &target)? {
        let _ = fs::remove_dir_all(&target);
        return Err(BioMcpError::SourceUnavailable {
            source_name: DATAHUB_API.to_string(),
            reason: format!(
                "Installed study '{study_id}' could not be validated by the local study loader"
            ),
            suggestion: "Retry the download or inspect the extracted study files.".to_string(),
        });
    }

    Ok(StudyInstallResult {
        study_id: study_id.to_string(),
        path: target,
        downloaded: true,
    })
}

#[cfg(test)]
mod tests;
