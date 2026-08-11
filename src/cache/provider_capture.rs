use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::{
    ffi::CString,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    os::unix::fs::MetadataExt,
};

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
    max_retained_bytes: u64,
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
            root: cache_root.as_ref().to_path_buf(),
            max_retained_bytes: MAX_RETAINED_BYTES,
            #[cfg(test)]
            pause_after_blob_parent_ready: None,
        }
    }

    #[cfg(test)]
    fn with_max_retained_bytes(mut self, max_retained_bytes: u64) -> Self {
        assert!(max_retained_bytes > 0, "capture capacity must be nonzero");
        self.max_retained_bytes = max_retained_bytes;
        self
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

    #[cfg(unix)]
    pub(crate) fn capture<R: Read>(
        &self,
        provider: ProviderCaptureProvider,
        media_type: impl Into<String>,
        mut body: R,
        capture_binding: Option<CspecCaptureBinding>,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        #[cfg(not(unix))]
        {
            let _ = (provider, media_type, body, capture_binding);
            return Err(ProviderCaptureError::Corrupt);
        }
        #[cfg(unix)]
        let tree = CaptureTree::open_or_create(&self.root)?;
        #[cfg(unix)]
        let staging_dir = CaptureDirectory::from(tree.directory(".staging", true)?);
        #[cfg(unix)]
        let (mut file, staged) = (0..100)
            .find_map(|attempt| {
                let staged = format!(".capture.{}.{}.tmp", std::process::id(), attempt);
                staging_dir
                    .create_file(&staged)
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
            #[cfg(unix)]
            tree.with_lock(|| {
                if let Ok(existing) = self.read_complete_at(&tree, &manifest.capture_id)
                    && existing.manifest.expires_at > now
                {
                    return if existing.manifest.capture_binding == manifest.capture_binding {
                        Ok(existing.manifest)
                    } else {
                        Err(ProviderCaptureError::BindingConflict)
                    };
                }
                let parent = tree.directory(provider.as_str(), true)?;
                let parent = CaptureDirectory::from(parent).directory("sha256", true)?;
                let shard = parent.directory(&sha256[..2], true)?;
                #[cfg(test)]
                self.pause_after_blob_parent_ready();
                tree.revalidate(&[provider.as_str(), "sha256", &sha256[..2]], &shard)?;
                staging_dir.rename(&staged, &shard, &sha256)?;
                shard.sync()?;
                let record = Metadata {
                    manifest: manifest.clone(),
                    last_access_at: now,
                };
                let metadata_dir = tree.metadata_dir(provider, &sha256, true)?;
                metadata_dir.write_atomic(
                    &format!("{sha256}.json"),
                    &serde_json::to_vec(&record).map_err(|_| ProviderCaptureError::Corrupt)?,
                )?;
                self.maintain_locked_at(&tree, now)?;
                Ok(manifest)
            })
        })();
        #[cfg(unix)]
        let _ = staging_dir.unlink_file(&staged);
        capture_result
    }

    #[cfg(not(unix))]
    pub(crate) fn capture<R: Read>(
        &self,
        _provider: ProviderCaptureProvider,
        _media_type: impl Into<String>,
        _body: R,
        _capture_binding: Option<CspecCaptureBinding>,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        Err(ProviderCaptureError::Corrupt)
    }

    pub(crate) fn read_manifest(
        &self,
        capture_id: &str,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        #[cfg(unix)]
        {
            let tree = CaptureTree::open_or_create(&self.root)?;
            tree.with_lock(|| {
                self.read_complete_at(&tree, capture_id)
                    .map(|record| record.manifest)
            })
        }
        #[cfg(not(unix))]
        Err(ProviderCaptureError::Corrupt)
    }

    pub(crate) fn read(&self, capture_id: &str) -> Result<Vec<u8>, ProviderCaptureError> {
        #[cfg(unix)]
        {
            let tree = CaptureTree::open_or_create(&self.root)?;
            tree.with_lock(|| {
                let mut record = self.read_complete_at(&tree, capture_id)?;
                let now = now_secs()?;
                if record.manifest.expires_at <= now {
                    return Err(ProviderCaptureError::Unavailable);
                }
                record.last_access_at = now;
                let metadata =
                    tree.metadata_dir(record.manifest.provider, &record.manifest.sha256, true)?;
                metadata.write_atomic(
                    &format!("{}.json", record.manifest.sha256),
                    &serde_json::to_vec(&record).map_err(|_| ProviderCaptureError::Corrupt)?,
                )?;
                self.maintain_locked_at(&tree, now)?;
                self.verified_bytes_at(&tree, &record)
            })
        }
        #[cfg(not(unix))]
        Err(ProviderCaptureError::Corrupt)
    }

    pub(crate) fn retained_bytes(&self) -> Result<u64, ProviderCaptureError> {
        #[cfg(unix)]
        {
            let tree = CaptureTree::open_or_create(&self.root)?;
            tree.with_lock(|| tree.retained_regular_file_bytes())
        }
        #[cfg(not(unix))]
        Err(ProviderCaptureError::Corrupt)
    }

    pub(crate) fn maintain(&self) -> Result<u64, ProviderCaptureError> {
        #[cfg(unix)]
        {
            let tree = CaptureTree::open_or_create(&self.root)?;
            tree.with_lock(|| self.maintain_locked_at(&tree, now_secs()?))
        }
        #[cfg(not(unix))]
        Err(ProviderCaptureError::Corrupt)
    }

    pub(crate) fn planned_maintenance_bytes_freed(&self) -> Result<u64, ProviderCaptureError> {
        #[cfg(unix)]
        {
            let tree = CaptureTree::open_or_create(&self.root)?;
            tree.with_lock(|| self.planned_maintenance_bytes_freed_locked_at(&tree, now_secs()?))
        }
        #[cfg(not(unix))]
        Err(ProviderCaptureError::Corrupt)
    }

    #[cfg(unix)]
    fn read_complete_at(
        &self,
        tree: &CaptureTree,
        capture_id: &str,
    ) -> Result<Metadata, ProviderCaptureError> {
        let (provider, digest) = parse_handle(capture_id)?;
        let metadata_dir = tree.metadata_dir(provider, digest, true)?;
        let bytes = metadata_dir
            .read_file(&format!("{digest}.json"))?
            .ok_or(ProviderCaptureError::Unavailable)?;
        let metadata: Metadata =
            serde_json::from_slice(&bytes).map_err(|_| ProviderCaptureError::Corrupt)?;
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
        self.verified_bytes_at(tree, &metadata)?;
        Ok(metadata)
    }

    #[cfg(unix)]
    fn verified_bytes_at(
        &self,
        tree: &CaptureTree,
        record: &Metadata,
    ) -> Result<Vec<u8>, ProviderCaptureError> {
        let blob_dir = tree.blob_dir(record.manifest.provider, &record.manifest.sha256, false)?;
        let blob = blob_dir
            .read_file(&record.manifest.sha256)?
            .ok_or(ProviderCaptureError::Corrupt)?;
        if blob.len() as u64 != record.manifest.byte_length
            || format!("{:x}", Sha256::digest(&blob)) != record.manifest.sha256
        {
            return Err(ProviderCaptureError::Corrupt);
        }
        Ok(blob)
    }

    #[cfg(unix)]
    fn maintain_locked_at(
        &self,
        tree: &CaptureTree,
        now: u64,
    ) -> Result<u64, ProviderCaptureError> {
        let mut freed = self.cleanup_staging_at(tree)?;
        let mut records = self.metadata_entries_at(tree)?;
        for record in &records {
            if record.metadata.manifest.expires_at <= now {
                freed += self.remove_record_at(tree, record)?;
            }
        }
        records = self.metadata_entries_at(tree)?;
        let referenced = records
            .iter()
            .map(|record| record.metadata.manifest.sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for blob in self.blob_entries_at(tree)? {
            if !referenced.contains(blob.digest.as_str()) {
                let provider = CaptureDirectory::from(tree.directory("cspec", false)?);
                let blobs = provider.directory("sha256", false)?;
                blobs
                    .directory(&blob.shard, false)?
                    .unlink_file(&blob.digest)?;
                freed += blob.size;
            }
        }
        let capacity_entries = self.capacity_entries_at(tree, &records)?;
        let retained = tree.retained_regular_file_bytes()?;
        for index in plan_capacity_evictions(&capacity_entries, retained, self.max_retained_bytes) {
            freed += self.remove_record_at(tree, &records[index])?;
        }
        Ok(freed)
    }

    #[cfg(unix)]
    fn planned_maintenance_bytes_freed_locked_at(
        &self,
        tree: &CaptureTree,
        now: u64,
    ) -> Result<u64, ProviderCaptureError> {
        let mut freed = self.staging_bytes_at(tree)?;
        let mut retained_records = Vec::new();
        for record in self.metadata_entries_at(tree)? {
            if record.metadata.manifest.expires_at <= now {
                freed += self.record_regular_file_bytes_at(tree, &record)?;
            } else {
                retained_records.push(record);
            }
        }
        let referenced = retained_records
            .iter()
            .map(|record| record.metadata.manifest.sha256.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for blob in self.blob_entries_at(tree)? {
            if !referenced.contains(blob.digest.as_str()) {
                freed += blob.size;
            }
        }
        let mut retained = tree
            .retained_regular_file_bytes()?
            .checked_sub(freed)
            .ok_or(ProviderCaptureError::Corrupt)?;
        let capacity_entries = self.capacity_entries_at(tree, &retained_records)?;
        for index in plan_capacity_evictions(&capacity_entries, retained, self.max_retained_bytes) {
            let bytes = capacity_entries[index].2;
            retained = retained
                .checked_sub(bytes)
                .ok_or(ProviderCaptureError::Corrupt)?;
            freed += bytes;
        }
        Ok(freed)
    }

    #[cfg(unix)]
    fn cleanup_staging_at(&self, tree: &CaptureTree) -> Result<u64, ProviderCaptureError> {
        let dir = CaptureDirectory::from(tree.directory(".staging", true)?);
        let mut freed = 0;
        for name in dir.entries()? {
            let stat = dir
                .file_status(&name)?
                .ok_or(ProviderCaptureError::Corrupt)?;
            dir.unlink_file(&name)?;
            freed += stat.st_size as u64;
        }
        Ok(freed)
    }

    #[cfg(unix)]
    fn staging_bytes_at(&self, tree: &CaptureTree) -> Result<u64, ProviderCaptureError> {
        let dir = CaptureDirectory::from(tree.directory(".staging", true)?);
        dir.entries()?.into_iter().try_fold(0u64, |total, name| {
            total
                .checked_add(
                    dir.file_status(&name)?
                        .ok_or(ProviderCaptureError::Corrupt)?
                        .st_size as u64,
                )
                .ok_or(ProviderCaptureError::Corrupt)
        })
    }

    #[cfg(unix)]
    fn remove_record_at(
        &self,
        tree: &CaptureTree,
        record: &MetadataEntry,
    ) -> Result<u64, ProviderCaptureError> {
        let digest = &record.metadata.manifest.sha256;
        let freed = self.record_regular_file_bytes_at(tree, record)?;
        tree.metadata_dir(record.provider, digest, false)?
            .unlink_file(&format!("{digest}.json"))?;
        let _ = tree
            .blob_dir(record.provider, digest, false)?
            .unlink_file(digest);
        Ok(freed)
    }

    #[cfg(unix)]
    fn record_regular_file_bytes_at(
        &self,
        tree: &CaptureTree,
        record: &MetadataEntry,
    ) -> Result<u64, ProviderCaptureError> {
        let digest = &record.metadata.manifest.sha256;
        let metadata = tree
            .metadata_dir(record.provider, digest, false)?
            .file_status(&format!("{digest}.json"))?
            .ok_or(ProviderCaptureError::Corrupt)?
            .st_size as u64;
        let blob = tree
            .blob_dir(record.provider, digest, false)?
            .file_status(digest)?
            .map_or(0, |stat| stat.st_size as u64);
        metadata
            .checked_add(blob)
            .ok_or(ProviderCaptureError::Corrupt)
    }

    #[cfg(unix)]
    fn capacity_entries_at(
        &self,
        tree: &CaptureTree,
        records: &[MetadataEntry],
    ) -> Result<Vec<CapacityEntry>, ProviderCaptureError> {
        records
            .iter()
            .map(|record| {
                Ok((
                    record.metadata.last_access_at,
                    record.metadata.manifest.capture_id.clone(),
                    self.record_regular_file_bytes_at(tree, record)?,
                ))
            })
            .collect()
    }

    #[cfg(unix)]
    fn metadata_entries_at(
        &self,
        tree: &CaptureTree,
    ) -> Result<Vec<MetadataEntry>, ProviderCaptureError> {
        let provider = CaptureDirectory::from(tree.directory("cspec", true)?);
        let metadata = provider.directory("metadata", true)?;
        let mut entries = Vec::new();
        for shard_name in metadata.entries()? {
            let shard = metadata.directory(&shard_name, false)?;
            for name in shard.entries()? {
                let record: Metadata = serde_json::from_slice(
                    &shard
                        .read_file(&name)?
                        .ok_or(ProviderCaptureError::Corrupt)?,
                )
                .map_err(|_| ProviderCaptureError::Corrupt)?;
                let (provider, digest) = parse_handle(&record.manifest.capture_id)?;
                if provider != ProviderCaptureProvider::Cspec
                    || record.manifest.provider != provider
                    || record.manifest.sha256 != digest
                    || record.manifest.schema_version != SCHEMA_VERSION
                    || record
                        .manifest
                        .capture_binding
                        .as_ref()
                        .is_some_and(|binding| binding.binding_schema_version != 1)
                    || shard_name != digest[..2]
                    || name != format!("{digest}.json")
                {
                    return Err(ProviderCaptureError::Corrupt);
                }
                self.verified_bytes_at(tree, &record)?;
                entries.push(MetadataEntry {
                    provider,
                    metadata: record,
                });
            }
        }
        Ok(entries)
    }

    #[cfg(unix)]
    fn blob_entries_at(&self, tree: &CaptureTree) -> Result<Vec<BlobEntry>, ProviderCaptureError> {
        let provider = CaptureDirectory::from(tree.directory("cspec", true)?);
        let blobs = provider.directory("sha256", true)?;
        let mut entries = Vec::new();
        for shard_name in blobs.entries()? {
            let shard = blobs.directory(&shard_name, false)?;
            for digest in shard.entries()? {
                let size = shard
                    .file_status(&digest)?
                    .ok_or(ProviderCaptureError::Corrupt)?
                    .st_size as u64;
                entries.push(BlobEntry {
                    digest,
                    shard: shard_name.clone(),
                    size,
                });
            }
        }
        Ok(entries)
    }

    #[cfg(all(test, unix))]
    fn blob_entries(&self) -> Result<Vec<BlobEntry>, ProviderCaptureError> {
        self.blob_entries_at(&CaptureTree::open_or_create(&self.root)?)
    }
}

#[cfg(unix)]
struct CaptureTree {
    root: File,
}

#[cfg(unix)]
struct CaptureDirectory {
    file: File,
}

#[cfg(unix)]
impl From<File> for CaptureDirectory {
    fn from(file: File) -> Self {
        Self { file }
    }
}

#[cfg(unix)]
impl CaptureTree {
    fn open_or_create(path: &Path) -> Result<Self, ProviderCaptureError> {
        let configured_root =
            open_directory(libc::AT_FDCWD, path).map_err(|_| ProviderCaptureError::Corrupt)?;
        let captures = open_or_create_directory(configured_root.as_raw_fd(), "captures", true)?;
        Ok(Self { root: captures })
    }

    fn directory(&self, name: &str, create: bool) -> Result<File, ProviderCaptureError> {
        open_or_create_directory(self.root.as_raw_fd(), name, create)
    }

    fn with_lock<T>(
        &self,
        action: impl FnOnce() -> Result<T, ProviderCaptureError>,
    ) -> Result<T, ProviderCaptureError> {
        let lock = CaptureDirectory::from(
            self.root
                .try_clone()
                .map_err(|_| ProviderCaptureError::Corrupt)?,
        )
        .open_file(".lock", libc::O_RDWR | libc::O_CREAT, 0o600)?;
        lock.lock_exclusive()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        let result = action();
        let _ = lock.unlock();
        result
    }

    fn metadata_dir(
        &self,
        provider: ProviderCaptureProvider,
        digest: &str,
        create: bool,
    ) -> Result<CaptureDirectory, ProviderCaptureError> {
        self.data_dir(provider, "metadata", digest, create)
    }

    fn blob_dir(
        &self,
        provider: ProviderCaptureProvider,
        digest: &str,
        create: bool,
    ) -> Result<CaptureDirectory, ProviderCaptureError> {
        self.data_dir(provider, "sha256", digest, create)
    }

    fn data_dir(
        &self,
        provider: ProviderCaptureProvider,
        kind: &str,
        digest: &str,
        create: bool,
    ) -> Result<CaptureDirectory, ProviderCaptureError> {
        let provider = CaptureDirectory::from(self.directory(provider.as_str(), create)?);
        let kind = provider.directory(kind, create)?;
        kind.directory(&digest[..2], create)
    }

    fn retained_regular_file_bytes(&self) -> Result<u64, ProviderCaptureError> {
        CaptureDirectory::from(
            self.root
                .try_clone()
                .map_err(|_| ProviderCaptureError::Corrupt)?,
        )
        .retained_regular_file_bytes()
    }

    fn revalidate(
        &self,
        components: &[&str],
        held: &CaptureDirectory,
    ) -> Result<(), ProviderCaptureError> {
        let mut current = self
            .root
            .try_clone()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        for component in components {
            current = open_or_create_directory(current.as_raw_fd(), component, false)?;
        }
        let current = current
            .metadata()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        let held = held
            .file
            .metadata()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        if current.dev() == held.dev() && current.ino() == held.ino() {
            Ok(())
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }
}

#[cfg(unix)]
impl CaptureDirectory {
    fn directory(&self, name: &str, create: bool) -> Result<Self, ProviderCaptureError> {
        Ok(Self::from(open_or_create_directory(
            self.file.as_raw_fd(),
            name,
            create,
        )?))
    }

    fn create_file(&self, name: &str) -> Result<File, ProviderCaptureError> {
        self.open_file(name, libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL, 0o600)
    }

    fn open_file(
        &self,
        name: &str,
        flags: libc::c_int,
        mode: libc::mode_t,
    ) -> Result<File, ProviderCaptureError> {
        let name = CString::new(name).map_err(|_| ProviderCaptureError::Corrupt)?;
        // SAFETY: `name` is NUL-terminated and the parent descriptor remains open.
        let fd = unsafe {
            libc::openat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                mode,
            )
        };
        if fd < 0 {
            return Err(ProviderCaptureError::Corrupt);
        }
        // SAFETY: `openat` returned a new owned descriptor.
        let file = File::from(unsafe { OwnedFd::from_raw_fd(fd) });
        if file
            .metadata()
            .map_err(|_| ProviderCaptureError::Corrupt)?
            .is_file()
        {
            Ok(file)
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }

    fn rename(
        &self,
        source: &str,
        destination: &Self,
        target: &str,
    ) -> Result<(), ProviderCaptureError> {
        let source = CString::new(source).map_err(|_| ProviderCaptureError::Corrupt)?;
        let target = CString::new(target).map_err(|_| ProviderCaptureError::Corrupt)?;
        // SAFETY: both names are NUL-terminated and both descriptors remain open.
        if unsafe {
            libc::renameat(
                self.file.as_raw_fd(),
                source.as_ptr(),
                destination.file.as_raw_fd(),
                target.as_ptr(),
            )
        } == 0
        {
            Ok(())
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }

    fn unlink_file(&self, name: &str) -> Result<(), ProviderCaptureError> {
        let name = CString::new(name).map_err(|_| ProviderCaptureError::Corrupt)?;
        // SAFETY: `name` is NUL-terminated and the descriptor remains open.
        if unsafe { libc::unlinkat(self.file.as_raw_fd(), name.as_ptr(), 0) } == 0
            || io::Error::last_os_error().kind() == io::ErrorKind::NotFound
        {
            Ok(())
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }

    fn sync(&self) -> Result<(), ProviderCaptureError> {
        self.file
            .sync_all()
            .map_err(|_| ProviderCaptureError::Corrupt)
    }

    fn read_file(&self, name: &str) -> Result<Option<Vec<u8>>, ProviderCaptureError> {
        match self.open_file(name, libc::O_RDONLY, 0) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)
                    .map_err(|_| ProviderCaptureError::Corrupt)?;
                Ok(Some(bytes))
            }
            Err(ProviderCaptureError::Corrupt) if self.entry_missing(name)? => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn entry_missing(&self, name: &str) -> Result<bool, ProviderCaptureError> {
        let name = CString::new(name).map_err(|_| ProviderCaptureError::Corrupt)?;
        let mut stat = std::mem::MaybeUninit::uninit();
        // SAFETY: `name` is NUL-terminated and `stat` points to writable storage.
        if unsafe {
            libc::fstatat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
        {
            Ok(false)
        } else if io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
            Ok(true)
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }

    fn file_status(&self, name: &str) -> Result<Option<libc::stat>, ProviderCaptureError> {
        let name = CString::new(name).map_err(|_| ProviderCaptureError::Corrupt)?;
        let mut stat = std::mem::MaybeUninit::uninit();
        // SAFETY: `name` is NUL-terminated and `stat` points to writable storage.
        if unsafe {
            libc::fstatat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } == 0
        {
            // SAFETY: fstatat initialized stat after returning zero.
            let stat = unsafe { stat.assume_init() };
            match stat.st_mode & libc::S_IFMT {
                libc::S_IFREG => Ok(Some(stat)),
                libc::S_IFDIR => Ok(None),
                _ => Err(ProviderCaptureError::Corrupt),
            }
        } else if io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(ProviderCaptureError::Corrupt)
        }
    }

    fn write_atomic(&self, name: &str, bytes: &[u8]) -> Result<(), ProviderCaptureError> {
        let temporary = format!(".{name}.{}.tmp", std::process::id());
        let result = (|| {
            let mut file = self.create_file(&temporary)?;
            file.write_all(bytes)
                .map_err(|_| ProviderCaptureError::Corrupt)?;
            file.sync_all().map_err(|_| ProviderCaptureError::Corrupt)?;
            self.rename(&temporary, self, name)?;
            self.sync()
        })();
        let _ = self.unlink_file(&temporary);
        result
    }

    fn entries(&self) -> Result<Vec<String>, ProviderCaptureError> {
        // SAFETY: dup returns an independent descriptor, consumed by fdopendir on success.
        let duplicate = unsafe { libc::dup(self.file.as_raw_fd()) };
        if duplicate < 0 {
            return Err(ProviderCaptureError::Corrupt);
        }
        // SAFETY: duplicate is a valid owned directory descriptor.
        let directory = unsafe { libc::fdopendir(duplicate) };
        if directory.is_null() {
            // SAFETY: fdopendir did not consume the descriptor on failure.
            unsafe { libc::close(duplicate) };
            return Err(ProviderCaptureError::Corrupt);
        }
        let mut entries = Vec::new();
        loop {
            // SAFETY: errno storage is thread-local and readdir reports errors through it.
            unsafe { *errno_location() = 0 };
            // SAFETY: directory is valid until closed below; readdir returns a borrowed entry.
            let entry = unsafe { libc::readdir(directory) };
            if entry.is_null() {
                // SAFETY: errno storage is valid for this thread.
                if unsafe { *errno_location() } != 0 {
                    // SAFETY: closes directory and its owned duplicate descriptor.
                    unsafe { libc::closedir(directory) };
                    return Err(ProviderCaptureError::Corrupt);
                }
                break;
            }
            // SAFETY: d_name is NUL-terminated by readdir.
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
            let name = name.to_str().map_err(|_| ProviderCaptureError::Corrupt)?;
            if name != "." && name != ".." {
                entries.push(name.to_owned());
            }
        }
        // SAFETY: closes directory and its owned duplicate descriptor.
        if unsafe { libc::closedir(directory) } != 0 {
            return Err(ProviderCaptureError::Corrupt);
        }
        Ok(entries)
    }

    fn retained_regular_file_bytes(&self) -> Result<u64, ProviderCaptureError> {
        self.entries()?.into_iter().try_fold(0u64, |total, name| {
            if name == ".lock" {
                return Ok(total);
            }
            if let Some(stat) = self.file_status(&name)? {
                return total
                    .checked_add(stat.st_size as u64)
                    .ok_or(ProviderCaptureError::Corrupt);
            }
            let child = self.directory(&name, false)?;
            total
                .checked_add(child.retained_regular_file_bytes()?)
                .ok_or(ProviderCaptureError::Corrupt)
        })
    }
}

#[cfg(unix)]
fn errno_location() -> *mut libc::c_int {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    unsafe {
        libc::__errno_location()
    }
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    unsafe {
        libc::__error()
    }
}

#[cfg(unix)]
fn open_directory(parent: libc::c_int, path: &Path) -> Result<File, io::Error> {
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    // SAFETY: `path` is NUL-terminated; the resulting descriptor is owned below.
    let fd = unsafe {
        libc::openat(
            parent,
            path.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `openat` returned a new owned descriptor.
    Ok(File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}

#[cfg(unix)]
fn open_or_create_directory(
    parent: libc::c_int,
    name: &str,
    create: bool,
) -> Result<File, ProviderCaptureError> {
    let name = CString::new(name).map_err(|_| ProviderCaptureError::Corrupt)?;
    // SAFETY: `name` is NUL-terminated and the parent descriptor remains open.
    let mut fd = unsafe {
        libc::openat(
            parent,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 && create && io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
        // SAFETY: `name` is NUL-terminated and the parent descriptor remains open.
        if unsafe { libc::mkdirat(parent, name.as_ptr(), 0o700) } != 0
            && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists
        {
            return Err(ProviderCaptureError::Corrupt);
        }
        // SAFETY: as above.
        fd = unsafe {
            libc::openat(
                parent,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
    }
    if fd < 0 {
        return Err(ProviderCaptureError::Corrupt);
    }
    // SAFETY: `openat` returned a new owned descriptor.
    Ok(File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}

struct MetadataEntry {
    provider: ProviderCaptureProvider,
    metadata: Metadata,
}

type CapacityEntry = (u64, String, u64);

fn plan_capacity_evictions(
    entries: &[CapacityEntry],
    mut retained_bytes: u64,
    max_retained_bytes: u64,
) -> Vec<usize> {
    let mut oldest = (0..entries.len()).collect::<Vec<_>>();
    oldest.sort_by_key(|index| {
        let entry = &entries[*index];
        (entry.0, entry.1.as_str())
    });

    oldest
        .into_iter()
        .take_while(|index| {
            if retained_bytes <= max_retained_bytes {
                return false;
            }
            retained_bytes = retained_bytes.saturating_sub(entries[*index].2);
            true
        })
        .collect()
}
struct BlobEntry {
    digest: String,
    shard: String,
    size: u64,
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    use super::{
        MAX_CAPTURE_BYTES, Metadata, ProviderCaptureError, ProviderCaptureProvider,
        ProviderCaptureStore, TestPublicationPause, plan_capacity_evictions,
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

    #[cfg(unix)]
    #[test]
    fn rejects_a_directory_in_place_of_capture_metadata() {
        let root = TempDirGuard::new("provider-capture-metadata-directory");
        let store = ProviderCaptureStore::new(root.path());
        let manifest = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"original")
            .expect("capture");
        let metadata = root
            .path()
            .join("captures/cspec/metadata")
            .join(&manifest.sha256[..2])
            .join(format!("{}.json", manifest.sha256));
        std::fs::remove_file(&metadata).expect("remove metadata");
        std::fs::create_dir(&metadata).expect("replace metadata with directory");

        assert_eq!(
            store.read_manifest(&manifest.capture_id),
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
        let store = ProviderCaptureStore::new(root.path()).with_max_retained_bytes(64 * 1024);
        let body = vec![0; 1024];
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
        for marker in 1..8u8 {
            let mut body = vec![0; 1024];
            body[0] = marker;
            store
                .capture_bytes(
                    ProviderCaptureProvider::Cspec,
                    "application/octet-stream",
                    &body,
                )
                .expect("capture");
        }

        let restarted = ProviderCaptureStore::new(root.path()).with_max_retained_bytes(6 * 1024);
        restarted.maintain().expect("maintain after restart");
        assert!(restarted.retained_bytes().expect("retained bytes") <= 6 * 1024);
        assert_eq!(
            restarted.read(&oldest.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
    }

    #[test]
    fn capacity_planner_covers_exact_plus_one_and_stable_ties() {
        let entries = [(1, "capture-b".into(), 4), (1, "capture-a".into(), 4)];

        assert!(plan_capacity_evictions(&entries, 8, 8).is_empty());
        assert_eq!(plan_capacity_evictions(&entries, 9, 8), vec![1]);
        assert_eq!(plan_capacity_evictions(&entries, 12, 4), vec![1, 0]);
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
