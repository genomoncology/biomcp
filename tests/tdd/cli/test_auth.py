"""Unit tests for STREAMABLE_HTTP authentication functionality."""

import os
from unittest.mock import patch

import pytest
from starlette.applications import Starlette
from starlette.responses import PlainTextResponse
from starlette.routing import Route
from starlette.testclient import TestClient

from biomcp.auth import BearerTokenMiddleware, validate_auth_token


class TestValidateAuthToken:
    """Tests for validate_auth_token function."""

    def test_validate_token_not_set(self):
        """Test that None is returned when MCP_AUTH_TOKEN is not set."""
        with patch.dict(os.environ, {}, clear=True):
            result = validate_auth_token()
            assert result is None

    def test_validate_token_empty_string(self):
        """Test that None is returned when MCP_AUTH_TOKEN is empty string."""
        with patch.dict(os.environ, {"MCP_AUTH_TOKEN": ""}, clear=True):
            result = validate_auth_token()
            assert result is None

    def test_validate_token_whitespace_only(self):
        """Test that None is returned when MCP_AUTH_TOKEN is whitespace only."""
        with patch.dict(os.environ, {"MCP_AUTH_TOKEN": "   "}, clear=True):
            result = validate_auth_token()
            assert result is None

    def test_validate_token_too_short(self):
        """Test that ValueError is raised when token is less than 32 chars."""
        with (
            patch.dict(os.environ, {"MCP_AUTH_TOKEN": "short"}, clear=True),
            pytest.raises(ValueError, match="at least 32 characters"),
        ):
            validate_auth_token()

    def test_validate_token_31_chars_fails(self):
        """Test that exactly 31 characters fails validation."""
        token_31 = "a" * 31
        with (
            patch.dict(os.environ, {"MCP_AUTH_TOKEN": token_31}, clear=True),
            pytest.raises(ValueError, match="at least 32 characters"),
        ):
            validate_auth_token()

    def test_validate_token_exactly_32_chars(self):
        """Test that exactly 32 characters passes validation."""
        token_32 = "a" * 32
        with patch.dict(os.environ, {"MCP_AUTH_TOKEN": token_32}, clear=True):
            result = validate_auth_token()
            assert result == token_32

    def test_validate_token_long(self):
        """Test that a long token (64 chars) passes validation."""
        token_64 = "b" * 64
        with patch.dict(os.environ, {"MCP_AUTH_TOKEN": token_64}, clear=True):
            result = validate_auth_token()
            assert result == token_64

    def test_validate_token_strips_whitespace(self):
        """Test that whitespace is stripped from valid token."""
        token_32 = "c" * 32
        with patch.dict(
            os.environ, {"MCP_AUTH_TOKEN": f"  {token_32}  "}, clear=True
        ):
            result = validate_auth_token()
            assert result == token_32


class TestBearerTokenMiddleware:
    """Tests for BearerTokenMiddleware class."""

    @pytest.fixture
    def test_app(self):
        """Create a test Starlette app with health and other endpoints."""

        async def homepage(request):
            return PlainTextResponse("OK")

        async def health(request):
            return PlainTextResponse("healthy")

        app = Starlette(
            routes=[
                Route("/", homepage),
                Route("/health", health),
                Route("/mcp", homepage),
            ]
        )
        return app

    def test_middleware_no_token_configured(self, test_app):
        """Test that requests pass through when no token is configured."""
        test_app.add_middleware(BearerTokenMiddleware, auth_token=None)
        client = TestClient(test_app)

        response = client.get("/")
        assert response.status_code == 200
        assert response.text == "OK"

    def test_middleware_health_endpoint_bypasses_auth(self, test_app):
        """Test that /health endpoint bypasses auth even when token is set."""
        valid_token = "d" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        # Health endpoint should work without auth
        response = client.get("/health")
        assert response.status_code == 200
        assert response.text == "healthy"

    def test_middleware_options_preflight_bypasses_auth(self, test_app):
        """Test that OPTIONS preflight requests bypass auth for CORS."""
        valid_token = "k" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        # OPTIONS request without auth should succeed (for CORS preflight)
        response = client.options("/mcp")
        assert response.status_code != 401

    def test_middleware_missing_auth_header(self, test_app):
        """Test that missing Authorization header returns 401."""
        valid_token = "e" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        response = client.get("/mcp")
        assert response.status_code == 401
        data = response.json()
        assert data["error"] == "unauthorized"
        assert "Authorization" in data["error_description"]

    def test_middleware_invalid_auth_format(self, test_app):
        """Test that non-Bearer auth format returns 401."""
        valid_token = "f" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        # Try Basic auth instead of Bearer
        response = client.get(
            "/mcp", headers={"Authorization": "Basic xyz123"}
        )
        assert response.status_code == 401
        data = response.json()
        assert data["error"] == "unauthorized"

    def test_middleware_wrong_token(self, test_app):
        """Test that wrong Bearer token returns 401."""
        valid_token = "g" * 32
        wrong_token = "h" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        response = client.get(
            "/mcp", headers={"Authorization": f"Bearer {wrong_token}"}
        )
        assert response.status_code == 401
        data = response.json()
        assert data["error"] == "unauthorized"
        assert "Invalid token" in data["error_description"]

    def test_middleware_correct_token(self, test_app):
        """Test that correct Bearer token allows request through."""
        valid_token = "i" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        response = client.get(
            "/mcp", headers={"Authorization": f"Bearer {valid_token}"}
        )
        assert response.status_code == 200
        assert response.text == "OK"

    def test_middleware_401_response_format(self, test_app):
        """Test that 401 responses have correct JSON format."""
        valid_token = "j" * 32
        test_app.add_middleware(BearerTokenMiddleware, auth_token=valid_token)
        client = TestClient(test_app)

        response = client.get("/mcp")
        assert response.status_code == 401

        # Verify JSON response structure
        data = response.json()
        assert "error" in data
        assert "error_description" in data
        assert data["error"] == "unauthorized"
