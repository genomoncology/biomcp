"""Short CLI invocations share the configured HTTP cache size budget."""
from __future__ import annotations

from concurrent.futures import ThreadPoolExecutor
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import threading

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import retained_cache_env  # noqa: E402

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
        cwd=ROOT, env=retained_cache_env(cache.parent, {"BIOMCP_MYGENE_BASE": base}, base,
            cache_dir=cache, max_size=limit),
        capture_output=True, text=True, timeout=20,
    )
    assert result.returncode == 0, (result.stdout, result.stderr)
    assert json.loads(result.stdout)["results"]


def content_bytes(cache):
    with sqlite3.connect(cache / "biomcp.db") as connection:
        total = connection.execute("select bytes from cache_meta where id=1").fetchone()[0]
        assert total == connection.execute("select coalesce(sum(bytes),0) from entries").fetchone()[0]
        return total


def test_short_cli_runs_evict_within_the_shared_size_budget(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    sizes = []
    for index in range(6):
        run_gene(cache, gene_provider, index)
        sizes.append(content_bytes(cache))
    assert 0 < sizes[-1] <= 50000, f"cache bytes after each short process: {sizes}"



def test_concurrent_short_processes_do_not_lose_size_reservations(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    run_gene(cache, gene_provider, 0, limit=1000000)
    with ThreadPoolExecutor(max_workers=4) as workers:
        list(workers.map(lambda index: run_gene(cache, gene_provider, index, limit=1000000), range(1, 9)))
    assert content_bytes(cache) > 0, "concurrent puts retain exact committed accounting"


def test_explicit_cleanup_previews_then_removes_committed_cache_bytes(tmp_path, gene_provider):
    cache = tmp_path / "cache"
    run_gene(cache, gene_provider, 0, limit=1000000)
    stored_bytes = content_bytes(cache)
    assert stored_bytes > 0
    env = retained_cache_env(tmp_path, {}, gene_provider, cache_dir=cache)
    command = [str(BINARY), "--json", "cache", "clean", "--max-size", "1B"]
    preview = subprocess.run(command + ["--dry-run"], cwd=ROOT, env=env,
                             capture_output=True, text=True, timeout=20)
    assert preview.returncode == 0, (preview.stdout, preview.stderr)
    assert json.loads(preview.stdout)["errors"] == []
    assert content_bytes(cache) == stored_bytes
    cleaned = subprocess.run(command, cwd=ROOT, env=env,
                             capture_output=True, text=True, timeout=20)
    assert cleaned.returncode == 0, (cleaned.stdout, cleaned.stderr)
    assert json.loads(cleaned.stdout)["errors"] == []
    assert content_bytes(cache) == 0
