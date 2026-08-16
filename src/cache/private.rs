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
        options
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
        use windows_sys::Win32::Storage::FileSystem::WRITE_DAC;

        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options
            .access_mode(GENERIC_READ | GENERIC_WRITE | WRITE_DAC)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = open_configured(options, path)?;
    validate_opened_file(&file, path)?;
    repair_private_file(&file)?;
    Ok(file)
}

pub(crate) fn open_managed_read(path: &Path) -> io::Result<File> {
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
    let file = open_configured(&options, path)?;
    validate_opened_file(&file, path)?;
    Ok(file)
}

fn open_configured(options: &OpenOptions, path: &Path) -> io::Result<File> {
    options.open(path).map_err(|error| {
        #[cfg(unix)]
        if error.raw_os_error() == Some(libc::ELOOP) {
            return io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "managed state entry must not be a symlink: {}",
                    path.display()
                ),
            );
        }
        error
    })
}

fn validate_opened_file(file: &File, path: &Path) -> io::Result<()> {
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
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

        if opened.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed state entry is a reparse point: {}", path.display()),
            ));
        }
        let links = windows_link_count(&file)?;
        if links != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed file has {links} links: {path:?}"),
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn repair_private_file(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = file.metadata()?;
    if metadata.permissions().mode() & 0o777 != 0o600 {
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(windows)]
fn repair_private_file(file: &File) -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_SUCCESS, HANDLE, LocalFree};
    use windows_sys::Win32::Security::Authorization::{
        EXPLICIT_ACCESS_W, SE_FILE_OBJECT, SET_ACCESS, SetEntriesInAclW, SetSecurityInfo,
        TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::{
        ACL, DACL_SECURITY_INFORMATION, GetTokenInformation, NO_INHERITANCE,
        PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    struct OwnedHandle(HANDLE);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // SAFETY: the handle was returned by OpenProcessToken and remains owned here.
            unsafe { CloseHandle(self.0) };
        }
    }
    struct OwnedAcl(*mut ACL);
    impl Drop for OwnedAcl {
        fn drop(&mut self) {
            // SAFETY: SetEntriesInAclW allocated this ACL with LocalAlloc.
            unsafe { LocalFree(self.0.cast()) };
        }
    }

    let mut token = std::ptr::null_mut();
    // SAFETY: token points to writable handle storage and the pseudo process handle is valid.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = OwnedHandle(token);

    let mut needed = 0_u32;
    // SAFETY: a zero-length probe is the documented way to obtain the buffer size.
    unsafe {
        GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut needed);
    }
    if needed == 0 {
        return Err(io::Error::last_os_error());
    }
    let word_size = std::mem::size_of::<usize>();
    let mut token_words = vec![0_usize; (needed as usize).div_ceil(word_size)];
    // SAFETY: the aligned allocation has at least `needed` writable bytes and lives
    // through all uses of the returned SID pointer.
    if unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            token_words.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful TokenUser retrieval initialized a TOKEN_USER at the buffer start.
    let user = unsafe { &*(token_words.as_ptr().cast::<TOKEN_USER>()) };
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: FILE_ALL_ACCESS,
        grfAccessMode: SET_ACCESS,
        grfInheritance: NO_INHERITANCE,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: user.User.Sid.cast(),
        },
    };
    let mut acl = std::ptr::null_mut();
    // SAFETY: access and acl are valid for the call; the SID outlives ACL construction.
    let status = unsafe { SetEntriesInAclW(1, &access, std::ptr::null(), &mut acl) };
    if status != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let acl = OwnedAcl(acl);
    // SAFETY: the file handle requests WRITE_DAC, and acl remains alive for the call.
    let status = unsafe {
        SetSecurityInfo(
            file.as_raw_handle() as HANDLE,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            acl.0,
            std::ptr::null(),
        )
    };
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status as i32))
    }
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
    fn private_open_rejects_a_reparse_point() {
        let root = tempfile::tempdir().expect("temporary root");
        let target = root.path().join("target");
        fs::write(&target, b"fixture-only").expect("target file");
        let link = root.path().join("managed-link");
        std::os::windows::fs::symlink_file(&target, &link).expect("file symlink");
        let mut options = OpenOptions::new();
        options.read(true).write(true);

        assert_eq!(
            open_private(&mut options, &link)
                .expect_err("reparse point must be rejected")
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert_eq!(
            fs::read(&target).expect("target unchanged"),
            b"fixture-only"
        );
    }

    #[test]
    fn unreadable_handle_metadata_fails_closed() {
        assert!(windows_link_count_from_handle(std::ptr::null_mut()).is_err());
    }
}
