from enum import Enum
from typing import Annotated

import typer
from dotenv import load_dotenv

from .. import logger, mcp_app  # mcp_app is already instantiated in core.py

# Load environment variables from .env file
load_dotenv()

server_app = typer.Typer(help="Server operations")


class ServerMode(str, Enum):
    STDIO = "stdio"
    WORKER = "worker"
    STREAMABLE_HTTP = "streamable_http"


def run_stdio_server():
    """Run server in STDIO mode."""
    logger.info("Starting MCP server with STDIO transport:")
    mcp_app.run(transport="stdio")


def run_http_server(host: str, port: int, mode: ServerMode):
    """Run server in HTTP-based mode (worker or streamable_http)."""
    try:
        import uvicorn

        if mode == ServerMode.WORKER:
            logger.info("Starting MCP server with Worker/SSE transport")
            from ..workers.worker import app
        else:  # STREAMABLE_HTTP
            logger.info(
                f"Starting MCP server with Streamable HTTP transport on {host}:{port}"
            )
            logger.info(f"Endpoint: http://{host}:{port}/mcp")
            logger.info("Using FastMCP's native Streamable HTTP support")
            from ..workers.worker import app

        uvicorn.run(
            app,
            host=host,
            port=port,
            log_level="info",
        )
    except ImportError as e:
        logger.error(f"Failed to start {mode.value} mode: {e}")
        raise typer.Exit(1) from e
    except Exception as e:
        logger.error(f"An unexpected error occurred: {e}", exc_info=True)
        raise typer.Exit(1) from e


@server_app.command("run")
def run_server(
    mode: Annotated[
        ServerMode,
        typer.Option(
            help="Server mode: stdio (local), worker (legacy SSE), or streamable_http (MCP spec compliant)",
            case_sensitive=False,
        ),
    ] = ServerMode.STDIO,
    host: Annotated[
        str,
        typer.Option(
            help="Host to bind to (for HTTP modes)",
        ),
    ] = "0.0.0.0",  # noqa: S104 - Required for Docker container networking
    port: Annotated[
        int,
        typer.Option(
            help="Port to bind to (for HTTP modes)",
        ),
    ] = 8000,
):
    """Run the BioMCP server with selected transport mode."""
    if mode == ServerMode.STDIO:
        run_stdio_server()
    else:
        run_http_server(host, port, mode)
