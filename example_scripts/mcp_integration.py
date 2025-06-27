#!/usr/bin/env -S uv --quiet run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "mcp",
# ]
# ///

# Scripts to reproduce this page:
# https://biomcp.org/mcp_integration/

import asyncio

from mcp.client.session import ClientSession
from mcp.client.stdio import StdioServerParameters, stdio_client
from mcp.types import TextContent


async def check_server():
    # Run with pypi package using `uv` not `uvx`
    server_params = StdioServerParameters(
        command="uv",
        args=["run", "--with", "biomcp-python", "biomcp", "run"],
    )
    #
    # Run with local code
    # server_params = StdioServerParameters(
    #     command="python",
    #     args=["-m", "biomcp", "run"],
    # )

    async with (
        stdio_client(server_params) as (read, write),
        ClientSession(read, write) as session,
    ):
        await session.initialize()

        # list prompts
        prompts = await session.list_prompts()
        print("Available prompts:", prompts)

        # list resources
        resources = await session.list_resources()
        print("Available resources:", resources)

        # list tools
        tool_result = await session.list_tools()
        tools = tool_result.tools
        print("Available tools:", tools)
        assert len(tools) == 13  # 3 core tools + 10 individual tools

        # IMPORTANT: Always use think tool first!
        think_result = await session.call_tool(
            "think",
            {
                "thought": "Planning to analyze variant rs113488022 for BRAF gene...",
                "thoughtNumber": 1,
                "totalThoughts": 2,
                "nextThoughtNeeded": True,
            },
        )
        assert (
            think_result.isError is False
        ), f"Think error: {think_result.content}"

        # Now fetch variant details using unified fetch tool
        tool_name = "fetch"
        tool_args = {"domain": "variant", "id_": "rs113488022"}
        result = await session.call_tool(tool_name, tool_args)
        assert result.isError is False, f"Error: {result.content}"

        # --- Assertions ---
        # 1. Check the call was successful (not an error)
        assert (
            result.isError is False
        ), f"Tool call resulted in error: {result.content}"

        # 2. Check there is content
        assert result.content is not None
        assert len(result.content) >= 1

        # 3. Check the type of the first content block
        content_block = result.content[0]
        assert isinstance(content_block, TextContent)

        markdown_output = content_block.text
        # print(markdown_output)
        assert isinstance(markdown_output, str)
        assert "rs113488022" in markdown_output
        assert "BRAF" in markdown_output
        assert "Pathogenic" in markdown_output
        print(f"Successfully called tool '{tool_name}' with args {tool_args}")


if __name__ == "__main__":
    asyncio.run(check_server())
