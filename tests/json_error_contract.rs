use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct CommandResult {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_biomcp(args: &[&str]) -> CommandResult {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn biomcp");

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().expect("poll biomcp").is_some() {
            let output = child.wait_with_output().expect("collect biomcp output");
            return CommandResult {
                code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            };
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("collect timed-out biomcp output");
            panic!(
                "biomcp timed out after 10s\nargs: {args:?}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn assert_json_error(result: &CommandResult, expected_exit: i32, expected_code: &str) {
    assert_eq!(
        result.code,
        Some(expected_exit),
        "unexpected exit\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
    assert!(
        result.stderr.trim().is_empty(),
        "json errors should not write stderr\nstderr:\n{}",
        result.stderr
    );

    let value: serde_json::Value =
        serde_json::from_str(&result.stdout).expect("stdout should be valid JSON");
    assert_eq!(value["error"]["code"], expected_code, "json={value}");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| !message.is_empty()),
        "json error should include a message: {value}"
    );
}

#[test]
fn json_mode_not_found_error_writes_json_stdout_and_exit_1() {
    let result = run_biomcp(&["--json", "skill", "show", "not-a-real-skill"]);

    assert_json_error(&result, 1, "not_found");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid json");
    assert_eq!(value["_meta"]["not_found"], true, "json={value}");
}

#[test]
fn json_mode_gene_not_found_error_writes_json_stdout_and_exit_1() {
    let result = run_biomcp(&["--json", "get", "gene", "ZZZNOTAREALGENE"]);

    assert_json_error(&result, 1, "not_found");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid json");
    assert_eq!(value["_meta"]["not_found"], true, "json={value}");
}

#[test]
fn json_mode_invalid_argument_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["--json", "get", "variant", "not-a-variant-xyz"]);

    assert_json_error(&result, 2, "invalid_argument");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid json");
    assert_eq!(value["_meta"]["not_found"], false, "json={value}");
}

#[test]
fn json_mode_missing_required_arg_parse_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["--json", "get", "variant"]);

    assert_json_error(&result, 2, "invalid_argument");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid json");
    assert_eq!(value["_meta"]["not_found"], false, "json={value}");
}

#[test]
fn short_json_flag_missing_required_arg_parse_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["-j", "get", "variant"]);

    assert_json_error(&result, 2, "invalid_argument");
}

#[test]
fn json_mode_unknown_subcommand_parse_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["--json", "get", "not-an-entity"]);

    assert_json_error(&result, 2, "invalid_argument");
}

#[test]
fn json_mode_unknown_flag_parse_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["--json", "get", "variant", "BRAF V600E", "--not-a-flag"]);

    assert_json_error(&result, 2, "invalid_argument");
}

#[test]
fn human_mode_parse_error_stays_plain_stderr() {
    let result = run_biomcp(&["get", "variant"]);

    assert_eq!(result.code, Some(2));
    assert!(
        result.stdout.trim().is_empty(),
        "human parse errors should not write stdout\nstdout:\n{}",
        result.stdout
    );
    assert!(
        !result.stderr.trim().is_empty(),
        "human parse errors should stay on stderr"
    );
    assert!(
        serde_json::from_str::<serde_json::Value>(&result.stderr).is_err(),
        "human stderr should not become JSON"
    );
}

#[test]
fn human_mode_error_stays_plain_stderr() {
    let result = run_biomcp(&["skill", "show", "not-a-real-skill"]);

    assert_eq!(result.code, Some(1));
    assert!(
        result.stdout.trim().is_empty(),
        "human errors should not write stdout\nstdout:\n{}",
        result.stdout
    );
    assert!(
        result.stderr.starts_with("Error: "),
        "human errors should stay on stderr\nstderr:\n{}",
        result.stderr
    );
    assert!(
        serde_json::from_str::<serde_json::Value>(&result.stderr).is_err(),
        "human stderr should not become JSON"
    );
}
