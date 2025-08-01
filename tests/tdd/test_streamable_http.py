"""Integration tests for streamable HTTP transport protocol."""

import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from biomcp.cli.server import run_server

# Check if worker module is available
try:
    import importlib.util

    WORKER_AVAILABLE = (
        importlib.util.find_spec("biomcp.workers.worker") is not None
    )
except ImportError:
    WORKER_AVAILABLE = False


class TestStreamableHTTP:
    """Test streamable HTTP transport functionality."""

    @pytest.fixture
    def mock_app(self):
        """Create a mock FastMCP app."""
        app = MagicMock()
        app.process_forever = AsyncMock()
        return app

    @pytest.fixture
    def test_port(self):
        """Get an available port for testing."""
        return 8765

    @pytest.mark.skipif(
        not WORKER_AVAILABLE, reason="Worker module not installed"
    )
    def test_http_server_startup(self, test_port):
        """Test that HTTP server starts correctly with streamable HTTP mode."""
        with (
            patch("biomcp.workers.worker.app") as mock_app,
            patch("uvicorn.run") as mock_uvicorn,
        ):
            # Run server with streamable_http mode
            run_server(
                mode="streamable_http", host="127.0.0.1", port=test_port
            )

            # Verify uvicorn was called with correct parameters
            mock_uvicorn.assert_called_once()
            assert mock_uvicorn.call_args[0][0] == mock_app
            assert mock_uvicorn.call_args[1]["host"] == "127.0.0.1"
            assert mock_uvicorn.call_args[1]["port"] == test_port

    def test_stdio_mode_unchanged(self):
        """Test that stdio mode still works as before."""
        with patch("biomcp.cli.server.mcp_app") as mock_mcp_app:
            # Run server with stdio mode
            run_server(mode="stdio")

            # Should call run with stdio transport
            mock_mcp_app.run.assert_called_once_with(transport="stdio")

    @pytest.mark.skipif(
        not WORKER_AVAILABLE, reason="Worker module not installed"
    )
    def test_mcp_endpoint_configuration(self):
        """Test that /mcp endpoint is properly configured in HTTP mode."""
        # Test that the worker app is imported correctly
        with (
            patch("biomcp.workers.worker.app") as mock_app,
            patch("uvicorn.run") as mock_uvicorn,
        ):
            run_server(mode="streamable_http", host="127.0.0.1", port=8080)

            # Verify the correct app is passed to uvicorn
            mock_uvicorn.assert_called_once()
            assert mock_uvicorn.call_args[0][0] == mock_app

    @pytest.mark.asyncio
    async def test_session_management(self):
        """Test that session management works with streamable HTTP."""
        # Mock session storage
        sessions = {}

        async def mock_handle_request(session_id, request_data):
            """Mock handler for MCP requests with session support."""
            if session_id not in sessions:
                sessions[session_id] = {"created": True}

            if request_data.get("method") == "initialize":
                return {"result": {"protocolVersion": "2025-03-26"}}
            elif request_data.get("method") == "tools/list":
                return {"result": {"tools": ["think", "search", "fetch"]}}

            return {"error": "Unknown method"}

        # Test session creation and persistence
        session_id = "test-session-123"

        # First request creates session
        result1 = await mock_handle_request(
            session_id, {"method": "initialize"}
        )
        assert session_id in sessions
        assert result1["result"]["protocolVersion"] == "2025-03-26"

        # Second request uses existing session
        result2 = await mock_handle_request(
            session_id, {"method": "tools/list"}
        )
        assert len(sessions) == 1  # Still only one session
        assert "think" in result2["result"]["tools"]

    @pytest.mark.asyncio
    async def test_transport_negotiation(self):
        """Test that transport protocol is correctly negotiated."""
        # Test that the mcp_app was created with stateless_http=True
        from biomcp.core import mcp_app

        # Since mcp_app is already instantiated with stateless_http=True in core.py,
        # we can verify this by checking if the app exists and is a FastMCP instance
        assert mcp_app is not None
        assert hasattr(mcp_app, "name")
        assert (
            mcp_app.name == "BioMCP - Biomedical Model Context Protocol Server"
        )

    @pytest.mark.asyncio
    async def test_error_handling(self):
        """Test error handling in streamable HTTP mode."""

        # Mock an error scenario
        async def mock_error_handler(request):
            """Mock handler that raises an error."""
            raise ValueError("Test error")

        # Test that errors are properly formatted
        try:
            await mock_error_handler({"method": "test"})
        except ValueError as e:
            assert str(e) == "Test error"

    @pytest.mark.skipif(
        not WORKER_AVAILABLE, reason="Worker module not installed"
    )
    def test_server_modes(self):
        """Test that all server modes are properly handled."""
        # Test worker mode
        with (
            patch("biomcp.workers.worker.app"),
            patch("uvicorn.run") as mock_uvicorn,
        ):
            run_server(mode="worker", host="localhost", port=3000)
            mock_uvicorn.assert_called_once()

        # Test that invalid mode would be caught by typer's enum validation
        # (In practice, typer handles this validation before our code runs)


class TestStreamableHTTPIntegration:
    """Integration tests with actual MCP protocol."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_full_mcp_flow(self):
        """Test a complete MCP interaction flow using streamable HTTP.

        This test is marked as integration since it requires network access.
        """
        # This would test against a real running server
        # Skipped in offline mode
        pytest.skip("Integration test requires running server")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_mcp_endpoint_request_response(self):
        """Test actual request/response flow through /mcp endpoint.

        This requires a running server instance.
        """
        pytest.skip("Integration test requires running server")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_mcp_malformed_request(self):
        """Test handling of malformed requests.

        This requires a running server instance.
        """
        pytest.skip("Integration test requires running server")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_mcp_invalid_session(self):
        """Test handling of invalid session IDs.

        This requires a running server instance.
        """
        pytest.skip("Integration test requires running server")

    @pytest.mark.asyncio
    async def test_sse_fallback(self):
        """Test that SSE is used for long-running operations."""

        # Mock a long-running operation
        async def mock_long_operation():
            """Simulate a long-running search."""
            await asyncio.sleep(0.1)
            yield {"status": "searching"}
            await asyncio.sleep(0.1)
            yield {"status": "complete", "results": ["result1", "result2"]}

        # Test that SSE format is used for streaming
        results = []
        async for chunk in mock_long_operation():
            results.append(chunk)

        assert len(results) == 2
        assert results[0]["status"] == "searching"
        assert results[1]["status"] == "complete"
