"""Worker implementation for BioMCP."""

import os
from fastapi import FastAPI, Response
from fastapi.middleware.cors import CORSMiddleware

from .. import logger, mcp_app

app = FastAPI(title="BioMCP Worker", version="0.1.10")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Add health check endpoint  
@app.get("/health")
async def health_check():
    return {"status": "healthy"}

# Add OPTIONS endpoint for CORS preflight
@app.options("/{path:path}")
async def options_handler(path: str):
    """Handle CORS preflight requests."""
    return Response(
        content="",
        status_code=204,
        headers={
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Allow-Headers": "*",
            "Access-Control-Max-Age": "86400",  # 24 hours
        },
    )

# Get the appropriate MCP ASGI app based on environment
transport_mode = os.environ.get("MCP_MODE", "sse").lower()
logger.info(f"Transport mode: {transport_mode}")

# Mount the appropriate MCP app
if transport_mode == "streamable_http" and hasattr(mcp_app, 'streamable_http_app'):
    try:
        # Use streamable HTTP transport which provides /mcp endpoint
        mcp_asgi_app = mcp_app.streamable_http_app()
        app.mount("/", mcp_asgi_app)
        logger.info("Mounted streamable_http_app - /mcp endpoint available")
    except Exception as e:
        logger.error(f"Failed to mount streamable_http_app: {e}")
        # Fall back to SSE
        sse_app = mcp_app.sse_app()
        app.mount("/", sse_app)
        logger.info("Fell back to SSE app")
else:
    # Default to SSE which provides /messages and /sse endpoints
    sse_app = mcp_app.sse_app()
    app.mount("/", sse_app)
    logger.info("Mounted SSE app - /messages and /sse endpoints available")


# Export for compatibility
def create_worker_app() -> FastAPI:
    """Create the worker app."""
    return app