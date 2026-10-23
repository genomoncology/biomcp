"""Plain PubMed discovery remains available without variant annotations."""

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
from replay_env import REPO_ROOT, retained_cache_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
CAPTURE = REPO_ROOT / "tests/fixtures/article/pubmed-floor/akt1-e17k-esearch.json"


@pytest.mark.parametrize("case", ["normal", "missing_summary", "hard_limit", "slow_literal", "provider_resume"])
def test_default_variant_articles_preserves_every_plain_pubmed_pmid(tmp_path: Path, case: str) -> None:
    # Only ESearch is recorded. All other replies are authored empty or metadata
    # controls. In particular, no annotation establishes gene/variant identity.
    recorded = json.loads(CAPTURE.read_text())
    expected = recorded["esearchresult"]["idlist"]
    requests: list[str] = []
    peer_started = threading.Event()
    wait_failures: list[str] = []
    allow_tail = threading.Event()
    europe_starts: list[str] = []
    europe_tails: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def reply(self, value: object, status: int = 200) -> None:
            body = json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            path = target.path
            if path.endswith("/esearch.fcgi"):
                term = query["term"][0]
                requests.append(term)
                ids = expected if term == "AKT1 E17K" and case != "provider_resume" else []
                if "[Title/Abstract]" in term:
                    peer_started.set()
                if ids and case == "slow_literal" and not peer_started.wait(HANG_TIMEOUT):
                    wait_failures.append("resolution-dependent PubMed worker did not start")
                if ids:
                    assert query.get("sort") == ["relevance"]
                start = int(query.get("retstart", ["0"])[0])
                size = int(query.get("retmax", ["25"])[0])
                return self.reply({"esearchresult": {
                    "count": str(10001 if ids and case == "hard_limit" else len(ids)),
                    "idlist": ids[start:start + size],
                }})
            if path.endswith("/esummary.fcgi"):
                ids = [pmid for pmid in query["id"][0].split(",")
                       if not (case == "missing_summary" and pmid == "33123537")]
                return self.reply({"result": {"uids": ids, **{
                    pmid: {"uid": pmid, "title": "Authored metadata control",
                           "pubdate": "2020", "articleids": []}
                    for pmid in ids
                }}})
            if path in ("/v1/query", "/query"):
                return self.reply({"total": 0, "hits": []})
            if path.startswith("/v1/variant/") or path == "/allele":
                return self.reply({"error": "Authored absent identity"}, 404)
            if path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                return self.reply([])
            if path == "/search/":
                return self.reply({"results": [], "count": 0, "total_pages": 0, "current": 1})
            if path == "/search":
                term = query.get("query", [""])[0]
                if case == "provider_resume" and "TITLE_ABS" in term:
                    cursor = query.get("cursorMark", ["*"])[0]
                    if cursor == "*":
                        europe_starts.append(term)
                        pmid = "99999991"
                        continuation = {"nextCursorMark": "tail"}
                    else:
                        europe_tails.append(term)
                        if not allow_tail.is_set():
                            return self.reply({"error": "Authored unavailable tail"}, 503)
                        pmid = "99999992"
                        continuation = {}
                    return self.reply({"hitCount": 2, **continuation,
                                       "resultList": {"result": [{"id": pmid, "pmid": pmid,
                                                                  "source": "MED", "title": "Authored retained provider row"}]}})
                return self.reply({"hitCount": 0, "resultList": {"result": []}})
            if path.startswith("/graph/v1/paper/search"):
                return self.reply({"total": 0, "data": []})
            if path == "/publications/export/biocjson":
                return self.reply({"PubTator3": []})
            return self.reply({"error": "Authored unavailable source"}, 404)

        def do_POST(self) -> None:  # noqa: N802
            value = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
            self.reply([None] * len(value.get("ids", [])))

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {name: origin for name in (
        "BIOMCP_PUBTATOR_BASE", "BIOMCP_EUROPEPMC_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE", "BIOMCP_MYGENE_BASE",
        "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    )}
    providers.update({"BIOMCP_MYVARIANT_BASE": origin + "/v1",
                      "BIOMCP_PUBMED_BASE": origin + "/entrez/eutils"})
    env = retained_cache_env(tmp_path, providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    found: set[str] = set()
    try:
        initial_starts = 0
        offsets = [0, 0] if case == "provider_resume" else (
            [0] if case == "hard_limit" else range(0, len(expected), 50))
        for invocation, offset in enumerate(offsets):
            if case == "provider_resume" and invocation == 1:
                initial_starts = len(europe_starts)
                assert initial_starts > 0
                assert "99999991" in found
                assert "99999992" not in found
                allow_tail.set()
            process = subprocess.run([
                os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
                "--json", "variant", "articles", "AKT1 E17K", "--limit", "50",
                "--offset", str(offset), "--verify-identity",
            ], env=env, cwd=REPO_ROOT, capture_output=True, text=True, timeout=HANG_TIMEOUT)
            assert process.returncode in (0, 1), (process.stdout, process.stderr)
            result = json.loads(process.stdout)
            for row in result["results"]:
                found.add(row["pmid"])
                assert row["identity"]["status"] == "unverified"
                if case == "missing_summary" and row["pmid"] == "33123537":
                    assert row["metadata_status"]["status"] != "available"
        if case == "provider_resume":
            assert {"99999991", "99999992"} <= found
            assert len(europe_starts) == initial_starts, europe_starts
            assert europe_tails
            return
        assert not wait_failures, wait_failures
        if case == "slow_literal":
            assert peer_started.is_set()
        if case == "hard_limit":
            assert found
            assert not result["complete"]
            literal_status = next(status for status in result["source_status"]
                                  if status["route"] == "literal_pubmed")
            assert literal_status["status"] == "degraded"
            assert "query_result_cap" in literal_status["reason_codes"]
            assert "10000-ID" in literal_status["detail"]
            return
        assert set(expected) <= found, {
            "missing_plain_pubmed_pmids": [pmid for pmid in expected if pmid not in found],
            "requested_pubmed_queries": sorted(set(requests)),
        }
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
