#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["mcp>=1.1.1"]
# ///
#
# Start the server first:
#   biomcp serve-http --host 127.0.0.1 --port 8080
#
# Then run this demo:
#   uv run --script examples/streamable-http/streamable_http_client.py
#   uv run --script examples/streamable-http/streamable_http_client.py http://127.0.0.1:8080
import asyncio
import sys
from datetime import timedelta

DEFAULT_BASE_URL = "http://127.0.0.1:8080"
WORKFLOW = [
    "biomcp search all --gene BRAF --disease melanoma --counts-only",
    'biomcp get variant "BRAF V600E" clinvar',
    'biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5',
]


def resolve_base_url(argv: list[str]) -> str:
    if len(argv) > 2:
        raise SystemExit("Usage: examples/streamable-http/streamable_http_client.py [base_url]")
    return argv[1] if len(argv) > 1 else DEFAULT_BASE_URL


async def main(base_url: str) -> None:
    from mcp import ClientSession, types
    from mcp.client.streamable_http import streamable_http_client

    mcp_url = f"{base_url.rstrip('/')}/mcp"
    print(f"Connecting to {mcp_url}\n")

    # The current Python MCP client warns on 202 Accepted during session teardown.
    async with streamable_http_client(
        mcp_url,
        terminate_on_close=False,
    ) as (read_stream, write_stream, _):
        async with ClientSession(
            read_stream,
            write_stream,
            read_timeout_seconds=timedelta(seconds=30),
        ) as session:
            await session.initialize()
            for command in WORKFLOW:
                print(f"Command: {command}")
                call_result = await session.call_tool(
                    "biomcp",
                    arguments={"command": command},
                )
                for content in call_result.content:
                    if isinstance(content, types.TextContent):
                        print(content.text)


if __name__ == "__main__":
    asyncio.run(main(resolve_base_url(sys.argv)))
