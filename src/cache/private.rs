use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;

pub(crate) fn secure_managed_tree(root: &Path) -> io::Result<()> {
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
    secure_entry(root)
}

pub(crate) fn open_private(options: &mut OpenOptions, path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
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
fn secure_entry(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if metadata.nlink() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "managed file has {} links: {}",
                    metadata.nlink(),
                    path.display()
                ),
            ));
        }
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(io::Error::other("unsupported managed file type"));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    for entry in fs::read_dir(path)? {
        secure_entry(&entry?.path())?;
    }
    Ok(())
}

#[cfg(windows)]
fn secure_entry(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        let links = std::process::Command::new("fsutil.exe")
            .args(["hardlink", "list"])
            .arg(path)
            .output()?;
        if !links.status.success() || String::from_utf8_lossy(&links.stdout).lines().count() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("cannot verify one-link managed file: {}", path.display()),
            ));
        }
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            secure_entry(&entry?.path())?;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported managed file type: {}", path.display()),
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
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "cannot secure: {}",
            path.display()
        )))
    }
}
