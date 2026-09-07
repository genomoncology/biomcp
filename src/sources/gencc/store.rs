//! Private immutable generations and mutable GenCC lifecycle state.
use std::collections::HashMap;
#[cfg(not(unix))]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use chrono::DateTime;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use super::model::GenCcDataset;
const STATE_SCHEMA: u32 = 1;
static GENERATION_LEASES: OnceLock<Mutex<HashMap<PathBuf, Weak<File>>>> = OnceLock::new();
#[cfg(test)]
#[rustfmt::skip]
pub(crate) const PUBLICATION_CRASH_POINTS: [&str; 36] = [
    "before-anchor-file-fsync", "after-anchor-file-fsync", "before-anchor-parent-fsync", "after-anchor-parent-fsync", "before-refresh-lock-file-fsync", "after-refresh-lock-file-fsync",
    "before-refresh-lock-parent-fsync", "after-refresh-lock-parent-fsync", "before-store-lock-file-fsync", "after-store-lock-file-fsync", "before-store-lock-parent-fsync", "after-store-lock-parent-fsync",
    "before-raw-file-fsync", "after-raw-file-fsync", "before-raw-parent-fsync", "after-raw-parent-fsync",
    "before-temporary-generations-parent-fsync", "after-temporary-generations-parent-fsync", "before-index-file-fsync", "after-index-file-fsync", "before-lease-file-fsync", "after-lease-file-fsync", "before-manifest-file-fsync", "after-manifest-file-fsync", "before-generation-directory-fsync", "after-generation-directory-fsync", "before-generation-rename", "after-generation-rename",
    "before-generations-directory-fsync", "after-generations-directory-fsync", "before-state-file-fsync", "after-state-file-fsync", "before-state-rename", "after-state-rename", "before-root-directory-fsync", "after-root-directory-fsync",
];
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[rustfmt::skip]
pub(crate) enum StoreError { #[error("GenCC store is unavailable")] Unavailable, #[error("GenCC store data is invalid")] Invalid, #[error("GenCC store update failed after namespace publication")] PostRenameSync, #[error("GenCC store lock deadline expired")] Deadline }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[rustfmt::skip]
pub(crate) struct Manifest { pub schema_version: u32, pub endpoint: String, pub body_sha256: String, pub index_sha256: String, pub row_count: usize, pub assertion_count: usize, pub retrieved_at: String, pub etag: String, pub last_modified: String, pub upstream_version: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[rustfmt::skip]
pub(crate) struct State { pub schema_version: u32, pub active_generation: Option<String>, pub checked_at: Option<String>, pub attempted_at: Option<String>, pub last_attempt: Option<Attempt> }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Attempt {
    Success200,
    Success304,
    Failure,
}
#[rustfmt::skip]
impl Default for State {
    fn default() -> Self {
        Self { schema_version: STATE_SCHEMA, active_generation: None, checked_at: None, attempted_at: None, last_attempt: None }
    }
}
#[derive(Debug, Clone)]
#[rustfmt::skip]
pub(crate) struct Snapshot { pub state: State, pub manifest: Manifest, pub dataset: GenCcDataset, pub(crate) lease: Arc<File> }
#[rustfmt::skip]
pub(crate) struct PublishMetadata<'a> { pub now: &'a str, pub etag: &'a str, pub last_modified: &'a str, pub endpoint: &'a str, pub body_sha256: &'a str, pub row_count: usize }
#[rustfmt::skip]
pub(crate) struct RawCsvTemp { path: PathBuf, parent: File, name: String, file: File, hasher: Sha256, len: usize }
#[rustfmt::skip]
struct OwnedTemporary { path: PathBuf, parent: File, name: String, directory: bool, armed: bool }
#[rustfmt::skip]
impl OwnedTemporary {
    fn new(path: PathBuf, parent: &File, name: String, directory: bool) -> Result<Self, StoreError> {
        Ok(Self { path, parent: parent.try_clone().map_err(|_| StoreError::Unavailable)?, name, directory, armed: true })
    }
    fn disarm(&mut self) { self.armed = false; }
}
#[rustfmt::skip]
impl Drop for OwnedTemporary {
    fn drop(&mut self) {
        if !self.armed { return; }
        #[cfg(unix)]
        let result = if self.directory { remove_owned_dir_at(&self.parent, &self.name) } else { unlink_file_at(&self.parent, &self.name) };
        #[cfg(not(unix))]
        let result = if self.directory { fs::remove_dir_all(&self.path) } else { fs::remove_file(&self.path) };
        if let Err(error) = result { tracing::warn!(path = %self.path.display(), %error, "failed to remove owned GenCC temporary"); }
        if let Err(error) = self.parent.sync_all() { tracing::warn!(path = %self.path.display(), %error, "failed to sync GenCC temporary parent"); }
    }
}
#[rustfmt::skip]
impl RawCsvTemp {
    pub(crate) fn write_chunk(&mut self, bytes: &[u8], max: usize) -> Result<(), StoreError> {
        self.len = self.len.checked_add(bytes.len()).filter(|len| *len <= max).ok_or(StoreError::Invalid)?;
        self.file.write_all(bytes).map_err(|_| StoreError::Unavailable)?;
        self.hasher.update(bytes);
        Ok(())
    }
    pub(crate) fn finish(&mut self) -> Result<(Vec<u8>, String), StoreError> {
        injected("before-raw-file-fsync", StoreError::Unavailable)?;
        self.file.sync_all().map_err(|_| StoreError::Unavailable)?;
        injected("after-raw-file-fsync", StoreError::Unavailable)?;
        let mut reader = self.file.try_clone().map_err(|_| StoreError::Unavailable)?;
        reader.seek(SeekFrom::Start(0)).map_err(|_| StoreError::Unavailable)?;
        let mut bytes = Vec::with_capacity(self.len);
        reader.read_to_end(&mut bytes).map_err(|_| StoreError::Unavailable)?;
        Ok((bytes, format!("{:x}", self.hasher.clone().finalize())))
    }
}
#[rustfmt::skip]
impl Drop for RawCsvTemp {
    fn drop(&mut self) {
        #[cfg(unix)] let result = unlink_file_at(&self.parent, &self.name);
        #[cfg(not(unix))] let result = fs::remove_file(&self.path).map_err(|_| StoreError::Unavailable);
        if let Err(error) = result { tracing::warn!(path = %self.path.display(), %error, "failed to remove owned GenCC raw temporary"); }
        if let Err(error) = sync_file_injected(&self.parent, "raw-parent", StoreError::Unavailable) { tracing::warn!(path = %self.path.display(), %error, "failed to sync GenCC raw temporary parent"); }
    }
}
#[rustfmt::skip]
pub(crate) struct Store { root: PathBuf, root_dir: File, generations_dir: File, refresh_lock: File, store_lock: File, deadline: std::time::Instant }
impl Store {
    #[cfg(test)]
    pub(crate) fn open() -> Result<Self, StoreError> {
        Self::open_until(std::time::Instant::now() + std::time::Duration::from_secs(2))
    }
    #[rustfmt::skip]
    pub(crate) fn open_until(deadline: std::time::Instant) -> Result<Self, StoreError> {
        let (root, anchor_dir) = selected_root()?;
        #[cfg(unix)]
        return Self::open_until_unix(root, anchor_dir, deadline);
        #[cfg(not(unix))]
        {
            validate_path_components(&anchor_dir)?;
            let anchor_name = format!(".biomcp-gencc-root-{}.lock", hex_sha256(root.as_os_str().as_encoded_bytes()));
            let anchor = open_private_file(&anchor_dir.join(anchor_name))?;
            validate_open_identity(&anchor, &anchor_dir.join(format!(".biomcp-gencc-root-{}.lock", hex_sha256(root.as_os_str().as_encoded_bytes()))))?;
            lock_exclusive_until(&anchor, deadline)?;
            if !root.exists() {
                if let Some(parent) = root.parent() && !parent.exists() && parent.parent() == Some(anchor_dir.as_path()) {
                    fs::create_dir(parent).map_err(|_| StoreError::Unavailable)?;
                    set_private_dir(parent)?;
                    sync_dir(&anchor_dir)?;
                }
                validate_path_components(root.parent().ok_or(StoreError::Unavailable)?)?;
                fs::create_dir(&root).map_err(|_| StoreError::Unavailable)?;
                set_private_dir(&root)?;
                sync_dir(root.parent().ok_or(StoreError::Unavailable)?)?;
            }
            validate_path_components(&root)?;
            set_private_dir(&root)?;
            let refresh_lock = open_private_file(&root.join(".refresh.lock"))?;
            let store_lock = open_private_file(&root.join(".store.lock"))?;
            let generations = root.join("generations");
            if !generations.exists() {
                fs::create_dir(&generations).map_err(|_| StoreError::Unavailable)?;
                set_private_dir(&generations)?;
                sync_dir(&root)?;
            }
            validate_existing_directory(&generations)?;
            set_private_dir(&generations)?;
            lock_shared_until(&store_lock, deadline)?;
            FileExt::unlock(&anchor).map_err(|_| StoreError::Unavailable)?;
            FileExt::unlock(&store_lock).map_err(|_| StoreError::Unavailable)?;
            Ok(Self { root_dir: File::open(&root).map_err(|_| StoreError::Unavailable)?, generations_dir: File::open(&generations).map_err(|_| StoreError::Unavailable)?, root, refresh_lock, store_lock, deadline })
        }
    }
    #[cfg(unix)]
    #[rustfmt::skip]
    fn open_until_unix(root: PathBuf, anchor_path: PathBuf, deadline: std::time::Instant) -> Result<Self, StoreError> {
        let anchor_dir = open_secure_directory_chain(&anchor_path, true)?;
        let anchor_name = format!(".biomcp-gencc-root-{}.lock", hex_sha256(root.as_os_str().as_encoded_bytes()));
        let anchor = open_private_at(&anchor_dir, &anchor_name)?;
        lock_exclusive_until(&anchor, deadline)?;
        let relative = root.strip_prefix(&anchor_path).map_err(|_| StoreError::Unavailable)?;
        let mut root_dir = anchor_dir.try_clone().map_err(|_| StoreError::Unavailable)?;
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(StoreError::Unavailable);
            };
            root_dir = open_or_create_private_directory_at(&root_dir, name)?;
        }
        let refresh_lock = open_private_at(&root_dir, ".refresh.lock")?;
        let store_lock = open_private_at(&root_dir, ".store.lock")?;
        let generations_dir = open_or_create_private_directory_at(&root_dir, "generations".as_ref())?;
        lock_shared_until(&store_lock, deadline)?;
        FileExt::unlock(&anchor).map_err(|_| StoreError::Unavailable)?;
        FileExt::unlock(&store_lock).map_err(|_| StoreError::Unavailable)?;
        Ok(Self { root, root_dir, generations_dir, refresh_lock, store_lock, deadline })
    }
    pub(crate) fn try_lock_refresh(&self) -> Result<bool, StoreError> {
        match FileExt::try_lock_exclusive(&self.refresh_lock) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
            Err(_) => Err(StoreError::Unavailable),
        }
    }
    pub(crate) fn unlock_refresh(&self) {
        let _ = FileExt::unlock(&self.refresh_lock);
    }
    pub(crate) fn create_raw_temp(&self) -> Result<RawCsvTemp, StoreError> {
        self.revalidate_root()?;
        let name = format!(".raw-{}.tmp", unique_suffix()?);
        let path = self.root.join(&name);
        #[cfg(unix)]
        let file = create_file_at(&self.root_dir, &name)?;
        #[cfg(not(unix))]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| StoreError::Unavailable)?;
        #[cfg(not(unix))]
        if set_private_file(&path).is_err() {
            let _ = fs::remove_file(&path);
            return Err(StoreError::Unavailable);
        }
        Ok(RawCsvTemp {
            path,
            parent: self
                .root_dir
                .try_clone()
                .map_err(|_| StoreError::Unavailable)?,
            name,
            file,
            hasher: Sha256::new(),
            len: 0,
        })
    }
    #[rustfmt::skip]
    pub(crate) fn cleanup_abandoned(&self) {
        if self.revalidate_root().is_err() || self.revalidate_generations().is_err() { return; }
        if lock_exclusive_until(&self.store_lock, self.deadline).is_err() { return; }
        let mut root_changed = false;
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !(name.starts_with(".raw-") || name.starts_with(".state-")) || !name.ends_with(".tmp") { continue; }
                if entry.file_type().is_ok_and(|kind| kind.is_file() && !kind.is_symlink()) {
                    let result = injected("before-abandoned-root-delete", StoreError::Unavailable)
                        .and_then(|()| remove_file_owned(&self.root_dir, &self.root, &name))
                        .and_then(|()| injected("after-abandoned-root-delete", StoreError::Unavailable));
                    match result {
                        Ok(()) => root_changed = true,
                        Err(error) => tracing::warn!(%error, "GenCC cleanup retained root temporary"),
                    }
                }
            }
        }
        if root_changed && let Err(error) = sync_file_injected(&self.root_dir, "abandoned-root-directory", StoreError::Unavailable) {
            tracing::warn!(%error, "GenCC cleanup could not sync root temporary deletion");
        }
        let generations = self.root.join("generations");
        let mut generations_changed = false;
        if let Ok(entries) = fs::read_dir(&generations) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if !name.starts_with(".tmp-") { continue; }
                if entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) {
                    let result = injected("before-abandoned-generation-delete", StoreError::Unavailable)
                        .and_then(|()| remove_dir_owned(&self.generations_dir, &generations, &name))
                        .and_then(|()| injected("after-abandoned-generation-delete", StoreError::Unavailable));
                    match result {
                        Ok(()) => generations_changed = true,
                        Err(error) => tracing::warn!(%error, "GenCC cleanup retained generation temporary"),
                    }
                }
            }
        }
        if generations_changed && let Err(error) = sync_file_injected(&self.generations_dir, "abandoned-generations-directory", StoreError::Unavailable) {
            tracing::warn!(%error, "GenCC cleanup could not sync generation deletion");
        }
        let _ = FileExt::unlock(&self.store_lock);
    }
    pub(crate) fn load(&self) -> Result<Option<Snapshot>, StoreError> {
        self.revalidate_root()?;
        lock_shared_until(&self.store_lock, self.deadline)?;
        let result = self.load_authoritative_locked();
        let _ = FileExt::unlock(&self.store_lock);
        if result.is_ok() {
            return result;
        }
        lock_exclusive_until(&self.store_lock, self.deadline)?;
        let result = match self.load_authoritative_locked() {
            Ok(snapshot) => Ok(snapshot),
            Err(_) => self.recover_locked(),
        };
        let _ = FileExt::unlock(&self.store_lock);
        result
    }
    pub(crate) fn load_state(&self) -> Result<State, StoreError> {
        self.revalidate_root()?;
        lock_shared_until(&self.store_lock, self.deadline)?;
        let path = self.root.join("state.json");
        let result = if path.exists() {
            #[cfg(unix)]
            let bytes = read_at(&self.root_dir, "state.json")?;
            #[cfg(not(unix))]
            let bytes = read_regular(&path)?;
            let state: State = serde_json::from_slice(&bytes).map_err(|_| StoreError::Invalid)?;
            (state.schema_version == STATE_SCHEMA)
                .then_some(state)
                .ok_or(StoreError::Invalid)
        } else {
            Ok(State::default())
        };
        let _ = FileExt::unlock(&self.store_lock);
        result
    }
    fn load_authoritative_locked(&self) -> Result<Option<Snapshot>, StoreError> {
        let state_path = self.root.join("state.json");
        if !state_path.exists() {
            return Err(StoreError::Invalid);
        }
        #[cfg(unix)]
        let state_bytes = read_at(&self.root_dir, "state.json")?;
        #[cfg(not(unix))]
        let state_bytes = read_regular(&state_path)?;
        let state: State = serde_json::from_slice(&state_bytes).map_err(|_| StoreError::Invalid)?;
        if state.schema_version != STATE_SCHEMA {
            return Err(StoreError::Invalid);
        }
        let Some(generation) = state.active_generation.clone() else {
            return Ok(None);
        };
        self.load_generation(&generation, state).map(Some)
    }
    #[rustfmt::skip]
    fn load_generation(&self, generation: &str, state: State) -> Result<Snapshot, StoreError> {
        if !safe_generation_id(generation) { return Err(StoreError::Invalid); }
        self.revalidate_generations()?;
        let directory = self.root.join("generations").join(generation);
        #[cfg(unix)]
        let directory_handle = open_directory_at(&self.generations_dir, generation.as_ref())?;
        #[cfg(unix)]
        validate_directory_owner_mode(&directory_handle, true)?;
        #[cfg(not(unix))]
        validate_existing_directory(&directory)?;
        #[cfg(unix)]
        let lease = acquire_generation_lease_at(&directory_handle, &directory.join("lease.lock"), self.deadline)?;
        #[cfg(not(unix))]
        let lease = acquire_generation_lease(&directory.join("lease.lock"), self.deadline)?;
        #[cfg(unix)]
        let manifest_bytes = read_at(&directory_handle, "manifest.json")?;
        #[cfg(not(unix))]
        let manifest_bytes = read_regular(&directory.join("manifest.json"))?;
        #[cfg(unix)]
        let index_bytes = read_at(&directory_handle, "index.json")?;
        #[cfg(not(unix))]
        let index_bytes = read_regular(&directory.join("index.json"))?;
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes).map_err(|_| StoreError::Invalid)?;
        if manifest.schema_version != STATE_SCHEMA
            || manifest.endpoint != super::ENDPOINT
            || manifest.upstream_version.is_some()
            || manifest.row_count > 100_000
            || manifest.assertion_count > manifest.row_count
            || manifest.body_sha256.len() != 64
            || !manifest.body_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || manifest.index_sha256 != hex_sha256(&index_bytes)
            || !super::valid_etag(&manifest.etag)
            || !super::valid_http_date(&manifest.last_modified)
            || DateTime::parse_from_rfc3339(&manifest.retrieved_at).is_err()
        {
            return Err(StoreError::Invalid);
        }
        let dataset = serde_json::from_slice::<GenCcDataset>(&index_bytes).map_err(|_| StoreError::Invalid)?;
        if dataset.assertions().len() != manifest.assertion_count || dataset.row_count() != manifest.row_count { return Err(StoreError::Invalid); }
        Ok(Snapshot { state, manifest, dataset, lease })
    }
    fn recover_locked(&self) -> Result<Option<Snapshot>, StoreError> {
        let generations = self.root.join("generations");
        let mut candidates = Vec::new();
        for entry in fs::read_dir(&generations).map_err(|_| StoreError::Unavailable)? {
            let entry = entry.map_err(|_| StoreError::Unavailable)?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            let state = State {
                active_generation: Some(name.to_string()),
                ..State::default()
            };
            if let Ok(snapshot) = self.load_generation(name, state) {
                candidates.push(snapshot);
            }
        }
        candidates.sort_by(|left, right| {
            right
                .manifest
                .retrieved_at
                .cmp(&left.manifest.retrieved_at)
                .then_with(|| {
                    right
                        .state
                        .active_generation
                        .cmp(&left.state.active_generation)
                })
        });
        let Some(mut snapshot) = candidates.into_iter().next() else {
            return Ok(None);
        };
        snapshot.state.checked_at = Some(snapshot.manifest.retrieved_at.clone());
        snapshot.state.attempted_at = Some(snapshot.manifest.retrieved_at.clone());
        snapshot.state.last_attempt = Some(Attempt::Success200);
        self.replace_state_locked(&snapshot.state)?;
        if let Err(error) = self.cleanup_locked(snapshot.state.active_generation.as_deref()) {
            tracing::warn!(%error, "GenCC recovery cleanup retained extra files");
        }
        Ok(Some(snapshot))
    }
    #[rustfmt::skip]
    pub(crate) fn publish(&self, dataset: &GenCcDataset, metadata: PublishMetadata<'_>) -> Result<Snapshot, StoreError> {
        self.revalidate_root()?;
        self.revalidate_generations()?;
        let index = serde_json::to_vec(dataset).map_err(|_| StoreError::Invalid)?;
        let index_hash = hex_sha256(&index);
        let suffix = unique_suffix()?;
        let generation = format!("{}-{suffix}", &index_hash[..24]);
        let temporary_name = format!(".tmp-{generation}");
        let temporary = self.root.join("generations").join(&temporary_name);
        #[cfg(unix)]
        let temporary_dir = create_directory_at(&self.generations_dir, &temporary_name)?;
        #[cfg(not(unix))]
        create_private_dir(&temporary)?;
        let mut owned_temporary = OwnedTemporary::new(temporary.clone(), &self.generations_dir, temporary_name.clone(), true)?;
        injected("before-temporary-generations-parent-fsync", StoreError::Unavailable)?;
        self.generations_dir.sync_all().map_err(|_| StoreError::Unavailable)?;
        injected("after-temporary-generations-parent-fsync", StoreError::Unavailable)?;
        #[cfg(unix)]
        write_new_at(&temporary_dir, "index.json", &index, "index-file")?;
        #[cfg(not(unix))]
        write_new_synced(&temporary.join("index.json"), &index, "index-file")?;
        #[cfg(unix)]
        write_new_at(&temporary_dir, "lease.lock", b"", "lease-file")?;
        #[cfg(not(unix))]
        write_new_synced(&temporary.join("lease.lock"), b"", "lease-file")?;
        let manifest = Manifest { schema_version: STATE_SCHEMA, endpoint: metadata.endpoint.to_string(), body_sha256: metadata.body_sha256.to_string(), index_sha256: index_hash, row_count: metadata.row_count, assertion_count: dataset.assertions().len(), retrieved_at: metadata.now.to_string(), etag: metadata.etag.to_string(), last_modified: metadata.last_modified.to_string(), upstream_version: None };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(|_| StoreError::Invalid)?;
        #[cfg(unix)]
        write_new_at(&temporary_dir, "manifest.json", &manifest_bytes, "manifest-file")?;
        #[cfg(not(unix))]
        write_new_synced(&temporary.join("manifest.json"), &manifest_bytes, "manifest-file")?;
        injected("before-generation-directory-fsync", StoreError::Unavailable)?;
        #[cfg(unix)]
        temporary_dir.sync_all().map_err(|_| StoreError::Unavailable)?;
        #[cfg(not(unix))]
        sync_dir(&temporary)?;
        injected("after-generation-directory-fsync", StoreError::Unavailable)?;
        lock_exclusive_until(&self.store_lock, self.deadline)?;
        #[cfg(not(unix))]
        let final_path = self.root.join("generations").join(&generation);
        let result = (|| {
            injected("before-generation-rename", StoreError::Unavailable)?;
            #[cfg(unix)]
            rename_at(&self.generations_dir, &temporary_name, &generation)?;
            #[cfg(not(unix))]
            fs::rename(&temporary, &final_path).map_err(|_| StoreError::Unavailable)?;
            owned_temporary.disarm();
            injected("after-generation-rename", StoreError::Unavailable)?;
            injected("before-generations-directory-fsync", StoreError::Unavailable)?;
            self.generations_dir.sync_all().map_err(|_| StoreError::Unavailable)?;
            injected("after-generations-directory-fsync", StoreError::Unavailable)?;
            let state = State { active_generation: Some(generation.clone()), checked_at: Some(metadata.now.to_string()), attempted_at: Some(metadata.now.to_string()), last_attempt: Some(Attempt::Success200), ..State::default() };
            self.replace_state_locked(&state)?;
            let snapshot = self.load_generation(&generation, state)?;
            if let Err(error) = self.cleanup_locked(Some(&generation)) {
                tracing::warn!(%error, "GenCC publication cleanup retained extra files");
            }
            Ok(snapshot)
        })();
        let _ = FileExt::unlock(&self.store_lock);
        result
    }
    pub(crate) fn record_304(&self, mut state: State, now: &str) -> Result<State, StoreError> {
        state.checked_at = Some(now.to_string());
        state.attempted_at = Some(now.to_string());
        state.last_attempt = Some(Attempt::Success304);
        self.replace_state(&state)?;
        Ok(state)
    }
    pub(crate) fn record_failure(&self, mut state: State, now: &str) -> Result<State, StoreError> {
        state.attempted_at = Some(now.to_string());
        state.last_attempt = Some(Attempt::Failure);
        self.replace_state(&state)?;
        Ok(state)
    }
    fn replace_state(&self, state: &State) -> Result<(), StoreError> {
        self.revalidate_root()?;
        lock_exclusive_until(&self.store_lock, self.deadline)?;
        let result = self.replace_state_locked(state);
        let _ = FileExt::unlock(&self.store_lock);
        result
    }
    fn replace_state_locked(&self, state: &State) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(state).map_err(|_| StoreError::Invalid)?;
        let temporary_name = format!(".state-{}.tmp", unique_suffix()?);
        let temporary = self.root.join(&temporary_name);
        let mut owned_temporary = OwnedTemporary::new(
            temporary.clone(),
            &self.root_dir,
            temporary_name.clone(),
            false,
        )?;
        #[cfg(unix)]
        write_new_at(&self.root_dir, &temporary_name, &bytes, "state-file")?;
        #[cfg(not(unix))]
        write_new_synced(&temporary, &bytes, "state-file")?;
        injected("before-state-rename", StoreError::Unavailable)?;
        #[cfg(unix)]
        rename_at(&self.root_dir, &temporary_name, "state.json")?;
        #[cfg(not(unix))]
        fs::rename(&temporary, self.root.join("state.json"))
            .map_err(|_| StoreError::Unavailable)?;
        owned_temporary.disarm();
        injected("after-state-rename", StoreError::PostRenameSync)?;
        injected("before-root-directory-fsync", StoreError::PostRenameSync)?;
        self.root_dir
            .sync_all()
            .map_err(|_| StoreError::PostRenameSync)?;
        injected("after-root-directory-fsync", StoreError::PostRenameSync)
    }
    #[rustfmt::skip]
    fn cleanup_locked(&self, active: Option<&str>) -> Result<(), StoreError> {
        self.revalidate_generations()?;
        let generations = self.root.join("generations");
        let mut valid = Vec::new();
        let mut invalid = Vec::new();
        for entry in fs::read_dir(&generations).map_err(|_| StoreError::Unavailable)? {
            let entry = entry.map_err(|_| StoreError::Unavailable)?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else { continue };
            if name.starts_with('.') || !entry.file_type().is_ok_and(|kind| kind.is_dir() && !kind.is_symlink()) { continue; }
            if !safe_generation_id(&name) {
                invalid.push(name);
                continue;
            }
            let state = State { active_generation: Some(name.clone()), ..State::default() };
            if let Ok(snapshot) = self.load_generation(&name, state) {
                valid.push((name, snapshot.manifest.retrieved_at));
            } else {
                invalid.push(name);
            }
        }
        valid.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
        let newest_other = valid.iter().find(|(name, _)| Some(name.as_str()) != active).map(|(name, _)| name.clone());
        let mut changed = false;
        for name in invalid {
            if Some(name.as_str()) == active { continue; }
            changed |= remove_generation_if_unleased(&self.generations_dir, &generations, &name)?;
        }
        for (name, _) in valid {
            if Some(name.as_str()) == active || newest_other.as_deref() == Some(name.as_str()) { continue; }
            changed |= remove_generation_if_unleased(&self.generations_dir, &generations, &name)?;
        }
        if changed {
            injected("before-cleanup-generations-directory-fsync", StoreError::Unavailable)?;
            self.generations_dir.sync_all().map_err(|_| StoreError::Unavailable)?;
            injected("after-cleanup-generations-directory-fsync", StoreError::Unavailable)?;
        }
        Ok(())
    }
    #[rustfmt::skip]
    fn revalidate_root(&self) -> Result<(), StoreError> { validate_open_identity(&self.root_dir, &self.root) }
    #[rustfmt::skip]
    fn revalidate_generations(&self) -> Result<(), StoreError> { validate_open_identity(&self.generations_dir, &self.root.join("generations")) }
}
#[rustfmt::skip]
fn remove_generation_if_unleased(parent: &File, _generations: &Path, name: &str) -> Result<bool, StoreError> {
    #[cfg(not(unix))]
    let directory = _generations.join(name);
    #[cfg(unix)]
    let directory_handle = open_directory_at(parent, name.as_ref())?;
    #[cfg(unix)]
    let lease = open_existing_at(&directory_handle, "lease.lock")?;
    #[cfg(not(unix))]
    let lease = match open_existing_private_file(&directory.join("lease.lock")) {
        Ok(lease) => lease,
        Err(_) => return Ok(false),
    };
    match FileExt::try_lock_exclusive(&lease) { Ok(()) => {}, Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false), Err(_) => return Ok(false) }
    #[cfg(unix)]
    validate_directory_owner_mode(&directory_handle, true)?;
    #[cfg(not(unix))]
    validate_existing_directory(&directory)?;
    injected("before-cleanup-generation-delete", StoreError::Unavailable)?;
    #[cfg(unix)]
    {
        let quarantine = format!(".delete-{}", unique_suffix()?);
        rename_at(parent, name, &quarantine)?;
        parent.sync_all().map_err(|_| StoreError::Unavailable)?;
        remove_owned_dir_at(parent, &quarantine)?;
    }
    #[cfg(not(unix))]
    fs::remove_dir_all(directory).map_err(|_| StoreError::Unavailable)?;
    injected("after-cleanup-generation-delete", StoreError::Unavailable)?;
    Ok(true)
}
#[rustfmt::skip]
fn selected_root() -> Result<(PathBuf, PathBuf), StoreError> {
    if let Some(raw) = std::env::var_os("BIOMCP_GENCC_DIR") {
        let raw = raw.to_str().ok_or(StoreError::Unavailable)?.trim();
        let root = PathBuf::from(raw);
        if raw.is_empty() || !root.is_absolute() || root.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir)) { return Err(StoreError::Unavailable); }
        let parent = root.parent().ok_or(StoreError::Unavailable)?.to_path_buf();
        return Ok((root, parent));
    }
    let anchor = dirs::data_dir().ok_or(StoreError::Unavailable)?;
    Ok((anchor.join("biomcp").join("gencc"), anchor))
}
#[cfg(not(unix))]
fn validate_existing_directory(path: &Path) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::Unavailable)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(StoreError::Unavailable);
    }
    Ok(())
}
#[cfg(not(unix))]
fn validate_path_components(path: &Path) -> Result<(), StoreError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if matches!(component, Component::Normal(_)) {
            validate_existing_directory(&current)?;
        }
    }
    Ok(())
}
#[cfg(unix)]
#[rustfmt::skip]
fn open_secure_directory_chain(path: &Path, private_leaf: bool) -> Result<File, StoreError> {
    let mut directory = File::open("/").map_err(|_| StoreError::Unavailable)?;
    let mut normals = path.components().filter_map(|component| match component { Component::Normal(name) => Some(name), _ => None }).peekable();
    while let Some(name) = normals.next() {
        directory = open_directory_at(&directory, name)?;
        validate_directory_owner_mode(&directory, private_leaf && normals.peek().is_none())?;
    }
    Ok(directory)
}
#[cfg(unix)]
#[rustfmt::skip]
fn open_directory_at(parent: &File, name: &std::ffi::OsStr) -> Result<File, StoreError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(name.as_bytes()).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: the component is NUL-terminated and parent remains open.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if fd < 0 { return Err(StoreError::Unavailable); }
    // SAFETY: openat returned a new owned descriptor.
    Ok(File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}
#[cfg(unix)]
#[rustfmt::skip]
fn create_directory_at(parent: &File, name: &str) -> Result<File, StoreError> {
    use std::os::fd::AsRawFd;
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: parent and the NUL-terminated name remain valid.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 { return Err(StoreError::Unavailable); }
    open_directory_at(parent, std::ffi::OsStr::new(name.to_str().map_err(|_| StoreError::Unavailable)?))
}
#[cfg(unix)]
#[rustfmt::skip]
fn create_file_at(parent: &File, name: &str) -> Result<File, StoreError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: parent and the NUL-terminated name remain valid.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC, 0o600) };
    if fd < 0 { return Err(StoreError::Unavailable); }
    Ok(File::from(unsafe { OwnedFd::from_raw_fd(fd) }))
}
#[cfg(unix)]
#[rustfmt::skip]
fn write_new_at(parent: &File, name: &str, bytes: &[u8], point: &str) -> Result<(), StoreError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: parent and the NUL-terminated name remain valid.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC, 0o600) };
    if fd < 0 { return Err(StoreError::Unavailable); }
    let mut file = File::from(unsafe { OwnedFd::from_raw_fd(fd) });
    file.write_all(bytes).map_err(|_| StoreError::Unavailable)?;
    injected(&format!("before-{point}-fsync"), StoreError::Unavailable)?;
    file.sync_all().map_err(|_| StoreError::Unavailable)?;
    injected(&format!("after-{point}-fsync"), StoreError::Unavailable)
}
#[cfg(unix)]
#[rustfmt::skip]
fn open_existing_at(parent: &File, name: &str) -> Result<File, StoreError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Invalid)?;
    // SAFETY: parent and the NUL-terminated name remain valid.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    if fd < 0 { return Err(StoreError::Invalid); }
    let file = File::from(unsafe { OwnedFd::from_raw_fd(fd) });
    let metadata = file.metadata().map_err(|_| StoreError::Invalid)?;
    if !metadata.is_file() || metadata.uid() != unsafe { libc::geteuid() } || metadata.nlink() != 1 || metadata.permissions().mode() & 0o777 != 0o600 { return Err(StoreError::Invalid); }
    Ok(file)
}
#[cfg(unix)]
#[rustfmt::skip]
fn validate_at_identity(parent: &File, name: &str, file: &File) -> Result<(), StoreError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::MetadataExt;
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Invalid)?;
    let opened = file.metadata().map_err(|_| StoreError::Invalid)?;
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptors/pointers are valid; success initializes `named`.
    if unsafe { libc::fstatat(parent.as_raw_fd(), name.as_ptr(), named.as_mut_ptr(), libc::AT_SYMLINK_NOFOLLOW) } != 0 { return Err(StoreError::Invalid); }
    let named = unsafe { named.assume_init() };
    (opened.dev() == named.st_dev && opened.ino() == named.st_ino).then_some(()).ok_or(StoreError::Invalid)
}
#[cfg(unix)]
fn read_at(parent: &File, name: &str) -> Result<Vec<u8>, StoreError> {
    let mut file = open_existing_at(parent, name)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| StoreError::Invalid)?;
    Ok(bytes)
}
#[cfg(unix)]
#[rustfmt::skip]
fn rename_at(parent: &File, from: &str, to: &str) -> Result<(), StoreError> {
    use std::os::fd::AsRawFd;
    let from = std::ffi::CString::new(from).map_err(|_| StoreError::Unavailable)?;
    let to = std::ffi::CString::new(to).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: the one parent descriptor anchors both valid names.
    if unsafe { libc::renameat(parent.as_raw_fd(), from.as_ptr(), parent.as_raw_fd(), to.as_ptr()) } != 0 { return Err(StoreError::Unavailable); }
    Ok(())
}
#[cfg(unix)]
fn unlink_file_at(parent: &File, name: &str) -> Result<(), StoreError> {
    use std::os::fd::AsRawFd;
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    // SAFETY: parent and the NUL-terminated name remain valid.
    (unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), 0) } == 0)
        .then_some(())
        .ok_or(StoreError::Unavailable)
}
#[cfg(unix)]
fn remove_owned_dir_at(parent: &File, name: &str) -> Result<(), StoreError> {
    use std::os::fd::AsRawFd;
    let directory = open_directory_at(parent, name.as_ref())?;
    for child in ["index.json", "lease.lock", "manifest.json"] {
        let _ = unlink_file_at(&directory, child);
    }
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    (unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } == 0)
        .then_some(())
        .ok_or(StoreError::Unavailable)
}
#[cfg(unix)]
fn remove_file_owned(parent: &File, _path: &Path, name: &str) -> Result<(), StoreError> {
    unlink_file_at(parent, name)
}
#[cfg(not(unix))]
fn remove_file_owned(_parent: &File, path: &Path, name: &str) -> Result<(), StoreError> {
    fs::remove_file(path.join(name)).map_err(|_| StoreError::Unavailable)
}
#[cfg(unix)]
fn remove_dir_owned(parent: &File, _path: &Path, name: &str) -> Result<(), StoreError> {
    remove_owned_dir_at(parent, name)
}
#[cfg(not(unix))]
fn remove_dir_owned(_parent: &File, path: &Path, name: &str) -> Result<(), StoreError> {
    fs::remove_dir_all(path.join(name)).map_err(|_| StoreError::Unavailable)
}
#[cfg(unix)]
#[rustfmt::skip]
fn validate_directory_owner_mode(directory: &File, private: bool) -> Result<(), StoreError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let metadata = directory.metadata().map_err(|_| StoreError::Unavailable)?;
    let mode = metadata.permissions().mode() & 0o7777;
    let owner_ok = metadata.uid() == unsafe { libc::geteuid() };
    let trusted_system = metadata.uid() == 0 && (mode & 0o022 == 0 || mode & 0o1000 != 0);
    if !metadata.is_dir() || (!owner_ok && !trusted_system)
        || (private && (!owner_ok || mode & 0o777 != 0o700))
        || (!private && mode & 0o022 != 0 && mode & 0o1000 == 0) { return Err(StoreError::Unavailable); }
    Ok(())
}
#[cfg(unix)]
#[rustfmt::skip]
fn open_or_create_private_directory_at(
    parent: &File,
    name: &std::ffi::OsStr,
) -> Result<File, StoreError> {
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;
    match open_directory_at(parent, name) {
        Ok(directory) => { validate_directory_owner_mode(&directory, true)?; Ok(directory) }
        Err(_) => {
            let name = std::ffi::CString::new(name.as_bytes()).map_err(|_| StoreError::Unavailable)?;
            // SAFETY: the component is NUL-terminated and parent remains open.
            if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
                return Err(StoreError::Unavailable);
            }
            injected("before-bootstrap-directory-parent-fsync", StoreError::Unavailable)?;
            parent.sync_all().map_err(|_| StoreError::Unavailable)?;
            injected("after-bootstrap-directory-parent-fsync", StoreError::Unavailable)?;
            let directory = open_directory_at(parent, std::ffi::OsStr::from_bytes(name.as_bytes()))?;
            validate_directory_owner_mode(&directory, true)?;
            Ok(directory)
        }
    }
}
#[cfg(unix)]
#[rustfmt::skip]
fn open_private_at(parent: &File, name: &str) -> Result<File, StoreError> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let point = if name.starts_with(".biomcp-gencc-root-") { "anchor" } else if name == ".refresh.lock" { "refresh-lock" } else { "store-lock" };
    let name = std::ffi::CString::new(name).map_err(|_| StoreError::Unavailable)?;
    let flags = libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    // SAFETY: the name is NUL-terminated and parent remains open.
    let fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0o600) };
    if fd < 0 { return Err(StoreError::Unavailable); }
    // SAFETY: openat returned a new owned descriptor.
    let file = File::from(unsafe { OwnedFd::from_raw_fd(fd) });
    let metadata = file.metadata().map_err(|_| StoreError::Unavailable)?;
    if !metadata.is_file() || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.nlink() != 1 || metadata.permissions().mode() & 0o777 != 0o600 { return Err(StoreError::Unavailable); }
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptors/pointers are valid; success initializes `named`.
    if unsafe { libc::fstatat(parent.as_raw_fd(), name.as_ptr(), named.as_mut_ptr(), libc::AT_SYMLINK_NOFOLLOW) } != 0 { return Err(StoreError::Unavailable); }
    let named = unsafe { named.assume_init() };
    if metadata.dev() != named.st_dev || metadata.ino() != named.st_ino { return Err(StoreError::Unavailable); }
    injected(&format!("before-{point}-file-fsync"), StoreError::Unavailable)?;
    file.sync_all().map_err(|_| StoreError::Unavailable)?;
    injected(&format!("after-{point}-file-fsync"), StoreError::Unavailable)?;
    injected(&format!("before-{point}-parent-fsync"), StoreError::Unavailable)?;
    parent.sync_all().map_err(|_| StoreError::Unavailable)?;
    injected(&format!("after-{point}-parent-fsync"), StoreError::Unavailable)?;
    Ok(file)
}
#[cfg(not(unix))]
fn open_private_file(path: &Path) -> Result<File, StoreError> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    let file =
        crate::cache::open_private(&mut options, path).map_err(|_| StoreError::Unavailable)?;
    validate_open_identity(&file, path)?;
    Ok(file)
}
#[cfg(unix)]
fn validate_open_identity(file: &File, path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::MetadataExt;
    let opened = file.metadata().map_err(|_| StoreError::Unavailable)?;
    let named = fs::symlink_metadata(path).map_err(|_| StoreError::Unavailable)?;
    if opened.dev() != named.dev() || opened.ino() != named.ino() {
        return Err(StoreError::Unavailable);
    }
    Ok(())
}
#[cfg(not(unix))]
fn validate_open_identity(_file: &File, path: &Path) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::Unavailable)?;
    (!metadata.file_type().is_symlink())
        .then_some(())
        .ok_or(StoreError::Unavailable)
}
#[cfg(not(unix))]
fn open_existing_private_file(path: &Path) -> Result<File, StoreError> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    crate::cache::open_private(&mut options, path).map_err(|_| StoreError::Invalid)
}
#[cfg(unix)]
fn acquire_generation_lease_at(
    directory: &File,
    key: &Path,
    deadline: std::time::Instant,
) -> Result<Arc<File>, StoreError> {
    let leases = GENERATION_LEASES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut leases = leases.lock().map_err(|_| StoreError::Unavailable)?;
    if let Some(lease) = leases.get(key).and_then(Weak::upgrade)
        && validate_at_identity(directory, "lease.lock", &lease).is_ok()
    {
        return Ok(lease);
    }
    leases.retain(|_, lease| lease.strong_count() > 0);
    let lease = open_existing_at(directory, "lease.lock")?;
    lock_shared_until(&lease, deadline)?;
    let lease = Arc::new(lease);
    leases.insert(key.to_path_buf(), Arc::downgrade(&lease));
    Ok(lease)
}
#[cfg(not(unix))]
fn acquire_generation_lease(
    path: &Path,
    deadline: std::time::Instant,
) -> Result<Arc<File>, StoreError> {
    let leases = GENERATION_LEASES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut leases = leases.lock().map_err(|_| StoreError::Unavailable)?;
    if let Some(lease) = leases.get(path).and_then(Weak::upgrade)
        && validate_open_identity(&lease, path).is_ok()
    {
        return Ok(lease);
    }
    leases.retain(|_, lease| lease.strong_count() > 0);
    let lease = open_existing_private_file(path)?;
    lock_shared_until(&lease, deadline)?;
    let lease = Arc::new(lease);
    leases.insert(path.to_path_buf(), Arc::downgrade(&lease));
    Ok(lease)
}
fn lock_shared_until(file: &File, deadline: std::time::Instant) -> Result<(), StoreError> {
    lock_until(file, false, deadline)
}
fn lock_exclusive_until(file: &File, deadline: std::time::Instant) -> Result<(), StoreError> {
    lock_until(file, true, deadline)
}
fn lock_until(
    file: &File,
    exclusive: bool,
    deadline: std::time::Instant,
) -> Result<(), StoreError> {
    loop {
        let result = if exclusive {
            FileExt::try_lock_exclusive(file)
        } else {
            FileExt::try_lock_shared(file)
        };
        match result {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => return Err(StoreError::Unavailable),
        }
        let now = std::time::Instant::now();
        if now >= deadline {
            return Err(StoreError::Deadline);
        }
        std::thread::sleep(
            deadline
                .saturating_duration_since(now)
                .min(std::time::Duration::from_millis(10)),
        );
    }
}
#[cfg(not(unix))]
fn read_regular(path: &Path) -> Result<Vec<u8>, StoreError> {
    let mut file = crate::cache::open_managed_read(path).map_err(|_| StoreError::Invalid)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| StoreError::Invalid)?;
    Ok(bytes)
}
#[cfg(not(unix))]
fn write_new_synced(path: &Path, bytes: &[u8], point: &str) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| StoreError::Unavailable)?;
    set_private_file(path)?;
    file.write_all(bytes).map_err(|_| StoreError::Unavailable)?;
    injected(&format!("before-{point}-fsync"), StoreError::Unavailable)?;
    file.sync_all().map_err(|_| StoreError::Unavailable)?;
    injected(&format!("after-{point}-fsync"), StoreError::Unavailable)
}
fn sync_file_injected(file: &File, point: &str, error: StoreError) -> Result<(), StoreError> {
    injected(&format!("before-{point}-fsync"), error)?;
    file.sync_all().map_err(|_| error)?;
    injected(&format!("after-{point}-fsync"), error)
}
fn injected(point: &str, error: StoreError) -> Result<(), StoreError> {
    #[cfg(debug_assertions)]
    {
        if std::env::var("BIOMCP_GENCC_TEST_CRASH_AT").as_deref() == Ok(point) {
            if let Some(marker) = std::env::var_os("BIOMCP_GENCC_TEST_CRASH_MARKER") {
                let _ = fs::write(marker, point);
            }
            std::process::abort();
        }
        if std::env::var("BIOMCP_GENCC_TEST_FAIL_AT").as_deref() == Ok(point) {
            return Err(error);
        }
    }
    let _ = point;
    Ok(())
}
#[cfg(not(unix))]
fn set_private_file(_path: &Path) -> Result<(), StoreError> {
    Ok(())
}
#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> Result<(), StoreError> {
    fs::create_dir(path).map_err(|_| StoreError::Unavailable)?;
    set_private_dir(path)
}
#[cfg(not(unix))]
fn set_private_dir(_path: &Path) -> Result<(), StoreError> {
    Ok(())
}
#[cfg(not(unix))]
fn sync_dir(path: &Path) -> Result<(), StoreError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|_| StoreError::Unavailable)
}
fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn unique_suffix() -> Result<String, StoreError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| StoreError::Unavailable)?;
    Ok(hex_sha256(&bytes)[..32].to_string())
}
fn safe_generation_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
