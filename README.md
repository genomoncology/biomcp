# BioMCP: Biomedical Model Context Protocol

[![smithery badge](https://smithery.ai/badge/@genomoncology/biomcp)](https://smithery.ai/server/@genomoncology/biomcp)

BioMCP is an open source (MIT License) toolkit for biomedical research AI
assistants and agents. Built following the Model Context Protocol (MCP),
it supports searching and retrieving clinical trials, pubmed articles, and
genomic variants.

[![▶️ Watch the video](./docs/blog/images/what_is_biomcp_thumbnail.png)](https://www.youtube.com/watch?v=bKxOWrWUUhM)

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

**NOTE**: BioMCP is installable via the python package name `biomcp-python`.

### Quick Start Options

#### For Claude Desktop Users

The easiest way to install BioMCP for Claude Desktop is via [Smithery](https://smithery.ai/server/@genomoncology/biomcp):

```bash
npx -y @smithery/cli install @genomoncology/biomcp --client claude
```

This automatically configures the BioMCP MCP server for use with Claude Desktop.

#### For Python/CLI Users

Install the BioMCP package using pip or uv:

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

To verify your BioMCP MCP Server installation, use the MCP Inspector: Server, using the MCP Inspector, run the following command:

```bash
npx @modelcontextprotocol/inspector uv run biomcp run
```

## Command Line Interface

BioMCP provides a comprehensive CLI for direct interaction with biomedical
databases. Note, the package name is `biomcp-python`, not `biomcp`.

```bash
# Install the package as tool using uv (note: package name is `biomcp-python`)
uv tool install biomcp-python

# Get help (note CLI name is `biomcp`)
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

BioMCP integrates data from several authoritative biomedical databases, each with their own terms of service and data usage policies. Users should be aware of these policies when utilizing BioMCP for various applications.

### PubTator3 / PubMed

- **Provider**: National Center for Biotechnology Information (NCBI), National Library of Medicine (NLM)
- **Content**: Biomedical literature indexed in PubMed and PubMed Central
- **Terms of Service**: [NCBI/NLM Terms and Conditions](https://www.ncbi.nlm.nih.gov/home/about/policies/)
- **Access Limits**: Standard access is limited to a specific number of requests per second. Higher throughput available with an API key.
- **Citation Requirements**: Publications using data from PubMed should cite the appropriate NCBI/NLM resources.
- **Underlying Sources**: Articles from thousands of biomedical journals and repositories.

### ClinicalTrials.gov

- **Provider**: National Library of Medicine (NLM)
- **Content**: Clinical trials registry and results database
- **Terms of Service**: [ClinicalTrials.gov Terms and Conditions](https://clinicaltrials.gov/ct2/about-site/terms-conditions)
- **Data Usage**: Data is freely available for research and academic purposes. Commercial use may have additional restrictions.
- **API Access**: [ClinicalTrials.gov API](https://clinicaltrials.gov/data-api/about-api)
- **Citation Requirements**: Any publication using data from ClinicalTrials.gov should properly acknowledge the source.

### MyVariant.info

- **Provider**: The Su Lab at Scripps Research
- **Content**: Comprehensive genetic variant annotation compiled from multiple databases
- **Terms of Service**: [MyVariant.info Terms of Service](https://myvariant.info/terms)
- **Annotation Sources**: [Comprehensive list of annotation sources](https://docs.myvariant.info/en/latest/doc/data.html)
- **Attribution**: Users should properly attribute both MyVariant.info and the original annotation sources in publications.
- **Primary Data Sources Include**:
  - **ClinVar**: Clinical interpretations of genetic variants
  - **COSMIC**: Catalogue of Somatic Mutations in Cancer
  - **dbSNP**: Database of single nucleotide polymorphisms
  - **CIViC**: Clinical Interpretations of Variants in Cancer
  - **gnomAD**: Genome Aggregation Database
  - **CADD**: Combined Annotation Dependent Depletion scores

### Important Usage Notes

When using BioMCP, please be aware that:

1. **Attribution Requirements**: Publications or applications using data obtained through BioMCP should cite both BioMCP and the original data sources appropriately.
2. **Data Currency**: Information retrieved may not represent the most current data available from the original sources due to update frequencies.
3. **Usage Restrictions**: While BioMCP provides access to these resources, users must comply with the individual terms of service for each data source, particularly for commercial applications.
4. **Protected Health Information**: BioMCP does not handle or provide access to protected health information (PHI) or personally identifiable information (PII).

For additional information about specific data sources or usage policies, please refer to the documentation provided by each source database.
