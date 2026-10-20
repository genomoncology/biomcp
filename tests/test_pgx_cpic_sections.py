"""CPIC section requests use provider column names and retain paging/errors."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest
from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("section,endpoint,order,output_key", [
    ("frequencies", "population_frequency_view", "genesymbol.asc,name.asc,population_group.asc", "allele"),
    ("guidelines", "guideline_summary_view", "guideline_name.asc", "name"),
])
def test_pgx_sections_use_supported_columns_for_pages_and_report_provider_errors(
    tmp_path: Path, section: str, endpoint: str, order: str, output_key: str,
) -> None:
    # Receipted provider rows, sorted and paged by the strict local provider.
    fixture = "frequency" if section == "frequencies" else "guideline"
    rows = json.loads((ROOT / f"testdata/sources/cpic/{fixture}_cyp2d6_20260803.json").read_text())
    columns = [part.split(".")[0] for part in order.split(",")]
    rows.sort(key=lambda row: tuple(row.get(column) or "" for column in columns))
    requests = []
    provider_failed = False

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            url = urlsplit(self.path)
            query = parse_qs(url.query)
            requests.append((url.path, query))
            offset = int(query.get("offset", ["0"])[0])
            expected = {"select": ["*"], "limit": ["2"], "offset": [str(offset)], "order": [order]}
            expected.update({"genesymbol": ["eq.CYP2D6"]} if section == "frequencies"
                            else {"genes": ['cs.[{"symbol":"CYP2D6"}]']})
            accepted = url.path == f"/{endpoint}" and query == expected and not provider_failed
            payload = rows[offset:offset + 2] if accepted else {"code": "42703", "message": "CPIC column is unavailable"}
            body = json.dumps(payload).encode()
            self.send_response(200 if accepted else 400)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Content-Range", f"{offset}-{offset + 1}/{len(rows)}")
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_CPIC_BASE": base}, base)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))

    def run(offset: int):
        return subprocess.run([binary, "--json", "get", "pgx", "CYP2D6", section,
                               "--limit", "1", "--offset", str(offset)],
                              env=env, text=True, capture_output=True, timeout=30, check=False)

    try:
        for offset in (0, 1):
            result = run(offset)
            assert result.returncode == 0, result.stdout + result.stderr
            card = json.loads(result.stdout)
            source_key = "name" if section == "frequencies" else "guideline_name"
            assert card[section][0][output_key] == rows[offset][source_key]
            page = card["section_pagination"][section]
            assert (page["offset"], page["returned"], page["next_offset"]) == (offset, 1, offset + 1)
        assert len(requests) == 2
        provider_failed = True
        failed = run(0)
        assert failed.returncode != 0
        assert "CPIC" in failed.stdout + failed.stderr
        assert json.loads(failed.stdout)["error"]["code"] == "api"
        assert len(requests) == 3
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
