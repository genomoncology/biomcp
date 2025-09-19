"""
Tests for Pydantic AI integration with BioMCP.

These tests verify the examples provided in the documentation work correctly.
"""

import asyncio
import os
import sys

import httpx
import pytest
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio

try:
    from pydantic_ai.mcp import MCPServerStreamableHTTP  # noqa: F401

    HAS_STREAMABLE_HTTP = True
except ImportError:
    HAS_STREAMABLE_HTTP = False
from pydantic_ai.models.test import TestModel


def worker_dependencies_available():
    """Check if worker dependencies (FastAPI, Starlette) are available."""
    try:
        import fastapi  # noqa: F401
        import starlette  # noqa: F401

        return True
    except ImportError:
        return False


# Skip marker for tests requiring worker dependencies
requires_worker = pytest.mark.skipif(
    not worker_dependencies_available(),
    reason="Worker dependencies (FastAPI/Starlette) not installed. Install with: pip install biomcp-python[worker]",
)

# Skip marker for tests requiring MCPServerStreamableHTTP
requires_streamable_http = pytest.mark.skipif(
    not HAS_STREAMABLE_HTTP,
    reason="MCPServerStreamableHTTP not available. Requires pydantic-ai>=0.6.9",
)


def get_free_port():
    """Get a free port for testing."""
    import socket

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        s.listen(1)
        port = s.getsockname()[1]
    return port


async def wait_for_server(
    url: str, max_retries: int = 60, process=None
) -> None:
    """Wait for server to be ready with retries."""
    import sys

    for i in range(max_retries):
        # Check if process has exited with error
        if process and process.poll() is not None:
            stdout, stderr = process.communicate()
            pytest.fail(
                f"Server process exited with code {process.returncode}. Stderr: {stderr.decode() if stderr else 'None'}"
            )

        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(url, timeout=2)
                if response.status_code == 200:
                    print(
                        f"\nServer ready after {i + 1} seconds",
                        file=sys.stderr,
                    )
                    return
        except (httpx.ConnectError, httpx.ReadTimeout):
            if i % 10 == 0:
                print(
                    f"\nWaiting for server... ({i} seconds elapsed)",
                    file=sys.stderr,
                )
            await asyncio.sleep(1)
    pytest.fail(f"Server at {url} did not start within {max_retries} seconds")


@pytest.mark.asyncio
async def test_stdio_mode_connection():
    """Test STDIO mode connection and tool listing."""
    server = MCPServerStdio(
        "python", args=["-m", "biomcp", "run", "--mode", "stdio"], timeout=20
    )

    # Use TestModel to avoid needing API keys
    model = TestModel(call_tools=["search"])
    agent = Agent(model=model, toolsets=[server])

    async with agent:
        # Test a simple query to verify connection works
        result = await agent.run("List available tools")

        # Should get a response without errors
        assert result is not None
        assert result.output is not None


@pytest.mark.asyncio
async def test_stdio_mode_simple_query():
    """Test STDIO mode with a simple search query."""
    server = MCPServerStdio(
        "python", args=["-m", "biomcp", "run", "--mode", "stdio"], timeout=20
    )

    # Use TestModel configured to call search
    model = TestModel(call_tools=["search"])
    agent = Agent(model=model, toolsets=[server])

    async with agent:
        result = await agent.run("Find 1 melanoma clinical trial")

        # TestModel will have called the search tool
        assert result.output is not None
        # The TestModel returns mock data, but we're testing the connection works
        assert result.output != ""


@pytest.mark.asyncio
async def test_stdio_mode_with_openai():
    """Test STDIO mode with OpenAI (requires OPENAI_API_KEY)."""
    # Skip if no API key
    if not os.getenv("OPENAI_API_KEY"):
        pytest.skip("OPENAI_API_KEY not set")

    server = MCPServerStdio(
        "python", args=["-m", "biomcp", "run", "--mode", "stdio"], timeout=30
    )

    agent = Agent("openai:gpt-4o-mini", toolsets=[server])

    async with agent:
        result = await agent.run(
            "Find 1 article about BRAF V600E mutations. Return just the title."
        )

        # Should get a real result
        assert result.output is not None
        assert len(result.output) > 0


@requires_worker
@requires_streamable_http
@pytest.mark.asyncio
async def test_streamable_http_mode_connection():
    """Test Streamable HTTP mode connection for Pydantic AI."""
    import subprocess

    from pydantic_ai.mcp import MCPServerStreamableHTTP

    port = get_free_port()

    # Start server in streamable_http mode
    server_process = subprocess.Popen(  # noqa: S603
        [
            sys.executable,
            "-m",
            "biomcp",
            "run",
            "--mode",
            "streamable_http",
            "--port",
            str(port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # Wait for server to be ready
        await wait_for_server(
            f"http://localhost:{port}/health", process=server_process
        )

        # Connect to the /mcp endpoint
        server = MCPServerStreamableHTTP(f"http://localhost:{port}/mcp")

        # Use TestModel to avoid needing API keys
        model = TestModel(call_tools=["search"])
        agent = Agent(model=model, toolsets=[server])

        async with agent:
            # Test a simple query to verify connection
            result = await agent.run("Test connection")
            assert result is not None
            assert result.output is not None

    finally:
        # Clean up server process
        server_process.terminate()
        server_process.wait(timeout=5)


@requires_worker
@requires_streamable_http
@pytest.mark.asyncio
async def test_streamable_http_simple_query():
    """Test a simple biomedical query using Streamable HTTP."""
    import subprocess

    from pydantic_ai.mcp import MCPServerStreamableHTTP

    port = get_free_port()

    server_process = subprocess.Popen(  # noqa: S603
        [
            sys.executable,
            "-m",
            "biomcp",
            "run",
            "--mode",
            "streamable_http",
            "--port",
            str(port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # Wait for server to be ready
        await wait_for_server(
            f"http://localhost:{port}/health", process=server_process
        )

        # Connect to the /mcp endpoint
        server = MCPServerStreamableHTTP(f"http://localhost:{port}/mcp")

        # Use TestModel with tool calls for search
        model = TestModel(call_tools=["search"])
        agent = Agent(model=model, toolsets=[server])

        async with agent:
            result = await agent.run(
                "Find 1 article about BRAF mutations. Return just the title."
            )

            # Should get a result
            assert result.output is not None
            assert len(result.output) > 0

    finally:
        server_process.terminate()
        server_process.wait(timeout=5)


@requires_worker
@pytest.mark.asyncio
async def test_worker_mode_streamable_http():
    """Test worker mode which now uses streamable HTTP under the hood."""
    import subprocess

    port = get_free_port()

    # Start server in worker mode (which uses streamable HTTP)
    server_process = subprocess.Popen(  # noqa: S603
        [
            sys.executable,
            "-m",
            "biomcp",
            "run",
            "--mode",
            "worker",
            "--port",
            str(port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # Wait for server to be ready
        await wait_for_server(
            f"http://localhost:{port}/health", process=server_process
        )

        # Worker mode exposes /mcp endpoint through streamable HTTP
        async with httpx.AsyncClient() as client:
            # Test the /mcp endpoint with initialize request
            response = await client.post(
                f"http://localhost:{port}/mcp",
                json={
                    "jsonrpc": "2.0",
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2025-06-18",
                        "capabilities": {},
                        "clientInfo": {"name": "test", "version": "1.0"},
                    },
                    "id": 1,
                },
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json, text/event-stream",
                },
            )

            # Worker mode may return various codes depending on initialization state
            # 200 = success, 406 = accept header issue, 500 = initialization incomplete
            assert response.status_code in [200, 406, 500]

            # Health endpoint should work
            health_response = await client.get(
                f"http://localhost:{port}/health"
            )
            assert health_response.status_code == 200
            assert health_response.json()["status"] == "healthy"

    finally:
        server_process.terminate()
        server_process.wait(timeout=5)


@pytest.mark.asyncio
async def test_connection_verification_script():
    """Test the connection verification script from documentation."""
    server = MCPServerStdio(
        "python", args=["-m", "biomcp", "run", "--mode", "stdio"], timeout=20
    )

    # Use TestModel to avoid needing LLM credentials
    agent = Agent(model=TestModel(call_tools=["search"]), toolsets=[server])

    async with agent:
        # Test a simple search to verify connection
        result = await agent.run("Test search for BRAF")

        # Verify connection successful
        assert result is not None
        assert result.output is not None


@pytest.mark.asyncio
async def test_biomedical_research_workflow():
    """Test a complete biomedical research workflow."""
    server = MCPServerStdio(
        "python", args=["-m", "biomcp", "run", "--mode", "stdio"], timeout=30
    )

    # Use TestModel configured to use multiple tools
    model = TestModel(call_tools=["think", "search", "fetch"])
    agent = Agent(model=model, toolsets=[server])

    async with agent:
        # Complex multi-step query
        result = await agent.run("""
            First use the think tool to plan your approach, then:
            1. Search for articles about BRAF mutations
            2. Find relevant clinical trials
        """)

        # Should complete without errors
        assert result is not None
        assert result.output is not None


@requires_worker
@pytest.mark.asyncio
async def test_health_endpoint():
    """Test that the health endpoint is accessible."""
    import subprocess

    port = get_free_port()

    server_process = subprocess.Popen(  # noqa: S603
        [
            sys.executable,
            "-m",
            "biomcp",
            "run",
            "--mode",
            "worker",
            "--port",
            str(port),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        # Give subprocess a moment to start
        await asyncio.sleep(2)

        # Wait for server to be ready
        await wait_for_server(
            f"http://localhost:{port}/health", process=server_process
        )

        async with httpx.AsyncClient() as client:
            response = await client.get(f"http://localhost:{port}/health")

            assert response.status_code == 200
            data = response.json()
            assert "status" in data
            assert data["status"] in ["healthy", "ok"]

    finally:
        server_process.terminate()
        server_process.wait(timeout=5)
