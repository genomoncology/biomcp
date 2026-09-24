//! Patient record contract across the CLI, stdio MCP, and serve-http.
//!
//! Every FHIR response here is synthetic and comes from an in-test fixture.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use biomcp_mcp_contract_client::{ContractHarness, call_biomcp, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::json;

const ID: &str = "SYNTH-PT-7Q.42";
const REFUSAL: &str = "`serve-http` refuses them";
const NO_FILTER: &str = "search patient needs at least one of";
const SNOMED: &str = "http://snomed.info/sct|44054006";

type Route = dyn Fn(&str) -> (u16, Vec<(String, String)>, String) + Send + Sync;

/// A blocking HTTP/1.1 fixture that records each request target.
struct FhirFixture {
    base: String,
    targets: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl FhirFixture {
    fn start(
        route: impl Fn(&str) -> (u16, Vec<(String, String)>, String) + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind fixture");
        listener.set_nonblocking(true).expect("nonblocking");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let targets = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let route: Arc<Route> = Arc::new(route);
        let (thread_targets, thread_stop) = (Arc::clone(&targets), Arc::clone(&stop));
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).ok();
                        let mut request = Vec::new();
                        let mut chunk = [0_u8; 4096];
                        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                            match stream.read(&mut chunk) {
                                Ok(0) | Err(_) => break,
                                Ok(read) => request.extend_from_slice(&chunk[..read]),
                            }
                        }
                        let text = String::from_utf8_lossy(&request);
                        let target = text
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or_default()
                            .to_string();
                        thread_targets.lock().expect("targets").push(target.clone());
                        let (status, headers, body) = route(&target);
                        let mut head = format!(
                            "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n",
                            body.len()
                        );
                        for (name, value) in headers {
                            head.push_str(&format!("{name}: {value}\r\n"));
                        }
                        head.push_str("\r\n");
                        let _ = stream.write_all(head.as_bytes());
                        let _ = stream.write_all(body.as_bytes());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => return,
                }
            }
        });
        Self {
            base,
            targets,
            stop,
            thread: Some(thread),
        }
    }

    fn fhir_base(&self) -> String {
        format!("{}/fhir", self.base)
    }

    fn requests(&self) -> usize {
        self.targets.lock().expect("targets").len()
    }
}

impl Drop for FhirFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn json_reply(status: u16, body: serde_json::Value) -> (u16, Vec<(String, String)>, String) {
    (
        status,
        vec![("Content-Type".into(), "application/fhir+json".into())],
        body.to_string(),
    )
}

fn redirect(location: String) -> (u16, Vec<(String, String)>, String) {
    (302, vec![("Location".into(), location)], String::new())
}

fn patient() -> serde_json::Value {
    json!({"resourceType": "Patient", "id": ID, "gender": "female", "birthDate": "1970-01-01"})
}

fn bundle(next: Option<String>) -> serde_json::Value {
    let mut bundle = json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": [{"search": {"mode": "match"}, "resource": {
            "resourceType": "Condition",
            "id": "synthetic-condition-1",
            "subject": {"reference": format!("Patient/{ID}")},
            "clinicalStatus": {"coding": [{"code": "active"}]},
            "code": {"text": "Synthetic condition one"}
        }}]
    });
    if let Some(next) = next {
        bundle["link"] = json!([{"relation": "next", "url": next}]);
    }
    bundle
}

fn capability() -> serde_json::Value {
    json!({"resourceType": "CapabilityStatement", "rest": [{"mode": "server", "resource": [{
        "type": "Patient",
        "searchParam": [{"name": "gender"}, {"name": "birthdate"}, {"name": "_has"}]
    }]}]})
}

/// A full synthetic Patient that a server ignoring `_elements` would send.
fn leaky_patient(id: &str) -> serde_json::Value {
    json!({
        "resourceType": "Patient", "id": id, "gender": "female", "birthDate": "1970-01-01",
        "name": [{"family": "Synthleak", "given": ["Nameleak"]}],
        "address": [{"line": ["1 Addressleak Way"], "city": "Cityleak"}],
        "telecom": [{"system": "phone", "value": "555-0100-leak"}]
    })
}

/// A Patient search page that ignores `_elements` and `_count`.
fn patient_page() -> serde_json::Value {
    let mut entries = vec![
        json!({"resource": {"resourceType": "Condition", "id": "condleak", "code": {"text": "Conditionleak"}}}),
        json!({"resource": leaky_patient("a/b"), "search": {"mode": "match"}}),
    ];
    for n in 1..=5 {
        entries.push(json!({"resource": leaky_patient(&format!("SYNTH-PT-{n}")), "search": {"mode": "match"}}));
    }
    json!({"resourceType": "Bundle", "type": "searchset", "total": 42, "entry": entries})
}

/// A server with one patient, one page of conditions, metadata that lists
/// every patient search parameter, and a leaky Patient search page.
fn healthy_fixture() -> FhirFixture {
    FhirFixture::start(|target| {
        if target == "/fhir/metadata" {
            json_reply(200, capability())
        } else if target.starts_with("/fhir/Patient?") {
            json_reply(200, patient_page())
        } else if target.starts_with("/fhir/Patient/") {
            json_reply(200, patient())
        } else {
            json_reply(200, bundle(None))
        }
    })
}

fn harness() -> ContractHarness {
    ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"))
}

fn typed(tool: &'static str, arguments: serde_json::Value) -> CallToolRequestParams {
    let arguments = arguments.as_object().expect("object").clone();
    CallToolRequestParams::new(tool).with_arguments(arguments)
}

async fn assert_http_refuses(request: CallToolRequestParams) -> anyhow::Result<()> {
    let fixture = healthy_fixture();
    let harness = harness();
    let env = [("BIOMCP_FHIR_BASE", fixture.fhir_base())];
    let (mut child, base_url) = harness.spawn_http_server(&env).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        let response = client.peer().call_tool(request).await?;
        let text = first_text(&response.content);
        anyhow::ensure!(response.is_error == Some(true), "not refused: {text}");
        anyhow::ensure!(text.contains(REFUSAL), "{text}");
        anyhow::ensure!(!text.contains(ID), "{text}");
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;
    child.kill().await.ok();
    result?;
    assert_eq!(fixture.requests(), 0, "serve-http sent a FHIR request");
    Ok(())
}

fn shell(command: String) -> CallToolRequestParams {
    typed("biomcp", json!({"command": command}))
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_get_patient_through_the_shell_tool() -> anyhow::Result<()> {
    assert_http_refuses(shell(format!("biomcp get patient {ID} conditions"))).await
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_a_filtered_patient_search_through_the_shell_tool() -> anyhow::Result<()>
{
    assert_http_refuses(shell(format!(
        "biomcp search patient --gender female --condition \"{SNOMED}\" --count"
    )))
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_patient_through_typed_search() -> anyhow::Result<()> {
    assert_http_refuses(typed("search", json!({"entity": "patient"}))).await
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_patient_through_typed_get() -> anyhow::Result<()> {
    assert_http_refuses(typed(
        "get",
        json!({"entity": "patient", "id": ID, "sections": ["conditions"]}),
    ))
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_batch_patient() -> anyhow::Result<()> {
    assert_http_refuses(shell(format!("biomcp batch patient {ID},SYNTH-PT-2"))).await
}

#[tokio::test(flavor = "multi_thread")]
async fn stdio_typed_get_reads_the_patient_and_conditions() -> anyhow::Result<()> {
    let fixture = healthy_fixture();
    let cache = tempfile::tempdir()?;
    let env = [
        ("BIOMCP_FHIR_BASE", fixture.fhir_base()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
    ];
    let client = harness().spawn_stdio_client(&env).await?;
    let response = client
        .peer()
        .call_tool(typed(
            "get",
            json!({"entity": "patient", "id": ID, "sections": ["conditions"]}),
        ))
        .await?;
    let text = first_text(&response.content).to_string();
    assert_eq!(response.is_error, Some(false), "{text}");
    assert!(text.contains("Synthetic condition one"), "{text}");
    assert!(!text.contains(&fixture.base), "{text}");

    let search = client
        .peer()
        .call_tool(typed("search", json!({"entity": "patient"})))
        .await?;
    assert!(first_text(&search.content).contains(NO_FILTER));
    let typed_gender = client
        .peer()
        .call_tool(typed("search", json!({"entity": "patient", "gender": "female"})))
        .await;
    let refusal = match typed_gender {
        Ok(response) => {
            assert_eq!(response.is_error, Some(true));
            first_text(&response.content).to_string()
        }
        Err(error) => error.to_string(),
    };
    assert!(refusal.contains("unknown patient search field: gender"), "{refusal}");
    let shell = call_biomcp(&client, "biomcp search patient").await?;
    assert!(first_text(&shell.content).contains(NO_FILTER));
    let requests = fixture.requests();

    let count = call_biomcp(
        &client,
        &format!("biomcp search patient --gender female --condition \"{SNOMED}\" --count"),
    )
    .await?;
    let count_text = first_text(&count.content).to_string();
    client.cancel().await?;
    assert_eq!(requests, 2, "one Patient read and one Condition page");
    assert!(count_text.contains("Server-reported total: 42"), "{count_text}");
    assert_eq!(
        fixture.requests(),
        4,
        "the shell search reads metadata and one Patient page"
    );
    Ok(())
}

fn cli(env: &[(&str, String)], args: &[&str]) -> std::process::Output {
    let harness = harness();
    let mut command = Command::new(&harness.biomcp_bin);
    command.env_remove("BIOMCP_FHIR_BASE");
    for (key, value) in env {
        command.env(key, value);
    }
    command.args(args).output().expect("run biomcp")
}

#[test]
fn ids_outside_the_fhir_rule_are_refused_before_any_request() {
    let fixture = healthy_fixture();
    let long = "a".repeat(65);
    for id in ["a/b", "a?b", "../Patient", long.as_str()] {
        let output = cli(
            &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
            &["get", "patient", id],
        );
        assert!(!output.status.success(), "{id}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains(id), "{stderr}");
    }
    assert_eq!(fixture.requests(), 0);
}

#[test]
fn an_unset_base_names_the_variable_and_exits_non_zero() {
    let output = cli(&[], &["get", "patient", ID]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("BIOMCP_FHIR_BASE"), "{stderr}");
}

#[test]
fn bad_search_values_limits_and_no_filter_send_no_request() {
    let fixture = healthy_fixture();
    let env = [("BIOMCP_FHIR_BASE", fixture.fhir_base())];
    let cases: &[&[&str]] = &[
        &["--gender", "F"],
        &["--gender", "male,female"],
        &["--born-after", "1950-13-01"],
        &["--born-after", "ge1950"],
        &["--born-before", "1950,1960"],
        &["--condition", "44054006"],
        &["--condition", "|44054006"],
        &["--condition", "http://snomed.info/sct|"],
        &["--condition", "http://snomed.info/sct|1,2"],
        &["--gender", "female", "--limit", "0"],
        &["--gender", "female", "--limit", "51"],
        &[],
        &["free", "text"],
    ];
    for case in cases {
        let mut args = vec!["search", "patient"];
        args.extend_from_slice(case);
        let output = cli(&env, &args);
        assert!(!output.status.success(), "{case:?}");
        if case.is_empty() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains(NO_FILTER),
                "{case:?}"
            );
        }
    }
    assert_eq!(fixture.requests(), 0, "a refused search sent a request");
}

#[test]
fn search_output_keeps_only_id_gender_and_birth_date() {
    let fixture = healthy_fixture();
    let env = [("BIOMCP_FHIR_BASE", fixture.fhir_base())];
    for json in [false, true] {
        let mut args = vec!["search", "patient", "--gender", "female", "--limit", "3"];
        if json {
            args.insert(0, "--json");
        }
        let output = cli(&env, &args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{stdout}");
        for leak in ["leak", "Condition", "a/b"] {
            assert!(!stdout.contains(leak), "{leak} in {stdout}");
        }
        assert!(stdout.contains("SYNTH-PT-2"), "{stdout}");
        assert!(!stdout.contains("SYNTH-PT-3"), "more than --limit: {stdout}");
        assert!(stdout.contains("biomcp get patient SYNTH-PT-1"), "{stdout}");
    }
}

#[test]
fn count_prints_the_server_reported_total_or_says_there_is_none() {
    let fixture = healthy_fixture();
    let env = [("BIOMCP_FHIR_BASE", fixture.fhir_base())];
    let output = cli(&env, &["search", "patient", "--condition", SNOMED, "--count"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Server-reported total: 42"));
    let no_total = FhirFixture::start(|target| {
        if target == "/fhir/metadata" {
            json_reply(200, capability())
        } else {
            json_reply(200, json!({"resourceType": "Bundle", "type": "searchset", "entry": [
                {"resource": leaky_patient("SYNTH-PT-1")}
            ]}))
        }
    });
    let env = [("BIOMCP_FHIR_BASE", no_total.fhir_base())];
    let output = cli(&env, &["search", "patient", "--condition", SNOMED, "--count"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("The FHIR server reported no count."), "{stdout}");
    assert!(!stdout.contains("total: 1"), "{stdout}");
    let output = cli(&env, &["--json", "search", "patient", "--condition", SNOMED, "--count"]);
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("count JSON");
    assert_eq!(value["server_reported_total"], serde_json::Value::Null);
}

fn files_containing(root: &Path, needle: &str) -> Vec<String> {
    let mut hits = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if std::fs::read(&path)
                .is_ok_and(|bytes| String::from_utf8_lossy(&bytes).contains(needle))
                || path.display().to_string().contains(needle)
            {
                hits.push(path.display().to_string());
            }
        }
    }
    hits
}

/// Runs `get patient` at debug and at trace level against one fixture. The ID
/// and the server's host and port must stay out of stderr, the error, and the
/// cache directory. A failing case must print its exact user-facing error.
fn assert_trace_run_leaks_nothing(case: &str, fixture: &FhirFixture, error: Option<&str>) {
    for level in ["debug", "trace"] {
        assert_run_leaks_nothing(&format!("{case} at {level}"), fixture, error, level);
    }
}

fn assert_run_leaks_nothing(case: &str, fixture: &FhirFixture, error: Option<&str>, level: &str) {
    let expect_error = error.is_some();
    let cache = tempfile::tempdir().expect("cache dir");
    let env = [
        ("BIOMCP_FHIR_BASE", fixture.fhir_base()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        (
            "XDG_CACHE_HOME",
            cache.path().join("xdg").display().to_string(),
        ),
        ("RUST_LOG", level.to_string()),
    ];
    let authority = fixture.base.trim_start_matches("http://");
    for json in [false, true] {
        let mut args = vec!["get", "patient", ID, "conditions"];
        if json {
            args.insert(0, "--json");
        }
        let output = cli(&env, &args);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(!output.status.success(), expect_error, "{case}: {stderr}");
        if let Some(text) = error {
            if json {
                assert!(
                    stdout.contains(&format!("\"{text}.\"")),
                    "{case}: JSON error: {stdout}"
                );
            } else {
                assert!(
                    stderr.contains(&format!("Error: {text}.")),
                    "{case}: error: {stderr}"
                );
            }
        }
        assert!(!stderr.contains(ID), "{case}: stderr names the ID");
        assert!(
            !stderr.contains(authority),
            "{case}: stderr names the server host and port"
        );
        assert!(
            !stdout.contains(authority),
            "{case}: stdout names the server host and port"
        );
        if expect_error {
            assert!(!stdout.contains(ID), "{case}: error output names the ID");
        }
    }
    let hits = files_containing(cache.path(), ID);
    assert!(hits.is_empty(), "{case}: cache holds the ID in {hits:?}");
    let hits = files_containing(cache.path(), "Synthetic condition");
    assert!(
        hits.is_empty(),
        "{case}: cache holds a response in {hits:?}"
    );
}

#[test]
fn trace_logging_leaks_no_id_or_server_on_any_path() {
    let other = FhirFixture::start(|_| json_reply(200, patient()));
    let other_base = other.base.clone();
    let degraded = FhirFixture::start(move |target| {
        if target.starts_with("/fhir/Patient/") {
            json_reply(200, patient())
        } else if target.starts_with("/fhir/Condition?") {
            json_reply(200, bundle(Some(format!("/fhir?page=2&patient={ID}"))))
        } else {
            redirect(format!("{other_base}/fhir/{ID}/page-2"))
        }
    });
    assert_trace_run_leaks_nothing("degraded walk", &degraded, None);

    let conditions_fail = FhirFixture::start(|target| {
        if target.starts_with("/fhir/Patient/") {
            json_reply(200, patient())
        } else {
            json_reply(
                500,
                json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error", "diagnostics": format!("Patient/{ID} failed")}]}),
            )
        }
    });
    assert_trace_run_leaks_nothing("conditions 500", &conditions_fail, None);

    let server_error = FhirFixture::start(|_| {
        json_reply(
            500,
            json!({"resourceType": "OperationOutcome", "issue": [{"severity": "fatal", "diagnostics": format!("Patient/{ID} failed")}]}),
        )
    });
    assert_trace_run_leaks_nothing(
        "patient 500",
        &server_error,
        Some("the FHIR server answered HTTP 500"),
    );

    let not_found = FhirFixture::start(|_| {
        json_reply(
            404,
            json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error", "diagnostics": format!("Patient/{ID} is not known")}]}),
        )
    });
    assert_trace_run_leaks_nothing(
        "not found",
        &not_found,
        Some("the FHIR server has no record with that patient ID"),
    );

    let other_base = other.base.clone();
    let moved = FhirFixture::start(move |target| redirect(format!("{other_base}{target}")));
    assert_trace_run_leaks_nothing(
        "patient redirect",
        &moved,
        Some("the FHIR server redirected off the configured base, so the request stopped"),
    );

    assert_eq!(other.requests(), 0, "a redirect left the configured origin");
}
