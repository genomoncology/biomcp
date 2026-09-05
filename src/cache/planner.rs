use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ssri::{Algorithm, Integrity};

#[derive(Debug, thiserror::Error)]
pub(crate) enum CachePlannerError {
    #[error("failed to inspect cache index at {cache_path}: {source}")]
    Index {
        cache_path: PathBuf,
        #[source]
        source: cacache::Error,
    },

    #[error("failed to inspect cache content tree at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheEntry {
    pub(crate) key: String,
    pub(crate) integrity: Integrity,
    pub(crate) time_ms: u128,
    pub(crate) size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheBlob {
    pub(crate) integrity: Integrity,
    pub(crate) path: PathBuf,
    pub(crate) size_bytes: u64,
    pub(crate) refcount: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheSnapshot {
    pub(crate) cache_path: PathBuf,
    pub(crate) entries: Vec<CacheEntry>,
    pub(crate) blobs: Vec<CacheBlob>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheCleanupPlan {
    pub(crate) entry_removals: Vec<CacheEntry>,
    pub(crate) blob_removals: Vec<CacheBlob>,
    pub(crate) reclaimed_blob_bytes: u64,
}

pub(crate) fn snapshot_cache(cache_path: &Path) -> Result<CacheSnapshot, CachePlannerError> {
    match fs::symlink_metadata(cache_path) {
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(CachePlannerError::Io {
                    path: cache_path.to_path_buf(),
                    source: io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{} is not a directory", cache_path.display()),
                    ),
                });
            }
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return Ok(CacheSnapshot {
                cache_path: cache_path.to_path_buf(),
                entries: Vec::new(),
                blobs: Vec::new(),
            });
        }
        Err(source) => {
            return Err(CachePlannerError::Io {
                path: cache_path.to_path_buf(),
                source,
            });
        }
    }

    let index_path = cache_path.join(super::INDEX_DIR);
    let mut entries = if path_exists(&index_path)? {
        let mut listed = Vec::new();
        for result in cacache::list_sync(cache_path) {
            let metadata = result.map_err(|source| CachePlannerError::Index {
                cache_path: cache_path.to_path_buf(),
                source,
            })?;
            listed.push(CacheEntry {
                key: metadata.key,
                integrity: metadata.integrity,
                time_ms: metadata.time,
                size_bytes: metadata.size as u64,
            });
        }
        listed
    } else {
        Vec::new()
    };
    entries.sort_by(|left, right| left.key.cmp(&right.key));

    let refcounts = entry_refcounts(&entries);
    let content_root = super::content_root(cache_path);
    let mut blobs = if path_exists(&content_root)? {
        walk_content_tree(cache_path, &content_root, &refcounts)?
    } else {
        Vec::new()
    };
    blobs.sort_by(blob_sort_key);

    Ok(CacheSnapshot {
        cache_path: cache_path.to_path_buf(),
        entries,
        blobs,
    })
}

fn path_exists(path: &Path) -> Result<bool, CachePlannerError> {
    path.try_exists().map_err(|source| CachePlannerError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn plan_orphan_gc(snapshot: &CacheSnapshot) -> CacheCleanupPlan {
    let blob_removals = snapshot
        .blobs
        .iter()
        .filter(|blob| blob.refcount == 0)
        .cloned()
        .collect::<Vec<_>>();
    CacheCleanupPlan {
        entry_removals: Vec::new(),
        reclaimed_blob_bytes: sum_blob_bytes(&blob_removals),
        blob_removals,
    }
}

pub(crate) fn plan_age_cleanup(
    snapshot: &CacheSnapshot,
    now_ms: u128,
    max_age: Duration,
) -> CacheCleanupPlan {
    let cutoff_ms = now_ms.saturating_sub(max_age.as_millis());
    let entry_removals = snapshot
        .entries
        .iter()
        .filter(|entry| entry.time_ms < cutoff_ms)
        .cloned()
        .collect::<Vec<_>>();
    let blob_removals = projected_blob_removals(snapshot, &entry_removals);
    CacheCleanupPlan {
        reclaimed_blob_bytes: sum_blob_bytes(&blob_removals),
        entry_removals,
        blob_removals,
    }
}

pub(crate) fn plan_size_lru(
    snapshot: &CacheSnapshot,
    max_referenced_blob_bytes: u64,
) -> CacheCleanupPlan {
    let blob_by_integrity = snapshot
        .blobs
        .iter()
        .map(|blob| (blob.integrity.clone(), blob))
        .collect::<HashMap<_, _>>();
    let mut remaining_refcounts = entry_refcounts(&snapshot.entries);
    let mut referenced_blob_bytes = snapshot
        .blobs
        .iter()
        .filter(|blob| blob.refcount > 0)
        .map(|blob| blob.size_bytes)
        .sum::<u64>();

    let mut candidates = snapshot.entries.clone();
    candidates.sort_by(|left, right| {
        left.time_ms
            .cmp(&right.time_ms)
            .then_with(|| left.key.cmp(&right.key))
    });

    let mut selected = Vec::new();
    for entry in candidates {
        if referenced_blob_bytes <= max_referenced_blob_bytes {
            break;
        }

        let integrity = entry.integrity.clone();
        // Size planning is driven by on-disk blob bytes only. Entries whose blobs are missing
        // (or zero bytes) cannot help the budget and should not displace live data.
        let Some(blob) = blob_by_integrity.get(&integrity) else {
            continue;
        };
        if blob.size_bytes == 0 {
            continue;
        }
        let previous_refcount = remaining_refcounts
            .get(&integrity)
            .copied()
            .unwrap_or_default();
        if previous_refcount > 0 {
            let next_refcount = previous_refcount - 1;
            if next_refcount == 0 {
                remaining_refcounts.remove(&integrity);
                referenced_blob_bytes = referenced_blob_bytes.saturating_sub(blob.size_bytes);
            } else {
                remaining_refcounts.insert(integrity.clone(), next_refcount);
            }
        }
        selected.push(entry);
    }

    let blob_removals = projected_blob_removals(snapshot, &selected);
    CacheCleanupPlan {
        reclaimed_blob_bytes: sum_blob_bytes(&blob_removals),
        entry_removals: selected,
        blob_removals,
    }
}

pub(crate) fn plan_composite_cleanup(
    snapshot: &CacheSnapshot,
    now_ms: u128,
    max_age: Option<Duration>,
    max_size: Option<u64>,
) -> CacheCleanupPlan {
    let orphan_plan = plan_orphan_gc(snapshot);
    let age_plan = max_age.map_or_else(CacheCleanupPlan::empty, |limit| {
        plan_age_cleanup(snapshot, now_ms, limit)
    });
    let projected_after_age =
        projected_snapshot_after_entry_removals(snapshot, &age_plan.entry_removals);
    let size_plan = max_size.map_or_else(CacheCleanupPlan::empty, |limit| {
        plan_size_lru(&projected_after_age, limit)
    });

    let mut entry_removals = age_plan.entry_removals;
    entry_removals.extend(size_plan.entry_removals);

    let mut blob_removals = orphan_plan.blob_removals;
    blob_removals.extend(age_plan.blob_removals);
    blob_removals.extend(size_plan.blob_removals);
    blob_removals = dedup_blob_removals(blob_removals);

    CacheCleanupPlan {
        reclaimed_blob_bytes: sum_blob_bytes(&blob_removals),
        entry_removals,
        blob_removals,
    }
}

fn walk_content_tree(
    cache_path: &Path,
    content_root: &Path,
    refcounts: &HashMap<Integrity, usize>,
) -> Result<Vec<CacheBlob>, CachePlannerError> {
    let metadata = fs::symlink_metadata(content_root).map_err(|source| CachePlannerError::Io {
        path: content_root.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() {
        return Err(CachePlannerError::Io {
            path: content_root.to_path_buf(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not a directory", content_root.display()),
            ),
        });
    }

    let mut blobs = Vec::new();
    walk_content_dir(
        cache_path,
        content_root,
        content_root,
        refcounts,
        &mut blobs,
    )?;
    Ok(blobs)
}

fn walk_content_dir(
    cache_path: &Path,
    content_root: &Path,
    current_dir: &Path,
    refcounts: &HashMap<Integrity, usize>,
    blobs: &mut Vec<CacheBlob>,
) -> Result<(), CachePlannerError> {
    let dir_entries = fs::read_dir(current_dir).map_err(|source| CachePlannerError::Io {
        path: current_dir.to_path_buf(),
        source,
    })?;

    for dir_entry in dir_entries {
        let dir_entry = dir_entry.map_err(|source| CachePlannerError::Io {
            path: current_dir.to_path_buf(),
            source,
        })?;
        let path = dir_entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| CachePlannerError::Io {
            path: path.clone(),
            source,
        })?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            walk_content_dir(cache_path, content_root, &path, refcounts, blobs)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        let Some(integrity) = integrity_from_blob_path(content_root, &path) else {
            continue;
        };
        blobs.push(CacheBlob {
            refcount: refcounts.get(&integrity).copied().unwrap_or_default(),
            integrity,
            path: path
                .strip_prefix(cache_path)
                .map_or_else(|_| path.clone(), PathBuf::from),
            size_bytes: metadata.len(),
        });
    }

    Ok(())
}

fn integrity_from_blob_path(content_root: &Path, blob_path: &Path) -> Option<Integrity> {
    let relative = blob_path.strip_prefix(content_root).ok()?;
    let mut parts = relative.iter();
    let algorithm = parts.next()?.to_str()?.parse::<Algorithm>().ok()?;
    let first = parts.next()?.to_str()?;
    let second = parts.next()?.to_str()?;
    let rest = parts.next()?.to_str()?;
    if parts.next().is_some() {
        return None;
    }
    if first.len() != 2 || second.len() != 2 || rest.is_empty() {
        return None;
    }
    let hex = format!("{first}{second}{rest}");
    Integrity::from_hex(hex, algorithm).ok()
}

fn blob_path_for_integrity(cache_path: &Path, integrity: &Integrity) -> PathBuf {
    super::content_path(cache_path, integrity)
}

fn entry_refcounts(entries: &[CacheEntry]) -> HashMap<Integrity, usize> {
    let mut refcounts = HashMap::new();
    for entry in entries {
        *refcounts.entry(entry.integrity.clone()).or_insert(0) += 1;
    }
    refcounts
}

impl CacheCleanupPlan {
    fn empty() -> Self {
        Self {
            entry_removals: Vec::new(),
            blob_removals: Vec::new(),
            reclaimed_blob_bytes: 0,
        }
    }
}

fn projected_snapshot_after_entry_removals(
    snapshot: &CacheSnapshot,
    entry_removals: &[CacheEntry],
) -> CacheSnapshot {
    let removed_keys = entry_removals
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<HashSet<_>>();
    let entries = snapshot
        .entries
        .iter()
        .filter(|entry| !removed_keys.contains(entry.key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let projected_refcounts = entry_refcounts(&entries);
    let mut blobs = snapshot
        .blobs
        .iter()
        .map(|blob| CacheBlob {
            refcount: projected_refcounts
                .get(&blob.integrity)
                .copied()
                .unwrap_or_default(),
            ..blob.clone()
        })
        .collect::<Vec<_>>();
    blobs.sort_by(blob_sort_key);

    CacheSnapshot {
        cache_path: snapshot.cache_path.clone(),
        entries,
        blobs,
    }
}

fn projected_blob_removals(
    snapshot: &CacheSnapshot,
    entry_removals: &[CacheEntry],
) -> Vec<CacheBlob> {
    let removed_keys = entry_removals
        .iter()
        .map(|entry| &entry.key)
        .collect::<HashSet<_>>();
    let projected_entries = snapshot
        .entries
        .iter()
        .filter(|entry| !removed_keys.contains(&entry.key))
        .cloned()
        .collect::<Vec<_>>();
    let projected_refcounts = entry_refcounts(&projected_entries);

    let mut blob_removals = snapshot
        .blobs
        .iter()
        .filter(|blob| {
            blob.refcount > 0
                && projected_refcounts
                    .get(&blob.integrity)
                    .copied()
                    .unwrap_or_default()
                    == 0
        })
        .cloned()
        .collect::<Vec<_>>();
    blob_removals.sort_by(blob_sort_key);
    blob_removals
}

fn blob_sort_key(left: &CacheBlob, right: &CacheBlob) -> std::cmp::Ordering {
    let (left_algorithm, left_hex) = left.integrity.to_hex();
    let (right_algorithm, right_hex) = right.integrity.to_hex();
    left_algorithm
        .to_string()
        .cmp(&right_algorithm.to_string())
        .then_with(|| left_hex.cmp(&right_hex))
}

fn dedup_blob_removals(blobs: Vec<CacheBlob>) -> Vec<CacheBlob> {
    let mut by_integrity = HashMap::new();
    for blob in blobs {
        by_integrity.entry(blob.integrity.clone()).or_insert(blob);
    }
    let mut deduped = by_integrity.into_values().collect::<Vec<_>>();
    deduped.sort_by(blob_sort_key);
    deduped
}

fn sum_blob_bytes(blobs: &[CacheBlob]) -> u64 {
    blobs.iter().map(|blob| blob.size_bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::{
        CacheBlob, CacheEntry, CachePlannerError, CacheSnapshot, blob_path_for_integrity,
        plan_age_cleanup, plan_composite_cleanup, plan_orphan_gc, plan_size_lru, snapshot_cache,
    };
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use crate::test_support::TempDirGuard;
    use ssri::Integrity;

    fn write_entry(cache_path: &Path, key: &str, bytes: &[u8], time_ms: Option<u128>) -> Integrity {
        match time_ms {
            Some(time_ms) => {
                let mut writer = cacache::WriteOpts::new()
                    .size(bytes.len())
                    .time(time_ms)
                    .open_sync(cache_path, key)
                    .expect("open writer");
                writer.write_all(bytes).expect("write bytes");
                writer.commit().expect("commit writer")
            }
            None => cacache::write_sync(cache_path, key, bytes).expect("write entry"),
        }
    }

    fn add_orphan_blob(cache_path: &Path, bytes: &[u8]) -> CacheBlob {
        let integrity = Integrity::from(bytes);
        let path = blob_path_for_integrity(cache_path, &integrity);
        fs::create_dir_all(path.parent().expect("orphan parent")).expect("create orphan dirs");
        fs::write(&path, bytes).expect("write orphan blob");
        CacheBlob {
            integrity,
            path: path
                .strip_prefix(cache_path)
                .expect("relative orphan path")
                .to_path_buf(),
            size_bytes: bytes.len() as u64,
            refcount: 0,
        }
    }

    #[test]
    fn snapshot_cache_returns_empty_snapshot_for_missing_cache_root() {
        let cache_root = TempDirGuard::new("missing").path().join("http");
        let snapshot = snapshot_cache(&cache_root).expect("missing root should be empty");
        assert!(snapshot.entries.is_empty());
        assert!(snapshot.blobs.is_empty());
        assert_eq!(snapshot.cache_path, cache_root);
    }

    #[test]
    fn snapshot_cache_returns_empty_snapshot_for_uninitialized_cache_root() {
        let root = TempDirGuard::new("uninitialized");
        let cache_root = root.path().join("http");
        fs::create_dir_all(&cache_root).expect("create cache root");

        let snapshot = snapshot_cache(&cache_root).expect("uninitialized root should be empty");
        assert!(snapshot.entries.is_empty());
        assert!(snapshot.blobs.is_empty());
    }

    #[test]
    fn snapshot_cache_errors_when_cache_root_is_not_a_directory() {
        let root = TempDirGuard::new("root-file");
        let cache_root = root.path().join("http");
        fs::write(&cache_root, b"not a directory").expect("write cache root file");

        let err = snapshot_cache(&cache_root).expect_err("file cache root should fail");
        match err {
            CachePlannerError::Io { path, .. } => {
                assert_eq!(path, cache_root);
            }
            other => panic!("expected io error, got {other:?}"),
        }
    }

    #[test]
    fn snapshot_cache_reports_seeded_entries_and_blobs_deterministically() {
        let root = TempDirGuard::new("seeded");
        let cache_root = root.path().join("http");
        let second = write_entry(&cache_root, "z-key", b"two", Some(200));
        let first = write_entry(&cache_root, "a-key", b"one", Some(100));

        let snapshot = snapshot_cache(&cache_root).expect("seeded cache should snapshot");

        assert_eq!(
            snapshot.entries,
            vec![
                CacheEntry {
                    key: "a-key".into(),
                    integrity: first.clone(),
                    time_ms: 100,
                    size_bytes: 3,
                },
                CacheEntry {
                    key: "z-key".into(),
                    integrity: second.clone(),
                    time_ms: 200,
                    size_bytes: 3,
                },
            ]
        );
        let mut expected_blobs = vec![
            CacheBlob {
                integrity: first.clone(),
                path: blob_path_for_integrity(&cache_root, &first)
                    .strip_prefix(&cache_root)
                    .expect("relative blob path")
                    .to_path_buf(),
                size_bytes: 3,
                refcount: 1,
            },
            CacheBlob {
                integrity: second.clone(),
                path: blob_path_for_integrity(&cache_root, &second)
                    .strip_prefix(&cache_root)
                    .expect("relative blob path")
                    .to_path_buf(),
                size_bytes: 3,
                refcount: 1,
            },
        ];
        expected_blobs.sort_by(super::blob_sort_key);
        assert_eq!(snapshot.blobs, expected_blobs);
    }

    #[test]
    fn snapshot_cache_includes_orphan_blobs_without_synthesizing_missing_referenced_blobs() {
        let root = TempDirGuard::new("orphan-and-drift");
        let cache_root = root.path().join("http");
        let retained = write_entry(&cache_root, "retained", b"retained", Some(100));
        let missing = write_entry(&cache_root, "missing", b"missing", Some(200));
        let orphan = add_orphan_blob(&cache_root, b"orphan-bytes");
        cacache::remove_hash_sync(&cache_root, &missing).expect("remove referenced blob");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");

        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot.blobs.iter().any(|blob| blob.integrity == retained));
        assert!(
            snapshot
                .blobs
                .iter()
                .any(|blob| blob.integrity == orphan.integrity && blob.refcount == 0)
        );
        assert!(
            snapshot.blobs.iter().all(|blob| blob.integrity != missing),
            "missing referenced blob should not be synthesized into snapshot.blobs"
        );
    }

    #[test]
    fn planner_walk_errors_when_content_root_is_not_a_directory() {
        let root = TempDirGuard::new("content-file");
        let cache_root = root.path().join("http");
        fs::create_dir_all(&cache_root).expect("create cache root");
        fs::write(cache_root.join("content-v2"), b"not a dir").expect("write content file");

        let err = snapshot_cache(&cache_root).expect_err("content-v2 file should fail");
        match err {
            CachePlannerError::Io { path, .. } => {
                assert_eq!(path, cache_root.join("content-v2"));
            }
            other => panic!("expected io error, got {other:?}"),
        }
    }

    #[test]
    fn planner_walk_skips_malformed_blob_leaves() {
        let root = TempDirGuard::new("malformed-leaf");
        let cache_root = root.path().join("http");
        let live = write_entry(&cache_root, "live", b"live-bytes", Some(100));
        let malformed = cache_root
            .join("content-v2")
            .join("sha256")
            .join("zz")
            .join("yy")
            .join("not-hex");
        fs::create_dir_all(malformed.parent().expect("malformed parent"))
            .expect("create malformed path");
        fs::write(&malformed, b"malformed").expect("write malformed leaf");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        assert_eq!(snapshot.blobs.len(), 1);
        assert_eq!(snapshot.blobs[0].integrity, live);
    }

    #[test]
    fn composite_cleanup_plans_age_then_size_on_one_snapshot() {
        let old = Integrity::from(b"old");
        let new = Integrity::from(b"new");
        let snapshot = CacheSnapshot {
            cache_path: PathBuf::from("/tmp/cache/http"),
            entries: vec![
                CacheEntry {
                    key: "old".into(),
                    integrity: old.clone(),
                    time_ms: 100,
                    size_bytes: 3,
                },
                CacheEntry {
                    key: "new".into(),
                    integrity: new.clone(),
                    time_ms: 900,
                    size_bytes: 3,
                },
            ],
            blobs: vec![
                CacheBlob {
                    integrity: old.clone(),
                    path: PathBuf::from("content-v2/sha512/aa/bb/old"),
                    size_bytes: 3,
                    refcount: 1,
                },
                CacheBlob {
                    integrity: new.clone(),
                    path: PathBuf::from("content-v2/sha512/cc/dd/new"),
                    size_bytes: 3,
                    refcount: 1,
                },
            ],
        };

        let plan =
            plan_composite_cleanup(&snapshot, 1_000, Some(Duration::from_millis(500)), Some(3));

        assert_eq!(
            plan.entry_removals
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["old"]
        );
        assert_eq!(
            plan.blob_removals
                .iter()
                .map(|blob| blob.integrity.clone())
                .collect::<Vec<_>>(),
            vec![old]
        );
        assert_eq!(plan.reclaimed_blob_bytes, 3);
    }

    #[cfg(unix)]
    #[test]
    fn planner_walk_skips_symlinked_content_entries() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("symlink");
        let cache_root = root.path().join("http");
        let integrity = write_entry(&cache_root, "live", b"live-bytes", Some(100));
        let content_root = cache_root.join("content-v2");
        let target = blob_path_for_integrity(&cache_root, &integrity);
        let symlink_path = content_root.join("sha256").join("ff");
        fs::create_dir_all(symlink_path.parent().expect("symlink parent")).expect("create path");
        symlink(&target, &symlink_path).expect("create symlink");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        assert_eq!(snapshot.blobs.len(), 1);
    }

    #[test]
    fn plan_orphan_gc_selects_only_zero_refcount_blobs() {
        let root = TempDirGuard::new("orphan-plan");
        let cache_root = root.path().join("http");
        write_entry(&cache_root, "live", b"live", Some(100));
        let orphan = add_orphan_blob(&cache_root, b"orphan");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        let plan = plan_orphan_gc(&snapshot);

        assert!(plan.entry_removals.is_empty());
        assert_eq!(plan.blob_removals.len(), 1);
        assert_eq!(plan.blob_removals[0].integrity, orphan.integrity);
        assert_eq!(plan.reclaimed_blob_bytes, orphan.size_bytes);
    }

    #[test]
    fn shared_integrity_age_cleanup_only_removes_blob_after_last_reference() {
        let root = TempDirGuard::new("age-shared");
        let cache_root = root.path().join("http");
        let integrity = write_entry(&cache_root, "older", b"shared-bytes", Some(100));
        let second = write_entry(&cache_root, "newer", b"shared-bytes", Some(500));
        assert_eq!(integrity, second);

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        let keep_one_plan = plan_age_cleanup(&snapshot, 600, std::time::Duration::from_millis(450));
        assert_eq!(
            keep_one_plan
                .entry_removals
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["older"]
        );
        assert!(keep_one_plan.blob_removals.is_empty());
        assert_eq!(keep_one_plan.reclaimed_blob_bytes, 0);

        let remove_both_plan =
            plan_age_cleanup(&snapshot, 1_000, std::time::Duration::from_millis(100));
        let shared_blob_size = snapshot.blobs[0].size_bytes;
        assert_eq!(
            remove_both_plan
                .entry_removals
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["newer", "older"]
        );
        assert_eq!(remove_both_plan.blob_removals.len(), 1);
        assert_eq!(remove_both_plan.reclaimed_blob_bytes, shared_blob_size);
    }

    #[test]
    fn shared_integrity_size_lru_uses_projected_refcounts_and_oldest_first_tiebreak() {
        let root = TempDirGuard::new("size-shared");
        let cache_root = root.path().join("http");
        let shared = write_entry(&cache_root, "a-first", b"shared-bytes", Some(100));
        let shared_again = write_entry(&cache_root, "b-second", b"shared-bytes", Some(100));
        let unique = write_entry(&cache_root, "c-third", b"unique", Some(200));
        assert_eq!(shared, shared_again);

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        let unique_blob_size = snapshot
            .blobs
            .iter()
            .find(|blob| blob.integrity == unique)
            .expect("unique blob")
            .size_bytes;
        let plan = plan_size_lru(&snapshot, unique_blob_size);

        assert_eq!(
            plan.entry_removals
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["a-first", "b-second"]
        );
        assert_eq!(plan.blob_removals.len(), 1);
        assert_eq!(plan.blob_removals[0].integrity, shared);
        assert_eq!(plan.reclaimed_blob_bytes, plan.blob_removals[0].size_bytes);
    }

    #[test]
    fn size_lru_missing_blob_entries_do_not_displace_live_entries() {
        let root = TempDirGuard::new("size-missing");
        let cache_root = root.path().join("http");
        let missing = write_entry(&cache_root, "a-missing", b"stale-bytes", Some(100));
        let live = write_entry(&cache_root, "b-live", b"live-bytes", Some(200));
        cacache::remove_hash_sync(&cache_root, &missing).expect("remove missing blob");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        let plan = plan_size_lru(&snapshot, 0);

        assert_eq!(
            plan.entry_removals
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["b-live"]
        );
        assert_eq!(plan.blob_removals.len(), 1);
        assert_eq!(plan.blob_removals[0].integrity, live);
    }

    #[test]
    fn size_lru_ignores_orphan_bytes() {
        let root = TempDirGuard::new("size-drift");
        let cache_root = root.path().join("http");
        let kept = write_entry(&cache_root, "kept", b"live-bytes", Some(100));
        let _orphan = add_orphan_blob(&cache_root, b"large-orphan-blob");

        let snapshot = snapshot_cache(&cache_root).expect("snapshot should succeed");
        let kept_blob_size = snapshot
            .blobs
            .iter()
            .find(|blob| blob.integrity == kept)
            .expect("kept blob")
            .size_bytes;

        let plan = plan_size_lru(&snapshot, kept_blob_size);

        assert!(
            plan.entry_removals.is_empty(),
            "orphan bytes should not force eviction"
        );
        assert!(plan.blob_removals.is_empty());
        assert_eq!(plan.reclaimed_blob_bytes, 0);
    }
}
