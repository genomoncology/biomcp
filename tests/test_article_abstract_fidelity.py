"""Article detail prefers PubMed's text while retaining PubTator annotations."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from xml.sax.saxutils import escape

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
# Short source excerpt from PMID 33311791; remaining text is synthetic.
ABSTRACT = "Elderly patients (≧65 years, n=91). Control: ≥70, ≤80, <5, >2, ±1 µg."
LOSSY = ABSTRACT.replace("≧", " ")


@pytest.mark.parametrize("pubmed", ["available", "empty", "unavailable"])
def test_article_keeps_pubmed_symbols_and_pubtator_annotations(pubmed: str, tmp_path: Path) -> None:
    requests: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            path = self.path.split("?", 1)[0]
            requests.append(path)
            if path.endswith("efetch.fcgi"):
                if pubmed == "unavailable":
                    return self.reply(b"Unavailable", 404)
                text = escape(ABSTRACT) if pubmed == "available" else ""
                return self.reply((f"<PubmedArticleSet><PubmedArticle><MedlineCitation>"
                                   f"<PMID>33311791</PMID><Article><Abstract><AbstractText>"
                                   f"{text}</AbstractText></Abstract></Article>"
                                   f"</MedlineCitation></PubmedArticle></PubmedArticleSet>").encode())
            if path == "/publications/export/biocjson":
                return self.reply(json.dumps({"PubTator3": [{
                    "pmid": 33311791, "passages": [
                        {"infons": {"type": "title"}, "text": "Source fidelity control"},
                        {"infons": {"type": "abstract"}, "text": LOSSY,
                         "annotations": [{"id": "1", "text": "cancer", "infons": {
                             "type": "Disease", "identifier": "MESH:D009369"}}]},
                    ]}]}).encode())
            if path == "/search":
                return self.reply(b'{"hitCount":0,"resultList":{"result":[]}}')
            if path.startswith("/graph/v1/paper/"):
                return self.reply(b'{"paperId":"control"}')
            self.reply(b"Unknown", 404)

        def reply(self, body: bytes, status: int = 200) -> None:
            self.send_response(status)
            self.send_header("Content-Type", "application/xml" if body.startswith(b"<") else "application/json")
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
    try:
        result = subprocess.run([binary, "--json", "--no-cache", "get", "article", "33311791"],
                                env=env, capture_output=True, text=True, timeout=30, check=False)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert result.returncode == 0, result.stderr + result.stdout
    article = json.loads(result.stdout)
    assert article["abstract_text"] == (ABSTRACT if pubmed == "available" else LOSSY)
    assert article["annotations"]["diseases"], article["annotations"]
    assert requests.count("/entrez/eutils/efetch.fcgi") == 1
