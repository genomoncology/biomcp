from __future__ import annotations

import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


class CountingHandler(BaseHTTPRequestHandler):
    requests = 0

    def do_GET(self) -> None:  # noqa: N802
        type(self).requests += 1
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"data":[],"total":0}')

    def log_message(self, *_args: object) -> None:
        pass


def test_rejected_nci_filters_never_reach_local_transport() -> None:
    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
    assert binary.exists(), f"missing biomcp binary: {binary}"
    server = ThreadingHTTPServer(("127.0.0.1", 0), CountingHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    env = os.environ | {
        "NCI_API_KEY": "test-key",
        "BIOMCP_NCI_CTS_BASE": f"http://127.0.0.1:{server.server_port}",
    }
    rejected = [
        ["--study-type", "interventional"],
        ["--sponsor", "NCI"],
        ["--date-from", "2026-01-01"],
        ["--date-to", "2026-01-01"],
        ["--biomarker", "BRAF", "--mutation", "V600E"],
        ["--biomarker", "BRAF", "V600E"],
    ]
    try:
        for filters in rejected:
            result = subprocess.run(
                [binary, "search", "trial", "--source", "nci", *filters],
                cwd=REPO_ROOT,
                env=env,
                text=True,
                capture_output=True,
                check=False,
            )
            assert result.returncode != 0, (filters, result.stdout, result.stderr)
        assert CountingHandler.requests == 0
    finally:
        server.shutdown()
        thread.join()
        server.server_close()
