import asyncio

import typer

from .. import logger, mcp_app

server_app = typer.Typer(help="Server operations")


@server_app.command("run")
def run_server():
    """Run the BioMCP server with STDIO transport."""

    tools = asyncio.run(mcp_app.list_tools())
    tools_str = ",".join(t.name for t in tools)
    logger.info(f"Tools loaded: {tools_str}")

    logger.info("Starting MCP server:")
    mcp_app.run()
