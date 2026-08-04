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
    pub(crate) capture_binding: Option<CspecCaptureBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CspecCaptureBinding {
    pub(crate) binding_schema_version: u8,
    pub(crate) normalized_gene: String,
    pub(crate) resource_iri: String,
    pub(crate) specification_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderCaptureError {
    UnsupportedProvider,
    Oversize,
    Unavailable,
    Corrupt,
    BindingConflict,
}

#[derive(Debug, Clone)]
pub(crate) struct ProviderCaptureStore {
    root: PathBuf,
    #[cfg(test)]
    pause_after_blob_parent_ready: Option<std::sync::Arc<TestPublicationPause>>,
}

#[cfg(test)]
#[derive(Debug)]
struct TestPublicationPause {
    ready: std::sync::mpsc::Sender<()>,
    resume: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
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
            #[cfg(test)]
            pause_after_blob_parent_ready: None,
        }
    }

    #[cfg(test)]
    fn with_blob_parent_pause(mut self, pause: std::sync::Arc<TestPublicationPause>) -> Self {
        self.pause_after_blob_parent_ready = Some(pause);
        self
    }

    #[cfg(test)]
    fn pause_after_blob_parent_ready(&self) {
        if let Some(pause) = &self.pause_after_blob_parent_ready {
            pause.ready.send(()).expect("test waits for publication");
            pause
                .resume
                .lock()
                .expect("test publication pause lock")
                .recv()
                .expect("test resumes publication");
        }
    }

    pub(crate) fn capture_bytes(
        &self,
        provider: ProviderCaptureProvider,
        media_type: impl Into<String>,
        bytes: &[u8],
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        self.capture(provider, media_type, io::Cursor::new(bytes), None)
    }

    pub(crate) fn capture_cspec_bytes(
        &self,
        binding: CspecCaptureBinding,
        bytes: &[u8],
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        self.capture(
            ProviderCaptureProvider::Cspec,
            "application/json",
            io::Cursor::new(bytes),
            Some(binding),
        )
    }

    pub(crate) fn capture<R: Read>(
        &self,
        provider: ProviderCaptureProvider,
        media_type: impl Into<String>,
        mut body: R,
        capture_binding: Option<CspecCaptureBinding>,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        self.ensure_directory(&self.root)?;
        self.ensure_directory(&self.root.join(".staging"))?;
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
                capture_binding,
            };
            self.with_lock(|| {
                let blob = self.blob_path(provider, &sha256);
                let metadata = self.metadata_path(provider, &sha256);
                if let Ok(existing) = self.read_complete(&manifest.capture_id)
                    && existing.manifest.expires_at > now
                {
                    return if existing.manifest.capture_binding == manifest.capture_binding {
                        Ok(existing.manifest)
                    } else {
                        Err(ProviderCaptureError::BindingConflict)
                    };
                }
                let parent = blob.parent().ok_or(ProviderCaptureError::Corrupt)?;
                self.ensure_directory(parent)?;
                #[cfg(test)]
                self.pause_after_blob_parent_ready();
                fs::rename(&staged, &blob).map_err(|_| ProviderCaptureError::Corrupt)?;
                sync_dir(parent)?;
                let record = Metadata {
                    manifest: manifest.clone(),
                    last_access_at: now,
                };
                self.ensure_directory(metadata.parent().ok_or(ProviderCaptureError::Corrupt)?)?;
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

    pub(crate) fn read_manifest(
        &self,
        capture_id: &str,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        self.with_lock(|| self.read_complete(capture_id).map(|record| record.manifest))
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
            self.ensure_directory(path.parent().ok_or(ProviderCaptureError::Corrupt)?)?;
            write_atomic(
                &path,
                &serde_json::to_vec(&record).map_err(|_| ProviderCaptureError::Corrupt)?,
            )?;
            self.maintain_locked(now)?;
            self.verified_bytes(&record)
        })
    }

    pub(crate) fn retained_bytes(&self) -> Result<u64, ProviderCaptureError> {
        self.with_lock(|| retained_regular_file_bytes(&self.root))
    }

    pub(crate) fn maintain(&self) -> Result<u64, ProviderCaptureError> {
        self.with_lock(|| self.maintain_locked(now_secs()?))
    }

    pub(crate) fn planned_maintenance_bytes_freed(&self) -> Result<u64, ProviderCaptureError> {
        self.with_lock(|| self.planned_maintenance_bytes_freed_locked(now_secs()?))
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
            || metadata.manifest.provider == ProviderCaptureProvider::Cspec
                && metadata
                    .manifest
                    .capture_binding
                    .as_ref()
                    .is_some_and(|binding| binding.binding_schema_version != 1)
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
        for record in records {
            if retained_regular_file_bytes(&self.root)? <= MAX_RETAINED_BYTES {
                break;
            }
            freed += self.remove_record(&record)?;
        }
        Ok(freed)
    }

    fn planned_maintenance_bytes_freed_locked(
        &self,
        now: u64,
    ) -> Result<u64, ProviderCaptureError> {
        let mut freed = 0;
        let staging = self.root.join(".staging");
        if regular_dir(&staging) {
            for entry in fs::read_dir(&staging).map_err(|_| ProviderCaptureError::Corrupt)? {
                let path = entry.map_err(|_| ProviderCaptureError::Corrupt)?.path();
                if regular_file(&path) {
                    freed += fs::metadata(&path)
                        .map_err(|_| ProviderCaptureError::Corrupt)?
                        .len();
                }
            }
        }

        let mut records = self.metadata_entries()?;
        let mut retained_records = Vec::new();
        for record in records.drain(..) {
            if record.metadata.manifest.expires_at <= now {
                freed += self.record_regular_file_bytes(&record)?;
            } else {
                retained_records.push(record);
            }
        }
        let referenced = retained_records
            .iter()
            .map(|record| record.metadata.manifest.sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for blob in self.blob_entries()? {
            if !referenced.contains(blob.digest.as_str()) {
                freed += blob.size;
            }
        }
        retained_records.sort_by_key(|record| {
            (
                record.metadata.last_access_at,
                record.provider.as_str(),
                record.metadata.manifest.capture_id.clone(),
            )
        });
        let mut retained = retained_regular_file_bytes(&self.root)?
            .checked_sub(freed)
            .ok_or(ProviderCaptureError::Corrupt)?;
        for record in retained_records {
            if retained <= MAX_RETAINED_BYTES {
                break;
            }
            let record_bytes = self.record_regular_file_bytes(&record)?;
            retained = retained
                .checked_sub(record_bytes)
                .ok_or(ProviderCaptureError::Corrupt)?;
            freed += record_bytes;
        }
        Ok(freed)
    }

    fn remove_record(&self, record: &MetadataEntry) -> Result<u64, ProviderCaptureError> {
        let blob = self.blob_path(record.provider, &record.metadata.manifest.sha256);
        let freed = self.record_regular_file_bytes(record)?;
        fs::remove_file(&record.path).map_err(|_| ProviderCaptureError::Corrupt)?;
        let _ = fs::remove_file(blob);
        Ok(freed)
    }

    fn record_regular_file_bytes(
        &self,
        record: &MetadataEntry,
    ) -> Result<u64, ProviderCaptureError> {
        let metadata = fs::metadata(&record.path)
            .map_err(|_| ProviderCaptureError::Corrupt)?
            .len();
        let blob = self.blob_path(record.provider, &record.metadata.manifest.sha256);
        let blob = if regular_file(&blob) {
            fs::metadata(blob)
                .map_err(|_| ProviderCaptureError::Corrupt)?
                .len()
        } else {
            0
        };
        metadata
            .checked_add(blob)
            .ok_or(ProviderCaptureError::Corrupt)
    }

    fn metadata_entries(&self) -> Result<Vec<MetadataEntry>, ProviderCaptureError> {
        let mut entries = Vec::new();
        let dir = self.root.join("cspec").join("metadata");
        if !directory_exists(&dir)? {
            return Ok(entries);
        }
        for shard in fs::read_dir(dir).map_err(|_| ProviderCaptureError::Corrupt)? {
            let shard = shard.map_err(|_| ProviderCaptureError::Corrupt)?.path();
            if !regular_dir(&shard) {
                return Err(ProviderCaptureError::Corrupt);
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
                    || metadata
                        .manifest
                        .capture_binding
                        .as_ref()
                        .is_some_and(|binding| binding.binding_schema_version != 1)
                    || path != self.metadata_path(provider, digest)
                {
                    return Err(ProviderCaptureError::Corrupt);
                }
                self.verified_bytes(&metadata)?;
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
        if !directory_exists(&dir)? {
            return Ok(entries);
        }
        for shard in fs::read_dir(dir).map_err(|_| ProviderCaptureError::Corrupt)? {
            let shard = shard.map_err(|_| ProviderCaptureError::Corrupt)?.path();
            if !regular_dir(&shard) {
                return Err(ProviderCaptureError::Corrupt);
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

    fn ensure_directory(&self, path: &Path) -> Result<(), ProviderCaptureError> {
        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        match fs::symlink_metadata(&self.root) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => return Err(ProviderCaptureError::Corrupt),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                fs::create_dir_all(&self.root).map_err(|_| ProviderCaptureError::Corrupt)?;
                if !regular_dir(&self.root) {
                    return Err(ProviderCaptureError::Corrupt);
                }
            }
            Err(_) => return Err(ProviderCaptureError::Corrupt),
        }
        let mut current = self.root.clone();
        for component in relative.components() {
            current.push(component);
            match fs::symlink_metadata(&current) {
                Ok(metadata) if metadata.file_type().is_dir() => {}
                Ok(_) => return Err(ProviderCaptureError::Corrupt),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    fs::create_dir(&current).map_err(|_| ProviderCaptureError::Corrupt)?;
                }
                Err(_) => return Err(ProviderCaptureError::Corrupt),
            }
        }
        Ok(())
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

fn directory_exists(path: &Path) -> Result<bool, ProviderCaptureError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        _ => Err(ProviderCaptureError::Corrupt),
    }
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
    #[cfg(windows)]
    {
        let _ = path;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|_| ProviderCaptureError::Corrupt)
    }
}

fn retained_regular_file_bytes(path: &Path) -> Result<u64, ProviderCaptureError> {
    let mut total = 0;
    for entry in fs::read_dir(path).map_err(|_| ProviderCaptureError::Corrupt)? {
        let entry = entry.map_err(|_| ProviderCaptureError::Corrupt)?;
        let entry_path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        if entry.file_name() == ".lock" {
            continue;
        }
        if file_type.is_file() {
            total += entry
                .metadata()
                .map_err(|_| ProviderCaptureError::Corrupt)?
                .len();
        } else if file_type.is_dir() {
            total += retained_regular_file_bytes(&entry_path)?;
        } else {
            return Err(ProviderCaptureError::Corrupt);
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    use super::{
        MAX_CAPTURE_BYTES, MAX_RETAINED_BYTES, Metadata, ProviderCaptureError,
        ProviderCaptureProvider, ProviderCaptureStore, TestPublicationPause,
    };
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
    fn captures_survive_store_restart_then_detect_corruption() {
        let root = TempDirGuard::new("provider-capture-restart");
        let manifest = ProviderCaptureStore::new(root.path())
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        let restarted = ProviderCaptureStore::new(root.path());
        assert_eq!(
            restarted.read(&manifest.capture_id),
            Ok(b"original".to_vec()),
            "a fresh store instance must read the published capture"
        );
        std::fs::write(
            root.path()
                .join("captures/cspec/sha256")
                .join(&manifest.sha256[..2])
                .join(&manifest.sha256),
            b"changed",
        )
        .expect("corrupt blob");
        assert_eq!(
            restarted.read(&manifest.capture_id),
            Err(ProviderCaptureError::Corrupt)
        );
    }

    #[test]
    fn expires_captures_and_republishes_received_bytes() {
        let root = TempDirGuard::new("provider-capture-expired");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        let metadata_path = root
            .path()
            .join("captures/cspec/metadata")
            .join(&manifest.sha256[..2])
            .join(format!("{}.json", manifest.sha256));
        let mut metadata: Metadata =
            serde_json::from_slice(&std::fs::read(&metadata_path).expect("read metadata"))
                .expect("parse metadata");
        metadata.manifest.expires_at = 0;
        std::fs::write(
            &metadata_path,
            serde_json::to_vec(&metadata).expect("encode metadata"),
        )
        .expect("expire metadata");

        assert_eq!(
            store.read(&manifest.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
        let republished = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("republish expired capture");
        assert_eq!(republished.capture_id, manifest.capture_id);
        assert_eq!(
            store.read(&republished.capture_id),
            Ok(b"original".to_vec())
        );
    }

    #[test]
    fn cspec_capture_is_unavailable_through_other_clingen_provider_prefixes() {
        let root = TempDirGuard::new("provider-capture-cross-source");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"CSpec bytes")
            .expect("capture");

        assert_eq!(
            store.read(&manifest.capture_id),
            Ok(b"CSpec bytes".to_vec())
        );
        for provider in ["car", "erepo", "ldh"] {
            let foreign_handle =
                manifest
                    .capture_id
                    .replacen("capture:cspec:", &format!("capture:{provider}:"), 1);
            assert_eq!(
                store.read(&foreign_handle),
                Err(ProviderCaptureError::UnsupportedProvider),
                "{provider} must not reach CSpec capture bytes"
            );
        }
        assert_eq!(
            store.read(&manifest.capture_id),
            Ok(b"CSpec bytes".to_vec())
        );
    }

    #[test]
    fn rejects_oversize_and_invalid_handles() {
        let root = TempDirGuard::new("provider-capture-bound");
        let store = ProviderCaptureStore::new(root.path());
        assert_eq!(
            store.capture_bytes(
                ProviderCaptureProvider::Cspec,
                "text/plain",
                &vec![0; MAX_CAPTURE_BYTES as usize + 1]
            ),
            Err(ProviderCaptureError::Oversize)
        );
        assert_eq!(store.read("capture:other:sha256:0000000000000000000000000000000000000000000000000000000000000000"), Err(ProviderCaptureError::UnsupportedProvider));
    }

    #[test]
    fn enforces_namespace_capacity_with_deterministic_lru_eviction() {
        let root = TempDirGuard::new("provider-capture-capacity");
        let store = ProviderCaptureStore::new(root.path());
        let body = vec![0; MAX_CAPTURE_BYTES as usize];
        let oldest = store
            .capture_bytes(
                ProviderCaptureProvider::Cspec,
                "application/octet-stream",
                &body,
            )
            .expect("capture oldest");
        let metadata_path = root
            .path()
            .join("captures/cspec/metadata")
            .join(&oldest.sha256[..2])
            .join(format!("{}.json", oldest.sha256));
        let mut metadata: Metadata =
            serde_json::from_slice(&std::fs::read(&metadata_path).expect("read metadata"))
                .expect("parse metadata");
        metadata.last_access_at = 0;
        std::fs::write(
            &metadata_path,
            serde_json::to_vec(&metadata).expect("encode metadata"),
        )
        .expect("age metadata");
        for marker in 1..17u8 {
            let mut body = vec![0; MAX_CAPTURE_BYTES as usize];
            body[0] = marker;
            store
                .capture_bytes(
                    ProviderCaptureProvider::Cspec,
                    "application/octet-stream",
                    &body,
                )
                .expect("capture");
        }

        assert!(store.retained_bytes().expect("retained bytes") <= MAX_RETAINED_BYTES);
        assert_eq!(
            store.read(&oldest.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
    }

    #[test]
    fn retained_bytes_counts_every_owned_regular_file() {
        let root = TempDirGuard::new("provider-capture-retained-files");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"exact bytes")
            .expect("capture");
        let blob = root
            .path()
            .join("captures/cspec/sha256")
            .join(&manifest.sha256[..2])
            .join(&manifest.sha256);
        let metadata = root
            .path()
            .join("captures/cspec/metadata")
            .join(&manifest.sha256[..2])
            .join(format!("{}.json", manifest.sha256));
        let staging = root.path().join("captures/.staging/interrupted.tmp");
        std::fs::write(&staging, b"interrupted publication").expect("write staged bytes");
        let expected = std::fs::metadata(blob).expect("blob metadata").len()
            + std::fs::metadata(metadata).expect("capture metadata").len()
            + std::fs::metadata(staging).expect("staging metadata").len();

        assert_eq!(
            store.retained_bytes().expect("retained bytes"),
            expected,
            "the namespace bound must include every retained regular file except the lock"
        );
    }

    #[test]
    fn concurrent_same_content_captures_publish_one_complete_record() {
        let root = TempDirGuard::new("provider-capture-concurrent");
        let store = Arc::new(ProviderCaptureStore::new(root.path()));
        let handles = (0..8)
            .map(|_| {
                let store = Arc::clone(&store);
                std::thread::spawn(move || {
                    store
                        .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"same")
                        .expect("capture")
                        .capture_id
                })
            })
            .map(|thread| thread.join().expect("capture thread"))
            .collect::<Vec<_>>();

        assert!(handles.windows(2).all(|pair| pair[0] == pair[1]));
        assert_eq!(store.blob_entries().expect("blob entries").len(), 1);
        assert_eq!(store.read(&handles[0]), Ok(b"same".to_vec()));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_capture_shards() {
        let root = TempDirGuard::new("provider-capture-symlink");
        let store = ProviderCaptureStore::new(root.path());
        let outside = root.path().join("outside");
        std::fs::create_dir_all(&outside).expect("outside directory");
        let shard = root.path().join("captures/cspec/sha256/aa");
        std::fs::create_dir_all(shard.parent().expect("shard parent")).expect("create parent");
        std::os::unix::fs::symlink(&outside, &shard).expect("symlink shard");

        assert_eq!(store.retained_bytes(), Err(ProviderCaptureError::Corrupt));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_publish_through_a_symlinked_staging_directory() {
        let root = TempDirGuard::new("provider-capture-staging-symlink");
        let store = ProviderCaptureStore::new(root.path());
        let outside = root.path().join("outside");
        std::fs::create_dir_all(&outside).expect("outside directory");
        let captures = root.path().join("captures");
        std::fs::create_dir_all(&captures).expect("capture root");
        std::os::unix::fs::symlink(&outside, captures.join(".staging")).expect("symlink staging");

        assert_eq!(
            store.capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"bytes"),
            Err(ProviderCaptureError::Corrupt)
        );
        assert!(
            std::fs::read_dir(outside)
                .expect("read outside")
                .next()
                .is_none(),
            "capture must not create files outside its managed root"
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_blob_shard_swapped_after_validation_without_writing_outside() {
        use sha2::{Digest, Sha256};

        let root = TempDirGuard::new("provider-capture-shard-swap");
        let outside = root.path().join("outside");
        std::fs::create_dir_all(&outside).expect("outside directory");
        let bytes = b"swapped bytes";
        let digest = format!("{:x}", Sha256::digest(bytes));
        let shard = root.path().join("captures/cspec/sha256").join(&digest[..2]);
        std::fs::create_dir_all(&shard).expect("create canonical shard");
        let (ready, ready_for_test) = mpsc::channel();
        let (resume_for_test, resume) = mpsc::channel();
        let pause = Arc::new(TestPublicationPause {
            ready,
            resume: Mutex::new(resume),
        });
        let store = ProviderCaptureStore::new(root.path()).with_blob_parent_pause(pause);
        let capture = std::thread::spawn(move || {
            store.capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", bytes)
        });

        ready_for_test
            .recv_timeout(Duration::from_secs(5))
            .expect("capture must pause after validating the blob shard");
        let displaced = root.path().join("displaced-shard");
        std::fs::rename(&shard, &displaced).expect("displace validated shard");
        std::os::unix::fs::symlink(&outside, &shard).expect("replace shard with outside symlink");
        resume_for_test.send(()).expect("resume capture");

        assert_eq!(
            capture.join().expect("capture thread"),
            Err(ProviderCaptureError::Corrupt),
            "a post-validation component swap must fail publication"
        );
        assert!(
            std::fs::read_dir(&outside)
                .expect("read outside")
                .next()
                .is_none(),
            "capture publication must not write through the swapped shard"
        );
    }

    #[test]
    fn refuses_noncanonical_metadata_without_damaging_a_valid_capture() {
        let root = TempDirGuard::new("provider-capture-metadata-path");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        let metadata = root
            .path()
            .join("captures/cspec/metadata")
            .join(&manifest.sha256[..2])
            .join(format!("{}.json", manifest.sha256));
        let duplicate = root
            .path()
            .join("captures/cspec/metadata/ff/duplicate.json");
        std::fs::create_dir_all(duplicate.parent().expect("duplicate parent"))
            .expect("create duplicate parent");
        std::fs::copy(&metadata, &duplicate).expect("copy metadata");

        assert_eq!(store.maintain(), Err(ProviderCaptureError::Corrupt));
        assert_eq!(
            store.read(&manifest.capture_id),
            Err(ProviderCaptureError::Corrupt)
        );
        std::fs::remove_file(duplicate).expect("remove invalid metadata");
        assert_eq!(store.read(&manifest.capture_id), Ok(b"original".to_vec()));
    }

    #[test]
    fn rejects_corrupt_bytes_and_removes_unpublished_orphans() {
        let root = TempDirGuard::new("provider-capture-corrupt");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        let orphan = root.path().join("captures/cspec/sha256/ff/orphan");
        std::fs::create_dir_all(orphan.parent().expect("orphan parent")).expect("create parent");
        std::fs::write(&orphan, b"orphan").expect("write orphan");
        assert!(
            store
                .planned_maintenance_bytes_freed()
                .expect("plan maintenance")
                >= b"orphan".len() as u64
        );
        assert!(orphan.exists(), "maintenance planning must not delete data");
        store.maintain().expect("maintain");
        assert!(!orphan.exists());

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
    }
}
