use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;

pub(crate) fn secure_managed_tree(root: &Path, recurse: bool) -> io::Result<()> {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed state root is not a directory: {}", root.display()),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => create_private_dir(root)?,
        Err(error) => return Err(error),
    }
    secure_entry(root, recurse)
}

pub(crate) fn open_private(options: &mut OpenOptions, path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

pub(crate) fn open_managed_read(path: &Path) -> io::Result<File> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed state entry is not a regular file: {}",
                path.display()
            ),
        ));
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path)?;
    let opened = file.metadata()?;
    if !opened.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed state entry is not a regular file: {}",
                path.display()
            ),
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened.nlink() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed file has {} links: {path:?}", opened.nlink()),
            ));
        }
    }
    #[cfg(windows)]
    {
        let links = windows_link_count(&file)?;
        if links != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed file has {links} links: {path:?}"),
            ));
        }
    }
    Ok(file)
}

#[cfg(windows)]
fn windows_link_count(file: &File) -> io::Result<u32> {
    use std::os::windows::io::AsRawHandle;

    windows_link_count_from_handle(file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE)
}

#[cfg(windows)]
fn windows_link_count_from_handle(
    handle: windows_sys::Win32::Foundation::HANDLE,
) -> io::Result<u32> {
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };

    let mut information = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    // SAFETY: `information` points to writable storage for the duration of the
    // call, and callers supply an opened handle. Windows reports invalid or
    // unreadable handles by returning zero, which we convert to an error.
    let succeeded = unsafe { GetFileInformationByHandle(handle, information.as_mut_ptr()) };
    if succeeded == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful call initialized the complete structure.
    Ok(unsafe { information.assume_init() }.nNumberOfLinks)
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true).mode(0o700).create(path)
}

#[cfg(windows)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

#[cfg(unix)]
fn secure_entry(path: &Path, recurse: bool) -> io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if metadata.nlink() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed file has {} links: {path:?}", metadata.nlink()),
            ));
        }
        if metadata.permissions().mode() & 0o777 != 0o600 {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::other("unsupported managed file type"));
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    if recurse {
        for entry in fs::read_dir(path)? {
            secure_entry(&entry?.path(), true)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn secure_entry(path: &Path, recurse: bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        drop(open_managed_read(path)?);
    } else if metadata.is_dir() {
        if recurse {
            for entry in fs::read_dir(path)? {
                secure_entry(&entry?.path(), true)?;
            }
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported managed type: {}", path.display()),
        ));
    }
    let user = std::env::var("USERNAME").map_err(|_| io::Error::other("no USERNAME"))?;
    let grant = if metadata.is_dir() {
        format!("{user}:(OI)(CI)F")
    } else {
        format!("{user}:F")
    };
    let status = std::process::Command::new("icacls.exe")
        .arg(path)
        .args(["/inheritance:r", "/grant:r"])
        .arg(&grant)
        .status()?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| io::Error::other(format!("cannot secure: {path:?}")))
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn opened_managed_file_rejects_a_hard_link() {
        let root = tempfile::tempdir().expect("temporary root");
        let file = root.path().join("managed");
        fs::write(&file, b"fixture-only").expect("managed file");
        fs::hard_link(&file, root.path().join("other-name")).expect("hard link");

        assert_eq!(
            open_managed_read(&file)
                .expect_err("hard link must be rejected")
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }

    #[test]
    fn unreadable_handle_metadata_fails_closed() {
        assert!(windows_link_count_from_handle(std::ptr::null_mut()).is_err());
    }
}
