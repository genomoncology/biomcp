"""Plain article searches resolve an rsID through PubTator's typed entity."""

from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

pytestmark = [pytest.mark.needs_binary]
REPO_ROOT = Path(__file__).resolve().parents[1]
CAPTURE = REPO_ROOT / "testdata/sources/pubtator/autocomplete_rsid_rs113488022.json"
ENTITY = "@VARIANT_p.V600E_BRAF_human"


@pytest.mark.parametrize(
    "keyword,reply_kind,expected_query",
    [
        ("rs113488022", "matching", ENTITY),
        ("rs113488022", "empty", "rs113488022"),
        ("rs113488022", "mismatch", "rs113488022"),
        ("melanoma", "matching", "melanoma"),
    ],
)
def test_plain_article_search_resolves_only_a_matching_rsid_entity(
    keyword: str,
    reply_kind: str,
    expected_query: str,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    autocomplete = json.loads(CAPTURE.read_text())
    if reply_kind == "empty":
        autocomplete = []
    elif reply_kind == "mismatch":
        autocomplete[0]["db_id"] = "rs121913530##"
    requests: list[tuple[str, dict[str, list[str]]]] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            requests.append((target.path, params))
            if target.path == "/entity/autocomplete/":
                body = autocomplete
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            else:
                self.send_error(404, "No fixture for request")
                return
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    monkeypatch.setenv("BIOMCP_PUBTATOR_BASE", base)
    monkeypatch.setenv("BIOMCP_TEST_UNPACED_ORIGIN", base)
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.delenv("BIOMCP_DISABLE_SOURCES", raising=False)
    monkeypatch.delenv("NCBI_API_KEY", raising=False)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "--no-cache", "search", "article", keyword,
             "--source", "pubtator", "--limit", "1"],
            capture_output=True, text=True, check=False, timeout=30,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

    assert result.returncode == 0, result.stderr + result.stdout
    assert json.loads(result.stdout)["results"] == []
    assert [params["text"][0] for path, params in requests if path == "/search/"] == [
        expected_query
    ]
    expected_autocomplete = [] if keyword == "melanoma" else [keyword]
    assert [params["query"][0] for path, params in requests
            if path == "/entity/autocomplete/"] == expected_autocomplete
