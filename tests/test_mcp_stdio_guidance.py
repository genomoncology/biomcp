from __future__ import annotations

import json
import os
import select
import subprocess
from pathlib import Path
from typing import Any

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "release" / "biomcp"))


class StdioMcp:
    def __init__(self) -> None:
        assert RELEASE_BIN.exists(), f"missing BioMCP binary: {RELEASE_BIN}"
        self.process = subprocess.Popen(
            [str(RELEASE_BIN), "serve"],
            cwd=REPO_ROOT,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def call(self, request: dict[str, object]) -> dict[str, Any]:
        assert self.process.stdin is not None
        assert self.process.stdout is not None
        self.process.stdin.write(json.dumps(request, separators=(",", ":")) + "\n")
        self.process.stdin.flush()

        ready, _, _ = select.select([self.process.stdout], [], [], 3)
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
