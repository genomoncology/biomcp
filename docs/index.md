# BioMCP: Biomedical Model Context Protocol Server

[![Release](https://img.shields.io/github/v/tag/genomoncology/biomcp)](https://github.com/genomoncology/biomcp/tags)
[![Build status](https://img.shields.io/github/actions/workflow/status/genomoncology/biomcp/main.yml?branch=main)](https://github.com/genomoncology/biomcp/actions/workflows/main.yml?query=branch%3Amain)
[![Commit activity](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)
[![License](https://img.shields.io/github/license/genomoncology/biomcp)](https://img.shields.io/github/license/genomoncology/biomcp)

BioMCP is a specialized Model Context Protocol (MCP) server that connects AI assistants like Claude to biomedical data sources, including ClinicalTrials.gov, PubMed, MyVariant.info, and cBioPortal.

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
2. **Research Literature** - Query PubMed/PubTator3 for the latest biomedical research with automatic cBioPortal cancer genomics integration
3. **Genomic Variants** - Explore detailed genetic variant information from MyVariant.info
4. **Cancer Genomics** - Automatic integration with cBioPortal for mutation occurrence data in cancer studies

Through MCP, AI assistants can seamlessly invoke BioMCP tools during conversations, retrieving precise biomedical information without the user needing to understand complex query syntax or database-specific parameters.

## MCP Tools and Capabilities

BioMCP provides 13 specialized tools through the MCP interface:

### Core Tools (3)

#### Think Tool (CRITICAL - ALWAYS USE FIRST!)

- `think`: A sequential thinking tool for systematic analysis of biomedical problems. **MUST be used BEFORE any search operations** to ensure comprehensive research planning and analysis.

#### Unified Tools

- `search`: Powerful unified search across all biomedical data sources with cross-domain query support
- `fetch`: Retrieve detailed information for specific articles, trials, or variants

### Individual Tools (10)

#### Article Tools (2)

- `article_searcher`: Search PubMed/PubTator3 and preprints with automatic cBioPortal integration
- `article_getter`: Fetch detailed article content and metadata

**Note**: When searching articles with gene parameters, cBioPortal data is automatically included, providing:

- Gene-level mutation summaries across cancer studies
- Mutation-specific search capabilities (e.g., BRAF V600E)
- Dynamic cancer type categorization

#### Clinical Trial Tools (5)

- `trial_searcher`: Search for trials by condition, intervention, location, phase, etc.
- `trial_getter`: Fetch all details for a specific trial
- `trial_protocol_getter`: Get protocol information only
- `trial_references_getter`: Find publications related to trials
- `trial_outcomes_getter`: Access trial results and outcome data
- `trial_locations_getter`: Find where trials are conducted

#### Genomic Variant Tools (2)

- `variant_searcher`: Search MyVariant.info with clinical and functional filters
- `variant_getter`: Get comprehensive annotations including TCGA, 1000 Genomes, and cBioPortal data

## MCP Resources

BioMCP provides the following resources through the MCP interface:

### Instructions Resource

- `get_instructions`: Returns operational instructions and guidelines for effective use of BioMCP tools. This resource helps AI assistants understand best practices for biomedical research tasks.

### Researcher Persona Resource

- `get_researcher`: Provides a detailed biomedical researcher persona with extensive expertise across multiple domains. This persona helps AI assistants adopt appropriate research methodologies and communication styles for biomedical tasks.

## Tutorials

### Getting Started

- [**Claude Desktop Tutorial**](tutorials/claude-desktop.md) - Set up and use BioMCP with Claude Desktop
- [**MCP Inspector Tutorial**](tutorials/mcp-inspector.md) - Test and debug BioMCP directly
- [**Python SDK Tutorial**](tutorials/python-sdk.md) - Use BioMCP as a Python library
- [**MCP Client Tutorial**](tutorials/mcp-client.md) - Integrate with MCP clients programmatically

### Advanced Features

- [**AlphaGenome Setup Guide**](tutorials/alphagenome-setup.md) - Configure Google DeepMind's AlphaGenome for variant effect prediction
- [**AlphaGenome Prompt Examples**](tutorials/alphagenome-prompts.md) - Example prompts and workflows for variant analysis with AI
- [**AlphaGenome with Docker**](tutorials/docker-alphagenome.md) - Run AlphaGenome in Docker containers

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
biomcp article search --gene BRAF --disease Melanoma  # Includes cBioPortal data
biomcp article search --gene BRAF --keyword V600E     # Mutation-specific search
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
