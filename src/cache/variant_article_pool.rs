//! Immutable article pools and resumable source membership in the shared cache.
use std::io;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ResolvedCacheConfig, sqlite};

const SNAPSHOTS: &str = "variant-pool";
const POINTERS: &str = "variant-pool-pointer";
const PROGRESS: &str = "variant-pool-progress";
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Snapshot {
    pub(crate) schema_version: u32,
    pub(crate) id: String,
    pub(crate) query_key: String,
    pub(crate) created_ms: i64,
    pub(crate) expires_ms: i64,
    pub(crate) payload: Value,
}

#[derive(Clone)]
pub(crate) struct PoolStore {
    config: ResolvedCacheConfig,
}

impl PoolStore {
    pub(crate) fn new(config: ResolvedCacheConfig) -> Self {
        Self { config }
    }

    pub(crate) fn snapshot(&self, query_key: &str, payload: Value) -> Snapshot {
        let created_ms = sqlite::now_ms();
        // Infinite HTTP caching must not make a query session immortal.
        let ttl = self.config.max_age.min(crate::sources::DAY_CACHE_TTL);
        Snapshot {
            schema_version: SCHEMA_VERSION,
            id: uuid::Uuid::new_v4().to_string(),
            query_key: query_key.to_owned(),
            created_ms,
            expires_ms: created_ms.saturating_add(ttl.as_millis().min(i64::MAX as u128) as i64),
            payload,
        }
    }

    /// One transaction publishes the complete record and newest pointer.
    /// Successor publication never moves the legacy active pointer.
    pub(crate) fn publish(&self, snapshot: &Snapshot) -> io::Result<()> {
        if crate::sources::cache_is_bypassed() {
            return Ok(());
        }
        let bytes = serde_json::to_vec(snapshot).map_err(io::Error::other)?;
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        store.transaction(|tx, root| {
            // Snapshot IDs are immutable, even if a caller retries publication.
            if let Some(old) = sqlite::read_record(tx, root, SNAPSHOTS, &snapshot.id)? {
                if old.body != bytes {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "article pool ID already exists",
                    ));
                }
                return Ok(());
            }
            sqlite::put_record(
                tx,
                root,
                SNAPSHOTS,
                &snapshot.id,
                &snapshot.query_key,
                &[],
                &bytes,
                Some(snapshot.expires_ms),
            )?;
            sqlite::put_record(
                tx,
                root,
                POINTERS,
                &format!("newest:{}", snapshot.query_key),
                &snapshot.query_key,
                &[],
                snapshot.id.as_bytes(),
                Some(snapshot.expires_ms),
            )
        })?;
        sqlite::maintain(
            &mut store,
            &self.config,
            None,
            None,
            false,
            sqlite::Maintenance::Pressure,
        )?;
        Ok(())
    }

    pub(crate) fn read(&self, query_key: &str, id: &str) -> io::Result<Option<Snapshot>> {
        if crate::sources::cache_is_bypassed() {
            return Ok(None);
        }
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        let Some(record) = store.get(SNAPSHOTS, id)? else {
            return Ok(None);
        };
        let snapshot: Snapshot = serde_json::from_slice(&record.body).map_err(io::Error::other)?;
        if snapshot.schema_version != SCHEMA_VERSION
            || snapshot.id != id
            || snapshot.query_key != query_key
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "article pool identity or version mismatch",
            ));
        }
        Ok((snapshot.expires_ms > sqlite::now_ms()).then_some(snapshot))
    }

    pub(crate) fn selected(&self, query_key: &str, active: bool) -> io::Result<Option<Snapshot>> {
        if crate::sources::cache_is_bypassed() {
            return Ok(None);
        }
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        let kind = if active { "active" } else { "newest" };
        let Some(record) = store.get(POINTERS, &format!("{kind}:{query_key}"))? else {
            return Ok(None);
        };
        let id = std::str::from_utf8(&record.body).map_err(io::Error::other)?;
        self.read(query_key, id)
    }

    /// Only an explicit offset-zero session start calls this operation.
    pub(crate) fn activate(&self, snapshot: &Snapshot) -> io::Result<()> {
        if crate::sources::cache_is_bypassed() {
            return Ok(());
        }
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        store.put(
            POINTERS,
            &format!("active:{}", snapshot.query_key),
            &snapshot.query_key,
            &[],
            snapshot.id.as_bytes(),
            Some(snapshot.expires_ms),
        )
    }

    pub(crate) fn progress(&self, query_key: &str, source: &str) -> io::Result<Option<Value>> {
        if crate::sources::cache_is_bypassed() {
            return Ok(None);
        }
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        let Some(record) = store.get(PROGRESS, &format!("{query_key}:{source}"))? else {
            return Ok(None);
        };
        let value: Snapshot = serde_json::from_slice(&record.body).map_err(io::Error::other)?;
        if value.schema_version != SCHEMA_VERSION || value.query_key != query_key {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "article source progress version mismatch",
            ));
        }
        Ok((value.expires_ms > sqlite::now_ms()).then_some(value.payload))
    }

    pub(crate) fn save_progress(
        &self,
        query_key: &str,
        source: &str,
        payload: Value,
    ) -> io::Result<()> {
        if crate::sources::cache_is_bypassed() {
            return Ok(());
        }
        let value = self.snapshot(query_key, payload);
        let bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
        let mut store = sqlite::Store::open(&self.config.cache_root)?;
        store.put(
            PROGRESS,
            &format!("{query_key}:{source}"),
            query_key,
            &[],
            &bytes,
            Some(value.expires_ms),
        )?;
        sqlite::maintain(
            &mut store,
            &self.config,
            None,
            None,
            false,
            sqlite::Maintenance::Pressure,
        )?;
        Ok(())
    }
}
