"""Resolved coding forms recover authored papers within the alias cap."""

from __future__ import annotations

from hang_guard import HANG_TIMEOUT

import json
import os
import select
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


@pytest.mark.parametrize("surface", ["json", "debug", "markdown", "batch", "disabled", "disabled-markdown", "cap", "complete", "reported", "lexical", "gene", "requested38", "unknown", "mcp"])
def test_resolved_coding_forms_recover_papers_and_disclose_limits(surface: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    control = json.loads(FIXTURE.read_text())
    hit = control["hit"]
    if surface == "complete":
        hit["dbnsfp"]["hgvsp"] = ["R1437Q"]
        hit["dbnsfp"]["hgvsc"] = ["c.4310G>A", "c.4373G>A"]
    if surface == "cap":
        hit["dbnsfp"]["hgvsc"].extend(f"c.{index:04}G>A" for index in range(40))
    requests: list[tuple[str, dict]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            requests.append((target.path, parse_qs(target.query)))
            if target.path == "/query":
                # Authored empty MyGene facts keep this control about coding-form coverage.
                body = ({"total": 1, "hits": [{"symbol": "ABCC8", "entrezgene": 6833,
                                               "alias": ["SUR1"]}]} if surface == "gene"
                        else {"total": 0, "hits": []})
            elif target.path == "/v1/query":
                body = {"total": 1, "hits": [hit]}
            elif target.path.startswith("/v1/variant/"):
                body = hit
            elif target.path == "/allele":
                body = {"errorType": "NotFound", "description": "Authored control"}
            elif target.path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                body = []
            elif target.path == "/publications/export/biocjson":
                body = []
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path.endswith("/esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path == "/search":
                term = parse_qs(target.query).get("query", [""])[0]
                rows = [
                    {"id": pmid, "source": "MED", "pmid": pmid,
                     "title": f"Authored coding control {alias}",
                     "authorString": "Control A", "pubYear": "2026"}
                    for alias, pmid in (("c.4310G>A", "99000001"), ("c.4373G>A", "99000002"))
                    if alias in term
                ]
                body = {"hitCount": len(rows), "resultList": {"result": rows}}
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

        def do_POST(self) -> None:  # noqa: N802
            request = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
            if urlsplit(self.path).path != "/graph/v1/paper/batch":
                misses.append(self.path)
                self.send_error(404, "No authored control")
                return
            payload = json.dumps([None] * len(request["ids"])).encode()
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
    command += ["variant", "articles", "ABCC8 R1437Q", "--limit", "5"]
    if surface in ("complete", "lexical"):
        command += ["--strategy", "lexical"]
    if surface in ("batch", "requested38", "unknown"):
        request = tmp_path / "input.json"
        item = {"gene": "ABCC8", "protein": "R1437Q"}
        if surface in ("requested38", "unknown"):
            item["genomic"] = hit["_id"]
        if surface == "requested38":
            item["build"] = "GRCh38"
        request.write_text(json.dumps([item]))
        command.remove("ABCC8 R1437Q")
        command += ["--input", str(request)]
    if surface in ("debug", "batch", "reported", "gene"):
        command.append("--debug-plan")
    if surface == "reported":
        command.append("--verify-identity")
    try:
        if surface == "mcp":
            from test_mcp_2026_protocol import RawStdioMcp

            with monkeypatch.context() as isolated:
                for key in set(os.environ) - set(env):
                    isolated.delenv(key)
                for key, value in env.items():
                    isolated.setenv(key, value)
                client = RawStdioMcp()
                try:
                    client.send({"jsonrpc": "2.0", "id": "init", "method": "initialize",
                                 "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                                            "clientInfo": {"name": "assembly-contract", "version": "1"}}})
                    ready, _, _ = select.select([client.process.stdout], [], [], HANG_TIMEOUT)
                    assert ready
                    assert client.receive()["id"] == "init"
                    client.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
                    client.send({"jsonrpc": "2.0", "id": "articles", "method": "tools/call",
                                 "params": {"name": "variant_articles", "arguments": {
                                     "items": [{"gene": "ABCC8", "protein": "R1437Q"}], "limit": 5}}})
                    ready, _, _ = select.select([client.process.stdout], [], [], HANG_TIMEOUT)
                    assert ready
                    response = client.receive()["result"]
                    assert response.get("isError") is not True, response
                    payload = json.loads(response["content"][0]["text"])["items"][0]
                finally:
                    client.close()
        else:
            result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                    capture_output=True, text=True, timeout=HANG_TIMEOUT)
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert not misses, misses
    if surface != "mcp":
        assert result.returncode == (1 if disabled else 0), result.stdout + result.stderr
    if surface in ("markdown", "disabled-markdown"):
        text = result.stderr if disabled else result.stdout
        assert "chr11:g.17417157C>T (GRCh37; myvariant)" in text
        assert "ABCC8 c.4310G>A" in text
        assert "ABCC8 c.4373G>A" in text
        assert ("source_disabled" if disabled else "provider_query_attempted") in text
        if disabled:
            assert "ABCC8 R1437Q: omitted (source_disabled; planned: true)" in text
        return
    if surface != "mcp":
        payload = json.loads(result.stdout)
    if surface in ("batch", "requested38", "unknown"):
        payload = payload["items"][0]
    assert payload["resolution"]["status"] == "resolved", payload
    coordinates = payload["resolution"]["normalized_aliases"]["genomic_coordinates"]
    build = "GRCh38" if surface == "requested38" else "GRCh37"
    expected = [{"coordinate": hit["_id"], "genome_build": build,
                 "source": "myvariant", "provenance": "resolved source identity"}]
    if surface in ("requested38", "unknown"):
        expected.insert(0, {"coordinate": hit["_id"], "genome_build": (
            "GRCh38" if surface == "requested38" else "unknown"),
            "source": "requested", "provenance": "requested variant identity"})
    assert coordinates == expected
    assert payload["resolution"]["normalized_aliases"]["genomic_ids"] == [hit["_id"]]
    for row in payload["results"]:
        assert row["genomic_coordinates"] == coordinates
    coverage = payload["alias_coverage"]
    forms = hit["dbnsfp"]["hgvsp"] + hit["dbnsfp"]["hgvsc"]
    assert {row["alias"] for row in coverage} >= {f"ABCC8 {form}" for form in forms}
    for alias in ("ABCC8 c.4310G>A", "ABCC8 c.4373G>A"):
        row = next(row for row in coverage if row["alias"] == alias)
        omitted = surface == "lexical"
        assert row["planned"] is not omitted, row
        assert row["attempted"] is not (omitted or disabled), row
        assert row["reason"] == ("alias_limit" if omitted else
                                 "source_disabled" if disabled else "provider_query_attempted"), row
        sent = any(alias.split(" ", 1)[1] in json.dumps(query)
                   for path, query in requests if path == "/search")
        assert sent is not (omitted or disabled), (alias, requests)
    if not disabled and surface != "lexical":
        assert {row["pmid"] for row in payload["results"]} == {"99000001", "99000002"}
        assert all(row["match_reason"] == "unverified_alias_match" for row in payload["results"])
        if surface == "complete":
            assert payload["complete"] is True, json.dumps(payload["source_status"])
            assert payload["truncated"] is False, payload
    if surface == "lexical":
        assert any(row["reason"] == "alias_limit" for row in coverage)
        assert payload["complete"] is False, payload
        assert payload["truncated"] is True, payload
    primary = next(row for row in coverage if row["alias"] == "ABCC8 R1437Q")
    assert primary["planned"] is True
    assert primary["attempted"] is not disabled
    assert primary["disposition"] == ("omitted" if disabled else "attempted")
    assert primary["reason"] == ("source_disabled" if disabled else "provider_query_attempted")
    if disabled:
        assert all(path.startswith("/v1/") or path in ("/allele", "/query", "/variant/autocomplete/")
                   for path, _ in requests)
    else:
        assert any("R1437Q" in json.dumps(query) for _, query in requests)
    for path, query in requests:
        if path == "/search":
            assert len(query["query"][0]) <= 2048
        elif path.endswith("/esearch.fcgi"):
            assert len(query["term"][0]) <= 4096
    if surface in ("debug", "batch", "reported", "gene"):
        assert payload["debug_plan"]["alias_coverage"] == coverage
        plan = payload["debug_plan"]
        assert plan["normalized_aliases"]["genomic_coordinates"] == coordinates
        lexical_queries = next(row["queries"] for row in plan["routes"]
                               if row["route"] == "exact_lexical")
        if surface == "gene":
            assert "ABCC8 R1458Q" in lexical_queries, lexical_queries
            assert any(query.get("query", [""])[0].startswith("ABCC8 R1458Q AND NOT")
                       for path, query in requests if path == "/search"), requests
        for alias in ("ABCC8 c.4310G>A", "ABCC8 c.4373G>A"):
            assert alias in lexical_queries
            assert any(row["route"] == "strict" and row["query_alias"] == alias
                       for row in plan["provider_queries"])
            # Restricted strict work cannot stand in for all indexed fields.
            all_field_plans = [row for row in plan["provider_queries"]
                               if row["route"] == "exact_lexical"
                               and row["provider"] == "europepmc"
                               and alias in row.get("query_aliases", [row["query_alias"]])]
            assert all_field_plans, (alias, plan["provider_queries"])
            for all_field in all_field_plans:
                query = all_field["query"]
                assert "TITLE_ABS:" not in query
                assert "ABCC8" in query and alias.split(" ", 1)[1] in query
                assert any(path == "/search" and params.get("query") == [query]
                           for path, params in requests), query
        allocation = plan["work_allocation"]
        assert allocation["discovery"]["consumed"] <= allocation["discovery"]["limit"] == 197
        for work in (allocation["exact_lexical"]["item"], allocation["identity_verification"]["item"]):
            assert work["consumed"] <= work["limit"] == 50
