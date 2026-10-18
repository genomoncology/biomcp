"""Short CLI invocations share the configured HTTP cache size budget."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import threading

import pytest

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))


@pytest.fixture
def gene_provider():
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            body = json.dumps({"total": 1, "hits": [{"_id": "673", "symbol": "BRAF",
                "name": "cached BRAF", "entrezgene": "673", "unused": "x" * 16000}]}).encode()
            self.send_response(200)
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
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def run_gene(cache, base, index, limit=50000):
    result = subprocess.run(
        [str(BINARY), "--json", "search", "gene", f"BRAF{index}", "--limit", "1"],
        cwd=ROOT, env=os.environ | {"BIOMCP_CACHE_DIR": str(cache), "BIOMCP_MYGENE_BASE": base,
            "BIOMCP_CACHE_MODE": "default", "BIOMCP_CACHE_MAX_SIZE": str(limit), "BIOMCP_CACHE_MIN_DISK_FREE": "1B"},
        capture_output=True, text=True, timeout=20,
    )
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert json.loads(result.stdout)["results"]


def content_bytes(cache):
    return sum(p.stat().st_size for p in (cache / "http/content-v2").rglob("*") if p.is_file())


def test_short_cli_runs_evict_within_the_shared_size_budget(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    sizes = []
    for index in range(6):
        run_gene(cache, gene_provider, index)
        sizes.append(content_bytes(cache))
    assert 0 < sizes[-1] <= 50000, f"cache bytes after each short process: {sizes}"


@pytest.mark.parametrize("marker", [None, b"1", b"v2\n1\n"])
def test_missing_or_torn_total_recovers_before_the_next_short_run(tmp_path, gene_provider, marker):
    cache = tmp_path / "cache"
    for index in range(3):
        run_gene(cache, gene_provider, index, limit=1000000)
    total = cache / "http/.content-bytes-v1"
    if marker is None:
        total.unlink()
    else:
        total.write_bytes(marker)
    run_gene(cache, gene_provider, 4)
    assert 0 < content_bytes(cache) <= 50000, "invalid total must recount and evict"


def test_concurrent_short_processes_do_not_lose_size_reservations(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    run_gene(cache, gene_provider, 0, limit=1000000)
    with ThreadPoolExecutor(max_workers=4) as workers:
        list(workers.map(lambda index: run_gene(cache, gene_provider, index, limit=1000000), range(1, 9)))
    marker = (cache / "http/.content-bytes-v1").read_text()
    assert marker.startswith("v2 ") and marker.endswith("\n"), marker
    assert int(marker[3:-1]) >= content_bytes(cache), "concurrent puts must not lose additions"


def test_explicit_cleanup_reconciles_failed_write_reservations(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    run_gene(cache, gene_provider, 0, limit=1000000)
    stored_bytes = content_bytes(cache)
    assert stored_bytes > 0
    total = cache / "http/.content-bytes-v1"
    # A write reserves bytes before publishing content. If publication fails,
    # that reservation legitimately overcounts until maintenance reconciles it.
    reserved_total = f"v2 {stored_bytes + 100000}\n"
    total.write_text(reserved_total)
    env = os.environ | {"BIOMCP_CACHE_DIR": str(cache), "BIOMCP_CACHE_MIN_DISK_FREE": "1B"}
    command = [str(BINARY), "--json", "cache", "clean", "--max-size", "1B"]
    preview = subprocess.run(command + ["--dry-run"], cwd=ROOT, env=env,
                             capture_output=True, text=True, timeout=20)
    assert preview.returncode == 0, (preview.stdout, preview.stderr)
    assert json.loads(preview.stdout)["errors"] == []
    assert content_bytes(cache) == stored_bytes
    assert total.read_text() == reserved_total, "dry-run must not reconcile reservations"
    cleaned = subprocess.run(command, cwd=ROOT, env=env,
                             capture_output=True, text=True, timeout=20)
    assert cleaned.returncode == 0, (cleaned.stdout, cleaned.stderr)
    assert json.loads(cleaned.stdout)["errors"] == []
    assert content_bytes(cache) == 0
    assert total.read_text() == "v2 0\n", "successful cleanup must discard phantom reservations"
