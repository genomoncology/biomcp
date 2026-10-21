"""Frameshift spellings reach anchored provider queries without DNA guesses."""
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

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402
from hang_guard import HANG_TIMEOUT  # noqa: E402

pytestmark = [pytest.mark.needs_binary]


@pytest.mark.parametrize("identifier,surface", [
    (identifier, surface)
    for identifier in ("FANCL p.T367fs", "FANCL p.T367Nfs*13", "FANCL p.T367fs*13",
                       "FANCL p.Thr367AsnfsTer13", "FANCL p.T367Nfs*?")
    for surface in ("get", "articles")
] + [("FANCL p.T367fs", "different-frameshift")])
def test_frameshift_spelling_reaches_anchored_queries(identifier: str, surface: str, tmp_path: Path) -> None:
    requests: list[tuple[str, dict]] = []
    change = identifier.split(" p.", 1)[1]
    # Explicitly authored provider assertions exercise resolution, not a real
    # FANCL DNA/protein equivalence. The empty get control proves not_found.
    source_change = "T367Nfs*13" if surface == "different-frameshift" else change
    hit = {"_id": "chr2:g.100000dup", "dbnsfp": {"genename": "FANCL", "hgvsp": [source_change]}}

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            requests.append((target.path, query))
            if target.path == "/v1/query":
                body = {"total": int(surface != "get"),
                        "hits": [hit] if surface != "get" else []}
            elif target.path == "/query":
                body = {"total": 0, "hits": []}
            elif target.path == "/search":
                body = {"hitCount": 0, "resultList": {"result": []}}
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path.endswith("/esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path.startswith("/graph/"):
                body = {"total": 0, "data": []}
            else:
                body = []
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {key: origin for key in (
        "BIOMCP_PUBTATOR_BASE", "BIOMCP_EUROPEPMC_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE", "BIOMCP_MYGENE_BASE",
        "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    )}
    providers["BIOMCP_MYVARIANT_BASE"] = origin + "/v1"
    providers["BIOMCP_PUBMED_BASE"] = origin + "/entrez/eutils"
    env = isolated_command_env(tmp_path / "isolated", providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    args = ["get", "variant", identifier] if surface == "get" else [
        "variant", "articles", identifier, "--strategy", "lexical", "--debug-plan",
    ]
    try:
        result = subprocess.run([binary, "--no-cache", "--json", *args], env=env,
                                capture_output=True, text=True, timeout=HANG_TIMEOUT)
        assert "invalid_argument" not in result.stdout + result.stderr, result.stdout + result.stderr
        query_change = "T367Nfs*13" if surface == "get" and "Thr" in change else change
        anchored = [query for path, query in requests if path == "/v1/query"
                    and "FANCL" in query.get("q", [""])[0]
                    and query_change in query.get("q", [""])[0].replace("\\", "")]
        assert anchored, (requests, result.stdout, result.stderr)
        if surface == "get":
            assert result.returncode != 0
            assert "not_found" in result.stdout + result.stderr
        else:
            assert result.returncode == 0, result.stdout + result.stderr
            payload = json.loads(result.stdout)
            assert payload["requested_variant"]["protein_change"] == identifier.split(" ", 1)[1]
            if surface == "different-frameshift":
                assert payload["resolution"]["status"] != "resolved", payload["resolution"]
            else:
                assert payload["resolution"]["status"] == "resolved", payload["resolution"]
            assert any(path == "/search" and change in query.get("query", [""])[0].replace("\\", "")
                       for path, query in requests), requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)


@pytest.mark.parametrize("spelling", ["p.T367fsbogus", "p.T0fs", "p.T367fs*0", "p.T367fs*1", "p.T367fs*"])
def test_unparseable_variant_names_supported_forms(spelling: str, tmp_path: Path) -> None:
    env = isolated_command_env(tmp_path / "isolated", {}, "http://127.0.0.1:1")
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    result = subprocess.run([binary, "--json", "get", "variant", f"FANCL {spelling}"],
                            env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT)
    assert result.returncode != 0
    message = result.stdout + result.stderr
    assert "invalid_argument" in message
    assert "Supported formats" in message
    assert "FANCL p.T367fs" in message
