//! Ownership receipt and durable standalone-install transaction primitives.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::BioMcpError;

pub(crate) const RECEIPT_NAME: &str = "biomcp.install.json";
pub(crate) const RECEIPT_SCHEMA_VERSION: u8 = 1;
pub(crate) const INSTALLER_IDENTITY: &str = "biomcp-standalone-installer";
static RECEIPT_NONCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReceiptState {
    Installed,
    Pending,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct InstallReceipt {
    pub schema_version: u8,
    pub installer: String,
    pub state: ReceiptState,
    pub executable_path: PathBuf,
    pub version: String,
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_sha256: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct OwnedInstallation {
    pub executable: PathBuf,
    pub receipt_path: PathBuf,
    pub receipt: InstallReceipt,
}

pub(crate) fn receipt_path(executable: &Path) -> Result<PathBuf, BioMcpError> {
    executable
        .parent()
        .map(|parent| parent.join(RECEIPT_NAME))
        .ok_or_else(|| BioMcpError::InvalidArgument("Executable has no parent directory".into()))
}

pub(crate) fn sha256_file(path: &Path) -> Result<String, BioMcpError> {
    let mut file = open_regular_nofollow(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn open_regular_nofollow(path: &Path) -> Result<File, BioMcpError> {
    let metadata = std::fs::symlink_metadata(path).map_err(BioMcpError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(BioMcpError::PackageManagedInstall {
            guidance: format!(
                "Refusing to mutate non-regular installation path {}. Use its package manager or reinstall with the canonical standalone installer.",
                path.display()
            ),
        });
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    options.open(path).map_err(BioMcpError::Io)
}

fn package_guidance(executable: &Path) -> String {
    let path = executable.to_string_lossy().replace('\\', "/");
    if path.contains("/Cellar/") || path.contains("/homebrew/") {
        "This BioMCP installation is managed by Homebrew. Run `brew upgrade biomcp` or `brew uninstall biomcp`.".into()
    } else if path.contains("/pipx/venvs/") {
        "This BioMCP installation is managed by pipx. Run `pipx upgrade biomcp-cli` or `pipx uninstall biomcp-cli`.".into()
    } else if path.contains("/uv/tools/") || path.contains("/uv/tool/") {
        "This BioMCP installation is managed by uv. Run `uv tool upgrade biomcp-cli` or `uv tool uninstall biomcp-cli`.".into()
    } else if path.contains("site-packages") || path.contains("/.venv/") || path.contains("/venv/")
    {
        "This BioMCP installation is managed by Python packaging. Run the matching `uv pip`, `pip`, or environment-manager command for biomcp-cli.".into()
    } else {
        "BioMCP cannot prove this binary is owned by the standalone installer. Use its package manager or reinstall with the canonical installer.".into()
    }
}

fn read_receipt(path: &Path) -> Result<InstallReceipt, BioMcpError> {
    let file = open_regular_nofollow(path)?;
    let mut bytes = Vec::new();
    file.take(64 * 1024).read_to_end(&mut bytes)?;
    serde_json::from_slice(&bytes).map_err(|_| BioMcpError::PackageManagedInstall {
        guidance: format!("The standalone installer receipt at {} is malformed. Reinstall with the canonical installer before updating or uninstalling.", path.display()),
    })
}

fn validate_receipt_identity(
    receipt: &InstallReceipt,
    executable: &Path,
) -> Result<(), BioMcpError> {
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION
        || receipt.installer != INSTALLER_IDENTITY
        || receipt.executable_path != executable
    {
        return Err(BioMcpError::PackageManagedInstall {
            guidance: package_guidance(executable),
        });
    }
    Ok(())
}

pub(crate) fn validate_owned(executable: &Path) -> Result<OwnedInstallation, BioMcpError> {
    let canonical = std::fs::canonicalize(executable).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            BioMcpError::NotInstalled {
                path: executable.display().to_string(),
            }
        } else {
            BioMcpError::Io(error)
        }
    })?;
    if canonical != executable {
        return Err(BioMcpError::PackageManagedInstall {
            guidance: package_guidance(executable),
        });
    }
    let _ = open_regular_nofollow(executable)?;
    let receipt_path = receipt_path(executable)?;
    let mut receipt = match read_receipt(&receipt_path) {
        Ok(receipt) => receipt,
        Err(BioMcpError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(BioMcpError::PackageManagedInstall {
                guidance: package_guidance(executable),
            });
        }
        Err(error) => return Err(error),
    };
    validate_receipt_identity(&receipt, executable)?;
    let actual = sha256_file(executable)?;
    if receipt.state == ReceiptState::Pending {
        if receipt.new_sha256.as_deref() == Some(&actual) {
            receipt.state = ReceiptState::Installed;
            receipt.version = receipt.new_version.clone().unwrap_or(receipt.version);
            receipt.sha256 = actual.clone();
            clear_transaction(&mut receipt);
            write_receipt_atomic(&receipt_path, &receipt)?;
        } else if receipt.old_sha256.as_deref() == Some(&actual) {
            receipt.state = ReceiptState::Installed;
            receipt.version = receipt.old_version.clone().unwrap_or(receipt.version);
            receipt.sha256 = actual.clone();
            clear_transaction(&mut receipt);
            write_receipt_atomic(&receipt_path, &receipt)?;
        } else {
            return Err(BioMcpError::PackageManagedInstall {
                guidance: "The pending installer transaction does not match the installed executable. Reinstall with the canonical installer; no mutation was attempted.".into(),
            });
        }
    }
    if receipt.sha256 != actual {
        return Err(BioMcpError::PackageManagedInstall {
            guidance: "The installer receipt checksum does not match this executable. Use its package manager or reinstall with the canonical installer.".into(),
        });
    }
    Ok(OwnedInstallation {
        executable: executable.to_path_buf(),
        receipt_path,
        receipt,
    })
}

fn clear_transaction(receipt: &mut InstallReceipt) {
    receipt.transaction_nonce = None;
    receipt.old_version = None;
    receipt.old_sha256 = None;
    receipt.new_version = None;
    receipt.new_sha256 = None;
}

pub(crate) fn pending_receipt(
    owned: &OwnedInstallation,
    new_version: &str,
    new_sha256: &str,
    nonce: &str,
) -> InstallReceipt {
    InstallReceipt {
        state: ReceiptState::Pending,
        transaction_nonce: Some(nonce.into()),
        old_version: Some(owned.receipt.version.clone()),
        old_sha256: Some(owned.receipt.sha256.clone()),
        new_version: Some(new_version.into()),
        new_sha256: Some(new_sha256.into()),
        ..owned.receipt.clone()
    }
}

pub(crate) fn installed_receipt(pending: &InstallReceipt) -> InstallReceipt {
    let mut receipt = pending.clone();
    receipt.state = ReceiptState::Installed;
    receipt.version = receipt.new_version.clone().unwrap_or(receipt.version);
    receipt.sha256 = receipt.new_sha256.clone().unwrap_or(receipt.sha256);
    clear_transaction(&mut receipt);
    receipt
}

pub(crate) fn write_receipt_atomic(
    path: &Path,
    receipt: &InstallReceipt,
) -> Result<(), BioMcpError> {
    let parent = path
        .parent()
        .ok_or_else(|| BioMcpError::InvalidArgument("Receipt has no parent directory".into()))?;
    let bytes = serde_json::to_vec_pretty(receipt)?;
    let mut opened = None;
    for _ in 0..32 {
        let suffix = RECEIPT_NONCE.fetch_add(1, Ordering::Relaxed);
        let temp = parent.join(format!(
            ".{RECEIPT_NAME}.{}-{suffix}.tmp",
            std::process::id()
        ));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        match options.open(&temp) {
            Ok(file) => {
                opened = Some((temp, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    let (temp, mut file) = opened.ok_or_else(|| {
        BioMcpError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not create a unique receipt staging file",
        ))
    })?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        std::fs::rename(&temp, path)?;
        File::open(parent)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(BioMcpError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDirGuard;

    fn seed_owned(root: &Path) -> PathBuf {
        let executable = root.join("biomcp");
        std::fs::write(&executable, b"owned-binary").unwrap();
        let canonical = std::fs::canonicalize(&executable).unwrap();
        let receipt = InstallReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            installer: INSTALLER_IDENTITY.into(),
            state: ReceiptState::Installed,
            executable_path: canonical.clone(),
            version: "1.0.0".into(),
            sha256: sha256_file(&canonical).unwrap(),
            transaction_nonce: None,
            old_version: None,
            old_sha256: None,
            new_version: None,
            new_sha256: None,
        };
        write_receipt_atomic(&receipt_path(&canonical).unwrap(), &receipt).unwrap();
        canonical
    }

    #[test]
    fn valid_receipt_proves_ownership_and_pending_new_state_recovers() {
        let root = TempDirGuard::new("install-owned");
        let executable = seed_owned(root.path());
        let owned = validate_owned(&executable).unwrap();
        let new_hash = sha256_hex(b"new-binary");
        let pending = pending_receipt(&owned, "2.0.0", &new_hash, "nonce");
        write_receipt_atomic(&owned.receipt_path, &pending).unwrap();
        std::fs::write(&executable, b"new-binary").unwrap();
        let recovered = validate_owned(&executable).unwrap();
        assert_eq!(recovered.receipt.state, ReceiptState::Installed);
        assert_eq!(recovered.receipt.version, "2.0.0");
    }

    #[test]
    fn missing_malformed_mismatched_and_symlink_receipts_fail_closed() {
        let root = TempDirGuard::new("install-refuse");
        let executable = root.path().join("biomcp");
        std::fs::write(&executable, b"binary").unwrap();
        let executable = std::fs::canonicalize(executable).unwrap();
        assert!(matches!(
            validate_owned(&executable),
            Err(BioMcpError::PackageManagedInstall { .. })
        ));
        std::fs::write(receipt_path(&executable).unwrap(), b"{").unwrap();
        assert!(matches!(
            validate_owned(&executable),
            Err(BioMcpError::PackageManagedInstall { .. })
        ));
    }

    #[test]
    fn checksum_mismatch_and_package_manager_paths_fail_with_safe_guidance() {
        let root = TempDirGuard::new("install-mismatch");
        let executable = seed_owned(root.path());
        std::fs::write(&executable, b"tampered").unwrap();
        assert!(matches!(
            validate_owned(&executable),
            Err(BioMcpError::PackageManagedInstall { .. })
        ));
        assert!(
            package_guidance(Path::new("/opt/homebrew/Cellar/biomcp/1/bin/biomcp"))
                .contains("brew upgrade biomcp")
        );
        assert!(
            package_guidance(Path::new("/tmp/pipx/venvs/biomcp-cli/bin/biomcp"))
                .contains("pipx upgrade biomcp-cli")
        );
        assert!(
            package_guidance(Path::new("/tmp/uv/tools/biomcp-cli/bin/biomcp"))
                .contains("uv tool upgrade biomcp-cli")
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_executable_or_receipt_never_grants_ownership() {
        let root = TempDirGuard::new("install-symlink");
        let executable = seed_owned(root.path());
        let receipt = receipt_path(&executable).unwrap();
        let receipt_target = root.path().join("receipt-target");
        std::fs::rename(&receipt, &receipt_target).unwrap();
        std::os::unix::fs::symlink(&receipt_target, &receipt).unwrap();
        assert!(matches!(
            validate_owned(&executable),
            Err(BioMcpError::PackageManagedInstall { .. })
        ));

        std::fs::remove_file(&receipt).unwrap();
        std::fs::rename(&receipt_target, &receipt).unwrap();
        let link = root.path().join("biomcp-link");
        std::os::unix::fs::symlink(&executable, &link).unwrap();
        assert!(matches!(
            validate_owned(&link),
            Err(BioMcpError::PackageManagedInstall { .. })
        ));
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
