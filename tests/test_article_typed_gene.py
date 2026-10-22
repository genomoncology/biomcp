"""Explain typed gene exclusions without promoting literal mentions to identities.

The literal reply is the study's recorded Europe PMC FANCL response. The
annotation-only empty reply is a controlled overlay of its observed zero count.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = pytest.mark.needs_binary
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("json_output,typed_matches", [(True, False), (False, False), (True, True)])
def test_typed_gene_date_search_explains_why_literal_mentions_can_be_excluded(
    tmp_path: Path, json_output: bool, typed_matches: bool,
) -> None:
    literal = json.loads((ROOT / "tests/fixtures/article/typed-gene/europepmc-literal.json").read_text())
    queries: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            query = parse_qs(urlsplit(self.path).query).get("query", [""])[0]
            queries.append(query)
            body = {"hitCount": 0, "resultList": {"result": []}} if "GENE_PROTEIN:" in query and not typed_matches else literal
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
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        outputs = []
        for index, args in enumerate((["--gene", "FANCL"], ["--keyword", 'FANCL OR "Fanconi anemia complementation group L"'])):
            command = [binary, *(["--json"] if json_output else []), "search", "article", *args,
                       "--date-from", "2026-09-01", "--source", "europepmc", "--limit", "50", "--debug-plan"]
            run = subprocess.run(command, capture_output=True, text=True, check=False, timeout=HANG_TIMEOUT,
                                 env=isolated_command_env(tmp_path / str(index), {"BIOMCP_EUROPEPMC_BASE": base}, base))
            assert run.returncode == 0, run.stderr + run.stdout
            outputs.append(run.stdout)
            print(f"COMMAND: {' '.join(command[1:])}\nOUTPUT: {run.stdout}")
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)

    assert len(queries) == 2, queries
    assert "GENE_PROTEIN:FANCL" in queries[0]
    assert "GENE_PROTEIN:" not in queries[1]
    assert all("FIRST_PDATE:[2026-09-01 TO *]" in query for query in queries)
    if json_output:
        typed, text = map(json.loads, outputs)
        assert typed["complete"] is True
        assert (typed["count"] > 0) is typed_matches
        assert text["count"] > 0
        assert "42850567" in {row["pmid"] for row in text["results"]}
        note = typed.get("note", "")
        if typed_matches:
            assert "GENE_PROTEIN" not in note
        else:
            assert "GENE_PROTEIN" in note, typed
            assert "annotation" in note and "literal" in note and "--keyword" in note
        assert "GENE_PROTEIN" not in text.get("note", "")
    else:
        assert "GENE_PROTEIN" in outputs[0]
        assert "--keyword" in outputs[0]
        assert "42850567" in outputs[1]
