use std::io::Read;

use http_cache_reqwest::CacheMode;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;

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

fn replace_current_binary(new_bytes: &[u8]) -> Result<(), BioMcpError> {
    let current = std::env::current_exe()?;
    let Some(parent) = current.parent() else {
        return Err(BioMcpError::InvalidArgument(
            "Cannot determine current executable directory".into(),
        ));
    };

    let tmp_path = parent.join(format!(
        ".{}.new",
        current
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("biomcp")
    ));

    {
        let mut file = std::fs::File::create(&tmp_path)?;
        std::io::Write::write_all(&mut file, new_bytes).map_err(BioMcpError::Io)?;
        std::io::Write::flush(&mut file).map_err(BioMcpError::Io)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
        std::fs::rename(&tmp_path, &current)?;
    }

    #[cfg(windows)]
    {
        // Windows cannot overwrite a running executable directly.
        // Rename the current binary out of the way first, then move
        // the new one into place.  Clean up the old file best-effort.
        let old_path = current.with_extension("old.exe");
        let _ = std::fs::remove_file(&old_path);
        std::fs::rename(&current, &old_path)?;
        if let Err(err) = std::fs::rename(&tmp_path, &current) {
            // Restore the original if the swap failed.
            let _ = std::fs::rename(&old_path, &current);
            return Err(err.into());
        }
        let _ = std::fs::remove_file(&old_path);
    }

    Ok(())
}

fn binary_name_for_platform() -> &'static str {
    if cfg!(windows) {
        "biomcp.exe"
    } else {
        "biomcp"
    }
}

async fn fetch_latest_release() -> Result<GithubRelease, BioMcpError> {
    let client = crate::sources::shared_client()?;
    let resp = client
        .get(GITHUB_API)
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

async fn download_asset(url: &str) -> Result<Vec<u8>, BioMcpError> {
    let client = crate::sources::shared_client()?;
    let resp = client
        .get(url)
        .with_extension(CacheMode::NoStore)
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
    Ok(bytes.to_vec())
}

async fn download_asset_optional(url: &str) -> Result<Option<Vec<u8>>, BioMcpError> {
    let client = crate::sources::shared_client()?;
    let resp = client
        .get(url)
        .with_extension(CacheMode::NoStore)
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

async fn verify_archive_checksum_if_available(
    asset_url: &str,
    archive_bytes: &[u8],
) -> Result<bool, BioMcpError> {
    let checksum_url = format!("{asset_url}.sha256");
    let Some(checksum_bytes) = download_asset_optional(&checksum_url).await? else {
        return Ok(false);
    };

    let checksum_text = String::from_utf8_lossy(&checksum_bytes);
    let expected =
        parse_sha256_from_checksum_file(&checksum_text).ok_or_else(|| BioMcpError::Api {
            api: GITHUB_API_NAME.into(),
            message: format!("Invalid checksum file format at {checksum_url}"),
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

    Ok(true)
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

    let archive_bytes = download_asset(&asset.browser_download_url).await?;
    let checksum_warning = if verify_archive_checksum_if_available(
        &asset.browser_download_url,
        &archive_bytes,
    )
    .await?
    {
        None
    } else {
        Some(format!(
            "Warning: checksum file missing for {asset_name}; continuing without checksum verification."
        ))
    };
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

    replace_current_binary(&new_binary)?;

    let mut output = String::new();
    if let Some(warning) = checksum_warning {
        output.push_str(&warning);
        output.push('\n');
    }
    output.push_str(&format!("Updated BioMCP to {latest_tag}\n"));
    Ok(output)
}
