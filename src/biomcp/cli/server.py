from enum import Enum
from typing import Annotated

import typer

from .. import logger, mcp_app  # mcp_app is already instantiated in core.py

server_app = typer.Typer(help="Server operations")


class ServerMode(str, Enum):
    STDIO = "stdio"
    WORKER = "worker"


@server_app.command("run")
def run_server(
    mode: Annotated[
        ServerMode,
        typer.Option(
            help="Server mode: stdio (local) or worker (Cloudflare Worker/SSE)",
            case_sensitive=False,
        ),
    ] = ServerMode.STDIO,
):
    """Run the BioMCP server with selected transport mode."""

    if mode == ServerMode.STDIO:
        logger.info("Starting MCP server with STDIO transport:")
        mcp_app.run(transport="stdio")
    elif mode == ServerMode.WORKER:
        logger.info("Starting MCP server with Worker/SSE transport")
        try:
            mcp_app.run(transport="sse")
        except ImportError as e:
            logger.error(f"Failed to start worker/sse mode: {e}")
            logger.error(
                "Make sure you have the required dependencies installed:"
            )
            logger.error(
                "pip install uvicorn mcp-python[server]"
            )  # Ensure all server extras are installed
            raise typer.Exit(1) from e
        except Exception as e:
            logger.error(f"An unexpected error occurred: {e}", exc_info=True)
            raise typer.Exit(1) from e
