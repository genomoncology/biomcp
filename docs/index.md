# BioMCP: AI-Powered Biomedical Research

[![Release](https://img.shields.io/github/v/tag/genomoncology/biomcp)](https://github.com/genomoncology/biomcp/tags)
[![Build status](https://img.shields.io/github/actions/workflow/status/genomoncology/biomcp/main.yml?branch=main)](https://github.com/genomoncology/biomcp/actions/workflows/main.yml?query=branch%3Amain)
[![License](https://img.shields.io/github/license/genomoncology/biomcp)](https://img.shields.io/github/license/genomoncology/biomcp)

**Transform how you search and analyze biomedical data** with BioMCP - a powerful tool that connects AI assistants and researchers to critical biomedical databases through natural language.

### Built and Maintained by <a href="https://www.genomoncology.com"><img src="./assets/logo.png" width=200 valign="middle" /></a>

<div class="announcement-banner">
  <div class="announcement-content">
    <h2>
      <span class="badge-new">NEW</span>
      Remote BioMCP Now Available!
    </h2>
    <p>Connect to BioMCP instantly through Claude - no installation required!</p>

    <div class="announcement-features">
      <div class="feature-item">
        <strong>🚀 Instant Access</strong>
        <span>Start using BioMCP in under 2 minutes</span>
      </div>
      <div class="feature-item">
        <strong>☁️ Cloud-Powered</strong>
        <span>Always up-to-date with latest features</span>
      </div>
      <div class="feature-item">
        <strong>🔒 Secure Auth</strong>
        <span>Google OAuth authentication</span>
      </div>
      <div class="feature-item">
        <strong>🛠️ 23+ Tools</strong>
        <span>Full suite of biomedical research tools</span>
      </div>
    </div>

    <a href="tutorials/remote-connection/" class="cta-button">
      Connect to Remote BioMCP Now
    </a>

  </div>
</div>

## What Can You Do with BioMCP?

### Search Research Literature

Find articles about genes, variants, diseases, and drugs with automatic cancer genomics data from cBioPortal

```bash
biomcp article search --gene BRAF --disease melanoma
```

### Discover Clinical Trials

Search active trials by condition, location, phase, and eligibility criteria including genetic biomarkers

```bash
biomcp trial search --condition "lung cancer" --status RECRUITING
```

### Analyze Genetic Variants

Query variant databases, predict effects, and understand clinical significance

```bash
biomcp variant search --gene TP53 --significance pathogenic
```

### AI-Powered Analysis

Use with Claude Desktop for conversational biomedical research with sequential thinking

```python
# Claude automatically uses BioMCP tools
"What BRAF mutations are found in melanoma?"
```

## 5-Minute Quick Start

### Choose Your Interface

=== "Claude Desktop (Recommended)"

    **Best for**: Conversational research, complex queries, AI-assisted analysis

    1. **Install Claude Desktop** from [claude.ai/desktop](https://claude.ai/desktop)

    2. **Configure BioMCP**:
       ```json
       {
         "mcpServers": {
           "biomcp": {
             "command": "uv",
             "args": [
        "run", "--with", "biomcp-python",
        "biomcp", "run"
      ]
           }
         }
       }
       ```

    3. **Start researching**: Ask Claude about any biomedical topic!

    [Full Claude Desktop Guide →](getting-started/02-claude-desktop-integration.md)

=== "Command Line"

    **Best for**: Direct queries, scripting, automation

    1. **Install BioMCP**:
       ```bash
       # Using uv (recommended)
       uv tool install biomcp

       # Or using pip
       pip install biomcp-python
       ```

    2. **Run your first search**:
       ```bash
       biomcp article search \
         --gene BRAF --disease melanoma \
         --limit 5
       ```

    [CLI Reference →](user-guides/01-command-line-interface.md)

=== "Python SDK"

    **Best for**: Integration, custom applications, bulk operations

    1. **Install the package**:
       ```bash
       pip install biomcp-python
       ```

    2. **Use in your code**:
       ```python
       from biomcp import BioMCPClient

       async with BioMCPClient() as client:
           articles = await client.articles.search(
               genes=["BRAF"],
               diseases=["melanoma"]
           )
       ```

    [Python SDK Docs →](apis/python-sdk.md)

## Key Features

### Unified Search Across Databases

- **PubMed/PubTator3**: 30M+ research articles with entity recognition
- **ClinicalTrials.gov**: 400K+ clinical trials worldwide
- **MyVariant.info**: Comprehensive variant annotations
- **cBioPortal**: Automatic cancer genomics integration

### Intelligent Query Processing

- Natural language to structured queries
- Automatic synonym expansion
- OR logic support for flexible matching
- Cross-domain relationship discovery

### Built for AI Integration

- 24 specialized MCP tools
- Sequential thinking for complex analysis
- Streaming responses for real-time updates
- Context preservation across queries

[Explore All Features →](concepts/01-what-is-biomcp.md)

## Learn by Example

### Find Articles About a Specific Mutation

```bash
# Search for BRAF V600E mutations
biomcp article search --gene BRAF \
  --keyword "V600E|p.V600E|c.1799T>A"
```

### Discover Trials Near You

```bash
# Find cancer trials in Boston area
biomcp trial search --condition cancer \
  --latitude 42.3601 --longitude -71.0589 \
  --distance 50
```

### Get Gene Information

```bash
# Get comprehensive gene data
biomcp gene get TP53
```

[More Examples →](tutorials/biothings-prompts.md)

## Popular Workflows

### Literature Review

Systematic search across papers, preprints, and clinical trials
[Workflow Guide →](workflows/all-workflows.md#1-literature-review-workflow)

### Variant Interpretation

From variant ID to clinical significance and treatment implications
[Workflow Guide →](workflows/all-workflows.md#3-variant-interpretation-workflow)

### Trial Matching

Find eligible trials based on patient criteria and biomarkers
[Workflow Guide →](workflows/all-workflows.md#2-clinical-trial-matching-workflow)

### Drug Research

Connect drugs to targets, trials, and research literature
[Workflow Guide →](workflows/all-workflows.md)

## Advanced Features

- **[NCI Integration](getting-started/03-authentication-and-api-keys.md#nci-clinical-trials-api)**: Enhanced cancer trial search with biomarker filtering
- **[AlphaGenome](how-to-guides/04-predict-variant-effects-with-alphagenome.md)**: Predict variant effects on gene regulation
- **[BigQuery Logging](how-to-guides/05-logging-and-monitoring-with-bigquery.md)**: Monitor usage and performance
- **[HTTP Server Mode](developer-guides/01-server-deployment.md)**: Deploy as a service

## Documentation

- **[Getting Started](getting-started/01-quickstart-cli.md)** - Installation and first steps
- **[User Guides](user-guides/01-command-line-interface.md)** - Detailed usage instructions
- **[API Reference](apis/overview.md)** - Technical documentation
- **[FAQ](faq-condensed.md)** - Quick answers to common questions

## Community & Support

- **GitHub**: [github.com/genomoncology/biomcp](https://github.com/genomoncology/biomcp)
- **Issues**: [Report bugs or request features](https://github.com/genomoncology/biomcp/issues)
- **Discussions**: [Ask questions and share tips](https://github.com/genomoncology/biomcp/discussions)

## License

BioMCP is licensed under the MIT License. See [LICENSE](https://github.com/genomoncology/biomcp/blob/main/LICENSE) for details.
