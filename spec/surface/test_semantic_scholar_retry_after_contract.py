from __future__ import annotations

import json
import os
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

REPO_ROOT = Path(__file__).resolve().parents[2]
BIOMCP_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/release/biomcp"))
RETRY_AFTER_SECONDS = 2.0
RETRY_AFTER_TOLERANCE_SECONDS = 0.2
GUIDANCE = "Rate limited by Semantic Scholar. Set S2_API_KEY for a dedicated rate limit."


class _SemanticScholarState:
    def __init__(self, *, authenticated_retry_after: bool) -> None:
        self.authenticated_retry_after = authenticated_retry_after
        self.lock = threading.Lock()
        self.batch_seen_api_key: list[bool] = []
        self.citation_seen_api_key: list[bool] = []
        self.citation_times: list[float] = []


class _SemanticScholarHandler(BaseHTTPRequestHandler):
    server: "_SemanticScholarServer"

    def log_message(self, format: str, *args: Any) -> None:  # noqa: A002
        return

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path != "/graph/v1/paper/batch":
            self._send_json(404, {"error": "unexpected POST path"})
            return

        content_length = int(self.headers.get("content-length", "0"))
        if content_length:
            self.rfile.read(content_length)

        with self.server.state.lock:
            self.server.state.batch_seen_api_key.append("x-api-key" in self.headers)

        self._send_json(
            200,
            [
                {
                    "paperId": "paper-1",
                    "externalIds": {"PubMed": "22663011"},
                    "title": "Seed paper",
                    "venue": "Science",
                    "year": 2012,
                }
            ],
        )

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path != "/graph/v1/paper/paper-1/citations":
            self._send_json(404, {"error": "unexpected GET path"})
            return

        with self.server.state.lock:
            self.server.state.citation_times.append(time.monotonic())
            self.server.state.citation_seen_api_key.append("x-api-key" in self.headers)
            citation_attempt = len(self.server.state.citation_times)

        if self.server.state.authenticated_retry_after:
            if citation_attempt == 1:
                self._send_json(429, {"error": "slow down"}, {"Retry-After": "2"})
                return
            self._send_json(
                200,
                {
                    "data": [
                        {
                            "contexts": ["Recovered after Retry-After"],
                            "intents": ["Background"],
                            "isInfluential": False,
                            "citingPaper": {
                                "paperId": "paper-2",
                                "externalIds": {"PubMed": "24200969"},
                                "title": "Recovered after retry-after floor",
                                "venue": "Nature",
                                "year": 2024,
                            },
                        }
                    ]
                },
            )
            return

        self._send_json(429, {"error": "shared pool"})

    def _send_json(
        self, status: int, payload: object, headers: dict[str, str] | None = None
    ) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        for name, value in (headers or {}).items():
            self.send_header(name, value)
        self.end_headers()
        self.wfile.write(body)


class _SemanticScholarServer(ThreadingHTTPServer):
    state: _SemanticScholarState


class _RunningSemanticScholarServer:
    def __init__(self, *, authenticated_retry_after: bool) -> None:
        self.state = _SemanticScholarState(
            authenticated_retry_after=authenticated_retry_after
        )
        self.server = _SemanticScholarServer(("127.0.0.1", 0), _SemanticScholarHandler)
        self.server.state = self.state
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def __enter__(self) -> "_RunningSemanticScholarServer":
        self.thread.start()
        return self

    def __exit__(self, *exc: object) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=5)

    @property
    def base_url(self) -> str:
        host, port = self.server.server_address
        return f"http://{host}:{port}"


def _run_article_citations(
    server: _RunningSemanticScholarServer, *, api_key: str | None
) -> subprocess.CompletedProcess[str]:
    assert BIOMCP_BIN.exists(), f"missing biomcp binary: {BIOMCP_BIN}"
    with tempfile.TemporaryDirectory(prefix="biomcp-s2-retry-spec-") as cache_home:
        env = os.environ.copy()
        env.update(
            {
                "BIOMCP_S2_BASE": server.base_url,
                "BIOMCP_CACHE_MODE": "off",
                "XDG_CACHE_HOME": cache_home,
                "RUST_LOG": "error",
            }
        )
        if api_key is None:
            env.pop("S2_API_KEY", None)
        else:
            env["S2_API_KEY"] = api_key

        return subprocess.run(
            [str(BIOMCP_BIN), "article", "citations", "22663011", "--limit", "1"],
            cwd=REPO_ROOT,
            env=env,
            capture_output=True,
            text=True,
            timeout=20,
            check=False,
        )


def test_authenticated_semantic_scholar_retry_waits_for_retry_after() -> None:
    with _RunningSemanticScholarServer(authenticated_retry_after=True) as server:
        result = _run_article_citations(server, api_key="spec-test-key")

        assert result.returncode == 0, (
            "authenticated Semantic Scholar retry should recover after the local 429\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
        assert server.state.batch_seen_api_key and all(
            server.state.batch_seen_api_key
        ), "authenticated seed lookup should send x-api-key to Semantic Scholar"
        assert server.state.citation_seen_api_key and all(
            server.state.citation_seen_api_key
        ), "authenticated citation retries should keep x-api-key on every attempt"
        assert len(server.state.citation_times) >= 2, (
            "authenticated 429 should be retried and reach the recovery response"
        )

        second_retry_delay = server.state.citation_times[1] - server.state.citation_times[0]
        assert second_retry_delay >= RETRY_AFTER_SECONDS - RETRY_AFTER_TOLERANCE_SECONDS, (
            "authenticated Semantic Scholar 429 retried before the Retry-After floor: "
            f"observed {second_retry_delay:.3f}s, required at least "
            f"{RETRY_AFTER_SECONDS - RETRY_AFTER_TOLERANCE_SECONDS:.3f}s"
        )


def test_unauthenticated_semantic_scholar_shared_pool_429_fails_fast_without_retrying() -> None:
    with _RunningSemanticScholarServer(authenticated_retry_after=False) as server:
        result = _run_article_citations(server, api_key=None)

        assert result.returncode != 0, (
            "shared-pool Semantic Scholar 429 should fail with guidance, not recover via retry"
        )
        assert GUIDANCE in result.stderr, (
            "shared-pool 429 should keep the dedicated-key recovery guidance\n"
            f"stderr:\n{result.stderr}"
        )
        assert server.state.batch_seen_api_key and not any(
            server.state.batch_seen_api_key
        ), "unauthenticated seed lookup should not send x-api-key"
        assert server.state.citation_seen_api_key and not any(
            server.state.citation_seen_api_key
        ), "unauthenticated citation request should not send x-api-key"
        assert len(server.state.citation_times) == 1, (
            "shared-pool 429 should be converted to a fast-fail error without retrying"
        )
