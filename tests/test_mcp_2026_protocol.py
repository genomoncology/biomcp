from __future__ import annotations

import json
import os
import select
import socket
import subprocess
import time
import urllib.error
import urllib.request
from contextlib import contextmanager
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = Path(
    os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "release" / "biomcp")
)
MODERN_VERSION = "2026-07-28"
META = {
    "io.modelcontextprotocol/protocolVersion": MODERN_VERSION,
    "io.modelcontextprotocol/clientCapabilities": {},
    "io.modelcontextprotocol/clientInfo": {"name": "protocol-contract", "version": "1"},
}


class RawStdioMcp:
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

    def send(self, message: dict[str, object]) -> None:
        assert self.process.stdin is not None
        self.process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        self.process.stdin.flush()

    def receive(self) -> dict[str, Any]:
        assert self.process.stdout is not None
        ready, _, _ = select.select([self.process.stdout], [], [], 3)
        assert ready, "timed out waiting for an MCP message"
        line = self.process.stdout.readline()
        if not line:
            assert self.process.stderr is not None
            pytest.fail(
                "BioMCP exited before returning an MCP message: "
                + self.process.stderr.read().strip()
            )
        return json.loads(line)

    def call(
        self, request_id: str, method: str, params: dict[str, object] | None = None
    ) -> dict[str, Any]:
        self.send(
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": {"_meta": META, **(params or {})},
            }
        )
        response = self.receive()
        assert response["id"] == request_id
        return response

    def close(self) -> None:
        if self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=2)


@pytest.fixture
def modern_stdio() -> Iterator[RawStdioMcp]:
    client = RawStdioMcp()
    try:
        yield client
    finally:
        client.close()


def _assert_modern_result(result: dict[str, Any]) -> None:
    assert result["resultType"] == "complete"
    server_info = result["_meta"]["io.modelcontextprotocol/serverInfo"]
    assert server_info["name"] == "biomcp"
    assert server_info["version"]


def _assert_cacheable(result: dict[str, Any]) -> None:
    assert isinstance(result["ttlMs"], int) and result["ttlMs"] >= 0
    assert result["cacheScope"] in {"public", "private"}


def test_stdio_serves_modern_requests_without_a_handshake(
    modern_stdio: RawStdioMcp,
) -> None:
    discover = modern_stdio.call("discover", "server/discover")["result"]
    assert set(discover["supportedVersions"]) == {
        "2025-06-18",
        "2025-11-25",
        MODERN_VERSION,
    }
    _assert_modern_result(discover)
    _assert_cacheable(discover)

    requests = [
        ("tools", "tools/list", {}, "tools", True),
        ("resources", "resources/list", {}, "resources", True),
        (
            "templates",
            "resources/templates/list",
            {},
            "resourceTemplates",
            True,
        ),
        (
            "read",
            "resources/read",
            {"uri": "biomcp://help"},
            "contents",
            True,
        ),
        (
            "call",
            "tools/call",
            {"name": "biomcp", "arguments": {"command": "biomcp version"}},
            "content",
            False,
        ),
    ]
    for request_id, method, params, payload_field, cacheable in requests:
        result = modern_stdio.call(request_id, method, params)["result"]
        assert payload_field in result
        _assert_modern_result(result)
        if cacheable:
            _assert_cacheable(result)


def test_stdio_enforces_modern_metadata_and_removed_methods(
    modern_stdio: RawStdioMcp,
) -> None:
    unsupported = modern_stdio.call(
        "unsupported",
        "tools/list",
        {
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": "1900-01-01",
                "io.modelcontextprotocol/clientCapabilities": {},
            }
        },
    )["error"]
    assert unsupported["code"] == -32022
    assert unsupported["data"]["requested"] == "1900-01-01"
    assert MODERN_VERSION in unsupported["data"]["supported"]

    for request_id, meta in [
        ("missing-version", {}),
        (
            "missing-capabilities",
            {"io.modelcontextprotocol/protocolVersion": MODERN_VERSION},
        ),
    ]:
        response = modern_stdio.call(request_id, "tools/list", {"_meta": meta})
        assert response["error"]["code"] == -32602
        assert "result" not in response

    missing_resource = modern_stdio.call(
        "missing-resource", "resources/read", {"uri": "biomcp://missing"}
    )
    assert missing_resource["error"]["code"] == -32602

    for request_id, method in [
        ("ping", "ping"),
        ("logging", "logging/setLevel"),
        ("old-subscribe", "resources/subscribe"),
    ]:
        response = modern_stdio.call(request_id, method)
        assert response["error"]["code"] == -32601
        assert "result" not in response


def test_stdio_listen_acknowledges_only_emitted_notifications(
    modern_stdio: RawStdioMcp,
) -> None:
    modern_stdio.send(
        {
            "jsonrpc": "2.0",
            "id": "listen",
            "method": "subscriptions/listen",
            "params": {
                "_meta": META,
                "notifications": {
                    "toolsListChanged": True,
                    "promptsListChanged": True,
                    "resourcesListChanged": True,
                    "resourceSubscriptions": ["biomcp://help"],
                },
            },
        }
    )
    acknowledged = modern_stdio.receive()
    assert acknowledged.get("method") == "notifications/subscriptions/acknowledged"
    assert acknowledged["params"]["notifications"] == {}
    assert (
        acknowledged["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"]
        == "listen"
    )

    modern_stdio.send(
        {
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {"requestId": "listen"},
        }
    )


def _reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


@contextmanager
def _running_http_server() -> Iterator[str]:
    assert RELEASE_BIN.exists(), f"missing BioMCP binary: {RELEASE_BIN}"
    port = _reserve_port()
    base_url = f"http://127.0.0.1:{port}"
    process = subprocess.Popen(
        [
            str(RELEASE_BIN),
            "serve-http",
            "--host",
            "127.0.0.1",
            "--port",
            str(port),
        ],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        for _ in range(40):
            try:
                with urllib.request.urlopen(f"{base_url}/readyz", timeout=1):
                    yield base_url
                    return
            except urllib.error.URLError:
                time.sleep(0.25)
        pytest.fail("serve-http did not become ready")
    finally:
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)


def _http_rpc(
    base_url: str,
    request_id: str,
    method: str,
    *,
    params: dict[str, object] | None = None,
    protocol_header: str | None = MODERN_VERSION,
    method_header: str | None = None,
    name_header: str | None = None,
    origin: str | None = None,
) -> tuple[int, dict[str, str], dict[str, Any] | str]:
    body = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": {"_meta": META, **(params or {})},
    }
    headers = {
        "Accept": "application/json, text/event-stream",
        "Content-Type": "application/json",
    }
    if protocol_header is not None:
        headers["MCP-Protocol-Version"] = protocol_header
    if method_header is not None:
        headers["Mcp-Method"] = method_header
    if name_header is not None:
        headers["Mcp-Name"] = name_header
    if origin is not None:
        headers["Origin"] = origin
    request = urllib.request.Request(
        f"{base_url}/mcp",
        data=json.dumps(body, separators=(",", ":")).encode(),
        headers=headers,
        method="POST",
    )
    try:
        response = urllib.request.urlopen(request, timeout=3)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        response_headers = {
            key.lower(): value for key, value in response.headers.items()
        }
        raw = response.read().decode()
        for line in raw.splitlines():
            if line.startswith("data: "):
                raw = line.removeprefix("data: ")
                break
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            payload = raw
        return response.status, response_headers, payload


def test_http_serves_modern_requests_without_protocol_sessions() -> None:
    with _running_http_server() as base_url:
        listed = _http_rpc(base_url, "tools", "tools/list", method_header="tools/list")
        called = _http_rpc(
            base_url,
            "call",
            "tools/call",
            params={"name": "biomcp", "arguments": {"command": "biomcp version"}},
            method_header="tools/call",
            name_header="biomcp",
        )

    for status, headers, response in (listed, called):
        assert status == 200
        assert headers["content-type"].split(";", 1)[0] in {
            "application/json",
            "text/event-stream",
        }
        assert "mcp-session-id" not in headers
        assert isinstance(response, dict)
        _assert_modern_result(response["result"])
    assert isinstance(listed[2], dict)
    assert listed[2]["result"]["tools"]
    _assert_cacheable(listed[2]["result"])
    assert isinstance(called[2], dict)
    assert called[2]["result"]["content"]


def test_http_validates_modern_headers_metadata_and_origin() -> None:
    with _running_http_server() as base_url:
        mismatch = _http_rpc(
            base_url,
            "mismatch",
            "tools/list",
            protocol_header="2025-11-25",
            method_header="tools/list",
        )
        missing_method = _http_rpc(base_url, "missing-method", "tools/list")
        missing_name = _http_rpc(
            base_url,
            "missing-name",
            "tools/call",
            params={"name": "biomcp", "arguments": {"command": "biomcp version"}},
            method_header="tools/call",
        )
        missing_meta = _http_rpc(
            base_url,
            "missing-meta",
            "tools/list",
            params={"_meta": {}},
            method_header="tools/list",
        )
        unsupported = _http_rpc(
            base_url,
            "unsupported",
            "tools/list",
            params={
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": "1900-01-01",
                    "io.modelcontextprotocol/clientCapabilities": {},
                }
            },
            protocol_header="1900-01-01",
            method_header="tools/list",
        )
        removed_ping = _http_rpc(base_url, "ping", "ping", method_header="ping")
        invalid_origin = _http_rpc(
            base_url,
            "origin",
            "tools/list",
            method_header="tools/list",
            origin="https://attacker.example",
        )

    for status, _headers, response in (mismatch, missing_method, missing_name):
        assert status == 400
        assert isinstance(response, dict)
        assert response["error"]["code"] == -32020
    assert missing_meta[0] == 400
    assert isinstance(missing_meta[2], dict)
    assert missing_meta[2]["error"]["code"] == -32602
    assert unsupported[0] == 400
    assert isinstance(unsupported[2], dict)
    assert unsupported[2]["error"]["code"] == -32022
    assert unsupported[2]["error"]["data"]["requested"] == "1900-01-01"
    assert MODERN_VERSION in unsupported[2]["error"]["data"]["supported"]
    assert removed_ping[0] == 404
    assert isinstance(removed_ping[2], dict)
    assert removed_ping[2]["error"]["code"] == -32601
    assert invalid_origin[0] == 403
