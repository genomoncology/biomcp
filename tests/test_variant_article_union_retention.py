"""Authored source controls reproduce a PMID skipped by changing union ranks.

The two LitVar PMIDs stay constant. A second source adds support for the
second PMID on the first union page only. No fixture asserts biological facts.
"""

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


def test_union_pages_keep_component_pmids_when_source_support_changes(tmp_path: Path) -> None:
    control = json.loads(
        (REPO_ROOT / "tests/fixtures/article/gene-aliases/authored.json").read_text()
    )
    first, second = "99999991", "99999992"
    phase = {"support": False}
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            if target.path == "/v1/query":
                body = {"total": 1, "hits": [control["hit"]]}
            elif target.path.startswith("/v1/variant/"):
                body = control["hit"]
            elif target.path == "/allele":
                body = {"errorType": "NotFound", "description": "Authored control"}
            elif target.path == "/variant/autocomplete/":
                body = [{"_id": "authored-union-control", "gene": ["ABCC8"],
                         "name": "R1437Q"}]
            elif target.path.startswith("/variant/get/"):
                body = {"pmids": [int(first), int(second)]}
            elif target.path == "/entity/autocomplete/":
                body = []
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path == "/search":
                metadata = "EXT_ID:" in query.get("query", [""])[0]
                ids = [first, second] if metadata else ([second] if phase["support"] else [])
                rows = [{"id": pmid, "pmid": pmid, "source": "MED",
                         "title": "Authored retrieval membership control", "pubYear": "2026"}
                        for pmid in ids]
                body = {"hitCount": len(rows), "resultList": {"result": rows}}
            elif target.path.endswith("/esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path in ("/graph/v1/paper/search", "/graph/v1/paper/search/bulk"):
                body = {"total": 0, "data": []}
            elif target.path == "/publications/export/biocjson":
                body = {"PubTator3": []}
            else:
                misses.append(self.path)
                self.send_error(404, "Missing authored control")
                return
            self.reply(body)

        def do_POST(self) -> None:  # noqa: N802
            body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
            self.reply([None] * len(body.get("ids", [])))

        def reply(self, body: object) -> None:
            payload = json.dumps(body).encode()
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
    providers = {key: origin for key in (
        "BIOMCP_PUBTATOR_BASE", "BIOMCP_EUROPEPMC_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE", "BIOMCP_MYGENE_BASE",
        "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    )}
    providers["BIOMCP_MYVARIANT_BASE"] = origin + "/v1"
    providers["BIOMCP_PUBMED_BASE"] = origin + "/entrez/eutils"
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    pages: dict[str, list[dict]] = {"annotation": [], "union": []}
    try:
        for strategy in pages:
            for offset in (0, 1):
                phase["support"] = strategy == "union" and offset == 0
                env = isolated_command_env(tmp_path / f"{strategy}-{offset}", providers, origin)
                env["BIOMCP_DISABLE_SOURCES"] = "mygene-info,pubmed,pubtator3"
                result = subprocess.run(
                    [binary, "--no-cache", "--json", "variant", "articles", "ABCC8 R1437Q",
                     "--strategy", strategy, "--limit", "1", "--offset", str(offset), "--debug-plan"],
                    env=env, cwd=REPO_ROOT, capture_output=True, text=True, timeout=HANG_TIMEOUT,
                )
                assert result.returncode == 0, result.stdout + result.stderr
                pages[strategy].append(json.loads(result.stdout))
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
    assert not misses, misses
    component_pmids = {row["pmid"] for page in pages["annotation"] for row in page["results"]}
    union_pmids = {row["pmid"] for page in pages["union"] for row in page["results"]}
    assert component_pmids == {first, second}, pages
    assert component_pmids <= union_pmids, {
        "missing": sorted(component_pmids - union_pmids),
        "union_pages": [[row["pmid"] for row in page["results"]] for page in pages["union"]],
        "counts": [page["debug_plan"]["counts"] for page in pages["union"]],
    }
