"""Article full text reaches healthy routes despite stalled providers."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

from golden.replay_env import isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("held", ["pubtator", "semantic_scholar", "assets", "fulltext"])
def test_article_fulltext_bounds_stalled_routes(held: str, tmp_path: Path) -> None:
    """Feature: article. Stalled routes cannot hide available full text."""
    release = threading.Event()
    requests: list[tuple[str, float]] = []
    started = time.monotonic()

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            path = urlsplit(self.path).path
            requests.append((path, time.monotonic() - started))
            if ((held == "pubtator" and "biocjson" in path)
                    or (held == "semantic_scholar" and "/graph/" in path)
                    or (held in {"assets", "fulltext"} and path != "/search"
                        and not (held == "assets" and path.endswith("fullTextXML"))
                        and "biocjson" not in path and "/graph/" not in path
                        and not ("efetch" in path and "db=pubmed" in self.path))):
                release.wait()
                return
            code, content_type = 200, "application/json"
            if path == "/search":
                body = json.dumps({"hitCount": 1, "resultList": {"result": [{
                    "id": "40966517", "pmid": "40966517", "source": "MED",
                    "title": "Article route deadline control", "pmcid": "PMC12448885",
                }]}}).encode()
            elif path.endswith("fullTextXML"):
                content_type = "application/xml"
                body = b"<article><body><sec><title>Results</title><p>Available Europe PMC body.</p></sec></body></article>"
            elif "/graph/" in path:
                body = b'{"paperId":"control","title":"Control"}'
            else:
                code, body = 404, b'{}'
            self.send_response(code)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            try:
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

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
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env.pop("RUST_LOG", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run([binary, "--json", "--no-cache", "get", "article",
                                 "40966517", "fulltext"], env=env, capture_output=True,
                                text=True, timeout=40)
    except subprocess.TimeoutExpired:
        pytest.fail(f"Article command exceeded its bound; route times: {requests}")
    finally:
        release.set()
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stderr + result.stdout
    payload = json.loads(result.stdout)
    if held == "fulltext":
        assert payload["full_text_coverage"]["coverage"] == "unavailable", payload
        attempts = payload["full_text_coverage"]["attempts"]
        assert len(attempts) >= 4, (attempts, requests)
        failures = [attempt for attempt in attempts if attempt["reason"] == "source_unavailable"]
        assert len(failures) == 4, (attempts, requests)
        # The MED fallback has no fullTextXML resource and makes no network request.
        assert sum(attempt["reason"] == "no_content" for attempt in attempts) == 1, attempts
        assert "timed out" in payload["full_text_note"], payload
        for route in ("Europe PMC PMC XML", "NCBI EFetch PMC XML",
                      "PMC Open Access archive", "PMC HTML"):
            assert route in payload["full_text_note"], payload
    else:
        assert payload["full_text_coverage"]["coverage"] == "full_text", payload
        assert "Available Europe PMC body" in Path(payload["full_text_path"]).read_text()
        if held == "assets":
            assert "article asset summary timed out" in result.stderr, result.stderr
