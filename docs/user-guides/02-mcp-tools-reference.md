# MCP Tools Reference

BioMCP provides 24 specialized tools for biomedical research through the Model Context Protocol (MCP). This reference covers all available tools, their parameters, and usage patterns.

## Related Guides

- **Conceptual Overview**: [Sequential Thinking with the Think Tool](../concepts/03-sequential-thinking-with-the-think-tool.md)
- **Practical Examples**: See the [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for real-world usage patterns
- **Integration Setup**: [Claude Desktop Integration](../getting-started/02-claude-desktop-integration.md)

## Tool Categories

| Category            | Count | Tools                                                         |
| ------------------- | ----- | ------------------------------------------------------------- |
| **Core Tools**      | 3     | `search`, `fetch`, `think`                                    |
| **Article Tools**   | 2     | `article_searcher`, `article_getter`                          |
| **Trial Tools**     | 6     | `trial_searcher`, `trial_getter`, + 4 detail getters          |
| **Variant Tools**   | 3     | `variant_searcher`, `variant_getter`, `alphagenome_predictor` |
| **BioThings Tools** | 3     | `gene_getter`, `disease_getter`, `drug_getter`                |
| **NCI Tools**       | 6     | Organization, intervention, biomarker, and disease tools      |

## Core Unified Tools

### 1. search

**Universal search across all biomedical domains with unified query language.**

```python
search(
    query: str = None,              # Unified query syntax
    domain: str = None,             # Target domain
    genes: list[str] = None,        # Gene symbols
    diseases: list[str] = None,     # Disease/condition terms
    variants: list[str] = None,     # Variant notations
    chemicals: list[str] = None,    # Drug/chemical names
    keywords: list[str] = None,     # Additional keywords
    conditions: list[str] = None,   # Trial conditions
    interventions: list[str] = None,# Trial interventions
    lat: float = None,              # Latitude for trials
    long: float = None,             # Longitude for trials
    page: int = 1,                  # Page number
    page_size: int = 10,            # Results per page
    api_key: str = None             # For NCI domains
) -> dict
```

**Domains:** `article`, `trial`, `variant`, `gene`, `drug`, `disease`, `nci_organization`, `nci_intervention`, `nci_biomarker`, `nci_disease`

**Query Language Examples:**

- `"gene:BRAF AND disease:melanoma"`
- `"drugs.tradename:gleevec"`
- `"gene:TP53 AND (mutation OR variant)"`

**Usage Examples:**

```python
# Domain-specific search
search(domain="article", genes=["BRAF"], diseases=["melanoma"])

# Unified query language
search(query="gene:EGFR AND mutation:T790M")

# Clinical trials by location
search(domain="trial", conditions=["lung cancer"], lat=40.7128, long=-74.0060)
```

### 2. fetch

**Retrieve detailed information for any biomedical record.**

```python
fetch(
    id: str,                    # Record identifier
    domain: str = None,         # Domain (auto-detected if not provided)
    detail: str = None,         # Specific section for trials
    api_key: str = None         # For NCI records
) -> dict
```

**Supported IDs:**

- Articles: PMID (e.g., "38768446"), DOI (e.g., "10.1101/2024.01.20")
- Trials: NCT ID (e.g., "NCT03006926")
- Variants: HGVS, rsID, genomic coordinates
- Genes/Drugs/Diseases: Names or database IDs

**Detail Options for Trials:** `protocol`, `locations`, `outcomes`, `references`, `all`

**Usage Examples:**

```python
# Fetch article by PMID
fetch(id="38768446", domain="article")

# Fetch trial with specific details
fetch(id="NCT03006926", domain="trial", detail="locations")

# Auto-detect domain
fetch(id="rs121913529")  # Variant
fetch(id="BRAF")         # Gene
```

### 3. think

**Sequential thinking tool for structured problem-solving.**

```python
think(
    thought: str,               # Current reasoning step
    thoughtNumber: int,         # Sequential number (1, 2, 3...)
    totalThoughts: int = None,  # Estimated total thoughts
    nextThoughtNeeded: bool = True  # Continue thinking?
) -> str
```

**CRITICAL:** Always use `think` BEFORE any other BioMCP operation!

**Usage Pattern:**

```python
# Step 1: Problem decomposition
think(
    thought="Breaking down query: need to find BRAF inhibitor trials...",
    thoughtNumber=1,
    nextThoughtNeeded=True
)

# Step 2: Search strategy
think(
    thought="Will search trials for BRAF V600E melanoma, then articles...",
    thoughtNumber=2,
    nextThoughtNeeded=True
)

# Final step: Synthesis
think(
    thought="Ready to synthesize findings from 5 trials and 12 articles...",
    thoughtNumber=3,
    nextThoughtNeeded=False  # Analysis complete
)
```

## Article Tools

### 4. article_searcher

**Search PubMed/PubTator3 for biomedical literature.**

```python
article_searcher(
    chemicals: list[str] = None,
    diseases: list[str] = None,
    genes: list[str] = None,
    keywords: list[str] = None,    # Supports OR with "|"
    variants: list[str] = None,
    include_preprints: bool = True,
    include_cbioportal: bool = True,
    page: int = 1,
    page_size: int = 10
) -> str
```

**Features:**

- Automatic cBioPortal integration for gene searches
- Preprint inclusion from bioRxiv/medRxiv
- OR logic in keywords: `"V600E|p.V600E|c.1799T>A"`

**Example:**

```python
# Search with multiple filters
article_searcher(
    genes=["BRAF"],
    diseases=["melanoma"],
    keywords=["resistance|resistant"],
    include_cbioportal=True
)
```

### 5. article_getter

**Fetch detailed article information.**

```python
article_getter(
    pmid: str  # PubMed ID, PMC ID, or DOI
) -> str
```

**Supports:**

- PubMed IDs: "38768446"
- PMC IDs: "PMC7498215"
- DOIs: "10.1101/2024.01.20.23288905"

## Trial Tools

### 6. trial_searcher

**Search ClinicalTrials.gov with comprehensive filters.**

```python
trial_searcher(
    conditions: list[str] = None,
    interventions: list[str] = None,
    other_terms: list[str] = None,
    recruiting_status: str = "ANY",  # "OPEN", "CLOSED", "ANY"
    phase: str = None,               # "PHASE1", "PHASE2", etc.
    lat: float = None,               # Location-based search
    long: float = None,
    distance: int = None,            # Miles from coordinates
    age_group: str = None,           # "CHILD", "ADULT", "OLDER_ADULT"
    sex: str = None,                 # "MALE", "FEMALE", "ALL"
    study_type: str = None,          # "INTERVENTIONAL", "OBSERVATIONAL"
    funder_type: str = None,         # "NIH", "INDUSTRY", etc.
    page: int = 1,
    page_size: int = 10
) -> str
```

**Location Search Example:**

```python
# Trials near Boston
trial_searcher(
    conditions=["breast cancer"],
    lat=42.3601,
    long=-71.0589,
    distance=50,
    recruiting_status="OPEN"
)
```

### 7-11. Trial Detail Getters

```python
# Get complete trial information
trial_getter(nct_id: str) -> str

# Get specific sections
trial_protocol_getter(nct_id: str) -> str     # Core protocol info
trial_locations_getter(nct_id: str) -> str    # Sites and contacts
trial_outcomes_getter(nct_id: str) -> str     # Outcome measures
trial_references_getter(nct_id: str) -> str   # Publications
```

## Variant Tools

### 12. variant_searcher

**Search MyVariant.info for genetic variants.**

```python
variant_searcher(
    gene: str = None,
    hgvs: str = None,
    hgvsp: str = None,              # Protein HGVS
    hgvsc: str = None,              # Coding DNA HGVS
    rsid: str = None,
    region: str = None,             # "chr7:140753336-140753337"
    significance: str = None,        # Clinical significance
    frequency_min: float = None,
    frequency_max: float = None,
    cadd_score_min: float = None,
    sift_prediction: str = None,
    polyphen_prediction: str = None,
    sources: list[str] = None,
    include_cbioportal: bool = True,
    page: int = 1,
    page_size: int = 10
) -> str
```

**Significance Options:** `pathogenic`, `likely_pathogenic`, `uncertain_significance`, `likely_benign`, `benign`

**Example:**

```python
# Find rare pathogenic BRCA1 variants
variant_searcher(
    gene="BRCA1",
    significance="pathogenic",
    frequency_max=0.001,
    cadd_score_min=20
)
```

### 13. variant_getter

**Fetch comprehensive variant details.**

```python
variant_getter(
    variant_id: str,              # HGVS, rsID, or MyVariant ID
    include_external: bool = True  # Include TCGA, 1000 Genomes
) -> str
```

### 14. alphagenome_predictor

**Predict variant effects using Google DeepMind's AlphaGenome.**

```python
alphagenome_predictor(
    chromosome: str,              # e.g., "chr7"
    position: int,                # 1-based position
    reference: str,               # Reference allele
    alternate: str,               # Alternate allele
    interval_size: int = 131072,  # Analysis window
    tissue_types: list[str] = None,  # UBERON terms
    significance_threshold: float = 0.5,
    api_key: str = None          # AlphaGenome API key
) -> str
```

**Requires:** AlphaGenome API key (environment variable or per-request)

**Tissue Examples:**

- `UBERON:0002367` - prostate gland
- `UBERON:0001155` - colon
- `UBERON:0002048` - lung

**Example:**

```python
# Predict BRAF V600E effects
alphagenome_predictor(
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    tissue_types=["UBERON:0002367"],  # prostate
    api_key="your-key"
)
```

## BioThings Tools

### 15. gene_getter

**Get gene information from MyGene.info.**

```python
gene_getter(
    gene_id_or_symbol: str  # Gene symbol or Entrez ID
) -> str
```

**Returns:** Official name, aliases, summary, genomic location, database links

### 16. disease_getter

**Get disease information from MyDisease.info.**

```python
disease_getter(
    disease_id_or_name: str  # Disease name or ontology ID
) -> str
```

**Returns:** Definition, synonyms, MONDO/DOID IDs, associated phenotypes

### 17. drug_getter

**Get drug/chemical information from MyChem.info.**

```python
drug_getter(
    drug_id_or_name: str  # Drug name or database ID
) -> str
```

**Returns:** Chemical structure, mechanism, indications, trade names, identifiers

## NCI-Specific Tools

All NCI tools require an API key from [api.cancer.gov](https://api.cancer.gov).

### 18-19. Organization Tools

```python
# Search organizations
nci_organization_searcher(
    name: str = None,
    organization_type: str = None,
    city: str = None,              # Must use with state
    state: str = None,             # Must use with city
    api_key: str = None
) -> str

# Get organization details
nci_organization_getter(
    organization_id: str,
    api_key: str = None
) -> str
```

### 20-21. Intervention Tools

```python
# Search interventions
nci_intervention_searcher(
    name: str = None,
    intervention_type: str = None,  # "Drug", "Device", etc.
    synonyms: bool = True,
    api_key: str = None
) -> str

# Get intervention details
nci_intervention_getter(
    intervention_id: str,
    api_key: str = None
) -> str
```

### 22. Biomarker Search

```python
nci_biomarker_searcher(
    name: str = None,
    biomarker_type: str = None,
    api_key: str = None
) -> str
```

### 23. Disease Search (NCI)

```python
nci_disease_searcher(
    name: str = None,
    include_synonyms: bool = True,
    category: str = None,
    api_key: str = None
) -> str
```

## Best Practices

### 1. Always Think First

```python
# ✅ CORRECT - Think before searching
think(thought="Planning BRAF melanoma research...", thoughtNumber=1)
results = article_searcher(genes=["BRAF"], diseases=["melanoma"])

# ❌ INCORRECT - Skipping think tool
results = article_searcher(genes=["BRAF"])  # Poor results!
```

### 2. Use Unified Tools for Flexibility

```python
# Unified search supports complex queries
results = search(query="gene:EGFR AND (mutation:T790M OR mutation:C797S)")

# Unified fetch auto-detects domain
details = fetch(id="NCT03006926")  # Knows it's a trial
```

### 3. Leverage Domain-Specific Features

```python
# Article search with cBioPortal
articles = article_searcher(
    genes=["KRAS"],
    include_cbioportal=True  # Adds cancer genomics context
)

# Variant search with multiple filters
variants = variant_searcher(
    gene="TP53",
    significance="pathogenic",
    frequency_max=0.01,
    cadd_score_min=25
)
```

### 4. Handle API Keys Properly

```python
# For personal use - environment variable
# export NCI_API_KEY="your-key"
nci_results = search(domain="nci_organization", name="Mayo Clinic")

# For shared environments - per-request
nci_results = search(
    domain="nci_organization",
    name="Mayo Clinic",
    api_key="user-provided-key"
)
```

### 5. Use Appropriate Page Sizes

```python
# Large result sets - increase page_size
results = article_searcher(
    genes=["TP53"],
    page_size=50  # Get more results at once
)

# Iterative exploration - use pagination
page1 = trial_searcher(conditions=["cancer"], page=1, page_size=10)
page2 = trial_searcher(conditions=["cancer"], page=2, page_size=10)
```

## Error Handling

All tools include comprehensive error handling:

- **Invalid parameters**: Clear error messages with valid options
- **API failures**: Graceful degradation with informative messages
- **Rate limits**: Automatic retry with exponential backoff
- **Missing API keys**: Helpful instructions for obtaining keys

## Tool Selection Guide

| If you need to...              | Use this tool                                  |
| ------------------------------ | ---------------------------------------------- |
| Search across multiple domains | `search` with query language                   |
| Get any record by ID           | `fetch` with auto-detection                    |
| Plan your research approach    | `think` (always first!)                        |
| Find recent papers             | `article_searcher`                             |
| Locate clinical trials         | `trial_searcher`                               |
| Analyze genetic variants       | `variant_searcher` + `variant_getter`          |
| Predict variant effects        | `alphagenome_predictor`                        |
| Get gene/drug/disease info     | `gene_getter`, `drug_getter`, `disease_getter` |
| Access NCI databases           | `nci_*` tools with API key                     |

## Next Steps

- Review [Sequential Thinking](../concepts/03-sequential-thinking-with-the-think-tool.md) methodology
- Explore [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for complex workflows
- Set up [API Keys](../getting-started/03-authentication-and-api-keys.md) for enhanced features
