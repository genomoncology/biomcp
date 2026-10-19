"""Graph pages distinguish publisher withholding from empty data and failures."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "golden"))
from replay_env import isolated_command_env  # noqa: E402

pytestmark = pytest.mark.needs_binary
REPO_ROOT = Path(__file__).resolve().parents[2]
BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))


@pytest.mark.parametrize("direction", ["references", "citations"])
@pytest.mark.parametrize("json_output", [True, False])
@pytest.mark.parametrize(
    "case",
    [
        "elided",
        "empty",
        "malformed",
        "failure",
        "null",
        "null_offset",
        "elided_bad_offset",
        "missing_data_offset",
    ],
)
def test_graph_page_reports_provider_outcome(
    tmp_path: Path, direction: str, json_output: bool, case: str
) -> None:
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args: object) -> None:
            pass

        def send_json(self, status: int, body: object) -> None:
            raw = json.dumps(body).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)

        def do_POST(self) -> None:
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.send_json(
                200, [{"paperId": "fixture-paper", "title": "Fixture paper"}]
            )

        def do_GET(self) -> None:
            if case == "failure":
                self.send_json(400, {"error": "provider refused request"})
            elif case in {"elided", "elided_bad_offset"}:
                key = (
                    "citingPaperInfo" if direction == "references" else "citedPaperInfo"
                )
                self.send_json(
                    200,
                    {
                        **({"offset": 1} if case == "elided_bad_offset" else {}),
                        "data": None,
                        key: {
                            "openAccessPdf": {
                                "disclaimer": "Notice: The following paper fields have been elided "
                                f"by the publisher: {{'{direction}'}}."
                            }
                        },
                    },
                )
            elif case == "missing_data_offset":
                self.send_json(200, {"offset": 0})
            elif case == "null_offset":
                self.send_json(200, {"offset": 0, "data": None})
            elif case == "empty":
                self.send_json(200, {"offset": 0, "data": []})
            else:
                self.send_json(200, {"data": None if case == "null" else []})

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_S2_BASE": origin}, origin)
    args = [str(BINARY)] + (["--json"] if json_output else [])
    args += ["article", direction, "24200969", "--limit", "5"]
    try:
        result = subprocess.run(
            args, env=env, capture_output=True, text=True, timeout=20
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    if case in {
        "failure",
        "malformed",
        "null",
        "null_offset",
        "elided_bad_offset",
        "missing_data_offset",
    }:
        assert result.returncode != 0
        assert "Semantic Scholar" in result.stdout + result.stderr
        assert "BioMCP source" not in result.stdout + result.stderr
        return
    assert result.returncode == 0, result.stdout + result.stderr
    if json_output:
        payload = json.loads(result.stdout)
        assert payload["edges"] == []
        assert payload["pagination"]["returned"] == 0
        assert payload["pagination"]["next_offset"] is None
        assert payload["_meta"]["next_commands"] == []
        if case == "elided":
            assert payload["pagination"]["coverage_status"] == "unavailable"
            assert "Semantic Scholar" in payload["_meta"]["note"]
            assert "publisher" in payload["_meta"]["note"]
            assert direction in payload["_meta"]["note"]
        else:
            assert payload["pagination"]["coverage_status"] == "exhausted"
            assert "note" not in payload["_meta"]
    else:
        assert "No related papers returned" in result.stdout
        if case == "elided":
            assert "Semantic Scholar" in result.stdout
            assert "publisher" in result.stdout
            assert "elided" in result.stdout
            assert "coverage: unavailable" in result.stdout
        else:
            assert "coverage: exhausted" in result.stdout
            assert "elided" not in result.stdout
