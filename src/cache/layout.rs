use std::path::{Path, PathBuf};

use sha1::{Digest, Sha1};
use ssri::Integrity;

pub(crate) const CONTENT_DIR: &str = "content-v2";
pub(crate) const INDEX_DIR: &str = "index-v5";
pub(crate) const TEMP_DIR: &str = "tmp";

pub(crate) fn index_bucket_path(cache_path: &Path, key: &str) -> PathBuf {
    let digest = Sha1::digest(key.as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    cache_path
        .join(INDEX_DIR)
        .join(&hex[..2])
        .join(&hex[2..4])
        .join(&hex[4..])
}

pub(crate) fn content_root(cache_path: &Path) -> PathBuf {
    cache_path.join(CONTENT_DIR)
}

pub(crate) fn content_path(cache_path: &Path, integrity: &Integrity) -> PathBuf {
    let (algorithm, hex) = integrity.to_hex();
    content_root(cache_path)
        .join(algorithm.to_string())
        .join(&hex[..2])
        .join(&hex[2..4])
        .join(&hex[4..])
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::index_bucket_path;
    use crate::test_support::TempDirGuard;

    #[test]
    fn derived_index_bucket_is_the_file_cacache_writes() {
        let root = TempDirGuard::new("cache-index-layout");
        for key in [
            "plain-ascii-key",
            "https://example.test/search?q=BRAF%20V600E&lang=é",
        ] {
            cacache::write_sync(root.path(), key, b"value").expect("write cache entry");
            let bucket = index_bucket_path(root.path(), key);
            assert!(
                bucket.is_file(),
                "derived bucket missing: {}",
                bucket.display()
            );
            assert!(
                fs::read_to_string(bucket)
                    .expect("read index bucket")
                    .contains(key),
                "derived bucket does not contain key"
            );
        }
    }
}
