use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub(super) const INLINE_LIMIT: usize = 1024 * 1024;

pub(super) fn publish(root: &Path, bytes: &[u8]) -> io::Result<String> {
    let digest = format!("{:x}", Sha256::digest(bytes));
    let directory = root.join("blobs");
    super::super::private::secure_managed_dir(&directory)?;
    let path = directory.join(&digest);
    if let Ok(mut file) = super::super::open_managed_read(&path) {
        let mut existing = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut existing)?;
        if existing == bytes {
            return Ok(digest);
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cache blob integrity",
        ));
    }
    let temporary = directory.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = super::super::open_private(&mut options, &temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result.map(|()| digest)
}

pub(super) fn read(root: &Path, digest: &str) -> io::Result<Vec<u8>> {
    let path = blob_path(root, digest)?;
    let mut bytes = Vec::new();
    let mut file = super::super::open_managed_read(&path)?;
    std::io::Read::read_to_end(&mut file, &mut bytes)?;
    if format!("{:x}", Sha256::digest(&bytes)) != digest {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "cache blob integrity",
        ));
    }
    Ok(bytes)
}

pub(super) fn remove(root: &Path, digest: &str) -> io::Result<()> {
    let result = blob_path(root, digest).and_then(fs::remove_file);
    match result {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

fn blob_path(root: &Path, digest: &str) -> io::Result<PathBuf> {
    validate_digest(digest)?;
    let directory = root.join("blobs");
    let metadata = fs::symlink_metadata(&directory)?;
    if !metadata.is_dir() || super::super::private::is_link_or_reparse_point(&metadata) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid blob directory",
        ));
    }
    Ok(directory.join(digest))
}

fn validate_digest(digest: &str) -> io::Result<()> {
    if digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid cache digest",
        ))
    }
}
