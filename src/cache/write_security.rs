use std::io;
use std::path::Path;

use ssri::Integrity;

pub(crate) fn prepare_write_paths(cache_path: &Path, cache_key: &str) -> io::Result<()> {
    super::secure_managed_dir(cache_path)?;
    super::secure_managed_dir(&cache_path.join(super::layout::TEMP_DIR))?;

    let content_root = super::content_root(cache_path);
    super::secure_managed_dir(&content_root)?;
    super::secure_managed_dir(&content_root.join("sha256"))?;

    let bucket = super::index_bucket_path(cache_path, cache_key);
    let index_root = cache_path.join(super::layout::INDEX_DIR);
    super::secure_managed_dir(&index_root)?;
    let first_shard = bucket
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("derived cache index path has no first shard"))?;
    let second_shard = bucket
        .parent()
        .ok_or_else(|| io::Error::other("derived cache index path has no second shard"))?;
    super::secure_managed_dir(first_shard)?;
    super::secure_managed_dir(second_shard)?;
    super::prepare_managed_file(&bucket)
}

pub(crate) fn secure_written_content(cache_path: &Path, integrity: &Integrity) -> io::Result<()> {
    let blob = super::content_path(cache_path, integrity);
    let content_root = super::content_root(cache_path);
    super::secure_managed_dir(&content_root)?;

    let (algorithm, _) = integrity.to_hex();
    let algorithm = content_root.join(algorithm.to_string());
    let first_shard = blob
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("derived cache content path has no first shard"))?;
    let second_shard = blob
        .parent()
        .ok_or_else(|| io::Error::other("derived cache content path has no second shard"))?;
    super::secure_managed_dir(&algorithm)?;
    super::secure_managed_dir(first_shard)?;
    super::secure_managed_dir(second_shard)?;
    super::secure_managed_file(&blob)
}
