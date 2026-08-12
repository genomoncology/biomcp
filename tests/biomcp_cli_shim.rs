use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn output(binary: &Path, arguments: &[&str]) -> Output {
    for _ in 0..5 {
        match Command::new(binary).args(arguments).output() {
            Ok(output) => return output,
            Err(error) if error.raw_os_error() == Some(26) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(error) => panic!("run command: {error}"),
        }
    }
    panic!("run command: executable remained busy")
}

#[test]
fn shim_preserves_success_error_and_json_output() {
    let biomcp = Path::new(env!("CARGO_BIN_EXE_biomcp"));
    let shim = Path::new(env!("CARGO_BIN_EXE_biomcp-cli"));
    for arguments in [
        &["--version"][..],
        &["--json", "list"][..],
        &["not-a-command"][..],
    ] {
        let direct = output(biomcp, arguments);
        let forwarded = output(shim, arguments);
        assert_eq!(
            forwarded.status.code(),
            direct.status.code(),
            "{arguments:?}"
        );
        assert_eq!(forwarded.stdout, direct.stdout, "{arguments:?}");
        assert_eq!(forwarded.stderr, direct.stderr, "{arguments:?}");
    }
}

#[test]
fn shim_fails_clearly_without_searching_path_for_a_sibling() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let directory = temporary.path().join("wheel with spaces");
    fs::create_dir(&directory).expect("create test directory");
    let installed_shim = directory.join(if cfg!(windows) {
        "biomcp-cli.exe"
    } else {
        "biomcp-cli"
    });
    fs::copy(env!("CARGO_BIN_EXE_biomcp-cli"), &installed_shim).expect("copy shim");
    let result = output(&installed_shim, &["--version"]);
    assert_eq!(result.status.code(), Some(126));
    assert!(String::from_utf8_lossy(&result.stderr).contains("cannot run sibling"));
}

#[cfg(unix)]
#[test]
fn shim_rejects_a_non_executable_sibling() {
    use std::os::unix::fs::PermissionsExt;
    let temporary = tempfile::tempdir().expect("temporary directory");
    let shim = temporary.path().join("biomcp-cli");
    let sibling = temporary.path().join("biomcp");
    fs::copy(env!("CARGO_BIN_EXE_biomcp-cli"), &shim).expect("copy shim");
    fs::write(&sibling, b"not executable").expect("write sibling");
    fs::set_permissions(&sibling, fs::Permissions::from_mode(0o600)).expect("set permissions");
    let result = output(&shim, &["--version"]);
    assert_eq!(result.status.code(), Some(126));
    assert!(String::from_utf8_lossy(&result.stderr).contains("is not executable"));
}
