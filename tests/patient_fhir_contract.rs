//! Patient record contract across the CLI, stdio MCP, and serve-http.
//!
//! Every FHIR response here is synthetic and comes from an in-test fixture.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use biomcp_mcp_contract_client::{ContractHarness, call_biomcp, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::json;

const ID: &str = "SYNTH-PT-7Q.42";
const REFUSAL: &str = "`serve-http` refuses them";
const NEEDS_FILTER: &str =
    "needs at least one of --gender, --born-after, --born-before, or --condition";
const SNOMED_DIABETES: &str = "http://snomed.info/sct|44054006";

/// The CapabilityStatement params the command looks for, in flag order.
const PATIENT_SEARCH_PARAMS: &[&str] = &["_id", "gender", "birthdate", "_has"];

fn capability(params: &[&str]) -> serde_json::Value {
    json!({
        "resourceType": "CapabilityStatement",
        "rest": [{
            "mode": "server",
            "resource": [{
                "type": "Patient",
                "searchParam": params.iter().map(|name| json!({"name": name})).collect::<Vec<_>>()
            }]
        }]
    })
}

/// A Patient search bundle with `total` and the given entry resources.
fn patient_bundle(entries: Vec<serde_json::Value>, total: Option<u64>) -> serde_json::Value {
    let mut bundle = json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": entries
            .into_iter()
            .map(|resource| json!({"resource": resource, "search": {"mode": "match"}}))
            .collect::<Vec<_>>()
    });
    if let Some(total) = total {
        bundle["total"] = json!(total);
    }
    bundle
}

fn search_patient(id: &str) -> serde_json::Value {
    json!({"resourceType": "Patient", "id": id, "gender": "female", "birthDate": "1970-01-01"})
}

type Route = dyn Fn(&str) -> (u16, Vec<(String, String)>, String) + Send + Sync;

/// A blocking HTTP/1.1 fixture that records each request target and head.
struct FhirFixture {
    base: String,
    targets: Arc<Mutex<Vec<String>>>,
    heads: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl FhirFixture {
    fn start(
        route: impl Fn(&str) -> (u16, Vec<(String, String)>, String) + Send + Sync + 'static,
    ) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind fixture");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let targets = Arc::new(Mutex::new(Vec::new()));
        let heads = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let route: Arc<Route> = Arc::new(route);
        let (thread_targets, thread_heads, thread_stop) =
            (Arc::clone(&targets), Arc::clone(&heads), Arc::clone(&stop));
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        if thread_stop.load(Ordering::SeqCst) {
                            return;
                        }
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
                        thread_heads.lock().expect("heads").push(text.to_string());
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
                    Err(_) => return,
                }
            }
        });
        Self {
            base,
            targets,
            heads,
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

    fn targets(&self) -> Vec<String> {
        self.targets.lock().expect("targets").clone()
    }

    /// The full request head (request line and headers) at `index`.
    fn head(&self, index: usize) -> String {
        self.heads.lock().expect("heads")[index].clone()
    }
}

impl Drop for FhirFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wake the blocking accept without polling or recording a request.
        let _ = TcpStream::connect(self.base.trim_start_matches("http://"));
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

/// A server with one patient and one page of conditions.
fn healthy_fixture() -> FhirFixture {
    FhirFixture::start(|target| {
        if target.starts_with("/fhir/Patient/") {
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
    assert!(first_text(&search.content).contains(NEEDS_FILTER));
    let requests = fixture.requests();

    let shell = call_biomcp(&client, "biomcp search patient").await?;
    assert!(first_text(&shell.content).contains(NEEDS_FILTER));
    client.cancel().await?;
    assert_eq!(requests, 2, "one Patient read and one Condition page");
    assert_eq!(fixture.requests(), 2, "search patient sent a request");
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
fn search_patient_without_a_filter_fails_and_sends_no_request() {
    let fixture = healthy_fixture();
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &["search", "patient"],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(NEEDS_FILTER));
    assert_eq!(fixture.requests(), 0);
}

/// A FHIR fixture that answers `metadata` with `cap` and every Patient
/// search with `bundle`.
fn search_fixture(cap: serde_json::Value, bundle: serde_json::Value) -> FhirFixture {
    FhirFixture::start(move |target| {
        if target.starts_with("/fhir/metadata") {
            json_reply(200, cap.clone())
        } else if target.starts_with("/fhir/Patient?") {
            json_reply(200, bundle.clone())
        } else {
            json_reply(
                404,
                json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error"}]}),
            )
        }
    })
}

/// A full patient that ignores `_elements` and carries fields the output
/// must never show.
fn full_patient(id: &str) -> serde_json::Value {
    json!({
        "resourceType": "Patient",
        "id": id,
        "gender": "female",
        "birthDate": "1970-01-01",
        "name": [{"family": "Synthetic", "given": ["Synthia"]}],
        "address": [{"city": "Synthville"}],
        "telecom": [{"value": "555-0100"}]
    })
}

const PATIENT_SEARCH_QUERY_PREFIX: &str = "/fhir/Patient?";
const ENCODED_CONDITION: &str =
    "_has%3ACondition%3Apatient%3Acode=http%3A%2F%2Fsnomed.info%2Fsct%7C44054006";
const ENCODED_ELEMENTS: &str = "_elements=id%2Cgender%2CbirthDate";

#[test]
fn search_sends_the_exact_query_string_with_strict_handling() {
    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![search_patient("SYNTH-PT-1")], None),
    );
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "search",
            "patient",
            "--gender",
            "female",
            "--born-after",
            "1950-01-01",
            "--condition",
            SNOMED_DIABETES,
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.targets(),
        [
            "/fhir/metadata",
            format!(
                "{PATIENT_SEARCH_QUERY_PREFIX}gender=female&birthdate=gt1950-01-01&{ENCODED_CONDITION}&{ENCODED_ELEMENTS}&_count=10"
            )
            .as_str()
        ]
    );
    let head = fixture.head(1);
    assert!(
        head.to_ascii_lowercase()
            .contains("prefer: handling=strict"),
        "search lacks strict handling: {head}"
    );

    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![search_patient("SYNTH-PT-1")], None),
    );
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "search",
            "patient",
            "--born-before",
            "1960-12-31",
            "--limit",
            "3",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.targets(),
        [
            "/fhir/metadata",
            format!(
                "{PATIENT_SEARCH_QUERY_PREFIX}birthdate=lt1960-12-31&{ENCODED_ELEMENTS}&_count=3"
            )
            .as_str()
        ]
    );

    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![], Some(7)),
    );
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "search",
            "patient",
            "--condition",
            SNOMED_DIABETES,
            "--count",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.targets(),
        [
            "/fhir/metadata",
            format!("{PATIENT_SEARCH_QUERY_PREFIX}{ENCODED_CONDITION}&_summary=count").as_str()
        ]
    );
}

#[test]
fn a_capability_without_a_needed_parameter_refuses_and_sends_no_search() {
    for (missing, flag, value) in [
        ("_has", "--condition", SNOMED_DIABETES),
        ("birthdate", "--born-after", "1950-01-01"),
        ("gender", "--gender", "female"),
    ] {
        let params: Vec<&str> = PATIENT_SEARCH_PARAMS
            .iter()
            .copied()
            .filter(|param| *param != missing)
            .collect();
        let fixture = search_fixture(capability(&params), patient_bundle(vec![], None));
        let output = cli(
            &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
            &["search", "patient", flag, value],
        );
        assert!(!output.status.success(), "missing {missing}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&format!("`{missing}`")), "{stderr}");
        // The metadata read happened; no Patient search followed it.
        assert_eq!(fixture.targets(), ["/fhir/metadata"]);
    }
}

#[test]
fn bad_filter_values_fail_and_send_no_request() {
    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![], None),
    );
    let cases: &[(&str, [&str; 2])] = &[
        ("upper-case gender", ["--gender", "F"]),
        ("comma gender", ["--gender", "male,female"]),
        ("bad month", ["--born-after", "1950-13-01"]),
        ("bad day", ["--born-after", "1950-02-30"]),
        ("date prefix", ["--born-after", "ge1950"]),
        ("comma dates", ["--born-before", "1950,1960"]),
        ("code only", ["--condition", "44054006"]),
        ("empty system", ["--condition", "|44054006"]),
        ("empty code", ["--condition", "http://snomed.info/sct|"]),
        ("comma code", ["--condition", "http://snomed.info/sct|1,2"]),
    ];
    for (name, [flag, value]) in cases {
        let output = cli(
            &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
            &["search", "patient", flag, value],
        );
        assert!(!output.status.success(), "{name}");
        assert!(
            !String::from_utf8_lossy(&output.stderr).is_empty(),
            "{name}: no error text"
        );
    }
    assert_eq!(fixture.requests(), 0);
}

#[test]
fn count_prints_the_server_reported_total_or_the_no_count_message() {
    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![search_patient("SYNTH-PT-1")], Some(42)),
    );
    for extra in [&[][..], &["--limit", "3"][..]] {
        let mut args = vec![
            "search",
            "patient",
            "--condition",
            SNOMED_DIABETES,
            "--count",
        ];
        args.extend_from_slice(extra);
        let output = cli(&[("BIOMCP_FHIR_BASE", fixture.fhir_base())], &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Server-reported patient count: 42"),
            "{stdout}"
        );
    }
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "--json",
            "search",
            "patient",
            "--condition",
            SNOMED_DIABETES,
            "--count",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("count json");
    assert_eq!(body["server_reported_count"], json!(42), "{stdout}");

    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![], None),
    );
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "search",
            "patient",
            "--condition",
            SNOMED_DIABETES,
            "--count",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("The server reported no count."), "{stdout}");
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "--json",
            "search",
            "patient",
            "--condition",
            SNOMED_DIABETES,
            "--count",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("no-count json");
    assert_eq!(body["server_reported_count"], serde_json::Value::Null);
}

#[test]
fn out_of_range_limits_fail_and_send_no_request() {
    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(vec![], None),
    );
    for limit in ["0", "51"] {
        let output = cli(
            &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
            &["search", "patient", "--gender", "female", "--limit", limit],
        );
        assert!(!output.status.success(), "{limit}");
    }
    assert_eq!(fixture.requests(), 0);
}

#[test]
fn a_bundle_that_ignores_elements_is_trimmed_to_three_fields_and_the_limit() {
    let entries = vec![
        full_patient("SYNTH-PT-A"),
        json!({
            "resourceType": "Condition",
            "id": "synthetic-condition-1",
            "code": {"text": "Synthetic condition one"}
        }),
        full_patient("a/b"),
        full_patient("SYNTH-PT-B"),
        full_patient("SYNTH-PT-C"),
    ];
    let fixture = search_fixture(
        capability(PATIENT_SEARCH_PARAMS),
        patient_bundle(entries, None),
    );
    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &["search", "patient", "--gender", "female", "--limit", "3"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SYNTH-PT-A"), "{stdout}");
    assert!(stdout.contains("biomcp get patient SYNTH-PT-A"), "{stdout}");
    for leak in [
        "Synthia",
        "Synthville",
        "555-0100",
        "Synthetic condition",
        "a/b",
        "SYNTH-PT-C",
    ] {
        assert!(!stdout.contains(leak), "text leaked {leak}: {stdout}");
    }
    let data_rows = stdout
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| ID"))
        .count();
    assert_eq!(data_rows, 3, "at most --limit entries: {stdout}");

    let output = cli(
        &[("BIOMCP_FHIR_BASE", fixture.fhir_base())],
        &[
            "--json", "search", "patient", "--gender", "female", "--limit", "3",
        ],
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let body: serde_json::Value = serde_json::from_str(&stdout).expect("search json");
    assert_eq!(body["count"], json!(3), "{stdout}");
    assert_eq!(body["results"].as_array().map(Vec::len), Some(3));
    for leak in ["name", "address", "telecom", "Condition", "a/b"] {
        assert!(!stdout.contains(leak), "JSON leaked {leak}: {stdout}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn serve_http_refuses_patient_search_with_filters() -> anyhow::Result<()> {
    assert_http_refuses(shell(format!(
        "biomcp search patient --gender female --condition \"{SNOMED_DIABETES}\" --count"
    )))
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn stdio_typed_search_with_a_filter_is_an_unknown_field_and_sends_no_request()
-> anyhow::Result<()> {
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
            "search",
            json!({"entity": "patient", "gender": "female"}),
        ))
        .await?;
    let text = first_text(&response.content).to_string();
    assert_eq!(response.is_error, Some(true), "{text}");
    assert!(
        text.contains("unknown patient search field: gender"),
        "{text}"
    );
    client.cancel().await?;
    assert_eq!(fixture.requests(), 0, "typed filter reached the CLI");
    Ok(())
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
