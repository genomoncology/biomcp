use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use biomcp_mcp_contract_client::start_ols4_stub;

struct CommandResult {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_biomcp(args: &[&str]) -> CommandResult {
    run_biomcp_with_env(args, &[])
}

fn run_biomcp_with_env(args: &[&str], env: &[(&str, &str)]) -> CommandResult {
    let mut child = Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(args)
        .envs(env.iter().copied())
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

const POST_CHILD_FIXTURE_RESULT_TIMEOUT: Duration = Duration::from_secs(1);

struct MyGeneFixture {
    base_url: String,
    request_rx: mpsc::Receiver<Result<String, String>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl MyGeneFixture {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind MyGene fixture");
        listener
            .set_nonblocking(true)
            .expect("make MyGene fixture nonblocking");
        let base_url = format!("http://{}", listener.local_addr().expect("fixture address"));
        let (request_tx, request_rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            loop {
                if thread_stop.load(Ordering::Relaxed) {
                    return;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        let result = serve_mygene_request(stream);
                        let failed = result.is_err();
                        let _ = request_tx.send(result);
                        if failed {
                            return;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => {
                        let _ = request_tx.send(Err(format!("fixture accept failed: {error}")));
                        return;
                    }
                }
            }
        });
        Self {
            base_url,
            request_rx,
            stop,
            thread: Some(thread),
        }
    }

    fn received_request(&self) -> String {
        self.request_rx
            .recv_timeout(POST_CHILD_FIXTURE_RESULT_TIMEOUT)
            .expect("fixture result should be available after biomcp exits")
            .expect("fixture should serve a valid request")
    }
}

impl Drop for MyGeneFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join MyGene fixture thread");
        }
    }
}

fn serve_mygene_request(mut stream: TcpStream) -> Result<String, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set fixture read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("set fixture write timeout: {error}"))?;

    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = stream
            .read(&mut buffer)
            .map_err(|error| format!("read fixture request: {error}"))?;
        if read == 0 || request.len() + read > 16 * 1024 {
            return Err("fixture request ended early or exceeded 16 KiB".into());
        }
        request.extend_from_slice(&buffer[..read]);
    }

    let request = String::from_utf8(request).map_err(|error| format!("request UTF-8: {error}"))?;
    let request_target = request
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("GET "))
        .and_then(|line| line.strip_suffix(" HTTP/1.1"))
        .ok_or_else(|| format!("unexpected fixture request line: {request:?}"))?
        .to_owned();

    let body = r#"{"total":0,"hits":[]}"#;
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .map_err(|error| format!("write fixture response: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("flush fixture response: {error}"))?;
    Ok(request_target)
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
fn mygene_fixture_without_request_stops_on_drop() {
    let fixture = MyGeneFixture::start();
    drop(fixture);
}

#[test]
fn json_mode_gene_not_found_error_writes_json_stdout_and_exit_1() {
    let fixture = MyGeneFixture::start();
    let (_ols_thread, ols_url) = start_ols4_stub().expect("start OLS4 fixture");
    let result = run_biomcp_with_env(
        &["--json", "get", "gene", "ZZZNOTAREALGENE"],
        &[
            ("BIOMCP_MYGENE_BASE", &fixture.base_url),
            ("BIOMCP_OLS4_BASE", &ols_url),
            ("BIOMCP_HPO_BASE", "http://127.0.0.1:9"),
            ("UMLS_API_KEY", ""),
        ],
    );
    let request_target = fixture.received_request();

    assert!(
        request_target.starts_with("/query?"),
        "request={request_target}"
    );
    assert!(
        request_target.contains("q=symbol%3A%22ZZZNOTAREALGENE%22"),
        "request={request_target}"
    );
    assert!(
        request_target.contains("species=human"),
        "request={request_target}"
    );
    assert!(
        request_target.contains("size=1"),
        "request={request_target}"
    );
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
fn parsed_json_errors_keep_command_collection_paths_iterable() {
    let rows: &[(&[&str], &[&str])] = &[
        (
            &["--json", "search", "article", "test", "--limit", "999"],
            &["results"],
        ),
        (
            &["--json", "article", "recommendations", "1", "--limit", "0"],
            &["recommendations"],
        ),
        (
            &["--json", "article", "citations", "1", "--limit", "0"],
            &["edges"],
        ),
        (
            &[
                "--json", "search", "drug", "aspirin", "--region", "eu", "--target", "EGFR",
            ],
            &["regions", "eu", "results"],
        ),
        (
            &[
                "--json",
                "search",
                "adverse-event",
                "MMR",
                "--source",
                "vaers",
                "--count",
                "reaction",
            ],
            &["buckets"],
        ),
        (
            &["get", "article", "1", "assets", "--pdf", "--json"],
            &["assets"],
        ),
    ];

    for (args, path) in rows {
        let result = run_biomcp(args);
        assert_json_error(&result, 2, "invalid_argument");
        let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid JSON");
        let collection = path
            .iter()
            .fold(&value, |current, segment| &current[segment]);
        assert_eq!(
            collection,
            &serde_json::json!([]),
            "args={args:?}, json={value}"
        );
    }
}

#[test]
fn runtime_json_errors_keep_discover_concepts_iterable() {
    let result = run_biomcp_with_env(
        &["--no-cache", "--json", "discover", "melanoma"],
        &[
            ("BIOMCP_OLS4_BASE", "http://127.0.0.1:9"),
            ("BIOMCP_MEDLINEPLUS_BASE", "http://127.0.0.1:9"),
            ("UMLS_API_KEY", ""),
        ],
    );

    assert_json_error(&result, 1, "http_middleware");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid JSON");
    assert_eq!(value["concepts"], serde_json::json!([]), "json={value}");
}

#[test]
fn vaers_aggregate_and_pre_dispatch_errors_remain_keyless() {
    let vaers = run_biomcp(&[
        "--json",
        "search",
        "adverse-event",
        "MMR",
        "--source",
        "vaers",
        "--type",
        "recall",
    ]);
    assert_json_error(&vaers, 2, "invalid_argument");
    let value: serde_json::Value = serde_json::from_str(&vaers.stdout).expect("valid JSON");
    assert!(value.get("results").is_none(), "json={value}");
}

#[test]
fn json_mode_missing_required_arg_parse_error_writes_json_stdout_and_exit_2() {
    let result = run_biomcp(&["--json", "get", "variant"]);

    assert_json_error(&result, 2, "invalid_argument");
    let value: serde_json::Value = serde_json::from_str(&result.stdout).expect("valid json");
    assert_eq!(value["_meta"]["not_found"], false, "json={value}");
    assert!(
        value.get("results").is_none(),
        "pre-dispatch errors stay keyless: {value}"
    );
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
