# BioMCP Integration Guide

This guide provides comprehensive instructions for integrating BioMCP with various LLM platforms and MCP clients. It explains how to use BioMCP as an MCP server to enhance AI assistants with biomedical knowledge.

## Table of Contents

- [Understanding BioMCP as an MCP Server](#understanding-biomcp-as-an-mcp-server)
- [BioMCP MCP Architecture](#biomcp-mcp-architecture)
- [Integration with Claude Desktop](#integration-with-claude-desktop)
- [Integration with Other MCP Clients](#integration-with-other-mcp-clients)
- [Troubleshooting](#troubleshooting)
- [Example Conversations](#example-conversations)

## Understanding BioMCP as an MCP Server

BioMCP can function as a Model Context Protocol (MCP) server, enabling AI assistants to access specialized biomedical data sources through a standardized interface. The `biomcp run` command starts BioMCP in server mode, where it listens for MCP requests and provides responses.

### What is the `biomcp run` Command?

The `biomcp run` command starts BioMCP as an MCP server that:

1. **Listens for Input**: Waits for MCP-formatted JSON requests on standard input (stdin)
2. **Processes Requests**: Interprets requests, executes the appropriate tool, and retrieves data from biomedical sources
3. **Returns Results**: Sends MCP-formatted JSON responses to standard output (stdout)
4. **Maintains State**: Keeps the server running until terminated by a signal or closed input stream

Example of starting the server:

```bash
biomcp run
```

When run, the server initializes and waits silently for input. It's designed to be managed by a parent process (like an LLM client) that handles the communication pipeline.

### STDIO Transport Mechanism

BioMCP uses Standard Input/Output (STDIO) as its transport mechanism, which:

- **Simplifies Integration**: Works with any system that can spawn child processes
- **Reduces Dependencies**: Requires no network configuration or additional services
- **Enhances Security**: Limits communication to the parent process only
- **Enables Cross-Platform Use**: Functions consistently across operating systems

The STDIO transport works as follows:

1. The parent process (LLM client) spawns BioMCP as a child process
2. The client writes MCP requests as JSON to BioMCP's stdin
3. BioMCP processes the request and writes the response as JSON to stdout
4. The client reads the response from BioMCP's stdout

This approach makes BioMCP easy to integrate with various AI platforms and tools.

### Protocol Version

BioMCP implements version 1.0 of the Model Context Protocol. All requests should include the protocol version in the `mcp_protocol_version` field:

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "article_searcher",
  "tool_input": { ... }
}
```

### Input Validation and Error Handling

BioMCP performs validation on all incoming requests and provides structured error responses when issues are detected:

- **Missing or Invalid Fields**: If required fields are missing or have invalid types, BioMCP will return an error message specifying the issue
- **Tool Not Found**: If the requested tool doesn't exist, BioMCP will return an error indicating the tool wasn't found
- **API Errors**: If an underlying API returns an error, BioMCP will format it appropriately in the response

Error responses follow this format:

```json
{
  "result": null,
  "error": "Detailed error message describing the issue"
}
```

When handling BioMCP responses, always check for the presence of an `error` field before processing the `result`.

### Available MCP Tools

BioMCP exposes the following tools through the MCP interface:

#### Article Tools

- `article_searcher`: Searches PubMed/PubTator3 for biomedical literature

  - Input: `PubmedRequest` with fields for genes, variants, diseases, chemicals, and keywords
  - Output: Markdown-formatted list of matching articles

- `article_details`: Fetches detailed information for a specific article
  - Input: PubMed ID (PMID)
  - Output: Markdown-formatted article details including abstract and annotations

#### Trial Tools

- `trial_searcher`: Searches ClinicalTrials.gov for clinical trials

  - Input: `TrialQuery` with fields for conditions, interventions, recruiting status, etc.
  - Output: Markdown-formatted list of matching trials

- `trial_protocol`: Fetches protocol details for a specific trial

  - Input: NCT ID
  - Output: Markdown-formatted protocol details

- `trial_locations`: Fetches location information for a specific trial

  - Input: NCT ID
  - Output: Markdown-formatted location details

- `trial_outcomes`: Fetches outcome/results information for a specific trial

  - Input: NCT ID
  - Output: Markdown-formatted outcome details

- `trial_references`: Fetches publication references for a specific trial
  - Input: NCT ID
  - Output: Markdown-formatted reference details

#### Variant Tools

- `variant_searcher`: Searches MyVariant.info for genetic variants

  - Input: `VariantQuery` with fields for gene, protein change, significance, etc.
  - Output: Markdown-formatted list of matching variants

- `variant_details`: Fetches detailed annotation for a specific variant
  - Input: Variant ID
  - Output: Markdown-formatted variant details including clinical significance

## BioMCP MCP Architecture

BioMCP implements the Model Context Protocol to bridge AI assistants with specialized biomedical data sources:

![BioMCP MCP Architecture](assets/mcp_architecture.txt)

_Figure: BioMCP MCP Architecture showing the flow between AI assistants, BioMCP, and biomedical data sources_

### Key Components:

1. **AI Assistant (MCP Client)**: An LLM-powered application that needs access to biomedical data
2. **BioMCP MCP Server**: The bridge that translates between MCP requests and specialized API calls
3. **Biomedical Data Sources**: External APIs that provide specialized biomedical information
4. **Entity Normalization**: BioMCP's capability to map natural language terms to standardized identifiers
5. **Cache**: Performance optimization to reduce redundant API calls

### Data Flow:

1. The AI assistant receives a user query requiring biomedical information
2. The assistant formulates an MCP request and sends it to BioMCP
3. BioMCP processes the request, normalizes entities, and queries the appropriate data source
4. The data source returns raw information to BioMCP
5. BioMCP transforms the raw data into a structured, human-readable format
6. BioMCP sends the formatted response back to the AI assistant
7. The AI assistant incorporates the information into its response to the user

This architecture enables AI assistants to access specialized biomedical knowledge without needing to understand the complexities of multiple biomedical APIs.

## Integration with Claude Desktop

Claude Desktop is an AI assistant application that supports MCP tools through its configuration system. Here's how to integrate BioMCP with Claude Desktop:

### Configuration Steps

1. **Locate the Claude Desktop configuration file**:

   - On macOS: `~/Library/Application Support/Claude Desktop/config.json`
   - On Windows: `%APPDATA%\Claude Desktop\config.json`
   - On Linux: `~/.config/Claude Desktop/config.json`

2. **Edit the configuration file** to add BioMCP as an MCP server:

```json
{
  "globalShortcut": "",
  "mcpServers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["run"]
    }
  }
}
```

If you're using `uv` to manage Python packages, you can use this configuration instead:

```json
{
  "globalShortcut": "",
  "mcpServers": {
    "biomcp": {
      "command": "uv",
      "args": ["run", "--from", "biomcp-python", "biomcp", "run"]
    }
  }
}
```

3. **Save the configuration file** and restart Claude Desktop

### Verification Steps

To verify that BioMCP is properly integrated with Claude Desktop:

1. **Open Claude Desktop** and start a new conversation
2. **Ask a biomedical question** that would require BioMCP, such as:
   - "Can you find recent clinical trials for melanoma treatment?"
   - "What are the pathogenic variants in the BRCA1 gene?"
   - "Find recent research articles about EGFR inhibitors in lung cancer."
3. **Check the response**: Claude should use BioMCP to retrieve and display relevant information
4. **Verify tool usage**: Claude should mention that it's using BioMCP to access biomedical data

If Claude fails to use BioMCP, check the troubleshooting section below.

## Integration with Other MCP Clients

BioMCP can be integrated with various MCP clients beyond Claude Desktop. Here are examples for different platforms:

### Integration with Anthropic API

You can use BioMCP with the Anthropic API by implementing a client that manages the MCP server process and handles the communication:

```python
import json
import subprocess
import anthropic

# Start BioMCP as a subprocess
biomcp_process = subprocess.Popen(
    ["biomcp", "run"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
    bufsize=1
)

# Function to call BioMCP tools
def call_biomcp_tool(tool_name, tool_input):
    # Create MCP request
    mcp_request = {
        "mcp_protocol_version": "1.0",
        "tool_name": tool_name,
        "tool_input": tool_input
    }

    # Send request to BioMCP
    biomcp_process.stdin.write(json.dumps(mcp_request) + "\n")
    biomcp_process.stdin.flush()

    # Read response from BioMCP
    response_line = biomcp_process.stdout.readline()
    mcp_response = json.loads(response_line)

    return mcp_response["result"]

# Initialize Anthropic client
client = anthropic.Anthropic(api_key="your_api_key")

# Example: Search for melanoma clinical trials
trial_results = call_biomcp_tool("trial_searcher", {
    "conditions": ["Melanoma"],
    "recruiting_status": "OPEN"
})

# Send results to Claude
message = client.messages.create(
    model="claude-3-opus-20240229",
    max_tokens=1000,
    messages=[
        {"role": "user", "content": "What can you tell me about these melanoma clinical trials?\n\n" + trial_results}
    ]
)

print(message.content)

# Clean up
biomcp_process.terminate()
```

### Integration with JavaScript MCP Client

You can use the `@modelcontextprotocol/client` JavaScript library to integrate BioMCP:

```javascript
const { spawn } = require("child_process");
const { MCPClient } = require("@modelcontextprotocol/client");

// Start BioMCP process
const biomcpProcess = spawn("biomcp", ["run"]);

// Create MCP client
const mcpClient = new MCPClient({
  input: biomcpProcess.stdout,
  output: biomcpProcess.stdin,
});

// Example: Search for articles about BRAF in melanoma
async function searchArticles() {
  try {
    const result = await mcpClient.callTool("article_searcher", {
      genes: ["BRAF"],
      diseases: ["Melanoma"],
    });

    console.log("Article search results:");
    console.log(result);
  } catch (error) {
    console.error("Error calling BioMCP:", error);
  } finally {
    // Clean up
    biomcpProcess.kill();
  }
}

searchArticles();
```

### Integration with Python MCP Client

Here's an example of integrating BioMCP with a Python MCP client:

```python
import asyncio
import json
import subprocess
from typing import Any, Dict

class MCPClient:
    def __init__(self):
        self.process = subprocess.Popen(
            ["biomcp", "run"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            bufsize=1
        )

    async def call_tool(self, tool_name: str, tool_input: Dict[str, Any]) -> str:
        # Create MCP request
        mcp_request = {
            "mcp_protocol_version": "1.0",
            "tool_name": tool_name,
            "tool_input": tool_input
        }

        # Send request to BioMCP
        self.process.stdin.write(json.dumps(mcp_request) + "\n")
        self.process.stdin.flush()

        # Read response from BioMCP
        response_line = self.process.stdout.readline()
        mcp_response = json.loads(response_line)

        if mcp_response.get("error"):
            raise Exception(f"BioMCP error: {mcp_response['error']}")

        return mcp_response["result"]

    def close(self):
        self.process.terminate()

async def main():
    client = MCPClient()
    try:
        # Example: Get variant details
        variant_details = await client.call_tool("variant_details", "rs113488022")
        print("Variant details:")
        print(variant_details)
    finally:
        client.close()

if __name__ == "__main__":
    asyncio.run(main())
```

### Using MCP Inspector for Debugging

The MCP Inspector is a useful tool for testing and debugging BioMCP integration. It allows you to interact with BioMCP directly without needing to integrate with an AI assistant:

```bash
# Install the MCP Inspector
npm install -g @modelcontextprotocol/inspector

# Use it with BioMCP
npx @modelcontextprotocol/inspector biomcp run
```

This will open an interactive interface where you can:

1. Browse available tools
2. Construct and send MCP requests
3. View the raw responses
4. Test different input parameters

The MCP Inspector is particularly useful for:

- Verifying that BioMCP is functioning correctly
- Understanding the expected input format for each tool
- Seeing the exact output format that will be returned
- Debugging issues with specific queries

## Troubleshooting

### Security Considerations

When integrating BioMCP with AI assistants, consider these security aspects:

1. **Data Privacy**: BioMCP queries public biomedical databases that don't contain patient-specific information. However, be cautious about the queries you send, as they might reveal research interests or focus areas.

2. **API Usage Policies**: Respect the terms of service for the underlying APIs (PubMed, ClinicalTrials.gov, MyVariant.info). BioMCP implements rate limiting to help with this, but you should review their policies if you plan heavy usage.

3. **Process Isolation**: The STDIO transport provides natural isolation between the AI assistant and BioMCP. This isolation helps prevent potential security issues that might arise with network-based services.

4. **Input Validation**: While BioMCP validates inputs, it's good practice to sanitize any user-provided data before including it in BioMCP queries.

5. **Commercial Usage**: If you're using BioMCP in a commercial application, review the license terms and the terms of service for the underlying APIs.

### Common Connection Issues

| Issue                      | Possible Causes                     | Solutions                                                       |
| -------------------------- | ----------------------------------- | --------------------------------------------------------------- |
| `SPAWN ENOENT` error       | BioMCP executable not found in PATH | Provide full path to BioMCP or ensure it's in your PATH         |
| Process hangs              | Communication deadlock              | Ensure proper stdin/stdout handling; check for buffering issues |
| Process exits unexpectedly | Exception in BioMCP                 | Check logs; ensure valid input parameters                       |
| No response from BioMCP    | Input not properly formatted        | Verify JSON format; ensure newline after each request           |
| Permission denied          | Executable permissions              | Check file permissions; use `chmod +x` if needed                |

### MCP Tool Invocation Problems

| Issue                    | Possible Causes        | Solutions                                             |
| ------------------------ | ---------------------- | ----------------------------------------------------- |
| "Tool not found" error   | Incorrect tool name    | Verify tool name against documentation                |
| "Invalid input" error    | Malformed input JSON   | Check input structure against tool requirements       |
| Empty or null result     | No matching data found | Refine search parameters; check for typos             |
| Timeout                  | External API delay     | Increase timeout settings; check network connectivity |
| Unexpected output format | Version mismatch       | Ensure using latest BioMCP version                    |

### Authentication and Permissions

BioMCP currently does not require authentication for the tools it exposes. However:

1. **API Rate Limits**: The underlying APIs (PubMed, ClinicalTrials.gov, MyVariant.info) have rate limits that BioMCP respects
2. **Terms of Service**: Ensure your usage complies with the terms of service for each underlying API
3. **File System Access**: The process running BioMCP needs read/write access to its cache directory

### Performance Optimization

1. **Keep BioMCP Running**: Starting BioMCP for each request adds overhead; maintain a long-running process when possible
2. **Use Specific Queries**: Narrow searches perform better than broad ones
3. **Leverage Caching**: BioMCP caches results to improve performance; similar queries will be faster
4. **Batch Requests**: When possible, combine multiple small queries into fewer larger ones
5. **Monitor Resource Usage**: BioMCP's memory usage grows with the size and complexity of queries

## Example Conversations

### Example 1: Researching a Genetic Variant

**User**: "Can you tell me about the BRAF V600E mutation and its significance in melanoma?"

**AI Assistant**:
"I'll look up information about the BRAF V600E mutation and its significance in melanoma using BioMCP.

First, let me search for information about this variant."

_[AI uses BioMCP tool: variant_searcher]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "variant_searcher",
  "tool_input": {
    "gene": "BRAF",
    "protein": "p.V600E",
    "size": 5
  }
}
```

_[BioMCP response - truncated for brevity]_

```
# Record 1

Variant Id: rs113488022
Gene: BRAF
HGVS: NM_004333.4:c.1799T>A
Protein Change: p.Val600Glu
Significance: Pathogenic
...
```

"The BRAF V600E mutation (rs113488022) is a pathogenic variant in the BRAF gene. Let me now search for clinical trials related to this mutation in melanoma."

_[AI uses BioMCP tool: trial_searcher]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "trial_searcher",
  "tool_input": {
    "conditions": ["Melanoma"],
    "terms": ["BRAF V600E"],
    "recruiting_status": "OPEN"
  }
}
```

_[BioMCP response - truncated for brevity]_

```
# Record 1

Nct Number: NCT04123366
Study Title: Study of Dabrafenib + Trametinib in Patients With BRAF V600E Mutation-Positive Melanoma
Study Url: https://clinicaltrials.gov/study/NCT04123366
Study Status: RECRUITING
...
```

"Now, let me find recent research articles about this mutation in melanoma."

_[AI uses BioMCP tool: article_searcher]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "article_searcher",
  "tool_input": {
    "genes": ["BRAF"],
    "variants": ["V600E"],
    "diseases": ["Melanoma"]
  }
}
```

_[BioMCP response - truncated for brevity]_

```
# Record 1

Pmid: 36729348
Title: Targeting BRAF V600E in Melanoma: Beyond BRAF Inhibition
Journal: Int J Mol Sci
Authors: Tiago Bordeira Gaspar, Joana Figueiredo, Joana Paredes, Raquel Seruca
...
```

"Based on the information I've gathered using BioMCP, I can tell you that the BRAF V600E mutation is a pathogenic variant in the BRAF gene where valine is replaced by glutamic acid at position 600 of the protein. This mutation is particularly significant in melanoma because:

1. It's present in approximately 50% of melanoma cases
2. It leads to constitutive activation of the MAPK pathway, promoting cancer cell growth and survival
3. It's a targetable mutation with FDA-approved drugs like dabrafenib and vemurafenib
4. There are multiple active clinical trials specifically for melanoma patients with this mutation
5. Recent research continues to explore new therapeutic approaches beyond BRAF inhibition

Would you like me to elaborate on any specific aspect of this mutation or its treatment implications?"

### Example 2: Finding Clinical Trials for a Specific Condition

**User**: "I'm looking for information about clinical trials for glioblastoma that are currently recruiting patients."

**AI Assistant**:
"I'll help you find information about currently recruiting clinical trials for glioblastoma. Let me search for that information."

_[AI uses BioMCP tool: trial_searcher]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "trial_searcher",
  "tool_input": {
    "conditions": ["Glioblastoma"],
    "recruiting_status": "OPEN"
  }
}
```

_[BioMCP response - truncated for brevity]_

```
# Record 1

Nct Number: NCT04003649
Study Title: Trial of Pembrolizumab in Combination With Tetrahydrouridine and Decitabine in Patients With Recurrent Glioblastoma
Study Url: https://clinicaltrials.gov/study/NCT04003649
Study Status: RECRUITING
...
```

"I've found several recruiting clinical trials for glioblastoma. Let me get more details about one of these trials."

_[AI uses BioMCP tool: trial_protocol]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "trial_protocol",
  "tool_input": "NCT04003649"
}
```

_[BioMCP response - truncated for brevity]_

```
# Protocol: NCT04003649

## Brief Summary

This phase I trial studies the side effects and best dose of tetrahydrouridine and decitabine when given together with pembrolizumab in treating patients with glioblastoma that has come back (recurrent). Tetrahydrouridine and decitabine may stop the growth of tumor cells by blocking some of the enzymes needed for cell growth. Immunotherapy with monoclonal antibodies, such as pembrolizumab, may help the body's immune system attack the cancer, and may interfere with the ability of tumor cells to grow and spread...
```

"Let me also check where this trial is being conducted."

_[AI uses BioMCP tool: trial_locations]_

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "trial_locations",
  "tool_input": "NCT04003649"
}
```

_[BioMCP response - truncated for brevity]_

```
# Locations: NCT04003649

## United States

### California

- University of California San Diego, La Jolla, CA, 92093
  Status: Recruiting
  Contact: Huan Tran - 858-822-5354 - h3tran@health.ucsd.edu
...
```

"Based on my search using BioMCP, I've found several actively recruiting clinical trials for glioblastoma. One example is a Phase I trial (NCT04003649) studying the combination of pembrolizumab with tetrahydrouridine and decitabine for recurrent glioblastoma.

This trial is currently recruiting at multiple locations across the United States, including the University of California San Diego. The study is investigating whether this combination therapy can help the immune system attack glioblastoma while blocking enzymes needed for tumor growth.

Would you like me to:

1. Provide details about more glioblastoma trials?
2. Focus on trials with a specific treatment approach (like immunotherapy or targeted therapy)?
3. Look for trials in a specific location?
4. Search for information about the latest research on glioblastoma treatments?"

## Additional Resources

- [BioMCP GitHub Repository](https://github.com/genomoncology/biomcp)
- [Model Context Protocol Specification](https://github.com/anthropics/anthropic-cookbook/tree/main/model_context_protocol)
- [MCP Inspector Tool](https://github.com/anthropics/anthropic-cookbook/tree/main/model_context_protocol/inspector)
- [Claude Desktop Documentation](https://www.anthropic.com/claude)
