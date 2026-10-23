"""Article date sorting uses publication dates before bounds and pagination."""
from __future__ import annotations

import json
import os
import subprocess
import threading
from datetime import UTC, datetime, timedelta
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import isolated_command_env
from hang_guard import HANG_TIMEOUT

pytestmark = pytest.mark.needs_binary
ROOT = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("mode", ["dates", "bounds", "offset", "failure", "citations", "single", "no_work", "retracted", "facets"])
def test_article_publication_dates_precede_merge_bounds_and_pagination(
    mode: str, tmp_path: Path,
) -> None:
    today = datetime.now(UTC).date()
    yesterday = today - timedelta(days=1)
    future = f"{today.year + 1}-12-31"
    issue = "2019-01-01" if mode == "no_work" else future
    dates = [issue, issue, issue, issue, str(today), "2020-02", "2099-02-30", None]
    rows = [{"pmid": str(90001 + i), "title": f"BRCA1 article {i}", "date": date}
            for i, date in enumerate(dates)]
    requests: list[list[str]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path == "/pubtator/search/":
                if mode == "retracted":
                    body = {"results": [], "count": 0}
                elif mode == "facets":
                    selected = json.loads(params.get("filters", ["{}"]) [0]).get("year")
                    if selected:
                        assert selected == [str(today.year)]
                        body = {"results": [rows[4]], "count": 1}
                    else:
                        body = {"results": [{**row, "date": "2019-01-01"} for row in rows],
                                "count": 9, "facets": {"facet_fields": {"year": [
                                    {"name": "2019", "value": 8},
                                    {"name": str(today.year), "value": 1},
                                ]}}}
                else:
                    body = {"results": rows, "count": len(rows)}
            elif target.path == "/europe/search":
                if mode == "facets":
                    hits = []
                elif mode == "retracted":
                    if "PUB_TYPE:" in params.get("query", [""])[0]:
                        hits = [{"id": "90009", "pmid": "90009", "title": "Retracted paper",
                                 "firstPublicationDate": "2000-01-01",
                                 "pubTypeList": {"pubType": ["retracted publication"]}}]
                    else:
                        hits = [{"id": row["pmid"], "pmid": row["pmid"],
                                 "title": row["title"], "firstPublicationDate": row["date"]}
                                for row in [rows[0], rows[4]]]
                else:
                    hits = [{
                    "id": "90001", "pmid": "90001", "title": rows[0]["title"],
                    "firstPublicationDate": issue, "citedByCount": 0,
                }]
                body = {"hitCount": len(hits), "resultList": {"result": hits}}
            elif target.path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"count": "0", "idlist": []}}
            elif target.path.endswith("esummary.fcgi"):
                ids = params["id"][0].split(",")
                requests.append(ids)
                if mode == "failure":
                    self.send_error(400)
                    return
                electronic = {
                    "90001": yesterday.strftime("%Y %b %d"),
                    "90002": "2010 Jan 01", "90003": "2099 Feb 30", "90004": "",
                }
                body = {"result": {"uids": ids, **{pmid: {
                    "uid": pmid, "title": f"BRCA1 metadata {pmid}",
                    "sortpubdate": future.replace("-", "/") + " 00:00",
                    "pubdate": f"{today.year + 1} Dec 31", "epubdate": electronic[pmid],
                    "history": [{"pubstatus": "entrez", "date": "2000/01/01 00:00"}],
                } for pmid in ids}}}
            elif target.path.startswith("/s2/"):
                body = {"total": 0, "data": []}
            else:
                misses.append(target.path)
                self.send_error(404)
                return
            self.reply(body)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            self.reply([None for _ in payload["ids"]])

        def reply(self, body: object) -> None:
            data = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_PUBTATOR_BASE": base + "/pubtator",
        "BIOMCP_EUROPEPMC_BASE": base + "/europe",
        "BIOMCP_PUBMED_BASE": base + "/pubmed", "BIOMCP_S2_BASE": base + "/s2",
    }, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    args = ["--sort", "citations" if mode == "citations" else "date", "--limit", "50"]
    if mode == "single":
        args += ["--source", "pubtator"]
    if mode in {"retracted", "facets"}:
        args = ["--sort", "date", "--limit", "1"]
    if mode == "facets":
        args += ["--date-from", str(today)]
    if mode == "bounds":
        args += ["--date-from", str(today - timedelta(days=2)), "--date-to", str(today)]
    if mode == "offset":
        args = ["--sort", "date", "--limit", "1", "--offset", "1"]
    try:
        result = subprocess.run(
            [os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp")),
             "--json", "--no-cache", "search", "article", "-k", "BRCA1", "--full", *args],
            env=env, capture_output=True, text=True, timeout=HANG_TIMEOUT, check=False,
        )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == 0, result.stdout + result.stderr
    assert not misses, misses
    payload = json.loads(result.stdout)
    actual = payload["results"]
    if mode == "facets":
        assert not requests, requests
        assert [row["pmid"] for row in actual] == ["90005"]
        return
    if mode == "retracted":
        assert requests == [["90001"]], requests
        assert [row["pmid"] for row in actual] == ["90005"]
        return
    if mode == "no_work":
        assert not requests, requests
        assert actual[0]["pmid"] == "90005"
        return
    if mode == "citations":
        assert not requests, requests
        assert actual[0]["date"] == future
        return
    assert requests == [["90001", "90002", "90003", "90004"]], requests
    if mode == "bounds":
        assert [row["pmid"] for row in actual] == ["90005", "90001"]
    elif mode == "offset":
        assert [row["pmid"] for row in actual] == ["90001"]
    elif mode == "failure":
        assert actual[0]["pmid"] == "90005"
        assert payload["complete"] is False
        assert any(status["source"] == "pubmed" and status.get("status") == "degraded"
                   for status in payload["_meta"]["source_status"])
        assert len(actual) == len(rows)
    else:
        assert [row["pmid"] for row in actual[:4]] == ["90005", "90001", "90006", "90002"]
        mapped = {row["pmid"]: row for row in actual}
        assert mapped["90001"]["date"] == str(yesterday)
        sources = ["pubtator"] if mode == "single" else ["pubtator", "europepmc"]
        assert mapped["90001"]["matched_sources"] == sources
        assert mapped["90006"]["date"] == "2020-02"
        assert mapped["90003"]["date"] == mapped["90004"]["date"] == future
        assert mapped["90007"]["date"] == "2099-02-30"
        assert len(actual) == len(rows)
