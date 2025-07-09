# BioMCP: Biomedical Model Context Protocol

BioMCP is an open source (MIT License) toolkit that empowers AI assistants and
agents with specialized biomedical knowledge. Built following the Model Context
Protocol (MCP), it connects AI systems to authoritative biomedical data
sources, enabling them to answer questions about clinical trials, scientific
literature, and genomic variants with precision and depth.

[![▶️ Watch the video](./docs/blog/images/what_is_biomcp_thumbnail.png)](https://www.youtube.com/watch?v=bKxOWrWUUhM)

## MCP Review Certification

BioMCP is certified by [MCP Review](https://mcpreview.com/mcp-servers/genomoncology/biomcp). This certification ensures that BioMCP follows best practices for Model Context Protocol implementation and provides reliable biomedical data access.

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

BioMCP integrates with multiple biomedical data sources:

### Literature Sources

- **PubTator3/PubMed** - Peer-reviewed biomedical literature with entity annotations
- **bioRxiv/medRxiv** - Preprint servers for biology and health sciences
- **Europe PMC** - Open science platform including preprints

### Clinical & Genomic Sources

- **ClinicalTrials.gov** - Clinical trial registry and results database
- **MyVariant.info** - Consolidated genetic variant annotation
- **TCGA/GDC** - The Cancer Genome Atlas for cancer variant data
- **1000 Genomes** - Population frequency data via Ensembl
- **cBioPortal** - Cancer genomics portal with mutation occurrence data

## Available MCP Tools

BioMCP provides 13 specialized tools for biomedical research:

### Core Tools (3)

#### 1. Think Tool (ALWAYS USE FIRST!)

**CRITICAL**: The `think` tool MUST be your first step for ANY biomedical research task.

```python
# Start analysis with sequential thinking
think(
    thought="Breaking down the query about BRAF mutations in melanoma...",
    thoughtNumber=1,
    totalThoughts=3,
    nextThoughtNeeded=True
)
```

The sequential thinking tool helps:

- Break down complex biomedical problems systematically
- Plan multi-step research approaches
- Track reasoning progress
- Ensure comprehensive analysis

#### 2. Search Tool

The search tool supports two modes:

##### Unified Query Language (Recommended)

Use the `query` parameter with structured field syntax for powerful cross-domain searches:

```python
# Simple natural language
search(query="BRAF melanoma")

# Field-specific search
search(query="gene:BRAF AND trials.condition:melanoma")

# Complex queries
search(query="gene:BRAF AND variants.significance:pathogenic AND articles.date:>2023")

# Get searchable fields schema
search(get_schema=True)

# Explain how a query is parsed
search(query="gene:BRAF", explain_query=True)
```

**Supported Fields:**

- **Cross-domain**: `gene:`, `variant:`, `disease:`
- **Trials**: `trials.condition:`, `trials.phase:`, `trials.status:`, `trials.intervention:`
- **Articles**: `articles.author:`, `articles.journal:`, `articles.date:`
- **Variants**: `variants.significance:`, `variants.rsid:`, `variants.frequency:`

##### Domain-Based Search

Use the `domain` parameter with specific filters:

```python
# Search articles (includes automatic cBioPortal integration)
search(domain="article", genes=["BRAF"], diseases=["melanoma"])

# Search with mutation-specific cBioPortal data
search(domain="article", genes=["BRAF"], keywords=["V600E"])
search(domain="article", genes=["SRSF2"], keywords=["F57*"])  # Wildcard patterns

# Search trials
search(domain="trial", conditions=["lung cancer"], phase="3")

# Search variants
search(domain="variant", gene="TP53", significance="pathogenic")
```

**Note**: When searching articles with a gene parameter, cBioPortal data is automatically included:

- Gene-level summaries show mutation frequency across cancer studies
- Mutation-specific searches (e.g., "V600E") show study-level occurrence data
- Cancer types are dynamically resolved from cBioPortal API

#### 3. Fetch Tool

Retrieve full details for a single article, trial, or variant:

```python
# Fetch article details (supports both PMID and DOI)
fetch(domain="article", id="34567890")  # PMID
fetch(domain="article", id="10.1101/2024.01.20.23288905")  # DOI

# Fetch trial with all sections
fetch(domain="trial", id="NCT04280705", detail="all")

# Fetch variant details
fetch(domain="variant", id="rs113488022")
```

**Domain-specific options:**

- **Articles**: `detail="full"` retrieves full text if available
- **Trials**: `detail` can be "protocol", "locations", "outcomes", "references", or "all"
- **Variants**: Always returns full details

### Individual Tools (10)

For users who prefer direct access to specific functionality, BioMCP also provides 10 individual tools:

#### Article Tools (2)

- **article_searcher**: Search PubMed/PubTator3 and preprints
- **article_getter**: Fetch detailed article information (supports PMID and DOI)

#### Trial Tools (5)

- **trial_searcher**: Search ClinicalTrials.gov
- **trial_getter**: Fetch all trial details
- **trial_protocol_getter**: Fetch protocol information only
- **trial_references_getter**: Fetch trial publications
- **trial_outcomes_getter**: Fetch outcome measures and results
- **trial_locations_getter**: Fetch site locations and contacts

#### Variant Tools (2)

- **variant_searcher**: Search MyVariant.info database
- **variant_getter**: Fetch comprehensive variant details

**Note**: All individual tools that search by gene automatically include cBioPortal summaries when the `include_cbioportal` parameter is True (default).

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

## Configuration

### Environment Variables

BioMCP supports optional environment variables for enhanced functionality:

```bash
# cBioPortal API authentication (optional)
export CBIO_TOKEN="your-api-token"  # For authenticated access
export CBIO_BASE_URL="https://www.cbioportal.org/api"  # Custom API endpoint

# Performance tuning
export BIOMCP_USE_CONNECTION_POOL="true"  # Enable HTTP connection pooling (default: true)
export BIOMCP_METRICS_ENABLED="false"     # Enable performance metrics (default: false)
```

Note: All APIs work without authentication, but tokens may provide higher rate limits.

## Command Line Interface

BioMCP provides a comprehensive CLI for direct database interaction:

```bash
# Get help
biomcp --help

# Run the MCP server
biomcp run

# Article search examples
biomcp article search --gene BRAF --disease Melanoma  # Includes preprints by default
biomcp article search --gene BRAF --no-preprints      # Exclude preprints
biomcp article get 21717063 --full

# Clinical trial examples
biomcp trial search --condition "Lung Cancer" --phase PHASE3
biomcp trial get NCT04280705 Protocol

# Variant examples with external annotations
biomcp variant search --gene TP53 --significance pathogenic
biomcp variant get rs113488022  # Includes TCGA, 1000 Genomes, and cBioPortal data by default
biomcp variant get rs113488022 --no-external  # Core annotations only
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

### Developer Guides

- [HTTP Client Guide](./docs/HTTP_CLIENT_GUIDE.md) - Using the centralized HTTP client
- [Migration Examples](./docs/MIGRATION_EXAMPLES.md) - Migrating from direct HTTP usage
- [Error Handling Guide](./docs/ERROR_HANDLING.md) - Comprehensive error handling patterns
- [Integration Testing Guide](./docs/INTEGRATION_TESTING.md) - Best practices for reliable integration tests
- [Third-Party Endpoints](./THIRD_PARTY_ENDPOINTS.md) - Complete list of external APIs used

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
