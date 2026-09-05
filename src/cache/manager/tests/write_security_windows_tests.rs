#![cfg(windows)]

use std::fs;
use std::path::Path;

use http_cache::CacheManager;

use super::*;

const ACL_PATH_ENV: &str = "BIOMCP_TEST_CACHE_WRITE_ACL_PATH";
const ACL_CONTRACT: &str = r#"
$ErrorActionPreference = 'Stop'
$path = [Environment]::GetEnvironmentVariable('BIOMCP_TEST_CACHE_WRITE_ACL_PATH', 'Process')
if ([String]::IsNullOrWhiteSpace($path)) { exit 2 }
$acl = Get-Acl -LiteralPath $path
if (-not $acl.AreAccessRulesProtected) { exit 3 }
$current = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$rules = @($acl.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier]))
if ($rules.Count -ne 1) { exit 4 }
$rule = $rules[0]
if ($rule.IdentityReference.Value -ne $current) { exit 5 }
if ($rule.AccessControlType -ne [System.Security.AccessControl.AccessControlType]::Allow) { exit 6 }
if ($rule.IsInherited) { exit 7 }
if (($rule.FileSystemRights -band [System.Security.AccessControl.FileSystemRights]::FullControl) -ne
    [System.Security.AccessControl.FileSystemRights]::FullControl) { exit 8 }
"#;

fn inert_manager(cache_root: &Path) -> SizeAwareCacheManager {
    SizeAwareCacheManager::new_with_services(
        cache_root.join("http"),
        test_config(cache_root, u64::MAX / 2, DiskFreeThreshold::Percent(1)),
        |_| Ok(0),
        |_| {
            Ok(FilesystemSpace {
                available_bytes: 99,
                total_bytes: 100,
            })
        },
        |_, _, _, _| unreachable!("test cache must not schedule eviction"),
    )
}

fn assert_protected_current_user_acl(path: &Path) {
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", ACL_CONTRACT])
        .env(ACL_PATH_ENV, path)
        .output()
        .expect("run ACL contract");
    assert!(
        output.status.success(),
        "unexpected ACL for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn put_skips_unrelated_hardlink_and_protects_every_touched_path_acl() {
    let root = TempDirGuard::new("windows-bounded-put-hardening");
    let cache_path = root.path().join("http");
    let unrelated = cache_path.join("unrelated/sentinel");
    fs::create_dir_all(unrelated.parent().expect("unrelated parent")).expect("unrelated tree");
    fs::write(&unrelated, b"must remain untouched").expect("unrelated sentinel");
    let outside = root.path().join("outside-link");
    fs::hard_link(&unrelated, &outside).expect("unrelated hard link");
    let key = "https://example.test/windows?q=é";

    inert_manager(root.path())
        .put(
            key.into(),
            test_http_response(b"private body"),
            test_policy(),
        )
        .await
        .expect("put must not inspect unrelated hard link");

    assert_eq!(
        fs::read(&outside).expect("outside unchanged"),
        b"must remain untouched"
    );
    let metadata = cacache::metadata_sync(&cache_path, key)
        .expect("cache metadata")
        .expect("written metadata");
    let index = crate::cache::index_bucket_path(&cache_path, key);
    let blob = crate::cache::content_path(&cache_path, &metadata.integrity);
    let content_root = crate::cache::content_root(&cache_path);
    let (algorithm, _) = metadata.integrity.to_hex();
    for path in [
        root.path().join(".biomcp-operation.lock"),
        root.path().join(".biomcp-key-locks"),
        crate::cache::key_lock_path(root.path(), key),
        cache_path.clone(),
        cache_path.join("tmp"),
        cache_path.join("index-v5"),
        index
            .parent()
            .and_then(Path::parent)
            .expect("first index shard")
            .to_path_buf(),
        index.parent().expect("second index shard").to_path_buf(),
        index,
        content_root.clone(),
        content_root.join(algorithm.to_string()),
        blob.parent()
            .and_then(Path::parent)
            .expect("first content shard")
            .to_path_buf(),
        blob.parent().expect("second content shard").to_path_buf(),
        blob,
    ] {
        assert_protected_current_user_acl(&path);
    }
}
