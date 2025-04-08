# BioMCP: Biomedical Model Context Protocol Server

[![Release](https://img.shields.io/github/v/tag/genomoncology/biomcp)](https://github.com/genomoncology/biomcp/tags)
[![Build status](https://img.shields.io/github/actions/workflow/status/genomoncology/biomcp/main.yml?branch=main)](https://github.com/genomoncology/biomcp/actions/workflows/main.yml?query=branch%3Amain)
[![Commit activity](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)
[![License](https://img.shields.io/github/license/genomoncology/biomcp)](https://img.shields.io/github/license/genomoncology/biomcp)

BioMCP provides a unified command-line interface (CLI) and server protocol to simplify access to key biomedical data sources, including ClinicalTrials.gov, PubMed (via PubTator3), and MyVariant.info.

### Built and Maintained by <a href="https://www.genomoncology.com"><img src="./assets/logo.png" width=200 valign="middle" /></a>

## What is BioMCP?

Navigating the landscape of biomedical data often requires interacting with multiple distinct APIs, each with its own query syntax, parameters, and data formats. BioMCP solves this by offering:

- A **consistent Command-Line Interface (CLI)** for searching and retrieving information about clinical trials, biomedical literature, and genetic variants.
- An underlying **abstraction layer** that handles communication with the respective external APIs (ClinicalTrials.gov, PubTator3, MyVariant.info).
- A **server component** implementing the Biomedical Model Context Protocol, designed for programmatic interaction, potentially with systems like Large Language Models (LLMs).

Whether you need to quickly look up a gene variant, find relevant clinical trials, explore recent research articles, or integrate this data programmatically, BioMCP aims to streamline the process.

## Target Audience

- **Bioinformaticians & Researchers:** Quickly query biomedical databases from the command line.
- **Developers:** Integrate biomedical data into applications or workflows using the CLI or potentially the server protocol.
- **Data Scientists:** Aggregate data from multiple sources for analysis.

## Core Features

- **Trials CLI (`biomcp trial ...`):** Search ClinicalTrials.gov for studies based on conditions, interventions, status, location, and more. Retrieve detailed trial information.
- **Articles CLI (`biomcp article ...`):** Search PubMed/PubTator3 for articles using keywords, genes, diseases, chemicals, or variants. Fetch article abstracts and metadata.
- **Variants CLI (`biomcp variant ...`):** Search MyVariant.info for genetic variants by gene, protein change, rsID, or genomic location. Filter by clinical significance, population frequency, and functional predictions. Retrieve detailed variant annotations.
- **Workflow Integration:** Combine commands to perform common research tasks (see [Common Workflows](workflows.md)).
- **Server Protocol:** Run `biomcp run` to start a server for programmatic interaction (details in [Server Protocol](server_protocol.md)).

## Installation

**NOTE**: BioMCP is installable via the python package name `biomcp-python`.

### Quick Start Options

#### For Claude Desktop Users

The easiest way to install BioMCP for Claude Desktop is via [Smithery](https://smithery.ai/server/@genomoncology/biomcp):

```bash
npx -y @smithery/cli install @genomoncology/biomcp --client claude
```

This automatically configures the BioMCP MCP server for use with Claude Desktop.

#### For Python/CLI Users

Install the BioMCP package using pip:

```bash
pip install biomcp-python
```

Or preferably using uv for faster installation:

```bash
uv pip install biomcp-python
```

You can also run BioMCP commands directly without installation:

```bash
uvx --from biomcp-python biomcp trial search --condition "lung cancer" --intervention "pembro"
```

### Advanced Installation

#### Manual Claude Desktop Integration

To manually configure BioMCP as an MCP Server for Claude Desktop:

1. Open Claude Desktop settings
2. Navigate to the MCP Servers configuration section
3. Add the following configuration:

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

**Note:** If you get a `SPAWN ENOENT` warning, make sure your `uv` executable
is in your PATH or provide a full path to it (e.g. /Users/name/.local/bin/uv).

#### Verification

To verify your BioMCP MCP Server installation, use the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector uv run biomcp run
```

For more detailed instructions, see the [Installation Guide](installation.md).

## Getting Started

Once installed, try a simple command:

```bash
# Find information about the BRAF V600E variant
biomcp variant search --gene BRAF --protein p.V600E
```

This will output information about the variant in a human-readable Markdown format.

Explore more examples in the [Getting Started Guide](getting_started.md).

## Documentation Overview

- [Installation Guide](installation.md)
- [Getting Started Guide](getting_started.md)
- [Common Workflows](workflows.md)
- CLI Reference:
  - [Trials CLI](cli/trials.md)
  - [Articles CLI](cli/articles.md)
  - [Variants CLI](cli/variants.md)
- API Reference (Underlying APIs):
  - [ClinicalTrials.gov API](apis/clinicaltrials_gov.md)
  - [PubTator3 API](apis/pubtator3_api.md)
  - [MyVariant.info API](apis/myvariant_info.md)
- [Server Protocol Guide](server_protocol.md)
- [MCP Integration Guide](mcp_integration.md)
- [About GenomOncology](genomoncology.md)
- [Contributing Guide](contributing.md)
- [Changelog](changelog.md)

## License

BioMCP is licensed under the MIT License.
