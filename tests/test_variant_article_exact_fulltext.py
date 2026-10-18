"""The exact-name CLI searches one literal phrase across Europe PMC full text."""

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
CAPTURE = REPO_ROOT / "tests/fixtures/article/exact-name-fulltext.json"


@pytest.mark.parametrize("name,pmids", [("F1761I", ["10811118"]), ("BRCA1 F1761I", [])])
def test_exact_name_searches_fulltext_with_source_specific_yields(
    name: str, pmids: list[str], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    captures = json.loads(CAPTURE.read_text())["captures"]
    replies = {capture["query"]: capture["response"] for capture in captures}
    requests: list[tuple[str, dict[str, list[str]]]] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            requests.append((target.path, params))
            if target.path == "/search":
                query = params.get("query", [""])[0]
                if query not in replies:
                    self.send_error(404, "Unrecorded Europe PMC query")
                    return
                body = replies[query]
            elif target.path == "/entrez/eutils/esearch.fcgi":
                body = {"esearchresult": {"count": "0", "idlist": []}}
            elif target.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif target.path == "/graph/v1/paper/search/bulk":
                body = {"total": 0, "data": []}
            else:
                self.send_error(404, "No fixture for request")
                return
            self.reply(body)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            if urlsplit(self.path).path == "/graph/v1/paper/batch":
                self.reply([None])
            else:
                self.send_error(404, "No fixture for request")

        def reply(self, body: object) -> None:
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
    for key in (
        "BIOMCP_EUROPEPMC_BASE", "BIOMCP_PUBTATOR_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_MYVARIANT_BASE", "BIOMCP_CLINGEN_CAR_BASE",
        "BIOMCP_TEST_UNPACED_ORIGIN",
    ):
        monkeypatch.setenv(key, base)
    monkeypatch.setenv("BIOMCP_PUBMED_BASE", base + "/entrez/eutils")
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.delenv("BIOMCP_DISABLE_SOURCES", raising=False)
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    command = [
        binary, "--json", "--no-cache", "variant", "articles", name,
        "--strategy", "exact", "--debug-plan", "--limit", "5",
    ]
    try:
        result = subprocess.run(
            command, capture_output=True, text=True, check=False, timeout=60,
        )
        if not pmids:
            markdown_command = [arg for arg in command if arg not in ("--json", "--debug-plan")]
            markdown = subprocess.run(
                markdown_command, capture_output=True, text=True, check=False, timeout=60,
            )
            assert markdown.returncode == 0, markdown.stderr + markdown.stdout
            assert "No articles found matching the filters." in markdown.stdout
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

    assert result.returncode == 0, result.stderr + result.stdout
    payload = json.loads(result.stdout)
    plan = payload["debug_plan"]
    print(json.dumps({"command": command, "results": payload["results"],
                      "provider_queries": plan["provider_queries"],
                      "alias_yields": plan["alias_yields"]}))
    assert [row["pmid"] for row in payload["results"]] == pmids
    assert payload["complete"] is True
    assert [params["query"][0] for path, params in requests if path == "/search"] == [f'"{name}"']
    assert [params["term"][0] for path, params in requests
            if path == "/entrez/eutils/esearch.fcgi"] == [f'"{name}"[Title/Abstract]']
    queries = {row["provider"]: row for row in plan["provider_queries"]}
    assert len(plan["provider_queries"]) == 4
    assert queries["europepmc"]["query_template_version"] == "europepmc-fulltext-phrase-v1"
    assert queries["pubmed"]["query_template_version"] == "pubmed-title-abstract-phrase-v1"
    assert all(row["query_alias"] == name for row in queries.values())
    yields = {row["source"]: row for row in plan["alias_yields"]}
    assert set(yields) == {"pubmed", "europepmc", "pubtator", "semanticscholar"}
    for source, row in yields.items():
        assert row["alias"] == name
        assert row["route"] == "exact_name"
        assert row["hits_received"] == (len(pmids) if source == "europepmc" else 0)
        assert row["hits_kept"] == (len(pmids) if source == "europepmc" else 0)
        assert row["cut_by_cap"] is False
