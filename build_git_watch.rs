use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GitRefWatchPaths {
    pub(crate) head: PathBuf,
    pub(crate) current_ref: Option<PathBuf>,
    pub(crate) packed_refs: PathBuf,
}

pub(crate) fn git_ref_watch_paths(
    git_dir: impl AsRef<Path>,
    git_common_dir: impl AsRef<Path>,
    head_contents: &str,
) -> GitRefWatchPaths {
    let git_dir = git_dir.as_ref();
    let git_common_dir = git_common_dir.as_ref();
    let current_ref = head_contents
        .trim()
        .strip_prefix("ref:")
        .map(str::trim)
        .filter(|git_ref| !git_ref.is_empty())
        .map(|git_ref| git_common_dir.join(git_ref));

    GitRefWatchPaths {
        head: git_dir.join("HEAD"),
        current_ref,
        packed_refs: git_common_dir.join("packed-refs"),
    }
}
