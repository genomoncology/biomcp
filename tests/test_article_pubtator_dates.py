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
    "bounds,years,pmids,facet_kind",
    [
        (["--date-from", "2026-09-01"], ["2026", "2028"], ["90004", "90005"], "complete"),
        (["--date-to", "2025-12-31"], ["2019", "2025"], ["90001", "90002"], "complete"),
        (["--date-from", "2025-09-01", "--date-to", "2027-12-31"],
         ["2025", "2026"], ["90002", "90003", "90004"], "complete"),
        (["--date-from", "2030-01-01"], [], [], "complete"),
        (["--date-from", "2026-09-01"], ["2026", "2028"], ["90004", "99001"], "timeout"),
        *[(["--date-from", "2026-09-01"], None, ["90004", "90005"], kind)
          for kind in ("missing", "incomplete", "duplicate", "invalid", "zero", "overflow", "negative", "fraction", "unknown")],
    ],
)
def test_dated_gene_search_uses_complete_year_facets_without_a_clock_bound(
    bounds: list[str], years: list[str] | None, pmids: list[str], facet_kind: str,
    tmp_path: Path,
) -> None:
    requests: list[dict[str, list[str]]] = []
    release = threading.Event()
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path == "/entity/autocomplete/":
                body = [{"_id": "@GENE_FANCL", "name": "FANCL", "biotype": "gene"}]
            elif target.path == "/search/":
                requests.append(params)
                selected = json.loads(params.get("filters", ["{}"]) [0]).get("year")
                if selected is not None and facet_kind == "timeout":
                    release.wait(HANG_TIMEOUT)
                    return
                if selected is not None:
                    rows = [row for row in reversed(ROWS) if row["date"][:4] in selected]
                    body = {"results": rows, "count": len(rows)}
                elif params["page"] == ["1"]:
                    body = {
                        "results": [ROWS[0], ROWS[3]], "count": 1508,
                        "facets": {"facet_fields": {"year": [
                            {"name": "2019", "value": 1504},
                            {"name": "2025", "value": 1},
                            {"name": "2026", "value": 2},
                            {"name": "2028", "value": 1},
                        ]}},
                    }
                    buckets = body["facets"]["facet_fields"]["year"]
                    if facet_kind == "missing":
                        body["facets"] = {}
                    elif facet_kind == "incomplete":
                        buckets[0]["value"] -= 1
                    elif facet_kind == "duplicate":
                        buckets[0]["value"] -= 1
                        buckets.append({"name": "2019", "value": 1})
                    elif facet_kind == "invalid":
                        buckets[0]["name"] = "unknown"
                    elif facet_kind in {"zero", "overflow", "negative", "fraction"}:
                        buckets[0]["value"] = {
                            "zero": 0, "overflow": 2**64 - 1, "negative": -1, "fraction": 1.5,
                        }[facet_kind]
                    elif facet_kind == "unknown":
                        body["count"] = None
                elif years is None:
                    body = {"results": ROWS if params["page"] == ["2"] else [], "count": 1508}
                else:
                    self.send_error(400, "Historical-page churn is unnecessary")
                    return
            elif target.path == "/europe/search":
                body = {"hitCount": 1, "resultList": {"result": [{
                    "id": "99001", "pmid": "99001", "title": "FANCL companion article",
                    "firstPublicationDate": "2026-10-01", "abstractText": "Companion abstract",
                    "citedByCount": 0,
                }]}}
            elif target.path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif target.path.startswith("/s2/"):
                body = {"total": 0, "data": []}
            else:
                misses.append(target.path)
                self.send_error(404)
                return
            payload = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            body = json.dumps([None for _ in payload["ids"]]).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_PUBTATOR_BASE": base}, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    if facet_kind == "timeout":
        env.update({
            "BIOMCP_EUROPEPMC_BASE": base + "/europe",
            "BIOMCP_PUBMED_BASE": base + "/pubmed",
            "BIOMCP_S2_BASE": base + "/s2",
            "NCBI_API_KEY": "fixture-only-ncbi-secret",
            "S2_API_KEY": "fixture-only-s2-secret",
            "BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS": "2000",
        })
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "--no-cache", "search", "article", "-g", "FANCL",
             "--limit", "50", *([] if facet_kind == "timeout" else ["--source", "pubtator"]),
             *bounds],
            env=env, text=True, capture_output=True, timeout=HANG_TIMEOUT, check=False,
        )
    finally:
        release.set()
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, f"{result.stdout}{result.stderr} requests={requests}"
    response = json.loads(result.stdout)
    assert sorted(row["pmid"] for row in response["results"]) == pmids
    if facet_kind == "timeout":
        assert response["complete"] is False
        statuses = response.get("_meta", {}).get("source_status", [])
        status = next(status for status in statuses if status["source"] == "pubtator")
        assert "timeout" in status["message"], status
        description = status["message"].split("request=GET /search/ ", 1)[1]
        sent = json.loads(description)
        assert json.loads(sent["filters"]) == {"year": years}
        assert sent["text"] == "@GENE_FANCL" and sent["page"] == "1"
        assert len(requests) >= 2
        assert all("filters" in request for request in requests[1:])
        assert "fixture-only" not in result.stdout + result.stderr
    elif years is None:
        assert len(requests) == 3
        assert all("filters" not in request for request in requests)
        assert [request["page"] for request in requests] == [["1"], ["2"], ["3"]]
    elif years:
        assert [row["pmid"] for row in response["results"]] == list(reversed(pmids))
        assert response["pagination"]["total"] == len(pmids)
        assert response["pagination"]["has_more"] is False
        assert len(requests) == 2
        assert json.loads(requests[1]["filters"][0]) == {"year": years}
    else:
        assert len(requests) == 1
    assert not misses, misses
    assert all(request["text"] == ["@GENE_FANCL"] for request in requests)
    assert all("sort" not in request for request in requests)
    if years is not None:
        assert all(request["page"] == ["1"] for request in requests)
