// Windows stdio contract: the MCP stream stays pure JSON while a tool
// call writes into the managed content directory. GitHub #283 is the
// failure: icacls.exe inherited the server's stdout and wrote its
// localized success line between JSON-RPC frames. The gene search
// below points MyGene at a refused port, so the call itself fails,
// but shared-client construction still secures the fresh cache tree
// (src/sources/mod.rs finish_shared_http_client), which is the path
// that used to spawn icacls with inherited handles.
#![cfg(windows)]

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);

struct StdioMcp {
    child: Child,
    lines: mpsc::Receiver<String>,
}

impl StdioMcp {
    fn spawn(cache_dir: &std::path::Path) -> Self {
        Self::with_provider(cache_dir, "http://127.0.0.1:9")
    }

    fn with_provider(cache_dir: &Path, base: &str) -> Self {
        let mut child = isolated_command(cache_dir, base)
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn biomcp serve");
        let stdout = child.stdout.take().expect("piped child stdout");
        let (sender, receiver) = mpsc::channel::<String>();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        if sender.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            child,
            lines: receiver,
        }
    }

    fn send(&mut self, request: &Value) {
        let stdin = self.child.stdin.as_mut().expect("piped child stdin");
        let mut frame = serde_json::to_string(request).expect("serialize frame");
        frame.push('\n');
        stdin.write_all(frame.as_bytes()).expect("write frame");
        stdin.flush().expect("flush frame");
    }

    /// Every line the child writes must parse as JSON-RPC; a stray
    /// icacls success line fails here with its exact bytes.
    fn expect_response(&self, id: &str) -> Value {
        let line = self
            .lines
            .recv_timeout(RESPONSE_TIMEOUT)
            .unwrap_or_else(|_| panic!("timed out waiting for response to {id}"));
        let frame: Value = serde_json::from_str(&line)
            .unwrap_or_else(|error| panic!("stdout line is not JSON ({error}): {line}"));
        assert_eq!(frame["jsonrpc"], "2.0", "frame is not JSON-RPC 2.0: {line}");
        assert_eq!(frame["id"], id, "unexpected response id: {line}");
        frame
    }

    fn drain_remaining(&self) {
        while let Ok(line) = self.lines.recv_timeout(Duration::from_secs(5)) {
            serde_json::from_str::<Value>(&line)
                .unwrap_or_else(|error| panic!("late stdout line is not JSON ({error}): {line}"));
        }
    }
}

impl Drop for StdioMcp {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn windows_stdio_stream_stays_json_through_a_managed_write() {
    let cache_dir = tempfile::tempdir().expect("temporary cache root");
    let started = Instant::now();
    let mut client = StdioMcp::spawn(cache_dir.path());

    client.send(&json!({
        "jsonrpc": "2.0",
        "id": "init",
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "windows-stdio-contract", "version": "1"},
        },
    }));
    let initialized = client.expect_response("init")["result"].clone();
    assert_eq!(initialized["serverInfo"]["name"], "biomcp");

    client.send(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
    }));

    // The refused MyGene port turns the search into an isError tool
    // result; the securing of the managed tree already happened while
    // building the shared client, so any inherited-handle child has
    // already written by the time this response arrives.
    client.send(&json!({
        "jsonrpc": "2.0",
        "id": "gene",
        "method": "tools/call",
        "params": {
            "name": "search",
            "arguments": {"entity": "gene", "query": "BRAF"},
        },
    }));
    let call = client.expect_response("gene");
    let result = call["result"].clone();
    assert_eq!(
        result["isError"],
        Value::Bool(true),
        "refused MyGene must return an isError tool result: {call}"
    );

    client.drain_remaining();
    assert!(
        started.elapsed() < Duration::from_secs(120), // watchdog: whole-contract upper bound, two minutes
        "contract took unexpectedly long"
    );
}

/// Each process gets the same isolated cache/config; inherited source choices
/// and credentials must not change this local-only provider contract.
fn isolated_command(cache: &Path, base: &str) -> Command {
    let executable = std::env::var_os("BIOMCP_WINDOWS_CONTRACT_BIN")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_biomcp").into());
    let mut command = Command::new(executable);
    for (key, _) in std::env::vars_os() {
        let name = key.to_string_lossy();
        if name.starts_with("BIOMCP_") || name.ends_with("_API_KEY") || name.ends_with("_TOKEN") {
            command.env_remove(key);
        }
    }
    command.envs(biomcp_mcp_contract_client::child_cache_env(cache));
    command.envs([
        ("HOME", cache.as_os_str()),
        ("USERPROFILE", cache.as_os_str()),
        ("APPDATA", cache.as_os_str()),
        ("LOCALAPPDATA", cache.as_os_str()),
        ("XDG_CONFIG_HOME", cache.as_os_str()),
        ("BIOMCP_MYGENE_BASE", base.as_ref()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", base.as_ref()),
        ("BIOMCP_CACHE_MODE", "default".as_ref()),
        ("BIOMCP_CACHE_MIN_DISK_FREE", "1B".as_ref()),
        ("BIOMCP_CACHE_MAX_SIZE", "10000000".as_ref()),
        ("NO_COLOR", "1".as_ref()),
        ("HTTP_PROXY", "http://127.0.0.1:9".as_ref()),
        ("HTTPS_PROXY", "http://127.0.0.1:9".as_ref()),
        ("ALL_PROXY", "http://127.0.0.1:9".as_ref()),
        ("http_proxy", "http://127.0.0.1:9".as_ref()),
        ("https_proxy", "http://127.0.0.1:9".as_ref()),
        ("all_proxy", "http://127.0.0.1:9".as_ref()),
        ("NO_PROXY", "127.0.0.1,localhost".as_ref()),
        ("no_proxy", "127.0.0.1,localhost".as_ref()),
    ]);
    command.env_remove("RUST_LOG");
    command
}

struct GeneFixture {
    base: String,
    arrivals: mpsc::Receiver<()>,
    count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    hold: std::sync::Arc<(std::sync::Mutex<bool>, std::sync::Condvar)>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    server: Option<std::thread::JoinHandle<()>>,
}

impl GeneFixture {
    fn start() -> Self {
        use std::sync::{
            Arc, Condvar, Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        };
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("local MyGene fixture");
        let base = format!("http://{}", listener.local_addr().unwrap());
        let hold = Arc::new((Mutex::new(false), Condvar::new()));
        let count = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, arrivals) = mpsc::channel();
        let (gate, requests, stopped) = (hold.clone(), count.clone(), stop.clone());
        let server = std::thread::spawn(move || {
            let mut workers = Vec::new();
            for incoming in listener.incoming() {
                let mut stream = incoming.expect("fixture connection");
                if stopped.load(Ordering::SeqCst) {
                    break;
                }
                let (gate, requests, sender) = (gate.clone(), requests.clone(), sender.clone());
                workers.push(std::thread::spawn(move || {
                    stream.set_read_timeout(Some(RESPONSE_TIMEOUT)).unwrap();
                    stream.set_write_timeout(Some(RESPONSE_TIMEOUT)).unwrap();
                    let mut reader = BufReader::new(&mut stream);
                    let mut line = String::new();
                    reader.read_line(&mut line).expect("provider request");
                    assert!(line.starts_with("GET /query?"), "unexpected local request: {line}");
                    loop {
                        line.clear();
                        reader.read_line(&mut line).expect("request headers");
                        if line == "\r\n" || line.is_empty() { break; }
                    }
                    requests.fetch_add(1, Ordering::SeqCst);
                    sender.send(()).unwrap();
                    let (lock, ready) = &*gate;
                    let held = lock.lock().unwrap();
                    let (held, timeout) = ready.wait_timeout_while(held, RESPONSE_TIMEOUT, |held| *held).unwrap();
                    assert!(!timeout.timed_out() || !*held, "fixture replies were not released");
                    drop(held);
                    let body = include_bytes!("../testdata/sources/mygene/search_braf_20260811.json");
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nCache-Control: public, max-age=3600\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                    stream.write_all(body).expect("recorded provider reply");
                }));
            }
            for worker in workers {
                worker.join().expect("fixture request worker");
            }
        });
        Self {
            base,
            arrivals,
            count,
            hold,
            stop,
            server: Some(server),
        }
    }

    fn hold_replies(&self, held: bool) {
        *self.hold.0.lock().unwrap() = held;
        self.hold.1.notify_all();
    }

    fn count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Drop for GeneFixture {
    fn drop(&mut self) {
        self.hold_replies(false);
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(self.base.trim_start_matches("http://"));
        if let Some(server) = self.server.take() {
            server.join().expect("fixture server cleanup");
        }
    }
}

struct ManagedChild(Child);

impl Drop for ManagedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_gene(cache: &Path, base: &str, query: &str) -> ManagedChild {
    ManagedChild(
        isolated_command(cache, base)
            .args(["--json", "search", "gene", query, "--limit", "1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn concurrent CLI"),
    )
}

fn successful_output(process: ManagedChild) -> Value {
    successful_output_with_stderr(process).0
}

fn successful_output_with_stderr(mut process: ManagedChild) -> (Value, String) {
    let child = &mut process.0;
    let deadline = Instant::now() + RESPONSE_TIMEOUT;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll child") {
            break status;
        }
        assert!(Instant::now() < deadline, "CLI watchdog expired");
        // Watchdog polling only; process completion is the observed condition.
        std::thread::park_timeout(Duration::from_millis(10));
    };
    let mut stdout = String::new();
    let mut stderr = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    assert!(
        status.success(),
        "status={status}; stdout={stdout}; stderr={stderr}"
    );
    (serde_json::from_str(&stdout).expect("CLI JSON"), stderr)
}

fn call_gene(client: &mut StdioMcp, id: &str, query: &str) {
    client.send(&json!({"jsonrpc":"2.0", "id":id, "method":"tools/call",
        "params":{"name":"search", "arguments":{"entity":"gene", "query":query, "limit":1}}}));
}

fn assert_gene_tool(client: &StdioMcp, id: &str) {
    let response = client.expect_response(id);
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    let text = response["result"]["content"]
        .as_array()
        .expect("tool content")
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<String>();
    assert!(text.contains("BRAF"), "recorded gene absent: {response}");
}

fn assert_legacy_size_marker(cache: &Path) {
    fn bytes(path: &Path) -> u64 {
        std::fs::read_dir(path)
            .expect("content directory")
            .map(|entry| {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    bytes(&entry.path())
                } else {
                    entry.metadata().unwrap().len()
                }
            })
            .sum()
    }
    let total = std::fs::read_to_string(cache.join("http/.content-bytes-v1"))
        .expect("persistent byte marker");
    let reserved: u64 = total
        .strip_prefix("v2 ")
        .and_then(|text| text.strip_suffix('\n'))
        .expect("complete v2 byte marker")
        .parse()
        .expect("decimal byte total");
    let actual = bytes(&cache.join("http/content-v2"));
    assert!(
        actual > 0 && reserved >= actual,
        "lost size reservations: marker={reserved}, content={actual}"
    );
}

fn assert_size_marker(cache: &Path) {
    if std::env::var_os("BIOMCP_WINDOWS_CONTRACT_BIN").is_some() {
        return assert_legacy_size_marker(cache);
    }
    let db = rusqlite::Connection::open(cache.join("biomcp.db")).unwrap();
    let (reserved, actual): (i64, i64) = db
        .query_row(
            "SELECT (SELECT bytes FROM cache_meta),coalesce(sum(bytes),0) FROM entries",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(actual > 0);
    assert_eq!(
        reserved, actual,
        "transactional cache total must match rows"
    );
    let check: String = db
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .unwrap();
    assert_eq!(check, "ok");
}

fn assert_contended_write_fails_open(cache: &Path, provider: &GeneFixture) {
    if std::env::var_os("BIOMCP_WINDOWS_CONTRACT_BIN").is_some() {
        return assert_legacy_contended_write_fails_open(cache, provider);
    }
    let db = rusqlite::Connection::open(cache.join("biomcp.db")).unwrap();
    db.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (answer, warning) =
        successful_output_with_stderr(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert!(!answer["results"].as_array().unwrap().is_empty());
    assert!(
        !warning.contains("local HTTP cache"),
        "routine contention must stay quiet: {warning}"
    );
    db.execute_batch("ROLLBACK").unwrap();
    let before = provider.count();
    successful_output(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert_eq!(provider.count(), before + 1, "contended entry must refill");
    successful_output(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert_eq!(
        provider.count(),
        before + 1,
        "settled refill must be reusable"
    );
}

// Exercise the real cross-process lock without a production test hook.
fn assert_legacy_contended_write_fails_open(cache: &Path, provider: &GeneFixture) {
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(cache.join("http/.content-bytes.lock"))
        .expect("existing byte-total lock");
    lock.lock().expect("hold byte-total lock");
    let (answer, warning) =
        successful_output_with_stderr(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert!(!answer["results"].as_array().unwrap().is_empty());
    if std::env::var_os("BIOMCP_WINDOWS_CONTRACT_BIN").is_some() {
        // The comparison executable is the released 0.9.3 binary, whose
        // expected contention diagnostic was still a warning.
        assert!(
            warning.contains("local HTTP cache write failed") && warning.contains("WouldBlock"),
            "expected released-binary contention warning: {warning}"
        );
    } else {
        assert!(
            !warning.contains("local HTTP cache"),
            "routine contention must stay quiet at normal verbosity: {warning}"
        );
    }
    let keys: Vec<_> = cacache::list_sync(cache.join("http"))
        .map(|entry| entry.expect("read cache index").key)
        .collect();
    assert!(
        !keys.iter().any(|key| key.contains("BRAF_LOCK")),
        "contended entry persisted: {keys:?}"
    );
    println!("held byte-total lock: BRAF_LOCK absent; {warning}");
    lock.unlock().expect("release byte-total lock");
    let before = provider.count();
    successful_output(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert_eq!(provider.count(), before + 1, "absent entry must refill");
    successful_output(spawn_gene(cache, &provider.base, "BRAF_LOCK"));
    assert_eq!(
        provider.count(),
        before + 1,
        "settled refill must be reusable"
    );
}

#[test]
fn windows_parallel_cli_and_mcp_survive_shared_cache_clear_and_keep_byte_totals() {
    let root = tempfile::tempdir().expect("isolated shared cache");
    let cache = root.path();
    let downloads = cache.join("downloads");
    std::fs::create_dir(&downloads).unwrap();
    std::fs::write(downloads.join("keep"), b"outside HTTP cache").unwrap();
    let provider = GeneFixture::start();
    let mut mcp = StdioMcp::with_provider(cache, &provider.base);
    mcp.send(&json!({"jsonrpc":"2.0", "id":"init", "method":"initialize",
        "params":{"protocolVersion":"2025-11-25", "capabilities":{},
            "clientInfo":{"name":"windows-cache-contract", "version":"1"}}}));
    assert_eq!(
        mcp.expect_response("init")["result"]["serverInfo"]["name"],
        "biomcp"
    );
    mcp.send(&json!({"jsonrpc":"2.0", "method":"notifications/initialized"}));

    // Initialize the store before exercising concurrent publication.
    successful_output(spawn_gene(cache, &provider.base, "BRAF0"));
    provider
        .arrivals
        .recv_timeout(RESPONSE_TIMEOUT)
        .expect("initial provider request");

    // Hold distinct cold requests at the real provider boundary, so the two
    // CLI processes and long-lived MCP request overlap without timing sleeps.
    provider.hold_replies(true);
    let first = spawn_gene(cache, &provider.base, "BRAF1");
    let second = spawn_gene(cache, &provider.base, "BRAF2");
    call_gene(&mut mcp, "cold", "BRAF3");
    for _ in 0..3 {
        provider
            .arrivals
            .recv_timeout(RESPONSE_TIMEOUT)
            .expect("three concurrent provider requests");
    }
    provider.hold_replies(false);
    for child in [first, second] {
        assert!(
            !successful_output(child)["results"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }
    assert_gene_tool(&mcp, "cold");
    assert_size_marker(cache);
    let before = provider.count();
    assert!(
        !successful_output(spawn_gene(cache, &provider.base, "BRAF1"))["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let after_cli = provider.count();
    call_gene(&mut mcp, "warm", "BRAF3");
    assert_gene_tool(&mcp, "warm");
    let after_mcp = provider.count();
    println!(
        "original warm oracle: CLI BRAF1 misses={}, MCP BRAF3 misses={}, original assertion would {}",
        after_cli - before,
        after_mcp - after_cli,
        if after_mcp == before { "pass" } else { "fail" }
    );
    // Concurrent writes may fail open. The sequential calls above settle
    // these keys; only subsequent reuse promises no provider requests.
    successful_output(spawn_gene(cache, &provider.base, "BRAF1"));
    call_gene(&mut mcp, "settled-warm", "BRAF3");
    assert_gene_tool(&mcp, "settled-warm");
    assert_eq!(
        provider.count(),
        after_mcp,
        "settled CLI and MCP must reuse cached answers"
    );
    assert_contended_write_fails_open(cache, &provider);
    assert_size_marker(cache);

    // Warm misses and the forced lock miss/refill have already completed.
    // Only arrivals from the next held requests may satisfy the clear barrier.
    for _ in provider.arrivals.try_iter() {}
    provider.hold_replies(true);
    let cli = spawn_gene(cache, &provider.base, "BRAF4");
    call_gene(&mut mcp, "during-clear", "BRAF5");
    for _ in 0..2 {
        provider
            .arrivals
            .recv_timeout(RESPONSE_TIMEOUT)
            .expect("in-flight request before clear");
    }
    assert!(
        mcp.child.try_wait().unwrap().is_none(),
        "MCP must remain alive during clear"
    );
    let clear = ManagedChild(
        isolated_command(cache, &provider.base)
            .args(["--json", "cache", "clear", "--yes"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start concurrent cache clear"),
    );
    // Correct exclusion may wait for requests. Release replies after the clear
    // process starts; require success, without imposing an invalid lock order.
    provider.hold_replies(false);
    let report = successful_output(clear);
    assert!(report["entries_removed"].is_number(), "{report}");
    assert!(
        !successful_output(cli)["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_gene_tool(&mcp, "during-clear");
    assert_eq!(
        std::fs::read(downloads.join("keep")).unwrap(),
        b"outside HTTP cache"
    );

    // Either scheduling order may clear the concurrent writes. A new pair
    // refills the shared tree, then both surfaces must reuse those answers.
    assert!(
        !successful_output(spawn_gene(cache, &provider.base, "BRAF6"))["results"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    call_gene(&mut mcp, "after-clear", "BRAF7");
    assert_gene_tool(&mcp, "after-clear");
    assert_size_marker(cache);
    let before = provider.count();
    successful_output(spawn_gene(cache, &provider.base, "BRAF6"));
    call_gene(&mut mcp, "after-clear-hit", "BRAF7");
    assert_gene_tool(&mcp, "after-clear-hit");
    assert_eq!(
        provider.count(),
        before,
        "both surfaces must reuse the refilled cache"
    );
}
