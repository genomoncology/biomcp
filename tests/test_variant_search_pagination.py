"""Broad search pages retain the provider's order and bounded result window."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

import pytest

from hang_guard import HANG_TIMEOUT

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

pytestmark = pytest.mark.needs_binary


@pytest.mark.parametrize("identifiers,page_size,extra_filter", [
    ([3, 2, 1], 1, []),
    ([9, 4, 8, 2, 7, 1, 6, 3, 5], 2, ["--significance", "pathogenic"]),
])
def test_broad_variant_page_union_preserves_provider_order(
    tmp_path, identifiers, page_size, extra_filter,
):
    hits = [{"_id": f"chr1:g.{value}A>T", "dbnsfp": {"genename": "BRAF"}}
            for value in identifiers]
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            parsed = urlsplit(self.path)
            query = parse_qs(parsed.query)
            offset, limit = int(query["from"][0]), int(query["size"][0])
            requests.append((parsed.path, query, offset, limit))
            body = json.dumps({"hits": hits[offset:offset + limit], "total": len(hits)}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_MYVARIANT_BASE": origin + "/v1"}, origin)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))

    def search(limit, offset):
        result = subprocess.run(
            [binary, "--json", "--no-cache", "search", "variant", "-g", "BRAF",
             "--limit", str(limit), "--offset", str(offset), *extra_filter],
            env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT,
        )
        assert result.returncode == 0, result.stdout + result.stderr
        payload = json.loads(result.stdout)
        assert payload["_meta"]["pagination"]["total"] == len(hits)
        return [row["id"] for row in payload["results"]]

    try:
        unpaged = search(len(hits), 0)
        pages = [search(page_size, offset) for offset in range(0, len(hits), page_size)]
        union = [identifier for page in pages for identifier in page]
        assert union == unpaged
        assert len(set(union)) == len(hits)
        assert union == [hit["_id"] for hit in hits]
        assert search(page_size, len(hits)) == []
        expected_windows = [(0, len(hits)), *[(offset, page_size)
                            for offset in range(0, len(hits), page_size)], (len(hits), page_size)]
        assert [(offset, limit) for _, _, offset, limit in requests] == expected_windows
        assert all(path == "/v1/query" for path, _, _, _ in requests)
        assert all("BRAF" in query["q"][0] for _, query, _, _ in requests)
        if extra_filter:
            assert all("clinvar" in query["q"][0] and "pathogenic" in query["q"][0]
                       for _, query, _, _ in requests)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
