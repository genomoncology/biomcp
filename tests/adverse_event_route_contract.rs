use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const EMPTY_OPENFDA_PAGE: &str =
    r#"{"meta":{"results":{"skip":0,"limit":5,"total":0}},"results":[]}"#;
const FAERS_REPORT_PAGE: &str = r#"{
  "meta":{"results":{"skip":0,"limit":1,"total":1}},
  "results":[{
    "safetyreportid":"1001",
    "serious":"1",
    "receivedate":"20250101",
    "seriousnesshospitalization":"1",
    "patient":{
      "reaction":[{"reactionmeddrapt":"Rash"}],
      "drug":[
        {"medicinalproduct":"DRUG NAME","drugcharacterization":"1","drugindication":"LUNG CANCER"},
        {"medicinalproduct":"OTHER DRUG","drugcharacterization":"2"}
      ]
    }
  }]
}"#;

struct RequestFixture {
    base: String,
    requests: mpsc::Receiver<String>,
    thread: Option<thread::JoinHandle<()>>,
}

impl RequestFixture {
    fn start() -> Self {
        Self::start_with_body(EMPTY_OPENFDA_PAGE)
    }

    fn start_with_body(body: &'static str) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind OpenFDA fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let (request_tx, requests) = mpsc::channel();
        let thread = thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut request = Vec::new();
                let mut chunk = [0_u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let read = stream.read(&mut chunk).expect("read fixture request");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                }
                let head = String::from_utf8_lossy(&request).into_owned();
                let target = head
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or_default()
                    .to_string();
                request_tx.send(target).expect("capture request target");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self {
            base,
            requests,
            thread: Some(thread),
        }
    }

    fn request_target(&self) -> String {
        self.requests
            .recv_timeout(Duration::from_secs(2))
            .expect("provider request")
    }

    fn assert_no_request(&self) {
        assert!(
            self.requests
                .recv_timeout(Duration::from_millis(250))
                .is_err(),
            "invalid command contacted OpenFDA"
        );
    }
}

impl Drop for RequestFixture {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            // Invalid-input tests leave accept blocked; connecting releases it without becoming
            // a BioMCP provider request.
            if !thread.is_finished() {
                let _ = std::net::TcpStream::connect(self.base.trim_start_matches("http://"));
            }
            thread.join().expect("join OpenFDA fixture");
        }
    }
}

fn run_biomcp(args: &[&str], fixture: &RequestFixture) -> Output {
    Command::new(env!("CARGO_BIN_EXE_biomcp"))
        .args(args)
        .env("BIOMCP_OPENFDA_BASE", &fixture.base)
        .env("NO_PROXY", "*")
        .env("no_proxy", "*")
        .output()
        .expect("run biomcp")
}

fn captured_search(target: &str) -> String {
    reqwest::Url::parse(&format!("http://fixture{target}"))
        .expect("request URL")
        .query_pairs()
        .find_map(|(key, value)| (key == "search").then(|| value.into_owned()))
        .expect("search query")
}

#[test]
fn invalid_count_offset_is_rejected_without_provider_contact() {
    let fixture = RequestFixture::start();
    let output = run_biomcp(
        &[
            "search",
            "adverse-event",
            "aspirin",
            "--source",
            "faers",
            "--count",
            "reaction",
            "--offset",
            "1",
        ],
        &fixture,
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--count requires --offset 0"));
    fixture.assert_no_request();
}

#[test]
fn unknown_get_section_is_rejected_without_provider_contact() {
    let fixture = RequestFixture::start();
    let output = run_biomcp(
        &["--json", "get", "adverse-event", "1001", "not-a-section"],
        &fixture,
    );

    assert_eq!(output.status.code(), Some(2));
    let error: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("structured JSON error");
    assert_eq!(error["error"]["code"], "invalid_argument");
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Unknown section"))
    );
    fixture.assert_no_request();
}

#[test]
fn faers_subset_json_process_contract_is_bounded_and_truthful() {
    let fixture = RequestFixture::start_with_body(FAERS_REPORT_PAGE);
    let output = run_biomcp(
        &[
            "--json",
            "--no-cache",
            "get",
            "adverse-event",
            "1001",
            "guidance",
            "reactions",
            "guidance",
            "outcomes",
            "reactions",
        ],
        &fixture,
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("subset JSON");
    assert_eq!(value["type"], "faers");
    assert_eq!(value["data"]["report_id"], "1001");
    assert_eq!(value["data"]["drug"], "drug name");
    assert_eq!(value["data"]["reactions"], serde_json::json!(["Rash"]));
    assert_eq!(
        value["data"]["outcomes"],
        serde_json::json!(["Hospitalization"])
    );
    assert!(value["data"].get("concomitant_medications").is_none());
    assert!(value["data"].get("patient").is_none());
    assert!(value["data"].get("serious").is_none());
    assert_eq!(
        value["_meta"]["next_commands"],
        serde_json::json!([
            "biomcp drug adverse-events \"drug name\"",
            "biomcp search drug -q \"drug name\"",
            "biomcp drug trials \"drug name\"",
            "biomcp search disease --query \"LUNG CANCER\""
        ])
    );
    assert_eq!(
        value["_meta"]["next_commands"]
            .as_array()
            .expect("guidance commands")
            .len(),
        4,
        "duplicate sections must remain idempotent"
    );
    assert_eq!(
        value["_meta"]["section_sources"]
            .as_array()
            .expect("section sources")
            .iter()
            .map(|source| source["key"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["reactions", "outcomes"]
    );
    assert!(fixture.request_target().contains("drug/event.json"));
}

#[test]
fn unsectioned_and_all_faers_json_keep_the_same_full_contract() {
    fn get_json(sections: &[&str]) -> serde_json::Value {
        let fixture = RequestFixture::start_with_body(FAERS_REPORT_PAGE);
        let mut args = vec!["--json", "--no-cache", "get", "adverse-event", "1001"];
        args.extend_from_slice(sections);
        let output = run_biomcp(&args, &fixture);
        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value = serde_json::from_slice(&output.stdout).expect("full JSON");
        assert!(fixture.request_target().contains("drug/event.json"));
        value
    }

    let unsectioned = get_json(&[]);
    let all = get_json(&["all"]);
    assert_eq!(all, unsectioned);
    assert_eq!(all["type"], "faers");
    for key in [
        "report_id",
        "drug",
        "reactions",
        "outcomes",
        "concomitant_medications",
        "indication",
        "serious",
        "date",
    ] {
        assert!(
            all["data"].get(key).is_some(),
            "missing full field {key}: {all}"
        );
    }
    assert!(
        all["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| !commands.is_empty()),
        "full metadata changed: {all}"
    );
    assert!(
        all["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|sources| sources.iter().any(|source| source["key"] == "overview")),
        "full overview provenance changed: {all}"
    );
}

#[test]
fn combined_faers_filter_keeps_vaers_visibly_not_requested() {
    let fixture = RequestFixture::start();
    let output = run_biomcp(
        &[
            "--json",
            "--no-cache",
            "search",
            "adverse-event",
            "aspirin",
            "--source",
            "all",
            "--reaction",
            "rash",
            "--limit",
            "5",
        ],
        &fixture,
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("combined JSON");
    assert_eq!(value["vaers"]["status"], "unsupported_filters");
    assert_eq!(
        value["section_outcomes"]["vaers"]["outcome"],
        "not_requested"
    );
    assert!(captured_search(&fixture.request_target()).contains("reactionmeddrapt"));
}

#[test]
fn device_seriousness_requests_and_json_labels_are_exact() {
    let cases: &[(&[&str], &str, &str)] = &[
        (
            &["--serious"],
            "death_or_injury",
            "(event_type:\"Death\" OR event_type:\"Injury\")",
        ),
        (&["--serious", "death"], "death", "event_type:\"Death\""),
        (&["--serious", "injury"], "injury", "event_type:\"Injury\""),
    ];

    for (serious_args, label, predicate) in cases {
        let fixture = RequestFixture::start();
        let mut args = vec![
            "--json",
            "--no-cache",
            "search",
            "adverse-event",
            "--type",
            "device",
            "--device",
            "pump",
        ];
        args.extend_from_slice(serious_args);
        args.extend_from_slice(&["--limit", "5"]);
        let output = run_biomcp(&args, &fixture);

        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("device JSON");
        assert_eq!(value["query"], format!("device=pump, serious={label}"));
        let search = captured_search(&fixture.request_target());
        assert!(search.contains(predicate), "search={search}");
        if *label == "death" {
            assert!(!search.contains("event_type:\"Injury\""), "search={search}");
        }
        if *label == "injury" {
            assert!(!search.contains("event_type:\"Death\""), "search={search}");
        }
    }
}

#[test]
fn broad_device_markdown_names_death_or_injury() {
    let fixture = RequestFixture::start();
    let output = run_biomcp(
        &[
            "--no-cache",
            "search",
            "adverse-event",
            "--type",
            "device",
            "--device",
            "pump",
            "--serious",
            "any",
            "--limit",
            "5",
        ],
        &fixture,
    );

    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("serious=death_or_injury"));
    assert!(
        captured_search(&fixture.request_target())
            .contains("event_type:\"Death\" OR event_type:\"Injury\"")
    );
}
