"""CLI regressions against authored controls; no captured licensed payloads."""
from __future__ import annotations

import gzip
import json
import os
import shutil
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

import pytest

pytestmark = pytest.mark.needs_binary
ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/live-suspects"


@pytest.fixture
def suspect_cli(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    state = {"disease": "resolved", "discovery": "h3"}
    requests = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            target = urlsplit(self.path)
            requests.append(target.path)
            name = None
            if target.path == "/mychem/query":
                name = "aspirin-authored.json"
            elif target.path == "/disease/query":
                name = "disease-query-authored.json" if state["disease"] == "resolved" else "disease-empty-authored.json"
            elif target.path == "/disease/disease/MONDO:0033642":
                name = "disease-detail-authored.json"
            elif target.path == "/ols/api/search":
                if state["discovery"] == "h3":
                    name = "discover-authored.json"
                else:
                    controls = json.loads((FIXTURES / "discovery-controls-authored.json").read_text())
                    body = {"response": {"docs": [controls[state["discovery"]]]}}
            elif target.path == "/label/drug/label.json":
                body = {"results": []}
            elif target.path == "/medline/ws/query":
                payload = b'<nlmSearchResult><list count="0"/></nlmSearchResult>'
                self.send_response(200)
                self.end_headers()
                self.wfile.write(payload)
                return
            else:
                body = {"total": 0, "hits": [], "terms": []}
            if name:
                body = json.loads((FIXTURES / name).read_text())
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    for key, path in {"MYCHEM": "mychem", "MYDISEASE": "disease", "OLS4": "ols", "MYGENE": "gene", "HPO": "hpo", "MEDLINEPLUS": "medline", "OPENFDA": "label"}.items():
        monkeypatch.setenv(f"BIOMCP_{key}_BASE", f"{base}/{path}")
    monkeypatch.setenv("BIOMCP_TEST_UNPACED_ORIGIN", base)
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.setenv("BIOMCP_CACHE_MODE", "off")
    for key in ("UMLS_API_KEY", "NCBI_API_KEY", "BIOMCP_DISABLE_SOURCES"):
        monkeypatch.delenv(key, raising=False)
    ddinter = tmp_path / "ddinter"
    ddinter.mkdir()
    for code in "ABDHLPRV":
        shutil.copy(FIXTURES / "ddinter-authored.csv", ddinter / f"ddinter_downloads_code_{code}.csv")
    monkeypatch.setenv("BIOMCP_DDINTER_DIR", str(ddinter))
    gtr = tmp_path / "gtr"
    shutil.copytree(ROOT / "spec/fixtures/gtr", gtr)
    version = gtr / "test_version.gz"
    body = gzip.decompress(version.read_bytes())
    body += b'GTR000596648.2\t1\tAuthored diagnostic panel\tExample Diagnostics\tmolecular\tExample Lab\tExample Institute\t00D5966482\tNY\tUSA\tCurrent\tPublic\tMolecular genetics\tSequence analysis\tODC1\tMONDO:0033642\n'
    version.write_bytes(gzip.compress(body))
    with (gtr / "test_condition_gene.txt").open("a") as file:
        file.write("GTR000596648.2\tcondition\tneurodevelopmental disorder with alopecia and brain abnormalities\n")
    monkeypatch.setenv("BIOMCP_GTR_DIR", str(gtr))
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))

    def run(*args, json_output=True):
        command = [binary, *(["--json"] if json_output else []), *args]
        result = subprocess.run(command, capture_output=True, text=True, timeout=30)
        assert result.returncode == 0, result.stderr + result.stdout
        return json.loads(result.stdout) if json_output else result.stdout

    try:
        yield run, state, requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def test_aspirin_keeps_selected_source_name_without_absorbing_combo_ingredient(suspect_cli):
    run, _, _ = suspect_cli
    result = run("drug", "interactions", "aspirin")
    assert result["drugbank_id"] == "DB00945"
    assert result["chembl_id"] == "CHEMBL25"
    text = json.dumps(result)
    assert "Example partner" in text
    assert "dipyridamole" in text
    assert "not_in_ddinter_coverage" not in text


def test_discovery_preserves_exact_variant_before_fuzzy_chemical_and_vaccine(suspect_cli):
    run, _, _ = suspect_cli
    result = run("discover", "H3K27M")
    assert result["concepts"][0]["primary_id"] == "NCIT:C129310"
    assert result["concepts"][0]["primary_type"] == "Variant"
    assert "get drug" not in json.dumps(result)
    assert "H3K27M" in json.dumps(result["_meta"])


@pytest.mark.parametrize("kind,query,expected", [("chemical", "Example chemical therapy", "Unknown"), ("vaccine", "Example vaccine", "Unknown"), ("drug", "Example alias", "Drug")])
def test_discovery_requires_structured_drug_identity(suspect_cli, kind, query, expected):
    run, state, _ = suspect_cli
    state["discovery"] = kind
    result = run("discover", query)
    assert result["concepts"][0]["primary_type"] == expected
    if expected == "Drug":
        assert "Example alias" in json.dumps(result["_meta"])
    else:
        assert "get drug" not in json.dumps(result)


def test_diagnostic_synonym_and_literal_matching_explain_missing_resolution(suspect_cli):
    run, state, requests = suspect_cli
    args = ("search", "diagnostic", "--source", "gtr", "--disease")
    resolved = run(*args, "Bachmann-Bupp syndrome")
    assert resolved["results"][0]["disease_match"]["kind"] == "synonym"
    assert "/disease/disease/MONDO:0033642" in requests
    state["disease"] = "empty"
    empty = run(*args, "Bachmann-Bupp syndrome")
    assert empty["count"] == 0
    assert "MyDisease" in empty["note"] and "literal" in empty["note"]
    assert "MyDisease" in run(*args, "Bachmann-Bupp syndrome", json_output=False)
    literal = run(*args, "neurodevelopmental disorder with alopecia and brain abnormalities")
    assert literal["results"][0]["disease_match"]["kind"] == "requested"
