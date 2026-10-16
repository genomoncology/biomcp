use std::io;
use std::path::Path;

use super::{CacheSnapshot, ResolvedCacheConfig};
use crate::error::BioMcpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FilesystemSpace {
    pub(crate) available_bytes: u64,
    pub(crate) total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CacheUsage {
    pub(crate) total_blob_bytes: u64,
    pub(crate) referenced_blob_bytes: u64,
    pub(crate) orphan_blob_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CacheLimitEvaluation {
    pub(crate) usage: CacheUsage,
    pub(crate) over_max_size: bool,
    pub(crate) below_min_disk_free: bool,
    pub(crate) disk_deficit_bytes: u64,
    pub(crate) effective_max_size: u64,
}

pub(crate) fn summarize_cache_usage(snapshot: &CacheSnapshot) -> CacheUsage {
    let total_blob_bytes = snapshot
        .blobs
        .iter()
        .map(|blob| blob.size_bytes)
        .sum::<u64>();
    let referenced_blob_bytes = snapshot
        .blobs
        .iter()
        .filter(|blob| blob.refcount > 0)
        .map(|blob| blob.size_bytes)
        .sum::<u64>();

    CacheUsage {
        total_blob_bytes,
        referenced_blob_bytes,
        orphan_blob_bytes: total_blob_bytes.saturating_sub(referenced_blob_bytes),
    }
}

pub(crate) fn inspect_filesystem_space(path: &Path) -> Result<FilesystemSpace, BioMcpError> {
    let available_bytes = available_space(path).map_err(|err| {
        BioMcpError::Io(std::io::Error::new(
            err.kind(),
            format!(
                "failed to inspect available filesystem space at {}: {err}",
                path.display()
            ),
        ))
    })?;
    let total_bytes = total_space(path).map_err(|err| {
        BioMcpError::Io(std::io::Error::new(
            err.kind(),
            format!(
                "failed to inspect total filesystem space at {}: {err}",
                path.display()
            ),
        ))
    })?;

    Ok(FilesystemSpace {
        available_bytes,
        total_bytes,
    })
}

/// Available is the space an unprivileged caller may use, as fs2 read it.
#[cfg(unix)]
fn available_space(path: &Path) -> io::Result<u64> {
    let stats = statvfs(path)?;
    Ok(u64::from(stats.f_bavail) * u64::from(stats.f_frsize))
}

#[cfg(unix)]
fn total_space(path: &Path) -> io::Result<u64> {
    let stats = statvfs(path)?;
    Ok(u64::from(stats.f_blocks) * u64::from(stats.f_frsize))
}

#[cfg(unix)]
fn statvfs(path: &Path) -> io::Result<libc::statvfs> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes())?;
    let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: c_path is NUL-terminated and stats outlives the call.
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stats) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(stats)
}

#[cfg(windows)]
fn available_space(path: &Path) -> io::Result<u64> {
    disk_space(path).map(|(available, _)| available)
}

#[cfg(windows)]
fn total_space(path: &Path) -> io::Result<u64> {
    disk_space(path).map(|(_, total)| total)
}

#[cfg(windows)]
fn disk_space(path: &Path) -> io::Result<(u64, u64)> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut available: u64 = 0;
    let mut total: u64 = 0;
    // SAFETY: wide is NUL-terminated and both out pointers are valid.
    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            &mut total,
            std::ptr::null_mut(),
        )
    };
    if result == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((available, total))
}

pub(crate) fn evaluate_cache_limits(
    snapshot: &CacheSnapshot,
    config: &ResolvedCacheConfig,
    space: FilesystemSpace,
) -> CacheLimitEvaluation {
    let usage = summarize_cache_usage(snapshot);
    let disk_deficit_bytes = config
        .min_disk_free
        .required_free_bytes(space.total_bytes)
        .saturating_sub(space.available_bytes);
    let below_min_disk_free = disk_deficit_bytes > 0;
    let bytes_needed_from_referenced = disk_deficit_bytes.saturating_sub(usage.orphan_blob_bytes);
    let disk_floor_target = usage
        .referenced_blob_bytes
        .saturating_sub(bytes_needed_from_referenced);

    CacheLimitEvaluation {
        usage,
        over_max_size: usage.referenced_blob_bytes > config.max_size,
        below_min_disk_free,
        disk_deficit_bytes,
        effective_max_size: if below_min_disk_free {
            config.max_size.min(disk_floor_target)
        } else {
            config.max_size
        },
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;
    use std::time::Duration;

    use super::{FilesystemSpace, evaluate_cache_limits, summarize_cache_usage};
    use crate::cache::{
        CacheConfigOrigins, ConfigOrigin, DiskFreeThreshold, ResolvedCacheConfig, snapshot_cache,
    };
    use crate::test_support::TempDirGuard;

    fn write_entry(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) {
        let mut writer = cacache::WriteOpts::new()
            .size(bytes.len())
            .time(time_ms)
            .open_sync(cache_path, key)
            .expect("open writer");
        writer.write_all(bytes).expect("write bytes");
        writer.commit().expect("commit writer");
    }

    fn make_orphan(cache_path: &Path, key: &str, bytes: &[u8], time_ms: u128) {
        write_entry(cache_path, key, bytes, time_ms);
        cacache::remove_sync(cache_path, key).expect("remove orphaned key");
    }

    fn test_config(
        cache_root: &Path,
        max_size: u64,
        min_disk_free: DiskFreeThreshold,
    ) -> ResolvedCacheConfig {
        ResolvedCacheConfig {
            cache_root: cache_root.to_path_buf(),
            max_size,
            min_disk_free,
            max_age: Duration::from_secs(86_400),
            origins: CacheConfigOrigins {
                cache_root: ConfigOrigin::Default,
                max_size: ConfigOrigin::Default,
                min_disk_free: ConfigOrigin::Default,
                max_age: ConfigOrigin::Default,
            },
        }
    }

    #[test]
    fn summarize_cache_usage_distinguishes_referenced_and_orphan_blob_bytes() {
        let root = TempDirGuard::new("usage");
        let cache_path = root.path().join("http");
        write_entry(&cache_path, "shared-a", b"shared-bytes", 100);
        write_entry(&cache_path, "shared-b", b"shared-bytes", 101);
        make_orphan(&cache_path, "orphan", b"orphan-bytes", 102);

        let snapshot = snapshot_cache(&cache_path).expect("snapshot");
        let usage = summarize_cache_usage(&snapshot);

        assert_eq!(
            usage.total_blob_bytes,
            b"shared-bytes".len() as u64 + b"orphan-bytes".len() as u64
        );
        assert_eq!(usage.referenced_blob_bytes, b"shared-bytes".len() as u64);
        assert_eq!(usage.orphan_blob_bytes, b"orphan-bytes".len() as u64);
    }

    #[test]
    fn evaluate_cache_limits_converts_disk_deficit_into_effective_max_size() {
        let root = TempDirGuard::new("disk-floor");
        let cache_path = root.path().join("http");
        write_entry(&cache_path, "retained", b"live-bytes", 100);
        make_orphan(&cache_path, "orphan", b"orph", 101);
        let snapshot = snapshot_cache(&cache_path).expect("snapshot");
        let config = test_config(root.path(), 12, DiskFreeThreshold::Percent(20));

        let evaluation = evaluate_cache_limits(
            &snapshot,
            &config,
            FilesystemSpace {
                available_bytes: 14,
                total_bytes: 100,
            },
        );

        assert!(!evaluation.over_max_size);
        assert!(evaluation.below_min_disk_free);
        assert_eq!(evaluation.disk_deficit_bytes, 6);
        assert_eq!(
            evaluation.effective_max_size,
            b"live-bytes".len() as u64 - 2
        );
    }

    #[test]
    fn evaluate_cache_limits_can_drive_effective_max_size_to_zero() {
        let root = TempDirGuard::new("disk-floor-zero");
        let cache_path = root.path().join("http");
        write_entry(&cache_path, "retained", b"live-bytes", 100);
        make_orphan(&cache_path, "orphan", b"orph", 101);
        let snapshot = snapshot_cache(&cache_path).expect("snapshot");
        let config = test_config(root.path(), 12, DiskFreeThreshold::Bytes(30));

        let evaluation = evaluate_cache_limits(
            &snapshot,
            &config,
            FilesystemSpace {
                available_bytes: 0,
                total_bytes: 100,
            },
        );

        assert!(evaluation.below_min_disk_free);
        assert_eq!(evaluation.effective_max_size, 0);
    }
}
