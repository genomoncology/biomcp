use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(unix)]
mod pty_helpers;

struct CommandResult {
    stdout: String,
    stderr: String,
    status: std::process::ExitStatus,
}

impl CommandResult {
    fn from_output(output: Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            status: output.status,
        }
    }
}

fn temp_dir(label: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("biomcp-cache-clear-{label}-"))
        .tempdir()
        .expect("temp dir should be created")
}

fn resolved_cache_root(cache_home: &Path) -> PathBuf {
    cache_home.join("biomcp")
}

fn run_biomcp(args: &[&str], cache_home: &Path, config_home: &Path) -> CommandResult {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(args);
    // The platform cache root ignores XDG_CACHE_HOME on macOS, so the
    // fixture root travels through the explicit override every platform
    // honors.
    command.envs(biomcp_mcp_contract_client::child_cache_env(
        resolved_cache_root(cache_home),
    ));
    command.env("XDG_CONFIG_HOME", config_home);
    command.env_remove("BIOMCP_CACHE_MAX_SIZE");
    command.env_remove("BIOMCP_CACHE_MAX_AGE");
    command.env_remove("RUST_LOG");

    let output = command.output().expect("biomcp command should run");
    CommandResult::from_output(output)
}

#[test]
fn cache_clear_without_tty_refuses_with_plain_stderr() {
    let root = temp_dir("refusal");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    fs::create_dir_all(&cache_home).expect("cache home should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");

    let result = run_biomcp(&["cache", "clear"], &cache_home, &config_home);

    assert_eq!(
        result.status.code(),
        Some(1),
        "expected non-interactive refusal exit code 1\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(
        result.stdout.trim().is_empty(),
        "refusal should not write stdout\nstdout:\n{}",
        result.stdout
    );
    assert!(
        result.stderr.contains("--yes"),
        "refusal should mention --yes\nstderr:\n{}",
        result.stderr
    );
    assert!(
        result.stderr.contains("TTY"),
        "refusal should mention TTY requirements\nstderr:\n{}",
        result.stderr
    );
    assert!(
        !result.stderr.trim_start().starts_with('{'),
        "refusal should stay plain stderr, not JSON\nstderr:\n{}",
        result.stderr
    );
}

#[test]
fn cache_clear_without_tty_refuses_with_plain_stderr_even_under_json() {
    let root = temp_dir("json-refusal");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    fs::create_dir_all(&cache_home).expect("cache home should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");

    let result = run_biomcp(&["--json", "cache", "clear"], &cache_home, &config_home);

    assert_eq!(result.status.code(), Some(1));
    assert!(
        result.stdout.trim().is_empty(),
        "stdout:\n{}",
        result.stdout
    );
    assert!(
        result.stderr.contains("--yes"),
        "stderr:\n{}",
        result.stderr
    );
    assert!(
        !result.stderr.trim_start().starts_with('{'),
        "stderr should remain plain text under --json\nstderr:\n{}",
        result.stderr
    );
}

#[test]
fn cache_clear_yes_preserves_legacy_http_tree() {
    let root = temp_dir("delete-http-only");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let cache_root = resolved_cache_root(&cache_home);
    let http_dir = cache_root.join("http");
    let downloads_dir = cache_root.join("downloads");
    fs::create_dir_all(http_dir.join("nested")).expect("http tree should exist");
    fs::create_dir_all(&downloads_dir).expect("downloads dir should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(http_dir.join("nested").join("entry.bin"), b"clear-me").expect("seed http file");
    fs::write(downloads_dir.join("keep.bin"), b"keep-me").expect("seed downloads file");

    let result = run_biomcp(&["cache", "clear", "--yes"], &cache_home, &config_home);

    assert!(
        result.status.success(),
        "cache clear --yes should succeed\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(
        result.stdout.starts_with("Cache clear:"),
        "stdout:\n{}",
        result.stdout
    );
    assert!(http_dir.exists(), "legacy HTTP cache should remain");
    assert!(downloads_dir.is_dir(), "downloads sibling should remain");
    assert_eq!(
        fs::read(downloads_dir.join("keep.bin")).expect("downloads file should survive"),
        b"keep-me"
    );
}

#[test]
fn cache_clear_yes_json_reports_machine_shape() {
    let root = temp_dir("json-shape");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let http_dir = resolved_cache_root(&cache_home).join("http");
    fs::create_dir_all(&http_dir).expect("http dir should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(http_dir.join("entry.bin"), b"12345").expect("seed http file");

    let result = run_biomcp(
        &["--json", "cache", "clear", "--yes"],
        &cache_home,
        &config_home,
    );

    assert!(
        result.status.success(),
        "json cache clear should succeed\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    let value: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["bytes_freed"], serde_json::json!(0));
    assert_eq!(value["entries_removed"], serde_json::json!(0));
}

#[test]
fn cache_clear_yes_missing_path_is_idempotent() {
    let root = temp_dir("missing-path");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    fs::create_dir_all(&cache_home).expect("cache home should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");

    let result = run_biomcp(
        &["--json", "cache", "clear", "--yes"],
        &cache_home,
        &config_home,
    );

    assert!(
        result.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    let value: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["bytes_freed"], serde_json::json!(0));
    assert_eq!(value["entries_removed"], serde_json::json!(0));
}

#[cfg(unix)]
#[test]
fn legacy_cleanup_unlinks_root_symlink_without_traversal() {
    use std::os::unix::fs::symlink;

    let root = temp_dir("root-symlink");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let cache_root = resolved_cache_root(&cache_home);
    let target_dir = root.path().join("symlink-target");
    let target_file = target_dir.join("outside.txt");
    fs::create_dir_all(&cache_root).expect("cache root should exist");
    fs::create_dir_all(&target_dir).expect("target dir should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(&target_file, b"outside").expect("target file should exist");
    symlink(&target_dir, cache_root.join("http")).expect("http symlink should be created");

    let result = run_biomcp(
        &["--json", "cache", "clean-legacy", "--yes"],
        &cache_home,
        &config_home,
    );

    assert!(
        result.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    let value: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["bytes_freed"], serde_json::Value::Null);
    assert_eq!(value["entries_removed"], serde_json::json!(1));
    assert!(
        !cache_root.join("http").exists(),
        "root symlink should be removed"
    );
    assert!(
        target_file.is_file(),
        "symlink target must not be traversed or removed"
    );
}

#[cfg(unix)]
#[test]
fn cache_clear_tty_accepts_after_confirmation() {
    let root = temp_dir("tty-accept");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let http_dir = resolved_cache_root(&cache_home).join("http");
    fs::create_dir_all(http_dir.join("nested")).expect("http tree should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(http_dir.join("nested").join("entry.bin"), b"accept-me").expect("seed http file");

    let output =
        pty_helpers::run_biomcp_with_tty(&["cache", "clear"], &cache_home, &config_home, "yes")
            .expect("tty cache clear should run");

    assert!(
        output.contains("Cache clear: bytes_freed=0 entries_removed=0"),
        "unexpected PTY output:\n{output}"
    );
    assert!(
        http_dir.exists(),
        "legacy cache remains after current-cache clear"
    );
}

#[cfg(unix)]
#[test]
fn cache_clear_tty_decline_returns_zero_report_without_deleting() {
    let root = temp_dir("tty-decline");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let http_dir = resolved_cache_root(&cache_home).join("http");
    let nested_file = http_dir.join("nested").join("entry.bin");
    fs::create_dir_all(http_dir.join("nested")).expect("http tree should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(&nested_file, b"decline-me").expect("seed http file");

    let output =
        pty_helpers::run_biomcp_with_tty(&["cache", "clear"], &cache_home, &config_home, "no")
            .expect("tty cache clear should run");

    assert!(
        output.contains("Cache clear cancelled: bytes_freed=null entries_removed=0"),
        "unexpected PTY output:\n{output}"
    );
    assert!(
        http_dir.is_dir(),
        "http cache tree should remain after decline"
    );
    assert!(
        nested_file.is_file(),
        "existing cache files should remain after decline"
    );
}

#[test]
fn cache_clear_yes_preserves_downloads_sibling() {
    let root = temp_dir("preserve-downloads");
    let cache_home = root.path().join("cache-home");
    let config_home = root.path().join("config-home");
    let cache_root = resolved_cache_root(&cache_home);
    let http_dir = cache_root.join("http");
    let downloads_dir = cache_root.join("downloads");
    fs::create_dir_all(http_dir.join("nested")).expect("http tree should exist");
    fs::create_dir_all(&downloads_dir).expect("downloads dir should exist");
    fs::create_dir_all(&config_home).expect("config home should exist");
    fs::write(http_dir.join("nested").join("entry.bin"), b"http").expect("seed http file");
    fs::write(downloads_dir.join("keep.bin"), b"downloads").expect("seed downloads file");

    let result = run_biomcp(&["cache", "clear", "--yes"], &cache_home, &config_home);

    assert!(
        result.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(http_dir.exists(), "legacy HTTP cache should remain");
    assert!(downloads_dir.is_dir(), "downloads dir should remain");
    assert!(
        downloads_dir.join("keep.bin").is_file(),
        "downloads file should remain"
    );
}
