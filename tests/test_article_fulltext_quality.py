"""Saved fulltext headings determine the section quality flag."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("heading", [True, False], ids=["headings", "no-headings"])
def test_fulltext_section_quality_agrees_with_saved_sections(heading: bool, tmp_path: Path) -> None:
    identifier = "28736637"
    missed: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            if target.path == "/search":
                record = {"id": identifier, "pmid": identifier, "source": "MED",
                          "title": "Authored section control", "pmcid": "PMC5506279"}
                return self.reply({"hitCount": 1, "resultList": {"result": [record]}})
            if target.path == "/publications/export/biocjson":
                return self.reply({"error": "Authored PubTator lag control"}, 404)
            if target.path.startswith("/graph/v1/paper/"):
                return self.reply({"paperId": "control", "title": "Control"})
            if target.path == "/idconv/":
                return self.reply({"records": [{"status": "error", "errmsg": "Identifier not found in PMC"}]})
            if target.path.endswith("/fullTextXML"):
                return self.reply({}, 404)
            if target.path == "/articles/PMC5506279/":
                title = "<h2>Authored section</h2>" if heading else ""
                paragraphs = "<p>Authored body paragraph with enough text for fulltext extraction.</p>" * 8
                body = f"<html><body><main><article>{title}{paragraphs}</article></main></body></html>".encode()
                return self.body(body, 200, "text/html")
            if target.path == "/":
                return self.body(b"<ListBucketResult></ListBucketResult>", 200, "application/xml")
            if target.path.endswith("/efetch.fcgi") or target.path.startswith("/PMC5506279"):
                return self.reply({}, 404)
            missed.append(self.path)
            self.reply({}, 404)

        def reply(self, payload: object, code: int = 200) -> None:
            self.body(json.dumps(payload).encode(), code, "application/json")

        def body(self, payload: bytes, code: int, content_type: str) -> None:
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_EUROPEPMC_BASE": origin,
        "BIOMCP_PUBTATOR_BASE": origin,
        "BIOMCP_S2_BASE": origin,
        "BIOMCP_NCBI_IDCONV_BASE": origin + "/idconv/",
        "BIOMCP_PMC_HTML_BASE": origin,
        "BIOMCP_PMC_OA_BASE": origin,
        "BIOMCP_PUBMED_BASE": origin,
    }, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    command = ["get", "article", identifier, "fulltext"]
    try:
        result = subprocess.run([binary, "--json", "--no-cache", *command],
                                env=env, capture_output=True, text=True, timeout=60, check=False)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert result.returncode == 0, result.stderr + result.stdout
    assert not missed, missed
    payload = json.loads(result.stdout)
    article = payload
    assert article["full_text_manifest"]["source_kind"] == "pmc_html"
    count = article["full_text_summary"]["section_count"]
    assert (count > 0) == heading, article
    assert article["full_text_manifest"]["quality"]["has_sections"] == (count > 0), article
