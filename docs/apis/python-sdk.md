# Python SDK Reference

The BioMCP Python SDK provides async/await interfaces for accessing biomedical data from multiple sources.

## Installation

```bash
pip install biomcp-python
```

## Quick Start

```python
import asyncio
from biomcp import BioMCPClient

async def main():
    async with BioMCPClient() as client:
        # Search articles
        articles = await client.articles.search(
            genes=["BRAF"],
            diseases=["melanoma"],
            limit=5
        )

        # Get trial details
        trial = await client.trials.get("NCT03006926")

        # Search variants
        variants = await client.variants.search(
            gene="TP53",
            significance="pathogenic"
        )

asyncio.run(main())
```

## Client Initialization

### BioMCPClient

Main client class for accessing all BioMCP functionality.

```python
class BioMCPClient:
    def __init__(
        self,
        base_url: str = "http://localhost:8000",
        nci_api_key: Optional[str] = None,
        alphagenome_api_key: Optional[str] = None,
        cbio_token: Optional[str] = None,
        timeout: int = 120,
        max_retries: int = 3,
        cache_ttl: int = 900,  # 15 minutes
    ):
```

**Parameters:**

- `base_url`: BioMCP server URL (for remote deployments)
- `nci_api_key`: National Cancer Institute API key
- `alphagenome_api_key`: AlphaGenome API key
- `cbio_token`: cBioPortal access token
- `timeout`: Request timeout in seconds
- `max_retries`: Maximum retry attempts for failed requests
- `cache_ttl`: Cache time-to-live in seconds

**Example:**

```python
# Local development
client = BioMCPClient()

# Remote server
client = BioMCPClient(base_url="https://biomcp.example.com")

# With API keys
client = BioMCPClient(
    nci_api_key=os.getenv("NCI_API_KEY"),
    alphagenome_api_key=os.getenv("ALPHAGENOME_API_KEY")
)
```

## Article API

### articles.search()

Search PubMed/PubTator3 for biomedical literature.

```python
async def search(
    self,
    genes: Optional[List[str]] = None,
    diseases: Optional[List[str]] = None,
    chemicals: Optional[List[str]] = None,
    variants: Optional[List[str]] = None,
    keywords: Optional[List[str]] = None,
    pmids: Optional[List[str]] = None,
    include_preprints: bool = True,
    include_cbioportal: bool = True,
    limit: int = 10,
    page: int = 1,
) -> ArticleSearchResult:
```

**Parameters:**

- `genes`: List of gene symbols (e.g., ["BRAF", "KRAS"])
- `diseases`: List of disease/condition terms
- `chemicals`: List of drug/chemical names
- `variants`: List of variant notations
- `keywords`: Additional search keywords (supports OR with |)
- `pmids`: Specific PubMed IDs to retrieve
- `include_preprints`: Include bioRxiv/medRxiv preprints
- `include_cbioportal`: Include cBioPortal cancer genomics data
- `limit`: Maximum results per page
- `page`: Page number for pagination

**Returns:** `ArticleSearchResult` with articles and metadata

**Example:**

```python
# Basic search
results = await client.articles.search(
    genes=["EGFR"],
    diseases=["lung cancer"],
    limit=20
)

# Advanced search with keywords
results = await client.articles.search(
    genes=["BRAF"],
    keywords=["V600E|p.V600E|resistance"],
    chemicals=["vemurafenib", "dabrafenib"],
    include_preprints=False
)

# Iterate through results
for article in results.articles:
    print(f"{article.pmid}: {article.title}")
    print(f"Genes: {', '.join(article.genes)}")
    print(f"URL: {article.url}\n")
```

### articles.get()

Retrieve detailed information about a specific article.

```python
async def get(
    self,
    identifier: str,
    include_annotations: bool = True
) -> Article:
```

**Parameters:**

- `identifier`: PubMed ID or DOI
- `include_annotations`: Include PubTator3 entity annotations

**Returns:** `Article` object with full details

**Example:**

```python
# Get by PMID
article = await client.articles.get("38768446")

# Get by DOI (for preprints)
article = await client.articles.get("10.1101/2024.01.20.23288905")

# Access article data
print(f"Title: {article.title}")
print(f"Abstract: {article.abstract}")
print(f"Authors: {', '.join(article.authors)}")
print(f"Journal: {article.journal}")
print(f"Year: {article.year}")
```

## Trial API

### trials.search()

Search clinical trials from ClinicalTrials.gov or NCI.

```python
async def search(
    self,
    conditions: Optional[List[str]] = None,
    interventions: Optional[List[str]] = None,
    other_terms: Optional[List[str]] = None,
    nct_ids: Optional[List[str]] = None,
    status: Optional[str] = None,
    phase: Optional[str] = None,
    study_type: Optional[str] = None,
    lat: Optional[float] = None,
    long: Optional[float] = None,
    distance: Optional[int] = None,
    source: str = "ctgov",  # or "nci"
    expand_synonyms: bool = True,
    limit: int = 10,
    page: int = 1,
) -> TrialSearchResult:
```

**Parameters:**

- `conditions`: Disease/condition terms
- `interventions`: Treatment/intervention terms
- `other_terms`: Additional search terms
- `nct_ids`: Specific NCT IDs
- `status`: Trial status (RECRUITING, ACTIVE_NOT_RECRUITING, etc.)
- `phase`: Trial phase (PHASE1, PHASE2, PHASE3, etc.)
- `study_type`: INTERVENTIONAL or OBSERVATIONAL
- `lat`, `long`, `distance`: Geographic search parameters
- `source`: Data source ("ctgov" or "nci")
- `expand_synonyms`: Auto-expand disease synonyms
- `limit`, `page`: Pagination parameters

**Returns:** `TrialSearchResult` with trials and metadata

**Example:**

```python
# Basic search
trials = await client.trials.search(
    conditions=["melanoma"],
    status="RECRUITING",
    phase="PHASE3"
)

# Location-based search
trials = await client.trials.search(
    conditions=["breast cancer"],
    lat=40.7128,
    long=-74.0060,
    distance=50  # miles
)

# NCI search with mutations
trials = await client.trials.search(
    source="nci",
    conditions=["lung cancer"],
    required_mutations=["EGFR L858R"],
    allow_brain_mets=True
)
```

### trials.get()

Get detailed information about a specific trial.

```python
async def get(
    self,
    nct_id: str,
    include_all: bool = False,
    source: str = "ctgov"
) -> Trial:
```

**Parameters:**

- `nct_id`: Clinical trial identifier
- `include_all`: Include all available sections
- `source`: Data source ("ctgov" or "nci")

**Returns:** `Trial` object with full details

## Variant API

### variants.search()

Search genetic variants in MyVariant.info.

```python
async def search(
    self,
    gene: Optional[str] = None,
    hgvs: Optional[str] = None,
    rsid: Optional[str] = None,
    chromosome: Optional[str] = None,
    start: Optional[int] = None,
    end: Optional[int] = None,
    assembly: str = "hg38",
    significance: Optional[Union[str, List[str]]] = None,
    min_frequency: Optional[float] = None,
    max_frequency: Optional[float] = None,
    min_cadd: Optional[float] = None,
    include_cbioportal: bool = True,
    limit: int = 10,
) -> VariantSearchResult:
```

**Parameters:**

- `gene`: Gene symbol
- `hgvs`: HGVS notation
- `rsid`: dbSNP rsID
- `chromosome`, `start`, `end`: Genomic coordinates
- `assembly`: Genome assembly (hg19 or hg38)
- `significance`: Clinical significance filter
- `min_frequency`, `max_frequency`: Allele frequency filters
- `min_cadd`: Minimum CADD score
- `include_cbioportal`: Include cancer genomics data

**Returns:** `VariantSearchResult` with variants

**Example:**

```python
# Search pathogenic variants
variants = await client.variants.search(
    gene="BRCA1",
    significance=["pathogenic", "likely_pathogenic"],
    max_frequency=0.01
)

# Search by genomic region
variants = await client.variants.search(
    chromosome="7",
    start=140453136,
    end=140453137,
    assembly="hg38"
)
```

### variants.get()

Get detailed variant information.

```python
async def get(
    self,
    variant_id: str,
    include_external: bool = True
) -> Variant:
```

**Parameters:**

- `variant_id`: Variant identifier (HGVS, rsID, or genomic)
- `include_external`: Include external database annotations

**Returns:** `Variant` object with annotations

### variants.predict()

Predict variant effects using AlphaGenome.

```python
async def predict(
    self,
    chromosome: str,
    position: int,
    reference: str,
    alternate: str,
    tissue_types: Optional[List[str]] = None,
    interval: int = 20000,
) -> AlphaGenomePrediction:
```

**Parameters:**

- `chromosome`: Chromosome (e.g., "chr7")
- `position`: Genomic position
- `reference`: Reference allele
- `alternate`: Alternate allele
- `tissue_types`: UBERON tissue ontology terms
- `interval`: Analysis window size

**Returns:** `AlphaGenomePrediction` with effect predictions

## BioThings API

### genes.get()

Get gene information from MyGene.info.

```python
async def get(self, gene_symbol: str) -> Gene:
```

### diseases.get()

Get disease information from MyDisease.info.

```python
async def get(self, disease_term: str) -> Disease:
```

### drugs.get()

Get drug information from MyChem.info.

```python
async def get(self, drug_name: str) -> Drug:
```

## Unified Search API

### client.search()

Unified search across all domains.

```python
async def search(
    self,
    query: Optional[str] = None,
    domain: Optional[str] = None,
    **kwargs
) -> SearchResult:
```

**Parameters:**

- `query`: Unified query language string
- `domain`: Target domain
- `**kwargs`: Domain-specific parameters

**Query Language Examples:**

- `"gene:BRAF AND disease:melanoma"`
- `"drugs.tradename:gleevec"`
- `"gene:TP53 AND (mutation OR variant)"`

## Streaming API

### client.stream()

Stream large result sets efficiently.

```python
async def stream(
    self,
    domain: str,
    **search_params
) -> AsyncIterator[Dict]:
```

**Example:**

```python
# Stream all BRCA1 articles
async for article in client.stream(
    domain="article",
    genes=["BRCA1"]
):
    print(f"Processing {article['pmid']}")
```

## Batch Operations

### client.batch()

Process multiple queries efficiently.

```python
async def batch(
    self,
    queries: List[Dict]
) -> List[Result]:
```

**Example:**

```python
queries = [
    {"domain": "gene", "id": "BRAF"},
    {"domain": "gene", "id": "KRAS"},
    {"domain": "drug", "id": "vemurafenib"}
]

results = await client.batch(queries)
```

## Error Handling

```python
from biomcp.exceptions import (
    BioMCPError,
    NotFoundError,
    RateLimitError,
    ValidationError,
    APIKeyError
)

try:
    article = await client.articles.get("invalid-pmid")
except NotFoundError:
    print("Article not found")
except RateLimitError as e:
    print(f"Rate limited. Retry after {e.retry_after} seconds")
except BioMCPError as e:
    print(f"Error: {e}")
```

## Data Models

### Article

```python
class Article:
    pmid: str
    title: str
    abstract: str
    authors: List[str]
    journal: str
    year: int
    doi: Optional[str]
    url: str
    genes: List[str]
    diseases: List[str]
    chemicals: List[str]
    variants: List[str]
    metadata: Dict[str, Any]
```

### Trial

```python
class Trial:
    nct_id: str
    title: str
    status: str
    phase: Optional[str]
    conditions: List[str]
    interventions: List[str]
    sponsors: List[str]
    start_date: Optional[date]
    completion_date: Optional[date]
    locations: List[Location]
    eligibility: Eligibility
    description: str
    primary_outcomes: List[str]
    secondary_outcomes: List[str]
```

### Variant

```python
class Variant:
    id: str
    gene: Gene
    chromosome: str
    position: int
    ref: str
    alt: str
    hgvs: Optional[str]
    rsid: Optional[str]
    clinical_significance: Optional[str]
    frequencies: PopulationFrequencies
    predictions: FunctionalPredictions
    diseases: List[Disease]
    external_data: Dict[str, Any]
```

## Best Practices

### 1. Use Context Managers

```python
async with BioMCPClient() as client:
    # Client automatically handles cleanup
    results = await client.articles.search(genes=["TP53"])
```

### 2. Handle Pagination

```python
all_articles = []
page = 1

while True:
    results = await client.articles.search(
        genes=["BRCA1"],
        page=page,
        limit=100
    )
    all_articles.extend(results.articles)

    if len(results.articles) < 100:
        break
    page += 1
```

### 3. Implement Retry Logic

```python
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=4, max=10)
)
async def robust_search(client, **params):
    return await client.articles.search(**params)
```

### 4. Cache Results

```python
from functools import lru_cache
import hashlib
import json

@lru_cache(maxsize=1000)
async def cached_gene_get(client, gene_symbol):
    return await client.genes.get(gene_symbol)
```

## Complete Example

```python
import asyncio
from biomcp import BioMCPClient

async def variant_analysis_workflow(gene: str, disease: str):
    """Complete variant analysis workflow."""

    async with BioMCPClient() as client:
        # 1. Get gene information
        gene_info = await client.genes.get(gene)
        print(f"Analyzing {gene_info.name} ({gene_info.symbol})")

        # 2. Search for pathogenic variants
        variants = await client.variants.search(
            gene=gene,
            significance="pathogenic",
            max_frequency=0.01
        )
        print(f"Found {len(variants.variants)} pathogenic variants")

        # 3. Search related articles
        articles = await client.articles.search(
            genes=[gene],
            diseases=[disease],
            keywords=["therapy", "treatment"],
            limit=10
        )
        print(f"Found {len(articles.articles)} relevant articles")

        # 4. Find clinical trials
        trials = await client.trials.search(
            conditions=[disease],
            other_terms=[gene, f"{gene} mutation"],
            status="RECRUITING"
        )
        print(f"Found {len(trials.trials)} recruiting trials")

        # 5. Compile results
        return {
            "gene": gene_info,
            "pathogenic_variants": variants.variants[:5],
            "key_articles": articles.articles[:5],
            "active_trials": trials.trials[:5]
        }

# Run the workflow
results = asyncio.run(
    variant_analysis_workflow("BRAF", "melanoma")
)
```

## Additional Resources

- [Error Codes Reference](error-codes.md)
- [Performance Optimization Guide](../developer-guides/07-performance-optimizations.md)
- [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md)
