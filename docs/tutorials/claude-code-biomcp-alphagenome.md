# Using Claude Code with BioMCP for AlphaGenome Variant Analysis

This tutorial demonstrates how to use Claude Code with BioMCP to analyze genetic variants using Google DeepMind's AlphaGenome. We'll explore both the MCP server integration and CLI approaches, showing how Claude Code can seamlessly work with both interfaces.

## Prerequisites

- **Claude Code**: Latest version with MCP support
- **Python 3.11+**: Required for BioMCP and AlphaGenome
- **uv**: Modern Python package manager ([installation guide](https://docs.astral.sh/uv/getting-started/installation/))
- **AlphaGenome API Key**: Get free access at [Google DeepMind AlphaGenome](https://deepmind.google.com/science/alphagenome)

## Setup Overview

BioMCP offers two interfaces that work perfectly with Claude Code:

1. **MCP Server**: Integrated directly into Claude Code for seamless workflows
2. **CLI**: Command-line interface for direct terminal access

Both produce identical results, giving you flexibility in how you work.

## Part 1: MCP Server Setup

### Step 1: Install BioMCP CLI

```bash
# Install BioMCP CLI globally (note: biomcp-python, not biomcp!)
uv tool install -q biomcp-python

# Verify installation
biomcp --version
```

### Step 2: Configure MCP Server

Add BioMCP to your Claude Code MCP configuration:

```bash
# Basic setup (requires ALPHAGENOME_API_KEY environment variable)
claude mcp add biomcp -- uv run --with biomcp-python biomcp run

# Or with API key in configuration
claude mcp add biomcp -e ALPHAGENOME_API_KEY=your-api-key-here -- uv run --with biomcp-python biomcp run
```

Verify the setup:

```bash
claude mcp list
claude mcp get biomcp
```

### Step 3: Set Environment Variable

```bash
# Add to your shell profile (~/.zshrc or ~/.bashrc)
export ALPHAGENOME_API_KEY='your-api-key-here'

# Or set per-session
export ALPHAGENOME_API_KEY='your-api-key-here'
```

### Step 4: Install AlphaGenome

```bash
# Clone and install AlphaGenome
git clone https://github.com/google-deepmind/alphagenome.git
cd alphagenome && uv pip install .
```

## Part 2: Testing with Claude Code

### Example: DLG1 Exon Skipping Variant

Let's analyze the variant `chr3:197081044:TACTC>T` from the AlphaGenome paper, which demonstrates exon skipping in the DLG1 gene.

#### Using MCP Server (Recommended)

```python
# Claude Code automatically uses MCP when available
mcp__biomcp__alphagenome_predictor(
    chromosome="chr3",
    position=197081044,
    reference="TACTC",
    alternate="T"
)
```

**Result:**

```markdown
## AlphaGenome Variant Effect Predictions

**Variant**: chr3:197081044 TACTC>T
**Analysis window**: 131,072 bp

### Gene Expression

- **MELTF**: +2.57 log₂ fold change (↑ increases expression)

### Chromatin Accessibility

- **EFO:0005719 DNase-seq**: +17.27 log₂ change (↑ increases accessibility)

### Splicing

- Potential splicing alterations detected

### Summary

- Analyzed 11796 regulatory tracks
- 6045 tracks show substantial changes (|log₂| > 0.5)
```

#### Using CLI Interface

```bash
# Same analysis via CLI
export ALPHAGENOME_API_KEY='your-api-key-here'
uv run biomcp variant predict chr3 197081044 TACTC T
```

**Result:** Identical output to MCP server.

## Part 3: Why Both Interfaces Matter

### MCP Server Advantages 🔌

- **Persistent State**: No need to re-export environment variables
- **Workflow Integration**: Seamless chaining with other biomedical tools
- **Structured Data**: Direct programmatic access to results
- **Auto-Documentation**: Built-in parameter validation

### CLI Advantages 💻

- **Immediate Access**: No server setup required
- **Debugging**: Direct command-line testing
- **Scripting**: Easy integration into bash scripts
- **Standalone Use**: Works without Claude Code

### Claude Code Perspective

As Claude Code, both interfaces work equally well. The **MCP approach provides slight benefits**:

- Results persist across conversation turns
- Built-in error handling and validation
- Automatic integration with thinking and search workflows
- No need to manage environment variables per session

**Trade-off**: MCP requires initial setup, while CLI is immediately available.

## Part 4: Advanced Usage Examples

### Multi-Variant Analysis

```python
# Analyze multiple variants from AlphaGenome paper
variants = [
    ("chr3", 197081044, "TACTC", "T"),      # DLG1 exon skipping
    ("chr21", 46126238, "G", "C"),          # COL6A2 splice junction
    ("chr16", 173694, "A", "G")             # HBA2 polyadenylation
]

for chr, pos, ref, alt in variants:
    result = mcp__biomcp__alphagenome_predictor(
        chromosome=chr,
        position=pos,
        reference=ref,
        alternate=alt
    )
    print(f"Most affected gene: {result}")
```

### Tissue-Specific Analysis

```python
# Analyze with tissue context
mcp__biomcp__alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    tissue_types=["UBERON:0000310"]  # breast tissue
)
```

### Combined BioMCP Workflow

```python
# 1. First, search for known annotations
variant_data = mcp__biomcp__variant_searcher(gene="BRAF")

# 2. Then predict regulatory effects
regulatory_effects = mcp__biomcp__alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T"
)

# 3. Search literature for context
literature = mcp__biomcp__article_searcher(
    genes=["BRAF"],
    variants=["V600E"]
)
```

## Part 5: Validation and Quality Assurance

### How We Validated the Integration

1. **Raw API Testing**: Directly tested Google's AlphaGenome API
2. **Source Code Analysis**: Verified BioMCP uses correct API methods (`score_variant` + `get_recommended_scorers`)
3. **Cross-Validation**: Confirmed identical results across all three approaches:
   - Raw Python API: MELTF +2.57 log₂
   - BioMCP CLI: MELTF +2.57 log₂
   - BioMCP MCP: MELTF +2.57 log₂

### Key Scientific Finding

The variant `chr3:197081044:TACTC>T` most strongly affects **MELTF** (+2.57 log₂ fold change), not DLG1 as initially expected. This demonstrates that AlphaGenome considers the full regulatory landscape, not just the nearest gene.

## Part 6: Best Practices

### For MCP Usage

- Use structured thinking with `mcp__biomcp__think` for complex analyses
- Leverage `call_benefit` parameter to improve result quality
- Chain multiple tools for comprehensive variant characterization

### For CLI Usage

- Set `ALPHAGENOME_API_KEY` in your shell profile
- Use `--help` to explore all available parameters
- Combine with other CLI tools via pipes and scripts

### General Tips

- Start with default 131kb analysis window
- Use tissue-specific analysis when relevant
- Validate surprising results with literature search
- Consider both gene expression and chromatin accessibility effects

## Conclusion

BioMCP's dual interface approach (MCP + CLI) provides robust variant analysis capabilities. Claude Code works seamlessly with both, offering flexibility for different workflows. The MCP integration provides slight advantages for interactive analysis, while the CLI excels for scripting and debugging.

The combination of AlphaGenome's predictive power with BioMCP's comprehensive biomedical data access creates a powerful platform for genetic variant analysis and interpretation.

## Resources

- [BioMCP Documentation](https://biomcp.org)
- [AlphaGenome Paper](https://deepmind.google/science/alphagenome)
- [Claude Code MCP Guide](https://docs.anthropic.com/claude/docs/model-context-protocol)
- [uv Documentation](https://docs.astral.sh/uv/)
