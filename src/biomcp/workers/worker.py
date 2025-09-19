"""Worker implementation for BioMCP."""

from fastapi import FastAPI, Response
from fastapi.middleware.cors import CORSMiddleware
from starlette.responses import JSONResponse
from starlette.routing import Route

from .. import mcp_app

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


# Add health endpoint to the streamable app before mounting
async def health_check(request):
    return JSONResponse({"status": "healthy"})


health_route = Route("/health", health_check, methods=["GET"])
streamable_app.routes.append(health_route)

app.mount("/", streamable_app)


# Health endpoint is now added directly to the streamable_app above


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
