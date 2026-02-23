from __future__ import annotations

from pathlib import Path

import pytest
from mcp import types
from mcp.shared.exceptions import McpError

EXPECTED_HELP_RESOURCE = ("biomcp://help", "BioMCP Overview")


@pytest.mark.asyncio
async def test_initialize_advertises_tools_and_resources(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (_session, initialize_result):
        capabilities = initialize_result.capabilities
        assert capabilities.tools is not None
        assert capabilities.resources is not None


@pytest.mark.asyncio
async def test_list_tools_includes_shell(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_tools()
        names = {tool.name for tool in result.tools}
        assert "shell" in names


@pytest.mark.asyncio
async def test_shell_description_matches_list_contract(
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
        shell = next(tool for tool in result.tools if tool.name == "shell")
        description = shell.description
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

        assert actual
        assert actual[0] == EXPECTED_HELP_RESOURCE
        assert len({uri for uri, _name in actual}) == len(actual)

        skill_resources = actual[1:]
        assert skill_resources
        for uri, name in skill_resources:
            assert uri.startswith("biomcp://skill/")
            assert name.startswith("Pattern: ")
            assert "Pattern: Pattern:" not in name


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
