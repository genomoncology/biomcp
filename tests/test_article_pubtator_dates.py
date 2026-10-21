"""Dated typed searches avoid historical pages without changing relevance.

Authored replies use the live provider's complete year-facet shape. They are
controls for request selection, not recordings of these synthetic articles.
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

from golden.replay_env import isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = pytest.mark.needs_binary
ROOT = Path(__file__).resolve().parents[1]
ROWS = [
    {"pmid": str(90001 + i), "title": f"FANCL article {i}", "date": date}
    for i, date in enumerate([
        "2019-01-01", "2025-10-01", "2026-01-01", "2026-10-01", "2028-01-01",
    ])
]


@pytest.mark.parametrize(
    "bounds,years,pmids",
    [
        (["--date-from", "2026-09-01"], ["2026", "2028"], ["90004", "90005"]),
        (["--date-to", "2025-12-31"], ["2019", "2025"], ["90001", "90002"]),
        (["--date-from", "2025-09-01", "--date-to", "2027-12-31"],
         ["2025", "2026"], ["90002", "90003", "90004"]),
    ],
)
def test_dated_gene_search_uses_complete_year_facets_without_a_clock_bound(
    bounds: list[str], years: list[str], pmids: list[str], tmp_path: Path,
) -> None:
    requests: list[dict[str, list[str]]] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path == "/entity/autocomplete/":
                body = [{"_id": "@GENE_FANCL", "name": "FANCL", "biotype": "gene"}]
            elif target.path == "/search/":
                requests.append(params)
                selected = json.loads(params.get("filters", ["{}"]) [0]).get("year")
                if selected is not None:
                    rows = [row for row in ROWS if row["date"][:4] in selected]
                    body = {"results": rows, "count": len(rows)}
                elif params["page"] == ["1"]:
                    body = {
                        "results": [ROWS[0]], "count": 1508,
                        "facets": {"facet_fields": {"year": [
                            {"name": "2019", "value": 1504},
                            {"name": "2025", "value": 1},
                            {"name": "2026", "value": 2},
                            {"name": "2028", "value": 1},
                        ]}},
                    }
                else:
                    self.send_error(400, "Historical-page churn is unnecessary")
                    return
            else:
                self.send_error(404)
                return
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
    env = isolated_command_env(tmp_path, {"BIOMCP_PUBTATOR_BASE": base}, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "--no-cache", "search", "article", "-g", "FANCL",
             "--source", "pubtator", "--limit", "50", *bounds],
            env=env, text=True, capture_output=True, timeout=HANG_TIMEOUT, check=False,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stdout + result.stderr
    response = json.loads(result.stdout)
    assert sorted(row["pmid"] for row in response["results"]) == pmids
    assert len(requests) == 2
    assert json.loads(requests[1]["filters"][0]) == {"year": years}
    assert all(request["text"] == ["@GENE_FANCL"] for request in requests)
    assert all("sort" not in request for request in requests)
    assert all(request["page"] == ["1"] for request in requests)
