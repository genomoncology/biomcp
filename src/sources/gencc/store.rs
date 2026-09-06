//! Private immutable generations and mutable GenCC lifecycle state.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::model::GenCcDataset;

const STATE_SCHEMA: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub(crate) enum StoreError {
    #[error("GenCC store is unavailable")]
    Unavailable,
    #[error("GenCC store data is invalid")]
    Invalid,
    #[error("GenCC store update failed after namespace publication")]
    PostRenameSync,
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
}

pub(crate) struct PublishMetadata<'a> {
    pub now: &'a str,
    pub etag: &'a str,
    pub last_modified: &'a str,
    pub endpoint: &'a str,
    pub row_count: usize,
}

pub(crate) struct Store {
    root: PathBuf,
    refresh_lock: File,
    store_lock: File,
}

impl Store {
    pub(crate) fn open() -> Result<Self, StoreError> {
        let (root, anchor_dir) = selected_root()?;
        validate_existing_directory(&anchor_dir)?;
        let anchor_name = format!(
            ".biomcp-gencc-root-{}.lock",
            hex_sha256(root.as_os_str().as_encoded_bytes())
        );
        let anchor = open_private_file(&anchor_dir.join(anchor_name))?;
        FileExt::lock_exclusive(&anchor).map_err(|_| StoreError::Unavailable)?;

        if !root.exists() {
            if let Some(parent) = root.parent()
                && !parent.exists()
                && parent.parent() == Some(anchor_dir.as_path())
            {
                fs::create_dir(parent).map_err(|_| StoreError::Unavailable)?;
                set_private_dir(parent)?;
                sync_dir(&anchor_dir)?;
            }
            fs::create_dir(&root).map_err(|_| StoreError::Unavailable)?;
            set_private_dir(&root)?;
            sync_dir(&anchor_dir)?;
        }
        validate_existing_directory(&root)?;
        let refresh_lock = open_private_file(&root.join(".refresh.lock"))?;
        let store_lock = open_private_file(&root.join(".store.lock"))?;
        let generations = root.join("generations");
        if !generations.exists() {
            fs::create_dir(&generations).map_err(|_| StoreError::Unavailable)?;
            set_private_dir(&generations)?;
            sync_dir(&root)?;
        }
        validate_existing_directory(&generations)?;
        FileExt::lock_shared(&store_lock).map_err(|_| StoreError::Unavailable)?;
        FileExt::unlock(&anchor).map_err(|_| StoreError::Unavailable)?;
        FileExt::unlock(&store_lock).map_err(|_| StoreError::Unavailable)?;
        Ok(Self {
            root,
            refresh_lock,
            store_lock,
        })
    }

    pub(crate) fn lock_refresh(&self) -> Result<(), StoreError> {
        FileExt::lock_exclusive(&self.refresh_lock).map_err(|_| StoreError::Unavailable)
    }

    pub(crate) fn unlock_refresh(&self) {
        let _ = FileExt::unlock(&self.refresh_lock);
    }

    pub(crate) fn load(&self) -> Result<Option<Snapshot>, StoreError> {
        FileExt::lock_shared(&self.store_lock).map_err(|_| StoreError::Unavailable)?;
        let result = self.load_locked();
        let _ = FileExt::unlock(&self.store_lock);
        result
    }

    pub(crate) fn load_state(&self) -> Result<State, StoreError> {
        FileExt::lock_shared(&self.store_lock).map_err(|_| StoreError::Unavailable)?;
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

    fn load_locked(&self) -> Result<Option<Snapshot>, StoreError> {
        let state_path = self.root.join("state.json");
        if !state_path.exists() {
            return self.recover_locked();
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
        let manifest_bytes = read_regular(&directory.join("manifest.json"))?;
        let index_bytes = read_regular(&directory.join("index.json"))?;
        let manifest: Manifest =
            serde_json::from_slice(&manifest_bytes).map_err(|_| StoreError::Invalid)?;
        if manifest.schema_version != STATE_SCHEMA
            || manifest.upstream_version.is_some()
            || manifest.index_sha256 != hex_sha256(&index_bytes)
        {
            return Err(StoreError::Invalid);
        }
        let dataset = serde_json::from_slice::<GenCcDataset>(&index_bytes)
            .map_err(|_| StoreError::Invalid)?;
        if dataset.assertions().len() != manifest.assertion_count {
            return Err(StoreError::Invalid);
        }
        Ok(Snapshot {
            state,
            manifest,
            dataset,
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
        Ok(Some(snapshot))
    }

    pub(crate) fn publish(
        &self,
        dataset: &GenCcDataset,
        body: &[u8],
        metadata: PublishMetadata<'_>,
    ) -> Result<Snapshot, StoreError> {
        let index = serde_json::to_vec(dataset).map_err(|_| StoreError::Invalid)?;
        let index_hash = hex_sha256(&index);
        let suffix = format!("{:x}", unique_suffix());
        let generation = format!("{}-{suffix}", &index_hash[..24]);
        let temporary = self
            .root
            .join("generations")
            .join(format!(".tmp-{generation}"));
        fs::create_dir(&temporary).map_err(|_| StoreError::Unavailable)?;
        set_private_dir(&temporary)?;
        write_new_synced(&temporary.join("index.json"), &index)?;
        write_new_synced(&temporary.join("lease.lock"), b"")?;
        let manifest = Manifest {
            schema_version: STATE_SCHEMA,
            endpoint: metadata.endpoint.to_string(),
            body_sha256: hex_sha256(body),
            index_sha256: index_hash,
            row_count: metadata.row_count,
            assertion_count: dataset.assertions().len(),
            retrieved_at: metadata.now.to_string(),
            etag: metadata.etag.to_string(),
            last_modified: metadata.last_modified.to_string(),
            upstream_version: None,
        };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(|_| StoreError::Invalid)?;
        write_new_synced(&temporary.join("manifest.json"), &manifest_bytes)?;
        sync_dir(&temporary)?;

        FileExt::lock_exclusive(&self.store_lock).map_err(|_| StoreError::Unavailable)?;
        let final_path = self.root.join("generations").join(&generation);
        let result = (|| {
            fs::rename(&temporary, &final_path).map_err(|_| StoreError::Unavailable)?;
            sync_dir(&self.root.join("generations"))?;
            let state = State {
                active_generation: Some(generation.clone()),
                checked_at: Some(metadata.now.to_string()),
                attempted_at: Some(metadata.now.to_string()),
                last_attempt: Some(Attempt::Success200),
                ..State::default()
            };
            self.replace_state_locked(&state)?;
            self.load_generation(&generation, state)
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
        FileExt::lock_exclusive(&self.store_lock).map_err(|_| StoreError::Unavailable)?;
        let result = self.replace_state_locked(state);
        let _ = FileExt::unlock(&self.store_lock);
        result
    }

    fn replace_state_locked(&self, state: &State) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(state).map_err(|_| StoreError::Invalid)?;
        let temporary = self.root.join(format!(".state-{}.tmp", unique_suffix()));
        write_new_synced(&temporary, &bytes)?;
        fs::rename(&temporary, self.root.join("state.json"))
            .map_err(|_| StoreError::Unavailable)?;
        sync_dir(&self.root).map_err(|_| StoreError::PostRenameSync)
    }
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

fn open_private_file(path: &Path) -> Result<File, StoreError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| StoreError::Unavailable)?;
    let metadata = file.metadata().map_err(|_| StoreError::Unavailable)?;
    if !metadata.is_file() {
        return Err(StoreError::Unavailable);
    }
    set_private_file(path)?;
    Ok(file)
}

fn read_regular(path: &Path) -> Result<Vec<u8>, StoreError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StoreError::Invalid)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(StoreError::Invalid);
    }
    fs::read(path).map_err(|_| StoreError::Invalid)
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| StoreError::Unavailable)?;
    set_private_file(path)?;
    file.write_all(bytes).map_err(|_| StoreError::Unavailable)?;
    file.sync_all().map_err(|_| StoreError::Unavailable)
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

fn unique_suffix() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        ^ u128::from(std::process::id())
}

fn safe_generation_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
