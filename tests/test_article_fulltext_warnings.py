"""Full-text route failures identify requests without disclosing credentials."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

from golden.replay_env import isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("failure", ["manifest_http", "manifest_transport", "manifest_decode", "manifest_redirect", "xml_http", "xml_decode", "figshare_http", "figshare_decode"])
def test_fulltext_route_warnings_identify_safe_requests(tmp_path: Path, failure: str) -> None:
    """Feature: article. Failed full-text routes name safe URLs and failure details."""
    sentinel = "SYNTHETIC_CREDENTIAL_SENTINEL"
    unsafe_body = "SYNTHETIC_PROVIDER_BODY_SENTINEL"

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            path = urlsplit(self.path).path
            code, content_type = 200, "application/json"
            if path == "/epmc/search":
                body = json.dumps({"hitCount": 1, "resultList": {"result": [{
                    "id": "33766116", "pmid": "33766116", "source": "MED",
                    "title": "Route warning control", "pmcid": "PMC8010000",
                }]}}).encode()
            elif path.endswith("fullTextXML") and failure not in {"xml_http", "xml_decode"}:
                content_type = "application/xml"
                body = ("<article><body><sec><title>Results</title>"
                        + "<p>Available body.</p>" * 30 + "</sec></body></article>").encode()
            elif path == "/s2/graph/v1/paper/PMID:33766116" and failure.startswith("figshare"):
                body = json.dumps({"paperId": "control", "title": "Route warning control",
                                   "openAccessPdf": {"url": "https://figshare.com/articles/journal_contribution/control/123"}}).encode()
            elif path == "/v2/articles/123" and failure.startswith("figshare"):
                body = json.dumps({"id": 123, "title": "Route warning control", "files": []}).encode()
            elif failure == "manifest_redirect" and "list-type=" in self.path:
                self.send_response(302)
                self.send_header("Location", origin + "/redirected?token=" + sentinel)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            elif path == "/redirected":
                code, body = 503, (unsafe_body + sentinel).encode()
            elif "list-type=" in self.path or ("efetch" in path and "db=pmc" in self.path) or path.endswith("fullTextXML"):
                if failure == "manifest_transport" and "list-type=" in self.path:
                    self.close_connection = True
                    return
                code = 200 if ((failure == "manifest_decode" and "list-type=" in self.path)
                               or (failure == "xml_decode" and path.endswith("fullTextXML"))) else 503
                body = (unsafe_body + sentinel).encode()
            else:
                code, body = 404, b"{}"
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self) -> None:  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            body = (unsafe_body + sentinel).encode()
            self.send_response(200 if failure == "figshare_decode" else 503)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    provider_env = {name: origin + suffix for name, suffix in (
        ("BIOMCP_EUROPEPMC_BASE", "/epmc"), ("BIOMCP_PUBTATOR_BASE", "/pubtator"),
        ("BIOMCP_S2_BASE", "/s2"), ("BIOMCP_NCBI_IDCONV_BASE", "/idconv"),
        ("BIOMCP_PMC_HTML_BASE", "/html"), ("BIOMCP_PMC_OA_BASE", "/oa"),
        ("BIOMCP_PUBMED_BASE", "/pubmed"), ("BIOMCP_FIGSHARE_BASE", ""),
    )}
    env = isolated_command_env(tmp_path, provider_env, origin)
    env.update({"NCBI_API_KEY": sentinel, "RUST_LOG": "warn"})
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        args = [binary, "--json", "--no-cache", "get", "article", "33766116", "fulltext"]
        if failure not in {"xml_http", "xml_decode"}:
            args += ["--lines", "15:22"]
        result = subprocess.run(args,
                                env=env, capture_output=True, text=True, timeout=40)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stderr + result.stdout
    if failure.startswith("figshare"):
        warning = next(line for line in result.stderr.splitlines()
                       if "Figshare sibling search request failed" in line)
        assert origin + "/v2/articles/search" in warning, warning
    else:
        warning = next(line for line in result.stderr.splitlines()
                       if "PMC OA manifest request failed" in line)
        if failure == "manifest_redirect":
            assert origin + "/redirected?token=REDACTED" in warning, warning
        else:
            assert origin + "/oa" in warning, warning
            assert "prefix=PMC8010000" in warning, warning
    expected = ("transport error" if failure == "manifest_transport" else
                "HTTP 200; response error (decode)" if failure in {"manifest_decode", "figshare_decode"} else "HTTP 503")
    assert expected in warning, warning
    assert sentinel not in result.stdout + result.stderr
    assert unsafe_body not in result.stdout + result.stderr

    if failure == "xml_http":
        note = json.loads(result.stdout)["full_text_note"]
        assert origin + "/pubmed/efetch.fcgi?db=pmc&id=8010000&rettype=xml&api_key=REDACTED" in note
        assert "HTTP 503" in note

    if failure == "xml_decode":
        note = json.loads(result.stdout)["full_text_note"]
        assert "HTTP 200; response error (decode); request " + origin + "/epmc/PMC8010000/fullTextXML" in note
