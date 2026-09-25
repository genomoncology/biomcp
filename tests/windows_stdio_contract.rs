// Windows stdio contract: the MCP stream stays pure JSON while a tool
// call writes into the managed content directory. GitHub #283 is the
// failure: icacls.exe inherited the server's stdout and wrote its
// localized success line between JSON-RPC frames. The gene search
// below points MyGene at a refused port, so the call itself fails,
// but shared-client construction still secures the fresh cache tree
// (src/sources/mod.rs finish_shared_http_client), which is the path
// that used to spawn icacls with inherited handles.
#![cfg(windows)]

use std::io::{BufRead, BufReader, Write};
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
        let mut child = Command::new(env!("CARGO_BIN_EXE_biomcp"))
            .arg("serve")
            .env("BIOMCP_CACHE_DIR", cache_dir)
            .env("BIOMCP_MYGENE_BASE", "http://127.0.0.1:9")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
        started.elapsed() < Duration::from_secs(120),
        "contract took unexpectedly long"
    );
}
