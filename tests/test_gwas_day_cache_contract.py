"""A repeat GWAS search inside one day answers from the cache (ticket 2091).

The CLI surface contract for the GWAS day cache: run `biomcp search gwas`
twice against a counting local GWAS Catalog server with one isolated cache
root, and the second run makes no request — the stored answer serves even
though the provider forbade storage. The reader paths (aging past a day,
the bypasses, and the decode-failure replacement) are proven on the client
in `src/sources/gwas/tests/day_cache.rs`.
"""

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

import pytest

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

REPO_ROOT = Path(__file__).resolve().parents[1]
BIOMCP_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/release/biomcp"))

GWAS_SEARCH_BODY = json.dumps(
    {
        "_embedded": {
            "associations": [
                {
                    "accessionId": "GCST90002321",
                    "pValue": 5e-8,
                    "mappedGenes": ["BRAF"],
                    "reportedTrait": ["melanoma"],
                }
            ]
        },
        "page": {"totalElements": 1},
    }
)


class _CountingGwasState:
    def __init__(self) -> None:
        self.lock = threading.Lock()
        self.gwas_requests = 0
        self.other_requests = 0


class _CountingGwasHandler(BaseHTTPRequestHandler):
    server: "_CountingGwasServer"

    def log_message(self, format: str, *args: Any) -> None:  # noqa: A002
        return

    def do_GET(self) -> None:
        with self.server.state.lock:
            if self.path.startswith("/v2/associations"):
                self.server.state.gwas_requests += 1
            else:
                self.server.state.other_requests += 1
        body = GWAS_SEARCH_BODY.encode("utf-8")
        self.send_response(200)
        self.send_header("content-type", "application/json")
        # The day cache stores the answer whatever the headers say, so
        # the fixture forbids storage outright.
        self.send_header("cache-control", "no-store")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


class _CountingGwasServer(ThreadingHTTPServer):
    state: _CountingGwasState


def _run_search_gwas(base_url: str, cache_home: Path) -> subprocess.CompletedProcess[str]:
    assert BIOMCP_BIN.exists(), f"missing biomcp binary: {BIOMCP_BIN}"
    env = os.environ.copy()
    env.update(
        {
            "BIOMCP_GWAS_BASE": base_url,
            "BIOMCP_CACHE_DIR": str(cache_home),
            "XDG_CACHE_HOME": str(cache_home / "xdg-cache"),
            "RUST_LOG": "error",
        }
    )
    env.pop("BIOMCP_CACHE_MODE", None)
    return subprocess.run(
        [str(BIOMCP_BIN), "--json", "search", "gwas", "-g", "BRAF"],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )


def test_a_repeat_gwas_search_inside_a_day_makes_no_request() -> None:
    state = _CountingGwasState()
    server = _CountingGwasServer(("127.0.0.1", 0), _CountingGwasHandler)
    server.state = state
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        host, port = server.server_address
        base_url = f"http://{host}:{port}"
        with tempfile.TemporaryDirectory(prefix="biomcp-gwas-day-cache-") as cache_home:
            first = _run_search_gwas(base_url, Path(cache_home))
            assert first.returncode == 0, first.stderr
            second = _run_search_gwas(base_url, Path(cache_home))
            assert second.returncode == 0, second.stderr
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

    first_payload = json.loads(first.stdout)
    second_payload = json.loads(second.stdout)
    assert first_payload["results"], "the fixture answer reached the first run"
    assert second_payload["results"] == first_payload["results"], (
        "the cached repeat must return the same rows"
    )
    with state.lock:
        assert state.gwas_requests == 1, (
            "the second run must be answered from the cache: "
            f"{state.gwas_requests} GWAS requests"
        )
        assert state.other_requests == 0
