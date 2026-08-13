use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use http_cache_reqwest::CacheMode;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;
use crate::sources::provider_url_policy::{ProviderUrlConsumer, ProviderUrlPolicy};

const GITHUB_API: &str = "https://api.github.com/repos/genomoncology/biomcp/releases/latest";
const GITHUB_API_NAME: &str = "github";
const MAX_RELEASE_ARCHIVE_BYTES: usize = 256 * 1024 * 1024;
const MAX_EXTRACTED_BINARY_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn platform_asset_name() -> Result<&'static str, BioMcpError> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => Ok("biomcp-linux-x86_64.tar.gz"),
        ("linux", "aarch64") => Ok("biomcp-linux-arm64.tar.gz"),
        ("macos", "x86_64") => Ok("biomcp-darwin-x86_64.tar.gz"),
        ("macos", "aarch64") => Ok("biomcp-darwin-arm64.tar.gz"),
        ("windows", "x86_64") => Ok("biomcp-windows-x86_64.zip"),
        _ => Err(BioMcpError::InvalidArgument(format!(
            "Unsupported platform: {os} {arch}"
        ))),
    }
}

fn parse_semver(tag: &str) -> Option<semver::Version> {
    let trimmed = tag.trim();
    let trimmed = trimmed.strip_prefix('v').unwrap_or(trimmed);
    semver::Version::parse(trimmed).ok()
}

fn extract_binary_from_targz(bytes: &[u8], binary_name: &str) -> Result<Vec<u8>, BioMcpError> {
    if bytes.len() > MAX_RELEASE_ARCHIVE_BYTES {
        return Err(BioMcpError::Api {
            api: "update".into(),
            message: format!("Release archive exceeded {MAX_RELEASE_ARCHIVE_BYTES} bytes"),
        });
    }

    let gz = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(gz);
    let entries = archive.entries()?;

    for entry in entries {
        let entry = entry?;
        if entry.size() > MAX_EXTRACTED_BINARY_BYTES {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Binary in release archive exceeded size limit".into(),
            });
        }
        let path = entry.path()?;
        let Some(file_name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if file_name != binary_name {
            continue;
        }

        let mut out: Vec<u8> = Vec::new();
        let mut reader = entry.take(MAX_EXTRACTED_BINARY_BYTES + 1);
        reader.read_to_end(&mut out)?;
        if out.len() as u64 > MAX_EXTRACTED_BINARY_BYTES {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Binary in release archive exceeded size limit".into(),
            });
        }
        if out.is_empty() {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Downloaded archive contained an empty binary".into(),
            });
        }
        return Ok(out);
    }

    Err(BioMcpError::NotFound {
        entity: "release asset".into(),
        id: binary_name.to_string(),
        suggestion: "Release archive did not contain expected biomcp binary".into(),
    })
}

fn extract_binary_from_zip(bytes: &[u8], binary_name: &str) -> Result<Vec<u8>, BioMcpError> {
    if bytes.len() > MAX_RELEASE_ARCHIVE_BYTES {
        return Err(BioMcpError::Api {
            api: "update".into(),
            message: format!("Release archive exceeded {MAX_RELEASE_ARCHIVE_BYTES} bytes"),
        });
    }

    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|err| BioMcpError::Api {
        api: "update".into(),
        message: format!("ZIP error: {err}"),
    })?;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|err| BioMcpError::Api {
            api: "update".into(),
            message: format!("ZIP error: {err}"),
        })?;
        let name = file
            .name()
            .rsplit('/')
            .find(|s| !s.is_empty())
            .unwrap_or(file.name());
        if name != binary_name {
            continue;
        }
        if file.size() > MAX_EXTRACTED_BINARY_BYTES {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Binary in release archive exceeded size limit".into(),
            });
        }
        let mut out: Vec<u8> = Vec::new();
        let mut reader = file.take(MAX_EXTRACTED_BINARY_BYTES + 1);
        reader.read_to_end(&mut out)?;
        if out.len() as u64 > MAX_EXTRACTED_BINARY_BYTES {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Binary in release archive exceeded size limit".into(),
            });
        }
        if out.is_empty() {
            return Err(BioMcpError::Api {
                api: "update".into(),
                message: "Downloaded archive contained an empty binary".into(),
            });
        }
        return Ok(out);
    }

    Err(BioMcpError::NotFound {
        entity: "release asset".into(),
        id: binary_name.to_string(),
        suggestion: "Release archive did not contain expected biomcp binary".into(),
    })
}

static STAGE_NONCE: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
fn replace_owned_binary_at(
    current: &Path,
    new_bytes: &[u8],
    new_version: &str,
) -> Result<(), BioMcpError> {
    let owned = crate::cli::install::validate_owned(current)?;
    let Some(parent) = current.parent() else {
        return Err(BioMcpError::InvalidArgument(
            "Cannot determine current executable directory".into(),
        ));
    };

    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
    let installed_mode = std::fs::symlink_metadata(current)?.permissions().mode();
    let mut staged = None;
    for _ in 0..32 {
        let nonce = format!(
            "{}-{}",
            std::process::id(),
            STAGE_NONCE.fetch_add(1, Ordering::Relaxed)
        );
        let path = parent.join(format!(".biomcp-stage-{nonce}"));
        let opened = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o700)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path);
        match opened {
            Ok(file) => {
                staged = Some((path, nonce, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    let (stage_path, nonce, mut stage_file) = staged.ok_or_else(|| {
        BioMcpError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not create unique update staging file",
        ))
    })?;
    let result = (|| {
        stage_file.write_all(new_bytes)?;
        stage_file.set_permissions(std::fs::Permissions::from_mode(installed_mode))?;
        stage_file.sync_all()?;
        drop(stage_file);
        let smoke = std::process::Command::new(&stage_path)
            .arg("version")
            .output()?;
        let reported = String::from_utf8_lossy(&smoke.stdout);
        let requested = new_version.trim().trim_start_matches('v');
        if !smoke.status.success()
            || !reported
                .split_whitespace()
                .any(|word| word.trim_start_matches('v') == requested)
        {
            return Err(std::io::Error::other(format!(
                "staged BioMCP did not report requested version {new_version}"
            )));
        }
        let new_sha = sha256_hex(new_bytes);
        let revalidated = crate::cli::install::validate_owned(current)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        if revalidated.receipt != owned.receipt {
            return Err(std::io::Error::other(
                "installer receipt changed during update",
            ));
        }
        let pending =
            crate::cli::install::pending_receipt(&revalidated, new_version, &new_sha, &nonce);
        crate::cli::install::write_receipt_atomic(&revalidated.receipt_path, &pending)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        std::fs::rename(&stage_path, current)?;
        std::fs::File::open(parent)?.sync_all()?;
        let final_receipt = crate::cli::install::installed_receipt(&pending);
        crate::cli::install::write_receipt_atomic(&revalidated.receipt_path, &final_receipt)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        Ok::<(), std::io::Error>(())
    })();
    if stage_path.exists() {
        let _ = std::fs::remove_file(&stage_path);
    }
    result.map_err(BioMcpError::Io)
}

fn replace_current_binary(new_bytes: &[u8], new_version: &str) -> Result<(), BioMcpError> {
    #[cfg(windows)]
    {
        let _ = (new_bytes, new_version);
        return Err(BioMcpError::InvalidArgument(
            "Self-update is unsupported on Windows; use the verified standalone installer.".into(),
        ));
    }
    #[cfg(unix)]
    {
        let current = std::fs::canonicalize(std::env::current_exe()?)?;
        replace_owned_binary_at(&current, new_bytes, new_version)
    }
}

fn binary_name_for_platform() -> &'static str {
    if cfg!(windows) {
        "biomcp.exe"
    } else {
        "biomcp"
    }
}

async fn fetch_latest_release_from(url: &str) -> Result<GithubRelease, BioMcpError> {
    let (client, url) = update_client(url)?;
    let resp = client
        .get(url)
        .with_extension(CacheMode::NoStore)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    let status = resp.status();
    let bytes = crate::sources::read_limited_body(resp, GITHUB_API_NAME).await?;
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(&bytes);
        return Err(BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }

    serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
        api: GITHUB_API_NAME.into(),
        source,
    })
}

async fn fetch_latest_release() -> Result<GithubRelease, BioMcpError> {
    fetch_latest_release_from(GITHUB_API).await
}

async fn download_asset_with_limit(url: &str, max_bytes: usize) -> Result<Vec<u8>, BioMcpError> {
    let (client, url) = update_client(url)?;
    let request = client.get(url).with_extension(CacheMode::NoStore);
    let resp = crate::sources::with_response_body_limit(request, max_bytes, GITHUB_API_NAME)
        .send()
        .await?;
    let status = resp.status();
    let bytes =
        crate::sources::read_limited_body_with_limit(resp, GITHUB_API_NAME, max_bytes).await?;
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(&bytes);
        return Err(BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }
    Ok(bytes.to_vec())
}

async fn download_archive(url: &str) -> Result<Vec<u8>, BioMcpError> {
    download_asset_with_limit(url, MAX_RELEASE_ARCHIVE_BYTES).await
}

async fn download_asset_optional(url: &str) -> Result<Option<Vec<u8>>, BioMcpError> {
    let (client, url) = update_client(url)?;
    let request = client.get(url).with_extension(CacheMode::NoStore);
    let resp = crate::sources::with_response_body_limit(
        request,
        crate::sources::DEFAULT_MAX_BODY_BYTES,
        GITHUB_API_NAME,
    )
    .send()
    .await?;
    let status = resp.status();
    let bytes = crate::sources::read_limited_body(resp, GITHUB_API_NAME).await?;
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        let excerpt = crate::sources::body_excerpt(&bytes);
        return Err(BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!("HTTP {status}: {excerpt}"),
        });
    }
    Ok(Some(bytes.to_vec()))
}

fn update_client(
    raw_url: &str,
) -> Result<(reqwest_middleware::ClientWithMiddleware, reqwest::Url), BioMcpError> {
    let url = reqwest::Url::parse(raw_url)
        .map_err(|_| BioMcpError::InvalidArgument("Invalid release URL".into()))?;
    #[cfg(test)]
    let policy = if url.host_str().is_some_and(|host| {
        host.trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
    }) {
        ProviderUrlPolicy::test_fixture(ProviderUrlConsumer::GithubRelease, &url)?
    } else {
        ProviderUrlPolicy::for_consumer(ProviderUrlConsumer::GithubRelease, None)?
    };
    #[cfg(not(test))]
    let policy = ProviderUrlPolicy::for_consumer(ProviderUrlConsumer::GithubRelease, None)?;
    policy.validate_url(&url)?;
    Ok((crate::sources::provider_url_client(&policy)?, url))
}

fn parse_sha256_from_checksum_file(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|token| token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()))
        .map(|token| token.to_ascii_lowercase())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

async fn fetch_checksum_status(
    asset_url: &str,
    archive_bytes: &[u8],
) -> Result<ChecksumStatus, BioMcpError> {
    let checksum_url = format!("{asset_url}.sha256");
    let Some(checksum_bytes) = download_asset_optional(&checksum_url).await? else {
        return Ok(ChecksumStatus::MissingSidecar);
    };

    let checksum_text = String::from_utf8_lossy(&checksum_bytes);
    verify_archive_against_checksum(&checksum_text, archive_bytes)?;
    Ok(ChecksumStatus::Verified)
}

// ---- 331 fail-closed checksum policy ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// dead-code reason: update::ChecksumStatus is exercised by binary dispatch or CLI contracts
#[allow(dead_code)]
enum ChecksumStatus {
    Verified,
    MissingSidecar,
}

fn enforce_checksum_policy(status: ChecksumStatus, asset_name: &str) -> Result<(), BioMcpError> {
    match status {
        ChecksumStatus::Verified => Ok(()),
        ChecksumStatus::MissingSidecar => Err(BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!(
                "Release checksum verification failed for {asset_name}: SHA256 checksum sidecar is missing. Use the verified standalone installer instead."
            ),
        }),
    }
}

fn verify_archive_against_checksum(
    checksum_text: &str,
    archive_bytes: &[u8],
) -> Result<(), BioMcpError> {
    let expected =
        parse_sha256_from_checksum_file(checksum_text).ok_or_else(|| BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: "Invalid checksum file format".into(),
        })?;
    let actual = sha256_hex(archive_bytes);

    if actual != expected {
        return Err(BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!(
                "Checksum mismatch for downloaded asset. expected={expected} actual={actual}"
            ),
        });
    }

    Ok(())
}

fn install_binary_after_checksum_policy<F>(
    status: ChecksumStatus,
    asset_name: &str,
    new_binary: &[u8],
    replace_binary: F,
) -> Result<(), BioMcpError>
where
    F: FnOnce(&[u8]) -> Result<(), BioMcpError>,
{
    enforce_checksum_policy(status, asset_name)?;
    replace_binary(new_binary)?;
    Ok(())
}

fn render_check_output(current: &str, latest_tag: &str, status_line: &str) -> String {
    format!("Current version: {current}\nLatest version: {latest_tag}\nStatus: {status_line}\n")
}

/// Checks for and optionally installs the latest release binary.
///
/// # Errors
///
/// Returns an error if release metadata cannot be fetched, download verification
/// fails, archive extraction fails, or the local binary cannot be replaced.
pub async fn run(check_only: bool) -> Result<String, BioMcpError> {
    let current = env!("CARGO_PKG_VERSION").trim();
    let current_v = semver::Version::parse(current).ok();

    let release = fetch_latest_release().await?;
    let latest_tag = release.tag_name.trim().to_string();
    let latest_v = parse_semver(&latest_tag);

    let update_available = match (current_v.as_ref(), latest_v.as_ref()) {
        (Some(cur), Some(latest)) => latest > cur,
        _ => false,
    };

    if check_only {
        let status_line = if update_available {
            "not up to date (update available)"
        } else {
            "up to date"
        };
        return Ok(render_check_output(current, &latest_tag, status_line));
    }

    #[cfg(windows)]
    return Err(BioMcpError::InvalidArgument(
        "Self-update is unsupported on Windows; use the verified standalone installer.".into(),
    ));

    if !update_available {
        return Ok(render_check_output(current, &latest_tag, "up to date"));
    }

    let asset_name = platform_asset_name()?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| BioMcpError::NotFound {
            entity: "release asset".into(),
            id: asset_name.to_string(),
            suggestion: "Check GitHub releases for a compatible platform build".into(),
        })?;

    let archive_bytes = download_archive(&asset.browser_download_url).await?;
    let checksum_status =
        fetch_checksum_status(&asset.browser_download_url, &archive_bytes).await?;
    let bin_name = binary_name_for_platform();

    let new_binary = if asset_name.ends_with(".tar.gz") {
        extract_binary_from_targz(&archive_bytes, bin_name)?
    } else if asset_name.ends_with(".zip") {
        extract_binary_from_zip(&archive_bytes, bin_name)?
    } else {
        return Err(BioMcpError::InvalidArgument(format!(
            "Unsupported asset format: {asset_name}"
        )));
    };

    install_binary_after_checksum_policy(checksum_status, asset_name, &new_binary, |bytes| {
        replace_current_binary(bytes, &latest_tag)
    })?;
    Ok(format!("Updated BioMCP to {latest_tag}\n"))
}

#[cfg(test)]
mod tests;
