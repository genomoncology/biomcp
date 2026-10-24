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


@pytest.mark.parametrize("case", ["normal", "missing_summary", "hard_limit", "slow_literal", "slow_provider", "provider_resume", "semantic_resume", "literal_resume", "strict_missing_summary", "strict_failed_summary", "pubtator_resume", "litvar_resume"])
def test_default_variant_articles_preserves_every_plain_pubmed_pmid(tmp_path: Path, case: str) -> None:
    # Only ESearch is recorded. All other replies are authored empty or metadata
    # controls. In particular, no annotation establishes gene/variant identity.
    recorded = json.loads(CAPTURE.read_text())
    expected = recorded["esearchresult"]["idlist"]
    requests: list[str] = []
    peer_started = threading.Event()
    slow_provider_ready = threading.Event()
    slow_provider_requests = 0
    slow_provider_lock = threading.Lock()
    wait_failures: list[str] = []
    allow_tail = threading.Event()
    provider_starts: list[str] = []
    provider_tails: list[str] = []
    literal_offsets: list[int] = []
    litvar_calls: list[str] = []
    pubtator_lookups: list[str] = []
    pubtator_empty_queries: list[str] = []
    source_case = case in ("provider_resume", "semantic_resume", "pubtator_resume", "litvar_resume")
    strict_metadata = case in ("strict_missing_summary", "strict_failed_summary")
    pubtator_pmids = [str(99999001 + index) for index in range(1251)]

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
            nonlocal slow_provider_requests
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            path = target.path
            if path.endswith("/esearch.fcgi"):
                term = query["term"][0]
                requests.append(term)
                if case == "slow_provider":
                    with slow_provider_lock:
                        slow_provider_requests += 1
                        if slow_provider_requests >= 2:
                            slow_provider_ready.set()
                    if not peer_started.wait(HANG_TIMEOUT):
                        wait_failures.append("fast Europe PMC worker did not start while PubMed plans were blocked")
                ids = expected if term == "AKT1 E17K" and not source_case and not strict_metadata else []
                if strict_metadata and "[Title/Abstract]" in term:
                    ids = ["99999991"]
                if "[Title/Abstract]" in term:
                    peer_started.set()
                if ids and case == "slow_literal" and not peer_started.wait(HANG_TIMEOUT):
                    wait_failures.append("resolution-dependent PubMed worker did not start")
                if ids and term == "AKT1 E17K":
                    assert query.get("sort") == ["relevance"]
                start = int(query.get("retstart", ["0"])[0])
                size = int(query.get("retmax", ["25"])[0])
                if ids:
                    literal_offsets.append(start)
                    if case == "literal_resume" and start > 0 and not allow_tail.is_set():
                        return self.reply({"error": "Authored unavailable literal tail"}, 503)
                return self.reply({"esearchresult": {
                    "count": str(10001 if ids and case == "hard_limit" else len(ids)),
                    "idlist": ids[start:start + size],
                }})
            if path.endswith("/esummary.fcgi"):
                if case == "strict_failed_summary":
                    return self.reply({"error": "Authored missing summary service"}, 503)
                ids = [pmid for pmid in query["id"][0].split(",")
                       if not (case == "missing_summary" and pmid == "33123537")]
                if case == "strict_missing_summary":
                    ids = []
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
                if case == "pubtator_resume" and path == "/entity/autocomplete/":
                    pubtator_lookups.append(self.path)
                    return self.reply([{"_id": "@VARIANT_p.E17K_AKT1_human", "biotype": "variant", "name": "E17K"}])
                if case == "litvar_resume" and path == "/variant/autocomplete/":
                    litvar_calls.append(path)
                    return self.reply([{"_id": f"authored-{index}", "gene": ["AKT1"], "name": "E17K"}
                                       for index in range(1, 4)])
                return self.reply([])
            if path.startswith("/variant/get/") and case == "litvar_resume":
                litvar_calls.append(path)
                index = int(path.split("authored-")[1].split("/")[0])
                pmids = [99998000 + index]
                if index == 1 and allow_tail.is_set():
                    pmids.append(99998004)
                return self.reply({"pmids": pmids, "pmids_count": 2 if index == 1 else 1})
            if path == "/search/":
                if case == "pubtator_resume" and query.get("text", [""])[0].startswith("@VARIANT_"):
                    page = int(query.get("page", ["1"])[0])
                    if page == 1:
                        provider_starts.append("pubtator")
                    else:
                        provider_tails.append(str(page))
                    ids = pubtator_pmids[(page - 1) * 25:page * 25]
                    return self.reply({"results": [{"_id": pmid, "pmid": int(pmid),
                                                    "title": "Authored retained provider row"} for pmid in ids],
                                       "count": len(pubtator_pmids), "total_pages": 51,
                                       "current": page, "page_size": 25})
                if case == "pubtator_resume":
                    pubtator_empty_queries.append(self.path)
                return self.reply({"results": [], "count": 0, "total_pages": 0, "current": 1})
            if path == "/search":
                if case == "slow_provider":
                    if not slow_provider_ready.wait(HANG_TIMEOUT):
                        wait_failures.append("two PubMed workers were not admitted")
                    peer_started.set()
                term = query.get("query", [""])[0]
                if case == "provider_resume" and "TITLE_ABS" in term:
                    cursor = query.get("cursorMark", ["*"])[0]
                    if cursor == "*":
                        provider_starts.append(term)
                        pmid = "99999991"
                        continuation = {"nextCursorMark": "tail"}
                    else:
                        provider_tails.append(term)
                        if not allow_tail.is_set():
                            return self.reply({"error": "Authored unavailable tail"}, 503)
                        pmid = "99999992"
                        continuation = {}
                    return self.reply({"hitCount": 2, **continuation,
                                       "resultList": {"result": [{"id": pmid, "pmid": pmid,
                                                                  "source": "MED", "title": "Authored retained provider row"}]}})
                return self.reply({"hitCount": 0, "resultList": {"result": []}})
            if path.startswith("/graph/v1/paper/search"):
                if case == "semantic_resume" and path.endswith("/bulk"):
                    term = query.get("query", [""])[0]
                    if query.get("token") == ["tail"]:
                        provider_tails.append(term)
                        if not allow_tail.is_set():
                            return self.reply({"error": "Authored unavailable semantic tail"}, 503)
                        pmid = "99999992"
                        continuation = {}
                    else:
                        provider_starts.append(term)
                        pmid = "99999991"
                        continuation = {"token": "tail"}
                    return self.reply({"total": 2, **continuation, "data": [{
                        "paperId": pmid, "externalIds": {"PubMed": pmid},
                        "title": "Authored retained provider row"}]})
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
        offsets = ([0, 0, 50, 100, 150] if case == "literal_resume" else (
            [0, 0, 1250] if case == "pubtator_resume" else ([0, 0] if source_case else (
                [0] if case == "hard_limit" or strict_metadata else range(0, len(expected), 50)))))
        for invocation, offset in enumerate(offsets):
            if case == "literal_resume" and invocation == 1:
                assert found
                literal_status = next(status for status in result["source_status"]
                                      if status["route"] == "literal_pubmed")
                assert literal_status["status"] == "degraded"
                assert not result["complete"]
                allow_tail.set()
            if source_case and invocation == 1:
                initial_starts = len(provider_starts)
                if case == "pubtator_resume":
                    assert set(pubtator_pmids[:50]) <= found
                    # Autocomplete and the empty fallback search share the
                    # existing 51-unit PubTator allowance with 49 native pages.
                    assert provider_tails == [str(page) for page in range(2, 50)]
                    assert not result["complete"]
                    provider_statuses = [status for status in result["source_status"]
                                         if status["source"] == "pubtator"]
                    assert len(pubtator_lookups) == 1, pubtator_lookups
                    assert len(pubtator_empty_queries) == 1, pubtator_empty_queries
                    assert all(status["work"]["timed_out"] == 0 for status in provider_statuses), provider_statuses
                    assert any(status["work"]["not_attempted"] > 0 for status in provider_statuses), provider_statuses
                elif case == "litvar_resume":
                    assert {"99998001", "99998002"} <= found
                    assert "99998003" not in found
                else:
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
        if case == "literal_resume":
            assert literal_offsets.count(0) == 1, literal_offsets
        if strict_metadata:
            row = next(row for row in result["results"] if row["pmid"] == "99999991")
            assert row["identity"]["status"] == "unverified"
            assert row["metadata_status"]["status"] != "available"
            return
        if source_case:
            wanted = ({pubtator_pmids[-1]} if case == "pubtator_resume" else (
                {"99998001", "99998002", "99998003", "99998004"} if case == "litvar_resume" else {"99999991", "99999992"}))
            assert wanted <= {row["pmid"] for row in result["results"]}
            assert len(provider_starts) == initial_starts, provider_starts
            if case == "litvar_resume":
                assert litvar_calls.count("/variant/autocomplete/") == 1
                for index in range(1, 4):
                    assert litvar_calls.count(f"/variant/get/authored-{index}/publications") == (2 if index == 1 else 1)
            elif case == "pubtator_resume":
                assert provider_tails == [str(page) for page in range(2, 52)]
            else:
                assert provider_tails
            return
        assert not wait_failures, wait_failures
        if case in ("slow_literal", "slow_provider"):
            assert peer_started.is_set()
        if case == "slow_provider":
            assert slow_provider_requests > 2
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
