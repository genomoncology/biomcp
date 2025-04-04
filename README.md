# BioMCP: Biomedical Model Context Protocol
[![smithery badge](https://smithery.ai/badge/@genomoncology/biomcp)](https://smithery.ai/server/@genomoncology/biomcp)

BioMCP provides LLMs with structured access to critical biomedical databases
through the Model Context Protocol (MCP).

## Tools

- PubTator3 (PubMed/PMC)
  - Article Search
  - Full Text
- ClinicalTrials.gov
  - Clinical Trial Advanced Search
  - Protocols
  - Outcomes
  - Locations
  - Reference
- MyVariant.info
  - Variant Search
  - Annotations (CIViC, ClinVar, COSMIC, dbSNP, etc.)

## Installation

### Installing via Smithery

To install biomcp for Claude Desktop automatically via [Smithery](https://smithery.ai/server/@genomoncology/biomcp):

```bash
npx -y @smithery/cli install @genomoncology/biomcp --client claude
```

To install the BioMCP package to use it as a Python package or Command Line
Interface (CLI), run the following command:

```bash
pip install biomcp-python
```

Or preferably:

```bash
uv pip install biomcp-python
```

Or just run it:

```bash
uvx --from biomcp-python biomcp trial search --condition "lung cancer" --intervention "pembro"
```

To install it as a MCP Server to Claude Desktop or similar "MCP Clients", try
something like:

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

Note: If you get a `SPAWN ENOENT` warning, make sure your `uv` executable
is in your PATH or provide a full path to it (e.g. /Users/name/.local/bin/uv).

For comprehensive instructions on integrating BioMCP with Claude Desktop and other MCP clients, see the [MCP Integration Guide](docs/mcp_integration.md).

To test the MCP Server, using the MCP Inspector, run the following command:

```bash
npx @modelcontextprotocol/inspector uv run biomcp run
```

## Command Line Interface

BioMCP provides a comprehensive CLI for direct interaction with biomedical
databases:

```bash
# Get help
biomcp --help

# Run the MCP server
biomcp run

# Search for articles
biomcp article search --gene BRAF --disease Melanoma

# Get article details by PMID
biomcp article get 21717063 --full

# Search for clinical trials
biomcp trial search --condition "Lung Cancer" --phase Phase_3

# Get trial details by NCT ID
biomcp trial get NCT04280705 Protocol

# Search for variants
biomcp variant search --gene BRAF --significance Pathogenic

# Get variant details
biomcp variant get rs113488022
```

## Commercial Version: OncoMCP

OncoMCP extends BioMCP with GenomOncology's enterprise-grade precision oncology
platform (POP), providing healthcare organizations with:

- On-premise deployment with full HIPAA compliance
- Real-time clinical trial recruiting status and arm-level matching
- Seamless EHR and clinical data warehouse integration
- Curated knowledge base of 15,000+ clinical trials and FDA approvals
- Patient-trial matching using integrated clinical and molecular profiles
- Advanced NLP for structured data extraction and normalization
- Comprehensive biomarker and mutation rule processing

Find out more about GenomOncology and OncoMCP by visiting:
[GenomOncology](https://genomoncology.com/).

## License

This project is licensed under the MIT License.


## Sources

Below are the sources leveraged in BioMCP and their individual Terms of
Service. Please recognize that each of these services may aggregate 
from multiple underlying sources that may have their own Terms of Service.


- MyVariant.info
    * [Terms of Service](https://myvariant.info/terms)
    * [Annotation Source](https://docs.myvariant.info/en/latest/doc/data.html)


