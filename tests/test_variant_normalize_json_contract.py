from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


class EmptyNormalizeHandler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802 - stdlib callback name
        body = b"{}"
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, format: str, *args: object) -> None:
        return


def test_variant_normalize_json_no_result_emits_parseable_non_empty_stdout() -> None:
    server = ThreadingHTTPServer(("127.0.0.1", 0), EmptyNormalizeHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base_url = f"http://127.0.0.1:{server.server_port}"
    env = os.environ.copy()
    env["BIOMCP_MUTALYZER_BASE_URL"] = base_url
    env["BIOMCP_VARIANTVALIDATOR_BASE_URL"] = base_url

    try:
        result = subprocess.run(
            [
                "cargo",
                "run",
                "--quiet",
                "--bin",
                "biomcp",
                "--",
                "variant",
                "normalize",
                "all",
                "NM_000000.0:c.1A>T",
                "--json",
            ],
            cwd=REPO_ROOT,
            env=env,
            capture_output=True,
            text=True,
            check=False,
            timeout=60,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)

    assert result.returncode == 0, result.stderr
    assert result.stdout.strip(), "successful JSON normalize command must not emit empty stdout"
    payload = json.loads(result.stdout)
    assert payload["status"] == "no_result"
    assert payload["results"] == []
    assert "No normalized variant result" in payload["message"]
    assert payload["_meta"]["next_commands"]
