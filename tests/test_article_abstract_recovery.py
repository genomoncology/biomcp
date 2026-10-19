"""Synthetic source replies exercise abstract recovery through the shipped CLI."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit
from xml.sax.saxutils import escape

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
ABSTRACT = "Synthetic cancer cohort: ≥65 years; ≤0.05; <5, >2; 2·5 ±0.1."


@pytest.mark.parametrize("source,surface", [
    ("europepmc", "search"), ("europepmc", "get"), ("pubmed", "search"),
    ("pubmed", "get"),
])
def test_article_recovers_source_abstract(source: str, surface: str, tmp_path: Path) -> None:
    pmids = ["36780833", "42423773"] if source == "europepmc" else ["22890969", "41790359"]
    fetches: list[list[str]] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path.endswith("esearch.fcgi"):
                return self.reply({"esearchresult": {"count": "2", "idlist": pmids}})
            if target.path.endswith("esummary.fcgi"):
                return self.reply({"result": {"uids": pmids, **{
                    p: {"uid": p, "title": f"Synthetic cancer cohort {p}", "pubdate": "2026"}
                    for p in pmids
                }}})
            if target.path.endswith("efetch.fcgi"):
                ids = params["id"][0].split(",")
                fetches.append(ids)
                articles = "".join(
                    f"<PubmedArticle><MedlineCitation><PMID>{p}</PMID><Article>"
                    f"<Abstract><AbstractText>{escape(ABSTRACT) if source == 'pubmed' else ''}"
                    "</AbstractText></Abstract></Article></MedlineCitation></PubmedArticle>"
                    for p in ids
                )
                return self.bytes(f"<PubmedArticleSet>{articles}</PubmedArticleSet>".encode(), "application/xml")
            if target.path == "/search":
                if source != "europepmc":
                    return self.reply({"hitCount": 0, "resultList": {"result": []}})
                query = params.get("query", [""])[0]
                ids = [p for p in pmids if p in query] if "EXT_ID:" in query else pmids
                return self.reply({"hitCount": len(ids), "resultList": {"result": [
                    {"id": p, "pmid": p, "source": "MED", "title": f"Synthetic cancer cohort {p}",
                     "abstractText": ABSTRACT if params.get("resultType") == ["core"] else None,
                     "journalInfo": {"journal": {"title": "Synthetic Oncology Journal"}}}
                    for p in ids
                ]}})
            if target.path == "/publications/export/biocjson":
                pmid = params["pmids"][0]
                return self.reply({"PubTator3": [{"pmid": int(pmid), "passages": [
                    {"infons": {"type": "title"}, "text": f"Synthetic cancer cohort {pmid}"},
                    {"infons": {"type": "abstract"}, "text": "   "},
                ]}]})
            if target.path.startswith("/graph/v1/paper/"):
                return self.reply({"paperId": "synthetic"})
            self.send_error(404)

        def do_POST(self) -> None:  # noqa: N802
            payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            self.reply([None for _ in payload["ids"]])

        def reply(self, body: object) -> None:
            self.bytes(json.dumps(body).encode(), "application/json")

        def bytes(self, body: bytes, content_type: str) -> None:
            self.send_response(200)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_PUBMED_BASE": origin + "/entrez/eutils", "BIOMCP_PUBTATOR_BASE": origin,
        "BIOMCP_EUROPEPMC_BASE": origin, "BIOMCP_S2_BASE": origin,
    }, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    args = (["search", "article", "cancer", "--source", source, "--limit", "2", "--full"]
            if surface == "search" else ["get", "article", pmids[-1]])
    try:
        result = subprocess.run([binary, "--json", "--no-cache", *args], env=env,
                                capture_output=True, text=True, timeout=30, check=False)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert result.returncode == 0, result.stderr + result.stdout
    reply = json.loads(result.stdout)
    if surface == "search":
        assert len(reply["results"]) == 2
        assert all(row.get("abstract_snippet") == ABSTRACT for row in reply["results"]), reply
        if source == "europepmc":
            assert all(row["journal"] == "Synthetic Oncology Journal" for row in reply["results"])
        if source == "pubmed":
            assert fetches == [pmids], "Recover the visible abstracts in one PubMed request"
    else:
        assert reply.get("abstract_text") == ABSTRACT
        if source == "europepmc":
            assert reply["journal"] == "Synthetic Oncology Journal"
