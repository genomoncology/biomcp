"""Credential canaries never reach persistent cache bytes or CLI diagnostics."""
from __future__ import annotations

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import threading
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import retained_cache_env
from hang_guard import HANG_TIMEOUT

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))
CANARY = "fake-credential-canary-never-a-real-key"
LEGACY = ("http", "http-cacache", "citation-evidence", "captures")


def run(env, *args):
    return subprocess.run([str(BINARY), "--json", *args], env=env, cwd=ROOT,
                          capture_output=True, timeout=HANG_TIMEOUT)


def assert_no_canary(root, *outputs):
    # Never include suspect bytes in an assertion message.
    assert all(CANARY.encode() not in output for output in outputs), "credential in CLI output"
    assert all(CANARY.encode() not in path.read_bytes() for path in root.rglob("*")
               if path.is_file()), "credential in persistent cache"


@pytest.fixture
def replay(tmp_path):
    received = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            target = urlsplit(self.path)
            received.append((target.path, parse_qs(target.query), dict(self.headers)))
            if target.path.endswith("esearch.fcgi"):
                payload = {"esearchresult": {"count": "0", "idlist": []}}
            else:
                payload = {"data": [], "total": 0}
            # A provider may echo request context. Cache publication must not
            # persist an authentication value even in a response body.
            if (any(CANARY in values for values in parse_qs(target.query).values())
                    or CANARY in self.headers.values()):
                payload["request_context"] = CANARY
            body = json.dumps(payload).encode()
            self.send_response(503 if "credential-failure" in self.path else 200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Cache-Control", "public, max-age=3600")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = retained_cache_env(tmp_path, {
        "BIOMCP_PUBMED_BASE": origin + "/entrez/eutils", "BIOMCP_S2_BASE": origin,
        "BIOMCP_NCI_CTS_BASE": origin,
    }, origin)
    env["RUST_LOG"] = "debug"
    try:
        yield env, received
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


@pytest.mark.parametrize("key,args", [
    ("NCBI_API_KEY", ("search", "article", "BRAF", "--source", "pubmed", "--limit", "1")),
    ("S2_API_KEY", ("search", "article", "BRAF", "--source", "semanticscholar", "--limit", "1")),
    ("NCI_API_KEY", ("search", "trial", "--source", "nci", "--condition", "melanoma", "--limit", "1")),
])
def test_keyed_requests_keep_cache_and_debug_output_credential_free(replay, key, args):
    env, received = replay
    env[key] = CANARY
    result = run(env, *args)
    assert result.returncode == 0, "fake keyed request failed"
    assert received, "the real CLI must reach the replay transport"
    assert any(CANARY in values for _, query, headers in received
               for values in [*query.values(), *[[value] for value in headers.values()]]), \
        "the fake key must actually be sent"
    assert_no_canary(Path(env["BIOMCP_CACHE_DIR"]), result.stdout, result.stderr)


def test_first_use_removes_legacy_canaries_and_preserves_current_cache(replay):
    env, _ = replay
    cache = Path(env["BIOMCP_CACHE_DIR"])
    for name in LEGACY:
        directory = cache / name
        directory.mkdir(parents=True)
        (directory / "index").write_text("https://example.org/?api_key=" + CANARY)
    downloads = cache / "downloads"
    downloads.mkdir()
    (downloads / "saved.txt").write_text("saved article")
    retained_entries = None
    for _ in range(2):
        result = run(env, "cache", "stats")
        assert result.returncode == 0, "cache initialization failed"
        assert all(not (cache / name).exists() for name in LEGACY), "legacy cache survived"
        assert (cache / "biomcp.db").is_file()
        assert (downloads / "saved.txt").read_text() == "saved article"
        assert_no_canary(cache, result.stdout, result.stderr)
        if retained_entries is not None:
            with sqlite3.connect(cache / "biomcp.db") as connection:
                assert connection.execute("SELECT count(*) FROM entries").fetchone()[0] == retained_entries
        else:
            assert run(env, "search", "article", "BRAF", "--source", "pubmed", "--limit", "1").returncode == 0
            with sqlite3.connect(cache / "biomcp.db") as connection:
                retained_entries = connection.execute("SELECT count(*) FROM entries").fetchone()[0]
            assert retained_entries > 0


def test_upgrade_removes_canary_from_old_sqlite_pages_and_blobs(replay):
    env, _ = replay
    assert run(env, "cache", "stats").returncode == 0
    cache = Path(env["BIOMCP_CACHE_DIR"])
    database = cache / "biomcp.db"
    with sqlite3.connect(database) as connection:
        connection.execute("PRAGMA user_version=1")
        connection.execute("INSERT INTO entries(namespace,key,base_key,metadata,body,bytes,created,last_access) "
                           "VALUES('http','old','old',?,X'7B7D',?,0,0)",
                           (CANARY.encode(), len(CANARY) + 2))
    blobs = cache / "blobs"
    blobs.mkdir(exist_ok=True)
    (blobs / "old-response").write_text(CANARY)
    result = run(env, "cache", "stats")
    assert result.returncode == 0, "cache migration failed"
    assert_no_canary(cache, result.stdout, result.stderr)
    with sqlite3.connect(database) as connection:
        assert connection.execute("SELECT count(*) FROM entries").fetchone() == (0,)


@pytest.mark.parametrize("key,args", [
    ("NCBI_API_KEY", ("search", "article", "credential-failure", "--source", "pubmed", "--limit", "1")),
    ("S2_API_KEY", ("search", "article", "credential-failure", "--source", "semanticscholar", "--limit", "1")),
    ("NCI_API_KEY", ("search", "trial", "--source", "nci", "--condition", "credential-failure", "--limit", "1")),
])
def test_failed_keyed_request_redacts_debug_and_error_output(replay, key, args):
    env, received = replay
    env[key] = CANARY
    result = run(env, *args)
    assert result.returncode != 0
    assert received
    assert_no_canary(Path(env["BIOMCP_CACHE_DIR"]), result.stdout, result.stderr)


def test_legacy_cleanup_unlinks_symlinks_without_touching_their_targets(replay, tmp_path):
    env, _ = replay
    cache = Path(env["BIOMCP_CACHE_DIR"])
    cache.mkdir(parents=True, exist_ok=True)
    outside = tmp_path / "saved-outside-cache"
    outside.mkdir()
    sentinel = outside / "saved.txt"
    sentinel.write_text("keep saved data")
    for name in LEGACY:
        (cache / name).symlink_to(outside, target_is_directory=True)
    result = run(env, "cache", "stats")
    assert result.returncode == 0, "cache initialization failed"
    assert all(not (cache / name).is_symlink() for name in LEGACY)
    assert sentinel.read_text() == "keep saved data"


def test_upgrade_with_active_reader_retries_before_marking_sanitization_complete(replay):
    env, _ = replay
    assert run(env, "cache", "stats").returncode == 0
    cache = Path(env["BIOMCP_CACHE_DIR"])
    database = cache / "biomcp.db"
    with sqlite3.connect(database) as connection:
        connection.execute("PRAGMA user_version=1")
        connection.execute("INSERT INTO entries(namespace,key,base_key,metadata,body,bytes,created,last_access) "
                           "VALUES('http','old','old',?,X'7B7D',?,0,0)",
                           (CANARY.encode(), len(CANARY) + 2))
    reader = sqlite3.connect(database)
    try:
        reader.execute("BEGIN")
        reader.execute("SELECT count(*) FROM entries").fetchone()
        result = run(env, "cache", "stats")
        assert result.returncode != 0, "occupied migration must not claim success"
        assert CANARY.encode() not in result.stdout + result.stderr
        with sqlite3.connect(database) as probe:
            assert probe.execute("PRAGMA user_version").fetchone() == (1,)
    finally:
        reader.close()
    result = run(env, "cache", "stats")
    assert result.returncode == 0, "migration must retry after the reader closes"
    assert_no_canary(cache, result.stdout, result.stderr)
    with sqlite3.connect(database) as connection:
        assert connection.execute("PRAGMA user_version").fetchone() == (2,)


@pytest.mark.parametrize("command", [("cache", "clean"), ("cache", "clear", "--yes")])
def test_explicit_clean_and_clear_remove_retired_stores(replay, command):
    env, _ = replay
    assert run(env, "cache", "stats").returncode == 0
    cache = Path(env["BIOMCP_CACHE_DIR"])
    for name in LEGACY:
        directory = cache / name
        directory.mkdir()
        (directory / "index").write_text("https://example.org/?api_key=" + CANARY)
    result = run(env, *command)
    assert result.returncode == 0, "explicit cache cleanup failed"
    assert all(not (cache / name).exists() for name in LEGACY)
    assert_no_canary(cache, result.stdout, result.stderr)
