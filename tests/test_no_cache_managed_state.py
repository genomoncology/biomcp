from __future__ import annotations

from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import threading


ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))


class GeneHandler(BaseHTTPRequestHandler):
    payload = {"total": 1, "hits": [{"_id": "673", "symbol": "BRAF", "name": "cached-name", "entrezgene": "673"}]}
    status = 200
    requests = 0

    def do_GET(self) -> None:
        type(self).requests += 1
        body = json.dumps(type(self).payload).encode()
        self.send_response(type(self).status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Cache-Control", "max-age=3600")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args: object) -> None:
        pass


class SlowGeneHandler(GeneHandler):
    started = threading.Event()
    release = threading.Event()

    def do_GET(self) -> None:
        type(self).started.set()
        type(self).release.wait(5)
        super().do_GET()


def run(cache: Path, base: str, *arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [BINARY, *arguments],
        cwd=ROOT,
        env=os.environ | {"BIOMCP_CACHE_DIR": str(cache), "BIOMCP_MYGENE_BASE": base},
        text=True,
        capture_output=True,
    )


def snapshot(root: Path) -> dict[str, bytes]:
    if not root.exists():
        return {}
    return {path.relative_to(root).as_posix(): path.read_bytes() for path in root.rglob("*") if path.is_file()}


def test_no_cache_bypasses_prepopulated_cache_without_updating_it(tmp_path: Path) -> None:
    GeneHandler.requests = 0
    GeneHandler.status = 200
    GeneHandler.payload = {"total": 1, "hits": [{"_id": "673", "symbol": "BRAF", "name": "cached-name", "entrezgene": "673"}]}
    server = ThreadingHTTPServer(("127.0.0.1", 0), GeneHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    cache = tmp_path / "cache"
    try:
        seeded = run(cache, base, "search", "gene", "BRAF", "--limit", "1")
        assert seeded.returncode == 0, seeded.stderr
        assert "cached-name" in seeded.stdout
        before = snapshot(cache)
        GeneHandler.payload = {"total": 1, "hits": [{"_id": "673", "symbol": "BRAF", "name": "fresh-name", "entrezgene": "673"}]}
        fresh = run(cache, base, "--no-cache", "--json", "search", "gene", "BRAF", "--limit", "1")
        assert fresh.returncode == 0, fresh.stderr
        assert "fresh-name" in fresh.stdout
        assert GeneHandler.requests == 2
        assert snapshot(cache) == before
    finally:
        server.shutdown()
        thread.join()


def test_no_cache_session_is_rejected_before_transport_in_both_orders(tmp_path: Path) -> None:
    for arguments in (
        ("--no-cache", "search", "article", "fixture", "--session", "local"),
        ("search", "article", "fixture", "--session", "local", "--no-cache"),
    ):
        cache = tmp_path / ("cache-" + str(len(arguments)) + arguments[0].replace("-", ""))
        result = run(cache, "http://127.0.0.1:9", *arguments)
        assert result.returncode == 2
        assert "cannot be combined with --session" in result.stderr
        assert not cache.exists()


def test_no_cache_cache_commands_reject_before_filesystem_in_both_orders(tmp_path: Path) -> None:
    subcommands = (
        ("path",),
        ("stats",),
        ("clean", "--dry-run"),
        ("clear", "--yes"),
    )
    for json_mode in (False, True):
        for position in ("before", "after"):
            for subcommand in subcommands:
                cache = tmp_path / f"{json_mode}-{position}-{'-'.join(subcommand)}"
                prefix = ("--json",) if json_mode else ()
                if position == "before":
                    arguments = (*prefix, "--no-cache", "cache", *subcommand)
                else:
                    arguments = (*prefix, "cache", *subcommand, "--no-cache")
                result = run(cache, "http://127.0.0.1:9", *arguments)

                assert result.returncode == 2, (arguments, result.stdout, result.stderr)
                assert not cache.exists(), arguments
                if json_mode:
                    assert result.stderr == "", arguments
                    value = json.loads(result.stdout)
                    assert value["error"]["code"] == "invalid_argument"
                    assert "--no-cache" in value["error"]["message"]
                else:
                    assert result.stdout == "", arguments
                    assert "--no-cache" in result.stderr


def test_failed_no_cache_request_leaves_no_managed_state(tmp_path: Path) -> None:
    GeneHandler.requests = 0
    GeneHandler.status = 500
    server = ThreadingHTTPServer(("127.0.0.1", 0), GeneHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    cache = tmp_path / "failure-cache"
    try:
        result = run(cache, f"http://127.0.0.1:{server.server_port}", "--no-cache", "search", "gene", "BRAF", "--limit", "1")
        assert result.returncode != 0
        assert GeneHandler.requests > 0
        assert not cache.exists()
    finally:
        server.shutdown()
        thread.join()


def test_empty_and_interrupted_no_cache_requests_leave_no_state(tmp_path: Path) -> None:
    GeneHandler.status = 200
    GeneHandler.payload = {"total": 0, "hits": []}
    server = ThreadingHTTPServer(("127.0.0.1", 0), GeneHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    empty_cache = tmp_path / "empty-cache"
    try:
        empty = run(empty_cache, f"http://127.0.0.1:{server.server_port}", "--no-cache", "search", "gene", "NOTREAL", "--limit", "1")
        assert empty.returncode == 0, empty.stderr
        assert not empty_cache.exists()
    finally:
        server.shutdown()
        thread.join()

    SlowGeneHandler.started.clear()
    SlowGeneHandler.release.clear()
    slow = ThreadingHTTPServer(("127.0.0.1", 0), SlowGeneHandler)
    slow_thread = threading.Thread(target=slow.serve_forever, daemon=True)
    slow_thread.start()
    interrupted_cache = tmp_path / "interrupted-cache"
    process = subprocess.Popen(
        [BINARY, "--no-cache", "search", "gene", "BRAF", "--limit", "1"],
        cwd=ROOT,
        env=os.environ | {
            "BIOMCP_CACHE_DIR": str(interrupted_cache),
            "BIOMCP_MYGENE_BASE": f"http://127.0.0.1:{slow.server_port}",
        },
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        assert SlowGeneHandler.started.wait(2)
        process.terminate()
        process.communicate(timeout=5)
        assert not interrupted_cache.exists()
    finally:
        SlowGeneHandler.release.set()
        slow.shutdown()
        slow_thread.join()
