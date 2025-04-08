# biomcp

[![Release](https://img.shields.io/github/v/release/genomoncology/biomcp)](https://img.shields.io/github/v/release/genomoncology/biomcp)
[![Build status](https://img.shields.io/github/actions/workflow/status/genomoncology/biomcp/main.yml?branch=main)](https://github.com/genomoncology/biomcp/actions/workflows/main.yml?query=branch%3Amain)
[![Commit activity](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)](https://img.shields.io/github/commit-activity/m/genomoncology/biomcp)
[![License](https://img.shields.io/github/license/genomoncology/biomcp)](https://img.shields.io/github/license/genomoncology/biomcp)

**BioMCP: Biomedical Model Context Protocol Server**

BioMCP provides a unified command-line interface (CLI) and server protocol to simplify access to key biomedical data sources, including ClinicalTrials.gov, PubMed (via PubTator3), and MyVariant.info.

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

Install BioMCP using pip:

```bash
pip install biomcp-python
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
- [Contributing Guide](contributing.md)
- [Changelog](changelog.md)

## License

BioMCP is licensed under the MIT License.
