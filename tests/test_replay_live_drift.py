"""Release drift checks use local HTTP replies, never a public provider."""

from __future__ import annotations

import hashlib
import json
import runpy
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests/golden"))
from replay_env import isolated_command_env  # noqa: E402

CHECKER = ROOT / "tools/check-replay-live"
RECORDED = {"hits": [{"symbol": "BRAF", "name": "B-Raf"}], "total": 1}


@pytest.fixture
def provider():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            self.server.requests.append((time.monotonic(), dict(self.headers)))
            self.send_response(self.server.status)
            self.send_header("Content-Type", self.server.content_type)
            if self.server.retry_after:
                self.send_header("Retry-After", "30")
            self.end_headers()
            body = self.server.body
            self.wfile.write(
                body if isinstance(body, bytes) else json.dumps(body).encode()
            )

        def do_POST(self):
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.do_GET()

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.requests = []
    server.status = 200
    server.content_type = "application/json; charset=utf-8"
    server.body = RECORDED
    server.retry_after = False
    thread = threading.Thread(target=server.serve_forever)
    thread.start()
    try:
        yield server, f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def recording(tmp_path, origin, count=1, provider="mygene"):
    replay = tmp_path / "replay"
    replay.mkdir()
    (replay / "reply.json").write_text(json.dumps(RECORDED))
    routes = {
        f"GET /{provider}/query?q={index}": {
            "file": "reply.json",
            "status": 200,
            "headers": {"content-type": "application/json"},
            "upstream": f"{origin}/query?q={index}",
        }
        for index in range(count)
    }
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    return replay, routes


def run_check(tmp_path, replay, *args, env_values=None):
    env = isolated_command_env(tmp_path / "home", {}, "http://127.0.0.1")
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env.pop("BIOMCP_DISABLE_KEGG", None)
    env.update(env_values or {})
    process = subprocess.run(
        [sys.executable, str(CHECKER), "--replay-dir", str(replay), *args],
        capture_output=True,
        text=True,
        env=env,
        timeout=30,
    )
    assert process.stdout, process.stderr
    lines = [json.loads(line) for line in process.stdout.splitlines()]
    return process, lines[:-1], lines[-1]["summary"]


def test_inventory_never_contacts_provider_without_live_opt_in(tmp_path, provider):
    server, origin = provider
    replay, _ = recording(tmp_path, origin)
    process, _, summary = run_check(tmp_path, replay)
    assert process.returncode == 0
    assert summary["total"] == summary["ready"] == 1
    assert summary["checked"] == 0
    assert not server.requests


@pytest.mark.parametrize(
    ("change", "detail"),
    [
        ("field", "missing field $/hits/*/symbol"),
        ("type", "changed type $/hits/*/symbol"),
        ("status", "changed status"),
        ("content-type", "changed content-type"),
    ],
)
def test_live_check_reports_deliberately_stale_recordings(
    tmp_path, provider, change, detail
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin)
    if change == "field":
        server.body = {"hits": [{"renamedSymbol": "BRAF", "name": "B-Raf"}], "total": 1}
    elif change == "type":
        server.body = {"hits": [{"symbol": 42, "name": "B-Raf"}], "total": 1}
    elif change == "status":
        server.status = 503
    else:
        server.content_type = "text/html"
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == 1
    assert detail in results[0]["details"]


def test_live_check_ignores_values_order_and_unused_fields_and_paces_calls(
    tmp_path, provider
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, count=2)
    server.body = {
        "total": 99,
        "hits": [{"name": "changed", "symbol": "changed", "unused": 2}],
        "timestamp": "changed",
    }
    process, _, summary = run_check(
        tmp_path, replay, "--live", env_values={"NCBI_API_KEY": "private-marker"}
    )
    assert process.returncode == 0
    assert summary["total"] == summary["checked"] == summary["passed"] == 2
    assert server.requests[1][0] - server.requests[0][0] >= 1.9
    assert "private-marker" not in process.stdout + process.stderr + str(
        server.requests
    )
    assert all("Authorization" not in headers for _, headers in server.requests)


def test_rate_limit_stops_remaining_provider_requests_without_retry(tmp_path, provider):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, count=2)
    server.status = 429
    server.retry_after = True
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == summary["blocked"] == 1
    assert len(server.requests) == 1


@pytest.mark.parametrize(
    "fault",
    [
        "content-type",
        "body",
        "hash",
        "credential-url",
        "credential-header",
        "credential-body",
        "unsupported",
    ],
)
def test_incomplete_or_unsafe_recordings_are_counted_and_never_sent(
    tmp_path, provider, fault
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    key, entry = next(iter(routes.items()))
    if fault == "content-type":
        entry.pop("headers")
    elif fault in {"body", "hash"}:
        routes = {"POST /mygene/query#0000000000000000": entry}
        if fault == "hash":
            entry["request_body"] = '{"ids":["BRAF"]}'
    elif fault == "credential-url":
        entry["upstream"] += "&api_key=private-marker"
    elif fault == "credential-header":
        entry["request_headers"] = {"Authorization": "private-marker"}
    elif fault == "credential-body":
        body = '{"token":"private-marker"}'
        digest = hashlib.sha256(body.encode()).hexdigest()[:16]
        routes = {f"POST /mygene/query#{digest}": {**entry, "request_body": body}}
    else:
        (replay / "reply.json").write_text('{"unconsumedProviderField":true}')
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["total"] == summary["blocked"] == 1
    assert summary["checked"] == 0
    assert not server.requests
    assert "private-marker" not in process.stdout + process.stderr


def test_disabled_source_groups_and_legacy_kegg_alias_prevent_requests(
    tmp_path, provider
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, provider="kegg")
    for env in (
        {"BIOMCP_DISABLE_SOURCES": "license-required"},
        {"BIOMCP_DISABLE_KEGG": "1"},
    ):
        process, _, summary = run_check(tmp_path, replay, "--live", env_values=env)
        assert process.returncode == 0
        assert summary["total"] == summary["disabled"] == 1
        assert summary["checked"] == 0
    assert not server.requests


def test_recorder_keeps_case_insensitive_response_headers_and_safe_post_metadata(
    tmp_path, provider
):
    _, origin = provider
    module = runpy.run_path(str(ROOT / "tools/replay-server"))
    store = module["ReplayStore"](tmp_path / "replay")
    recorder = module["ReplayServer"]({}, {}, store, lambda _: None, True)
    for payload, safe in [
        (b'{"ids":["BRAF"]}', True),
        (b'{"token":"private-marker"}', False),
    ]:
        key = "POST /mygene/query#" + hashlib.sha256(payload).hexdigest()[:16]
        recorder.record_reply(
            key,
            "mygene",
            origin,
            "/query",
            "",
            "POST",
            {"Content-Type": "application/json", "Authorization": "private-marker"},
            payload,
        )
        entry = store.routes[key]
        assert entry["headers"]["content-type"].startswith("application/json")
        assert ("request_body" in entry) is safe
        assert "private-marker" not in json.dumps(entry)


def test_kegg_requires_access_authorization_and_detects_stale_text_shape(
    tmp_path, provider
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin, provider="kegg")
    entry = next(iter(routes.values()))
    entry["headers"]["content-type"] = "text/plain"
    routes = {"GET /kegg/find/pathway/MAPK": entry}
    (replay / "reply.json").write_text("map04010\tMAPK signaling pathway\n")
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["blocked"] == 1
    assert not server.requests
    server.content_type = "text/plain"
    server.body = b"provider changed to a different format"
    process, results, summary = run_check(
        tmp_path, replay, "--live", "--allow-restricted", "kegg"
    )
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == 1
    assert results[0]["details"] == ["live response has unsupported or invalid shape"]


def test_projection_reads_models_after_test_constructors_and_provider_transforms(
    tmp_path, provider
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, provider="mydisease")
    recorded = {"_id": "MONDO:1", "mondo": {"name": "old label"}}
    (replay / "reply.json").write_text(json.dumps(recorded))
    server.body = {"_id": "MONDO:2", "mondo": {"label": "new label"}}
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == 1
    assert "missing field $/mondo/name" in results[0]["details"]
