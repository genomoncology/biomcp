"""Every resolved form gets an honest disposition, including the alias cap."""

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

pytestmark = [pytest.mark.needs_binary]
FIXTURE = REPO_ROOT / "tests/fixtures/article/coding-aliases/authored.json"


@pytest.mark.parametrize("surface", ["json", "debug", "markdown", "batch", "disabled", "disabled-markdown"])
def test_resolved_forms_omitted_by_alias_cap_are_named(surface: str, tmp_path: Path) -> None:
    control = json.loads(FIXTURE.read_text())
    hit = control["hit"]
    requests: list[tuple[str, dict]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            requests.append((target.path, parse_qs(target.query)))
            if target.path == "/query":
                # Authored empty MyGene facts keep this control about coding-form coverage.
                body = {"total": 0, "hits": []}
            elif target.path == "/v1/query":
                body = {"total": 1, "hits": [hit]}
            elif target.path.startswith("/v1/variant/"):
                body = hit
            elif target.path == "/allele":
                body = {"errorType": "NotFound", "description": "Authored control"}
            elif target.path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                body = []
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path.endswith("/esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path == "/search":
                body = {"hitCount": 0, "resultList": {"result": []}}
            elif target.path in ("/graph/v1/paper/search", "/graph/v1/paper/search/bulk"):
                body = {"total": 0, "data": []}
            else:
                misses.append(self.path)
                self.send_error(404, "No authored control")
                return
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
    disabled = surface.startswith("disabled")
    if disabled:
        env["BIOMCP_DISABLE_SOURCES"] = "pubmed,europe-pmc,pubtator3,semantic-scholar"
    command = [os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
               "--no-cache"]
    if surface not in ("markdown", "disabled-markdown"):
        command.append("--json")
    command += ["variant", "articles", "ABCC8 R1437Q", "--strategy", "lexical", "--limit", "5"]
    if surface == "batch":
        request = tmp_path / "input.json"
        request.write_text(json.dumps([{"gene": "ABCC8", "protein": "R1437Q"}]))
        command.remove("ABCC8 R1437Q")
        command += ["--input", str(request)]
    if surface in ("debug", "batch"):
        command.append("--debug-plan")
    try:
        result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                capture_output=True, text=True, timeout=60)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert result.returncode == (1 if disabled else 0), result.stdout + result.stderr
    assert not misses, misses
    if surface in ("markdown", "disabled-markdown"):
        text = result.stderr if disabled else result.stdout
        assert "ABCC8 c.4310G>A" in text
        assert "ABCC8 c.4373G>A" in text
        assert "alias_limit" in text
        if disabled:
            assert "ABCC8 R1437Q: omitted (source_disabled; planned: true)" in text
        return
    payload = json.loads(result.stdout)
    if surface == "batch":
        payload = payload["items"][0]
    assert payload["resolution"]["status"] == "resolved", payload
    coverage = payload["alias_coverage"]
    forms = hit["dbnsfp"]["hgvsp"] + hit["dbnsfp"]["hgvsc"]
    assert {row["alias"] for row in coverage} >= {f"ABCC8 {form}" for form in forms}
    for alias in ("ABCC8 c.4310G>A", "ABCC8 c.4373G>A"):
        row = next(row for row in coverage if row["alias"] == alias)
        assert row == {"alias": alias, "planned": False, "attempted": False,
                       "disposition": "omitted", "reason": "alias_limit"}
        assert all(alias not in json.dumps(query) for _, query in requests)
    primary = next(row for row in coverage if row["alias"] == "ABCC8 R1437Q")
    assert primary["planned"] is True
    assert primary["attempted"] is not disabled
    assert primary["disposition"] == ("omitted" if disabled else "attempted")
    assert primary["reason"] == ("source_disabled" if disabled else "provider_query_attempted")
    if disabled:
        assert all(path.startswith("/v1/") or path in ("/allele", "/query") for path, _ in requests)
    else:
        assert any("R1437Q" in json.dumps(query) for _, query in requests)
    if surface in ("debug", "batch"):
        assert payload["debug_plan"]["alias_coverage"] == coverage
