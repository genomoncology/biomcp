from __future__ import annotations

import json
import os
import signal
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

pytestmark = [pytest.mark.needs_binary, pytest.mark.timeout(120)]

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
            start_new_session=True,
        )

    def call(self, request: dict[str, object]) -> dict[str, Any]:
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
        self.process.stdin.flush()

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
        try:
            os.killpg(self.process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait()


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
        })
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
    responses: list[dict[str, Any]] = []
    errors: list[Exception] = []

    def ping_until_exit() -> None:
        assert client.process.stdin is not None
        assert client.process.stdout is not None
        try:
            while client.process.poll() is None:
                client.process.stdin.write('{"jsonrpc":"2.0","id":1,"method":"ping"}\n')
                client.process.stdin.flush()
                line = client.process.stdout.readline()
                if not line:
                    return
                responses.append(json.loads(line))
                threading.Event().wait(0.05)
        except BrokenPipeError:
            pass  # Startup expiry closes the stream while the client stays open.
        except Exception as error:
            errors.append(error)

    writer = threading.Thread(target=ping_until_exit, daemon=True) if send_pings else None
    started = time.monotonic()
    try:
        if writer is not None:
            writer.start()
        client.process.wait()
        if writer is not None:
            writer.join()
        assert not errors, errors
        assert all(response["id"] == 1 and response["result"] == {} for response in responses)
        assert time.monotonic() - started >= 4
        assert client.process.stdin is not None
        assert not client.process.stdin.closed
        assert client.process.returncode == 1
        assert client.process.stdout is not None
        assert client.process.stdout.read() == ""
        assert client.process.stderr is not None
        assert "expects an MCP client on stdin" in client.process.stderr.read()
    finally:
        client.close()
        if writer is not None:
            writer.join()


def test_client_not_reading_ping_responses_expires_startup_wait(tmp_path: Path) -> None:
    client = StdioMcp(isolated_command_env(tmp_path, {}, "http://127.0.0.1:9"))
    assert client.process.stdin is not None

    def send_pings() -> None:
        try:
            while True:
                client.process.stdin.write(
                    '{"jsonrpc":"2.0","id":1,"method":"ping"}\n' * 200,
                )
                client.process.stdin.flush()
        except (BrokenPipeError, ValueError):
            pass

    writer = threading.Thread(target=send_pings, daemon=True)
    writer.start()
    started = time.monotonic()
    try:
        client.process.wait()
        assert time.monotonic() - started >= 4
        assert client.process.returncode == 1
        assert not client.process.stdin.closed
        assert client.process.stderr is not None
        assert "expects an MCP client on stdin" in client.process.stderr.read()
    finally:
        client.close()
        writer.join()
