# MCP Tools Reference

BioMCP provides 12 specialized tools for biomedical research through the Model Context Protocol (MCP). This reference covers all available tools, their parameters, and usage patterns.

## Related Guides

- **Conceptual Overview**: [Sequential Thinking with the Think Tool](../concepts/03-sequential-thinking-with-the-think-tool.md)
- **Practical Examples**: See the [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for real-world usage patterns
- **Integration Setup**: [Claude Desktop Integration](../getting-started/02-claude-desktop-integration.md)

## Tool Overview

| Category | Tools | Actions |
|----------|-------|---------|
| **Core Tools** | `search`, `fetch`, `think` | Universal access across all domains |
| **Domain Tools** | `article`, `trial`, `variant`, `alphagenome` | Biomedical research |
| **BioThings Tools** | `gene`, `drug`, `disease` | Database lookups |
| **Specialized Tools** | `fda`, `nci` | FDA data & NCI clinical trials |

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

**Domains:** `article`, `trial`, `variant`, `gene`, `drug`, `disease`, `nci_organization`, `nci_intervention`, `nci_biomarker`, `nci_disease`, `fda_adverse`, `fda_label`, `fda_device`, `fda_approval`, `fda_recall`, `fda_shortage`

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

# FDA adverse events
search(domain="fda_adverse", chemicals=["aspirin"])
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
- FDA Records: Report IDs, Application Numbers, Recall Numbers

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
    nextThoughtNeeded=False
)
```

## Domain Tools

### 4. article - PubMed & Biomedical Literature

**Search and retrieve biomedical articles from PubMed/PubTator3.**

#### Search Articles

```python
article(
    action="search",
    genes: list[str] = None,
    diseases: list[str] = None,
    chemicals: list[str] = None,
    variants: list[str] = None,
    keywords: list[str] = None,        # Supports OR with "|"
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
article(
    action="search",
    genes=["BRAF"],
    diseases=["melanoma"],
    keywords=["resistance|resistant"],
    include_cbioportal=True
)
```

#### Get Article Details

```python
article(
    action="get",
    id: str  # PubMed ID, PMC ID, or DOI
) -> str
```

**Supports:** PubMed IDs ("38768446"), PMC IDs ("PMC7498215"), DOIs ("10.1101/2024.01.20.23288905")

---

### 5. trial - Clinical Trials

**Search ClinicalTrials.gov with comprehensive filters.**

#### Search Trials

```python
trial(
    action="search",
    conditions: list[str] = None,
    interventions: list[str] = None,
    other_terms: list[str] = None,
    recruiting_status: str = "ANY",    # "OPEN", "CLOSED", "ANY"
    phase: str = None,                 # "PHASE1", "PHASE2", etc.
    lat: float = None,                 # Location-based search
    long: float = None,
    distance: int = None,              # Miles from coordinates
    age_group: str = None,             # "CHILD", "ADULT", "OLDER_ADULT"
    sex: str = None,                   # "MALE", "FEMALE", "ALL"
    study_type: str = None,            # "INTERVENTIONAL", "OBSERVATIONAL"
    funder_type: str = None,           # "NIH", "INDUSTRY", etc.
    page: int = 1,
    page_size: int = 10
) -> str
```

**Location Search Example:**

```python
trial(
    action="search",
    conditions=["breast cancer"],
    lat=42.3601,
    long=-71.0589,
    distance=50,
    recruiting_status="OPEN"
)
```

#### Get Trial Details

```python
trial(
    action="get",
    id: str,              # NCT ID
    detail: str = None    # "protocol", "locations", "outcomes", "references", "all"
) -> str
```

**Detail Options:** Retrieve specific sections or all trial information.

---

### 6. variant - Genetic Variants

**Search MyVariant.info and retrieve comprehensive variant annotations.**

#### Search Variants

```python
variant(
    action="search",
    gene: str = None,
    hgvs: str = None,                  # General HGVS notation
    hgvsp: str = None,                 # Protein HGVS
    hgvsc: str = None,                 # Coding DNA HGVS
    rsid: str = None,
    region: str = None,                # "chr7:140753336-140753337"
    significance: str = None,          # Clinical significance
    frequency_min: float = None,
    frequency_max: float = None,
    cadd_score_min: float = None,
    sift_prediction: str = None,
    polyphen_prediction: str = None,
    include_cbioportal: bool = True,
    include_oncokb: bool = True,
    page: int = 1,
    page_size: int = 10
) -> str
```

**Significance Options:** `pathogenic`, `likely_pathogenic`, `uncertain_significance`, `likely_benign`, `benign`, `conflicting`

**Example:**

```python
variant(
    action="search",
    gene="BRCA1",
    significance="pathogenic",
    frequency_max=0.001,
    cadd_score_min=20
)
```

#### Get Variant Details

```python
variant(
    action="get",
    variant_id: str,              # HGVS, rsID, or MyVariant ID
    include_external: bool = True  # Include TCGA, 1000 Genomes
) -> str
```

---

### 7. alphagenome - Variant Effect Prediction

**Predict variant effects using Google DeepMind's AlphaGenome.**

```python
alphagenome(
    action="predict",
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
alphagenome(
    action="predict",
    chromosome="chr7",
    position=140753336,
    reference="A",
    alternate="T",
    tissue_types=["UBERON:0002367"]
)
```

---

### 8. gene - Gene Information

**Get comprehensive gene information from MyGene.info.**

```python
gene(
    action="get",
    id: str  # Gene symbol or Entrez ID
) -> str
```

**Returns:** Official name, aliases, summary, genomic location, database links

**Example:**

```python
gene(action="get", id="BRAF")
```

---

### 9. disease - Disease Information

**Get disease information from MyDisease.info.**

```python
disease(
    action="get",
    id: str  # Disease name or ontology ID
) -> str
```

**Returns:** Definition, synonyms, MONDO/DOID IDs, associated phenotypes

**Example:**

```python
disease(action="get", id="melanoma")
```

---

### 10. drug - Drug Information

**Get drug/chemical information from MyChem.info.**

```python
drug(
    action="get",
    id: str  # Drug name or database ID
) -> str
```

**Returns:** Chemical structure, mechanism, indications, trade names, identifiers

**Example:**

```python
drug(action="get", id="imatinib")
```

---

## Specialized Tools

### 11. fda - OpenFDA Data Access

**Access FDA databases for drug safety, labeling, and regulatory information.**

All OpenFDA tools support optional API keys for higher rate limits (240/min vs 40/min). Get a free key at [open.fda.gov/apis/authentication](https://open.fda.gov/apis/authentication/).

#### Domains

- **adverse** - FDA Adverse Event Reporting System (FAERS)
- **label** - Drug Product Labels (SPL)
- **device** - Medical Device Adverse Events (MAUDE)
- **approval** - Drug Approvals (Drugs@FDA)
- **recall** - Enforcement Reports
- **shortage** - Drug Shortages

#### Search FDA Data

```python
fda(
    domain: str,              # "adverse", "label", "device", etc.
    action="search",
    drug: str = None,
    limit: int = 25,
    page: int = 1,
    # Domain-specific parameters...
    api_key: str = None
) -> str
```

**Examples:**

```python
# Search adverse events
fda(domain="adverse", action="search", drug="warfarin", serious=True)

# Search drug labels
fda(domain="label", action="search", drug="keytruda", indication="melanoma")

# Search device events
fda(domain="device", action="search", device="sequencing", genomics_only=True)

# Search drug approvals
fda(domain="approval", action="search", approval_year="2024")

# Search recalls
fda(domain="recall", action="search", recall_class="1")

# Search drug shortages
fda(domain="shortage", action="search", status="current")
```

#### Get FDA Record Details

```python
fda(
    domain: str,
    action="get",
    id: str,                  # Domain-specific ID
    api_key: str = None
) -> str
```

**ID Requirements by Domain:**
- adverse: `report_id`
- label: `set_id`
- device: `mdr_report_key`
- approval: `application_number` (e.g., "BLA125514")
- recall: `recall_number`
- shortage: drug name

---

### 12. nci - NCI Clinical Trials API

**Access NCI organization, intervention, biomarker, and disease databases.**

All NCI tools require an API key from [api.cancer.gov](https://api.cancer.gov).

#### Resources

- **organization** - Cancer centers and research institutions
- **intervention** - Drugs, devices, and treatment interventions
- **biomarker** - Biomarkers used in trial eligibility
- **disease** - NCI Thesaurus disease terms

#### Search NCI Data

```python
nci(
    resource: str,           # "organization", "intervention", etc.
    action="search",
    name: str = None,
    # Resource-specific parameters...
    page_size: int = 20,
    page: int = 1,
    api_key: str = None
) -> str
```

**Examples:**

```python
# Search organizations
nci(
    resource="organization",
    action="search",
    name="Cancer Center",
    city="Boston",
    state="MA"
)

# Search interventions
nci(
    resource="intervention",
    action="search",
    name="pembrolizumab",
    intervention_type="Drug"
)

# Search biomarkers
nci(
    resource="biomarker",
    action="search",
    name="PD-L1"
)

# Search diseases
nci(
    resource="disease",
    action="search",
    name="melanoma",
    include_synonyms=True
)
```

#### Get NCI Record Details

```python
nci(
    resource: str,
    action="get",
    id: str,                 # Resource-specific ID
    api_key: str = None
) -> str
```

---

## Best Practices

### 1. Always Think First

```python
# ✅ CORRECT - Think before searching
think(thought="Planning BRAF melanoma research...", thoughtNumber=1)
results = article(action="search", genes=["BRAF"], diseases=["melanoma"])

# ❌ INCORRECT - Skipping think tool
results = article(action="search", genes=["BRAF"])  # Poor results!
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
articles = article(
    action="search",
    genes=["KRAS"],
    include_cbioportal=True  # Adds cancer genomics context
)

# Variant search with multiple filters
variants = variant(
    action="search",
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
nci_results = nci(resource="organization", action="search", name="Mayo Clinic")

# For shared environments - per-request
nci_results = nci(
    resource="organization",
    action="search",
    name="Mayo Clinic",
    api_key="user-provided-key"
)
```

### 5. Use Appropriate Page Sizes

```python
# Large result sets - increase page_size
results = article(
    action="search",
    genes=["TP53"],
    page_size=50  # Get more results at once
)

# Iterative exploration - use pagination
page1 = trial(action="search", conditions=["cancer"], page=1, page_size=10)
page2 = trial(action="search", conditions=["cancer"], page=2, page_size=10)
```

## Error Handling

All tools include comprehensive error handling:

- **Invalid parameters**: Clear error messages with valid options
- **API failures**: Graceful degradation with informative messages
- **Rate limits**: Automatic retry with exponential backoff
- **Missing API keys**: Helpful instructions for obtaining keys

## Tool Selection Guide

| If you need to... | Use this tool |
|-------------------|---------------|
| Search across multiple domains | `search` with query language |
| Get any record by ID | `fetch` with auto-detection |
| Plan your research approach | `think` (always first!) |
| Find recent papers | `article` |
| Locate clinical trials | `trial` |
| Analyze genetic variants | `variant` |
| Predict variant effects | `alphagenome` |
| Get gene/drug/disease info | `gene`, `drug`, `disease` |
| Access FDA databases | `fda` |
| Search NCI databases | `nci` |

## Next Steps

- Review [Sequential Thinking](../concepts/03-sequential-thinking-with-the-think-tool.md) methodology
- Explore [How-to Guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for complex workflows
- Set up [API Keys](../getting-started/03-authentication-and-api-keys.md) for enhanced features
