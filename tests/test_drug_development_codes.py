"""Resolve development-code identity through the CLI, using authored replies.

The two public ChEMBL identifiers, preferred names and exact synonym spellings
were checked through molecule.json on 2026-10-19. The replies below are authored.
"""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

pytestmark = pytest.mark.needs_binary
ROOT = Path(__file__).resolve().parents[1]
IDENTITIES = {
    "MK-3475": ("CHEMBL3137343", "PEMBROLIZUMAB"),
    "CT-011": ("CHEMBL2108680", "PIDILIZUMAB"),
}


@pytest.fixture
def development_cli(tmp_path, monkeypatch):
    state = {"mode": "unique"}
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            requests.append((target.path, query))
            body = {"total": 0, "hits": [], "results": [], "response": {"docs": []}}
            status = 200
            if target.path == "/mychem/query":
                term = query.get("q", [""])[0]
                identity = next((row for row in IDENTITIES.values() if term in row), None)
                hit = {"_id": "authored-text-only", "_score": 30}
                if identity:
                    chembl_id, name = identity
                    if state["mode"] == "conflicting_card":
                        chembl_id = "CHEMBL9999999"
                    hit = {"_id": chembl_id, "chembl": {"molecule_chembl_id": chembl_id, "pref_name": name}}
                body = {"total": 1, "hits": [hit]}
            elif target.path == "/chembl/molecule.json":
                code = query.get("molecule_synonyms__molecule_synonym__iexact", [""])[0]
                chembl_id, name = IDENTITIES.get(code.upper(), ("CHEMBL9999998", "AUTHORED"))
                rows = [{"molecule_chembl_id": chembl_id, "pref_name": name,
                         "molecule_synonyms": [{"molecule_synonym": code}]}]
                if state["mode"] == "ambiguous":
                    rows.append({"molecule_chembl_id": "CHEMBL9999998", "pref_name": "OTHER",
                                 "molecule_synonyms": [{"molecule_synonym": code}]})
                elif state["mode"] == "substring":
                    rows[0]["molecule_synonyms"] = [{"molecule_synonym": code + " unrelated"}]
                elif state["mode"] == "unavailable":
                    status = 503
                body = {"molecules": rows, "page_meta": {"total_count": len(rows), "next": None}}
                if state["mode"] == "incomplete":
                    body["page_meta"]["next"] = "/molecule.json?offset=100"
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    for key, path in {"MYCHEM": "mychem", "CHEMBL": "chembl", "OPENFDA": "openfda",
                      "OLS4": "ols", "MYGENE": "gene", "MYDISEASE": "disease"}.items():
        monkeypatch.setenv(f"BIOMCP_{key}_BASE", f"{base}/{path}")
    monkeypatch.setenv("BIOMCP_TEST_UNPACED_ORIGIN", base)
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.setenv("BIOMCP_CACHE_MODE", "off")
    for key in ("UMLS_API_KEY", "NCBI_API_KEY", "OPENFDA_API_KEY", "BIOMCP_DISABLE_SOURCES"):
        monkeypatch.delenv(key, raising=False)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))

    def run(*args):
        result = subprocess.run([binary, "--json", *args], capture_output=True, text=True, timeout=90)
        return result, json.loads(result.stdout)

    try:
        yield run, state, requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join()


def test_development_codes_resolve_the_same_identity_singly_and_in_batch(development_cli):
    run, _state, requests = development_cli
    singles = []
    for code, (chembl_id, name) in IDENTITIES.items():
        result, payload = run("get", "drug", code, "label")
        assert result.returncode == 0, payload
        assert payload["name"].casefold() == name.casefold()
        assert payload["chembl_id"] == chembl_id
        singles.append((payload["name"], payload["chembl_id"]))
    result, batch = run("batch", "drug", ",".join(IDENTITIES), "--sections", "label")
    assert result.returncode == 0, batch
    assert [(row["name"], row["chembl_id"]) for row in [item["result"] for item in batch["items"]]] == singles
    assert all("molecule_synonyms__molecule_synonym__iexact" in query
               for path, query in requests if path == "/chembl/molecule.json")


@pytest.mark.parametrize("mode", ["ambiguous", "substring", "conflicting_card", "incomplete", "unavailable"])
def test_development_code_refuses_unconfirmed_identity(development_cli, mode):
    run, state, _requests = development_cli
    state["mode"] = mode
    result, payload = run("get", "drug", "MK-3475", "label")
    assert result.returncode != 0, payload
    assert "chembl_id" not in payload
    if mode == "unavailable":
        assert payload["error"]["code"] != "not_found", payload
        assert payload["error"]["source"] == "ChEMBL"
