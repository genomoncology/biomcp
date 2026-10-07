"""The shipped helper owns output policy and refusal after shared admission."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest
import test_mcp_stdio_guidance as stdio

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))


@pytest.fixture
def transport(monkeypatch, tmp_path):
    state = {"body": b"{}", "status": 200, "first_failure": False, "no_change": False, "requests": []}

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            path = urlsplit(self.path)
            status = 200
            if path.path == "/annotate/mutations/byProteinChange":
                query = parse_qs(path.query)
                state["requests"].append((query, self.headers.get("Authorization")))
                status, body = state["status"], state["body"]
                if state["first_failure"] and len(state["requests"]) == 1:
                    status, body = 400, b"first spelling refused"
            else:
                assert path.path == "/query" or path.path.startswith("/variant/"), self.path
                hit = {
                    "_id": "chr7:g.140453136A>T",
                    "dbnsfp": {"genename": "BRAF", "hgvsp": "p.Val600Glu"},
                }
                if state["no_change"]:
                    hit = {"_id": "chr7:g.140453136A>T", "dbnsfp": {"genename": "BRAF"},
                           "dbsnp": {"rsid": "rs113488022"}}
                body = json.dumps(
                    {"total": 1, "hits": [hit]} if path.path == "/query" else hit
                ).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    for key, value in {
        "BIOMCP_MYVARIANT_BASE": base,
        "BIOMCP_ONCOKB_BASE": base,
        "BIOMCP_TEST_UNPACED_ORIGIN": base,
        "BIOMCP_CACHE_DIR": str(tmp_path),
        "BIOMCP_CACHE_MODE": "off",
        "ONCOKB_TOKEN": "synthetic-token",
        "RUST_LOG": "warn,reqwest_retry=error",
    }.items():
        monkeypatch.setenv(key, value)
    monkeypatch.setattr(stdio, "RELEASE_BIN", BINARY)
    try:
        yield state
    finally:
        server.shutdown()
        thread.join(timeout=5)
        server.server_close()


def cli(*arguments):
    assert BINARY.exists(), f"missing biomcp binary: {BINARY}"
    return subprocess.run(
        [BINARY, *arguments], cwd=ROOT, text=True, capture_output=True, timeout=30
    )


def attempts(state, expected):
    assert state["requests"] == [
        ({"hugoSymbol": ["BRAF"], "alteration": [value]}, "Bearer synthetic-token")
        for value in expected
    ]
    state["requests"].clear()


def test_oncokb_helper_preserves_json_markdown_and_raw_mcp(transport):
    # Seven unique rows, a duplicate and unsorted duplicate drugs own the cap.
    body = b'''{"oncogenic":" Oncogenic ","mutationEffect":{"knownEffect":" Gain-of-function "},
      "highestSensitiveLevel":"LEVEL_1","highestResistanceLevel":"LEVEL_R1","treatments":[
      {"level":"LEVEL_R2","drugs":[{"drugName":"G"}]},
      {"level":"LEVEL_3B","drugs":[{"drugName":"D"}]},
      {"level":"LEVEL_1","drugs":[{"drugName":"B"},{"drugName":"A"},{"drugName":"A"}],"cancerType":{"name":" Example "}},
      {"level":"LEVEL_R1","drugs":[{"drugName":"F"}]},
      {"level":"LEVEL_4","drugs":[{"drugName":"E"}]},
      {"level":"LEVEL_2","drugs":[{"drugName":"C"}]},
      {"level":"LEVEL_3A","drugs":[{"drugName":"CC"}]},
      {"level":"LEVEL_1","drugs":[{"drugName":"A"},{"drugName":"B"}],"cancerType":{"name":"Example"}}]}'''
    expected = {
        "gene": "BRAF", "alteration": "V600E", "oncogenic": "Oncogenic",
        "effect": "Gain-of-function", "level": "Level 1",
        "therapies": [
            {"level": "Level 1", "drugs": ["A", "B"], "cancer_type": "Example"},
            {"level": "Level 2", "drugs": ["C"]},
            {"level": "Level 3A", "drugs": ["CC"]},
            {"level": "Level 3B", "drugs": ["D"]},
            {"level": "Level 4", "drugs": ["E"]},
            {"level": "Level R1", "drugs": ["F"], "note": "(and 1 more)"},
        ],
    }
    markdown = """# OncoKB

Gene: BRAF
Alteration: V600E
Level: Level 1
Oncogenic: Oncogenic
Effect: Gain-of-function

## Therapies

| Drug | Level | Cancer Type | Note |
|------|-------|-------------|------|
| A + B | Level 1 | Example | - |
| C | Level 2 | - | - |
| CC | Level 3A | - | - |
| D | Level 3B | - | - |
| E | Level 4 | - | - |
| F | Level R1 | - | (and 1 more) |

[OncoKB](https://www.oncokb.org/gene/BRAF/V600E)
"""
    transport.update(body=body, first_failure=True)
    result = cli("--json", "variant", "oncokb", "BRAF V600E")
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == expected
    attempts(transport, ["V600E", "p.V600E"])
    transport["first_failure"] = False
    result = cli("variant", "oncokb", "BRAF V600E")
    assert result.returncode == 0, result.stderr
    assert result.stdout.rstrip("\n") == markdown.rstrip("\n")
    attempts(transport, ["V600E"])

    client = stdio.StdioMcp()
    try:
        client.call({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-03-26", "capabilities": {},
            "clientInfo": {"name": "helper-contract", "version": "0"},
        }})
        for command, literal in [
            ('--json variant oncokb "BRAF V600E"', expected),
            ('variant oncokb "BRAF V600E"', markdown),
        ]:
            response = client.call({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {
                "name": "biomcp", "arguments": {"command": command},
            }})["result"]
            assert not response.get("isError", False), response
            text = response["content"][0]["text"]
            assert (json.loads(text) if isinstance(literal, dict) else text.rstrip("\n")) == (literal.rstrip("\n") if isinstance(literal, str) else literal)
            attempts(transport, ["V600E"])
    finally:
        client.close()

    # Independent caller literals own equal-level stability, blank fallback,
    # unknown levels and JSON omission. This is not the producer admission table.
    for source, literal in [
        (b'{"highestSensitiveLevel":" ","highestResistanceLevel":"LEVEL_R1"}',
         {"gene": "BRAF", "alteration": "V600E", "level": "Level R1"}),
        (b'{}', {"gene": "BRAF", "alteration": "V600E"}),
        (b'{"treatments":[{"level":"LEVEL_1","drugs":[{"drugName":"Z"}]},{"level":"LEVEL_1","drugs":[{"drugName":"A"}]},{"drugs":[]},{"level":" custom "}]}',
         {"gene": "BRAF", "alteration": "V600E", "therapies": [
             {"level": "Level 1", "drugs": ["Z"]},
             {"level": "Level 1", "drugs": ["A"]},
             {"level": "Unknown"}, {"level": "custom"},
         ]}),
        ((ROOT / "testdata/sources/oncokb/annotation_braf_v600e.json").read_bytes(),
         {"gene": "BRAF", "alteration": "V600E", "oncogenic": "Oncogenic",
          "effect": "Gain-of-function", "level": "Level 1", "therapies": [
              {"level": "Level 1", "drugs": ["Dabrafenib"], "cancer_type": "Melanoma"},
          ]}),
    ]:
        transport["body"] = source
        result = cli("--json", "variant", "oncokb", "BRAF V600E")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout) == literal
        attempts(transport, ["V600E"])
    transport["body"] = b"{}"
    result = cli("variant", "oncokb", "BRAF V600E")
    assert result.returncode == 0, result.stderr
    assert "No therapy implications returned by OncoKB." in result.stdout
    attempts(transport, ["V600E"])


def test_oncokb_helper_refuses_failures_without_false_annotation(transport, monkeypatch):
    # A formerly ignored deep payload proves actual caller admission tightened.
    deep = b'{"ignored":' + b'[' * 127 + b'"synthetic-private-marker"' + b']' * 127 + b'}'
    cases = [
        ("missing token", "BRAF V600E", "", 200, b"{}", "api_key_required", []),
        ("no protein change", "rs113488022", "synthetic-token", 200, b"{}", "invalid_argument", []),
        ("HTTP refusal", "BRAF V600E", "synthetic-token", 400, b"upstream failed", "api", ["V600E", "p.V600E"]),
        ("malformed success", "BRAF V600E", "synthetic-token", 200,
         b'{"treatments":"synthetic-private-marker"}', "api_json", ["V600E", "p.V600E"]),
        ("ignored depth refusal", "BRAF V600E", "synthetic-token", 200,
         deep, "api_json", ["V600E", "p.V600E"]),
    ]
    for name, variant, token, status, body, category, requested in cases:
        monkeypatch.setenv("ONCOKB_TOKEN", token)
        transport.update(body=body, status=status, no_change=name == "no protein change")
        result = cli("--json", "variant", "oncokb", variant)
        assert result.returncode != 0, (name, result.stdout)
        error = json.loads(result.stdout)
        assert error["error"]["code"] == category, (name, error)
        assert "synthetic-private-marker" not in result.stdout + result.stderr
        if name == "no protein change":
            assert "requires a protein change" in error["error"]["message"]
        assert "therapies" not in error
        attempts(transport, requested)
