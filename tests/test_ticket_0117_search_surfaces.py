from __future__ import annotations

from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import threading
from typing import Iterator


ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/debug/biomcp"))
CTGOV_FIXTURE = b'{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Local CTGov search result"},"statusModule":{"overallStatus":"RECRUITING"}}}],"totalCount":1}'
NCI_FIXTURE = json.dumps(
    {
        "data": [
            {
                "nct_id": "NCT05929768",
                "brief_title": "Local NCI search result",
                "current_trial_status": "Active",
                "phase": "III",
                "diseases": [{"name": "Melanoma"}],
                "lead_org": "NCI fixture sponsor",
            }
        ],
        "total": 1,
    },
    separators=(",", ":"),
).encode()


@contextmanager
def _provider_server() -> Iterator[tuple[str, type[BaseHTTPRequestHandler]]]:
    class Handler(BaseHTTPRequestHandler):
        body = b"{}"
        status = 200

        def do_GET(self) -> None:  # noqa: N802
            self.send_response(type(self).status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(type(self).body)))
            self.end_headers()
            try:
                self.wfile.write(type(self).body)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}", Handler
    finally:
        server.shutdown()
        thread.join(timeout=5)
        server.server_close()


def _run(provider: str, base: str, *extra: str) -> subprocess.CompletedProcess[str]:
    env = os.environ | {
        "BIOMCP_TEST_UNPACED_ORIGIN": base,
        "BIOMCP_CACHE_DIR": str(ROOT / ".cache/ticket-0117-surface"),
    }
    if provider == "ctgov":
        env["BIOMCP_CTGOV_BASE"] = base
        query = ["--condition", "CLI-CONDITION-SENTINEL-0117"]
    else:
        env["BIOMCP_NCI_CTS_BASE"] = base
        env["NCI_API_KEY"] = "CLI-NCI-CREDENTIAL-SENTINEL-0117"
        query = ["--mutation", "BRAF"]
    return subprocess.run(
        [BINARY, *extra, "search", "trial", "--source", provider, *query, "--limit", "1"],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        timeout=20,
        check=False,
    )


def _assert_current_binary_surfaces(provider: str, valid: bytes, nct_id: str) -> None:
    with _provider_server() as (base, handler):
        handler.body = valid
        json_result = _run(provider, base, "--json")
        markdown_result = _run(provider, base)
        assert json_result.returncode == 0, json_result.stderr
        assert markdown_result.returncode == 0, markdown_result.stderr
        assert json.loads(json_result.stdout)["results"][0]["nct_id"] == nct_id
        assert nct_id in markdown_result.stdout
        public = json_result.stdout + json_result.stderr + markdown_result.stdout + markdown_result.stderr
        assert "CLI-NCI-CREDENTIAL-SENTINEL-0117" not in public

        malformed = f'{{"studies":[{{"protocolSection":{{"identificationModule":{{"nctId":"bad","briefTitle":"MALFORMED-{provider}-SENTINEL-0117"}},"statusModule":{{"overallStatus":"RECRUITING"}}}}}}]}}'.encode()
        if provider == "nci":
            malformed = f'{{"data":[{{"nct_id":"bad","brief_title":"MALFORMED-{provider}-SENTINEL-0117","current_trial_status":"Active"}}]}}'.encode()
        handler.body = malformed
        malformed_result = _run(provider, base, "--json")
        assert malformed_result.returncode != 0
        assert f"MALFORMED-{provider}-SENTINEL-0117" not in malformed_result.stdout + malformed_result.stderr

        limit_sentinel = f"LIMIT-{provider}-SENTINEL-0117"
        handler.body = limit_sentinel.encode() + b" " * (8 * 1024 * 1024 + 1)
        limited_result = _run(provider, base, "--json")
        assert limited_result.returncode != 0
        limited_public = limited_result.stdout + limited_result.stderr
        expected_source = (
            "ClinicalTrials.gov" if provider == "ctgov" else "NCI Clinical Trials Search"
        )
        assert expected_source in limited_public
        assert limit_sentinel not in limited_public


def test_ctgov_current_binary_json_markdown_errors_limits_and_privacy() -> None:
    _assert_current_binary_surfaces("ctgov", CTGOV_FIXTURE, "NCT41300001")
    with _provider_server() as (base, handler):
        sentinel = "CTGOV-PROVIDER-BODY-SENTINEL-0117"
        handler.status = 400
        handler.body = f"Error parsing query in Intervention / treatment: {sentinel}".encode()
        result = _run("ctgov", base, "--json")
        assert result.returncode != 0
        assert sentinel not in result.stdout + result.stderr


def test_nci_current_binary_json_markdown_errors_limits_and_privacy() -> None:
    _assert_current_binary_surfaces("nci", NCI_FIXTURE, "NCT05929768")
