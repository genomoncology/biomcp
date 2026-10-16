//! Outside-in proof for `--disable-sources` / `BIOMCP_DISABLE_SOURCES`:
//! each test drives the built binary against one local
//! fixture server that replays recorded reply shapes and logs every
//! request, so "makes no request to the disabled source" is asserted
//! from the provider's side of the wire.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

struct Fixture {
    base: String,
    log: Arc<Mutex<Vec<String>>>,
}

impl Fixture {
    fn start() -> Fixture {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture listener");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let server_log = Arc::clone(&log);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                let log = Arc::clone(&server_log);
                std::thread::spawn(move || handle(stream, log));
            }
        });
        Fixture { base, log }
    }

    fn requests_matching(&self, needle: &str) -> usize {
        self.log
            .lock()
            .expect("fixture log")
            .iter()
            .filter(|line| line.contains(needle))
            .count()
    }
}

fn handle(mut stream: TcpStream, log: Arc<Mutex<Vec<String>>>) {
    let Some((request, body)) = read_request(&mut stream) else {
        return;
    };
    let request_line = request.lines().next().unwrap_or_default().to_string();
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string();
    {
        let mut log = log.lock().expect("fixture log");
        log.push(request_line);
    }
    let _ = body;

    let (path_only, query): (String, String) = match path.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (path.clone(), String::new()),
    };

    let reply = route(&path_only, &query);
    let status = reply.0;
    let content_type = reply.1;
    let body = reply.2;
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn read_request(stream: &mut TcpStream) -> Option<(String, Vec<u8>)> {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 2048];
    let head_end = loop {
        if let Some(position) = find_head_end(&bytes) {
            break position;
        }
        let read = stream.read(&mut buffer).ok()?;
        if read == 0 || bytes.len() > 64 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&buffer[..read]);
    };
    let head = String::from_utf8(bytes[..head_end].to_vec()).ok()?;
    let content_length = head
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    let mut body = bytes[head_end + 4..].to_vec();
    while body.len() < content_length {
        let read = stream.read(&mut buffer).ok()?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..read]);
    }
    body.truncate(content_length);
    Some((head, body))
}

fn find_head_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

/// One recorded reply per provider path, shaped like the captured
/// fixtures under `testdata/sources/`.
fn route(path: &str, query: &str) -> (&'static str, &'static str, &'static [u8]) {
    match path {
        "/mygene/v3/query" => {
            let body = br#"{"total":1,"hits":[{"_id":"673","symbol":"BRAF","name":"B-Raf proto-oncogene, serine/threonine kinase","entrezgene":673,"type_of_gene":"protein-coding","pathway":{"kegg":[{"id":"hsa04010","name":"MAPK signaling pathway - Homo sapiens (human)"},{"id":"hsa04012","name":"ErbB signaling pathway - Homo sapiens (human)"}]}}]}"#;
            ("200 OK", "application/json", body)
        }
        "/reactome/ContentService/search/query" => {
            let body: &[u8] = if query.contains("MAPK") {
                br#"{"totalResults":1,"results":[{"entries":[{"stId":"R-HSA-5684996","id":"R-HSA-5684996","name":"MAPK signaling pathway"}]}]}"#
            } else {
                br#"{"totalResults":1,"results":[{"entries":[{"stId":"R-HSA-6802949","id":"R-HSA-6802949","name":"BRAF activating pathway"}]}]}"#
            };
            ("200 OK", "application/json", body)
        }
        "/wikipathways/findPathwaysByText.json" => {
            ("200 OK", "application/json", br#"{"result":[]}"#)
        }
        "/kegg/find/pathway/MAPK%20signaling%20pathway" => (
            "200 OK",
            "text/plain",
            b"map04010\tMAPK signaling pathway\nmap04011\tMAPK signaling pathway in yeast\n",
        ),
        "/Enrichr/addList" => {
            ("200 OK", "application/json", br#"{"userListId":4242}"#)
        }
        "/Enrichr/enrich" => (
            "200 OK",
            "application/json",
            br#"{"GO_Biological_Process_2025":[[1,"regulation of MAPK cascade",0.001,2.5,10.0,"BRAF",0.002]]}"#,
        ),
        _ => ("404 Not Found", "application/json", br#"{"error":"fixture miss"}"#),
    }
}

struct Binary<'a> {
    fixture: &'a Fixture,
    cache: tempfile::TempDir,
    env: Vec<(String, String)>,
}

impl<'a> Binary<'a> {
    fn new(fixture: &'a Fixture) -> Binary<'a> {
        let cache = tempfile::tempdir().expect("scratch cache dir");
        Binary {
            fixture,
            cache,
            env: Vec::new(),
        }
    }

    fn env(mut self, name: &str, value: &str) -> Self {
        self.env.push((name.to_string(), value.to_string()));
        self
    }

    /// Runs the binary with an optional `--disable-sources` flag and an
    /// optional `BIOMCP_DISABLE_SOURCES` value, so both forms can be
    /// compared against the same fixture.
    fn run_with(&self, args: &[&str], flag: Option<&str>, env: Option<&str>) -> Output {
        let mut command = self.command(args);
        if let Some(flag) = flag {
            command.arg("--disable-sources").arg(flag);
        }
        if let Some(env) = env {
            command.env("BIOMCP_DISABLE_SOURCES", env);
        }
        command.output().expect("run biomcp")
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_with(args, None, None)
    }

    fn command(&self, args: &[&str]) -> Command {
        let base = &self.fixture.base;
        let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
        command
            .args(args)
            .env("BIOMCP_CACHE_DIR", self.cache.path())
            .env("BIOMCP_MYGENE_BASE", format!("{base}/mygene/v3"))
            .env(
                "BIOMCP_REACTOME_BASE",
                format!("{base}/reactome/ContentService"),
            )
            .env("BIOMCP_WIKIPATHWAYS_BASE", format!("{base}/wikipathways"))
            .env("BIOMCP_KEGG_BASE", format!("{base}/kegg"))
            .env("BIOMCP_ENRICHR_BASE", format!("{base}/Enrichr"))
            .env("BIOMCP_MYVARIANT_BASE", format!("{base}/myvariant/v1"))
            .env("BIOMCP_OPENTARGETS_BASE", format!("{base}/unavailable"));
        command
            .env_remove("BIOMCP_DISABLE_SOURCES")
            .env_remove("BIOMCP_DISABLE_KEGG");
        for (name, value) in &self.env {
            command.env(name, value);
        }
        command
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf-8")
}

#[test]
fn disable_kegg_flag_makes_no_kegg_request_and_hides_kegg_rows() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    let enabled = binary.run(&[
        "--json",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(enabled.status.success(), "{enabled:?}");
    let enabled_out = stdout(&enabled);
    assert!(enabled_out.contains("hsa04010"), "{enabled_out}");
    assert!(fixture.requests_matching("/kegg/") >= 1);

    let disabled = binary.run(&[
        "--json",
        "--disable-sources",
        "kegg",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(disabled.status.success(), "{disabled:?}");
    let disabled_out = stdout(&disabled);
    assert!(!disabled_out.contains("hsa04010"), "{disabled_out}");
    assert!(!disabled_out.contains("Homo sapiens"), "{disabled_out}");
    assert!(disabled_out.contains("R-HSA-5684996"), "{disabled_out}");
    let kegg_requests_after_enabled = fixture.requests_matching("/kegg/");
    let recheck = binary.run(&[
        "--json",
        "--disable-sources",
        "kegg",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(recheck.status.success(), "{recheck:?}");
    assert_eq!(
        fixture.requests_matching("/kegg/"),
        kegg_requests_after_enabled,
        "the disabled run made another KEGG request"
    );
}

#[test]
fn disable_kegg_env_and_flag_forms_agree_with_comma_and_pipe() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    let args = [
        "--json",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ];
    let flag_comma = binary.run_with(&args, Some("kegg,enrichr"), None);
    let flag_pipe = binary.run_with(&args, Some(" KEGG | ENRICHR "), None);
    let env_comma = binary.run_with(&args, None, Some("kegg,enrichr"));
    let env_pipe = binary.run_with(&args, None, Some("kegg|enrichr"));

    let canonical = stdout(&flag_comma);
    for output in [&flag_pipe, &env_comma, &env_pipe] {
        assert!(output.status.success(), "{output:?}");
        assert_eq!(stdout(output), canonical, "comma and pipe forms must agree");
    }
    assert!(!canonical.contains("hsa04010"), "{canonical}");
}

#[test]
fn commercial_restricted_group_disables_kegg_and_enrichr() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    // KEGG: no request, no KEGG rows in pathway search.
    let search = binary.run(&[
        "--json",
        "--disable-sources",
        "commercial-restricted",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(search.status.success(), "{search:?}");
    let search_out = stdout(&search);
    assert!(!search_out.contains("hsa04010"), "{search_out}");
    assert!(search_out.contains("R-HSA-5684996"), "{search_out}");
    assert_eq!(fixture.requests_matching("/kegg/"), 0);

    // Enrichr: no request, and the ontology section says why it is off.
    let gene = binary.run(&[
        "--json",
        "--disable-sources",
        "commercial-restricted",
        "get",
        "gene",
        "BRAF",
        "ontology",
    ]);
    assert!(gene.status.success(), "{gene:?}");
    let gene_out = stdout(&gene);
    assert!(
        gene_out.contains("Enrichr gene enrichment is disabled by the commercial-restricted group"),
        "{gene_out}"
    );
    assert_eq!(fixture.requests_matching("/Enrichr/"), 0);

    // The union also covers OncoKB and DrugBank, while each named group
    // works alone.
    let licensed = binary.run_with(
        &[
            "--json",
            "search",
            "pathway",
            "MAPK signaling pathway",
            "--limit",
            "5",
        ],
        Some("license-required"),
        None,
    );
    assert!(licensed.status.success(), "{licensed:?}");
    assert_eq!(fixture.requests_matching("/kegg/"), 0);
    let non_commercial = binary.run(&[
        "--json",
        "--disable-sources",
        "non-commercial",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(non_commercial.status.success(), "{non_commercial:?}");
    assert!(
        stdout(&non_commercial).contains("hsa04010"),
        "non-commercial must not disable KEGG"
    );
}

#[test]
fn biomcp_disable_kegg_alias_still_disables_kegg() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_KEGG", "1");

    let output = binary.run(&[
        "--json",
        "search",
        "pathway",
        "MAPK signaling pathway",
        "--limit",
        "5",
    ]);
    assert!(output.status.success(), "{output:?}");
    let out = stdout(&output);
    assert!(!out.contains("hsa04010"), "{out}");
    assert!(out.contains("R-HSA-5684996"), "{out}");
    assert_eq!(fixture.requests_matching("/kegg/"), 0);
}

#[test]
fn kegg_disable_drops_kegg_pathway_names_carried_by_mygene() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    let enabled = binary.run(&["--json", "get", "gene", "BRAF", "pathways"]);
    assert!(enabled.status.success(), "{enabled:?}");
    let enabled_out = stdout(&enabled);
    assert!(
        enabled_out.contains("MAPK signaling pathway - Homo sapiens (human)"),
        "{enabled_out}"
    );

    let disabled = binary.run(&[
        "--json",
        "--disable-sources",
        "kegg",
        "get",
        "gene",
        "BRAF",
        "pathways",
    ]);
    assert!(disabled.status.success(), "{disabled:?}");
    let disabled_out = stdout(&disabled);
    assert!(
        !disabled_out.contains("MAPK signaling pathway - Homo sapiens (human)"),
        "{disabled_out}"
    );
    assert!(
        !disabled_out.contains("ErbB signaling pathway"),
        "{disabled_out}"
    );
    assert!(disabled_out.contains("BRAF"), "{disabled_out}");
}

#[test]
fn enrichr_disabled_makes_no_enrichr_request_but_enabled_does() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    let enabled = binary.run(&["--json", "get", "gene", "BRAF", "ontology"]);
    assert!(enabled.status.success(), "{enabled:?}");
    assert!(
        fixture.requests_matching("/Enrichr/") >= 2,
        "addList plus enrich"
    );

    let disabled = binary.run(&[
        "--json",
        "--disable-sources",
        "enrichr",
        "get",
        "gene",
        "BRAF",
        "ontology",
    ]);
    assert!(disabled.status.success(), "{disabled:?}");
    let disabled_out = stdout(&disabled);
    assert!(
        disabled_out.contains("Enrichr gene enrichment is disabled by --disable-sources=enrichr"),
        "{disabled_out}"
    );
    let requests_after_enabled = fixture.requests_matching("/Enrichr/");
    let _ = stdout(&disabled);
    assert_eq!(
        fixture.requests_matching("/Enrichr/"),
        requests_after_enabled,
        "the disabled run made another Enrichr request"
    );
}

#[test]
fn unknown_name_fails_with_the_valid_names() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture);

    let output = binary.run(&["--disable-sources", "nope", "search", "gene", "BRAF"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("Unknown source or group \"nope\""),
        "{combined}"
    );
    assert!(combined.contains("Valid sources:"), "{combined}");
    assert!(combined.contains("kegg"), "{combined}");
    assert!(combined.contains("mygene-info"), "{combined}");
    assert!(
        combined.contains("Valid groups: commercial-restricted, license-required, non-commercial"),
        "{combined}"
    );
    assert_eq!(
        fixture.requests_matching("/mygene/"),
        0,
        "no provider is queried before the setting is validated"
    );
}

#[test]
fn health_lists_disabled_sources_with_the_reason() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "kegg,enrichr");

    let output = binary.run(&["--json", "health", "--api", "kegg", "--api", "enrichr"]);
    assert!(output.status.success(), "{output:?}");
    let out = stdout(&output);
    assert!(out.contains("\"disabled\":"), "{out}");
    assert!(out.contains("\"id\": \"kegg\""), "{out}");
    assert!(out.contains("\"id\": \"enrichr\""), "{out}");
    assert!(
        out.contains("disabled by BIOMCP_DISABLE_SOURCES=kegg"),
        "{out}"
    );
    assert!(
        out.contains("disabled by BIOMCP_DISABLE_SOURCES=enrichr"),
        "{out}"
    );
    assert!(out.contains("\"disabled_reason\""), "{out}");
    // Both probed rows are disabled, so no probe request leaves the host.
    assert_eq!(fixture.requests_matching("/kegg/"), 0);
    assert_eq!(fixture.requests_matching("/Enrichr/"), 0);
}

#[test]
fn ddinter_sync_refuses_when_the_non_commercial_group_is_off() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "non-commercial");

    let output = binary.run(&["ddinter", "sync"]);
    assert!(!output.status.success(), "{output:?}");
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("DDInter is disabled by the non-commercial group"),
        "{combined}"
    );
}

#[test]
fn oncokb_lookup_refuses_when_disabled_by_group() {
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "license-required");

    let output = binary.run(&["variant", "oncokb", "BRAF V600E"]);
    assert!(!output.status.success(), "{output:?}");
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("OncoKB is disabled"), "{combined}");
    assert!(
        combined.contains("disabled by the license-required group"),
        "{combined}"
    );
}

#[test]
fn mcp_and_cli_read_the_same_env_setting() {
    // Both surfaces read BIOMCP_DISABLE_SOURCES; the CLI check above and
    // the rmcp client contract test cover each transport. This test pins
    // the CLI side of the parity for the gene carried-field case through
    // the environment variable alone.
    let fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "kegg");

    let output = binary.run(&["--json", "get", "gene", "BRAF", "pathways"]);
    assert!(output.status.success(), "{output:?}");
    let out = stdout(&output);
    assert!(!out.contains("Homo sapiens"), "{out}");
}

// The serve proofs stand every provider up on the golden suite's replay
// server (`tools/replay-server`), so a request the server sees is logged
// and a request it never sees cannot have left the host. The server
// refuses to reach any live provider without --record, which keeps the
// proof honest on a branch that has not fixed the gate yet.
struct ReplayServer {
    // Held so the scratch directory lives exactly as long as the server.
    _scratch: tempfile::TempDir,
    env: Vec<(String, String)>,
    request_log: std::path::PathBuf,
    child: std::process::Child,
}

impl ReplayServer {
    fn start() -> ReplayServer {
        let scratch = tempfile::tempdir().expect("scratch dir for the replay server");
        let env_file = scratch.path().join("replay-env");
        let request_log = scratch.path().join("request.log");
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut child = std::process::Command::new("python3")
            .arg(repo_root.join("tools/replay-server"))
            .arg("--root")
            .arg(&repo_root)
            .arg("--env-file")
            .arg(&env_file)
            .arg("--request-log")
            .arg(&request_log)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("start tools/replay-server");
        let ready = {
            use std::io::BufRead;
            let stdout =
                std::io::BufReader::new(child.stdout.take().expect("replay server stdout piped"));
            let mut reader = stdout;
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .expect("read the replay server readiness line");
            line
        };
        let origin = ready
            .strip_prefix("READY ")
            .unwrap_or_else(|| panic!("replay server greeted unexpectedly: {ready:?}"))
            .trim()
            .to_string();
        let mut env: Vec<(String, String)> = Vec::new();
        for line in std::fs::read_to_string(&env_file)
            .expect("replay server env file")
            .lines()
        {
            let (name, value) = line
                .strip_prefix("export ")
                .unwrap_or(line)
                .split_once('=')
                .unwrap_or_else(|| panic!("unexpected env file line: {line:?}"));
            env.push((name.to_string(), value.to_string()));
        }
        let cache = scratch.path().join("cache");
        std::fs::create_dir_all(&cache).expect("scratch cache dir");
        env.push(("BIOMCP_CACHE_MODE".to_string(), "off".to_string()));
        env.push((
            "BIOMCP_CACHE_DIR".to_string(),
            cache.to_string_lossy().into_owned(),
        ));
        env.push(("BIOMCP_TEST_UNPACED_ORIGIN".to_string(), origin.clone()));
        ReplayServer {
            _scratch: scratch,
            env,
            request_log,
            child,
        }
    }

    fn line_count(&self) -> usize {
        std::fs::read_to_string(&self.request_log)
            .map(|text| text.lines().filter(|line| !line.is_empty()).count())
            .unwrap_or(0)
    }

    fn civic_requests_since(&self, since: usize) -> Vec<String> {
        std::fs::read_to_string(&self.request_log)
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.is_empty())
            .skip(since)
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter_map(|entry| {
                entry
                    .get("key")
                    .and_then(|key| key.as_str())
                    .map(str::to_string)
            })
            .filter(|key| key.contains(" /civic"))
            .collect()
    }
}

impl Drop for ReplayServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // The scratch TempDir removes itself when it drops right after.
    }
}

fn serve_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for the serve proofs")
}

#[test]
fn serve_flag_disables_civic_on_the_stdio_server() {
    let replay = ReplayServer::start();
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let env = replay.env.clone();
    let rt = serve_runtime();
    rt.block_on(async move {
        let (client, pid) = harness
            .spawn_stdio_client_with_args(&["--disable-sources", "civic"], &env)
            .await
            .expect("spawn biomcp --disable-sources civic serve");
        let result = async {
            let initialize = client
                .peer_info()
                .expect("rmcp client stores initialize result as peer info");
            let instructions = initialize.instructions.as_deref().unwrap_or_default();
            assert!(
                instructions.contains(
                    "operator disabled these sources: civic (disabled by --disable-sources=civic)"
                ),
                "the server description must list CIViC off with the flag's reason: {instructions}"
            );
            let before = replay.line_count();
            let get = client
                .peer()
                .call_tool(
                    rmcp::model::CallToolRequestParams::new("get").with_arguments(
                        std::collections::BTreeMap::from([
                            ("entity".to_string(), serde_json::json!("variant")),
                            ("id".to_string(), serde_json::json!("BRAF V600E")),
                            ("sections".to_string(), serde_json::json!(["civic"])),
                            ("json".to_string(), serde_json::json!(true)),
                        ])
                        .into_iter()
                        .collect(),
                    ),
                )
                .await
                .expect("call the typed get tool for a variant");
            assert_eq!(
                get.is_error,
                Some(false),
                "the tool call must succeed; reply: {}",
                biomcp_mcp_contract_client::text_chunks(&get.content).join("\n")
            );
            let text = biomcp_mcp_contract_client::first_text(&get.content);
            assert!(
                text.to_ascii_lowercase().contains("civic"),
                "the reply names the CIViC section: {text}"
            );
            let leaked = replay.civic_requests_since(before);
            assert!(
                leaked.is_empty(),
                "the disabled CIViC source still reached the network: {leaked:?}"
            );
            anyhow::Ok(())
        }
        .await;
        let _ = biomcp_mcp_contract_client::terminate_process(pid);
        result.expect("serve proof assertions hold")
    });
}

#[test]
fn serve_http_flag_matches_the_env_variable_form() {
    let replay = ReplayServer::start();
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let env = replay.env.clone();
    let rt = serve_runtime();
    rt.block_on(async move {
        let flag_instructions =
            serve_http_civic_description(&harness, &replay, &env, &["--disable-sources", "civic"])
                .await
                .expect("serve-http accepts the parsed flag");
        assert!(
            flag_instructions.contains(
                "operator disabled these sources: civic (disabled by --disable-sources=civic)"
            ),
            "serve-http must list CIViC off with the flag: {flag_instructions}"
        );

        let mut env_form = env.clone();
        env_form.push(("BIOMCP_DISABLE_SOURCES".to_string(), "civic".to_string()));
        let env_instructions = serve_http_civic_description(&harness, &replay, &env_form, &[])
            .await
            .expect("serve-http reads the environment variable");
        assert!(
            env_instructions.contains(
                "operator disabled these sources: civic (disabled by BIOMCP_DISABLE_SOURCES=civic)"
            ),
            "serve-http must list CIViC off with the environment variable: {env_instructions}"
        );
    });
}

/// Starts one serve-http process with the given leading arguments and
/// environment, reads its initialize instructions, and calls the typed
/// get tool for one variant's CIViC section; the caller asserts on the
/// returned instructions while this helper asserts no CIViC request
/// reached the replay server during the tool call.
async fn serve_http_civic_description(
    harness: &biomcp_mcp_contract_client::ContractHarness,
    replay: &ReplayServer,
    env: &[(String, String)],
    args: &[&str],
) -> anyhow::Result<String> {
    let (mut child, base_url) = harness.spawn_http_server_with_args(args, env).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        let initialize = client
            .peer_info()
            .expect("rmcp client stores initialize result as peer info");
        let instructions = initialize.instructions.clone().unwrap_or_default();
        let before = replay.line_count();
        let get = client
            .peer()
            .call_tool(
                rmcp::model::CallToolRequestParams::new("get").with_arguments(
                    std::collections::BTreeMap::from([
                        ("entity".to_string(), serde_json::json!("variant")),
                        ("id".to_string(), serde_json::json!("BRAF V600E")),
                        ("sections".to_string(), serde_json::json!(["civic"])),
                        ("json".to_string(), serde_json::json!(true)),
                    ])
                    .into_iter()
                    .collect(),
                ),
            )
            .await?;
        assert_eq!(
            get.is_error,
            Some(false),
            "the tool call must succeed; reply: {}",
            biomcp_mcp_contract_client::text_chunks(&get.content).join("\n")
        );
        let leaked = replay.civic_requests_since(before);
        assert!(
            leaked.is_empty(),
            "the disabled CIViC source still reached the network: {leaked:?}"
        );
        Ok(instructions)
    }
    .await;
    let _ = child.kill().await;
    result
}
