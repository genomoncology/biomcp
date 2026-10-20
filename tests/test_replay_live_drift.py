"""Release drift checks use local HTTP replies, never a public provider."""

from __future__ import annotations

import hashlib
import json
import runpy
import subprocess
import sys
import threading
import time
import tomllib
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


def test_live_check_ignores_values_and_order_reports_added_paths_and_paces_calls(
    tmp_path, provider
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, count=2)
    server.body = {
        "total": 99,
        "hits": [{"name": "changed", "symbol": "changed", "unused": 2}],
        "timestamp": "changed",
    }
    process, results, summary = run_check(
        tmp_path, replay, "--live", env_values={"NCBI_API_KEY": "private-marker"}
    )
    assert process.returncode == 0
    assert summary["total"] == summary["checked"] == summary["passed"] == 2
    assert results[0]["added_paths"] == ["$/hits/*/unused", "$/timestamp"]
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
        (replay / "reply.json").write_text("unsupported plaintext reply")
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


def test_plaintext_stays_blocked_even_with_restricted_access_authorization(
    tmp_path, provider
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin, provider="kegg")
    entry = next(iter(routes.values()))
    entry["headers"]["content-type"] = "text/plain"
    (replay / "reply.json").write_text("map04010\tMAPK signaling pathway\n")
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    for args, detail in [
        ([], "restricted provider needs explicit access authorization or disabling"),
        (
            ["--allow-restricted", "kegg"],
            "unsupported response format; structure comparison supports JSON and XML only",
        ),
    ]:
        process, results, summary = run_check(tmp_path, replay, "--live", *args)
        assert process.returncode == 1
        assert summary["blocked"] == 1
        assert summary["checked"] == 0
        assert results[0]["details"] == [detail]
    assert not server.requests


@pytest.mark.parametrize("accept", [None, "application/xml"])
def test_live_check_sends_current_product_headers_and_recorded_accept(
    tmp_path, provider, accept
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    if accept:
        next(iter(routes.values()))["request_headers"] = {"aCcEpT": accept}
        (replay / "manifest.json").write_text(
            json.dumps({"schema_version": 1, "routes": routes})
        )
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 0
    assert summary["checked"] == summary["passed"] == 1
    headers = {name.lower(): value for name, value in server.requests[0][1].items()}
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    assert headers["user-agent"] == f"biomcp-cli/{version}"
    assert headers["accept"] == (accept or "*/*")
    assert headers["cache-control"] == "no-store"
    assert headers["accept-encoding"] == "identity"


def test_live_check_reports_removed_nested_json_paths(tmp_path, provider):
    server, origin = provider
    replay, _ = recording(tmp_path, origin, provider="mydisease")
    recorded = {"_id": "MONDO:1", "mondo": {"name": "old label"}}
    (replay / "reply.json").write_text(json.dumps(recorded))
    server.body = {"_id": "MONDO:2", "mondo": {"label": "new label"}}
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == 1
    assert "missing field $/mondo/name" in results[0]["details"]


@pytest.mark.parametrize(
    ("recorded", "live", "detail"),
    [
        ({"symbol": None}, {"symbol": 42}, "changed type $/symbol"),
        (
            {"hits": [{"symbol": "x"}]},
            {"hits": [{"symbol": "y"}, {"symbol": 42}]},
            "changed type $/hits/*/symbol",
        ),
        (
            {"hits": [{"symbol": None}, {"symbol": "x"}]},
            {"hits": [{"symbol": "y"}]},
            "changed type $/hits/*/symbol",
        ),
        (
            {"uniprot": {"providerCode": "x"}},
            {"uniprot": {"renamed": "x"}},
            "missing field $/uniprot/providerCode",
        ),
        (
            {"authors": [{"ids": ["a"]}]},
            {"authors": [{"ids": [42]}]},
            "changed type $/authors/*/ids/*",
        ),
    ],
)
def test_all_json_paths_and_observed_type_unions_are_checked(
    tmp_path, provider, recorded, live, detail
):
    server, origin = provider
    replay, _ = recording(tmp_path, origin)
    (replay / "reply.json").write_text(json.dumps(recorded))
    server.body = live
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["checked"] == summary["drift"] == 1
    assert detail in results[0]["details"]


@pytest.mark.parametrize("removed", [False, True])
def test_xml_element_paths_ignore_text_order_and_repeated_elements(
    tmp_path, provider, removed
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = "application/xml"
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    (replay / "reply.json").write_text(
        "<reply><row><id>1</id><name>A</name></row></reply>"
    )
    server.content_type = "application/xml"
    server.body = (
        b"<reply><row><name>B</name></row></reply>"
        if removed
        else b"<reply><row><name>B</name><id>2</id><extra/></row><row><id>3</id></row></reply>"
    )
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == int(removed)
    assert summary["checked"] == 1
    if removed:
        assert "missing field $/reply/row/id" in results[0]["details"]
    else:
        assert summary["passed"] == 1
        assert results[0]["added_paths"] == ["$/reply/row/extra"]


@pytest.mark.parametrize("historical", [False, True])
@pytest.mark.parametrize(
    "invalid_xml",
    [b"<reply>", b'<?xml version="1.0" encoding="unknown-codec"?><reply/>'],
)
def test_invalid_xml_keeps_complete_recording_accounting(
    tmp_path, provider, historical, invalid_xml
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = "application/xml"
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    (replay / "reply.json").write_bytes(invalid_xml if historical else b"<reply/>")
    server.content_type = "application/xml"
    server.body = b"<reply/>" if historical else invalid_xml
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert not process.stderr
    assert len(results) == summary["total"] == 1
    assert summary["checked"] == int(not historical)
    assert summary["blocked" if historical else "drift"] == 1
    assert len(server.requests) == int(not historical)
    assert results[0]["details"] == [
        "invalid XML response structure"
        if historical
        else "live response has unsupported or invalid shape"
    ]


FORMAT_CASES = [
    ("image/jpeg", b"\xff\xd8\xff\xe0photo", b"not a jpeg"),
    ("application/msword", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1document", b"not a doc"),
    ("application/zip", b"PK\x03\x04archive", b"not a zip"),
    (
        "text/html",
        b"<!DOCTYPE html><html><body>article</body></html>",
        b"not html",
    ),
    ("binary/octet-stream", b'{"Key":"value","tail":1}', b""),
]


@pytest.mark.parametrize(("content_type", "body", "malformed"), FORMAT_CASES)
@pytest.mark.parametrize("fault", [None, "historical", "live", "status", "type"])
def test_real_formats_report_only_the_checks_performed(
    tmp_path, provider, content_type, body, malformed, fault
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = content_type
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    (replay / "reply.json").write_bytes(malformed if fault == "historical" else body)
    server.content_type = content_type if fault != "type" else "application/json"
    server.body = malformed if fault == "live" else body
    server.status = 503 if fault == "status" else 200
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == int(fault is not None), process.stdout
    assert summary["checked"] == int(fault != "historical")
    state = "blocked" if fault == "historical" else "drift" if fault else "passed"
    assert summary[state] == 1
    assert results[0]["added_paths"] == []
    assert results[0]["qualification"] == "status/type/signature only"
    if fault == "status":
        assert "changed status" in results[0]["details"]
    if fault == "type":
        assert "changed content-type" in results[0]["details"]


@pytest.mark.parametrize(
    ("doctype", "refused"),
    [
        (b'<!DOCTYPE reply SYSTEM "UPSTREAM/no-fetch">', False),
        (b'<!DOCTYPE reply [<!ENTITY word "expanded">]>', True),
        (b'<!DOCTYPE reply [<!ENTITY word SYSTEM "file:///never-read">]>', True),
        (
            b'<!DOCTYPE reply [<!ENTITY % remote SYSTEM "UPSTREAM/no-fetch">%remote;]>',
            True,
        ),
    ],
)
@pytest.mark.parametrize("historical", [False, True])
def test_doctype_allows_declarations_but_refuses_entity_expansion(
    tmp_path, provider, doctype, refused, historical
):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = "application/xml"
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    xml = doctype.replace(b"UPSTREAM", origin.encode()) + b"<reply><row/></reply>"
    (replay / "reply.json").write_bytes(
        xml if historical else b"<reply><row/></reply>"
    )
    server.content_type = "application/xml"
    server.body = b"<reply><row/></reply>" if historical else xml
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == int(refused), process.stdout
    state = "blocked" if historical and refused else "drift" if refused else "passed"
    assert summary[state] == 1
    assert len(server.requests) == int(not (historical and refused))


def test_real_formats_keep_root_object_provider_from_capture_provenance(tmp_path):
    replay, routes = recording(tmp_path, "https://mygene.info/v3")
    entry = next(iter(routes.values()))
    entry["recording_provenance"] = {"provider": "mygene"}
    entry["upstream"] = "https://mygene.info/v3/object?q=1"
    (replay / "manifest.json").write_text(
        json.dumps({
            "schema_version": 1, "routes": {"GET /v3/object?q=1": entry}
        })
    )
    process, results, summary = run_check(tmp_path, replay)
    assert process.returncode == 0
    assert results[0]["provider"] == "mygene"
    assert summary["ready"] == 1


def test_real_formats_do_not_parse_octet_stream_json_as_json(tmp_path, provider):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = "application/octet-stream"
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    (replay / "reply.json").write_bytes(b'{"same-prefix":  "recorded"}')
    server.content_type = "application/octet-stream"
    server.body = b'{"same-prefix":  invalid JSON remains uninterpreted'
    process, results, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 0
    assert summary["passed"] == 1
    assert results[0]["qualification"] == "status/type/signature only"


@pytest.mark.parametrize("historical", [False, True])
def test_real_formats_reject_truncated_opaque_signature(tmp_path, provider, historical):
    server, origin = provider
    replay, routes = recording(tmp_path, origin)
    next(iter(routes.values()))["headers"]["content-type"] = "binary/octet-stream"
    (replay / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "routes": routes})
    )
    body = b'{"recorded":"opaque body"}'
    (replay / "reply.json").write_bytes(body[:8] if historical else body)
    server.content_type = "binary/octet-stream"
    server.body = body if historical else body[:8]
    process, _, summary = run_check(tmp_path, replay, "--live")
    assert process.returncode == 1
    assert summary["blocked" if historical else "drift"] == 1
    assert summary["checked"] == int(not historical)
