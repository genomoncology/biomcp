from __future__ import annotations

import json
import os
import socket
import subprocess
import time
import urllib.error
import urllib.request
from contextlib import contextmanager
from collections.abc import Iterator
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target" / "release" / "biomcp"))


def _reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _require_release_binary() -> Path:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    return RELEASE_BIN


def _read_json(url: str) -> tuple[dict[str, str], str]:
    with urllib.request.urlopen(url, timeout=2) as response:
        body = response.read().decode("utf-8")
        return json.loads(body), response.headers.get_content_type()


def _mcp_status(base_url: str, host: str) -> int:
    request = urllib.request.Request(
        f"{base_url}/mcp", data=b"{}", headers={"Host": host}, method="POST"
    )
    try:
        with urllib.request.urlopen(request, timeout=2) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


def _route_status(base_url: str, path: str, host: str) -> int:
    request = urllib.request.Request(f"{base_url}{path}", headers={"Host": host})
    try:
        with urllib.request.urlopen(request, timeout=2) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


@contextmanager
def _running_server(*extra_args: str) -> Iterator[str]:
    binary = _require_release_binary()
    port = _reserve_port()
    base_url = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen(
        [str(binary), "serve-http", "--host", "127.0.0.1", "--port", str(port), *extra_args],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    try:
        for _ in range(40):
            try:
                if _route_status(base_url, "/health", "127.0.0.1") < 500:
                    yield base_url
                    return
            except urllib.error.URLError:
                time.sleep(0.25)
        raise AssertionError(f"serve-http did not start at {base_url}")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)


@pytest.fixture
def http_server() -> Iterator[str]:
    binary = _require_release_binary()
    port = _reserve_port()
    base_url = f"http://127.0.0.1:{port}"
    proc = subprocess.Popen(
        [str(binary), "serve-http", "--host", "127.0.0.1", "--port", str(port)],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )

    try:
        for _ in range(40):
            try:
                payload, content_type = _read_json(f"{base_url}/health")
                if payload == {"status": "ok"} and content_type == "application/json":
                    yield base_url
                    return
            except urllib.error.URLError:
                time.sleep(0.25)

        raise AssertionError(f"serve-http did not become ready on {base_url}/health")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait(timeout=5)


def test_http_routes_advertise_streamable_http_surface(http_server: str) -> None:
    root_payload, root_content_type = _read_json(f"{http_server}/")
    assert root_content_type == "application/json"
    assert root_payload == {
        "name": "biomcp",
        "version": root_payload["version"],
        "transport": "streamable-http",
        "mcp": "/mcp",
    }

    health_payload, health_content_type = _read_json(f"{http_server}/health")
    assert health_content_type == "application/json"
    assert health_payload == {"status": "ok"}

    ready_payload, ready_content_type = _read_json(f"{http_server}/readyz")
    assert ready_content_type == "application/json"
    assert ready_payload == {"status": "ok"}


def test_loopback_server_rejects_unrelated_host_headers(http_server: str) -> None:
    port = http_server.rsplit(":", 1)[1]
    for path in ("/", "/health", "/readyz"):
        assert _route_status(http_server, path, "attacker.example") == 403
    assert _mcp_status(http_server, "attacker.example") == 403
    for host in ("localhost", f"localhost:{port}", "127.0.0.1", f"127.0.0.1:{port}", "[::1]", f"[::1]:{port}"):
        assert _mcp_status(http_server, host) != 403


def test_explicit_and_unsafe_host_policies_apply_to_every_route() -> None:
    with _running_server("--allowed-hosts", "allowed.example") as base_url:
        for path in ("/", "/health", "/readyz", "/mcp"):
            status = (
                _mcp_status(base_url, "allowed.example")
                if path == "/mcp"
                else _route_status(base_url, path, "allowed.example")
            )
            assert status != 403
            denied = (
                _mcp_status(base_url, "denied.example")
                if path == "/mcp"
                else _route_status(base_url, path, "denied.example")
            )
            assert denied == 403

    with _running_server("--unsafe-allow-any-host") as base_url:
        for path in ("/", "/health", "/readyz"):
            assert _route_status(base_url, path, "anything.example") == 200


def test_serve_http_help_matches_runtime_surface() -> None:
    binary = _require_release_binary()
    result = subprocess.run(
        [str(binary), "serve-http", "--help"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0
    assert "Streamable HTTP" in result.stdout
    assert "/mcp" in result.stdout
    assert "--host <HOST>" in result.stdout
    assert "--port <PORT>" in result.stdout
    assert "--unsafe-allow-any-host" in result.stdout
    assert "does not add authentication or encryption" in result.stdout
    assert "SSE transport" not in result.stdout
    assert "--json" not in result.stdout
    assert "--no-cache" not in result.stdout


def test_non_loopback_bind_requires_explicit_host_policy() -> None:
    binary = _require_release_binary()
    port = _reserve_port()
    result = subprocess.run(
        [str(binary), "serve-http", "--host", "0.0.0.0", "--port", str(port)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode != 0
    assert "--allowed-hosts" in result.stderr
    assert "--unsafe-allow-any-host" in result.stderr


def test_top_level_help_hides_serve_sse_but_lists_serve_http() -> None:
    binary = _require_release_binary()
    result = subprocess.run(
        [str(binary), "--help"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0
    assert "serve-http" in result.stdout
    assert "serve-sse" not in result.stdout


def test_serve_sse_help_is_still_callable_and_deprecated() -> None:
    binary = _require_release_binary()
    result = subprocess.run(
        [str(binary), "serve-sse", "--help"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode == 0
    assert "serve-sse" in result.stdout
    assert "removed" in result.stdout or "deprecated" in result.stdout
    assert "serve-http" in result.stdout
    assert "/mcp" in result.stdout
    assert "--json" not in result.stdout
    assert "--no-cache" not in result.stdout


def test_serve_sse_exits_non_zero_with_migration_message() -> None:
    binary = _require_release_binary()
    result = subprocess.run(
        [str(binary), "serve-sse"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    combined = f"{result.stdout}\n{result.stderr}"
    assert result.returncode != 0
    assert "serve-http" in combined
    assert "removed" in combined or "deprecated" in combined
