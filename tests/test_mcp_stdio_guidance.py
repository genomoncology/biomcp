from __future__ import annotations

import json
import os
import select
import subprocess
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

import pytest

sys.path.insert(0, str(Path(__file__).parent / "golden"))
from replay_env import isolated_command_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]  # docs-only CI excludes this module

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "release" / "biomcp"))


class StdioMcp:
    def __init__(self, env: dict[str, str] | None = None) -> None:
        assert RELEASE_BIN.exists(), f"missing BioMCP binary: {RELEASE_BIN}"
        self.process = subprocess.Popen(
            [str(RELEASE_BIN), "serve"],
            cwd=REPO_ROOT,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            env=env,
        )

    def call(self, request: dict[str, object], timeout: float = 3) -> dict[str, Any]:
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
        self.process.stdin.flush()

        ready, _, _ = select.select([self.process.stdout], [], [], timeout)
        assert ready, "timed out waiting for an MCP response"
        line = self.process.stdout.readline()
        if not line:
            assert self.process.stderr is not None
            stderr = self.process.stderr.read()
            pytest.fail(
                f"BioMCP exited before returning an MCP response: {stderr.strip()}"
            )
        response = json.loads(line)
        assert response["id"] == request["id"]
        return response

    def close(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=2)


@pytest.fixture
def stdio_mcp() -> StdioMcp:
    client = StdioMcp()
    try:
        yield client
    finally:
        client.close()


@pytest.mark.parametrize("command", ["mcp", "serve"])
def test_stdio_no_input_prints_recovery_guidance(command: str) -> None:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    env = dict(os.environ)
    env.pop("RUST_LOG", None)

    result = subprocess.run(
        [str(RELEASE_BIN), command],
        cwd=REPO_ROOT,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )

    assert result.returncode != 0
    assert result.stdout == ""
    assert "expects an MCP client on stdin" in result.stderr
    assert "biomcp serve-http" in result.stderr
    assert "connection closed" not in result.stderr
    assert "initialized request" not in result.stderr


def test_discover_is_truthful_and_cacheable_before_handshake(
    stdio_mcp: StdioMcp,
) -> None:
    request = {
        "jsonrpc": "2.0",
        "id": "discover-1",
        "method": "server/discover",
        "params": {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientCapabilities": {},
            }
        },
    }
    first = stdio_mcp.call(request)["result"]
    request["id"] = "discover-2"
    second = stdio_mcp.call(request)["result"]

    assert set(first["supportedVersions"]) == {
        "2025-06-18",
        "2025-11-25",
        "2026-07-28",
    }
    assert first["capabilities"]["tools"] == {}
    assert first["capabilities"]["resources"] == {}
    server_info = first["_meta"]["io.modelcontextprotocol/serverInfo"]
    assert server_info["name"] == "biomcp"
    assert server_info["version"]
    assert first["resultType"] == "complete"
    assert isinstance(first["ttlMs"], int) and first["ttlMs"] > 0
    assert first["cacheScope"] in {"public", "private"}
    assert second == first


def test_pre_handshake_error_does_not_end_the_stream(stdio_mcp: StdioMcp) -> None:
    rejected = stdio_mcp.call(
        {"jsonrpc": "2.0", "id": "early", "method": "tools/list", "params": {}}
    )

    assert rejected["jsonrpc"] == "2.0"
    assert isinstance(rejected["error"]["code"], int)
    assert rejected["error"]["message"]
    assert "result" not in rejected

    initialized = stdio_mcp.call(
        {
            "jsonrpc": "2.0",
            "id": "legacy",
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "contract-test", "version": "1"},
            },
        }
    )["result"]
    assert initialized["protocolVersion"] == "2025-11-25"
    assert initialized["serverInfo"]["name"] == "biomcp"
    assert not {"resultType", "ttlMs", "cacheScope", "_meta"} & initialized.keys()


@pytest.mark.parametrize("initialized", [False, True])
def test_slow_tool_call_outlives_startup_wait(
    initialized: bool, tmp_path: Path,
) -> None:
    requests: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            requests.append(self.path)
            threading.Event().wait(6)
            payload = json.dumps({"total": 1, "hits": [{
                "_id": "673", "symbol": "BRAF", "name": "authored slow gene",
                "entrezgene": "673",
            }]}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            try:
                self.wfile.write(payload)
            except BrokenPipeError:
                pass  # The unfixed server closes the invocation at startup timeout.

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {"BIOMCP_MYGENE_BASE": origin}, origin)
    client = StdioMcp(env)
    try:
        if initialized:
            _initialize(client)
        params: dict[str, object] = {
            "name": "biomcp", "arguments": {
                "command": "biomcp search gene BRAF --limit 1",
            },
        }
        if not initialized:
            params["_meta"] = {
                "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientCapabilities": {},
            }
        started = time.monotonic()
        response = client.call({
            "jsonrpc": "2.0", "id": "slow", "method": "tools/call", "params": params,
        }, timeout=15)
        result = response["result"]
        assert not result.get("isError"), result
        assert "authored slow gene" in json.dumps(result)
        assert time.monotonic() - started >= 6
        assert len(requests) == 1, requests
        if not initialized:
            _initialize(client)
        assert client.call({"jsonrpc": "2.0", "id": "ping", "method": "ping"})["result"] == {}
    finally:
        client.close()
        server.shutdown()
        server.server_close()
        thread.join()


def _initialize(client: StdioMcp) -> None:
    response = client.call({
        "jsonrpc": "2.0", "id": "initialize", "method": "initialize",
        "params": {"protocolVersion": "2025-11-25", "capabilities": {},
                   "clientInfo": {"name": "startup-contract", "version": "1"}},
    })
    assert response["result"]["protocolVersion"] == "2025-11-25"
    assert client.process.stdin is not None
    client.process.stdin.write(json.dumps({
        "jsonrpc": "2.0", "method": "notifications/initialized",
    }) + "\n")
    client.process.stdin.flush()


@pytest.mark.parametrize("send_pings", [False, True])
def test_client_without_initialize_expires_startup_wait(
    send_pings: bool, tmp_path: Path,
) -> None:
    client = StdioMcp(isolated_command_env(tmp_path, {}, "http://127.0.0.1:9"))
    started = time.monotonic()
    try:
        if send_pings:
            for request_id in range(3):
                threading.Event().wait(1)
                assert client.call({
                    "jsonrpc": "2.0", "id": request_id, "method": "ping",
                })["result"] == {}
        client.process.wait(timeout=max(0.1, 7 - (time.monotonic() - started)))
        assert 4 <= time.monotonic() - started < 7
        assert client.process.stdin is not None
        assert not client.process.stdin.closed
        assert client.process.returncode == 1
        assert client.process.stdout is not None
        assert client.process.stdout.read() == ""
        assert client.process.stderr is not None
        assert "expects an MCP client on stdin" in client.process.stderr.read()
    finally:
        client.close()
