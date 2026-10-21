//! Bind a retained connection to the directory and database it actually opened.
use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
struct FileIdentity(u64, u64);

pub(super) struct Identity {
    root: FileIdentity,
    database: FileIdentity,
}

impl Identity {
    pub(super) fn capture(root: &Path) -> Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || crate::cache::private::is_link_or_reparse_point(&metadata) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "cache directory changed while open",
            ));
        }
        validate_database_file(&root.join(DATABASE_NAME), false)?;
        Ok(Self {
            root: file_identity(root)?,
            database: file_identity(&root.join(DATABASE_NAME))?,
        })
    }

    pub(super) fn validate(&self, root: &Path, connection: &Connection) -> Result<()> {
        let current = Self::capture(root)?;
        if self.root != current.root || self.database != current.database {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "cache path changed while open",
            ));
        }
        validate_opened_file(connection)
    }
}

#[cfg(unix)]
fn file_identity(path: &Path) -> Result<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::symlink_metadata(path)?;
    Ok(FileIdentity(metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn validate_opened_file(connection: &Connection) -> Result<()> {
    let mut moved = 0_i32;
    // SAFETY: SQLite owns this live connection. The main database name is a
    // static C string and HAS_MOVED writes exactly one integer. Its Unix VFS
    // compares the live database's inode without opening another descriptor.
    let status = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
            (&mut moved as *mut i32).cast(),
        )
    };
    if status != rusqlite::ffi::SQLITE_OK || moved != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "opened cache database changed",
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn file_identity(path: &Path) -> Result<FileIdentity> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    // Windows does not release another handle's SQLite locks on close. Backup
    // semantics permit a directory handle; OPEN_REPARSE_POINT avoids following
    // a substituted link. These handles never read database content.
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let mut information = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    // SAFETY: the owned file handle stays alive and the output points to writable
    // storage for the complete Windows file-information structure.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, information.as_mut_ptr()) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful GetFileInformationByHandle initialized every field.
    let information = unsafe { information.assume_init() };
    Ok(FileIdentity(
        u64::from(information.dwVolumeSerialNumber),
        (u64::from(information.nFileIndexHigh) << 32) | u64::from(information.nFileIndexLow),
    ))
}

#[cfg(windows)]
fn validate_opened_file(_connection: &Connection) -> Result<()> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn file_identity(_path: &Path) -> Result<FileIdentity> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "cache file identity is unavailable",
    ))
}

#[cfg(not(any(unix, windows)))]
fn validate_opened_file(_connection: &Connection) -> Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "cache file identity is unavailable",
    ))
}
