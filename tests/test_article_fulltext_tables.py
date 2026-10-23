"""Article line reads preserve tables from fallback source XML."""
from __future__ import annotations

from hang_guard import HANG_TIMEOUT

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit
from xml.etree import ElementTree

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("placement", ["back", "body", "floats-group", "abstract"])
def test_fulltext_lines_keep_back_matter_tables_from_ncbi_fallback(
    placement: str, tmp_path: Path,
) -> None:
    """Feature: article. Source tables remain searchable in returned lines."""
    identifier = "20516115"
    missed: list[str] = []
    source = (ROOT / "testdata/sources/ncbi_efetch/pmc3040717.xml").read_bytes()
    if placement != "back":
        # Authored placements check that the final pass does not repeat tables
        # rendered in place, including source tables without optional IDs.
        document = ElementTree.fromstring(source)
        parents = {child: parent for parent in document.iter() for child in parent}
        article = document.find("article")
        assert article is not None
        destination = (
            ElementTree.SubElement(article, placement) if placement == "floats-group"
            else article.find("front/article-meta/abstract" if placement == "abstract" else "body")
        )
        assert destination is not None
        for table in list(document.iter("table-wrap")):
            parents[table].remove(table)
            table.attrib.pop("id", None)
            destination.append(table)
        source = ElementTree.tostring(article)

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            if target.path == "/search":
                record = {"id": identifier, "pmid": identifier, "source": "MED",
                          "title": "Captured fallback table control", "pmcid": "PMC3040717"}
                return self.reply({"hitCount": 1, "resultList": {"result": [record]}})
            if target.path == "/publications/export/biocjson":
                return self.reply({"error": "Authored PubTator lag control"}, 404)
            if target.path.startswith("/graph/v1/paper/"):
                return self.reply({"paperId": "control", "title": "Control"})
            if target.path == "/idconv/":
                return self.reply({"records": [{"status": "error", "errmsg": "Identifier not found in PMC"}]})
            if target.path.endswith("/fullTextXML"):
                return self.reply({}, 404)
            if target.path.endswith("/efetch.fcgi"):
                return self.body(source, 200, "application/xml")
            if target.path == "/":
                return self.body(b"<ListBucketResult></ListBucketResult>", 200, "application/xml")
            if target.path == "/articles/PMC3040717/" or target.path.startswith("/PMC3040717"):
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
    command = ["get", "article", identifier, "fulltext", "--lines", "1:500"]
    try:
        result = subprocess.run([binary, "--json", "--no-cache", *command],
                                env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT, check=False)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stderr + result.stdout
    assert not missed, missed
    payload = json.loads(result.stdout)
    assert payload["truncated"] is False, payload
    assert payload["text"].count("**Table 1.") == 1
    assert payload["text"].count("**Table 2.") == 1
    assert "P1749R" in payload["text"], "Table 2 source token is missing from full-text lines"
    assert "|" in next(line for line in payload["text"].splitlines() if "P1749R" in line)
