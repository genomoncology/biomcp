"""Article continuations retain a session and refuse an unavailable pool."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

from hang_guard import HANG_TIMEOUT

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, retained_cache_env  # noqa: E402

pytestmark = pytest.mark.needs_binary


def test_article_tokens_bind_the_pool_and_request_configuration(tmp_path):
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            path = urlsplit(self.path).path
            if path == "/search":
                body = {"hitCount": 3, "resultList": {"result": [
                    {"id": str(pmid), "source": "MED", "pmid": str(pmid),
                     "title": f"BRAF V600E paper {pmid}", "pubYear": "2020"}
                    for pmid in (101, 102, 103)]}}
            elif path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"count": "0", "idlist": []}}
            elif path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif path.endswith("/search/bulk"):
                body = {"total": 0, "data": []}
            else:
                self.send_error(404)
                return
            self.reply(body)

        def do_POST(self):  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.reply([None, None, None])

        def reply(self, body):
            data = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = retained_cache_env(tmp_path, {
        name: origin for name in ("BIOMCP_EUROPEPMC_BASE", "BIOMCP_PUBMED_BASE",
                                 "BIOMCP_PUBTATOR_BASE", "BIOMCP_S2_BASE")}, origin)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))

    def invoke(*args, command_env=None):
        return subprocess.run(
            [binary, "--json", "variant", "articles", "BRAF V600E", "--strategy", "exact",
             "--limit", "1", *args], env=command_env or env, capture_output=True,
            text=True, timeout=HANG_TIMEOUT,
        )

    try:
        first = invoke()
        assert first.returncode == 0, first.stdout + first.stderr
        first = json.loads(first.stdout)
        token = first["pagination"]["next_page_token"]
        assert token, "every stored article session supplies its bound continuation"
        seen = [row["pmid"] for row in first["results"]]
        while first["pagination"]["has_more"]:
            following = invoke("--page-token", token)
            assert following.returncode == 0, following.stdout + following.stderr
            first = json.loads(following.stdout)
            assert first["pagination"]["total"] == 3
            seen.extend(row["pmid"] for row in first["results"])
            token = first["pagination"]["next_page_token"]
        assert len(seen) == len(set(seen)) == 3
        empty = invoke("--page-token", token)
        assert empty.returncode == 0, empty.stdout + empty.stderr
        assert json.loads(empty.stdout)["results"] == []
        assert json.loads(empty.stdout)["pagination"]["has_more"] is False
        conflict = invoke("--page-token", token, "--verify-identity")
        assert conflict.returncode != 0
        assert "conflict" in (conflict.stdout + conflict.stderr).lower()
        bypass = dict(env, BIOMCP_CACHE_MODE="off")
        expired = invoke("--page-token", token, command_env=bypass)
        assert expired.returncode != 0
        assert "offset 0" in expired.stdout + expired.stderr
        missing = invoke("--offset", "1", command_env=bypass)
        assert missing.returncode != 0
        assert "offset 0" in missing.stdout + missing.stderr
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
