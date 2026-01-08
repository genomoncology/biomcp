"""Authentication module for BioMCP remote server modes."""

import os
import secrets

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import JSONResponse


def validate_auth_token() -> str | None:
    """
    Validate MCP_AUTH_TOKEN environment variable at startup.

    Returns the token if valid, None if not set.
    Raises ValueError if token is set but invalid.
    """
    token = os.getenv("MCP_AUTH_TOKEN")
    if not token:
        return None

    token = token.strip()
    if not token:
        return None

    if len(token) < 32:
        raise ValueError("MCP_AUTH_TOKEN must be at least 32 characters long")

    return token


class BearerTokenMiddleware(BaseHTTPMiddleware):
    """
    Middleware to validate Bearer token for remote server authentication.

    If auth_token is provided, requests must include a matching
    Authorization: Bearer <token> header. If not provided, authentication
    is skipped (backwards compatible).
    """

    def __init__(self, app, auth_token: str | None = None):
        super().__init__(app)
        self._auth_token = auth_token

    async def dispatch(self, request: Request, call_next):
        # Skip auth if no token is configured
        if not self._auth_token:
            return await call_next(request)

        # Allow health checks without auth
        if request.url.path == "/health":
            return await call_next(request)

        # Allow CORS preflight requests without auth
        if request.method == "OPTIONS":
            return await call_next(request)

        auth_header = request.headers.get("Authorization", "")
        if not auth_header.startswith("Bearer "):
            return JSONResponse(
                {
                    "error": "unauthorized",
                    "error_description": "Missing or invalid Authorization header",
                },
                status_code=401,
                headers={
                    "WWW-Authenticate": "Bearer",
                    "Access-Control-Allow-Origin": "*",
                    "Access-Control-Allow-Headers": "*",
                },
            )

        token = auth_header[7:]  # Strip "Bearer " prefix
        if not secrets.compare_digest(token, self._auth_token):
            return JSONResponse(
                {
                    "error": "unauthorized",
                    "error_description": "Invalid token",
                },
                status_code=401,
                headers={
                    "WWW-Authenticate": "Bearer",
                    "Access-Control-Allow-Origin": "*",
                    "Access-Control-Allow-Headers": "*",
                },
            )

        return await call_next(request)
