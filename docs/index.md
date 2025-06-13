# BioMCP: Biomedical Model Context Protocol Server

[![Release](https://img.shields.io/github/v/tag/genomoncology/biomcp)](https://github.com/genomoncology/biomcp/tags)
[![Build status](https://img.shields.io/github/actions/workflow/status/genomoncology/biomcp/main.yml?branch=main)](https://github.com/genomoncology/biomcp/actions/workflows/main.yml?query=branch%3Amain)
[![Commit activity](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)
[![License](https://img.shields.io/github/license/genomoncology/biomcp)](https://img.shields.io/github/license/genomoncology/biomcp)

BioMCP is a specialized Model Context Protocol (MCP) server that connects AI assistants like Claude to biomedical data sources, including ClinicalTrials.gov, PubMed, and MyVariant.info.

### Built and Maintained by <a href="https://www.genomoncology.com"><img src="./assets/logo.png" width=200 valign="middle" /></a>

## Quick Start: Claude Desktop Setup

The fastest way to get started with BioMCP is to set it up with Claude Desktop:

1. **Install Claude Desktop** from [Anthropic](https://claude.ai/desktop)

2. **Ensure `uv` is installed**:

   ```bash
   # Install uv if you don't have it
   # MacOS: brew install uv
   # Windows: pip install uv
   ```

3. **Configure Claude Desktop**:

   - Open Claude Desktop settings
   - Navigate to Developer section
   - Click "Edit Config" and add:

   ```json
   {
     "mcpServers": {
       "biomcp": {
         "command": "uv",
         "args": ["run", "--with", "biomcp-python", "biomcp", "run"]
       }
     }
   }
   ```

   - Save and restart Claude Desktop

4. **Start chatting with Claude** about biomedical topics!

For detailed setup instructions and examples, see our [Claude Desktop Tutorial](tutorials/claude-desktop.md).

## What is BioMCP?

BioMCP is a specialized MCP (Model Context Protocol) server that bridges the gap between AI systems and critical biomedical data sources. While Large Language Models (LLMs) like Claude have extensive general knowledge, they often lack real-time access to specialized databases needed for in-depth biomedical research.

Using the Model Context Protocol, BioMCP provides Claude and other AI assistants with structured, real-time access to:

1. **Clinical Trials** - Searchable access to ClinicalTrials.gov for finding relevant studies
2. **Research Literature** - Query PubMed/PubTator3 for the latest biomedical research
3. **Genomic Variants** - Explore detailed genetic variant information from MyVariant.info

Through MCP, AI assistants can seamlessly invoke BioMCP tools during conversations, retrieving precise biomedical information without the user needing to understand complex query syntax or database-specific parameters.

## MCP Tools and Capabilities

BioMCP exposes the following tools through the MCP interface:

### Clinical Trial Tools

- `trial_searcher`: Search for trials by condition, intervention, location, phase, etc.
- `trial_protocol`: Get detailed protocol information for specific trials
- `trial_locations`: Find where trials are conducted
- `trial_outcomes`: Access trial results and outcome data
- `trial_references`: Find publications related to specific trials

### Literature Tools

- `article_searcher`: Find biomedical articles across multiple dimensions
- `article_details`: Retrieve detailed article content and metadata

### Genomic Tools

- `variant_searcher`: Search for genetic variants with filtering options
- `variant_details`: Get comprehensive annotations for specific variants

### Sequential Thinking Tool

- `sequential_thinking`: A problem-solving tool for dynamic and reflective thinking, helping analyze complex biomedical problems through a flexible, adaptive process

## MCP Resources

BioMCP provides the following resources through the MCP interface:

### Instructions Resource

- `get_instructions`: Returns operational instructions and guidelines for effective use of BioMCP tools. This resource helps AI assistants understand best practices for biomedical research tasks.

### Researcher Persona Resource

- `get_researcher`: Provides a detailed biomedical researcher persona with extensive expertise across multiple domains. This persona helps AI assistants adopt appropriate research methodologies and communication styles for biomedical tasks.

## Tutorials

- [**Claude Desktop Tutorial**](tutorials/claude-desktop.md) - Set up and use BioMCP with Claude Desktop
- [**MCP Inspector Tutorial**](tutorials/mcp-inspector.md) - Test and debug BioMCP directly
- [**Python SDK Tutorial**](tutorials/python-sdk.md) - Use BioMCP as a Python library
- [**MCP Client Tutorial**](tutorials/mcp-client.md) - Integrate with MCP clients programmatically

## Verification and Testing

The easiest way to test your BioMCP setup is with the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

This launches a web interface where you can test each BioMCP tool directly. For detailed instructions, see the [MCP Inspector Tutorial](tutorials/mcp-inspector.md).

## Additional Usage Options

While BioMCP is primarily designed as an MCP server for AI assistants, it can also be used in other ways:

### Command Line Interface

BioMCP includes a comprehensive CLI for direct interaction with biomedical databases:

```bash
# Examples of CLI usage
biomcp trial search --condition "Melanoma" --phase PHASE3
biomcp article search --gene BRAF --disease Melanoma
biomcp variant search --gene TP53 --significance pathogenic
```

### Python SDK

For programmatic access, BioMCP can be used as a Python library:

```bash
# Install the package
pip install biomcp-python
```

See the [Python SDK Tutorial](tutorials/python-sdk.md) for code examples.

### MCP Client Integration

For developers building MCP-compatible applications, BioMCP can be integrated using the MCP client libraries. See the [MCP Client Tutorial](tutorials/mcp-client.md) for details.

## License

BioMCP is licensed under the MIT License.
