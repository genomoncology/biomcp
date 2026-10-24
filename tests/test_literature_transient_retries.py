"""Literature searches retry a transient leg once and preserve the other legs."""
from __future__ import annotations

import json
from datetime import datetime, timedelta, timezone
from email.utils import format_datetime
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

import pytest

from hang_guard import HANG_TIMEOUT

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env, retained_cache_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]


@pytest.mark.parametrize("provider,failure,recover,surface", [
    ("europepmc", failure, recover, "search")
    for failure in (503, 502, 429, 400, "timeout", "retry_after_date", "retry_after_too_long")
    for recover in (True, False)
] + [("pubtator", 503, False, "search"), ("semanticscholar", 503, False, "search"),
     ("semanticscholar", "shared429", False, "search"),
     ("semanticscholar", "shared429_after_retry", False, "search"),
     ("semanticscholar", "batch503", False, "search"),
     ("europepmc", 503, False, "variant"), ("europepmc", 400, False, "variant")])
@pytest.mark.parametrize("cached", [False, True])
def test_literature_search_retries_one_transient_leg(
    provider: str, failure: int | str, recover: bool, surface: str, cached: bool, tmp_path: Path,
) -> None:
    """Feature: provider-retry. A failed leg cannot hide another source's paper."""
    attempts: list[str] = []
    attempt_dates: list[datetime] = []
    retry_date: datetime | None = None
    release = threading.Event()
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            nonlocal retry_date
            path = urlsplit(self.path).path
            status = 200
            failed_path = {"europepmc": "/europe/search", "pubtator": "/pubtator/search/",
                           "semanticscholar": "/s2/graph/v1/paper/search"}[provider]
            if path == failed_path and failure != "batch503":
                attempts.append(self.path)
                attempt_dates.append(datetime.now(timezone.utc))
                ordinal = len(attempts)
                if failure == "timeout" and (ordinal == 1 or not recover):
                    # The command's return releases a held reply. No fixed sleep
                    # races the loaded host or pretends a transport timeout.
                    release.wait(HANG_TIMEOUT)
                    return
                release.set()
                if ordinal == 1 or not recover:
                    status = (503 if ordinal == 1 else 429) if failure == "shared429_after_retry" else (
                        429 if str(failure).startswith("retry_after_") or failure == "shared429" else int(failure))
            if path == "/europe/search":
                body = {"hitCount": 1, "resultList": {"result": [{"id": "12345678",
                        "pmid": "12345678", "title": "Other route paper",
                        "firstPublicationDate": "2026-01-01", "abstractText": "Other route abstract",
                        "citedByCount": 7}]}}
            elif path == "/pubtator/search/":
                body = {"results": [{"pmid": "12345678", "title": "Other route paper",
                                     "date": "2026-01-01"}], "count": 1,
                        "total_pages": 1, "current": 1}
            elif path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"idlist": [], "count": "0"}}
            elif path.startswith("/s2/"):
                body = {"data": [], "total": 0}
            else:
                misses.append(self.path)
                self.send_error(404)
                return
            payload = json.dumps(body).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            if status == 429:
                if failure == "retry_after_date":
                    retry_date = (datetime.now(timezone.utc) + timedelta(seconds=1)).replace(microsecond=0)
                    self.send_header("Retry-After", format_datetime(retry_date, usegmt=True))
                else:
                    self.send_header("Retry-After", "9" * 40 if failure == "retry_after_too_long" else "0")
            self.end_headers()
            try:
                self.wfile.write(payload)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def do_POST(self) -> None:  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            payload = json.dumps([{"paperId": "authored-paper", "externalIds": {"PubMed": "12345678"},
                                   "abstract": "Other route abstract", "citationCount": 7}]).encode()
            status = 200
            if failure == "batch503":
                attempts.append(self.path)
                status = 503
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {"BIOMCP_EUROPEPMC_BASE": origin + "/europe",
                 "BIOMCP_PUBTATOR_BASE": origin + "/pubtator",
                 "BIOMCP_PUBMED_BASE": origin + "/pubmed",
                 "BIOMCP_S2_BASE": origin + "/s2"}
    env_factory = retained_cache_env if cached else isolated_command_env
    env = env_factory(tmp_path / "isolated", providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env.pop("BIOMCP_TEST_RETRY_SLEEP_OFF", None)
    if not str(failure).startswith("shared429"):
        env["S2_API_KEY"] = "fixture-only-key"
    # Only deliberate stalled replies need shortened deadlines. HTTP retry
    # cases keep the product budgets for cache and provider work on shared hosts.
    if failure == "timeout":
        env["BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS"] = "3000"
        env["BIOMCP_TEST_FEDERATED_ARTICLE_SOURCE_TIMEOUT_MS"] = "2000"
    command = [os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
               "--no-cache", "--json", "search", "article", "--keyword", "BRAF V600E",
               "--limit", "1"]
    if surface == "variant":
        command = command[:3] + ["variant", "articles", "BRAF V600E", "--strategy", "exact", "--limit", "1"]
    if cached:
        command.remove("--no-cache")
    if failure == "timeout":
        command += ["--date-from", "2025-01-01"]
    try:
        result = subprocess.run(command, env=env, cwd=REPO_ROOT, text=True,
                                capture_output=True, timeout=HANG_TIMEOUT)
    finally:
        release.set()
        server.shutdown()
        server.server_close()
        worker.join()
    assert result.returncode == 0, result.stdout + result.stderr
    payload = json.loads(result.stdout)
    print(json.dumps({"failure": failure, "recover": recover,
                      "attempts": attempts, "payload": payload}))
    assert any(row["pmid"] == "12345678" for row in payload["results"]), payload
    cannot_retry = failure in (400, "retry_after_too_long", "shared429")
    assert len(attempts) == (1 if cannot_retry else 2), attempts
    if failure == "retry_after_date" and recover:
        assert retry_date is not None
        assert attempt_dates[1] >= retry_date, (attempt_dates, retry_date)
    statuses = (payload.get("source_status", []) if surface == "variant" else
                payload.get("_meta", {}).get("source_status", []))
    failed = [row for row in statuses if row["source"] == provider
              and row.get("status") in ("degraded", "unavailable")]
    if recover and not cannot_retry:
        assert not failed, failed
        assert payload["complete"] is True, payload
    else:
        assert payload["complete"] is False, payload
        text = json.dumps(failed)
        expected_class = "rate_limited" if str(failure).startswith("shared429") else (
            "timeout" if failure == "timeout" else (
                "http_429" if str(failure).startswith("retry_after_") else "http_503" if failure == "batch503" else f"http_{failure}"))
        if surface == "variant":
            reasons = {reason for row in failed for reason in row["reason_codes"]}
            assert expected_class in reasons, failed
            assert ("retry_attempted" in reasons) is (not cannot_retry), failed
        else:
            assert f"failure_class={expected_class}" in text
            assert f"retry_attempted={'false' if cannot_retry else 'true'}" in text
        if str(failure).startswith("shared429"):
            assert "S2_API_KEY" in text
    assert not misses, misses
