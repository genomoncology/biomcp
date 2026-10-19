"""Name absence requires successful provider lookups."""
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


@pytest.mark.parametrize(
    "entity,failure,source",
    [
        ("disease", None, None),
        ("disease", "mydisease", "MyDisease.info"),
        ("disease", "ols", "OLS4"),
        ("drug", None, None),
        ("drug", "mychem_search", "MyChem.info"),
    ],
)
def test_name_lookup_distinguishes_provider_failure_from_absence(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, entity, failure, source
):
    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            target = urlsplit(self.path)
            fields = parse_qs(target.query).get("fields", [""])[0]
            failed = failure is not None and (
                target.path.startswith(f"/{failure}/")
                or (
                    failure == "mychem_search"
                    and target.path == "/mychem/query"
                    and "drugbank.drug_interactions" not in fields
                )
            )
            body = (
                {"error": "authored unavailable control"}
                if failed
                else {"response": {"docs": []}, "total": 0, "hits": [], "terms": [], "results": []}
            )
            self.send_response(503 if failed else 200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(body).encode())

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    for key, path in {
        "MYDISEASE": "mydisease", "OLS4": "ols", "MYGENE": "gene",
        "MYCHEM": "mychem", "OPENFDA": "openfda",
        "HPO": "hpo", "MEDLINEPLUS": "medline",
    }.items():
        monkeypatch.setenv(f"BIOMCP_{key}_BASE", f"{base}/{path}")
    monkeypatch.setenv("BIOMCP_TEST_UNPACED_ORIGIN", base)
    monkeypatch.setenv("HOME", str(tmp_path))
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    monkeypatch.setenv("BIOMCP_CACHE_MODE", "off")
    for key in ("UMLS_API_KEY", "NCBI_API_KEY", "OPENFDA_API_KEY", "BIOMCP_DISABLE_SOURCES"):
        monkeypatch.delenv(key, raising=False)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "get", entity, "--name", "authored absent condition"],
            capture_output=True, text=True, timeout=90,
        )
        assert result.returncode == 1, result.stderr + result.stdout
        payload = json.loads(result.stdout)
        print(json.dumps({"entity": entity, "failure": failure, "output": payload}))
        if source is None:
            assert payload["error"]["code"] == "not_found"
            assert payload["_meta"]["not_found"] is True
        else:
            assert payload["error"]["code"] != "not_found", payload
            assert payload["_meta"]["not_found"] is False
            assert payload["error"]["source"] == source
    finally:
        server.shutdown()
        server.server_close()
        thread.join()
