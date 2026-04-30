from __future__ import annotations

import base64
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import pytest
from mcp import types
from mcp.shared.exceptions import McpError

EXPECTED_HELP_RESOURCE = ("biomcp://help", "BioMCP Overview")


def _expected_skill_resources() -> list[tuple[str, str]]:
    repo = Path(__file__).resolve().parents[1]
    resources: list[tuple[str, str]] = []
    for path in sorted((repo / "skills" / "use-cases").glob("[0-9][0-9]-*.md")):
        title = next(
            line.removeprefix("# ").strip()
            for line in path.read_text(encoding="utf-8").splitlines()
            if line.startswith("# ")
        )
        name = title if title.lower().startswith("pattern:") else f"Pattern: {title}"
        resources.append((f"biomcp://skill/{path.stem[3:]}", name))
    return resources


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
        assert "leading public biomedical data sources" in initialize_result.instructions
        assert "15 sources" not in initialize_result.instructions
        assert "15 biomedical sources" not in initialize_result.instructions
        assert "biomcp skill list" not in initialize_result.instructions
        assert 'biomcp suggest "<question>"' in initialize_result.instructions
        assert "biomcp skill" in initialize_result.instructions


@pytest.mark.asyncio
async def test_list_tools_includes_biomcp(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_tools()
        names = {tool.name for tool in result.tools}
        assert "biomcp" in names
        assert "shell" not in names

        biomcp = next(tool for tool in result.tools if tool.name == "biomcp")
        annotations = biomcp.annotations
        assert annotations is not None
        assert getattr(annotations, "title", None) == "BioMCP"
        read_only = getattr(annotations, "readOnlyHint", None)
        if read_only is None:
            read_only = getattr(annotations, "read_only_hint", None)
        assert read_only is True


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
        'suggest "What drugs treat melanoma?"',
        'search phenotype "seizure, developmental delay"',
    ]
    article_markers = [
        "Turn a literature question into article filters",
        "known gene/disease/drug anchors go in `-g/-d/--drug`; free-text concepts go in `-k`",
        "PubMed ESearch cleans question-format terms provider-locally",
    ]
    article_details = [
        "## Query formulation",
        "apoptosis gene regulation",
        "photosensitivity mechanism",
        "TCGA mutation analysis dataset",
    ]
    for marker in required:
        assert marker in list_contract
    for marker in article_markers:
        assert marker in list_contract
    for detail in article_details:
        assert detail not in list_contract

    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_tools()
        biomcp = next(tool for tool in result.tools if tool.name == "biomcp")
        description = biomcp.description
        assert "read-only" in description.lower()
        for marker in required:
            assert marker in description
        for marker in article_markers:
            assert marker in description
        for detail in article_details:
            assert detail not in description
        assert "leading public biomedical data sources" in description
        assert "15 biomedical sources" not in description
        assert "SEARCH FILTERS:" in description
        assert "AGENT GUIDANCE:" in description
        assert "biomcp list" in description
        assert "ema sync" not in description
        assert "who sync" not in description
        assert "cvx sync" not in description
        assert "gtr sync" not in description
        assert "who-ivd sync" not in description
        assert "skill install" not in description
        assert "study download --list" in description
        assert "study download <study_id>" not in description
        assert "study download [--list] [<study_id>]" not in description
        assert "cache path" not in description
        assert "cache stats" not in description
        assert "cache clean" not in description
        assert "cache clear" not in description
        assert not any(
            line.strip().startswith("- `update ")
            for line in description.splitlines()
        )
        assert "uninstall" not in description


@pytest.mark.asyncio
async def test_list_resources_returns_expected_inventory(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.list_resources()
        actual = [(str(resource.uri), resource.name) for resource in result.resources]

        assert actual == [EXPECTED_HELP_RESOURCE, *_expected_skill_resources()]


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
                if uri == "biomcp://help":
                    assert "## Routing rules" in content.text
                    assert "## How-to reference" in content.text
                    assert "../docs/" not in content.text
                    assert ".md)" not in content.text


@pytest.mark.asyncio
async def test_skill_render_matches_help_resource_body(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        render = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp skill render"},
        )
        help_resource = await session.read_resource("biomcp://help")

    assert _is_error(render) is False
    assert render.content
    assert isinstance(render.content[0], types.TextContent)
    help_text = next(
        content.text
        for content in help_resource.contents
        if isinstance(content, types.TextResourceContents)
    )
    assert render.content[0].text == help_text


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
async def test_mutating_study_download_is_rejected_in_mcp_mode(
    mcp_session_factory,
) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp study download msk_impact_2017"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "BioMCP allows read-only commands only" in result.content[0].text


@pytest.mark.asyncio
async def test_gtr_sync_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp gtr sync"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "BioMCP allows read-only commands only" in result.content[0].text


@pytest.mark.asyncio
async def test_who_ivd_sync_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp who-ivd sync"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "BioMCP allows read-only commands only" in result.content[0].text


@pytest.mark.asyncio
async def test_cache_path_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp cache path"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "CLI-only over MCP" in result.content[0].text
    assert "workstation-local filesystem paths" in result.content[0].text


@pytest.mark.asyncio
async def test_cache_stats_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp cache stats"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "CLI-only over MCP" in result.content[0].text
    assert "workstation-local filesystem paths" in result.content[0].text


@pytest.mark.asyncio
async def test_cache_clean_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp cache clean"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "CLI-only over MCP" in result.content[0].text
    assert "workstation-local filesystem paths" in result.content[0].text


@pytest.mark.asyncio
async def test_cache_clear_is_rejected_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp cache clear"},
        )

    assert _is_error(result) is True
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "CLI-only over MCP" in result.content[0].text
    assert "workstation-local filesystem paths" in result.content[0].text


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


class _Ols4StubHandler(BaseHTTPRequestHandler):
    """Minimal OLS4 stub returning a single HGNC hit for any query."""

    _RESPONSE = json.dumps({
        "response": {
            "numFound": 1,
            "start": 0,
            "docs": [
                {
                    "iri": "http://identifiers.org/hgnc/1100",
                    "ontology_name": "hgnc",
                    "ontology_prefix": "HGNC",
                    "short_form": "HGNC_1100",
                    "obo_id": "HGNC:1100",
                    "label": "BRCA1",
                    "description": ["BRCA1 DNA repair associated"],
                    "type": "class",
                    "is_defining_ontology": True,
                }
            ],
        }
    }).encode()

    def do_GET(self) -> None:  # noqa: N802
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(self._RESPONSE)

    def log_message(self, *_args: object) -> None:
        pass


@pytest.fixture()
def ols4_stub() -> str:
    """Start a local OLS4 stub and return its base URL."""
    server = ThreadingHTTPServer(("127.0.0.1", 0), _Ols4StubHandler)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield f"http://127.0.0.1:{port}"
    server.shutdown()


@pytest.mark.asyncio
async def test_discover_command_is_allowed_via_mcp(
    mcp_session_factory, ols4_stub
) -> None:
    async with mcp_session_factory(
        extra_env={"BIOMCP_OLS4_BASE": ols4_stub, "UMLS_API_KEY": ""}
    ) as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": "biomcp discover BRCA1"},
        )

    assert _is_error(result) is False
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "BRCA1" in result.content[0].text


@pytest.mark.asyncio
async def test_biomcp_suggest_is_allowed_in_mcp_mode(mcp_session_factory) -> None:
    async with mcp_session_factory() as (session, _initialize_result):
        result = await session.call_tool(
            "biomcp",
            arguments={"command": 'biomcp suggest "What drugs treat melanoma?"'},
        )

    assert _is_error(result) is False
    assert result.content
    assert isinstance(result.content[0], types.TextContent)
    assert "treatment-lookup" in result.content[0].text
    assert "biomcp skill treatment-lookup" in result.content[0].text
