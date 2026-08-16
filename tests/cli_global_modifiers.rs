use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(args)
        .output()
        .expect("run biomcp")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout JSON")
}

#[test]
fn json_help_forms_are_success_objects_on_stdout() {
    for args in [
        ["--json", "--help"].as_slice(),
        ["-j", "-h"].as_slice(),
        ["--json", "help"].as_slice(),
        ["--json", "cache", "--help"].as_slice(),
        ["-j", "cache", "-h"].as_slice(),
    ] {
        let output = run(args);
        assert!(output.status.success(), "args={args:?}, output={output:?}");
        assert!(output.stderr.is_empty(), "args={args:?}, output={output:?}");
        let value = stdout_json(&output);
        let object = value.as_object().expect("help object");
        assert_eq!(object.len(), 2, "args={args:?}, json={value}");
        assert_eq!(object.get("kind"), Some(&serde_json::json!("help")));
        assert!(
            object
                .get("content")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|content| content.contains("Usage: biomcp")),
            "args={args:?}, json={value}"
        );
    }
}

#[test]
fn json_display_version_exactly_matches_version_command() {
    let canonical = run(&["--json", "version"]);
    assert!(canonical.status.success(), "{canonical:?}");
    let canonical = stdout_json(&canonical);

    for args in [["--json", "--version"].as_slice(), ["-j", "-V"].as_slice()] {
        let output = run(args);
        assert!(output.status.success(), "args={args:?}, output={output:?}");
        assert!(output.stderr.is_empty(), "args={args:?}, output={output:?}");
        assert_eq!(stdout_json(&output), canonical, "args={args:?}");
    }
}

#[test]
fn genuine_json_parse_error_remains_exit_two_error() {
    let output = run(&["--json", "get", "variant"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty(), "{output:?}");
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "invalid_argument");
}
