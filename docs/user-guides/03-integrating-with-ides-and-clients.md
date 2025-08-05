# Integrating with IDEs and Clients

BioMCP can be integrated into your development workflow through multiple approaches. This guide covers integration with IDEs, Python applications, and MCP-compatible clients.

## Integration Methods Overview

| Method         | Best For                  | Installation | Usage Pattern            |
| -------------- | ------------------------- | ------------ | ------------------------ |
| **Cursor IDE** | Interactive development   | Smithery CLI | Natural language queries |
| **Python SDK** | Application development   | pip/uv       | Direct function calls    |
| **MCP Client** | AI assistants & protocols | Subprocess   | Tool-based communication |

## Cursor IDE Integration

Cursor IDE provides the most seamless integration for interactive biomedical research during development.

### Installation

1. **Prerequisites:**

   - [Cursor IDE](https://cursor.sh/) installed
   - [Smithery](https://smithery.ai/) account and token

2. **Install BioMCP:**

   ```bash
   npx -y @smithery/cli@latest install @genomoncology/biomcp --client cursor
   ```

3. **Configuration:**
   - The Smithery CLI automatically configures Cursor
   - No manual configuration needed

### Usage in Cursor

Once installed, you can query biomedical data using natural language:

#### Clinical Trials

```
"Find Phase 3 clinical trials for lung cancer with immunotherapy"
```

#### Research Articles

```
"Summarize recent research on EGFR mutations in lung cancer"
```

#### Genetic Variants

```
"What's the clinical significance of the BRAF V600E mutation?"
```

#### Complex Queries

```
"Compare treatment outcomes for ALK-positive vs EGFR-mutant NSCLC"
```

### Cursor Tips

1. **Be Specific**: Include gene names, disease types, and treatment modalities
2. **Iterate**: Refine queries based on initial results
3. **Cross-Reference**: Ask for both articles and trials on the same topic
4. **Export Results**: Copy formatted results for documentation

## Python SDK Integration

The Python SDK provides programmatic access to BioMCP for building applications.

### Installation

```bash
# Using pip
pip install biomcp-python

# Using uv
uv add biomcp-python

# For scripts
uv pip install biomcp-python
```

### Basic Usage

```python
import asyncio
from biomcp import BioMCP

async def main():
    # Initialize client
    client = BioMCP()

    # Search for articles
    articles = await client.articles.search(
        genes=["BRAF"],
        diseases=["melanoma"],
        limit=5
    )

    # Search for trials
    trials = await client.trials.search(
        conditions=["breast cancer"],
        interventions=["CDK4/6 inhibitor"],
        recruiting_status="RECRUITING"
    )

    # Get variant details
    variant = await client.variants.get("rs121913529")

    return articles, trials, variant

# Run the async function
results = asyncio.run(main())
```

### Advanced Features

#### Domain-Specific Modules

```python
from biomcp import BioMCP
from biomcp.variants import search_variants, get_variant
from biomcp.trials import search_trials, get_trial
from biomcp.articles import search_articles, fetch_articles

# Direct module usage
async def variant_analysis():
    # Search pathogenic TP53 variants
    results = await search_variants(
        gene="TP53",
        significance="pathogenic",
        frequency_max=0.01,
        limit=20
    )

    # Get detailed annotations
    for variant in results:
        details = await get_variant(variant.id)
        print(f"{variant.id}: {details.clinical_significance}")
```

#### Output Formats

```python
# JSON for programmatic use
articles_json = await client.articles.search(
    genes=["KRAS"],
    format="json"
)

# Markdown for display
articles_md = await client.articles.search(
    genes=["KRAS"],
    format="markdown"
)
```

#### Error Handling

```python
from biomcp.exceptions import BioMCPError, APIError, ValidationError

try:
    results = await client.articles.search(genes=["INVALID_GENE"])
except ValidationError as e:
    print(f"Invalid input: {e}")
except APIError as e:
    print(f"API error: {e}")
except BioMCPError as e:
    print(f"General error: {e}")
```

### Example: Building a Variant Report

```python
import asyncio
from biomcp import BioMCP

async def generate_variant_report(gene: str, mutation: str):
    client = BioMCP()

    # 1. Get gene information
    gene_info = await client.genes.get(gene)

    # 2. Search for the specific variant
    variants = await client.variants.search(
        gene=gene,
        keywords=[mutation]
    )

    # 3. Find relevant articles
    articles = await client.articles.search(
        genes=[gene],
        keywords=[mutation],
        limit=10
    )

    # 4. Look for clinical trials
    trials = await client.trials.search(
        conditions=["cancer"],
        other_terms=[f"{gene} {mutation}"],
        recruiting_status="RECRUITING"
    )

    # 5. Generate report
    report = f"""
# Variant Report: {gene} {mutation}

## Gene Information
- **Official Name**: {gene_info.name}
- **Summary**: {gene_info.summary}

## Variant Details
Found {len(variants)} matching variants

## Literature ({len(articles)} articles)
Recent publications discussing this variant...

## Clinical Trials ({len(trials)} active trials)
Currently recruiting studies...
"""

    return report

# Generate report
report = asyncio.run(generate_variant_report("BRAF", "V600E"))
print(report)
```

## MCP Client Integration

The Model Context Protocol (MCP) provides a standardized way to integrate BioMCP with AI assistants and other tools.

### Understanding MCP

MCP is a protocol for communication between:

- **Clients**: AI assistants, IDEs, or custom applications
- **Servers**: Tool providers like BioMCP

### Critical Requirement: Think Tool

**IMPORTANT**: When using MCP, you MUST call the `think` tool first before any search or fetch operations. This ensures systematic analysis and optimal results.

### Basic MCP Integration

```python
import asyncio
import subprocess
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

async def run_biomcp_query():
    # Start BioMCP server
    server_params = StdioServerParameters(
        command="uv",
        args=["run", "--with", "biomcp-python", "biomcp", "run"],
        env={"PYTHONUNBUFFERED": "1"}
    )

    async with stdio_client(server_params) as (read, write):
        async with ClientSession(read, write) as session:
            # Initialize and discover tools
            await session.initialize()
            tools = await session.list_tools()

            # CRITICAL: Always think first!
            await session.call_tool(
                "think",
                arguments={
                    "thought": "Analyzing BRAF V600E in melanoma...",
                    "thoughtNumber": 1,
                    "nextThoughtNeeded": True
                }
            )

            # Now search for articles
            result = await session.call_tool(
                "article_searcher",
                arguments={
                    "genes": ["BRAF"],
                    "diseases": ["melanoma"],
                    "keywords": ["V600E"]
                }
            )

            return result

# Run the query
result = asyncio.run(run_biomcp_query())
```

### Available MCP Tools

BioMCP provides 24 tools through MCP:

#### Core Tools (Always Use First)

- `think` - Sequential reasoning (MANDATORY first step)
- `search` - Unified search across domains
- `fetch` - Retrieve specific records

#### Domain-Specific Tools

- **Articles**: `article_searcher`, `article_getter`
- **Trials**: `trial_searcher`, `trial_getter`, plus detail getters
- **Variants**: `variant_searcher`, `variant_getter`, `alphagenome_predictor`
- **BioThings**: `gene_getter`, `disease_getter`, `drug_getter`
- **NCI**: Organization, intervention, biomarker, disease tools

### MCP Integration Patterns

#### Pattern 1: AI Assistant Integration

```python
# Example for integrating with an AI assistant
class BioMCPAssistant:
    def __init__(self):
        self.session = None

    async def connect(self):
        # Initialize MCP connection
        server_params = StdioServerParameters(
            command="biomcp",
            args=["run"]
        )
        # ... connection setup ...

    async def process_query(self, user_query: str):
        # 1. Always think first
        await self.think_about_query(user_query)

        # 2. Determine appropriate tools
        tools_needed = self.analyze_query(user_query)

        # 3. Execute tool calls
        results = []
        for tool in tools_needed:
            result = await self.session.call_tool(tool.name, tool.args)
            results.append(result)

        # 4. Synthesize results
        return self.format_response(results)
```

#### Pattern 2: Custom Client Implementation

```python
import json
from typing import Any, Dict

class BioMCPClient:
    """Custom client for specific biomedical workflows"""

    async def variant_to_trials_pipeline(self, variant_id: str):
        """Find trials for patients with specific variants"""

        # Step 1: Think and plan
        await self.think(
            "Planning variant-to-trials search pipeline...",
            thoughtNumber=1
        )

        # Step 2: Get variant details
        variant = await self.call_tool("variant_getter", {
            "variant_id": variant_id
        })

        # Step 3: Extract gene and disease associations
        gene = variant.get("gene", {}).get("symbol")
        diseases = self.extract_diseases(variant)

        # Step 4: Search for relevant trials
        trials = await self.call_tool("trial_searcher", {
            "conditions": diseases,
            "other_terms": [f"{gene} mutation"],
            "recruiting_status": "RECRUITING"
        })

        return {
            "variant": variant,
            "associated_trials": trials
        }
```

### MCP Best Practices

1. **Always Think First**

   ```python
   # ✅ Correct
   await think(thought="Planning research...", thoughtNumber=1)
   await search(...)

   # ❌ Wrong - skips thinking
   await search(...)  # Will produce poor results
   ```

2. **Use Appropriate Tools**

   ```python
   # For broad searches across domains
   await call_tool("search", {"query": "gene:BRAF AND melanoma"})

   # For specific domain searches
   await call_tool("article_searcher", {"genes": ["BRAF"]})
   ```

3. **Handle Tool Responses**
   ```python
   try:
       result = await session.call_tool("variant_getter", {
           "variant_id": "rs121913529"
       })
       # Process structured result
       if result.get("error"):
           handle_error(result["error"])
       else:
           process_variant(result["data"])
   except Exception as e:
       logger.error(f"Tool call failed: {e}")
   ```

## Choosing the Right Integration

### Use Cursor IDE When:

- Doing interactive research during development
- Exploring biomedical data for new projects
- Need quick answers without writing code
- Want natural language queries

### Use Python SDK When:

- Building production applications
- Need type-safe interfaces
- Want direct function calls
- Require custom error handling

### Use MCP Client When:

- Integrating with AI assistants
- Building protocol-compliant tools
- Need standardized tool interfaces
- Want language-agnostic integration

## Integration Examples

### Example 1: Research Dashboard (Python SDK)

```python
from biomcp import BioMCP
import streamlit as st

async def create_dashboard():
    client = BioMCP()

    st.title("Biomedical Research Dashboard")

    # Gene input
    gene = st.text_input("Enter gene symbol:", "BRAF")

    if st.button("Search"):
        # Fetch comprehensive data
        col1, col2 = st.columns(2)

        with col1:
            st.subheader("Recent Articles")
            articles = await client.articles.search(genes=[gene], limit=5)
            for article in articles:
                st.write(f"- [{article.title}]({article.url})")

        with col2:
            st.subheader("Active Trials")
            trials = await client.trials.search(
                other_terms=[gene],
                recruiting_status="RECRUITING",
                limit=5
            )
            for trial in trials:
                st.write(f"- [{trial.nct_id}]({trial.url})")
```

### Example 2: Variant Analysis Pipeline (MCP)

```python
async def comprehensive_variant_analysis(session, hgvs: str):
    """Complete variant analysis workflow using MCP"""

    # Think about the analysis
    await session.call_tool("think", {
        "thought": f"Planning comprehensive analysis for {hgvs}",
        "thoughtNumber": 1
    })

    # Get variant details
    variant = await session.call_tool("variant_getter", {
        "variant_id": hgvs
    })

    # Search related articles
    articles = await session.call_tool("article_searcher", {
        "variants": [hgvs],
        "limit": 10
    })

    # Find applicable trials
    gene = variant.get("gene", {}).get("symbol")
    trials = await session.call_tool("trial_searcher", {
        "other_terms": [f"{gene} mutation"],
        "recruiting_status": "RECRUITING"
    })

    # Predict functional effects if genomic coordinates available
    if variant.get("chrom") and variant.get("pos"):
        prediction = await session.call_tool("alphagenome_predictor", {
            "chromosome": f"chr{variant['chrom']}",
            "position": variant["pos"],
            "reference": variant["ref"],
            "alternate": variant["alt"]
        })

    return {
        "variant": variant,
        "articles": articles,
        "trials": trials,
        "prediction": prediction
    }
```

## Troubleshooting

### Common Issues

1. **"Think tool not called" errors**

   - Always call think before other operations
   - Include thoughtNumber parameter

2. **API rate limits**

   - Add delays between requests
   - Use API keys for higher limits

3. **Connection failures**

   - Check network connectivity
   - Verify server is running
   - Ensure correct installation

4. **Invalid gene symbols**
   - Use official HGNC symbols
   - Check [genenames.org](https://www.genenames.org)

### Debug Mode

Enable debug logging:

```python
# Python SDK
import logging
logging.basicConfig(level=logging.DEBUG)

# MCP Client
server_params = StdioServerParameters(
    command="biomcp",
    args=["run", "--log-level", "DEBUG"]
)
```

## Next Steps

- Explore [tool-specific documentation](02-mcp-tools-reference.md)
- Review [API authentication](../getting-started/03-authentication-and-api-keys.md)
- Check [example workflows](../how-to-guides/01-find-articles-and-cbioportal-data.md) for your use case
