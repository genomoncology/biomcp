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
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
CAPTURE = REPO_ROOT / "tests/fixtures/article/pubmed-floor/akt1-e17k-esearch.json"


def test_default_variant_articles_preserves_every_plain_pubmed_pmid(tmp_path: Path) -> None:
    # Only ESearch is recorded. All other replies are authored empty or metadata
    # controls. In particular, no annotation establishes gene/variant identity.
    recorded = json.loads(CAPTURE.read_text())
    expected = recorded["esearchresult"]["idlist"]
    requests: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def reply(self, value: object, status: int = 200) -> None:
            body = json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
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
                ids = expected if term == "AKT1 E17K" else []
                start = int(query.get("retstart", ["0"])[0])
                size = int(query.get("retmax", ["25"])[0])
                return self.reply({"esearchresult": {
                    "count": str(len(ids)), "idlist": ids[start:start + size],
                }})
            if path.endswith("/esummary.fcgi"):
                ids = query["id"][0].split(",")
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
    env = isolated_command_env(tmp_path, providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    found: set[str] = set()
    try:
        for offset in range(0, len(expected), 50):
            process = subprocess.run([
                os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
                "--json", "variant", "articles", "AKT1 E17K", "--limit", "50",
                "--offset", str(offset), "--verify-identity",
            ], env=env, cwd=REPO_ROOT, capture_output=True, text=True, timeout=HANG_TIMEOUT)
            assert process.returncode in (0, 1), (process.stdout, process.stderr)
            result = json.loads(process.stdout)
            for row in result["results"]:
                found.add(row["pmid"])
                assert row.get("identity_verification", {}).get("status") != "confirmed"
        assert set(expected) <= found, {
            "missing_plain_pubmed_pmids": [pmid for pmid in expected if pmid not in found],
            "requested_pubmed_queries": sorted(set(requests)),
        }
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
