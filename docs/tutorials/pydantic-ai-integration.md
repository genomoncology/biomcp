# Pydantic AI Integration Guide

This guide explains how to integrate BioMCP with Pydantic AI for building biomedical AI agents.

## Server Modes and Endpoints

BioMCP supports three transport modes for Pydantic AI integration:

### Available Transport Modes

| Mode              | Endpoints                  | Pydantic AI Client        | Use Case                     |
| ----------------- | -------------------------- | ------------------------- | ---------------------------- |
| `stdio`           | N/A (subprocess)           | `MCPServerStdio`          | Local development, testing   |
| `worker`          | `GET /sse`, `GET /health`  | `MCPServerSSE`            | Legacy SSE-based deployments |
| `streamable_http` | `POST /mcp`, `GET /health` | `MCPServerStreamableHTTP` | Production HTTP deployments  |

The streamable HTTP mode uses FastMCP's native streamable HTTP implementation for full MCP protocol compliance.

## Working Examples for Pydantic AI

Here are three working configurations for connecting Pydantic AI to BioMCP:

### 1. STDIO Mode (Recommended for Local Development)

This mode runs BioMCP as a subprocess without needing an HTTP server:

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio

async def main():
    # Run BioMCP as a subprocess
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    agent = Agent("openai:gpt-4o-mini", toolsets=[server])

    async with agent:
        # Verify connection
        tools = await agent.list_tools()
        print(f"Connected! Available tools: {len(tools)}")

        # Example query
        result = await agent.run(
            "Find articles about BRAF V600E mutations in melanoma"
        )
        print(result.output)

if __name__ == "__main__":
    asyncio.run(main())
```

### 2. SSE Mode (For HTTP Deployments)

If you're running BioMCP as an HTTP server, use the SSE endpoint:

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerSSE

async def main():
    # Connect to the SSE endpoint
    server = MCPServerSSE("http://localhost:8000/sse")

    agent = Agent("openai:gpt-4o-mini", toolsets=[server])

    async with agent:
        # Verify connection
        tools = await agent.list_tools()
        print(f"Connected! Available tools: {len(tools)}")

        # Example query
        result = await agent.run(
            "Search for active clinical trials for NSCLC with EGFR mutations"
        )
        print(result.output)

if __name__ == "__main__":
    asyncio.run(main())
```

To run the server for this mode:

```bash
# Using pip installation
biomcp run --mode worker --host 0.0.0.0 --port 8000

# Or using Docker
docker run -p 8000:8000 genomoncology/biomcp:latest biomcp run --mode worker
```

### 3. Streamable HTTP Mode (Recommended for Production)

For production deployments with proper MCP compliance (v0.6.9+):

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStreamableHTTP

async def main():
    # Connect to the /mcp endpoint
    server = MCPServerStreamableHTTP("http://localhost:8000/mcp")

    agent = Agent("openai:gpt-4o-mini", toolsets=[server])

    async with agent:
        # Example queries
        result = await agent.run(
            "Find recent articles about BRAF V600E in melanoma"
        )
        print(result.output)

if __name__ == "__main__":
    asyncio.run(main())
```

To run the server for this mode:

```bash
# Using pip installation (v0.6.9+)
biomcp run --mode streamable_http --host 0.0.0.0 --port 8000

# Or using Docker
docker run -p 8000:8000 genomoncology/biomcp:latest biomcp run --mode streamable_http
```

### 4. JSON-RPC Mode (Alternative HTTP)

You can also use the JSON-RPC endpoint at the root path:

```python
import httpx
import json

async def call_biomcp_jsonrpc(method, params=None):
    """Direct JSON-RPC calls to BioMCP"""
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://localhost:8000/",
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params or {}
            }
        )
        return response.json()

# Example usage
result = await call_biomcp_jsonrpc("tools/list")
print("Available tools:", result)
```

## Troubleshooting Common Issues

### Issue: Connection refused

**Solution**: Ensure the server is running with the correct host binding:

```bash
biomcp run --mode worker --host 0.0.0.0 --port 8000
```

### Issue: CORS errors in browser

**Solution**: The server includes CORS headers by default. If you still have issues, check if a proxy or firewall is blocking the headers.

### Issue: Health endpoint returns 404

**Solution**: The health endpoint is available at `GET /health`. Ensure you're using the latest version:

```bash
pip install --upgrade biomcp-python
```

## Testing Your Connection

Here's a simple test script to verify your setup:

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.models.test import TestModel
from pydantic_ai.mcp import MCPServerStdio

async def test_connection():
    # Use TestModel to avoid needing LLM credentials
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    agent = Agent(
        model=TestModel(call_tools=["search"]),
        toolsets=[server]
    )

    async with agent:
        tools = await agent.list_tools()
        print(f"✅ Connection successful!")
        print(f"📦 Found {len(tools)} tools")
        print(f"🔧 Tool names: {[t.name for t in tools[:5]]}...")

        # Test a simple search
        result = await agent.run("Test search for BRAF")
        print(f"✅ Tool execution successful!")

if __name__ == "__main__":
    asyncio.run(test_connection())
```

## Using BioMCP Tools with Pydantic AI

Once connected, you can use BioMCP's biomedical research tools:

```python
async def biomedical_research_example():
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    agent = Agent("openai:gpt-4o-mini", toolsets=[server])

    async with agent:
        # Important: Always use the think tool first for complex queries
        result = await agent.run("""
            First use the think tool to plan your approach, then:
            1. Search for articles about immunotherapy resistance in melanoma
            2. Find clinical trials testing combination therapies
            3. Look up genetic markers associated with treatment response
        """)

        print(result.output)
```

## Production Deployment Considerations

For production deployments:

1. **Use STDIO mode** when running in containerized environments where the agent and BioMCP can run in the same container
2. **Use SSE mode** when you need HTTP-based communication between separate services
3. **Implement proper error handling** and retry logic for network failures
4. **Set appropriate timeouts** for long-running biomedical searches
5. **Cache frequently accessed data** to reduce API calls to backend services

## Next Steps

- Review the [MCP Tools Reference](../user-guides/02-mcp-tools-reference.md) for available biomedical research tools
- See [CLI Guide](../user-guides/01-command-line-interface.md) for more server configuration options
- Check [Transport Protocol Guide](../developer-guides/04-transport-protocol.md) for detailed protocol information

## Support

If you continue to experience issues:

1. Verify your BioMCP version: `biomcp --version`
2. Check server logs for error messages
3. Open an issue on [GitHub](https://github.com/genomoncology/biomcp/issues) with:
   - Your BioMCP version
   - Server startup command
   - Complete error messages
   - Minimal reproduction code
