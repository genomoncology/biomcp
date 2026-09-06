//! Private immutable generations and mutable GenCC lifecycle state.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
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
pub(crate) const PUBLICATION_CRASH_POINTS: [&str; 18] = [
    "before-index-file-fsync", "after-index-file-fsync", "before-lease-file-fsync",
    "after-lease-file-fsync", "before-manifest-file-fsync", "after-manifest-file-fsync",
    "before-generation-directory-fsync", "after-generation-directory-fsync",
    "before-generation-rename", "after-generation-rename", "before-generations-directory-fsync",
    "after-generations-directory-fsync", "before-state-file-fsync", "after-state-file-fsync",
    "before-state-rename", "after-state-rename", "before-root-directory-fsync",
    "after-root-directory-fsync",
];

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub(crate) enum StoreError {
    #[error("GenCC store is unavailable")]
    Unavailable,
    #[error("GenCC store data is invalid")]
    Invalid,
    #[error("GenCC store update failed after namespace publication")]
    PostRenameSync,
    #[error("GenCC store lock deadline expired")]
    Deadline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Manifest {
    pub schema_version: u32,
    pub endpoint: String,
    pub body_sha256: String,
    pub index_sha256: String,
    pub row_count: usize,
    pub assertion_count: usize,
    pub retrieved_at: String,
    pub etag: String,
    pub last_modified: String,
    pub upstream_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct State {
    pub schema_version: u32,
    pub active_generation: Option<String>,
    pub checked_at: Option<String>,
    pub attempted_at: Option<String>,
    pub last_attempt: Option<Attempt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Attempt {
    Success200,
    Success304,
    Failure,
}

impl Default for State {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA,
            active_generation: None,
            checked_at: None,
            attempted_at: None,
            last_attempt: None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Snapshot {
    pub state: State,
    pub manifest: Manifest,
    pub dataset: GenCcDataset,
    pub(crate) lease: Arc<File>,
}

pub(crate) struct PublishMetadata<'a> {
    pub now: &'a str,
    pub etag: &'a str,
    pub last_modified: &'a str,
    pub endpoint: &'a str,
    pub body_sha256: &'a str,
    pub row_count: usize,
}

pub(crate) struct RawCsvTemp {
    path: PathBuf,
    file: File,
    hasher: Sha256,
    len: usize,
}

impl RawCsvTemp {
    pub(crate) fn write_chunk(&mut self, bytes: &[u8], max: usize) -> Result<(), StoreError> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .filter(|len| *len <= max)
            .ok_or(StoreError::Invalid)?;
        self.file
            .write_all(bytes)
            .map_err(|_| StoreError::Unavailable)?;
        self.hasher.update(bytes);
        Ok(())
    }

    pub(crate) fn finish(&mut self) -> Result<(Vec<u8>, String), StoreError> {
        self.file.sync_all().map_err(|_| StoreError::Unavailable)?;
        let bytes = read_regular(&self.path)?;
        Ok((bytes, format!("{:x}", self.hasher.clone().finalize())))
    }
}

impl Drop for RawCsvTemp {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        if let Some(parent) = self.path.parent() {
            let _ = sync_dir(parent);
        }
    }
}

pub(crate) struct Store {
    root: PathBuf,
    refresh_lock: File,
    store_lock: File,
    deadline: std::time::Instant,
}

impl Store {
    #[cfg(test)]
    pub(crate) fn open() -> Result<Self, StoreError> {
        Self::open_until(std::time::Instant::now() + std::time::Duration::from_secs(2))
    }

    pub(crate) fn open_until(deadline: std::time::Instant) -> Result<Self, StoreError> {
        let (root, anchor_dir) = selected_root()?;
        validate_path_components(&anchor_dir)?;
        let anchor_name = format!(
            ".biomcp-gencc-root-{}.lock",
            hex_sha256(root.as_os_str().as_encoded_bytes())
        );
        let anchor = open_private_file(&anchor_dir.join(anchor_name))?;
        validate_open_identity(
            &anchor,
            &anchor_dir.join(format!(
                ".biomcp-gencc-root-{}.lock",
                hex_sha256(root.as_os_str().as_encoded_bytes())
            )),
        )?;
        lock_exclusive_until(&anchor, deadline)?;

        if !root.exists() {
            if let Some(parent) = root.parent()
                && !parent.exists()
                && parent.parent() == Some(anchor_dir.as_path())
            {
                fs::create_dir(parent).map_err(|_| StoreError::Unavailable)?;
                set_private_dir(parent)?;
                sync_dir(&anchor_dir)?;
            }
            validate_path_components(root.parent().ok_or(StoreError::Unavailable)?)?;
            fs::create_dir(&root).map_err(|_| StoreError::Unavailable)?;
            set_private_dir(&root)?;
            sync_dir(&anchor_dir)?;
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
        Ok(Self {
            root,
            refresh_lock,
            store_lock,
            deadline,
        })
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
        let path = self.root.join(format!(".raw-{}.tmp", unique_suffix()?));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| StoreError::Unavailable)?;
        if set_private_file(&path).is_err() {
            let _ = fs::remove_file(&path);
            return Err(StoreError::Unavailable);
        }
        Ok(RawCsvTemp {
            path,
            file,
            hasher: Sha256::new(),
            len: 0,
        })
    }

    pub(crate) fn cleanup_abandoned(&self) {
        if lock_exclusive_until(&self.store_lock, self.deadline).is_err() {
            return;
        }
        let mut root_changed = false;
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !(name.starts_with(".raw-") || name.starts_with(".state-"))
                    || !name.ends_with(".tmp")
                {
                    continue;
                }
                if entry
                    .file_type()
                    .is_ok_and(|file_type| file_type.is_file() && !file_type.is_symlink())
                    && fs::remove_file(entry.path()).is_ok()
                {
                    root_changed = true;
                }
            }
        }
        if root_changed {
            let _ = sync_dir(&self.root);
        }
        let generations = self.root.join("generations");
        let mut generations_changed = false;
        if let Ok(entries) = fs::read_dir(&generations) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.starts_with(".tmp-") {
                    continue;
                }
                if entry
                    .file_type()
                    .is_ok_and(|file_type| file_type.is_dir() && !file_type.is_symlink())
                    && fs::remove_dir_all(entry.path()).is_ok()
                {
                    generations_changed = true;
                }
            }
        }
        if generations_changed {
            let _ = sync_dir(&generations);
        }
        let _ = FileExt::unlock(&self.store_lock);
    }

    pub(crate) fn load(&self) -> Result<Option<Snapshot>, StoreError> {
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
        lock_shared_until(&self.store_lock, self.deadline)?;
        let path = self.root.join("state.json");
        let result = if path.exists() {
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

    fn load_generation(&self, generation: &str, state: State) -> Result<Snapshot, StoreError> {
        if !safe_generation_id(generation) {
            return Err(StoreError::Invalid);
        }
        let directory = self.root.join("generations").join(generation);
        validate_existing_directory(&directory)?;
        let lease = acquire_generation_lease(&directory.join("lease.lock"), self.deadline)?;
        let manifest_bytes = read_regular(&directory.join("manifest.json"))?;
        let index_bytes = read_regular(&directory.join("index.json"))?;
        let manifest: Manifest =
            serde_json::from_slice(&manifest_bytes).map_err(|_| StoreError::Invalid)?;
        if manifest.schema_version != STATE_SCHEMA
            || manifest.endpoint != super::ENDPOINT
            || manifest.upstream_version.is_some()
            || manifest.row_count > 100_000
            || manifest.assertion_count > manifest.row_count
            || manifest.body_sha256.len() != 64
            || !manifest
                .body_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
            || manifest.index_sha256 != hex_sha256(&index_bytes)
            || !super::valid_etag(&manifest.etag)
            || !super::valid_http_date(&manifest.last_modified)
            || DateTime::parse_from_rfc3339(&manifest.retrieved_at).is_err()
        {
            return Err(StoreError::Invalid);
        }
        let dataset = serde_json::from_slice::<GenCcDataset>(&index_bytes)
            .map_err(|_| StoreError::Invalid)?;
        if dataset.assertions().len() != manifest.assertion_count {
            return Err(StoreError::Invalid);
        }
        if dataset.row_count() != manifest.row_count {
            return Err(StoreError::Invalid);
        }
        Ok(Snapshot {
            state,
            manifest,
            dataset,
            lease,
        })
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
        let _ = self.cleanup_locked(snapshot.state.active_generation.as_deref());
        Ok(Some(snapshot))
    }

    pub(crate) fn publish(
        &self,
        dataset: &GenCcDataset,
        metadata: PublishMetadata<'_>,
    ) -> Result<Snapshot, StoreError> {
        let index = serde_json::to_vec(dataset).map_err(|_| StoreError::Invalid)?;
        let index_hash = hex_sha256(&index);
        let suffix = unique_suffix()?;
        let generation = format!("{}-{suffix}", &index_hash[..24]);
        let temporary = self
            .root
            .join("generations")
            .join(format!(".tmp-{generation}"));
        fs::create_dir(&temporary).map_err(|_| StoreError::Unavailable)?;
        set_private_dir(&temporary)?;
        write_new_synced(&temporary.join("index.json"), &index, "index-file")?;
        write_new_synced(&temporary.join("lease.lock"), b"", "lease-file")?;
        let manifest = Manifest {
            schema_version: STATE_SCHEMA,
            endpoint: metadata.endpoint.to_string(),
            body_sha256: metadata.body_sha256.to_string(),
            index_sha256: index_hash,
            row_count: metadata.row_count,
            assertion_count: dataset.assertions().len(),
            retrieved_at: metadata.now.to_string(),
            etag: metadata.etag.to_string(),
            last_modified: metadata.last_modified.to_string(),
            upstream_version: None,
        };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(|_| StoreError::Invalid)?;
        write_new_synced(
            &temporary.join("manifest.json"),
            &manifest_bytes,
            "manifest-file",
        )?;
        sync_dir_injected(&temporary, "generation-directory", StoreError::Unavailable)?;

        lock_exclusive_until(&self.store_lock, self.deadline)?;
        let final_path = self.root.join("generations").join(&generation);
        let result = (|| {
            rename_injected(
                &temporary,
                &final_path,
                "generation",
                StoreError::Unavailable,
            )?;
            sync_dir_injected(
                &self.root.join("generations"),
                "generations-directory",
                StoreError::Unavailable,
            )?;
            let state = State {
                active_generation: Some(generation.clone()),
                checked_at: Some(metadata.now.to_string()),
                attempted_at: Some(metadata.now.to_string()),
                last_attempt: Some(Attempt::Success200),
                ..State::default()
            };
            self.replace_state_locked(&state)?;
            let snapshot = self.load_generation(&generation, state)?;
            let _ = self.cleanup_locked(Some(&generation));
            Ok(snapshot)
        })();
        let _ = FileExt::unlock(&self.store_lock);
        if result.is_err() {
            let _ = fs::remove_dir_all(&temporary);
        }
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
        lock_exclusive_until(&self.store_lock, self.deadline)?;
        let result = self.replace_state_locked(state);
        let _ = FileExt::unlock(&self.store_lock);
        result
    }

    fn replace_state_locked(&self, state: &State) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(state).map_err(|_| StoreError::Invalid)?;
        let temporary = self.root.join(format!(".state-{}.tmp", unique_suffix()?));
        write_new_synced(&temporary, &bytes, "state-file")?;
        injected("before-state-rename", StoreError::Unavailable)?;
        fs::rename(&temporary, self.root.join("state.json"))
            .map_err(|_| StoreError::Unavailable)?;
        injected("after-state-rename", StoreError::PostRenameSync)?;
        sync_dir_injected(&self.root, "root-directory", StoreError::PostRenameSync)
    }

    fn cleanup_locked(&self, active: Option<&str>) -> Result<(), StoreError> {
        let generations = self.root.join("generations");
        let mut valid = Vec::new();
        let mut invalid = Vec::new();
        for entry in fs::read_dir(&generations).map_err(|_| StoreError::Unavailable)? {
            let entry = entry.map_err(|_| StoreError::Unavailable)?;
            let Some(name) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if name.starts_with('.')
                || !entry
                    .file_type()
                    .is_ok_and(|file_type| file_type.is_dir() && !file_type.is_symlink())
            {
                continue;
            }
            if !safe_generation_id(&name) {
                invalid.push(name);
                continue;
            }
            let state = State {
                active_generation: Some(name.clone()),
                ..State::default()
            };
            if let Ok(snapshot) = self.load_generation(&name, state) {
                valid.push((name, snapshot.manifest.retrieved_at));
            } else {
                invalid.push(name);
            }
        }
        valid.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| right.0.cmp(&left.0)));
        let newest_other = valid
            .iter()
            .find(|(name, _)| Some(name.as_str()) != active)
            .map(|(name, _)| name.clone());
        let mut changed = false;
        for name in invalid {
            if Some(name.as_str()) == active {
                continue;
            }
            changed |= remove_generation_if_unleased(&generations.join(name))?;
        }
        for (name, _) in valid {
            if Some(name.as_str()) == active || newest_other.as_deref() == Some(name.as_str()) {
                continue;
            }
            changed |= remove_generation_if_unleased(&generations.join(name))?;
        }
        if changed {
            sync_dir(&generations)?;
        }
        Ok(())
    }
}

fn remove_generation_if_unleased(directory: &Path) -> Result<bool, StoreError> {
    let lease = match open_existing_private_file(&directory.join("lease.lock")) {
        Ok(lease) => lease,
        Err(_) => return Ok(false),
    };
    match FileExt::try_lock_exclusive(&lease) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
        Err(_) => return Ok(false),
    }
    validate_existing_directory(directory)?;
    fs::remove_dir_all(directory).map_err(|_| StoreError::Unavailable)?;
    Ok(true)
}

fn selected_root() -> Result<(PathBuf, PathBuf), StoreError> {
    if let Some(raw) = std::env::var_os("BIOMCP_GENCC_DIR") {
        let raw = raw.to_str().ok_or(StoreError::Unavailable)?.trim();
        let root = PathBuf::from(raw);
        if raw.is_empty()
            || !root.is_absolute()
            || root
                .components()
                .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err(StoreError::Unavailable);
        }
        let parent = root.parent().ok_or(StoreError::Unavailable)?.to_path_buf();
        return Ok((root, parent));
    }
    let anchor = dirs::data_dir().ok_or(StoreError::Unavailable)?;
    Ok((anchor.join("biomcp").join("gencc"), anchor))
}

fn validate_existing_directory(path: &Path) -> Result<(), StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::Unavailable)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(StoreError::Unavailable);
    }
    Ok(())
}

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

fn open_existing_private_file(path: &Path) -> Result<File, StoreError> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    crate::cache::open_private(&mut options, path).map_err(|_| StoreError::Invalid)
}

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

fn read_regular(path: &Path) -> Result<Vec<u8>, StoreError> {
    let mut file = crate::cache::open_managed_read(path).map_err(|_| StoreError::Invalid)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| StoreError::Invalid)?;
    Ok(bytes)
}

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

fn sync_dir_injected(path: &Path, point: &str, error: StoreError) -> Result<(), StoreError> {
    injected(&format!("before-{point}-fsync"), error)?;
    sync_dir(path)?;
    injected(&format!("after-{point}-fsync"), error)
}

fn rename_injected(
    from: &Path,
    to: &Path,
    point: &str,
    error: StoreError,
) -> Result<(), StoreError> {
    injected(&format!("before-{point}-rename"), error)?;
    fs::rename(from, to).map_err(|_| StoreError::Unavailable)?;
    injected(&format!("after-{point}-rename"), error)
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

#[cfg(unix)]
fn set_private_file(path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| StoreError::Unavailable)
}

#[cfg(not(unix))]
fn set_private_file(_path: &Path) -> Result<(), StoreError> {
    Ok(())
}

#[cfg(unix)]
fn set_private_dir(path: &Path) -> Result<(), StoreError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| StoreError::Unavailable)
}

#[cfg(not(unix))]
fn set_private_dir(_path: &Path) -> Result<(), StoreError> {
    Ok(())
}

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

#[cfg(test)]
mod crash_tests {
    use std::process::{Command, Stdio};
    use std::sync::atomic::AtomicBool;

    use super::*;

    fn fixture() -> &'static [u8] {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/gencc/submissions-new-odc1.csv"
        ))
    }

    fn publish(store: &Store, dataset: &GenCcDataset, now: &str, etag: &str) -> Snapshot {
        store
            .publish(
                dataset,
                PublishMetadata {
                    now,
                    etag,
                    last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                    endpoint: super::super::ENDPOINT,
                    body_sha256: &hex_sha256(fixture()),
                    row_count: dataset.row_count(),
                },
            )
            .unwrap()
    }

    #[test]
    #[serial_test::serial(gencc_env)]
    fn crash_boundaries_preserve_one_complete_namespace_generation() {
        for point in PUBLICATION_CRASH_POINTS {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("gencc");
            let marker = temp.path().join("crashed");
            let previous = std::env::var_os("BIOMCP_GENCC_DIR");
            unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
            let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
            let old = publish(
                &Store::open().unwrap(),
                &dataset,
                "2026-01-01T00:00:00Z",
                "\"crash-old\"",
            )
            .state
            .active_generation
            .unwrap();
            match previous {
                Some(value) => unsafe { std::env::set_var("BIOMCP_GENCC_DIR", value) },
                None => unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") },
            }
            let status = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "sources::gencc::tests::gencc_subprocess_client",
                ])
                .env("BIOMCP_GENCC_DIR", &root)
                .env("BIOMCP_GENCC_CHILD_CRASH_PUBLISH", "1")
                .env("BIOMCP_GENCC_TEST_CRASH_AT", point)
                .env("BIOMCP_GENCC_TEST_CRASH_MARKER", &marker)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(!status.success(), "{point}");
            assert_eq!(fs::read_to_string(&marker).unwrap(), point);
            unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
            let store = Store::open().unwrap();
            let visible = store.load().unwrap().unwrap();
            let renamed = matches!(
                point,
                "after-state-rename" | "before-root-directory-fsync" | "after-root-directory-fsync"
            );
            assert_eq!(
                visible.state.active_generation.as_deref() != Some(&old),
                renamed,
                "{point}"
            );
            assert_eq!(visible.manifest.etag == "\"crash-new\"", renamed, "{point}");
            store.cleanup_abandoned();
            unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
        }
    }

    #[test]
    #[serial_test::serial(gencc_env)]
    fn subprocess_lease_defers_old_generation_cleanup_until_reader_exits() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("gencc");
        let entered = temp.path().join("entered");
        let release = temp.path().join("release");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
        let store = Store::open().unwrap();
        drop(publish(&store, &dataset, "2026-01-01T00:00:00Z", "\"g1\""));
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "sources::gencc::tests::gencc_subprocess_client",
            ])
            .env("BIOMCP_GENCC_DIR", &root)
            .env("BIOMCP_GENCC_CHILD_HOLD_LEASE", &entered)
            .env("BIOMCP_GENCC_CHILD_RELEASE", &release)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !entered.exists() {
            assert!(child.try_wait().unwrap().is_none());
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        drop(publish(&store, &dataset, "2026-01-02T00:00:00Z", "\"g2\""));
        drop(publish(&store, &dataset, "2026-01-03T00:00:00Z", "\"g3\""));
        assert_eq!(fs::read_dir(root.join("generations")).unwrap().count(), 3);
        fs::write(&release, b"release").unwrap();
        assert!(child.wait().unwrap().success());
        drop(publish(&store, &dataset, "2026-01-04T00:00:00Z", "\"g4\""));
        assert_eq!(fs::read_dir(root.join("generations")).unwrap().count(), 2);
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }

    #[test]
    #[serial_test::serial(gencc_env)]
    fn injected_state_rename_failures_report_the_visible_namespace() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("gencc");
        unsafe { std::env::set_var("BIOMCP_GENCC_DIR", &root) };
        let dataset = GenCcDataset::parse(fixture(), &AtomicBool::new(false)).unwrap();
        let store = Store::open().unwrap();
        let old = publish(&store, &dataset, "2026-01-01T00:00:00Z", "\"old\"")
            .state
            .active_generation
            .unwrap();
        for (point, renamed) in [("before-state-rename", false), ("after-state-rename", true)] {
            unsafe { std::env::set_var("BIOMCP_GENCC_TEST_FAIL_AT", point) };
            let result = store.publish(
                &dataset,
                PublishMetadata {
                    now: "2026-02-01T00:00:00Z",
                    etag: "\"new\"",
                    last_modified: "Sun, 06 Sep 2026 06:00:29 GMT",
                    endpoint: super::super::ENDPOINT,
                    body_sha256: &hex_sha256(fixture()),
                    row_count: dataset.row_count(),
                },
            );
            unsafe { std::env::remove_var("BIOMCP_GENCC_TEST_FAIL_AT") };
            assert_eq!(matches!(result, Err(StoreError::PostRenameSync)), renamed);
            assert_eq!(
                store
                    .load()
                    .unwrap()
                    .unwrap()
                    .state
                    .active_generation
                    .as_deref()
                    != Some(&old),
                renamed
            );
        }
        unsafe { std::env::remove_var("BIOMCP_GENCC_DIR") };
    }
}
