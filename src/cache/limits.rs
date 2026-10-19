use std::io;
use std::path::Path;

use super::ResolvedCacheConfig;
use crate::error::BioMcpError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FilesystemSpace {
    pub(crate) available_bytes: u64,
    pub(crate) total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CacheUsage {
    pub(crate) referenced_blob_bytes: u64,
    pub(crate) orphan_blob_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CacheLimitEvaluation {
    pub(crate) usage: CacheUsage,
    pub(crate) over_max_size: bool,
    pub(crate) below_min_disk_free: bool,
    pub(crate) effective_max_size: u64,
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
/// The products run through u128 because the field widths differ per
/// Unix platform (u64 on Linux, u32 on macOS) and cannot widen in place.
#[cfg(unix)]
fn available_space(path: &Path) -> io::Result<u64> {
    let stats = statvfs(path)?;
    Ok((stats.f_bavail as u128 * stats.f_frsize as u128) as u64)
}

#[cfg(unix)]
fn total_space(path: &Path) -> io::Result<u64> {
    let stats = statvfs(path)?;
    Ok((stats.f_blocks as u128 * stats.f_frsize as u128) as u64)
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

pub(crate) fn evaluate_cache_usage(
    usage: CacheUsage,
    config: &ResolvedCacheConfig,
    space: FilesystemSpace,
) -> CacheLimitEvaluation {
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
        effective_max_size: if below_min_disk_free {
            config.max_size.min(disk_floor_target)
        } else {
            config.max_size
        },
    }
}
