"""Visible search rows recover metadata once, in one overlapping request wave.

Search scaffolding and successful recovery are synthetic. Missing-metadata
replies are unmodified upstream captures described beside the fixture bodies.
"""
from __future__ import annotations

import json
import os
import subprocess
import threading
from collections import Counter
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/article/metadata-recovery"
PMIDS = ["42894377", "42891779"]


@pytest.mark.parametrize("covered,europe_failed,pubtator_empty", [
    (False, False, False), (True, False, False), (False, True, False), (True, False, True),
])
def test_visible_metadata_recovers_each_row_once_in_one_wave(
    covered: bool, europe_failed: bool, pubtator_empty: bool, tmp_path: Path,
) -> None:
    recovery: list[str] = []
    missed: list[str] = []
    barrier = threading.Barrier(2)
    overlaps: list[bool] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path.endswith("/esearch.fcgi"):
                return self.reply({"esearchresult": {"count": "2", "idlist": PMIDS}})
            if target.path.endswith("/esummary.fcgi"):
                return self.reply({"result": {"uids": PMIDS, **{
                    pmid: {"uid": pmid, "title": f"Synthetic row {pmid}",
                           "pubdate": "2026 Oct 19"} for pmid in PMIDS
                }}})
            if target.path == "/publications/export/biocjson":
                assert params["pmids"][0] in PMIDS
                if pubtator_empty:
                    return self.reply({"PubTator3": []})
                return self.bytes((FIXTURES / "pubtator-missing.body").read_bytes(), 400)
            if target.path == "/search" and params.get("query", [""])[0].startswith("EXT_ID:"):
                pmid = params["query"][0].split()[0].removeprefix("EXT_ID:")
                recovery.append(pmid)
                try:
                    barrier.wait(timeout=2)
                    overlaps.append(True)
                except threading.BrokenBarrierError:
                    overlaps.append(False)
                if europe_failed:
                    return self.bytes(b"Authored invalid JSON control")
                if covered:
                    return self.reply({"hitCount": 1, "resultList": {"result": [{
                        "id": pmid, "pmid": pmid, "title": "Synthetic recovered metadata",
                        "abstractText": f"Recovered abstract for {pmid}", "citedByCount": 7,
                    }]}})
                return self.bytes((FIXTURES / f"europepmc-exact-{pmid}.body").read_bytes())
            if target.path == "/search":
                return self.reply({"hitCount": 0, "resultList": {"result": []}})
            if target.path == "/search/":
                return self.reply({"results": [], "count": 0, "total_pages": 0, "current": 1})
            if target.path.endswith(("/paper/search/bulk", "/paper/search")):
                return self.reply({"total": 0, "data": []})
            missed.append(self.path)
            self.send_error(404)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            size = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(size))
            self.reply([None for _ in payload["ids"]])

        def reply(self, body: object) -> None:
            self.bytes(json.dumps(body).encode())

        def bytes(self, body: bytes, status: int = 200) -> None:
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_PUBMED_BASE": base + "/entrez/eutils",
        "BIOMCP_PUBTATOR_BASE": base,
        "BIOMCP_EUROPEPMC_BASE": base,
        "BIOMCP_S2_BASE": base,
    }, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env["RUST_LOG"] = "warn"
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "--no-cache", "search", "article", "BRAF", "--limit", "2", "--full"],
            env=env, capture_output=True, text=True, check=False, timeout=30,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert result.returncode == 0, result.stderr + result.stdout
    assert not missed, missed
    assert Counter(recovery) == Counter(PMIDS), recovery
    assert overlaps == [True, True], "Recovery requests must overlap before either reply"
    payload = json.loads(result.stdout)
    rows = payload["results"]
    assert payload["complete"] is False, payload
    failures = [status for status in payload["_meta"]["source_status"]
                if status["source"] == "pubtator" and status.get("status") == "degraded"]
    assert failures, payload["_meta"]["source_status"]
    failure_cause = "internal failure" if pubtator_empty else "http_status failure (HTTP 400)"
    assert any("visible article metadata fallback" in status["message"]
               and failure_cause in status["message"] for status in failures), failures
    if europe_failed:
        assert any(status["source"] == "europepmc" and status.get("status") == "degraded"
                   and "decode failure" in status["message"]
                   for status in payload["_meta"]["source_status"]), payload["_meta"]["source_status"]
    assert [row["pmid"] for row in rows] == PMIDS
    assert [row["title"] for row in rows] == [f"Synthetic row {pmid}" for pmid in PMIDS]
    if covered:
        assert [row["citation_count"] for row in rows] == [7, 7]
        assert all(row["abstract_snippet"].startswith("Recovered abstract") for row in rows)
        assert "visible article metadata fallback" not in result.stderr
    else:
        assert [row.get("citation_count") for row in rows] == [None, None]
        assert result.stderr.count("visible article metadata fallback") == 2, result.stderr
