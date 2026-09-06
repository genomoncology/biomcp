use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const CACHE_OPERATION_LOCK: &str = ".biomcp-operation.lock";

pub(crate) struct CacheOperationGuard {
    _file: File,
}

pub(crate) struct CacheKeyGuard {
    _shared: CacheOperationGuard,
    _key: CacheOperationGuard,
}

fn operation_lock_file(cache_root: &Path) -> io::Result<File> {
    secure_managed_dir(cache_root)?;
    let lock_path = cache_root.join(CACHE_OPERATION_LOCK);
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    open_private(&mut options, &lock_path)
}

pub(crate) fn lock_cache_maintenance(cache_root: &Path) -> io::Result<CacheOperationGuard> {
    use fs2::FileExt;

    let file = operation_lock_file(cache_root)?;
    if let Some(deadline) = crate::sources::current_variant_article_deadline() {
        loop {
            match file.try_lock_exclusive() {
                Ok(()) => return Ok(CacheOperationGuard { _file: file }),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    check_variant_article_deadline()?;
                    std::thread::sleep(
                        deadline
                            .remaining()
                            .min(std::time::Duration::from_millis(10)),
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }
    file.lock_exclusive()?;
    Ok(CacheOperationGuard { _file: file })
}

pub(crate) fn try_lock_cache_maintenance(
    cache_root: &Path,
) -> io::Result<Option<CacheOperationGuard>> {
    use fs2::FileExt;

    let file = operation_lock_file(cache_root)?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(CacheOperationGuard { _file: file })),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(error) => Err(error),
    }
}

pub(crate) fn lock_cache_shared(cache_root: &Path) -> io::Result<CacheOperationGuard> {
    use fs2::FileExt;

    let file = operation_lock_file(cache_root)?;
    if let Some(deadline) = crate::sources::current_variant_article_deadline() {
        loop {
            match FileExt::try_lock_shared(&file) {
                Ok(()) => return Ok(CacheOperationGuard { _file: file }),
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    check_variant_article_deadline()?;
                    std::thread::sleep(
                        deadline
                            .remaining()
                            .min(std::time::Duration::from_millis(10)),
                    );
                }
                Err(error) => return Err(error),
            }
        }
    }
    FileExt::lock_shared(&file)?;
    Ok(CacheOperationGuard { _file: file })
}

fn lock_cache_key(
    cache_root: &Path,
    key: &str,
    before_lock_dir_create: &dyn Fn(&Path),
) -> io::Result<CacheKeyGuard> {
    use fs2::FileExt;

    let shared = lock_cache_shared(cache_root)?;
    let lock_dir = cache_root.join(super::KEY_LOCK_DIR);
    secure_managed_dir_with(&lock_dir, || before_lock_dir_create(&lock_dir))?;
    let lock_path = super::key_lock_path(cache_root, key);
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    let file = open_private(&mut options, &lock_path)?;
    file.lock_exclusive()?;
    Ok(CacheKeyGuard {
        _shared: shared,
        _key: CacheOperationGuard { _file: file },
    })
}

pub(crate) async fn lock_cache_key_async(
    cache_root: PathBuf,
    key: String,
    before_lock_dir_create: Arc<dyn Fn(&Path) + Send + Sync>,
) -> io::Result<CacheKeyGuard> {
    if let Some(deadline) = crate::sources::current_variant_article_deadline() {
        let shared_file = operation_lock_file(&cache_root)?;
        let shared = lock_file_until(shared_file, false, &deadline).await?;
        let lock_dir = cache_root.join(super::KEY_LOCK_DIR);
        secure_managed_dir_with(&lock_dir, || before_lock_dir_create(&lock_dir))?;
        let lock_path = super::key_lock_path(&cache_root, &key);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        let key_file = open_private(&mut options, &lock_path)?;
        let key_guard = lock_file_until(key_file, true, &deadline).await?;
        return Ok(CacheKeyGuard {
            _shared: shared,
            _key: key_guard,
        });
    }
    tokio::task::spawn_blocking(move || {
        lock_cache_key(&cache_root, &key, before_lock_dir_create.as_ref())
    })
    .await
    .map_err(|error| io::Error::other(format!("cache key lock task failed: {error}")))?
}

pub(crate) async fn lock_cache_shared_until(
    cache_root: &Path,
    deadline: &crate::sources::VariantArticleDeadline,
) -> io::Result<CacheOperationGuard> {
    let file = operation_lock_file(cache_root)?;
    lock_file_until(file, false, deadline).await
}

async fn lock_file_until(
    file: File,
    exclusive: bool,
    deadline: &crate::sources::VariantArticleDeadline,
) -> io::Result<CacheOperationGuard> {
    use fs2::FileExt;

    loop {
        let result = if exclusive {
            file.try_lock_exclusive()
        } else {
            FileExt::try_lock_shared(&file)
        };
        match result {
            Ok(()) => return Ok(CacheOperationGuard { _file: file }),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                deadline
                    .run(tokio::time::sleep(std::time::Duration::from_millis(10)))
                    .await
                    .map_err(|_| {
                        io::Error::new(
                            io::ErrorKind::TimedOut,
                            "variant article invocation deadline exceeded",
                        )
                    })?;
            }
            Err(error) => return Err(error),
        }
    }
}

pub(crate) fn prepare_write_paths(cache_path: &Path, cache_key: &str) -> io::Result<()> {
    secure_managed_dir(cache_path)?;
    secure_managed_dir(&cache_path.join(super::TEMP_DIR))?;

    let content_root = super::content_root(cache_path);
    secure_managed_dir(&content_root)?;
    secure_managed_dir(&content_root.join("sha256"))?;

    let bucket = super::index_bucket_path(cache_path, cache_key);
    let index_root = cache_path.join(super::INDEX_DIR);
    secure_managed_dir(&index_root)?;
    let first_shard = bucket
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("derived cache index path has no first shard"))?;
    let second_shard = bucket
        .parent()
        .ok_or_else(|| io::Error::other("derived cache index path has no second shard"))?;
    secure_managed_dir(first_shard)?;
    secure_managed_dir(second_shard)?;
    prepare_managed_file(&bucket)
}

pub(crate) fn secure_written_content(
    cache_path: &Path,
    integrity: &ssri::Integrity,
) -> io::Result<()> {
    let blob = super::content_path(cache_path, integrity);
    let content_root = super::content_root(cache_path);
    secure_managed_dir(&content_root)?;

    let (algorithm, _) = integrity.to_hex();
    let algorithm = content_root.join(algorithm.to_string());
    let first_shard = blob
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("derived cache content path has no first shard"))?;
    let second_shard = blob
        .parent()
        .ok_or_else(|| io::Error::other("derived cache content path has no second shard"))?;
    secure_managed_dir(&algorithm)?;
    secure_managed_dir(first_shard)?;
    secure_managed_dir(second_shard)?;
    secure_managed_file(&blob)
}

pub(crate) fn secure_managed_tree(
    root: &Path,
    recurse: bool,
    managed_content_root: Option<&Path>,
) -> io::Result<()> {
    check_variant_article_deadline()?;
    match fs::symlink_metadata(root) {
        Ok(metadata) if is_link_or_reparse_point(&metadata) || !metadata.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("managed state root is not a directory: {}", root.display()),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => create_private_dir(root)?,
        Err(error) => return Err(error),
    }
    secure_entry(root, recurse, managed_content_root)
}

pub(crate) fn secure_managed_tree_until(
    root: &Path,
    recurse: bool,
    managed_content_root: Option<&Path>,
    deadline: &crate::sources::VariantArticleDeadline,
) -> io::Result<()> {
    if deadline.is_exhausted() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "variant article invocation deadline exceeded",
        ));
    }
    secure_managed_tree(root, recurse, managed_content_root)?;
    if deadline.is_exhausted() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "variant article invocation deadline exceeded",
        ));
    }
    Ok(())
}

pub(crate) fn secure_managed_dir(path: &Path) -> io::Result<()> {
    secure_managed_dir_with(path, || {})
}

fn secure_managed_dir_with<F>(path: &Path, before_create: F) -> io::Result<()>
where
    F: FnOnce(),
{
    match fs::symlink_metadata(path) {
        Ok(metadata) if is_link_or_reparse_point(&metadata) || !metadata.is_dir() => {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "managed state directory is not a directory: {}",
                    path.display()
                ),
            ))
        }
        Ok(_) => secure_entry(path, false, None),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            before_create();
            match create_private_dir_exact(path) {
                Ok(()) => secure_entry(path, false, None),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    secure_managed_dir(path)
                }
                Err(error) => Err(error),
            }
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn prepare_managed_file(path: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true).append(true).create(true);
    drop(open_private(&mut options, path)?);
    Ok(())
}

pub(crate) fn secure_managed_file(path: &Path) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    drop(open_private(&mut options, path)?);
    Ok(())
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

#[cfg(unix)]
fn create_private_dir_exact(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path)
}

#[cfg(windows)]
fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

#[cfg(windows)]
fn create_private_dir_exact(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

#[cfg(unix)]
fn secure_entry(path: &Path, recurse: bool, content_root: Option<&Path>) -> io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    check_variant_article_deadline()?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        if content_root.is_some_and(|root| path.starts_with(root))
            && symlink_targets_directory(path)?
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "managed content directory must not be a symlink: {}",
                    path.display()
                ),
            ));
        }
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
            check_variant_article_deadline()?;
            let entry_path = entry?.path();
            secure_entry(&entry_path, true, content_root)?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn secure_entry(path: &Path, recurse: bool, content_root: Option<&Path>) -> io::Result<()> {
    check_variant_article_deadline()?;
    let metadata = fs::symlink_metadata(path)?;
    if is_link_or_reparse_point(&metadata) {
        if content_root.is_some_and(|root| path.starts_with(root))
            && symlink_targets_directory(path)?
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "managed content directory must not be a reparse point: {}",
                    path.display()
                ),
            ));
        }
        return Ok(());
    }
    if metadata.is_file() {
        drop(open_managed_read(path)?);
    } else if metadata.is_dir() {
        if recurse {
            for entry in fs::read_dir(path)? {
                check_variant_article_deadline()?;
                let entry_path = entry?.path();
                secure_entry(&entry_path, true, content_root)?;
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

fn check_variant_article_deadline() -> io::Result<()> {
    if crate::sources::current_variant_article_deadline()
        .is_some_and(|deadline| deadline.is_exhausted())
    {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "variant article invocation deadline exceeded",
        ))
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;

    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

fn symlink_targets_directory(path: &Path) -> io::Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(all(test, unix))]
mod unix_tests {
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn constructor_repairs_and_independent_key_operations_do_not_serialize_globally() {
        let root = tempfile::tempdir().expect("temporary root");
        let constructor_a = lock_cache_shared(root.path()).expect("first shared constructor lock");
        let (constructor_tx, constructor_rx) = mpsc::channel();
        let root_path = root.path().to_path_buf();
        let constructor_b = std::thread::spawn(move || {
            let guard = lock_cache_shared(&root_path).expect("second shared constructor lock");
            constructor_tx.send(()).expect("report shared acquisition");
            guard
        });
        constructor_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("constructor repairs must overlap while first lock remains held");
        drop(constructor_a);
        drop(constructor_b.join().expect("constructor thread"));

        let key_a = "independent-key-a";
        let key_b = (0..10_000)
            .map(|candidate| format!("independent-key-{candidate}"))
            .find(|candidate| {
                super::super::key_lock_path(root.path(), candidate)
                    != super::super::key_lock_path(root.path(), key_a)
            })
            .expect("key in another lock shard");
        let operation_a = lock_cache_key(root.path(), key_a, &|_| {}).expect("first key lock");
        let (key_tx, key_rx) = mpsc::channel();
        let root_path = root.path().to_path_buf();
        let operation_b = std::thread::spawn(move || {
            let guard = lock_cache_key(&root_path, &key_b, &|_| {}).expect("independent key lock");
            key_tx.send(()).expect("report independent acquisition");
            guard
        });
        key_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("independent key operations must overlap");
        drop(operation_a);
        drop(operation_b.join().expect("key operation thread"));
    }

    #[test]
    fn directory_create_race_revalidates_a_hostile_winner() {
        let root = tempfile::tempdir().expect("temporary root");
        let outside = root.path().join("outside");
        let managed = root.path().join("managed");
        fs::create_dir(&outside).expect("outside directory");

        let error = secure_managed_dir_with(&managed, || {
            symlink(&outside, &managed).expect("hostile race winner");
        })
        .expect_err("race winner must be revalidated");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(
            fs::symlink_metadata(&managed)
                .expect("hostile link remains")
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn explicit_whole_tree_maintenance_repairs_an_unrelated_sentinel() {
        let root = tempfile::tempdir().expect("temporary root");
        let sentinel = root.path().join("unrelated/nested/sentinel");
        fs::create_dir_all(sentinel.parent().expect("sentinel parent")).expect("sentinel tree");
        fs::write(&sentinel, b"cached response").expect("sentinel");
        fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o644))
            .expect("permissive sentinel");

        secure_managed_tree(root.path(), true, None).expect("whole-tree maintenance");

        assert_eq!(
            fs::metadata(&sentinel)
                .expect("sentinel metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[test]
    fn whole_tree_maintenance_skips_unrelated_links_but_rejects_content_directory_links() {
        let root = tempfile::tempdir().expect("temporary root");
        let outside_file = root.path().join("outside-file");
        let outside_dir = root.path().join("outside-dir");
        fs::write(&outside_file, b"outside bytes").expect("outside file");
        fs::create_dir(&outside_dir).expect("outside directory");
        symlink(&outside_file, root.path().join("unrelated-link")).expect("unrelated link");
        fs::create_dir_all(root.path().join("http/content-v2/sha256")).expect("content root");
        symlink(
            &outside_dir,
            root.path().join("http/content-v2/sha256/directory-link"),
        )
        .expect("content directory link");

        let content_root = super::super::content_root(&root.path().join("http"));
        let error = secure_managed_tree(root.path(), true, Some(&content_root))
            .expect_err("content directory link must be rejected");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(
            fs::read(&outside_file).expect("outside unchanged"),
            b"outside bytes"
        );
        assert_eq!(fs::read_dir(&outside_dir).expect("outside dir").count(), 0);
    }

    #[test]
    fn unrelated_nested_content_v2_directory_symlink_keeps_skip_behavior() {
        let root = tempfile::tempdir().expect("temporary root");
        let outside = root.path().join("outside");
        let unrelated = root.path().join("other/content-v2");
        fs::create_dir_all(unrelated.parent().expect("unrelated parent"))
            .expect("unrelated parent");
        fs::create_dir(&outside).expect("outside directory");
        fs::write(outside.join("sentinel"), b"outside bytes").expect("outside sentinel");
        symlink(&outside, &unrelated).expect("unrelated directory symlink");

        let content_root = super::super::content_root(&root.path().join("http"));
        secure_managed_tree(root.path(), true, Some(&content_root))
            .expect("unrelated link remains skippable");

        assert!(
            fs::symlink_metadata(&unrelated)
                .expect("unrelated link metadata")
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read(outside.join("sentinel")).expect("outside unchanged"),
            b"outside bytes"
        );
    }

    #[test]
    fn cache_root_basename_never_changes_managed_content_semantics() {
        for root_name in ["http", "content-v2"] {
            let parent = tempfile::tempdir().expect("temporary parent");
            let cache_root = parent.path().join(root_name);
            let outside = parent.path().join("outside");
            fs::create_dir(&cache_root).expect("cache root");
            fs::create_dir(&outside).expect("outside directory");
            fs::write(outside.join("sentinel"), b"outside bytes").expect("outside sentinel");

            let unrelated = if root_name == "http" {
                cache_root.join("content-v2")
            } else {
                cache_root.join("unrelated-link")
            };
            symlink(&outside, &unrelated).expect("unrelated directory symlink");
            let content_root = super::super::content_root(&cache_root.join("http"));

            secure_managed_tree(&cache_root, true, Some(&content_root))
                .expect("basename collision must not make unrelated link strict");
            assert!(
                fs::symlink_metadata(&unrelated)
                    .expect("unrelated link metadata")
                    .file_type()
                    .is_symlink()
            );

            fs::create_dir_all(content_root.join("sha256")).expect("managed content tree");
            symlink(&outside, content_root.join("sha256/managed-link"))
                .expect("managed directory symlink");
            let error = secure_managed_tree(&cache_root, true, Some(&content_root))
                .expect_err("actual managed content directory link must be rejected");
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert_eq!(
                fs::read(outside.join("sentinel")).expect("outside unchanged"),
                b"outside bytes"
            );
        }
    }
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
