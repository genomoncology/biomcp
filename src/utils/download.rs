use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::BioMcpError;

pub fn cache_key(id: &str) -> String {
    format!("{:x}", md5::compute(id.as_bytes()))
}

fn download_path(id: &str) -> Result<PathBuf, BioMcpError> {
    let config = crate::cache::resolve_cache_config()?;
    Ok(download_path_for_config(id, &config))
}

fn download_path_for_config(id: &str, config: &crate::cache::ResolvedCacheConfig) -> PathBuf {
    config
        .cache_root
        .join("downloads")
        .join(format!("{}.txt", cache_key(id)))
}

async fn create_unique_sibling_temp(
    path: &Path,
    managed: bool,
) -> Result<(tokio::fs::File, PathBuf), BioMcpError> {
    let Some(dir) = path.parent() else {
        return Err(BioMcpError::InvalidArgument(
            "Invalid output path (no parent directory)".into(),
        ));
    };
    if managed {
        crate::cache::secure_managed_tree(dir, true)?;
    } else if !dir.is_dir() {
        return Err(BioMcpError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Output directory does not exist: {}", dir.display()),
        )));
    }

    let stem = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("tmp");
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    for attempt in 0..32_u32 {
        let candidate = dir.join(format!(
            ".{stem}.{}.tmp",
            seed.saturating_add(attempt as u128)
        ));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let opened = if managed {
            crate::cache::open_private(&mut options, &candidate)
        } else {
            options.open(&candidate)
        };
        match opened {
            Ok(file) => {
                if managed {
                    crate::cache::secure_managed_tree(dir, true)?;
                }
                return Ok((tokio::fs::File::from_std(file), candidate));
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err.into()),
        }
    }

    Err(BioMcpError::Io(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "Unable to allocate temporary output file",
    )))
}

async fn remove_temp_if_present(path: &Path) {
    let _ = tokio::fs::remove_file(path).await;
}

async fn existing_file_matches(path: &Path, content: &[u8], managed: bool) -> bool {
    let file = if managed {
        crate::cache::open_managed_read(path)
    } else {
        std::fs::File::open(path)
    };
    let Ok(file) = file else {
        return false;
    };
    let mut file = tokio::fs::File::from_std(file);
    let mut existing = Vec::new();
    file.read_to_end(&mut existing).await.is_ok() && existing == content
}

fn existing_regular_file(path: &Path, managed: bool) -> bool {
    if managed {
        crate::cache::open_managed_read(path).is_ok()
    } else {
        std::fs::symlink_metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
    }
}

async fn write_atomic_bytes_inner(
    path: &Path,
    content: &[u8],
    managed: bool,
) -> Result<(), BioMcpError> {
    let (mut file, tmp_path) = create_unique_sibling_temp(path, managed).await?;
    file.write_all(content).await?;
    file.flush().await?;
    file.sync_all().await?;
    drop(file);

    match tokio::fs::rename(&tmp_path, path).await {
        Ok(()) => Ok(()),
        Err(_err) if existing_file_matches(path, content, managed).await => {
            remove_temp_if_present(&tmp_path).await;
            Ok(())
        }
        Err(err) => {
            if !existing_regular_file(path, managed) {
                remove_temp_if_present(&tmp_path).await;
                return Err(err.into());
            }

            match tokio::fs::remove_file(path).await {
                Ok(()) => {}
                Err(remove_err) if remove_err.kind() == std::io::ErrorKind::NotFound => {}
                Err(remove_err) => {
                    remove_temp_if_present(&tmp_path).await;
                    return Err(remove_err.into());
                }
            }

            match tokio::fs::rename(&tmp_path, path).await {
                Ok(()) => Ok(()),
                Err(_retry_err) if existing_file_matches(path, content, managed).await => {
                    remove_temp_if_present(&tmp_path).await;
                    Ok(())
                }
                Err(retry_err) => {
                    remove_temp_if_present(&tmp_path).await;
                    Err(retry_err.into())
                }
            }
        }
    }
}

pub async fn write_atomic_bytes(path: &Path, content: &[u8]) -> Result<(), BioMcpError> {
    write_atomic_bytes_inner(path, content, true).await
}

/// Atomically writes a user-selected output without changing its directory permissions.
pub async fn write_user_atomic_bytes(path: &Path, content: &[u8]) -> Result<(), BioMcpError> {
    write_atomic_bytes_inner(path, content, false).await
}

pub async fn save_atomic(id: &str, content: &str) -> Result<PathBuf, BioMcpError> {
    let path = download_path(id)?;
    save_atomic_to_path(path, content).await
}

async fn save_atomic_to_path(path: PathBuf, content: &str) -> Result<PathBuf, BioMcpError> {
    let parent = path.parent().expect("download path has parent");
    crate::cache::secure_managed_tree(parent, true)?;
    match crate::cache::open_managed_read(&path) {
        Ok(_) => return Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    write_atomic_bytes(&path, content.as_bytes()).await?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        cache_key, download_path_for_config, save_atomic_to_path, write_atomic_bytes,
        write_user_atomic_bytes,
    };
    use crate::cache::{CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig};
    use crate::test_support::TempDirGuard;

    fn test_config(cache_root: impl Into<std::path::PathBuf>) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.into(),
            max_size: 10_000_000_000,
            min_disk_free: DiskFreeThreshold::Percent(10),
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    #[tokio::test]
    async fn write_atomic_bytes_replaces_existing_file_contents() {
        let root = TempDirGuard::new("replace-existing");
        let target = root.path().join("ema.json");
        std::fs::write(&target, b"old").expect("existing file should be writable");

        write_atomic_bytes(&target, b"new")
            .await
            .expect("atomic write should replace existing file");

        let updated = std::fs::read(&target).expect("updated file should be readable");
        assert_eq!(updated, b"new");
    }

    #[tokio::test]
    async fn write_atomic_bytes_errors_for_non_file_destination() {
        let root = TempDirGuard::new("destination-directory");
        let target = root.path().join("ema.json");
        std::fs::create_dir_all(&target).expect("target directory should be created");

        let err = write_atomic_bytes(&target, b"new")
            .await
            .expect_err("directory destination should fail");
        assert!(
            err.to_string().contains("Is a directory")
                || err.to_string().contains("directory")
                || err.to_string().contains("Access is denied"),
            "unexpected error: {err}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn user_output_does_not_restrict_parent_directory() {
        use std::os::unix::fs::PermissionsExt;

        let root = TempDirGuard::new("user-output-permissions");
        let output_dir = root.path().join("output");
        std::fs::create_dir(&output_dir).expect("output directory should be created");
        std::fs::set_permissions(&output_dir, std::fs::Permissions::from_mode(0o755))
            .expect("output directory permissions should be set");

        write_user_atomic_bytes(&output_dir.join("asset.bin"), b"bytes")
            .await
            .expect("user output should be written");

        assert_eq!(
            std::fs::metadata(&output_dir)
                .expect("output directory should exist")
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }

    #[test]
    fn download_path_for_config_resolves_to_cache_root_downloads() {
        let config = test_config("/tmp/biomcp-cache");
        let id = "pmid:12345";

        let path = download_path_for_config(id, &config);

        assert_eq!(
            path,
            std::path::PathBuf::from("/tmp/biomcp-cache")
                .join("downloads")
                .join(format!("{}.txt", cache_key(id)))
        );
    }

    #[test]
    fn download_path_for_config_keeps_relative_cache_roots_relative() {
        let config = test_config("relative-cache");
        let id = "pmid:12345";

        let path = download_path_for_config(id, &config);

        assert_eq!(
            path,
            std::path::PathBuf::from("relative-cache")
                .join("downloads")
                .join(format!("{}.txt", cache_key(id)))
        );
    }

    #[tokio::test]
    async fn save_atomic_to_path_writes_download_target() {
        let root = TempDirGuard::new("save-atomic-target");
        let id = "pmid:save-atomic";
        let path = root
            .path()
            .join("downloads")
            .join(format!("{}.txt", cache_key(id)));

        let saved_path = save_atomic_to_path(path.clone(), "hello world")
            .await
            .expect("save_atomic should write the target path");

        assert_eq!(saved_path, path);
        let content = std::fs::read_to_string(&saved_path).expect("saved file should exist");
        assert_eq!(content, "hello world");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn downloads_are_private_and_existing_files_are_repaired() {
        use std::os::unix::fs::PermissionsExt;

        let root = TempDirGuard::new("private-download");
        let target = root.path().join("downloads").join("article.txt");
        let saved = save_atomic_to_path(target.clone(), "private")
            .await
            .expect("private download");
        assert_eq!(
            std::fs::metadata(&saved).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(saved.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );

        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o664)).unwrap();
        save_atomic_to_path(target.clone(), "ignored")
            .await
            .expect("repair existing download");
        assert_eq!(
            std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[tokio::test]
    async fn save_atomic_to_path_errors_when_target_path_is_directory() {
        let root = TempDirGuard::new("save-atomic-directory-target");
        let id = "pmid:directory-target";
        let target = root
            .path()
            .join("downloads")
            .join(format!("{}.txt", cache_key(id)));
        std::fs::create_dir_all(&target).expect("directory target should exist");

        let err = save_atomic_to_path(target, "hello world")
            .await
            .expect_err("directory target should not short-circuit as a cached file");

        assert!(
            err.to_string().contains("Is a directory")
                || err.to_string().contains("directory")
                || err.to_string().contains("Access is denied"),
            "unexpected error: {err}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn saved_download_rejects_symlinks_and_hard_links() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("linked-download");
        let downloads = root.path().join("downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        let outside = root.path().join("outside.txt");
        std::fs::write(&outside, "outside").unwrap();

        let symlink_target = downloads.join("symlink.txt");
        symlink(&outside, &symlink_target).unwrap();
        assert!(
            save_atomic_to_path(symlink_target, "ignored")
                .await
                .is_err()
        );

        let hardlink_target = downloads.join("hardlink.txt");
        std::fs::hard_link(&outside, &hardlink_target).unwrap();
        assert!(
            save_atomic_to_path(hardlink_target, "ignored")
                .await
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(outside).unwrap(), "outside");
    }
}
