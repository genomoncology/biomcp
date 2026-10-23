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


@pytest.mark.parametrize("transport", [False, True])
def test_fulltext_route_warnings_identify_safe_requests(tmp_path: Path, transport: bool) -> None:
    """Feature: article. Failed full-text routes name safe URLs and failure details."""
    sentinel = "SYNTHETIC_CREDENTIAL_SENTINEL"
    unsafe_body = "SYNTHETIC_PROVIDER_BODY_SENTINEL"

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            path = urlsplit(self.path).path
            code, content_type = 200, "application/json"
            if path == "/search":
                body = json.dumps({"hitCount": 1, "resultList": {"result": [{
                    "id": "33766116", "pmid": "33766116", "source": "MED",
                    "title": "Route warning control", "pmcid": "PMC8010000",
                }]}}).encode()
            elif path.endswith("fullTextXML"):
                content_type = "application/xml"
                body = ("<article><body><sec><title>Results</title>"
                        + "<p>Available body.</p>" * 30 + "</sec></body></article>").encode()
            elif "list-type=" in self.path or "efetch" in path:
                if transport and "list-type=" in self.path:
                    self.close_connection = True
                    return
                code, body = 503, (unsafe_body + sentinel).encode()
            else:
                code, body = 404, b"{}"
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {name: origin for name in (
        "BIOMCP_EUROPEPMC_BASE", "BIOMCP_PUBTATOR_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_NCBI_IDCONV_BASE", "BIOMCP_PMC_HTML_BASE", "BIOMCP_PMC_OA_BASE",
        "BIOMCP_PUBMED_BASE",
    )}, origin)
    env.update({"NCBI_API_KEY": sentinel, "RUST_LOG": "warn"})
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run([binary, "--json", "--no-cache", "get", "article",
                                 "33766116", "fulltext", "--lines", "15:22"],
                                env=env, capture_output=True, text=True, timeout=40)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stderr + result.stdout
    warning = next(line for line in result.stderr.splitlines()
                   if "PMC OA manifest request failed" in line)
    assert origin in warning, warning
    assert "prefix=PMC8010000" in warning, warning
    assert ("transport error" if transport else "HTTP 503") in warning, warning
    assert sentinel not in result.stdout + result.stderr
    assert unsafe_body not in result.stdout + result.stderr
