//! Platform-specific atomic directory exchange used by forced skill repair.

use std::ffi::CString;
use std::path::Path;

use crate::error::BioMcpError;

#[cfg(test)]
static FAIL_NEXT_EXCHANGE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(test)]
pub(super) fn fail_next_exchange() {
    FAIL_NEXT_EXCHANGE.store(true, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
fn injected_failure() -> bool {
    FAIL_NEXT_EXCHANGE.swap(false, std::sync::atomic::Ordering::SeqCst)
}

#[cfg(unix)]
fn c_path(path: &Path) -> Result<CString, BioMcpError> {
    use std::os::unix::ffi::OsStrExt;
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| BioMcpError::InvalidArgument("Skill install path contains a NUL byte".into()))
}

#[cfg(target_os = "linux")]
pub(super) fn rename_absent(source: &Path, destination: &Path) -> Result<(), BioMcpError> {
    let source = c_path(source)?;
    let destination = c_path(destination)?;
    // SAFETY: both C strings are NUL-terminated and remain alive for the call.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(BioMcpError::Io(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "macos")]
pub(super) fn rename_absent(source: &Path, destination: &Path) -> Result<(), BioMcpError> {
    let source = c_path(source)?;
    let destination = c_path(destination)?;
    // SAFETY: both C strings are NUL-terminated and remain alive for the call.
    let result =
        unsafe { libc::renamex_np(source.as_ptr(), destination.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(BioMcpError::Io(std::io::Error::last_os_error()))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn rename_absent(_source: &Path, _destination: &Path) -> Result<(), BioMcpError> {
    Err(BioMcpError::InvalidArgument(
        "Atomic skill installation is unsupported on this platform".into(),
    ))
}

#[cfg(target_os = "linux")]
pub(super) fn exchange_directories(left: &Path, right: &Path) -> Result<(), BioMcpError> {
    #[cfg(test)]
    if injected_failure() {
        return Err(BioMcpError::Io(std::io::Error::other(
            "injected atomic exchange failure",
        )));
    }
    let left = c_path(left)?;
    let right = c_path(right)?;
    // SAFETY: both C strings are NUL-terminated and remain alive for the call.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            left.as_ptr(),
            libc::AT_FDCWD,
            right.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(BioMcpError::Io(std::io::Error::last_os_error()))
    }
}

#[cfg(target_os = "macos")]
pub(super) fn exchange_directories(left: &Path, right: &Path) -> Result<(), BioMcpError> {
    #[cfg(test)]
    if injected_failure() {
        return Err(BioMcpError::Io(std::io::Error::other(
            "injected atomic exchange failure",
        )));
    }
    let left = c_path(left)?;
    let right = c_path(right)?;
    // SAFETY: both C strings are NUL-terminated and remain alive for the call.
    let result = unsafe { libc::renamex_np(left.as_ptr(), right.as_ptr(), libc::RENAME_SWAP) };
    if result == 0 {
        Ok(())
    } else {
        Err(BioMcpError::Io(std::io::Error::last_os_error()))
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn exchange_directories(_left: &Path, _right: &Path) -> Result<(), BioMcpError> {
    Err(BioMcpError::InvalidArgument(
        "Atomic skill repair is unsupported on this platform".into(),
    ))
}
