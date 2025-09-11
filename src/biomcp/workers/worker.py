"""Worker implementation for BioMCP."""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .. import mcp_app

# First, get the SSE app from MCP
sse_app = mcp_app.sse_app()

# Create our main FastAPI app
app = FastAPI(title="BioMCP Worker", version="0.1.10")

# Add CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Add health check endpoint BEFORE mounting SSE app
@app.get("/health")
async def health_check():
    return {"status": "healthy"}


# Mount the SSE app at root - this handles both /sse and JSON-RPC at /
app.mount("/", sse_app)


# Create a stub for create_worker_app to satisfy imports
def create_worker_app() -> FastAPI:
    """Stub for create_worker_app to satisfy import in __init__.py."""
    return app
