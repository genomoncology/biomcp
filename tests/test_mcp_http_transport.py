from __future__ import annotations

import base64
import os
import socket
import subprocess
import time
import urllib.error
import urllib.request
from collections.abc import Iterator
from datetime import timedelta
from pathlib import Path

import pytest
from mcp import ClientSession, types
from mcp.client.streamable_http import streamable_http_client

REPO_ROOT = Path(__file__).resolve().parents[1]
RELEASE_BIN = REPO_ROOT / "target" / "release" / "biomcp"


def _reserve_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.bind(("127.0.0.1", 0))
        return int(sock.getsockname()[1])


def _require_release_binary() -> Path:
    assert RELEASE_BIN.exists(), f"missing release binary: {RELEASE_BIN}"
    return RELEASE_BIN


def _healthcheck(url: str) -> None:
    with urllib.request.urlopen(url, timeout=2) as response:
        assert response.status == 200


def _mime_type(content: object) -> str | None:
    return getattr(content, "mimeType", getattr(content, "mime_type", None))


@pytest.fixture
def http_server_url() -> Iterator[str]:
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
                _healthcheck(f"{base_url}/health")
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


@pytest.fixture
def study_http_server_url(study_fixture_env: dict[str, str]) -> Iterator[str]:
    binary = _require_release_binary()
    port = _reserve_port()
    base_url = f"http://127.0.0.1:{port}"
    env = dict(os.environ)
    env.update(study_fixture_env)
    proc = subprocess.Popen(
        [str(binary), "serve-http", "--host", "127.0.0.1", "--port", str(port)],
        cwd=REPO_ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        env=env,
    )

    try:
        for _ in range(40):
            try:
                _healthcheck(f"{base_url}/health")
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


@pytest.mark.asyncio
async def test_streamable_http_supports_initialize_list_tools_and_tool_call(
    http_server_url: str,
) -> None:
    async with streamable_http_client(f"{http_server_url}/mcp") as (
        read_stream,
        write_stream,
        _get_session_id,
    ):
        async with ClientSession(
            read_stream,
            write_stream,
            read_timeout_seconds=timedelta(seconds=20),
        ) as session:
            initialize_result = await session.initialize()
            assert initialize_result.capabilities.tools is not None
            assert initialize_result.instructions is not None
            assert "biomcp skill list" not in initialize_result.instructions
            assert "biomcp skill" in initialize_result.instructions

            tools_result = await session.list_tools()
            names = {tool.name for tool in tools_result.tools}
            assert "biomcp" in names
            assert "shell" not in names

            call_result = await session.call_tool(
                "biomcp",
                arguments={"command": "biomcp version"},
            )
            text_chunks = [
                content.text
                for content in call_result.content
                if isinstance(content, types.TextContent)
            ]
            image_chunks = [
                content
                for content in call_result.content
                if isinstance(content, types.ImageContent)
            ]
            assert text_chunks
            assert not image_chunks
            assert any(
                "0.8." in text or "biomcp" in text.lower() for text in text_chunks
            )


@pytest.mark.asyncio
async def test_streamable_http_returns_text_then_svg_image_for_charted_study_call(
    study_http_server_url: str,
) -> None:
    async with streamable_http_client(f"{study_http_server_url}/mcp") as (
        read_stream,
        write_stream,
        _get_session_id,
    ):
        async with ClientSession(
            read_stream,
            write_stream,
            read_timeout_seconds=timedelta(seconds=20),
        ) as session:
            await session.initialize()
            call_result = await session.call_tool(
                "biomcp",
                arguments={
                    "command": (
                        "biomcp study query --study msk_impact_2017 --gene TP53 "
                        "--type mutations --chart bar"
                    )
                },
            )

    assert len(call_result.content) == 2
    assert isinstance(call_result.content[0], types.TextContent)
    assert isinstance(call_result.content[1], types.ImageContent)
    assert (
        "# Study Mutation Frequency: TP53 (msk_impact_2017)"
        in call_result.content[0].text
    )
    assert _mime_type(call_result.content[1]) == "image/svg+xml"

    svg = base64.b64decode(call_result.content[1].data).decode("utf-8")
    stripped = svg.lstrip()
    assert stripped.startswith("<svg") or stripped.startswith("<?xml")
    assert "<svg" in svg
