"""Worker implementation for BioMCP."""

import asyncio
import json

from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse

from .. import logger, mcp_app

app = FastAPI(title="BioMCP Worker")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Mount the MCP SSE app - this is the key change
# This will handle /sse and /messages endpoints automatically
sse_app = mcp_app.sse_app()
app.mount("/", sse_app)


# Remove the problematic middleware that causes ASGI errors
# The logging can be done at the endpoint level if needed


# Add any additional custom endpoints if needed
@app.get("/health")
async def health_check():
    return {"status": "healthy"}


# Handle MCP requests at the root path (this is where mcp-remote sends them)
@app.post("/")
async def handle_root_mcp_request(request: Request):
    """Handle MCP protocol messages at the root path."""
    try:
        # Get request body from state (already read in middleware)
        if hasattr(request.state, "body"):
            body_bytes = request.state.body
        else:
            body_bytes = await request.body()

        body_str = body_bytes.decode("utf-8")
        logger.info(f"Processing MCP request at root path: {body_str}")

        # Parse JSON
        body = json.loads(body_str)

        # Process the request
        response = await mcp_app.process_request(body)  # type: ignore[attr-defined]
        logger.info(f"Response: {response}")

        # Return the response
        return Response(
            content=json.dumps(response), media_type="application/json"
        )
    except Exception as e:
        logger.error(f"Error processing MCP request: {e}", exc_info=True)
        error_response = {
            "jsonrpc": "2.0",
            "error": {"code": -32603, "message": f"Internal error: {e!s}"},
            "id": body.get("id") if "body" in locals() else None,
        }
        return Response(
            content=json.dumps(error_response),
            media_type="application/json",
            status_code=500,
        )


# Keep the /mcp endpoint for backward compatibility
@app.post("/mcp")
async def handle_mcp_endpoint(request: Request):
    """Handle MCP protocol messages at the /mcp endpoint."""
    return await handle_root_mcp_request(request)


# Add the SSE endpoint
@app.get("/sse")
async def sse_endpoint():
    """
    Server-Sent Events (SSE) endpoint for remote MCP connections.

    This endpoint establishes a persistent connection with the client
    and sends events as they occur.
    """
    logger.info("SSE connection established")

    async def event_generator():
        init_metadata = {
            "protocol_version": "1.9.1",
            "server_capabilities": ["tools", "resources"],
        }
        yield f"event: ready\ndata: {json.dumps(init_metadata)}\n\n"
        logger.info(f"SSE sent initial event: {init_metadata}")

        # Keep the connection alive with keepalive events
        while True:
            await asyncio.sleep(15)
            logger.debug("Sending keepalive")
            yield ":keepalive\n\n"

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "Access-Control-Allow-Origin": "*",
            "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
            "Access-Control-Allow-Headers": "*",
        },
    )


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


# Create a stub for create_worker_app to satisfy imports


def create_worker_app() -> FastAPI:
    """Stub for create_worker_app to satisfy import in __init__.py."""
    return app
