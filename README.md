# BioMCP: Biomedical Model Context Protocol

BioMCP is an open source (MIT License) toolkit that empowers AI assistants and
agents with specialized biomedical knowledge. Built following the Model Context
Protocol (MCP), it connects AI systems to authoritative biomedical data
sources, enabling them to answer questions about clinical trials, scientific
literature, and genomic variants with precision and depth.

[![▶️ Watch the video](./docs/blog/images/what_is_biomcp_thumbnail.png)](https://www.youtube.com/watch?v=bKxOWrWUUhM)

## Why BioMCP?

While Large Language Models have broad general knowledge, they often lack
specialized domain-specific information or access to up-to-date resources.
BioMCP bridges this gap for biomedicine by:

- Providing **structured access** to clinical trials, biomedical literature,
  and genomic variants
- Enabling **natural language queries** to specialized databases without
  requiring knowledge of their specific syntax
- Supporting **biomedical research** workflows through a consistent interface
- Functioning as an **MCP server** for AI assistants and agents

## Biomedical Data Sources

BioMCP integrates with three key biomedical data sources:

- **PubTator3/PubMed** - Biomedical literature with entity annotations
- **ClinicalTrials.gov** - Clinical trial registry and results database
- **MyVariant.info** - Consolidated genetic variant annotation from multiple
  databases

## Available MCP Tools

### PubMed & PubTator3

- `article_searcher`: Search for articles by genes, diseases, variants, or
  keywords
- `article_details`: Get detailed article information including abstracts and
  full text

### ClinicalTrials.gov

- `trial_searcher`: Advanced trial search with filtering by condition,
  intervention, phase, etc.
- `trial_protocol`: Detailed trial protocol information
- `trial_locations`: Trial site locations and contact information
- `trial_outcomes`: Results and outcome measures
- `trial_references`: Related publications

### MyVariant.info

- `variant_searcher`: Search for genetic variants with sophisticated filtering
- `variant_details`: Comprehensive annotations from multiple sources (CIViC,
  ClinVar, COSMIC, dbSNP, etc.)

## Quick Start

### For Claude Desktop Users

1. **Install `uv`** if you don't have it (recommended):

   ```bash
   # MacOS
   brew install uv

   # Windows/Linux
   pip install uv
   ```

2. **Configure Claude Desktop**:
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
   - Restart Claude Desktop and start chatting about biomedical topics!

### Python Package Installation

```bash
# Using pip
pip install biomcp-python

# Using uv (recommended for faster installation)
uv pip install biomcp-python

# Run directly without installation
uv run --with biomcp-python biomcp trial search --condition "lung cancer"
```

## Command Line Interface

BioMCP provides a comprehensive CLI for direct database interaction:

```bash
# Get help
biomcp --help

# Run the MCP server
biomcp run

# Examples
biomcp article search --gene BRAF --disease Melanoma
biomcp article get 21717063 --full
biomcp trial search --condition "Lung Cancer" --phase PHASE3
biomcp trial get NCT04280705 Protocol
biomcp variant search --gene TP53 --significance pathogenic
biomcp variant get rs113488022
```

## Testing & Verification

Test your BioMCP setup with the MCP Inspector:

```bash
npx @modelcontextprotocol/inspector uv run --with biomcp-python biomcp run
```

This opens a web interface where you can explore and test all available tools.

## Enterprise Version: OncoMCP

OncoMCP extends BioMCP with GenomOncology's enterprise-grade precision oncology
platform (POP), providing:

- **HIPAA-Compliant Deployment**: Secure on-premise options
- **Real-Time Trial Matching**: Up-to-date status and arm-level matching
- **Healthcare Integration**: Seamless EHR and data warehouse connectivity
- **Curated Knowledge Base**: 15,000+ trials and FDA approvals
- **Sophisticated Patient Matching**: Using integrated clinical and molecular
  profiles
- **Advanced NLP**: Structured extraction from unstructured text
- **Comprehensive Biomarker Processing**: Mutation and rule processing

Learn more: [GenomOncology](https://genomoncology.com/)

## MCP Registries

[![smithery badge](https://smithery.ai/badge/@genomoncology/biomcp)](https://smithery.ai/server/@genomoncology/biomcp)

<a href="https://glama.ai/mcp/servers/@genomoncology/biomcp">
<img width="380" height="200" src="https://glama.ai/mcp/servers/@genomoncology/biomcp/badge" />
</a>

## Documentation

For comprehensive documentation, visit [https://biomcp.org](https://biomcp.org)

## BioMCP Examples Repo

Looking to see BioMCP in action?

Check out the companion repository:
👉 **[biomcp-examples](https://github.com/genomoncology/biomcp-examples)**

It contains real prompts, AI-generated research briefs, and evaluation runs across different models.
Use it to explore capabilities, compare outputs, or benchmark your own setup.

Have a cool example of your own?
**We’d love for you to contribute!** Just fork the repo and submit a PR with your experiment.

## License

This project is licensed under the MIT License.
