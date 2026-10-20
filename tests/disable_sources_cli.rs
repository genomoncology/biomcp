//! Outside-in proof for `--disable-sources` / `BIOMCP_DISABLE_SOURCES`:
//! each test drives the built binary against one local
//! fixture server that replays recorded reply shapes and logs every
//! request, so "makes no request to the disabled source" is asserted
//! from the provider's side of the wire.

use biomcp_mcp_contract_client::test_http::TestHttpServer;
use std::process::{Command, Output};
use std::sync::{Arc, Mutex};

struct Fixture {
    base: String,
    log: Arc<Mutex<Vec<String>>>,
    server: TestHttpServer,
}

impl Fixture {
    fn start() -> Fixture {
        let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let server_log = log.clone();
        let server = TestHttpServer::respond(move |request| {
            let response = fixture_response(&request, &server_log);
            async move { Ok(response) }
        })
        .expect("start disabled-source fixture");
        let base = server.base.clone();
        Fixture { base, log, server }
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

fn fixture_response(request: &[u8], log: &Arc<Mutex<Vec<String>>>) -> Vec<u8> {
    let request = String::from_utf8_lossy(request);
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
    let mut response = response.into_bytes();
    response.extend_from_slice(body);
    response
}

/// One recorded reply per provider path, shaped like the captured
/// fixtures under `testdata/sources/`.
fn route(path: &str, query: &str) -> (&'static str, &'static str, &'static [u8]) {
    match path {
        "/fda-http/OOPD_Results.cfm" => ("503 Service Unavailable", "text/plain", b"unavailable"),
        "/fda-invalid/OOPD_Results.cfm" => ("200 OK", "text/html", b"not a designation table"),
        "/fda-valid/OOPD_Results.cfm" => ("200 OK", "text/html", include_bytes!("../testdata/sources/fda_orphan/provider-shaped.html")),
        "/fda-redirect/OOPD_Results.cfm" => ("302 Found", "text/plain", b""),
        "/gwas/rest/api/singleNucleotidePolymorphisms/rs7903146" => ("410 Gone", "text/plain", include_bytes!("../testdata/sources/gwas/associations_retired_410_body.txt")),
        "/gwas/rest/api/v2/associations" => ("200 OK", "application/json", include_bytes!("../testdata/sources/gwas/associations_gene_tcf7l2_v2_20260812.json")),
        "/europe/search" => ("200 OK", "application/json", br#"{"hitCount":0,"resultList":{"result":[]}}"#),
        "/ols4/api/search" => (
            "200 OK", "application/json",
            br#"{"response":{"numFound":0,"start":0,"docs":[]}}"#,
        ),
        path if path.starts_with("/myvariant/v1/variant/") => (
            "200 OK", "application/json",
            include_bytes!("../testdata/sources/myvariant/get_braf_v600e.json"),
        ),
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
            .envs(biomcp_mcp_contract_client::child_cache_env(
                self.cache.path(),
            ))
            .env("BIOMCP_MYGENE_BASE", format!("{base}/mygene/v3"))
            .env("BIOMCP_OLS4_BASE", format!("{base}/ols4"))
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
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn disable_kegg_env_and_flag_forms_agree_with_comma_and_pipe() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn commercial_restricted_group_disables_kegg_and_enrichr() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn commercial_groups_disable_orcid_records_without_requests() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture)
        .env("BIOMCP_HEALTH_PROBE_BASE", &fixture.base)
        .env("BIOMCP_ORCID_BASE", &format!("{}/orcid", fixture.base))
        .env("ORCID_ACCESS_TOKEN", "fixture-public-read-token");
    for group in ["commercial-restricted", "non-commercial"] {
        let health = binary.run(&[
            "--json",
            "--disable-sources",
            group,
            "health",
            "--api",
            "ORCID",
        ]);
        assert!(health.status.success(), "{health:?}");
        let value: serde_json::Value = serde_json::from_slice(&health.stdout).unwrap();
        assert_eq!(value["rows"][0]["status"], "disabled", "{value}");
        let reason = format!("disabled by the {group} group");
        assert!(
            value["rows"][0]["disabled_reason"]
                .as_str()
                .unwrap()
                .contains(&reason),
            "{value}"
        );
        assert!(
            value["disabled"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["id"] == "orcid"),
            "{value}"
        );
        for args in [
            vec![
                "--json",
                "--disable-sources",
                group,
                "get",
                "author",
                "orcid:0000-0002-1825-0097",
            ],
            vec![
                "--json",
                "--disable-sources",
                group,
                "author",
                "papers",
                "orcid:0000-0002-1825-0097",
            ],
        ] {
            let output = binary.run(&args);
            assert!(!output.status.success(), "{output:?}");
            let message = format!("{}{}", stdout(&output), stderr(&output));
            assert!(message.contains(&reason), "{message}");
        }
    }
    assert_eq!(
        fixture.requests_matching("/"),
        0,
        "disabled ORCID calls reached the fixture"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn biomcp_disable_kegg_alias_still_disables_kegg() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn kegg_disable_drops_kegg_pathway_names_carried_by_mygene() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn enrichr_disabled_makes_no_enrichr_request_but_enabled_does() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn unknown_name_fails_with_the_valid_names() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn health_lists_disabled_sources_with_the_reason() {
    let mut fixture = Fixture::start();
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
    fixture.server.finish().unwrap();
}

#[test]
fn ddinter_sync_refuses_when_the_non_commercial_group_is_off() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "non-commercial");

    let output = binary.run(&["ddinter", "sync"]);
    assert!(!output.status.success(), "{output:?}");
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("DDInter is disabled by the non-commercial group"),
        "{combined}"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn oncokb_lookup_refuses_when_disabled_by_group() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "license-required");

    let output = binary.run(&["variant", "oncokb", "BRAF V600E"]);
    assert!(!output.status.success(), "{output:?}");
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(combined.contains("OncoKB is disabled"), "{combined}");
    assert!(
        combined.contains("disabled by the license-required group"),
        "{combined}"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn mcp_and_cli_read_the_same_env_setting() {
    // Both surfaces read BIOMCP_DISABLE_SOURCES; the CLI check above and
    // the rmcp client contract test cover each transport. This test pins
    // the CLI side of the parity for the gene carried-field case through
    // the environment variable alone.
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_DISABLE_SOURCES", "kegg");

    let output = binary.run(&["--json", "get", "gene", "BRAF", "pathways"]);
    assert!(output.status.success(), "{output:?}");
    let out = stdout(&output);
    assert!(!out.contains("Homo sapiens"), "{out}");
    fixture.server.finish().unwrap();
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
        env.push(("BIOMCP_TEST_UNPACED_ORIGIN".to_string(), origin.clone()));
        ReplayServer {
            _scratch: scratch,
            env,
            request_log,
            child,
        }
    }

    fn test_env(&self) -> (tempfile::TempDir, Vec<(String, String)>) {
        let cache = tempfile::tempdir().expect("isolated replay test cache");
        let mut env = self.env.clone();
        env.push(("BIOMCP_CACHE_MODE".to_string(), "off".to_string()));
        env.extend(
            biomcp_mcp_contract_client::child_cache_env(cache.path())
                .map(|(key, value)| (key.to_string(), value.to_string_lossy().into_owned())),
        );
        (cache, env)
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

static SHARED_REPLAY: std::sync::OnceLock<std::sync::Mutex<Option<ReplayServer>>> =
    std::sync::OnceLock::new();

extern "C" fn stop_shared_replay() {
    if let Some(server) = SHARED_REPLAY.get() {
        let mut server = server
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        drop(server.take());
    }
}

fn shared_replay() -> std::sync::MutexGuard<'static, Option<ReplayServer>> {
    let server = SHARED_REPLAY.get_or_init(|| {
        // Static values do not drop at process exit. The runner owns normal
        // teardown here; the server's parent guard covers an abandoned runner.
        assert_eq!(unsafe { libc::atexit(stop_shared_replay) }, 0);
        std::sync::Mutex::new(Some(ReplayServer::start()))
    });
    server
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn serve_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for the serve proofs")
}

#[test]
fn serve_flag_disables_civic_on_the_stdio_server() {
    let replay_guard = shared_replay();
    let replay = replay_guard.as_ref().expect("runner-owned replay server");
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let (_cache, env) = replay.test_env();
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
fn serve_http_startup_failure_reports_child_exit_and_stderr() {
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let error = serve_runtime().block_on(async {
        harness
            .spawn_http_server_with_args(&["--not-a-valid-server-flag"], &[])
            .await
            .expect_err("the invalid flag must prevent server startup")
    });
    let message = error.to_string();
    assert!(message.contains("exited before healthcheck"), "{message}");
    assert!(message.contains("last probe:"), "{message}");
    assert!(message.contains("stderr:"), "{message}");
    assert!(message.contains("unexpected argument"), "{message}");
    assert!(message.contains("--not-a-valid-server-flag"), "{message}");
}

#[cfg(unix)]
#[test]
fn serve_http_waits_for_delayed_startup_and_preserves_exit_diagnostics() {
    let repo = env!("CARGO_MANIFEST_DIR");
    let binary = biomcp_mcp_contract_client::ContractHarness::from_repo_root(repo).biomcp_bin;
    let harness = biomcp_mcp_contract_client::ContractHarness::new("/bin/sh", repo);
    let binary = binary.to_str().expect("CLI path is UTF-8");
    serve_runtime().block_on(async {
        // Delay the real CLI before argument parsing, matching a live child
        // that has not yet produced either health responses or stderr.
        let prefix = ["-c", "sleep 6; exec \"$@\"", "delayed-cli", binary];
        let (mut child, _) = harness
            .spawn_http_server_with_args(&prefix, &[])
            .await
            .expect("a slow live child can still become healthy");
        child.kill().await.expect("reap the healthy server");
        let mut invalid = prefix.to_vec();
        invalid.push("--not-a-valid-server-flag");
        let error = harness
            .spawn_http_server_with_args(&invalid, &[])
            .await
            .expect_err("wait for the slow child's real argument error");
        let message = error.to_string();
        assert!(message.contains("exited before healthcheck"), "{message}");
        assert!(message.contains("unexpected argument"), "{message}");
        assert!(message.contains("--not-a-valid-server-flag"), "{message}");
    });
}

#[test]
fn serve_http_flag_matches_the_env_variable_form() {
    let replay_guard = shared_replay();
    let replay = replay_guard.as_ref().expect("runner-owned replay server");
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let (_cache, env) = replay.test_env();
    let rt = serve_runtime();
    rt.block_on(async move {
        let flag_instructions =
            serve_http_civic_description(&harness, replay, &env, &["--disable-sources", "civic"])
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
        let env_instructions = serve_http_civic_description(&harness, replay, &env_form, &[])
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

#[test]
fn disabled_health_aliases_never_probe_and_name_the_setting() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture).env("BIOMCP_HEALTH_PROBE_BASE", &fixture.base);
    for (id, api) in [
        ("mygene-info", "MyGene"),
        ("myvariant-info", "MyVariant"),
        ("mychem-info", "MyChem"),
        ("mydisease-info", "MyDisease"),
        ("monarch-initiative", "Monarch"),
        ("hpo-jax-api", "HPO"),
        ("human-protein-atlas", "HPA"),
        ("fda-orphan-designations", "FDA Orphan Drug Designations"),
        ("orcid", "ORCID"),
    ] {
        let output = binary.run(&["--json", "--disable-sources", id, "health", "--api", api]);
        assert!(output.status.success(), "{id}: {output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["rows"][0]["status"], "disabled", "{id}: {value}");
        assert!(
            value["rows"][0]["disabled_reason"]
                .as_str()
                .unwrap()
                .contains("--disable-sources=")
        );
    }
    assert_eq!(
        fixture.requests_matching("/"),
        0,
        "disabled health probes reached the network"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn disabled_variant_sections_drop_carried_data_and_report_disabled_without_retry() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture);
    let disabled = "clinvar,civic,gnomad";
    for section in ["clinvar", "civic", "population"] {
        let output = binary.run(&[
            "--json",
            "--disable-sources",
            disabled,
            "get",
            "variant",
            "chr7:g.140453136A>T",
            section,
        ]);
        assert!(output.status.success(), "{section}: {output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            value["section_outcomes"][section]["outcome"], "disabled",
            "{value}"
        );
        assert!(
            value["section_outcomes"][section]["message"]
                .as_str()
                .unwrap()
                .contains("disabled by --disable-sources="),
            "{value}"
        );
        assert!(value.get("civic").is_none(), "{value}");
        assert!(value.get("clinvar").is_none(), "{value}");
        assert!(value.get("clinvar_id").is_none(), "{value}");
        let text = binary.run(&[
            "--disable-sources",
            disabled,
            "get",
            "variant",
            "chr7:g.140453136A>T",
            section,
        ]);
        let text = stdout(&text);
        assert!(text.contains("disabled by --disable-sources="), "{text}");
        assert!(!text.contains("Retry:"), "{text}");
        assert!(!text.contains("request failed"), "{text}");
    }
    fixture.server.finish().unwrap();
}

#[test]
fn restricted_group_includes_recorded_cgi_terms() {
    let mut fixture = Fixture::start();
    let output = Binary::new(&fixture).run(&[
        "--json",
        "--disable-sources",
        "commercial-restricted",
        "health",
        "--api",
        "kegg",
    ]);
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        value["disabled"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "cancer-genome-interpreter"),
        "{value}"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn disabled_article_sources_are_skipped_without_warnings_or_incompleteness() {
    let mut fixture = Fixture::start();
    let binary =
        Binary::new(&fixture).env("BIOMCP_EUROPEPMC_BASE", &format!("{}/europe", fixture.base));
    let output = binary.run(&[
        "--json",
        "--disable-sources",
        "pubtator3,pubmed,semantic-scholar,litsense2",
        "search",
        "article",
        "melanoma",
    ]);
    assert!(output.status.success(), "{output:?}");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["complete"], true, "{value}");
    let statuses = value["_meta"]["source_status"]
        .as_array()
        .expect("source statuses");
    for source in ["pubtator", "semanticscholar"] {
        let status = statuses
            .iter()
            .find(|status| status["source"] == source)
            .expect("disabled source reported");
        assert_eq!(status["enabled"], false, "{value}");
        assert_eq!(status["status"], "skipped", "{value}");
        assert!(
            status["message"]
                .as_str()
                .unwrap()
                .contains("disabled by --disable-sources="),
            "{value}"
        );
    }
    assert!(!stderr(&output).contains("WARN"), "{}", stderr(&output));
    assert!(fixture.requests_matching("/europe/search") > 0);
    assert!(fixture.requests_matching("/ols4/api/search") > 0);
    let failed = Binary::new(&fixture)
        .env(
            "BIOMCP_EUROPEPMC_BASE",
            &format!("{}/missing", fixture.base),
        )
        .run(&[
            "--json",
            "--disable-sources",
            "pubtator3,pubmed,semantic-scholar,litsense2",
            "search",
            "article",
            "melanoma",
        ]);
    assert!(!failed.status.success(), "{failed:?}");
    assert!(stderr(&failed).contains("WARN"), "{}", stderr(&failed));
    fixture.server.finish().unwrap();
}

#[test]
fn mcp_override_refusal_names_the_original_flag_setting() {
    let cache = tempfile::tempdir().unwrap();
    let harness =
        biomcp_mcp_contract_client::ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    serve_runtime().block_on(async {
        let env = vec![
            ("BIOMCP_CACHE_MODE".to_string(), "off".to_string()),
            ("BIOMCP_DISABLE_SOURCES".to_string(), String::new()),
            ("BIOMCP_DISABLE_KEGG".to_string(), "0".to_string()),
        ]
        .into_iter()
        .chain(
            biomcp_mcp_contract_client::child_cache_env(cache.path())
                .map(|(key, value)| (key.to_string(), value.to_string_lossy().into_owned())),
        )
        .collect::<Vec<_>>();
        let (client, _) = harness
            .spawn_stdio_client_with_args(&["--disable-sources", "civic"], &env)
            .await
            .unwrap();
        let result = biomcp_mcp_contract_client::call_biomcp(
            &client,
            "biomcp --disable-sources kegg health --api kegg",
        )
        .await
        .unwrap();
        let text = biomcp_mcp_contract_client::first_text(&result.content);
        assert_eq!(result.is_error, Some(true), "{text}");
        assert!(
            text.contains("disabled by --disable-sources=civic"),
            "{text}"
        );
        assert!(!text.contains("by BIOMCP_DISABLE_SOURCES"), "{text}");
        client.cancel().await.unwrap();
    });
}

#[test]
fn variant_miss_keeps_assembly_outside_the_quoted_identifier() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture).env(
        "BIOMCP_MYVARIANT_BASE",
        &format!("{}/missing", fixture.base),
    );
    for json in [false, true] {
        let mut args = vec![
            "--no-cache",
            "get",
            "variant",
            "--assembly",
            "hg38",
            "chr7:g.140453136A>T",
        ];
        if json {
            args.insert(0, "--json");
        }
        let output = binary.run(&args);
        let message = if json {
            let value: serde_json::Value =
                serde_json::from_slice(&output.stdout).expect("error JSON");
            assert_eq!(value["error"]["code"], "not_found", "{value}");
            value["error"]["message"].as_str().unwrap().to_string()
        } else {
            stderr(&output)
        };
        eprintln!("{args:?}: exit {:?}; {message}", output.status.code());
        assert_eq!(output.status.code(), Some(1));
        assert!(
            message.contains("variant 'chr7:g.140453136A>T' not found."),
            "{message}"
        );
        assert!(message.contains("attempted GRCh38"), "{message}");
        assert!(message.contains("upstream HTTP 404"), "{message}");
        assert!(
            message.contains("biomcp search variant \"chr7:g.140453136A>T\""),
            "{message}"
        );
    }
    fixture.server.finish().unwrap();
}

#[test]
fn fda_nonredirect_health_failures_explain_the_observed_reason() {
    let mut fixture = Fixture::start();
    let mut failures = Vec::new();
    for (base, expected, code) in [
        ("fda-http", "HTTP 503", 1),
        ("fda-invalid", "invalid designation response", 1),
        (
            "fda-redirect",
            "HTTP 302, blocked: the FDA site redirects scripted clients",
            1,
        ),
        ("fda-valid", "ms", 0),
    ] {
        let output = Binary::new(&fixture)
            .env(
                "BIOMCP_FDA_ORPHAN_BASE",
                &format!("{}/{base}", fixture.base),
            )
            .run(&[
                "--json",
                "health",
                "--api",
                "FDA Orphan Drug Designations",
                "--fail-on-error",
            ]);
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("health JSON");
        eprintln!("FDA {base}: exit {:?}; {value}", output.status.code());
        if output.status.code() != Some(code) || !value.to_string().contains(expected) {
            failures.push(format!("{base}: {value}"));
        }
    }
    assert!(
        failures.is_empty(),
        "FDA must name the observed failure: {failures:?}"
    );
    fixture.server.finish().unwrap();
}

#[test]
fn gwas_health_checks_the_supported_association_search() {
    let mut fixture = Fixture::start();
    let binary = Binary::new(&fixture)
        .env("BIOMCP_HEALTH_PROBE_BASE", &fixture.base)
        .env(
            "BIOMCP_GWAS_BASE",
            &format!("{}/gwas/rest/api", fixture.base),
        );
    let search = binary.run(&[
        "--json",
        "--no-cache",
        "search",
        "gwas",
        "-g",
        "TCF7L2",
        "--limit",
        "1",
    ]);
    eprintln!(
        "GWAS search: exit {:?}; {}",
        search.status.code(),
        stdout(&search)
    );
    assert!(search.status.success(), "{}", stderr(&search));
    let health = binary.run(&[
        "--json",
        "health",
        "--api",
        "GWAS Catalog",
        "--fail-on-error",
    ]);
    eprintln!(
        "GWAS health: exit {:?}; {}",
        health.status.code(),
        stdout(&health)
    );
    assert!(health.status.success(), "{}", stdout(&health));
    assert_eq!(
        fixture.requests_matching("singleNucleotidePolymorphisms"),
        0
    );
    assert_eq!(fixture.requests_matching("v2/associations"), 2);
    fixture.server.finish().unwrap();
}
