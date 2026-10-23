"""Article continuations retain a session and refuse an unavailable pool."""
from __future__ import annotations

import json
import base64
import hashlib
import sqlite3
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit, parse_qs

import pytest

from hang_guard import HANG_TIMEOUT

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, retained_cache_env  # noqa: E402

pytestmark = pytest.mark.needs_binary


@pytest.fixture
def article_pool_replay(tmp_path):
    state = {"requests": [], "late": False, "ready": False}

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):  # noqa: N802
            state["requests"].append(self.path)
            path = urlsplit(self.path).path
            if path == "/search":
                query = parse_qs(urlsplit(self.path).query).get("query", [""])[0]
                pmids = (201, 202, 203) if "EGFR" in query else (101, 102, 103)
                body = {"hitCount": 3, "resultList": {"result": [
                    {"id": str(pmid), "source": "MED", "pmid": str(pmid),
                     "title": f"BRAF V600E paper {pmid}", "pubYear": "2020"}
                    for pmid in pmids]}}
            elif path.endswith("esearch.fcgi"):
                body = {"esearchresult": {"count": "0", "idlist": []}}
            elif path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif path.endswith("/search/bulk"):
                if state["late"] and not state["ready"]:
                    self.send_error(503, "authored unfinished source")
                    return
                if state["late"]:
                    body = {"total": 1, "data": [{
                        "paperId": "late-paper", "externalIds": {"PubMed": "104"},
                        "title": "BRAF V600E late paper", "abstract": "BRAF V600E evidence",
                        "citationCount": 1, "influentialCitationCount": 1,
                    }]}
                else:
                    body = {"total": 0, "data": []}
            else:
                self.send_error(404)
                return
            self.reply(body)

        def do_POST(self):  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            self.reply([None, None, None])

        def reply(self, body):
            data = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = retained_cache_env(tmp_path, {
        name: origin for name in ("BIOMCP_EUROPEPMC_BASE", "BIOMCP_PUBMED_BASE",
                                 "BIOMCP_PUBTATOR_BASE", "BIOMCP_S2_BASE")}, origin)
    try:
        yield env, state
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)


def test_article_tokens_bind_the_pool_and_request_configuration(article_pool_replay):
    env, _state = article_pool_replay
    env["BIOMCP_TEST_RETRY_SLEEP_OFF"] = "1"
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))

    def invoke(*args, command_env=None):
        return subprocess.run(
            [binary, "--json", "variant", "articles", "BRAF V600E", "--strategy", "exact",
             "--limit", "1", *args], env=command_env or env, capture_output=True,
            text=True, timeout=HANG_TIMEOUT,
        )

    first = invoke()
    assert first.returncode == 0, first.stdout + first.stderr
    first = json.loads(first.stdout)
    token = first["pagination"]["next_page_token"]
    assert token, "every stored article session supplies its bound continuation"
    seen = [row["pmid"] for row in first["results"]]
    while first["pagination"]["has_more"]:
        following = invoke("--page-token", token)
        assert following.returncode == 0, following.stdout + following.stderr
        first = json.loads(following.stdout)
        assert first["pagination"]["total"] == 3
        seen.extend(row["pmid"] for row in first["results"])
        token = first["pagination"]["next_page_token"]
    assert len(seen) == len(set(seen)) == 3
    empty = invoke("--page-token", token)
    assert empty.returncode == 0, empty.stdout + empty.stderr
    assert json.loads(empty.stdout)["results"] == []
    assert json.loads(empty.stdout)["pagination"]["has_more"] is False
    conflict = invoke("--page-token", token, "--verify-identity")
    assert conflict.returncode != 0
    assert "conflict" in (conflict.stdout + conflict.stderr).lower()
    changed_config = invoke("--page-token", token,
                            command_env=dict(env, BIOMCP_DEFAULT_ASSEMBLY="GRCh37"))
    assert changed_config.returncode != 0
    assert "conflict" in changed_config.stdout + changed_config.stderr
    bypass = dict(env, BIOMCP_CACHE_MODE="off")
    expired = invoke("--page-token", token, command_env=bypass)
    assert expired.returncode != 0
    assert "offset 0" in expired.stdout + expired.stderr
    missing = invoke("--offset", "1", command_env=bypass)
    assert missing.returncode != 0
    assert "offset 0" in missing.stdout + missing.stderr


def test_typed_article_items_continue_distinct_pools(article_pool_replay):
    from test_mcp_stdio_guidance import StdioMcp, _initialize

    client = StdioMcp(article_pool_replay[0])
    sequence = 0

    def articles(items):
        nonlocal sequence
        sequence += 1
        response = client.call({
            "jsonrpc": "2.0", "id": sequence, "method": "tools/call",
            "params": {"name": "variant_articles", "arguments": {
                "items": items, "strategy": "exact", "limit": 1,
            }},
        })
        result = response["result"]
        assert not result.get("isError"), result
        return json.loads(result["content"][0]["text"])["items"]

    try:
        _initialize(client)
        requests = [{"request_id": "braf", "gene": "BRAF", "protein": "V600E"},
                    {"request_id": "egfr", "gene": "EGFR", "protein": "L858R"}]
        first = articles(requests)
        tokens = [item["pagination"]["next_page_token"] for item in first]
        assert all(tokens) and tokens[0] != tokens[1]
        following = articles([dict(request, page_token=token)
                              for request, token in zip(requests, tokens)])
        for initial, next_page in zip(first, following):
            assert initial["request_id"] == next_page["request_id"]
            assert next_page["pagination"]["offset"] == 1
            assert next_page["results"][0]["pmid"] != initial["results"][0]["pmid"]
        assert following[0]["results"][0]["pmid"] != following[1]["results"][0]["pmid"]
    finally:
        client.close()


@pytest.mark.parametrize("corruption", ["expired", "version", "atom"])
def test_article_continuation_refuses_invalid_stored_pool(article_pool_replay, corruption):
    env, state = article_pool_replay
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    command = [binary, "--json", "variant", "articles", "BRAF V600E",
               "--strategy", "exact", "--limit", "1"]
    first = subprocess.run(command, env=env, capture_output=True, text=True,
                           timeout=HANG_TIMEOUT)
    assert first.returncode == 0, first.stdout + first.stderr
    token = json.loads(first.stdout)["pagination"]["next_page_token"]
    binding = json.loads(base64.urlsafe_b64decode(token + "=" * (-len(token) % 4)))
    with sqlite3.connect(Path(env["BIOMCP_CACHE_DIR"]) / "biomcp.db") as database:
        body, = database.execute(
            "SELECT body FROM entries WHERE namespace='variant-pool' AND key=?",
            (binding["snapshot"],),
        ).fetchone()
        snapshot = json.loads(body)
        if corruption == "expired":
            snapshot["expires_ms"] = 0
        elif corruption == "version":
            snapshot["schema_version"] = 999
        else:
            snapshot["payload"]["context"]["retrieval_path"] = "unknown-pool-label"
        encoded = json.dumps(snapshot, separators=(",", ":")).encode()
        database.execute(
            "UPDATE entries SET body=?, body_hash=? WHERE namespace='variant-pool' AND key=?",
            (encoded, hashlib.sha256(encoded).hexdigest(), binding["snapshot"]),
        )
    before = len(state["requests"])
    refused = subprocess.run([*command, "--page-token", token], env=env,
                             capture_output=True, text=True, timeout=HANG_TIMEOUT)
    assert refused.returncode != 0
    assert "offset 0" in refused.stdout + refused.stderr
    assert len(state["requests"]) == before, "invalid continuations must not rebuild a pool"


def test_late_source_publishes_successor_without_switching_bound_or_active_pool(article_pool_replay):
    env, state = article_pool_replay
    state["late"] = True
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))

    def page(*arguments):
        process = subprocess.run(
            [binary, "--json", "variant", "articles", "BRAF V600E", "--strategy", "exact",
             "--limit", "1", *arguments], env=env, capture_output=True, text=True,
            timeout=HANG_TIMEOUT,
        )
        assert process.returncode == 0, process.stdout + process.stderr
        return json.loads(process.stdout)

    initial = page()
    assert initial["complete"] is False
    assert initial["pagination"]["total"] == 3
    token = initial["pagination"]["next_page_token"]
    state["ready"] = True
    pinned = page("--page-token", token)
    assert pinned["pagination"]["total"] == 3
    assert pinned["complete"] is False
    legacy = page("--offset", "1")
    assert legacy["pagination"]["total"] == 3
    assert legacy["results"][0]["pmid"] == pinned["results"][0]["pmid"]
    successor = page()
    assert successor["pagination"]["total"] == 4
    assert successor["complete"] is True
    assert successor["pagination"]["next_page_token"] != token
    old = page("--page-token", token)
    assert old["pagination"]["total"] == 3
    assert old["results"][0]["pmid"] == pinned["results"][0]["pmid"]
    switched = page("--offset", "1")
    assert switched["pagination"]["total"] == 4
