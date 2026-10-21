"""Authored controls prove MyGene alias queries, without claiming real paper recovery."""

from __future__ import annotations

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
FIXTURE = REPO_ROOT / "tests/fixtures/article/gene-aliases/authored.json"


@pytest.mark.parametrize("surface", ["json", "debug", "markdown", "absent-markdown", "batch", pytest.param("mcp", marks=pytest.mark.timeout(120)), "absent", "unavailable", "slow", "disabled", "ambiguous", "missing_entrez", "duplicate_entrez", "truncated", "cap", "grouped", "grouped-markdown", "split", "verify"])
def test_source_backed_gene_alias_queries(surface: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    mode = surface.removesuffix("-markdown")
    control = json.loads(FIXTURE.read_text())
    hit = control["hit"]
    requests: list[tuple[str, dict]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            requests.append((target.path, parse_qs(target.query)))
            if target.path == "/query":
                if mode == "slow":
                    threading.Event().wait(8)
                    return
                if mode == "unavailable":
                    self.send_error(503, "Authored unavailable control")
                    return
                gene = dict(control["mygene"])
                if mode == "absent":
                    gene["alias"] = []
                if mode == "missing_entrez":
                    gene.pop("entrezgene")
                if mode == "split":
                    gene["alias"] = [f"GENE_CONTROL_{i:03d}_" + "A" * 32 for i in range(120)] + ["SUR1"]
                if mode in ("grouped", "verify"):
                    gene["alias"] = control["derived_alias_array"]["aliases"]
                if mode == "cap":
                    gene["alias"] = [f"A{i}" for i in range(12)] + ["SUR1"]
                hits = [gene]
                if mode == "ambiguous":
                    hits.append({"symbol": "OTHER", "entrezgene": 999, "alias": ["ABCC8"]})
                if mode == "duplicate_entrez":
                    hits.append({**gene, "entrezgene": 6834})
                body = {"total": len(hits) + (mode == "truncated"), "hits": hits}
            elif target.path == "/v1/query":
                body = {"total": 1, "hits": [hit]}
            elif target.path.startswith("/v1/variant/"):
                body = hit
            elif target.path == "/allele":
                body = {"errorType": "NotFound", "description": "Authored control"}
            elif target.path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                body = []
            elif target.path == "/publications/export/biocjson":
                body = control["untyped_article"]
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path.endswith("/esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path == "/search":
                term = parse_qs(target.query).get("query", [""])[0]
                contains_sur1 = "SUR1 R1437Q" in term or ('"SUR1"' in term and '"R1437Q"' in term)
                rows = [control["article"]] if contains_sur1 else []
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
    disabled = mode == "disabled"
    if disabled:
        env["BIOMCP_DISABLE_SOURCES"] = "mygene-info"
    command = [os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
               "--no-cache"]
    if not surface.endswith("markdown"):
        command.append("--json")
    command += ["variant", "articles", "ABCC8 R1437Q", "--strategy", "lexical", "--limit", "5"]
    if surface == "batch":
        request = tmp_path / "input.json"
        request.write_text(json.dumps([{"gene": "ABCC8", "protein": "R1437Q"}]))
        command.remove("ABCC8 R1437Q")
        command += ["--input", str(request)]
    if mode == "verify":
        command.append("--verify-identity")
    if surface in ("debug", "batch"):
        command.append("--debug-plan")
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
                    client.send({
                        "jsonrpc": "2.0", "id": "gene-alias-init", "method": "initialize",
                        "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                                   "clientInfo": {"name": "gene-alias-contract", "version": "1"}},
                    })
                    ready, _, _ = select.select([client.process.stdout], [], [], 60)
                    assert ready, "MCP initialization response did not arrive"
                    assert client.receive()["id"] == "gene-alias-init"
                    client.send({"jsonrpc": "2.0", "method": "notifications/initialized"})
                    client.send({
                        "jsonrpc": "2.0", "id": "gene-alias", "method": "tools/call",
                        "params": {"name": "variant_articles", "arguments": {
                            "items": [{"gene": "ABCC8", "protein": "R1437Q"}],
                            "strategy": "lexical", "limit": 5,
                        }},
                    })
                    ready, _, _ = select.select([client.process.stdout], [], [], 60)
                    assert ready, "variant article MCP response did not arrive"
                    response = client.receive()["result"]
                finally:
                    client.close()
                assert response.get("isError") is not True, response
                payload = json.loads(response["content"][0]["text"])["items"][0]
        else:
            result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                    capture_output=True, text=True, timeout=60)
            assert result.returncode == 0, result.stdout + result.stderr
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert not misses, misses
    if surface.endswith("markdown"):
        assert "mygene-info" in result.stdout
        if mode == "absent":
            assert "current-spelling-only" in result.stdout
        else:
            assert "SUR1 R1437Q" in result.stdout
            assert "current-spelling-only" not in result.stdout
            if mode == "grouped":
                assert "pubtator_alias_limit" in result.stdout
                assert "semanticscholar_alias_limit" in result.stdout
        return
    if surface != "mcp":
        payload = json.loads(result.stdout)
        if surface == "batch":
            payload = payload["items"][0]
    coverage = payload["gene_symbol_coverage"]
    assert coverage["source"] == "mygene-info"
    assert coverage["source_field"] == "alias"
    if surface in ("absent", "unavailable", "slow", "disabled", "ambiguous", "missing_entrez", "duplicate_entrez", "truncated"):
        assert coverage["status"] == ("unavailable" if surface == "slow" else "absent" if surface not in ("unavailable", "disabled") else surface)
        assert "current-spelling-only" in coverage["restriction"]
        assert not coverage["aliases"]
        assert all("SUR1" not in json.dumps(query) for _, query in requests)
        return
    assert coverage["status"] == "available"
    assert coverage["entrez_id"] == "6833"
    assert coverage["restriction"] is None
    coding = next(row for row in payload["alias_coverage"] if row["alias"] == "ABCC8 c.4310G>A")
    assert coding["planned"] and coding["attempted"], coding
    assert "SUR1" in coverage["recorded_aliases"]
    assert "SUR1 R1437Q" not in coverage["recorded_aliases"]
    for path, parameter, limit in (("/search", "query", 2048), ("/entrez/eutils/esearch.fcgi", "term", 4096)):
        grouped = [query[parameter][0] for observed, query in requests
                   if observed == path and query.get(parameter, [""])[0].startswith("((")]
        assert len(grouped) > 1 if mode == "split" else len(grouped) == 1, (path, grouped, requests)
        assert all(len(query) <= limit and '"R1437Q"' in query for query in grouped)
        for symbol in ["ABCC8", *coverage["recorded_aliases"]]:
            assert sum(f'"{symbol}"' in query for query in grouped) == 1, (symbol, grouped)
        for left, right in zip(grouped, grouped[1:]):
            prefix, suffix = left.rsplit(") AND ", 1)
            first_next = right.rsplit(") AND ", 1)[0][2:].split(" OR ", 1)[0]
            assert len(f"{prefix} OR {first_next}) AND {suffix}") > limit
    alias = next(row for row in coverage["aliases"] if row["alias"] == "SUR1 R1437Q")
    assert alias["planned"] and alias["attempted"]
    assert any("SUR1" in json.dumps(query) for _, query in requests)
    assert all("ABCC8 SUR1" not in json.dumps(query) for _, query in requests)
    queries = [query for path, query in requests if path == "/search"]
    assert "ABCC8" in json.dumps(queries[0])
    assert " OR " in json.dumps(queries[0])
    if mode != "split":
        assert '"SUR1"' in queries[0]["query"][0]
    row = next(row for row in payload["results"] if row["pmid"] == control["article"]["pmid"])
    assert row["matched_aliases"] == []
    assert row["match_reason"] == "unverified_alias_match"
    for fact in row.get("provenance", []):
        assert "SUR1 R1437Q" in fact["query_aliases"]
        assert fact.get("matched_alias") is None
    assert row.get("identity", {}).get("status") != "confirmed"
    if mode == "verify":
        assert row["identity"]["status"] == "unverified"
        assert row["identity"]["reason"] == "missing_typed_gene_annotation"
    if mode in ("cap", "grouped", "split", "verify"):
        omitted = [row for row in coverage["aliases"] if row["reason"].endswith("_alias_limit")]
        assert omitted
        assert not any(row["attempted"] or row["planned"] for row in omitted)
        if mode in ("grouped", "split", "verify"):
            sur1_omissions = {row["reason"] for row in omitted if row["alias"] == "SUR1 R1437Q"}
            assert sur1_omissions == {"pubtator_alias_limit", "semanticscholar_alias_limit"}
            assert all("SUR1 R1437Q" not in json.dumps(query) for path, query in requests
                       if path in ("/search/", "/graph/v1/paper/search", "/graph/v1/paper/search/bulk"))
    if surface in ("debug", "batch"):
        assert payload["debug_plan"]["gene_symbol_coverage"] == coverage
