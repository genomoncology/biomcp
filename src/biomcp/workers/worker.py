"""Worker implementation for BioMCP."""

import asyncio
import json

from fastapi import FastAPI, Request, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse

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

streamable_app = mcp_app.streamable_http_app()
app.mount("/", streamable_app)

# Add any additional custom endpoints if needed
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


# Create a stub for create_worker_app to satisfy imports
def create_worker_app() -> FastAPI:
    """Stub for create_worker_app to satisfy import in __init__.py."""
    return app
