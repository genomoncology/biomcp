#[cfg(any(unix, windows))]
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

#[cfg(windows)]
#[test]
fn windows_managed_tree_rejects_hardlinked_files_without_fsutil() {
    let parent = tempfile::tempdir().expect("temporary parent");
    let root = parent.path().join("managed");
    fs::create_dir_all(&root).expect("managed root");
    let file = root.join("local-metadata");
    fs::write(&file, b"fixture-only").expect("managed file");

    let accepted = std::process::Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["cache", "stats"])
        .env("BIOMCP_CACHE_DIR", &root)
        .output()
        .expect("run accepted stats");
    assert!(
        accepted.status.success(),
        "ordinary managed file was rejected: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );

    fs::hard_link(&file, parent.path().join("outside-link")).expect("hard link");

    let failed = std::process::Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["cache", "stats"])
        .env("BIOMCP_CACHE_DIR", &root)
        .output()
        .expect("run rejected stats");

    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("managed file has 2 links"), "{stderr}");
    assert!(!stderr.contains("fsutil"));
}

#[cfg(windows)]
fn run_cached_probe(root: &std::path::Path) -> std::process::Output {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;

    let listener = TcpListener::bind("127.0.0.1:0").expect("local MyGene fixture");
    listener.set_nonblocking(true).expect("nonblocking fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let server = std::thread::spawn(move || {
        for _ in 0..200 {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0_u8; 4096];
                    let _ = stream.read(&mut request);
                    let body = br#"{"total":1,"hits":[{"symbol":"BRAF","name":"B-Raf proto-oncogene","entrezgene":"673"}]}"#;
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .expect("fixture response headers");
                    stream.write_all(body).expect("fixture response body");
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("fixture accept: {error}"),
            }
        }
    });
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["search", "gene", "BRAF", "--limit", "1"])
        .env("BIOMCP_CACHE_DIR", root)
        .env("BIOMCP_MYGENE_BASE", base)
        .output()
        .expect("run cached probe");
    server.join().expect("fixture thread");
    output
}

#[cfg(windows)]
#[test]
fn windows_cache_epoch_files_are_user_only_and_reject_hard_links() {
    let parent = tempfile::tempdir().expect("temporary parent");
    let root = parent.path().join("managed");
    let initial = run_cached_probe(&root);
    assert!(
        initial.status.success(),
        "initial cached probe failed: {}",
        String::from_utf8_lossy(&initial.stderr)
    );
    let lock = root.join(".body-limit-cache-v1.lock");
    let marker = root.join(".body-limit-cache-v1");

    for path in [&lock, &marker] {
        assert!(path.is_file(), "epoch file was not created: {path:?}");
        let broadened = std::process::Command::new("icacls.exe")
            .arg(path)
            .args(["/inheritance:e", "/grant", "*S-1-1-0:(R)"])
            .status()
            .expect("broaden epoch ACL");
        assert!(broadened.success());
    }

    let repaired = run_cached_probe(&root);
    assert!(
        repaired.status.success(),
        "fast-path ACL repair failed: {}",
        String::from_utf8_lossy(&repaired.stderr)
    );
    let acl_contract = r#"
$ErrorActionPreference = 'Stop'
$current = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$acl = [System.IO.File]::GetAccessControl(
    $args[0],
    [System.Security.AccessControl.AccessControlSections]::Access
)
if (-not $acl.AreAccessRulesProtected) { exit 9 }
$rules = @($acl.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier]))
if ($rules.Count -ne 1) { exit 10 }
$rule = $rules[0]
if ($rule.IdentityReference.Value -ne $current) { exit 11 }
if ($rule.AccessControlType -ne [System.Security.AccessControl.AccessControlType]::Allow) { exit 12 }
if ($rule.IsInherited) { exit 13 }
if ($rule.FileSystemRights -ne [System.Security.AccessControl.FileSystemRights]::FullControl) { exit 14 }
"#;
    for path in [&lock, &marker] {
        let exact_acl = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", acl_contract])
            .arg(path)
            .status()
            .expect("enumerate repaired DACL");
        assert!(exact_acl.success(), "unexpected epoch DACL: {path:?}");
    }

    let outside = parent.path().join("outside-marker-link");
    fs::hard_link(&marker, &outside).expect("hard link marker");
    let rejected = run_cached_probe(&root);
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("managed file has 2 links"), "{stderr}");
    assert!(!stderr.contains("fsutil"));
}
