#[path = "../build_git_watch.rs"]
mod build_git_watch;

use std::path::PathBuf;

use build_git_watch::{GitRefWatchPaths, git_ref_watch_paths};

#[test]
fn branch_head_watches_worktree_head_common_branch_ref_and_packed_refs() {
    let paths = git_ref_watch_paths(
        PathBuf::from("/repo/.git/worktrees/feature"),
        PathBuf::from("/repo/.git"),
        "ref: refs/heads/feature\n",
    );

    assert_eq!(
        paths,
        GitRefWatchPaths {
            head: PathBuf::from("/repo/.git/worktrees/feature/HEAD"),
            current_ref: Some(PathBuf::from("/repo/.git/refs/heads/feature")),
            packed_refs: PathBuf::from("/repo/.git/packed-refs"),
        }
    );
}

#[test]
fn detached_head_only_needs_head_and_packed_refs() {
    let paths = git_ref_watch_paths(
        PathBuf::from("/repo/.git/worktrees/detached"),
        PathBuf::from("/repo/.git"),
        "0123456789abcdef0123456789abcdef01234567\n",
    );

    assert_eq!(paths.current_ref, None);
    assert_eq!(
        paths.head,
        PathBuf::from("/repo/.git/worktrees/detached/HEAD")
    );
    assert_eq!(paths.packed_refs, PathBuf::from("/repo/.git/packed-refs"));
}
