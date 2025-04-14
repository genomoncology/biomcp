# BioMCP with MCP Client Tutorial

## Overview

This tutorial explains how to integrate BioMCP with MCP clients programmatically. The Model Context Protocol (MCP) enables applications to interact with BioMCP as a specialized tool server, providing structured access to biomedical data sources.

## Key Features

- **STDIO Communication**: BioMCP uses standard input/output for MCP communication
- **Tool Discovery**: Clients can discover available tools through the MCP interface
- **Resource Access**: Provides access to BioMCP resources and instructions
- **Standard Protocol**: Follows the MCP 1.0 specification for compatibility with clients

## Main Components

The MCP integration relies on these key components:

1. **MCP Client**

   - Python client from the `mcp` package
   - Handles communication with the BioMCP server

2. **BioMCP Server**

   - Started as a subprocess with the `biomcp run` command
   - Exposes biomedical data tools through the MCP interface

3. **Tool Invocation**
   - Execute biomedical queries using standardized tool calls
   - Process structured responses for application integration

## Basic Usage Pattern

Integrating with BioMCP through MCP follows this pattern:

1. Start the BioMCP server as a subprocess
2. Create an MCP client session to communicate with the server
3. Discover available tools and resources
4. Call tools with appropriate parameters
5. Process the returned content

## Example Code

Here's a minimal example showing how to connect to BioMCP using the Python MCP client:

```python
import asyncio
from mcp.client.session import ClientSession
from mcp.client.stdio import StdioServerParameters, stdio_client

async def connect_to_biomcp():
    # Configure the BioMCP server
    server_params = StdioServerParameters(
        command="uv",
        args=["run", "--with", "biomcp-python", "biomcp", "run"]
    )

    # Connect to the server
    async with (
        stdio_client(server_params) as (read, write),
        ClientSession(read, write) as session
    ):
        # Initialize the session
        await session.initialize()

        # List available tools
        tool_result = await session.list_tools()
        print(f"Available tools: {[t.name for t in tool_result.tools]}")

        # Call a tool example (variant details)
        result = await session.call_tool(
            "variant_details",
            {"variant_id": "rs113488022"}
        )

        if not result.isError and result.content:
            # Access the text content from the first content block
            content = result.content[0].text
            print(f"Result snippet: {content[:100]}...")

# Run the example
if __name__ == "__main__":
    asyncio.run(connect_to_biomcp())
```

For a complete example of integrating BioMCP with an MCP client, see:

[BioMCP MCP Integration Example Script](https://github.com/genomoncology/biomcp/blob/main/example_scripts/mcp_integration.py)

## Available Tools

BioMCP exposes these tools through the MCP interface:

1. **Article Tools**

   - `article_searcher`: Search biomedical literature
   - `article_details`: Get details for a specific article

2. **Trial Tools**

   - `trial_searcher`: Search clinical trials
   - `trial_protocol`: Get trial protocol details
   - `trial_locations`: Get trial location information
   - `trial_outcomes`: Get trial outcome data
   - `trial_references`: Get trial references

3. **Variant Tools**
   - `variant_searcher`: Search genetic variants
   - `variant_details`: Get detailed variant information

## Integration Options

There are several ways to integrate BioMCP with MCP clients:

1. **Direct Python Integration**: Use the example above in Python applications
2. **Language-Specific Clients**: Implement MCP clients in other languages (JavaScript, etc.)
3. **AI Assistant Integration**: Configure LLM platforms to use BioMCP as a tool provider

## Troubleshooting

Common issues when integrating BioMCP:

- **Server Not Found**: Ensure BioMCP is installed and the command path is correct
- **Connection Errors**: Check subprocess management and STDIO handling
- **Tool Errors**: Verify tool names and parameter formats match BioMCP's expectations

## Next Steps

For more information on MCP integration with BioMCP:

- Explore the [MCP Client Library](https://github.com/modelcontextprotocol/python-client)
- Review the [MCP Specification](https://github.com/modelcontextprotocol/spec)
- Try the [MCP Inspector](https://github.com/modelcontextprotocol/inspector) for interactive debugging
