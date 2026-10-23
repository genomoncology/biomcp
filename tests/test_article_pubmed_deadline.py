"""PubMed date bounds prevent historical paging from consuming its source deadline."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import REPO_ROOT, isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = pytest.mark.needs_binary


@pytest.mark.parametrize(
    "gene,keyword,bounds,held",
    [
        ("CHEK2", None, ["--date-from", "2026-09-01"], None),
        ("CDKN2A", "c.149A>C", ["--date-from", "2026-09-01"], None),
        ("CHEK2", None, ["--date-to", "2026-12-31"], None),
        ("CHEK2", None, ["--date-from", "2026-09-01", "--date-to", "2026-12-31"], None),
        ("CHEK2", None, [], None),
        ("CHEK2", None, ["--date-from", "2026-09-01"], "esearch.fcgi"),
        ("CHEK2", None, ["--date-from", "2026-09-01"], "esummary.fcgi"),
    ],
)
def test_pubmed_date_search_finishes_when_provider_answers(
    gene: str, keyword: str | None, bounds: list[str], held: str | None, tmp_path: Path,
) -> None:
    """Feature: article. Prompt PubMed replies produce rows; held replies report timeout."""
    requests: list[tuple[str, dict[str, list[str]]]] = []
    misses: list[str] = []
    release = threading.Event()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            route = target.path.rsplit("/", 1)[-1]
            if route in {"esearch.fcgi", "esummary.fcgi"}:
                requests.append((route, params))
                if route == held:
                    release.wait(HANG_TIMEOUT)
                    return
                if route == "esearch.fcgi":
                    # ESearch ignores a date range unless both endpoints are present.
                    bounded = not bounds or bool(params.get("mindate") and params.get("maxdate"))
                    body = {"esearchresult": {
                        "count": "1", "idlist": ["90001" if bounded else "90000"],
                    }}
                else:
                    pmid = params["id"][0]
                    body = {"result": {"uids": [pmid], pmid: {
                        "uid": pmid, "title": f"{gene} dated article",
                        "sortpubdate": "2026/09/22 00:00" if pmid == "90001" else "2020/01/01 00:00",
                        "fulljournalname": "Fixture journal",
                    }}}
            elif target.path == "/pubtator/entity/autocomplete/":
                body = [{"_id": f"@GENE_{gene}", "name": gene, "biotype": "gene"}]
            elif target.path == "/pubtator/search/":
                body = {"results": [], "count": 0}
            elif target.path == "/europe/search":
                body = {"hitCount": 0, "resultList": {"result": []}}
            elif target.path.startswith("/s2/"):
                body = {"data": [], "total": 0}
            else:
                misses.append(target.path)
                self.send_error(404)
                return
            self.reply(body)

        def do_POST(self) -> None:  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.reply([{"paperId": "fixture-paper", "abstract": "Fixture abstract.",
                         "citationCount": 2, "influentialCitationCount": 1}])

        def reply(self, body: object) -> None:
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            try:
                self.wfile.write(payload)
            except (BrokenPipeError, ConnectionResetError):
                pass

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
    env["BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS"] = "1500"
    args = ["-g", gene, *bounds]
    if keyword:
        args.extend(["-k", keyword])
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run([
            binary, "--json", "--no-cache", "search", "article", *args,
            "--limit", "50", "--max-per-source", "50", "--sort", "date",
        ], env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT)
    finally:
        release.set()
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    payload = json.loads(result.stdout)
    statuses = [row for row in payload.get("_meta", {}).get("source_status", [])
                if row["source"] == "pubmed"]
    if held:
        assert payload["complete"] is False, payload
        assert statuses and "timed out" in statuses[0]["message"], statuses
        assert requests[-1][0] == held, requests
    else:
        assert result.returncode == 0, result.stderr + result.stdout
        assert [row["pmid"] for row in payload["results"]] == ["90001"], payload
        assert payload["complete"] is True, payload.get("_meta", {})
        assert not statuses, statuses
        assert [route for route, _ in requests] == ["esearch.fcgi", "esummary.fcgi"]
    assert not misses, misses
    assert "fixture-only-ncbi-secret" not in result.stdout + result.stderr
    assert "fixture-only-s2-secret" not in result.stdout + result.stderr
