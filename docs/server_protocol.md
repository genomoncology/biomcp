# BioMCP Server & The Model Context Protocol (MCP)

This document describes the server component of BioMCP, activated via the `biomcp run` command. It explains why BioMCP utilizes the Model Context Protocol (MCP) and how it enables advanced interaction between AI systems and specialized biomedical data sources.

## Why MCP for BioMCP? The Need for Biomedical Context

Modern AI models, including Large Language Models (LLMs), possess vast general knowledge but often lack the specific, up-to-date, and nuanced context required for complex domains like biomedicine. MCP provides a standardized solution to this "context gap."

BioMCP leverages MCP to specifically address these challenges in the biomedical field:

1.  **Bridging AI and Specialized Knowledge:** MCP provides the standard communication pathway, while BioMCP acts as the specialized "translator" on that pathway. It allows AI assistants to interact naturally with complex resources like PubMed, ClinicalTrials.gov, and MyVariant.info without needing deep knowledge of their individual APIs or query languages (BioMCP Point 1).
2.  **Standardizing Access to Fragmented Data:** Before MCP, connecting an AI to each biomedical database required custom integration code (an "N x M" problem). BioMCP acts as a unified MCP server for these key resources, simplifying integration into a standard protocol (MCP Point 2, BioMCP Point 3).
3.  **Enabling Context-Rich Biomedical AI:** By providing structured access to live data, BioMCP helps overcome the "knowledge cutoff" problem inherent in pre-trained models. AI assistants using BioMCP can access the latest research, trial statuses, and variant annotations (BioMCP Points 6, 12).
4.  **Foundation for Capable Biomedical Agents:** MCP provides the building blocks for AI agents. BioMCP provides the _specific biomedical tools_ within that framework, allowing agents to perform multi-step research tasks, compare trial data, analyze variant significance, and more, transforming them from chatbots into digital biomedical collaborators (MCP Points 6, 10).

## How BioMCP Implements MCP

BioMCP primarily functions as an **MCP Tool Server**. When you run `biomcp run`, it starts a server process that listens for requests according to the MCP standard.

**Key Implementation Details:**

- **Communication:** The server uses **Standard Input/Output (STDIO)** for communication. It expects JSON-formatted MCP requests on `stdin` and sends JSON-formatted MCP responses to `stdout`. This is ideal for integration as a subprocess managed by an LLM agent framework or other applications.
- **Tool Exposure:** BioMCP exposes its core functionalities (searching trials, articles, variants; fetching details) as distinct **MCP Tools**. An AI client can request a specific tool by name and provide structured input.
- **Intelligent Processing within MCP:** BioMCP doesn't just pass raw data. It adds value within the MCP interaction by:
  - **Entity Normalization:** Mapping natural language terms (e.g., "lung cancer", "her2") to standardized biomedical identifiers before querying backend databases (BioMCP Point 2).
  - **Intelligent Rendering:** Transforming complex, nested JSON responses from APIs into human-readable, consistently formatted Markdown suitable for display in chat interfaces (BioMCP Point 13).
  - **Transparent Attribution:** Ensuring results include source identifiers (PMIDs, NCT IDs) and URLs, allowing verification and deeper exploration (BioMCP Point 11).
  - **Performance Caching:** Utilizing caching to reduce redundant API calls, improving speed and respecting external API rate limits (BioMCP Point 14).

## Running the BioMCP Server

Start the server using the command:

```bash
biomcp run
```

The server will initialize and wait for MCP requests on its standard input. It's designed to be managed by a parent process that handles piping data. Terminate the server using Ctrl+C or by closing its input stream.

## Available BioMCP Tools (via MCP)

The following tools are exposed by the `biomcp run` server. Inputs are expected as JSON objects matching the specified Pydantic models, and outputs are typically Markdown strings within the MCP JSON response structure.

(Refer to the respective CLI documentation or source code (src/biomcp/[articles|trials|variants]/\*.py and their Pydantic models) for detailed input structures.)

### Article Tools

- `article_searcher`: Searches PubMed/PubTator3. (Input: PubmedRequest)
- `article_details`: Fetches details for a PMID. (Input: pmid)

### Trial Tools

- `trial_searcher`: Searches ClinicalTrials.gov. (Input: TrialQuery)
- `trial_protocol`: Fetches protocol details for an NCT ID. (Input: nct_id)
- `trial_locations`: Fetches location details for an NCT ID. (Input: nct_id)
- `trial_outcomes`: Fetches outcome/results details for an NCT ID. (Input: nct_id)
- `trial_references`: Fetches publication details for an NCT ID. (Input: nct_id)

### Variant Tools

- `variant_searcher`: Searches MyVariant.info. (Input: VariantQuery)
- `variant_details`: Fetches detailed annotation for a variant ID. (Input: variant_id)

## Interaction Example (Conceptual via STDIO)

Client Sends Request (to server's stdin):

```json
{
  "mcp_protocol_version": "1.0",
  "tool_name": "trial_searcher",
  "tool_input": {
    "conditions": ["Melanoma"],
    "recruiting_status": "OPEN",
    "phase": "PHASE2"
  }
}
```

Server Sends Response (on server's stdout):

```json
{
  "result": "# Record 1\n\nNct Number: NCT0xxxxxxx\nStudy Title: A Phase 2 Study of...\nStudy Url: https://clinicaltrials.gov/study/NCT0xxxxxxx\nStudy Status: RECRUITING\n...",
  "error": null
}
```

## Relation to CLI

The BioMCP server and CLI share the same underlying logic for interacting with biomedical APIs.

- **CLI:** Optimized for direct human interaction via terminal commands and flags. Output is printed directly.
- **Server:** Optimized for programmatic interaction using the MCP standard over STDIO. Facilitates integration with AI agent frameworks.

The server provides a way for automated systems to access the same powerful biomedical data integration capabilities available through the BioMCP CLI.

## Integration with MCP Clients

For detailed instructions on integrating BioMCP with various MCP clients (such as Claude Desktop, Anthropic API, and custom applications), please refer to the [MCP Integration Guide](mcp_integration.md). The integration guide provides:

- Step-by-step configuration instructions for Claude Desktop
- Code examples for JavaScript and Python MCP clients
- Troubleshooting tips for common integration issues
- Example conversations demonstrating BioMCP in action
