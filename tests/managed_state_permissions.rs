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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixtureContact {
    ValidRequest,
    NoContact,
}

#[cfg(windows)]
#[derive(Clone, Copy)]
enum ProbeMode {
    ValidRequest,
    NoContact,
}

#[cfg(windows)]
struct MyGeneFixture {
    base: String,
    stop: std::sync::mpsc::Sender<()>,
    server: std::thread::JoinHandle<Result<FixtureContact, String>>,
}

#[cfg(windows)]
fn start_mygene_fixture() -> MyGeneFixture {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    let listener = TcpListener::bind("127.0.0.1:0").expect("local MyGene fixture");
    listener.set_nonblocking(true).expect("nonblocking fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let (stop, stop_rx) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut valid_requests = 0;
        loop {
            if stop_rx.try_recv().is_ok() {
                match listener.accept() {
                    Ok(_) => return Err("unexpected extra MyGene request".to_string()),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        return match valid_requests {
                            0 => Ok(FixtureContact::NoContact),
                            1 => Ok(FixtureContact::ValidRequest),
                            count => Err(format!("observed {count} valid MyGene requests")),
                        };
                    }
                    Err(error) => return Err(format!("fixture final accept: {error}")),
                }
            }
            if Instant::now() >= deadline {
                return Err("local MyGene fixture timed out before contact".to_string());
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    if valid_requests != 0 {
                        return Err("unexpected extra MyGene request".to_string());
                    }
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .map_err(|error| format!("set fixture read timeout: {error}"))?;
                    let mut request = Vec::new();
                    let mut chunk = [0_u8; 1024];
                    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                        let count = stream
                            .read(&mut chunk)
                            .map_err(|error| format!("read MyGene request: {error}"))?;
                        if count == 0 {
                            return Err("MyGene request ended before its headers".to_string());
                        }
                        request.extend_from_slice(&chunk[..count]);
                        if request.len() > 16 * 1024 {
                            return Err("MyGene request headers exceeded 16 KiB".to_string());
                        }
                    }
                    let request = std::str::from_utf8(&request)
                        .map_err(|error| format!("MyGene request was not UTF-8: {error}"))?;
                    let request_line = request
                        .lines()
                        .next()
                        .ok_or_else(|| "MyGene request line was missing".to_string())?;
                    let mut parts = request_line.split_whitespace();
                    let method = parts.next();
                    let target = parts.next();
                    let version = parts.next();
                    if parts.next().is_some()
                        || method != Some("GET")
                        || version != Some("HTTP/1.1")
                    {
                        return Err(format!("unexpected MyGene request line: {request_line}"));
                    }
                    let (path, query) = target
                        .and_then(|target| target.split_once('?'))
                        .ok_or_else(|| {
                            format!("MyGene request query was missing: {request_line}")
                        })?;
                    if path != "/query" {
                        return Err(format!("unexpected MyGene request path: {path}"));
                    }
                    let params = query
                        .split('&')
                        .filter_map(|pair| pair.split_once('='))
                        .collect::<Vec<_>>();
                    let exact_param = |name: &str, value: &str| {
                        let values = params
                            .iter()
                            .filter_map(|(key, found)| (*key == name).then_some(*found))
                            .collect::<Vec<_>>();
                        values == [value]
                    };
                    let query_values = params
                        .iter()
                        .filter_map(|(key, value)| (*key == "q").then_some(*value))
                        .collect::<Vec<_>>();
                    if query_values.len() != 1
                        || !query_values[0].to_ascii_uppercase().contains("BRAF")
                        || !exact_param("species", "human")
                        || !exact_param("size", "1")
                        || !exact_param("from", "0")
                    {
                        return Err(format!("unexpected MyGene query parameters: {query}"));
                    }
                    let body = br#"{"total":1,"hits":[{"symbol":"BRAF","name":"B-Raf proto-oncogene","entrezgene":"673"}]}"#;
                    write!(
                        stream,
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .map_err(|error| format!("write fixture response headers: {error}"))?;
                    stream
                        .write_all(body)
                        .map_err(|error| format!("write fixture response body: {error}"))?;
                    valid_requests += 1;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => return Err(format!("fixture accept: {error}")),
            }
        }
    });
    MyGeneFixture { base, stop, server }
}

#[cfg(windows)]
fn finish_mygene_fixture(fixture: MyGeneFixture) -> Result<FixtureContact, String> {
    let _ = fixture.stop.send(());
    fixture
        .server
        .join()
        .map_err(|_| "local MyGene fixture thread panicked".to_string())?
}

#[cfg(windows)]
fn run_cached_probe(
    root: &std::path::Path,
    mode: ProbeMode,
) -> (std::process::Output, Result<(), String>) {
    let fixture = start_mygene_fixture();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(["search", "gene", "BRAF", "--limit", "1"])
        .env("BIOMCP_CACHE_DIR", root)
        .env("BIOMCP_MYGENE_BASE", &fixture.base)
        .output()
        .expect("run cached probe");
    let contact = finish_mygene_fixture(fixture).and_then(|contact| {
        if matches!(
            (mode, contact),
            (ProbeMode::ValidRequest, FixtureContact::ValidRequest)
                | (ProbeMode::NoContact, FixtureContact::NoContact)
        ) {
            Ok(())
        } else {
            Err(format!(
                "unexpected fixture contact for probe mode: {contact:?}"
            ))
        }
    });
    (output, contact)
}

#[cfg(windows)]
#[test]
fn windows_mygene_fixture_accepts_a_valid_request_after_cold_start_delay() {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let fixture = start_mygene_fixture();
    std::thread::sleep(Duration::from_millis(2_500));
    let mut stream = TcpStream::connect(fixture.base.trim_start_matches("http://"))
        .expect("connect to delayed fixture");
    stream
        .write_all(
            b"GET /query?size=1&q=BRAF&from=0&species=human HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
        )
        .expect("write delayed valid request");
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set delayed response timeout");
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    while !response.windows(6).any(|window| window == b"\"BRAF\"") {
        let count = stream.read(&mut chunk).expect("read delayed response");
        assert!(count > 0, "delayed fixture closed before its response");
        response.extend_from_slice(&chunk[..count]);
    }
    let response = String::from_utf8(response).expect("UTF-8 delayed response");
    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "{response}");
    assert_eq!(
        finish_mygene_fixture(fixture).expect("delayed fixture result"),
        FixtureContact::ValidRequest
    );
}

#[cfg(windows)]
const WINDOWS_ACL_PATH_ENV: &str = "BIOMCP_TEST_ACL_PATH";

#[cfg(windows)]
const WINDOWS_ACL_CONTRACT: &str = r#"
$ErrorActionPreference = 'Stop'
function FailContract([int] $Code, [string] $Message) {
    [Console]::Error.WriteLine($Message)
    exit $Code
}
$path = [Environment]::GetEnvironmentVariable('BIOMCP_TEST_ACL_PATH', 'Process')
if ([String]::IsNullOrWhiteSpace($path)) { FailContract 2 'BIOMCP_TEST_ACL_PATH is missing or blank' }
if (-not [System.IO.Path]::IsPathRooted($path)) { FailContract 3 'BIOMCP_TEST_ACL_PATH is not rooted' }
if (-not [System.IO.File]::Exists($path)) { FailContract 4 'BIOMCP_TEST_ACL_PATH does not exist' }
$attributes = [System.IO.File]::GetAttributes($path)
if (($attributes -band [System.IO.FileAttributes]::Directory) -ne 0 -or
    ($attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    FailContract 5 'BIOMCP_TEST_ACL_PATH is not a regular file'
}
$current = [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$acl = [System.IO.File]::GetAccessControl(
    $path,
    [System.Security.AccessControl.AccessControlSections]::Access
)
if (-not $acl.AreAccessRulesProtected) { FailContract 9 'DACL is not protected' }
$rules = @($acl.GetAccessRules($true, $true, [System.Security.Principal.SecurityIdentifier]))
if ($rules.Count -ne 1) { FailContract 10 "expected exactly one DACL rule; found $($rules.Count)" }
$rule = $rules[0]
if ($rule.IdentityReference.Value -ne $current) { FailContract 11 'DACL rule belongs to the wrong SID' }
if ($rule.AccessControlType -ne [System.Security.AccessControl.AccessControlType]::Allow) { FailContract 12 'DACL rule is not Allow' }
if ($rule.IsInherited) { FailContract 13 'DACL rule is inherited' }
if ($rule.FileSystemRights -ne [System.Security.AccessControl.FileSystemRights]::FullControl) { FailContract 14 "DACL rights are not exact FullControl: $($rule.FileSystemRights)" }
"#;

#[cfg(windows)]
fn inspect_windows_acl(path: Option<&std::path::Path>) -> std::process::Output {
    let mut command = std::process::Command::new("powershell.exe");
    command
        .args(["-NoProfile", "-NonInteractive"])
        .env_remove(WINDOWS_ACL_PATH_ENV);
    if let Some(path) = path {
        command.env(WINDOWS_ACL_PATH_ENV, path);
    }
    command
        .args(["-Command", WINDOWS_ACL_CONTRACT])
        .output()
        .expect("run self-contained ACL contract")
}

#[cfg(windows)]
#[test]
fn windows_acl_contract_rejects_missing_and_nonexistent_paths() {
    let missing = inspect_windows_acl(None);
    assert!(!missing.status.success());
    let missing_stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        missing_stderr.contains("missing or blank"),
        "{missing_stderr}"
    );

    let parent = tempfile::tempdir().expect("temporary ACL parent");
    let nonexistent = inspect_windows_acl(Some(&parent.path().join("does-not-exist")));
    assert!(!nonexistent.status.success());
    let nonexistent_stderr = String::from_utf8_lossy(&nonexistent.stderr);
    assert!(
        nonexistent_stderr.contains("does not exist"),
        "{nonexistent_stderr}"
    );
}

#[cfg(windows)]
#[test]
fn windows_cache_epoch_files_are_user_only_and_reject_hard_links() {
    let parent = tempfile::tempdir().expect("temporary parent");
    let root = parent.path().join("managed");
    let (initial, initial_contact) = run_cached_probe(&root, ProbeMode::ValidRequest);
    initial_contact.expect("initial MyGene fixture result");
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

    let (repaired, repaired_contact) = run_cached_probe(&root, ProbeMode::ValidRequest);
    repaired_contact.expect("repaired MyGene fixture result");
    assert!(
        repaired.status.success(),
        "fast-path ACL repair failed: {}",
        String::from_utf8_lossy(&repaired.stderr)
    );
    for path in [&lock, &marker] {
        let exact_acl = inspect_windows_acl(Some(path));
        let stderr = String::from_utf8_lossy(&exact_acl.stderr);
        assert!(
            exact_acl.status.success(),
            "unexpected epoch DACL for {path:?}: {stderr}"
        );
    }

    let outside = parent.path().join("outside-marker-link");
    fs::hard_link(&marker, &outside).expect("hard link marker");
    let (rejected, rejected_contact) = run_cached_probe(&root, ProbeMode::NoContact);
    rejected_contact.expect("hard-link MyGene fixture result");
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("managed file has 2 links"), "{stderr}");
    assert!(!stderr.contains("fsutil"));
}
