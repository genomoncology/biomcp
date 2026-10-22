"""Dated article searches retain the gene constraint and bounded provider work."""
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

pytestmark = [pytest.mark.needs_binary]
CAPTURES = REPO_ROOT / "tests/fixtures/article/pubtator-dated-query"
QUERY = "@GENE_CHEK2 AND (G167R OR Gly167Arg)"


@pytest.mark.parametrize("held", ["none", "search", "later_page"])
def test_dated_gene_keyword_search_bounds_pubtator_and_retains_other_sources(
    held: str, tmp_path: Path,
) -> None:
    """Feature: article. A precise dated query cannot scan all gene papers."""
    requests: list[dict[str, list[str]]] = []
    release = threading.Event()
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path == "/pubtator/entity/autocomplete/":
                body = json.loads((CAPTURES / "autocomplete.json").read_text())
            elif target.path == "/pubtator/search/":
                requests.append(params)
                if held == "search" or ((held == "later_page" or params["text"] != [QUERY])
                                         and params["page"] != ["1"]):
                    release.wait(HANG_TIMEOUT)
                    return
                name = "grouped" if params["text"] == [QUERY] else "ungrouped"
                body = json.loads((CAPTURES / f"{name}.json").read_text())
                if held == "later_page":
                    body["count"] = 26
                    body["total_pages"] = 2
                    body["results"][0]["date"] = "2026-09-22"
            elif target.path == "/europe/search":
                body = {"hitCount": 1, "resultList": {"result": [{
                    "id": "42800083", "pmid": "42800083", "title": "Other source dated paper",
                    "firstPublicationDate": "2026-09-22", "abstractText": "Recorded companion row",
                    "citedByCount": 2,
                }]}}
            elif target.path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path.startswith("/s2/"):
                body = {"data": [], "total": 0}
            else:
                misses.append(self.path)
                self.send_error(404)
                return
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            try:
                self.wfile.write(payload)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def do_POST(self) -> None:  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            payload = b'[null]'
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_PUBTATOR_BASE": origin + "/pubtator",
        "BIOMCP_EUROPEPMC_BASE": origin + "/europe",
        "BIOMCP_PUBMED_BASE": origin + "/pubmed",
        "BIOMCP_S2_BASE": origin + "/s2",
    }, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env["NCBI_API_KEY"] = "fixture-only-ncbi-secret"
    env["S2_API_KEY"] = "fixture-only-s2-secret"
    env["BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS"] = "2000"
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run([
            binary, "--json", "--no-cache", "search", "article", "-g", "CHEK2",
            "-k", "G167R OR Gly167Arg", "--date-from", "2026-09-01", "--limit", "50",
            "--max-per-source", "50", "--debug-plan",
        ], env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT)
    finally:
        release.set()
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stderr + result.stdout
    payload = json.loads(result.stdout)
    assert any(row["pmid"] == "42800083" for row in payload["results"]), payload
    assert requests and all(request["text"] == [QUERY] for request in requests), requests
    statuses = [status for status in payload.get("_meta", {}).get("source_status", [])
                if status["source"] == "pubtator"]
    if held != "none":
        assert payload["complete"] is False, payload
        assert statuses and "timeout" in statuses[0]["message"], statuses
        message = statuses[0]["message"]
        assert "request=GET /pubtator/" in message, message
        assert QUERY in message and '"size":"25"' in message, message
        assert f'"page":"{2 if held == "later_page" else 1}"' in message, message
        if held == "later_page":
            assert any(row["pmid"] == "36061833" for row in payload["results"]), payload
    else:
        assert len(requests) == 1, requests
        assert payload["complete"] is True, payload
        assert not statuses, statuses
    assert not misses, misses
    assert "fixture-only-ncbi-secret" not in result.stdout + result.stderr
    assert "fixture-only-s2-secret" not in result.stdout + result.stderr
