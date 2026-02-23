use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::AsyncWriteExt;

use crate::error::BioMcpError;

pub fn biomcp_cache_dir() -> PathBuf {
    match dirs::cache_dir() {
        Some(dir) => dir.join("biomcp"),
        None => std::env::temp_dir().join("biomcp"),
    }
}

pub fn biomcp_downloads_dir() -> PathBuf {
    std::env::temp_dir().join("biomcp")
}

pub fn cache_key(id: &str) -> String {
    format!("{:x}", md5::compute(id.as_bytes()))
}

pub fn cache_path(id: &str) -> PathBuf {
    biomcp_downloads_dir().join(format!("{}.txt", cache_key(id)))
}

pub async fn save_atomic(id: &str, content: &str) -> Result<PathBuf, BioMcpError> {
    let path = cache_path(id);
    if tokio::fs::metadata(&path).await.is_ok() {
        return Ok(path);
    }

    let Some(dir) = path.parent() else {
        return Err(BioMcpError::InvalidArgument(
            "Invalid cache path (no parent directory)".into(),
        ));
    };
    tokio::fs::create_dir_all(dir).await?;

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut tmp_path = None;
    let mut file_opt = None;
    for attempt in 0..32_u32 {
        let candidate = dir.join(format!(
            ".{}.{}.{}.tmp",
            cache_key(id),
            std::process::id(),
            seed.saturating_add(attempt as u128)
        ));
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
            .await
        {
            Ok(file) => {
                tmp_path = Some(candidate);
                file_opt = Some(file);
                break;
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err.into()),
        }
    }
    let Some(tmp_path) = tmp_path else {
        return Err(BioMcpError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Unable to allocate secure temporary cache file",
        )));
    };
    let Some(mut file) = file_opt else {
        return Err(BioMcpError::Io(std::io::Error::other(
            "Temporary cache file handle was not initialized",
        )));
    };
    {
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;
    }

    match tokio::fs::rename(&tmp_path, &path).await {
        Ok(()) => Ok(path),
        Err(_) if tokio::fs::metadata(&path).await.is_ok() => Ok(path),
        Err(err) => Err(err.into()),
    }
}
