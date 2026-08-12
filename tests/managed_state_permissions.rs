#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
#[test]
fn cache_open_narrows_only_managed_paths_and_rejects_hardlinks() {
    let parent = tempfile::tempdir().expect("temporary parent");
    let root = parent.path().join("managed");
    let http = root.join("http");
    fs::create_dir_all(&http).expect("managed dirs");
    let file = root.join("local-metadata");
    fs::write(&file, b"fixture-only").expect("managed file");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o777)).expect("broad root");
    fs::set_permissions(&http, fs::Permissions::from_mode(0o777)).expect("broad http");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o666)).expect("broad file");

    let status = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["cache", "stats"])
        .env("BIOMCP_CACHE_DIR", &root)
        .status()
        .expect("run cache stats");
    assert!(status.success());
    assert_eq!(
        fs::metadata(&root).expect("root mode").mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&http).expect("http mode").mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&file).expect("file mode").mode() & 0o777,
        0o600
    );
    assert_ne!(
        fs::metadata(parent.path()).expect("parent mode").mode() & 0o777,
        0o700
    );

    fs::hard_link(&file, parent.path().join("outside-link")).expect("hard link");
    let failed = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["cache", "stats"])
        .env("BIOMCP_CACHE_DIR", &root)
        .output()
        .expect("run rejected stats");
    assert!(!failed.status.success());
    assert_eq!(fs::metadata(&file).expect("links").nlink(), 2);
}

#[cfg(windows)]
#[test]
fn windows_cache_root_is_restricted_to_the_current_user() {
    let parent = tempfile::tempdir().expect("temporary parent");
    let root = parent.path().join("managed");
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["cache", "stats"])
        .env("BIOMCP_CACHE_DIR", &root)
        .status()
        .expect("run cache stats");
    assert!(status.success());
    let acl = std::process::Command::new("icacls.exe")
        .arg(&root)
        .output()
        .expect("inspect ACL");
    assert!(acl.status.success());
    let user = std::env::var("USERNAME").expect("current user");
    assert!(String::from_utf8_lossy(&acl.stdout).contains(&user));
}
