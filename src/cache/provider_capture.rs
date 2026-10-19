use super::sqlite::{self, sqlite_error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

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
}

impl ProviderCaptureStore {
    pub(crate) fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            max_retained_bytes: MAX_RETAINED_BYTES,
        }
    }
    #[cfg(test)]
    fn with_max_retained_bytes(mut self, limit: u64) -> Self {
        self.max_retained_bytes = limit;
        self
    }

    #[cfg(test)]
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
        body: R,
        binding: Option<CspecCaptureBinding>,
    ) -> Result<ProviderCaptureManifest, ProviderCaptureError> {
        let mut bytes = Vec::new();
        body.take(MAX_CAPTURE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        if bytes.len() as u64 > MAX_CAPTURE_BYTES {
            return Err(ProviderCaptureError::Oversize);
        }
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let now = (sqlite::now_ms() / 1000) as u64;
        let manifest = ProviderCaptureManifest {
            capture_id: format!("capture:{}:sha256:{sha256}", provider.as_str()),
            provider,
            media_type: media_type.into(),
            byte_length: bytes.len() as u64,
            sha256,
            captured_at: now,
            expires_at: now + RETENTION.as_secs(),
            schema_version: SCHEMA_VERSION,
            capture_binding: binding,
        };
        let mut store = self.store()?;
        let result = store
            .transaction(|tx, root| {
                if let Some(old) = sqlite::read_record(tx, root, "capture", &manifest.capture_id)? {
                    let old: ProviderCaptureManifest =
                        serde_json::from_slice(&old.metadata).map_err(io::Error::other)?;
                    if old.expires_at > now {
                        if old.capture_binding != manifest.capture_binding {
                            return Ok(Err(ProviderCaptureError::BindingConflict));
                        }
                        return Ok(Ok(old));
                    }
                }
                let metadata = serde_json::to_vec(&manifest).map_err(io::Error::other)?;
                sqlite::put_record(
                    tx,
                    root,
                    "capture",
                    &manifest.capture_id,
                    &manifest.capture_id,
                    &metadata,
                    &bytes,
                    Some((manifest.expires_at * 1000) as i64),
                )?;
                self.maintain_transaction(tx, root, false)?;
                Ok(Ok(manifest.clone()))
            })
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        store
            .collect_blobs()
            .map_err(|_| ProviderCaptureError::Corrupt)?;
        result
    }
    fn store(&self) -> Result<sqlite::Store, ProviderCaptureError> {
        sqlite::Store::open(&self.root).map_err(|_| ProviderCaptureError::Corrupt)
    }
    pub(crate) fn read_complete(
        &self,
        id: &str,
    ) -> Result<(ProviderCaptureManifest, Vec<u8>), ProviderCaptureError> {
        let parts = id.split(':').collect::<Vec<_>>();
        if parts.len() != 4 || parts[0] != "capture" || parts[2] != "sha256" {
            return Err(ProviderCaptureError::Unavailable);
        }
        ProviderCaptureProvider::parse(parts[1])?;
        let record = self
            .store()?
            .get("capture", id)
            .map_err(|_| ProviderCaptureError::Corrupt)?
            .ok_or(ProviderCaptureError::Unavailable)?;
        let manifest: ProviderCaptureManifest =
            serde_json::from_slice(&record.metadata).map_err(|_| ProviderCaptureError::Corrupt)?;
        if manifest.capture_id != id
            || manifest.schema_version != SCHEMA_VERSION
            || manifest.byte_length != record.body.len() as u64
            || manifest.sha256 != format!("{:x}", Sha256::digest(&record.body))
            || manifest
                .capture_binding
                .as_ref()
                .is_some_and(|b| b.binding_schema_version != 1)
        {
            return Err(ProviderCaptureError::Corrupt);
        }
        if manifest.expires_at <= (sqlite::now_ms() / 1000) as u64 {
            return Err(ProviderCaptureError::Unavailable);
        }
        Ok((manifest, record.body))
    }
    #[cfg(test)]
    pub(crate) fn read(&self, id: &str) -> Result<Vec<u8>, ProviderCaptureError> {
        self.read_complete(id).map(|(_, b)| b)
    }
    #[cfg(test)]
    pub(crate) fn retained_bytes(&self) -> Result<u64, ProviderCaptureError> {
        if !self.root.join(sqlite::DATABASE_NAME).exists() {
            return Ok(0);
        }
        self.store()?
            .connection()
            .query_row(
                "SELECT coalesce(sum(bytes),0) FROM entries WHERE namespace='capture'",
                [],
                |r| sqlite::read_u64(r, 0),
            )
            .map_err(|_| ProviderCaptureError::Corrupt)
    }
    fn maintain_transaction(
        &self,
        tx: &rusqlite::Transaction<'_>,
        root: &Path,
        dry_run: bool,
    ) -> io::Result<u64> {
        let mut total: u64 = tx
            .query_row(
                "SELECT coalesce(sum(bytes),0) FROM entries WHERE namespace='capture'",
                [],
                |r| sqlite::read_u64(r, 0),
            )
            .map_err(sqlite_error)?;
        let now = sqlite::now_ms();
        let mut statement=tx.prepare("SELECT key,bytes,expires FROM entries WHERE namespace='capture' ORDER BY last_access").map_err(sqlite_error)?;
        let rows = statement
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    sqlite::read_u64(r, 1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(sqlite_error)?;
        let mut freed = 0;
        for row in rows {
            let (key, bytes, expires) = row.map_err(sqlite_error)?;
            if expires > now && total <= self.max_retained_bytes {
                continue;
            }
            if !dry_run {
                sqlite::delete_record(tx, root, "capture", &key)?;
            }
            total = total.saturating_sub(bytes);
            freed += bytes;
        }
        Ok(freed)
    }
    pub(crate) fn clean_in_store(
        &self,
        store: &mut sqlite::Store,
        dry_run: bool,
    ) -> io::Result<u64> {
        let freed = store.transaction(|tx, root| self.maintain_transaction(tx, root, dry_run))?;
        if !dry_run {
            store.collect_blobs()?;
        }
        Ok(freed)
    }
    #[cfg(test)]
    pub(crate) fn maintain(&self) -> Result<u64, ProviderCaptureError> {
        self.store()?
            .transaction(|tx, root| self.maintain_transaction(tx, root, false))
            .map_err(|_| ProviderCaptureError::Corrupt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDirGuard;

    #[test]
    #[serial_test::serial(source_env)]
    fn explicit_capture_survives_restart_and_optional_cache_bypass() {
        let root = TempDirGuard::new("sqlite-explicit-capture");
        let _bypass = crate::sources::test_cache_mode::off();
        let store = ProviderCaptureStore::new(root.path());
        let first = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"exact bytes")
            .unwrap();
        let same = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"exact bytes")
            .unwrap();
        assert_eq!(first, same);
        assert_eq!(
            ProviderCaptureStore::new(root.path()).read(&first.capture_id),
            Ok(b"exact bytes".to_vec())
        );
        let connection =
            rusqlite::Connection::open(root.path().join(sqlite::DATABASE_NAME)).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT count(*) FROM entries WHERE namespace='http'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn captures_keep_binding_identity_and_detect_corruption_and_expiry() {
        let root = TempDirGuard::new("sqlite-capture-identity");
        let store = ProviderCaptureStore::new(root.path());
        let binding = CspecCaptureBinding {
            binding_schema_version: 1,
            normalized_gene: "BRAF".into(),
            resource_iri: "https://example.test/cspec".into(),
            specification_id: "example".into(),
        };
        let manifest = store
            .capture_cspec_bytes(binding.clone(), b"exact bytes")
            .unwrap();
        let mut other = binding;
        other.normalized_gene = "other".into();
        assert_eq!(
            store.capture_cspec_bytes(other, b"exact bytes"),
            Err(ProviderCaptureError::BindingConflict)
        );
        let connection =
            rusqlite::Connection::open(root.path().join(sqlite::DATABASE_NAME)).unwrap();
        connection
            .execute(
                "UPDATE entries SET body=?1 WHERE key=?2",
                rusqlite::params![b"corrupt".as_slice(), manifest.capture_id],
            )
            .unwrap();
        assert_eq!(
            store.read(&manifest.capture_id),
            Err(ProviderCaptureError::Corrupt)
        );
        let expired = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", b"other bytes")
            .unwrap();
        let mut metadata = expired.clone();
        metadata.expires_at = 0;
        connection
            .execute(
                "UPDATE entries SET metadata=?1,expires=0 WHERE key=?2",
                rusqlite::params![serde_json::to_vec(&metadata).unwrap(), expired.capture_id],
            )
            .unwrap();
        assert_eq!(
            store.read(&expired.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
        assert!(store.maintain().unwrap() > 0);
        assert_eq!(
            store.read(&expired.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
    }

    #[test]
    fn enforces_namespace_capacity_with_deterministic_lru_eviction() {
        let root = TempDirGuard::new("sqlite-capture-capacity");
        let store = ProviderCaptureStore::new(root.path()).with_max_retained_bytes(1024);
        let first = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", &[1; 256])
            .unwrap();
        let connection =
            rusqlite::Connection::open(root.path().join(sqlite::DATABASE_NAME)).unwrap();
        connection
            .execute("UPDATE entries SET last_access=1", [])
            .unwrap();
        let second = store
            .capture_bytes(ProviderCaptureProvider::Cspec, "text/plain", &[2; 256])
            .unwrap();
        assert_eq!(
            store.read(&first.capture_id),
            Err(ProviderCaptureError::Unavailable)
        );
        assert_eq!(store.read(&second.capture_id), Ok(vec![2; 256]));
        assert!(store.retained_bytes().unwrap() <= 1024);
    }
}
