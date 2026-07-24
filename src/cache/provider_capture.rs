use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_CAPTURE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_RETAINED_BYTES: u64 = 64 * 1024 * 1024;
const RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ProviderCaptureProvider {
    Cspec,
}

impl ProviderCaptureProvider {
    fn parse(value: &str) -> Result<Self, ProviderCaptureError> {
        match value {
            "cspec" => Ok(Self::Cspec),
            _ => Err(ProviderCaptureError::UnsupportedProvider),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Cspec => "cspec",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProviderCaptureManifest {
    pub(crate) capture_id: String,
    pub(crate) provider: ProviderCaptureProvider,
    pub(crate) media_type: String,
    pub(crate) byte_length: u64,
    pub(crate) sha256: String,
    pub(crate) captured_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) schema_version: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderCaptureError {
    UnsupportedProvider,
    Oversize,
    Unavailable,
    Corrupt,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderCaptureStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    manifest: ProviderCaptureManifest,
    last_access_at: u64,
}

impl ProviderCaptureStore {
    pub(crate) fn new(cache_root: impl AsRef<Path>) -> Self {
        Self {
            root: cache_root.as_ref().join("captures"),
        }
    }

    pub(crate) fn capture_bytes(
        &self,
        provider: ProviderCaptureProvider,
        media_type: impl Into<String>,
        bytes: &[u8],
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        self.capture(provider, media_type, io::Cursor::new(bytes))
    }

    pub(crate) fn capture<R: Read>(
        &self,
        provider: ProviderCaptureProvider,
        media_type: impl Into<String>,
        mut body: R,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        fs::create_dir_all(self.root.join(".staging"))
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        let (mut file, staged) = (0..100)
            .find_map(|attempt| {
                let staged = self.root.join(".staging").join(format!(
                    ".capture.{}.{}.tmp",
                    std::process::id(),
                    attempt
                ));
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&staged)
                    .ok()
                    .map(|file| (file, staged))
            })
            .ok_or(ProviderCaptureError::Corrupt)?;
        let capture_result = (|| {
            let mut hasher = Sha256::new();
            let mut length = 0u64;
            let mut buffer = [0u8; 8192];
            loop {
                let read = body
                    .read(&mut buffer)
                    .map_err(|_| ProviderCaptureError::Corrupt)?;
                if read == 0 {
                    break;
                }
                length = length
                    .checked_add(read as u64)
                    .ok_or(ProviderCaptureError::Oversize)?;
                if length > MAX_CAPTURE_BYTES {
                    return Err(ProviderCaptureError::Oversize);
                }
                hasher.update(&buffer[..read]);
                file.write_all(&buffer[..read])
                    .map_err(|_| ProviderCaptureError::Corrupt)?;
            }
            file.sync_all().map_err(|_| ProviderCaptureError::Corrupt)?;
            drop(file);

            let sha256 = format!("{:x}", hasher.finalize());
            let now = now_secs()?;
            let manifest = ProviderCaptureManifest {
                capture_id: format!("capture:{}:sha256:{sha256}", provider.as_str()),
                provider,
                media_type: media_type.into(),
                byte_length: length,
                sha256: sha256.clone(),
                captured_at: now,
                expires_at: now + RETENTION.as_secs(),
                schema_version: SCHEMA_VERSION,
            };
            self.with_lock(|| {
                let blob = self.blob_path(provider, &sha256);
                let metadata = self.metadata_path(provider, &sha256);
                if let Ok(existing) = self.read_complete(&manifest.capture_id) {
                    return Ok(existing.manifest);
                }
                let parent = blob.parent().ok_or(ProviderCaptureError::Corrupt)?;
                fs::create_dir_all(parent).map_err(|_| ProviderCaptureError::Corrupt)?;
                fs::rename(&staged, &blob).map_err(|_| ProviderCaptureError::Corrupt)?;
                sync_dir(parent)?;
                let record = Metadata {
                    manifest: manifest.clone(),
                    last_access_at: now,
                };
                write_atomic(
                    &metadata,
                    &serde_json::to_vec(&record).map_err(|_| ProviderCaptureError::Corrupt)?,
                )?;
                self.maintain_locked(now)?;
                Ok(manifest)
            })
        })();
        let _ = fs::remove_file(&staged);
        capture_result
    }

    pub(crate) fn read(&self, capture_id: &str) -> Result<Vec<u8>, ProviderCaptureError> {
        self.with_lock(|| {
            let mut record = self.read_complete(capture_id)?;
            let now = now_secs()?;
            if record.manifest.expires_at <= now {
                return Err(ProviderCaptureError::Unavailable);
            }
            record.last_access_at = now;
            let path = self.metadata_path(record.manifest.provider, &record.manifest.sha256);
            write_atomic(
                &path,
                &serde_json::to_vec(&record).map_err(|_| ProviderCaptureError::Corrupt)?,
            )?;
            self.maintain_locked(now)?;
            self.verified_bytes(&record)
        })
    }

    pub(crate) fn retained_bytes(&self) -> Result<u64, ProviderCaptureError> {
        self.blob_entries()
            .map(|entries| entries.into_iter().map(|entry| entry.size).sum())
    }

    pub(crate) fn maintain(&self) -> Result<u64, ProviderCaptureError> {
        self.with_lock(|| self.maintain_locked(now_secs()?))
    }

    fn with_lock<T>(
        &self,
        action: impl FnOnce() -> Result<T, ProviderCaptureError>,
    ) -> Result<T, ProviderCaptureError> {
        fs::create_dir_all(&self.root).map_err(|_| ProviderCaptureError::Corrupt)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.root.join(".lock"))
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        lock.lock_exclusive()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        let result = action();
        let _ = lock.unlock();
        result
    }

    fn read_complete(&self, capture_id: &str) -> Result<Metadata, ProviderCaptureError> {
        let (provider, digest) = parse_handle(capture_id)?;
        let metadata_path = self.metadata_path(provider, digest);
        if !regular_file(&metadata_path) {
            return Err(if fs::symlink_metadata(&metadata_path).is_ok() {
                ProviderCaptureError::Corrupt
            } else {
                ProviderCaptureError::Unavailable
            });
        }
        let metadata: Metadata = serde_json::from_slice(
            &fs::read(&metadata_path).map_err(|_| ProviderCaptureError::Corrupt)?,
        )
        .map_err(|_| ProviderCaptureError::Corrupt)?;
        if metadata.manifest.capture_id != capture_id
            || metadata.manifest.provider != provider
            || metadata.manifest.sha256 != digest
            || metadata.manifest.schema_version != SCHEMA_VERSION
        {
            return Err(ProviderCaptureError::Corrupt);
        }
        self.verified_bytes(&metadata)?;
        Ok(metadata)
    }

    fn verified_bytes(&self, record: &Metadata) -> Result<Vec<u8>, ProviderCaptureError> {
        let blob_path = self.blob_path(record.manifest.provider, &record.manifest.sha256);
        if !regular_file(&blob_path) {
            return Err(ProviderCaptureError::Corrupt);
        }
        let blob = fs::read(blob_path).map_err(|_| ProviderCaptureError::Corrupt)?;
        if blob.len() as u64 != record.manifest.byte_length
            || format!("{:x}", Sha256::digest(&blob)) != record.manifest.sha256
        {
            return Err(ProviderCaptureError::Corrupt);
        }
        Ok(blob)
    }

    fn maintain_locked(&self, now: u64) -> Result<u64, ProviderCaptureError> {
        let mut freed = 0;
        let staging = self.root.join(".staging");
        if regular_dir(&staging) {
            for entry in fs::read_dir(&staging).map_err(|_| ProviderCaptureError::Corrupt)? {
                let path = entry.map_err(|_| ProviderCaptureError::Corrupt)?.path();
                if regular_file(&path) {
                    let size = fs::metadata(&path)
                        .map_err(|_| ProviderCaptureError::Corrupt)?
                        .len();
                    fs::remove_file(path).map_err(|_| ProviderCaptureError::Corrupt)?;
                    freed += size;
                }
            }
        }
        let mut records = self.metadata_entries()?;
        for record in &records {
            if record.metadata.manifest.expires_at <= now {
                freed += self.remove_record(record)?;
            }
        }
        records = self.metadata_entries()?;
        let referenced = records
            .iter()
            .map(|record| record.metadata.manifest.sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for blob in self.blob_entries()? {
            if !referenced.contains(blob.digest.as_str()) {
                fs::remove_file(&blob.path).map_err(|_| ProviderCaptureError::Corrupt)?;
                freed += blob.size;
            }
        }
        records.sort_by_key(|record| {
            (
                record.metadata.last_access_at,
                record.provider.as_str(),
                record.metadata.manifest.capture_id.clone(),
            )
        });
        let mut retained: u64 = records
            .iter()
            .map(|record| record.metadata.manifest.byte_length)
            .sum();
        for record in records {
            if retained <= MAX_RETAINED_BYTES {
                break;
            }
            retained -= record.metadata.manifest.byte_length;
            freed += self.remove_record(&record)?;
        }
        Ok(freed)
    }

    fn remove_record(&self, record: &MetadataEntry) -> Result<u64, ProviderCaptureError> {
        let blob = self.blob_path(record.provider, &record.metadata.manifest.sha256);
        let freed = fs::metadata(&blob)
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        fs::remove_file(&record.path).map_err(|_| ProviderCaptureError::Corrupt)?;
        let _ = fs::remove_file(blob);
        Ok(freed)
    }

    fn metadata_entries(&self) -> Result<Vec<MetadataEntry>, ProviderCaptureError> {
        let mut entries = Vec::new();
        let dir = self.root.join("cspec").join("metadata");
        if !dir.exists() {
            return Ok(entries);
        }
        for shard in fs::read_dir(dir).map_err(|_| ProviderCaptureError::Corrupt)? {
            let shard = shard.map_err(|_| ProviderCaptureError::Corrupt)?.path();
            if !shard.is_dir() {
                continue;
            }
            for entry in fs::read_dir(shard).map_err(|_| ProviderCaptureError::Corrupt)? {
                let path = entry.map_err(|_| ProviderCaptureError::Corrupt)?.path();
                if !regular_file(&path) {
                    continue;
                }
                let metadata: Metadata = serde_json::from_slice(
                    &fs::read(&path).map_err(|_| ProviderCaptureError::Corrupt)?,
                )
                .map_err(|_| ProviderCaptureError::Corrupt)?;
                let (provider, digest) = parse_handle(&metadata.manifest.capture_id)?;
                if provider != ProviderCaptureProvider::Cspec
                    || metadata.manifest.provider != provider
                    || metadata.manifest.sha256 != digest
                    || metadata.manifest.schema_version != SCHEMA_VERSION
                {
                    return Err(ProviderCaptureError::Corrupt);
                }
                entries.push(MetadataEntry {
                    provider: ProviderCaptureProvider::Cspec,
                    metadata,
                    path,
                });
            }
        }
        Ok(entries)
    }

    fn blob_entries(&self) -> Result<Vec<BlobEntry>, ProviderCaptureError> {
        let mut entries = Vec::new();
        let dir = self.root.join("cspec").join("sha256");
        if !dir.exists() {
            return Ok(entries);
        }
        for shard in fs::read_dir(dir).map_err(|_| ProviderCaptureError::Corrupt)? {
            let shard = shard.map_err(|_| ProviderCaptureError::Corrupt)?.path();
            if !shard.is_dir() {
                continue;
            }
            for entry in fs::read_dir(shard).map_err(|_| ProviderCaptureError::Corrupt)? {
                let path = entry.map_err(|_| ProviderCaptureError::Corrupt)?.path();
                if regular_file(&path) {
                    entries.push(BlobEntry {
                        digest: path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .ok_or(ProviderCaptureError::Corrupt)?
                            .to_string(),
                        size: fs::metadata(&path)
                            .map_err(|_| ProviderCaptureError::Corrupt)?
                            .len(),
                        path,
                    });
                }
            }
        }
        Ok(entries)
    }

    fn blob_path(&self, provider: ProviderCaptureProvider, digest: &str) -> PathBuf {
        self.root
            .join(provider.as_str())
            .join("sha256")
            .join(&digest[..2])
            .join(digest)
    }

    fn metadata_path(&self, provider: ProviderCaptureProvider, digest: &str) -> PathBuf {
        self.root
            .join(provider.as_str())
            .join("metadata")
            .join(&digest[..2])
            .join(format!("{digest}.json"))
    }
}

struct MetadataEntry {
    provider: ProviderCaptureProvider,
    metadata: Metadata,
    path: PathBuf,
}
struct BlobEntry {
    digest: String,
    size: u64,
    path: PathBuf,
}

fn now_secs() -> Result<u64, ProviderCaptureError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| ProviderCaptureError::Corrupt)
}

fn parse_handle(value: &str) -> Result<(ProviderCaptureProvider, &str), ProviderCaptureError> {
    let mut parts = value.split(':');
    let (Some("capture"), Some(provider), Some("sha256"), Some(digest), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        return Err(ProviderCaptureError::Unavailable);
    };
    if digest.len() != 64
        || !digest.bytes().all(|byte| {
            byte.is_ascii_digit() || (byte.is_ascii_lowercase() && byte.is_ascii_hexdigit())
        })
    {
        return Err(ProviderCaptureError::Unavailable);
    }
    Ok((ProviderCaptureProvider::parse(provider)?, digest))
}

fn regular_file(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_file())
}

fn regular_dir(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_dir())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ProviderCaptureError> {
    let parent = path.parent().ok_or(ProviderCaptureError::Corrupt)?;
    fs::create_dir_all(parent).map_err(|_| ProviderCaptureError::Corrupt)?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or(ProviderCaptureError::Corrupt)?,
        std::process::id()
    ));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        file.write_all(bytes)
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        file.sync_all().map_err(|_| ProviderCaptureError::Corrupt)?;
        fs::rename(&temporary, path).map_err(|_| ProviderCaptureError::Corrupt)?;
        sync_dir(parent)
    })();
    let _ = fs::remove_file(temporary);
    result
}

fn sync_dir(path: &Path) -> Result<(), ProviderCaptureError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|_| ProviderCaptureError::Corrupt)
}

#[cfg(test)]
mod tests {
    use super::{ProviderCaptureError, ProviderCaptureProvider, ProviderCaptureStore};
    use crate::test_support::TempDirGuard;

    #[test]
    fn captures_and_reads_exact_bytes_with_content_addressed_dedupe() {
        let root = TempDirGuard::new("provider-capture");
        let store = ProviderCaptureStore::new(root.path());
        let first = store
            .capture_bytes(
                ProviderCaptureProvider::Cspec,
                "application/json",
                b"exact bytes",
            )
            .expect("capture");
        let same = store
            .capture_bytes(
                ProviderCaptureProvider::Cspec,
                "application/json",
                b"exact bytes",
            )
            .expect("dedupe");
        let changed = store
            .capture_bytes(
                ProviderCaptureProvider::Cspec,
                "application/json",
                b"exact byte!",
            )
            .expect("changed");
        assert_eq!(first.capture_id, same.capture_id);
        assert_ne!(first.capture_id, changed.capture_id);
        assert_eq!(store.read(&first.capture_id), Ok(b"exact bytes".to_vec()));
    }

    #[test]
    fn rejects_oversize_and_invalid_handles() {
        let root = TempDirGuard::new("provider-capture-bound");
        let store = ProviderCaptureStore::new(root.path());
        assert_eq!(
            store.capture_bytes(
                ProviderCaptureProvider::Cspec,
                "text/plain",
                &vec![0; 4 * 1024 * 1024 + 1]
            ),
            Err(ProviderCaptureError::Oversize)
        );
        assert_eq!(store.read("capture:other:sha256:0000000000000000000000000000000000000000000000000000000000000000"), Err(ProviderCaptureError::UnsupportedProvider));
    }

    #[test]
    fn rejects_corrupt_bytes_and_removes_unpublished_orphans() {
        let root = TempDirGuard::new("provider-capture-corrupt");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        std::fs::write(
            root.path()
                .join("captures/cspec/sha256")
                .join(&manifest.sha256[..2])
                .join(&manifest.sha256),
            b"changed",
        )
        .expect("corrupt blob");
        assert_eq!(
            store.read(&manifest.capture_id),
            Err(ProviderCaptureError::Corrupt)
        );

        let orphan = root.path().join("captures/cspec/sha256/ff/orphan");
        std::fs::create_dir_all(orphan.parent().expect("orphan parent")).expect("create parent");
        std::fs::write(&orphan, b"orphan").expect("write orphan");
        store.maintain().expect("maintain");
        assert!(!orphan.exists());
    }
}
