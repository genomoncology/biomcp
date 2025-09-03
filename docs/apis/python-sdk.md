# Python Package Reference

The BioMCP Python package provides direct access to biomedical data search and retrieval functions through modular domain-specific APIs.

## Installation

```bash
pip install biomcp-python
```

## Quick Start

```python
import asyncio
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance
from biomcp.articles.search import search_articles, PubmedRequest
from biomcp.trials.search import search_trials, TrialQuery

async def main():
    # Search for pathogenic variants
    variant_query = VariantQuery(
        gene="BRAF",
        significance=ClinicalSignificance.PATHOGENIC
    )
    variants_result = await search_variants(variant_query)

    # Search articles
    article_request = PubmedRequest(
        genes=["BRAF"],
        diseases=["melanoma"]
    )
    articles_result = await search_articles(article_request)

    # Search clinical trials
    trial_query = TrialQuery(
        conditions=["melanoma"],
        status="RECRUITING"
    )
    trials_result = await search_trials(trial_query)

asyncio.run(main())
```

## API Structure

The BioMCP package is organized into domain-specific modules that you import directly:

### Available Modules

- **Variants**: `biomcp.variants.search` - Search genetic variants
- **Articles**: `biomcp.articles.search` - Search biomedical literature
- **Trials**: `biomcp.trials.search` - Search clinical trials
- **Genes**: `biomcp.genes` - Get gene information
- **Diseases**: `biomcp.diseases` - Get disease information
- **Drugs**: `biomcp.drugs` - Get drug information

### Import Patterns

```python
# Variants
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance
from biomcp.variants.getter import get_variant
from biomcp.variants.alphagenome import predict_variant_effects

# Articles
from biomcp.articles.search import search_articles, PubmedRequest

# Trials
from biomcp.trials.search import search_trials, TrialQuery, TrialPhase

# Direct functions
from biomcp.genes import get_gene
from biomcp.diseases import get_disease
from biomcp.drugs import get_drug
```

## Articles API

### search_articles()

Search PubMed/PubTator3 for biomedical literature.

```python
from biomcp.articles.search import search_articles, PubmedRequest

async def search_articles(
    request: PubmedRequest,
    output_json: bool = False
) -> str:
```

**PubmedRequest Parameters:**

- `genes`: List of gene symbols (e.g., ["BRAF", "KRAS"])
- `diseases`: List of disease/condition terms
- `chemicals`: List of drug/chemical names
- `variants`: List of variant notations
- `keywords`: Additional search keywords (supports OR with |)

**Example:**

```python
from biomcp.articles.search import search_articles, PubmedRequest

# Basic search
request = PubmedRequest(
    genes=["EGFR"],
    diseases=["lung cancer"]
)
results = await search_articles(request)

# Advanced search with keywords
request = PubmedRequest(
    genes=["BRAF"],
    keywords=["V600E|p.V600E|resistance"],
    chemicals=["vemurafenib", "dabrafenib"]
)
results = await search_articles(request)
```

## Trials API

### search_trials()

Search clinical trials from ClinicalTrials.gov.

```python
from biomcp.trials.search import search_trials, TrialQuery, TrialPhase, RecruitingStatus

async def search_trials(
    query: TrialQuery,
    output_json: bool = False
) -> str:
```

**TrialQuery Parameters:**

- `conditions`: Disease/condition terms
- `interventions`: Treatment/intervention terms
- `other_terms`: Additional search terms
- `status`: Trial status (use RecruitingStatus enum)
- `phase`: Trial phase (use TrialPhase enum)
- `study_type`: INTERVENTIONAL or OBSERVATIONAL
- `lat`, `long`, `distance`: Geographic search parameters

**Available Enums:**

- `TrialPhase`: EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4, NOT_APPLICABLE
- `RecruitingStatus`: OPEN, CLOSED, ANY
- `StudyType`: INTERVENTIONAL, OBSERVATIONAL, EXPANDED_ACCESS

**Example:**

```python
from biomcp.trials.search import search_trials, TrialQuery, TrialPhase

# Basic search
query = TrialQuery(
    conditions=["melanoma"],
    phase=TrialPhase.PHASE3,
    recruiting_status="RECRUITING"
)
results = await search_trials(query)

# Location-based search
query = TrialQuery(
    conditions=["breast cancer"],
    lat=40.7128,
    long=-74.0060,
    distance=50
)
results = await search_trials(query)
```

## Variants API

### search_variants()

Search genetic variants in MyVariant.info.

```python
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance

async def search_variants(
    query: VariantQuery,
    output_json: bool = False,
    include_cbioportal: bool = True
) -> str:
```

**VariantQuery Parameters:**

- `gene`: Gene symbol (e.g. BRAF, TP53)
- `hgvsp`: Protein change notation (e.g., p.V600E, p.Arg557His)
- `hgvsc`: cDNA notation (e.g., c.1799T>A)
- `rsid`: dbSNP rsID (e.g., rs113488022)
- `region`: Genomic region as chr:start-end (e.g. chr1:12345-67890)
- `significance`: ClinVar clinical significance (use ClinicalSignificance enum)
- `min_frequency`, `max_frequency`: Allele frequency filters
- `cadd`: Minimum CADD phred score
- `polyphen`: PolyPhen-2 prediction (use PolyPhenPrediction enum)
- `sift`: SIFT prediction (use SiftPrediction enum)
- `sources`: Include only specific data sources
- `size`: Number of results to return
- `offset`: Result offset for pagination

**Available Enums:**

- `ClinicalSignificance`: PATHOGENIC, LIKELY_PATHOGENIC, UNCERTAIN_SIGNIFICANCE, LIKELY_BENIGN, BENIGN
- `PolyPhenPrediction`: PROBABLY_DAMAGING, POSSIBLY_DAMAGING, BENIGN
- `SiftPrediction`: DELETERIOUS, TOLERATED

**Example:**

```python
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance

# Search pathogenic variants
query = VariantQuery(
    gene="BRCA1",
    significance=ClinicalSignificance.PATHOGENIC,
    max_frequency=0.01
)
results = await search_variants(query)

# Search by genomic region
query = VariantQuery(
    region="chr7:140453136-140453137"
)
results = await search_variants(query)

# Search by protein change
query = VariantQuery(
    gene="BRAF",
    hgvsp="p.V600E"
)
results = await search_variants(query)
```

### get_variant()

Get detailed variant information.

```python
from biomcp.variants.getter import get_variant

async def get_variant(
    variant_id: str,
    output_json: bool = False,
    include_external: bool = False
) -> str:
```

**Parameters:**

- `variant_id`: Variant identifier (HGVS, rsID, or genomic like "chr7:g.140453136A>T")
- `output_json`: Return JSON format instead of markdown
- `include_external`: Include external database annotations

**Example:**

```python
# Get by HGVS
variant_info = await get_variant("chr7:g.140453136A>T")

# Get by rsID
variant_info = await get_variant("rs113488022")
```

### predict_variant_effects()

Predict variant effects using AlphaGenome AI.

```python
from biomcp.variants.alphagenome import predict_variant_effects

async def predict_variant_effects(
    chromosome: str,
    position: int,
    reference: str,
    alternate: str,
    interval_size: int = 131_072,
    tissue_types: list[str] | None = None,
    significance_threshold: float = 0.5,
    api_key: str | None = None
) -> str:
```

**Parameters:**

- `chromosome`: Chromosome (e.g., 'chr7')
- `position`: 1-based genomic position
- `reference`: Reference allele(s)
- `alternate`: Alternate allele(s)
- `interval_size`: Size of genomic context window (max 1,000,000)
- `tissue_types`: UBERON tissue ontology terms for tissue-specific predictions
- `significance_threshold`: Threshold for significant log2 fold changes
- `api_key`: AlphaGenome API key (or set ALPHAGENOME_API_KEY env var)

**Example:**

```python
# Predict effects of BRAF V600E mutation
prediction = await predict_variant_effects(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    api_key="your-alphagenome-api-key"
)
```

## Direct Data APIs

### get_gene()

Get gene information from MyGene.info.

```python
from biomcp.genes import get_gene

async def get_gene(
    gene_id_or_symbol: str,
    output_json: bool = False
) -> str:
```

**Example:**

```python
gene_info = await get_gene("BRCA1")
```

### get_disease()

Get disease information from MyDisease.info.

```python
from biomcp.diseases import get_disease

async def get_disease(
    disease_id_or_name: str,
    output_json: bool = False
) -> str:
```

**Example:**

```python
disease_info = await get_disease("melanoma")
```

### get_drug()

Get drug information from MyChem.info.

```python
from biomcp.drugs import get_drug

async def get_drug(
    drug_id_or_name: str,
    output_json: bool = False
) -> str:
```

**Example:**

```python
drug_info = await get_drug("imatinib")
```

## Complete Analysis Example

```python
import asyncio
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance
from biomcp.articles.search import search_articles, PubmedRequest
from biomcp.trials.search import search_trials, TrialQuery, TrialPhase
from biomcp.genes import get_gene

async def analyze_gene_variants(gene_symbol: str, disease: str):
    """Complete gene variant analysis workflow."""

    # 1. Get gene information
    gene_info = await get_gene(gene_symbol)
    print(f"Gene: {gene_symbol}")

    # 2. Search for pathogenic variants
    variant_query = VariantQuery(
        gene=gene_symbol,
        significance=ClinicalSignificance.PATHOGENIC,
        max_frequency=0.01  # Rare variants
    )
    variants_result = await search_variants(variant_query)
    print(f"Found pathogenic variants for {gene_symbol}")

    # 3. Search related literature
    article_request = PubmedRequest(
        genes=[gene_symbol],
        diseases=[disease],
        keywords=["therapy", "treatment", "prognosis"]
    )
    articles_result = await search_articles(article_request)
    print(f"Found literature on {gene_symbol} and {disease}")

    # 4. Find clinical trials
    trial_query = TrialQuery(
        conditions=[disease],
        other_terms=[gene_symbol, f"{gene_symbol} mutation"],
        phase=TrialPhase.PHASE3,
        recruiting_status="RECRUITING"
    )
    trials_result = await search_trials(trial_query)
    print(f"Found trials for {disease} with {gene_symbol}")

    return {
        "gene_info": gene_info,
        "variants": variants_result,
        "articles": articles_result,
        "trials": trials_result
    }

# Run the analysis
results = asyncio.run(analyze_gene_variants("BRAF", "melanoma"))
```

## LangChain Integration

```python
from langchain.tools import tool
from biomcp.variants.search import search_variants, VariantQuery, ClinicalSignificance
from biomcp.articles.search import search_articles, PubmedRequest

@tool
def search_pathogenic_variants(gene: str) -> str:
    """Search for pathogenic variants in a specific gene."""
    import asyncio

    async def _search():
        query = VariantQuery(
            gene=gene,
            significance=ClinicalSignificance.PATHOGENIC
        )
        return await search_variants(query)

    return asyncio.run(_search())

@tool
def search_gene_literature(gene: str, disease: str = None) -> str:
    """Search for scientific literature about a gene and optionally a disease."""
    import asyncio

    async def _search():
        request = PubmedRequest(
            genes=[gene],
            diseases=[disease] if disease else []
        )
        return await search_articles(request)

    return asyncio.run(_search())

# Use with your LLM/agent framework
tools = [search_pathogenic_variants, search_gene_literature]
```

## Key Differences from Other Documentation

❌ **Does NOT work:**

```python
from biomcp import BioMCPClient  # This class doesn't exist
```

✅ **Actually works:**

```python
from biomcp.variants.search import search_variants, VariantQuery
from biomcp.articles.search import search_articles, PubmedRequest
from biomcp.trials.search import search_trials, TrialQuery
```

## Summary

The BioMCP package provides powerful biomedical data access through:

- **Direct async functions** for each domain (variants, articles, trials, genes, diseases, drugs)
- **Pydantic models** for type-safe queries and responses
- **Comprehensive enums** for standardized values
- **No unified client** - use individual domain modules directly

This modular approach works well for building tools and integrating with frameworks like LangChain, as it provides direct access to specific functionality without the overhead of a unified client interface.

## Additional Resources

- [MCP Tools Reference](../mcp-tools/)
- [CLI Commands](../cli/)
- [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md)
