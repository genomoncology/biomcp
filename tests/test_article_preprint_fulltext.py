"""A preprint returns JATS text or a named failure without destroying a batch."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/article/preprint-fulltext"
DOI = "10.1101/2022.07.18.22276644"
PMIDS = ["14735200", "25922291", "18178632", "18509008", "12752119", "20301425"]
JATS = "/content/early/2022/07/22/2022.07.18.22276644.source.xml"


@pytest.mark.parametrize("batch,status", [
    pytest.param(False, 200, id="single-preprint-text"),
    pytest.param(True, 200, id="batch-preprint-text"),
    pytest.param(True, 404, id="batch-jats-absent"),
    pytest.param(True, 503, id="batch-jats-source-unavailable"),
    pytest.param(True, 0, id="batch-reject-untrusted-advertised-url"),
    pytest.param(False, -1, id="identity-only-empty-result-credits-id-converter"),
])
def test_preprint_text_and_named_failures_preserve_every_batch_item(
    batch: bool, status: int, tmp_path: Path,
) -> None:
    identifier = "10.1000/identity-control" if status == -1 else DOI
    requests: list[str] = []
    missed: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            requests.append(target.path)
            if target.path == "/search":
                query = params["query"][0]
                record = {"id": "PPR521666", "source": "PPR", "doi": identifier,
                          "title": "Preprint control", "pubType": "preprint"}
                if query.startswith("EXT_ID:"):
                    pmid = query.split()[0].removeprefix("EXT_ID:")
                    record = {"id": pmid, "pmid": pmid, "source": "MED",
                              "title": f"Synthetic article {pmid}"}
                return self.reply({"hitCount": 1, "resultList": {"result": [record]}})
            if target.path == "/publications/export/biocjson":
                return self.reply({"error": "Authored PubTator lag control"}, 404)
            if target.path.startswith("/graph/v1/paper/"):
                return self.reply({"paperId": "control", "title": "Control"})
            if target.path == "/idconv/":
                return self.reply({"records": [{"status": "error", "errmsg": "Identifier not found in PMC"}]})
            if target.path.endswith("/fullTextXML"):
                return self.reply({}, 404)
            if target.path == f"/details/medrxiv/{DOI}":
                return self.reply({"messages": [{"status": "ok"}], "collection": [{
                    "doi": DOI, "version": "1", "server": "medRxiv", "license": "cc_by",
                    "jatsxml": ("https://example.invalid" if status == 0 else "https://www.medrxiv.org") + JATS,
                }]})
            if target.path == JATS:
                body = (FIXTURES / "article.xml").read_bytes() if status == 200 else b"Authored source failure"
                return self.body(body, status, "application/xml")
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
        "BIOMCP_PREPRINT_FIXTURE_ORIGIN": origin,
    }, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    command = ["batch", "article", ",".join([*PMIDS, identifier]), "--mode", "detail", "--sections", "fulltext"] if batch else ["get", "article", identifier, "fulltext"]
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
    if batch:
        rows = payload["items"]
        assert [row["input"] for row in rows] == [*PMIDS, identifier]
        assert all(row["status"] == "ok" for row in rows)
        article = rows[-1]["result"]
    else:
        article = payload
    if status == -1:
        assert "NCBI ID Converter" in result.stdout
        assert not article.get("full_text_path")
        assert not any(path.startswith("/details/") for path in requests)
        return
    assert f"/details/medrxiv/{DOI}" in requests
    assert (JATS in requests) == (status != 0)
    if status == 200:
        text = Path(article["full_text_path"]).read_text()
        assert "Germline mutation" in text and "Orcadian" in text
        assert article["full_text_source"]["source"] == "bioRxiv/medRxiv"
        manifest = article["full_text_manifest"]
        assert manifest["source_identifier"] == DOI
        assert manifest["reuse"]["license"] == "cc_by"
        assert manifest["provenance"]["package_url"] == "https://www.medrxiv.org" + JATS
    else:
        assert not article.get("full_text_path")
        assert article["full_text_note"]
        attempts = article["full_text_coverage"]["attempts"]
        assert attempts[-1]["reason"] == ("no_content" if status == 404 else "source_unavailable")
