from __future__ import annotations

import base64
from pathlib import Path

import pytest
from mcp import types
from mcp.shared.exceptions import McpError

EXPECTED_HELP_RESOURCE = ("biomcp://help", "BioMCP Overview")


def _mime_type(content: object) -> str | None:
    return getattr(content, "mimeType", getattr(content, "mime_type", None))


def _is_error(result: object) -> bool | None:
    return getattr(result, "isError", getattr(result, "is_error", None))


@pytest.mark.asyncio
async def test_initialize_advertises_tools_and_resources(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (_session, initialize_result):
        capabilities = initialize_result.capabilities
        assert capabilities.tools is not None
        assert capabilities.resources is not None
        assert initialize_result.instructions is not None
        assert "biomcp skill list" not in initialize_result.instructions
        assert "biomcp skill" in initialize_result.instructions


@pytest.mark.asyncio
async def test_list_tools_includes_biomcp(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_tools()
        names = {tool.name for tool in result.tools}
        assert "biomcp" in names
        assert "shell" not in names


@pytest.mark.asyncio
async def test_biomcp_description_matches_list_contract(
    mcp_session_factory,
) -> None:
    repo = Path(__file__).resolve().parents[1]
    list_contract = (repo / "src/cli/list_reference.md").read_text()
    required = [
        "BioMCP Command Reference",
        "search <entity> [query|filters]",
        "search trial [filters]",
        "get <entity> <id> [section...]",
    ]
    for marker in required:
        assert marker in list_contract

    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_tools()
        biomcp = next(tool for tool in result.tools if tool.name == "biomcp")
        description = biomcp.description
        for marker in required:
            assert marker in description
        assert "SEARCH FILTERS:" in description
        assert "AGENT GUIDANCE:" in description
        assert "biomcp list" in description


@pytest.mark.asyncio
async def test_list_resources_returns_expected_inventory(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_resources()
        actual = [(str(resource.uri), resource.name) for resource in result.resources]

        assert actual == [EXPECTED_HELP_RESOURCE]


@pytest.mark.asyncio
async def test_read_resource_returns_markdown_for_every_uri(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        listed = await session.list_resources()
        resource_uris = [str(resource.uri) for resource in listed.resources]
        assert resource_uris

        for uri in resource_uris:
            result = await session.read_resource(uri)
            assert result.contents, f"{uri} returned no content"

            text_contents = [
                content
                for content in result.contents
                if isinstance(content, types.TextResourceContents)
            ]
            assert text_contents, f"{uri} did not return markdown text"

            for content in text_contents:
                mime_type = content.mimeType or getattr(content, "mime_type", None)
                assert str(content.uri) == uri
                assert mime_type == "text/markdown"
                assert content.text.strip()


@pytest.mark.asyncio
async def test_invalid_resource_uri_returns_mcp_error(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        with pytest.raises(McpError) as exc_info:
            await session.read_resource("biomcp://skill/not-a-real-resource")

        assert exc_info.value.error.code == -32002
        assert "Unknown resource:" in exc_info.value.error.message


@pytest.mark.asyncio
async def test_charted_study_call_returns_text_then_svg_image(
    mcp_session_factory,
    study_fixture_env,
) -> None:
    async with mcp_session_factory(study_fixture_env) as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={
                "command": (
                    "biomcp study query --study msk_impact_2017 --gene TP53 "
                    "--type mutations --chart bar"
                )
            },
        )

    assert _is_error(result) is False
    assert len(result.content) == 2
    assert isinstance(result.content[0], types.TextContent)
    assert isinstance(result.content[1], types.ImageContent)

    text = result.content[0].text
    assert "# Study Mutation Frequency: TP53 (msk_impact_2017)" in text

    image = result.content[1]
    assert _mime_type(image) == "image/svg+xml"
    svg = base64.b64decode(image.data).decode("utf-8")
    stripped = svg.lstrip()
    assert stripped.startswith("<svg") or stripped.startswith("<?xml")
    assert "<svg" in svg


@pytest.mark.asyncio
async def test_charted_study_call_rejects_output_file_in_mcp_mode(
    mcp_session_factory,
    study_fixture_env,
) -> None:
    async with mcp_session_factory(study_fixture_env) as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={
                "command": (
                    "biomcp study query --study msk_impact_2017 --gene TP53 "
                    "--type mutations --chart bar --output out.svg"
                )
            },
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "MCP chart responses do not support --output" in result.content[0].text
