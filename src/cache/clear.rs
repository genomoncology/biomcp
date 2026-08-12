use std::cmp::Reverse;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::BioMcpError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ClearReport {
    pub(crate) bytes_freed: Option<u64>,
    pub(crate) entries_removed: usize,
}

#[derive(Debug, Default)]
struct ClearPlan {
    regular_files: Vec<(PathBuf, u64)>,
    symlinks: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

pub(crate) fn execute_cache_clear(cache_path: &Path) -> Result<ClearReport, BioMcpError> {
    let metadata = match fs::symlink_metadata(cache_path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(ClearReport {
                bytes_freed: Some(0),
                entries_removed: 0,
            });
        }
        Err(err) => return Err(BioMcpError::Io(err)),
    };

    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        fs::remove_file(cache_path)?;
        return Ok(ClearReport {
            bytes_freed: None,
            entries_removed: 1,
        });
    }

    if !file_type.is_dir() {
        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear requires '{}' to be a directory or symlink, found {}",
            cache_path.display(),
            describe_file_type(&file_type)
        )));
    }

    super::secure_managed_tree(cache_path)
        .map_err(|error| BioMcpError::InvalidArgument(error.to_string()))?;
    let mut plan = ClearPlan::default();
    scan_directory(cache_path, &mut plan)?;
    plan.directories.push(cache_path.to_path_buf());

    let file_count = plan.regular_files.len();
    let symlink_count = plan.symlinks.len();
    let directory_count = plan.directories.len();
    let saw_symlink = symlink_count > 0;

    let mut bytes_freed = 0u64;
    for (path, size_bytes) in plan.regular_files {
        reject_linked_file(&path, &fs::symlink_metadata(&path)?)?;
        fs::remove_file(path)?;
        bytes_freed += size_bytes;
    }

    for path in plan.symlinks {
        fs::remove_file(path)?;
    }

    plan.directories
        .sort_by_key(|path| Reverse(path.components().count()));
    for path in plan.directories {
        fs::remove_dir(path)?;
    }

    Ok(ClearReport {
        bytes_freed: (!saw_symlink).then_some(bytes_freed),
        entries_removed: file_count + symlink_count + directory_count,
    })
}

#[cfg(unix)]
fn reject_linked_file(path: &Path, metadata: &fs::Metadata) -> Result<(), BioMcpError> {
    use std::os::unix::fs::MetadataExt;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear refuses linked or replaced file '{}'",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn reject_linked_file(_path: &Path, metadata: &fs::Metadata) -> Result<(), BioMcpError> {
    if metadata.is_file() {
        Ok(())
    } else {
        Err(BioMcpError::InvalidArgument(
            "cache file changed during clear".into(),
        ))
    }
}

fn scan_directory(path: &Path, plan: &mut ClearPlan) -> Result<(), BioMcpError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)?;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            plan.regular_files.push((entry_path, metadata.len()));
            continue;
        }

        if file_type.is_symlink() {
            plan.symlinks.push(entry_path);
            continue;
        }

        if file_type.is_dir() {
            scan_directory(&entry_path, plan)?;
            plan.directories.push(entry_path);
            continue;
        }

        return Err(BioMcpError::InvalidArgument(format!(
            "cache clear found unsupported entry at '{}': {}",
            entry_path.display(),
            describe_file_type(&file_type)
        )));
    }

    Ok(())
}

fn describe_file_type(file_type: &fs::FileType) -> &'static str {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;

        if file_type.is_socket() {
            return "socket";
        }
        if file_type.is_fifo() {
            return "fifo";
        }
        if file_type.is_block_device() {
            return "block device";
        }
        if file_type.is_char_device() {
            return "char device";
        }
    }

    if file_type.is_dir() {
        "directory"
    } else if file_type.is_file() {
        "file"
    } else if file_type.is_symlink() {
        "symlink"
    } else {
        "unsupported file type"
    }
}

#[cfg(test)]
mod tests {
    use super::{ClearReport, execute_cache_clear};
    use crate::error::BioMcpError;
    use crate::test_support::TempDirGuard;
    use std::fs;

    #[test]
    fn clear_missing_path_returns_zero_report() {
        let root = TempDirGuard::new("missing");

        let report =
            execute_cache_clear(&root.path().join("http")).expect("missing path should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: Some(0),
                entries_removed: 0,
            }
        );
    }

    #[cfg(unix)]
    #[test]
    fn clear_root_symlink_is_unlinked_without_traversing_target() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("root-symlink");
        let target_dir = root.path().join("target");
        let target_file = target_dir.join("outside.txt");
        fs::create_dir_all(&target_dir).expect("create target dir");
        fs::write(&target_file, b"outside").expect("seed target file");
        symlink(&target_dir, root.path().join("http")).expect("create root symlink");

        let report = execute_cache_clear(&root.path().join("http")).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: None,
                entries_removed: 1,
            }
        );
        assert!(
            !root.path().join("http").exists(),
            "root symlink should be removed"
        );
        assert!(target_file.is_file(), "target file should remain untouched");
    }

    #[cfg(unix)]
    #[test]
    fn clear_nested_symlink_unlinks_entry_and_sets_bytes_to_none() {
        use std::os::unix::fs::symlink;

        let root = TempDirGuard::new("nested-symlink");
        let http_dir = root.path().join("http");
        let target_dir = root.path().join("target");
        let target_file = target_dir.join("outside.txt");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::create_dir_all(&target_dir).expect("create target dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");
        fs::write(&target_file, b"outside").expect("seed target file");
        symlink(&target_dir, http_dir.join("nested").join("link")).expect("create nested symlink");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: None,
                entries_removed: 4,
            }
        );
        assert!(!http_dir.exists(), "http dir should be removed");
        assert!(target_file.is_file(), "symlink target should remain");
    }

    #[test]
    fn clear_removes_directory_tree_and_root_http_dir() {
        let root = TempDirGuard::new("remove-tree");
        let http_dir = root.path().join("http");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(
            report,
            ClearReport {
                bytes_freed: Some(3),
                entries_removed: 3,
            }
        );
        assert!(!http_dir.exists(), "http dir should be removed");
    }

    #[test]
    fn clear_preserves_sibling_downloads_directory() {
        let root = TempDirGuard::new("preserve-downloads");
        let http_dir = root.path().join("http");
        let downloads_dir = root.path().join("downloads");
        fs::create_dir_all(http_dir.join("nested")).expect("create http dir");
        fs::create_dir_all(&downloads_dir).expect("create downloads dir");
        fs::write(http_dir.join("nested").join("entry.bin"), b"abc").expect("seed file");
        fs::write(downloads_dir.join("keep.bin"), b"keep").expect("seed downloads file");

        let report = execute_cache_clear(&http_dir).expect("clear should succeed");

        assert_eq!(report.bytes_freed, Some(3));
        assert!(!http_dir.exists(), "http dir should be removed");
        assert!(downloads_dir.is_dir(), "downloads dir should remain");
        assert_eq!(
            fs::read(downloads_dir.join("keep.bin")).expect("downloads file should remain"),
            b"keep"
        );
    }

    #[test]
    fn clear_rejects_root_regular_file_before_mutation() {
        let root = TempDirGuard::new("root-file");
        let http_path = root.path().join("http");
        fs::write(&http_path, b"not-a-dir").expect("seed file");

        let err = execute_cache_clear(&http_path).expect_err("root file should be rejected");

        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(http_path.is_file(), "root file should remain untouched");
    }

    #[cfg(unix)]
    #[test]
    fn clear_rejects_special_file_before_mutation() {
        use std::os::unix::net::UnixListener;

        let root = tempfile::Builder::new()
            .prefix("biomcp-sock-")
            .tempdir_in("/tmp")
            .expect("create short socket root");
        let http_dir = root.path().join("http");
        let socket_path = http_dir.join("special.sock");
        assert!(
            socket_path.as_os_str().len() < 100,
            "socket fixture path is too long"
        );
        let regular_file = http_dir.join("entry.bin");
        fs::create_dir_all(&http_dir).expect("create http dir");
        fs::write(&regular_file, b"abc").expect("seed file");
        let _listener = UnixListener::bind(&socket_path).expect("bind unix socket");

        let err = execute_cache_clear(&http_dir).expect_err("special file should be rejected");

        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(http_dir.is_dir(), "http dir should remain");
        assert!(regular_file.is_file(), "regular file should remain");
        assert!(socket_path.exists(), "socket should remain");
    }
}
