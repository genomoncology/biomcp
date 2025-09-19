# Pydantic AI Integration Guide

This guide explains how to integrate BioMCP with Pydantic AI for building biomedical AI agents.

## Server Modes and Endpoints

BioMCP supports two primary transport modes for Pydantic AI integration:

### Available Transport Modes

| Mode              | Endpoints                  | Pydantic AI Client        | Use Case                        |
| ----------------- | -------------------------- | ------------------------- | ------------------------------- |
| `stdio`           | N/A (subprocess)           | `MCPServerStdio`          | Local development, testing      |
| `streamable_http` | `POST /mcp`, `GET /health` | `MCPServerStreamableHTTP` | Production HTTP deployments     |
| `worker`          | `POST /mcp`, `GET /health` | `MCPServerStreamableHTTP` | HTTP mode using streamable HTTP |

Both `streamable_http` and `worker` modes now use FastMCP's native streamable HTTP implementation for full MCP protocol compliance. The SSE-based transport has been deprecated.

## Working Examples for Pydantic AI

Here are the recommended configurations for connecting Pydantic AI to BioMCP:

### 1. STDIO Mode (Recommended for Local Development)

This mode runs BioMCP as a subprocess without needing an HTTP server:

```python
import asyncio
import os
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio

async def main():
    # Run BioMCP as a subprocess
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    # Use a real LLM model (requires API key)
    model = "openai:gpt-4o-mini"  # Set OPENAI_API_KEY environment variable

    agent = Agent(model, toolsets=[server])

    async with agent:
        # Example query that returns real results
        result = await agent.run(
            "Find articles about BRAF V600E mutations in melanoma"
        )
        print(result.output)

if __name__ == "__main__":
    asyncio.run(main())
```

### 2. Streamable HTTP Mode (Recommended for Production)

For production deployments with proper MCP compliance (requires pydantic-ai>=0.6.9):

```python
import asyncio
import os
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStreamableHTTP

async def main():
    # Connect to the /mcp endpoint
    server = MCPServerStreamableHTTP("http://localhost:8000/mcp")

    # Use a real LLM model (requires API key)
    # Options: openai:gpt-4o-mini, anthropic:claude-3-haiku-20240307, groq:llama-3.1-70b-versatile
    model = "openai:gpt-4o-mini"  # Set OPENAI_API_KEY environment variable

    agent = Agent(model, toolsets=[server])

    async with agent:
        # Example queries that return real results
        result = await agent.run(
            "Find recent articles about BRAF V600E in melanoma"
        )
        print(result.output)

if __name__ == "__main__":
    asyncio.run(main())
```

To run the server for this mode:

```bash
# Using streamable_http mode (recommended)
biomcp run --mode streamable_http --host 0.0.0.0 --port 8000

# Or using worker mode (also uses streamable HTTP)
biomcp run --mode worker --host 0.0.0.0 --port 8000

# Or using Docker
docker run -p 8000:8000 genomoncology/biomcp:latest biomcp run --mode streamable_http
```

### 3. Direct JSON-RPC Mode (Alternative HTTP)

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

### Issue: TestModel returns empty results

**Cause**: TestModel is a mock model for testing - it doesn't execute real searches.

**Solution**: This is expected behavior. TestModel returns `{"search":{"results":[]}}` by design. To get real results:

- Use a real LLM model with API key: `Agent("openai:gpt-4o-mini", toolsets=[server])`
- Use Groq for free tier: Sign up at console.groq.com, get API key, use `Agent("groq:llama-3.1-70b-versatile", toolsets=[server])`
- Or use BioMCP CLI directly (no API key needed): `biomcp article search --gene BRAF`

### Issue: Connection refused

**Solution**: Ensure the server is running with the correct host binding:

```bash
biomcp run --mode worker --host 0.0.0.0 --port 8000
```

### Issue: CORS errors in browser

**Solution**: The server includes CORS headers by default. If you still have issues, check if a proxy or firewall is blocking the headers.

### Issue: Health endpoint returns 404

**Solution**: The health endpoint is available at `GET /health` in both worker and streamable_http modes. Ensure you're using the latest version:

```bash
pip install --upgrade biomcp-python
```

### Issue: SSE endpoint not found

**Solution**: The SSE transport has been deprecated. Use streamable HTTP mode instead:

```python
# Old (deprecated)
# from pydantic_ai.mcp import MCPServerSSE
# server = MCPServerSSE("http://localhost:8000/sse")

# New (recommended)
from pydantic_ai.mcp import MCPServerStreamableHTTP
server = MCPServerStreamableHTTP("http://localhost:8000/mcp")
```

## Testing Your Connection

Here are test scripts to verify your setup for different modes:

### Testing STDIO Mode (Local Development)

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.models.test import TestModel
from pydantic_ai.mcp import MCPServerStdio

async def test_stdio_connection():
    # Use TestModel to verify connection (won't return real data)
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    agent = Agent(
        model=TestModel(call_tools=["search"]),
        toolsets=[server]
    )

    async with agent:
        print(f"✅ STDIO Connection successful!")

        # Test a simple search (returns mock data)
        result = await agent.run("Test search for BRAF")
        print(f"✅ Tool execution successful!")
        print(f"Note: TestModel returns mock data: {result.output}")

if __name__ == "__main__":
    asyncio.run(test_stdio_connection())
```

### Testing Streamable HTTP Mode (Production)

First, ensure the server is running:

```bash
# Start the server in a separate terminal
biomcp run --mode streamable_http --port 8000
```

Then test the connection:

```python
import asyncio
from pydantic_ai import Agent
from pydantic_ai.models.test import TestModel
from pydantic_ai.mcp import MCPServerStreamableHTTP

async def test_streamable_http_connection():
    # Connect to the running server's /mcp endpoint
    server = MCPServerStreamableHTTP("http://localhost:8000/mcp")

    # Create agent with TestModel (no API keys needed)
    agent = Agent(
        model=TestModel(call_tools=["search"]),
        toolsets=[server]
    )

    async with agent:
        print("✅ Streamable HTTP Connection successful!")

        # Test a query
        result = await agent.run("Find articles about BRAF")
        print("✅ Tool execution successful!")
        if result.output:
            print(f"📄 Received {len(result.output)} characters of output")

if __name__ == "__main__":
    asyncio.run(test_streamable_http_connection())
```

### Important: Understanding TestModel vs Real Results

**TestModel is a MOCK model** - it doesn't execute real searches:

- TestModel simulates tool calls but returns empty results: `{"search":{"results":[]}}`
- This is by design - TestModel is for testing the connection flow, not getting real data
- To get actual search results, you need to use a real LLM model

**To get real results:**

1. **Use a real LLM model** (requires API key):

```python
# Replace TestModel with a real model
agent = Agent(
    "openai:gpt-4o-mini",  # or "anthropic:claude-3-haiku"
    toolsets=[server]
)
```

2. **Use BioMCP CLI directly** (no API key needed):

```bash
# Get real search results via CLI
biomcp article search --gene BRAF --disease melanoma --json
```

3. **For integration testing** without API keys:

```python
import subprocess
import json

# Use CLI to get real results
result = subprocess.run(
    ["biomcp", "article", "search", "--gene", "BRAF", "--json"],
    capture_output=True,
    text=True
)
data = json.loads(result.stdout)
print(f"Found {len(data['articles'])} real articles")
```

**Note**: The Streamable HTTP tests in our test suite verify this functionality works correctly. If you encounter connection issues, ensure:

1. The server is fully started before connecting
2. You're using pydantic-ai >= 0.6.9
3. The port is not blocked by a firewall

### Complete Working Example with Real Results

Here's a complete example that connects to BioMCP via Streamable HTTP and retrieves real biomedical data:

```python
#!/usr/bin/env python3
"""
Working example of Pydantic AI + BioMCP with Streamable HTTP.
This will get real search results from your BioMCP server.

Requires one of:
- export OPENAI_API_KEY='your-key'
- export ANTHROPIC_API_KEY='your-key'
- export GROQ_API_KEY='your-key'  (free tier at console.groq.com)
"""

import asyncio
import os
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStreamableHTTP


async def main():
    # Server configuration
    SERVER_URL = "http://localhost:8000/mcp"  # Adjust port as needed

    # Detect which API key is available
    if os.getenv("OPENAI_API_KEY"):
        model = "openai:gpt-4o-mini"
        print("Using OpenAI GPT-4o-mini")
    elif os.getenv("ANTHROPIC_API_KEY"):
        model = "anthropic:claude-3-haiku-20240307"
        print("Using Claude 3 Haiku")
    elif os.getenv("GROQ_API_KEY"):
        model = "groq:llama-3.1-70b-versatile"  # Free tier available
        print("Using Groq Llama 3.1")
    else:
        print("No API key found! Please set OPENAI_API_KEY, ANTHROPIC_API_KEY, or GROQ_API_KEY")
        return

    # Connect to BioMCP server
    server = MCPServerStreamableHTTP(SERVER_URL)
    agent = Agent(model, toolsets=[server])

    async with agent:
        print("Connected to BioMCP!\n")

        # Search for articles (includes cBioPortal data for genes)
        result = await agent.run(
            "Search for 2 recent articles about BRAF V600E mutations in melanoma. "
            "List the title and first author for each."
        )
        print("Article Search Results:")
        print(result.output)
        print("\n" + "="*60 + "\n")

        # Search for clinical trials
        result2 = await agent.run(
            "Find 2 clinical trials for melanoma with BRAF mutations "
            "that are currently recruiting. Show NCT ID and title."
        )
        print("Clinical Trial Results:")
        print(result2.output)
        print("\n" + "="*60 + "\n")

        # Search for variant information
        result3 = await agent.run(
            "Search for pathogenic TP53 variants. Show 2 examples."
        )
        print("Variant Search Results:")
        print(result3.output)


if __name__ == "__main__":
    # Start your BioMCP server first:
    # biomcp run --mode streamable_http --port 8000

    asyncio.run(main())
```

**Running this example:**

1. Start the BioMCP server:

```bash
biomcp run --mode streamable_http --port 8000
```

2. Set your API key (choose one):

```bash
export OPENAI_API_KEY='your-key'        # OpenAI
export ANTHROPIC_API_KEY='your-key'     # Anthropic
export GROQ_API_KEY='your-key'          # Groq (free tier available)
```

3. Run the script:

```bash
python biomcp_example.py
```

This will return actual biomedical data from PubMed, ClinicalTrials.gov, and variant databases!

## Using BioMCP Tools with Pydantic AI

Once connected, you can use BioMCP's biomedical research tools:

```python
import os
from pydantic_ai import Agent
from pydantic_ai.mcp import MCPServerStdio

async def biomedical_research_example():
    server = MCPServerStdio(
        "python",
        args=["-m", "biomcp", "run", "--mode", "stdio"]
    )

    # Choose model based on available API key
    if os.getenv("OPENAI_API_KEY"):
        model = "openai:gpt-4o-mini"
    elif os.getenv("GROQ_API_KEY"):
        model = "groq:llama-3.1-70b-versatile"  # Free tier available
    else:
        raise ValueError("Please set OPENAI_API_KEY or GROQ_API_KEY")

    agent = Agent(model, toolsets=[server])

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

1. **Use STDIO mode** for local development or when running in containerized environments where the agent and BioMCP can run in the same container
2. **Use Streamable HTTP mode** when you need HTTP-based communication between separate services (recommended for production)
3. **Both `worker` and `streamable_http` modes** now use the same underlying streamable HTTP transport
4. **Require a real LLM model** - TestModel won't work for production as it only returns mock data
5. **Consider API costs** - Use cheaper models like `gpt-4o-mini` or Groq's free tier for testing
6. **Implement proper error handling** and retry logic for network failures
7. **Set appropriate timeouts** for long-running biomedical searches
8. **Cache frequently accessed data** to reduce API calls to backend services

### Important Notes

- **Real LLM required for results**: TestModel is only for testing connections - use a real LLM (OpenAI, Anthropic, Groq) to get actual biomedical data
- **SSE transport is deprecated**: The old SSE-based transport (`/sse` endpoint) has been removed in favor of streamable HTTP
- **Worker mode now uses streamable HTTP**: The `worker` mode has been updated to use streamable HTTP transport internally
- **Health endpoint**: The `/health` endpoint is available in both HTTP modes for monitoring
- **Free tier option**: Groq offers a free API tier at console.groq.com for testing without costs

## Migration Guide from SSE to Streamable HTTP

If you're upgrading from an older version that used SSE transport:

### Code Changes

```python
# Old code (deprecated)
from pydantic_ai.mcp import MCPServerSSE
server = MCPServerSSE("http://localhost:8000/sse")

# New code (recommended)
from pydantic_ai.mcp import MCPServerStreamableHTTP
server = MCPServerStreamableHTTP("http://localhost:8000/mcp")
```

### Server Command Changes

```bash
# Old: SSE endpoints were at /sse
# biomcp run --mode worker  # Used to expose /sse endpoint

# New: Both modes now use /mcp endpoint with streamable HTTP
biomcp run --mode worker         # Now uses /mcp with streamable HTTP
biomcp run --mode streamable_http # Also uses /mcp with streamable HTTP
```

### Key Differences

1. **Endpoint Change**: `/sse` → `/mcp`
2. **Protocol**: Server-Sent Events → Streamable HTTP (supports both JSON and SSE)
3. **Client Library**: `MCPServerSSE` → `MCPServerStreamableHTTP`
4. **Compatibility**: Requires pydantic-ai >= 0.6.9 for `MCPServerStreamableHTTP`

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
