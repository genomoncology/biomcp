# Quickstart: BioMCP CLI

Get started with BioMCP in under 5 minutes! This guide walks you through installation and your first biomedical search.

## Prerequisites

- Python 3.10 or higher
- [uv](https://docs.astral.sh/uv/) package manager (recommended) or pip

## Installation

### Option 1: Using uv (Recommended)

```bash
# Install uv if you haven't already
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install BioMCP
uv tool install biomcp
```

### Option 2: Using pip

```bash
pip install biomcp
```

## Your First Search

Let's search for recent articles about BRAF mutations in melanoma:

```bash
biomcp article search \
  --gene BRAF --disease melanoma --limit 5
```

This command:

- Searches PubMed/PubTator3 for articles
- Filters by BRAF gene and melanoma disease
- Returns the 5 most recent results
- Automatically includes cBioPortal cancer genomics data
- Includes preprints from bioRxiv/medRxiv by default

## Understanding the Output

The search returns:

1. **cBioPortal Summary** (if gene specified): Cancer genomics data showing mutation frequencies and hotspots
2. **Article Results**: Each result includes:
   - Title and authors
   - Journal and publication date
   - PubMed ID and direct link
   - Abstract snippet
   - Annotated entities (genes, diseases, chemicals)

## Essential Commands

### Search Clinical Trials

Find active trials for lung cancer:

```bash
biomcp trial search \
  --condition "lung cancer" \
  --status RECRUITING --limit 5
```

### Get Gene Information

Retrieve details about the TP53 tumor suppressor:

```bash
biomcp gene get TP53
```

### Look Up Drug Information

Get details about imatinib (Gleevec):

```bash
biomcp drug get imatinib
```

### Search for Genetic Variants

Find pathogenic variants in the BRCA1 gene:

```bash
biomcp variant search \
  --gene BRCA1 --significance pathogenic \
  --limit 5
```

## Next Steps

### Set Up API Keys (Optional but Recommended)

Some features require API keys for enhanced functionality:

```bash
# For NCI clinical trials database
export NCI_API_KEY="your-key-here"

# For AlphaGenome variant predictions
export ALPHAGENOME_API_KEY="your-key-here"

# For additional cBioPortal features
export CBIO_TOKEN="your-token-here"
```

See [Authentication and API Keys](03-authentication-and-api-keys.md) for detailed setup.

### Explore Advanced Features

- **Combine Multiple Filters**:

  ```bash
  biomcp article search \
    --gene EGFR --disease "lung cancer" \
    --chemical erlotinib
  ```

- **Use OR Logic in Keywords**:

  ```bash
  biomcp article search --gene BRAF --keyword "V600E|p.V600E|c.1799T>A"
  ```

- **Exclude Preprints**:
  ```bash
  biomcp article search --gene TP53 --no-preprints
  ```

### Get Help

View all available commands:

```bash
biomcp --help
```

Get help for a specific command:

```bash
biomcp article search --help
```

## Common Use Cases

### 1. Research a Specific Mutation

```bash
# Find articles about EGFR T790M resistance mutation
biomcp article search --gene EGFR \
  --keyword "T790M|p.T790M" \
  --disease "lung cancer"
```

### 2. Find Trials for a Patient

```bash
# Active trials for HER2-positive breast cancer
biomcp trial search \
  --condition "breast cancer" \
  --keyword "HER2 positive" \
  --status RECRUITING
```

### 3. Investigate Drug Mechanisms

```bash
# Get information about pembrolizumab
biomcp drug get pembrolizumab

# Find articles about its use in melanoma
biomcp article search --chemical pembrolizumab --disease melanoma
```

## Troubleshooting

### Command Not Found

If `biomcp` is not recognized:

- Ensure your PATH includes the installation directory
- Try running with full path: `~/.local/bin/biomcp`
- Restart your terminal after installation

### No Results Found

If searches return no results:

- Check spelling of gene names (use official symbols)
- Try broader search terms
- Remove filters one by one to identify the constraint

### API Rate Limits

If you encounter rate limit errors:

- Add delays between requests
- Consider setting up API keys for higher limits
- Use the `--limit` parameter to reduce result count

## Next Steps

Now that you've run your first searches, explore these resources:

1. **[Complete CLI Reference](../user-guides/01-command-line-interface.md)** - Comprehensive documentation for all commands and options
2. **[Claude Desktop Integration](02-claude-desktop-integration.md)** - Use BioMCP with AI assistants
3. **[Set up API Keys](03-authentication-and-api-keys.md)** - Enable advanced features with NCI, AlphaGenome, and cBioPortal
4. **[How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md)** - Step-by-step tutorials for complex research workflows
5. **[Deep Researcher Persona](../concepts/02-the-deep-researcher-persona.md)** - Learn about BioMCP's philosophy and methodology

Happy researching! 🧬🔬
