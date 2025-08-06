# BioMCP Quick Reference

## Command Cheat Sheet

### Installation

```bash
# Install BioMCP
uv tool install biomcp

# Update to latest version
uv tool install biomcp --force

# Check version
biomcp --version
```

### Article Search Commands

```bash
# Basic gene search
biomcp article search --gene BRAF

# Multiple filters
biomcp article search \
  --gene EGFR --disease "lung cancer" \
  --chemical erlotinib

# Exclude preprints
biomcp article search --gene TP53 --no-preprints

# OR logic in keywords
biomcp article search --gene PTEN \
  --keyword "R173|Arg173|p.R173"

# Get specific article
biomcp article get 38768446  # PMID
biomcp article get "10.1101/2024.01.20.23288905"  # DOI
```

### Trial Search Commands

```bash
# Basic disease search
biomcp trial search \
  --condition melanoma --status RECRUITING

# Location-based search (requires coordinates)
biomcp trial search --condition cancer \
  --latitude 40.7128 --longitude -74.0060 --distance 50

# Phase-specific search
biomcp trial search \
  --condition "breast cancer" --phase PHASE3

# Using NCI source (requires API key)
biomcp trial search --condition melanoma --source nci \
  --required-mutations "BRAF V600E" --api-key $NCI_API_KEY
```

### Variant Commands

```bash
# Search by gene
biomcp variant search \
  --gene BRCA1 --significance pathogenic

# Search by HGVS
biomcp variant search --hgvs "NM_007294.4:c.5266dupC"

# Search by frequency
biomcp variant search --gene TP53 \
  --max-frequency 0.01 --min-cadd 20

# Get variant details
biomcp variant get rs121913529
biomcp variant get "NM_007294.4:c.5266dupC"

# Predict effects (requires AlphaGenome key)
biomcp variant predict chr7 140753336 A T --tissue UBERON:0002367
```

### Gene/Drug/Disease Commands

```bash
# Get gene information
biomcp gene get TP53
biomcp gene get BRAF

# Get drug information
biomcp drug get imatinib
biomcp drug get pembrolizumab

# Get disease information
biomcp disease get melanoma
biomcp disease get "non-small cell lung cancer"
```

### NCI Commands (Require API Key)

```bash
# Search organizations
biomcp organization search --name "MD Anderson" \
  --city Houston --state TX --api-key $NCI_API_KEY

# Search interventions
biomcp intervention search --name pembrolizumab \
  --intervention-type Drug --api-key $NCI_API_KEY

# Search biomarkers
biomcp biomarker search --gene EGFR \
  --biomarker-type mutation --api-key $NCI_API_KEY
```

### Health Check

```bash
# Full health check
biomcp health check

# Check APIs only
biomcp health check --apis-only

# Verbose output
biomcp health check --verbose
```

## Common Parameter Reference

### Search Parameters

| Parameter  | Description   | Example         |
| ---------- | ------------- | --------------- |
| `--limit`  | Max results   | `--limit 20`    |
| `--page`   | Page number   | `--page 2`      |
| `--format` | Output format | `--format json` |

### Trial Status Values

| Status                  | Description            |
| ----------------------- | ---------------------- |
| `RECRUITING`            | Currently enrolling    |
| `ACTIVE_NOT_RECRUITING` | Ongoing, not enrolling |
| `NOT_YET_RECRUITING`    | Will start recruiting  |
| `COMPLETED`             | Trial has ended        |
| `SUSPENDED`             | Temporarily halted     |
| `TERMINATED`            | Stopped early          |

### Trial Phase Values

| Phase          | Description   |
| -------------- | ------------- |
| `EARLY_PHASE1` | Early Phase 1 |
| `PHASE1`       | Phase 1       |
| `PHASE2`       | Phase 2       |
| `PHASE3`       | Phase 3       |
| `PHASE4`       | Phase 4       |

### Clinical Significance

| Value                    | Description             |
| ------------------------ | ----------------------- |
| `pathogenic`             | Causes disease          |
| `likely_pathogenic`      | Probably causes disease |
| `uncertain_significance` | Unknown impact          |
| `likely_benign`          | Probably harmless       |
| `benign`                 | Does not cause disease  |

## Gene Symbol Quick Lookup

### Common Gene Aliases

| Common Name | Official Symbol |
| ----------- | --------------- |
| HER2        | ERBB2           |
| HER3        | ERBB3           |
| EGFR        | EGFR            |
| ALK         | ALK             |
| c-MET       | MET             |
| PD-1        | PDCD1           |
| PD-L1       | CD274           |
| CTLA-4      | CTLA4           |

## Location Coordinates

### Major US Cities

| City          | Latitude | Longitude |
| ------------- | -------- | --------- |
| New York      | 40.7128  | -74.0060  |
| Los Angeles   | 34.0522  | -118.2437 |
| Chicago       | 41.8781  | -87.6298  |
| Houston       | 29.7604  | -95.3698  |
| Philadelphia  | 39.9526  | -75.1652  |
| Boston        | 42.3601  | -71.0589  |
| Atlanta       | 33.7490  | -84.3880  |
| Miami         | 25.7617  | -80.1918  |
| Seattle       | 47.6062  | -122.3321 |
| San Francisco | 37.7749  | -122.4194 |

## Environment Variables

```bash
# API Keys
export NCI_API_KEY="your-nci-key"
export ALPHAGENOME_API_KEY="your-alphagenome-key"
export CBIO_TOKEN="your-cbioportal-token"

# Configuration
export BIOMCP_LOG_LEVEL="DEBUG"
export BIOMCP_CACHE_DIR="/path/to/cache"
export BIOMCP_TIMEOUT=300
export BIOMCP_MAX_CONCURRENT=5
```

## Output Format Examples

### JSON Output

```bash
biomcp article search --gene BRAF --format json | jq '.articles[0]'
```

### Extract Specific Fields

```bash
# Get PMIDs only
biomcp article search --gene TP53 --format json | \
  jq -r '.articles[].pmid'

# Get trial NCT IDs
biomcp trial search --condition melanoma --format json | \
  jq -r '.trials[].nct_id'
```

### Save to File

```bash
biomcp article search --gene BRCA1 --format json > results.json
```

## MCP Tool Names

### Core Tools

- `search` - Unified search
- `fetch` - Get details
- `think` - Sequential thinking

### Article Tools

- `article_searcher`
- `article_getter`

### Trial Tools

- `trial_searcher`
- `trial_getter`
- `trial_protocol_getter`
- `trial_references_getter`
- `trial_outcomes_getter`
- `trial_locations_getter`

### Variant Tools

- `variant_searcher`
- `variant_getter`
- `alphagenome_predictor`

### BioThings Tools

- `gene_getter`
- `disease_getter`
- `drug_getter`

### NCI Tools

- `nci_organization_searcher`
- `nci_organization_getter`
- `nci_intervention_searcher`
- `nci_intervention_getter`
- `nci_biomarker_searcher`
- `nci_disease_searcher`

## Query Language Syntax

### Unified Search Examples

```
gene:BRAF AND disease:melanoma
gene:EGFR AND (mutation OR variant)
drugs.tradename:gleevec
diseases.name:"lung cancer"
chemicals.mesh:D000069439
```

### Field Prefixes

- `gene:` - Gene symbol
- `disease:` - Disease/condition
- `chemical:` - Drug/chemical
- `variant:` - Genetic variant
- `pmid:` - PubMed ID
- `doi:` - Digital Object ID

## Common Workflows

### Find Articles About a Mutation

```bash
# Step 1: Search articles
biomcp article search --gene BRAF --keyword "V600E|p.V600E"

# Step 2: Get full article
biomcp article get [PMID]
```

### Check Trial Eligibility

```bash
# Step 1: Search trials
biomcp trial search --condition melanoma --status RECRUITING

# Step 2: Get trial details
biomcp trial get NCT03006926
```

### Variant Analysis

```bash
# Step 1: Search variant
biomcp variant search --gene BRCA1 --significance pathogenic

# Step 2: Get variant details
biomcp variant get rs80357906

# Step 3: Search related articles
biomcp article search --gene BRCA1 --variant rs80357906
```

## Error Code Quick Reference

### Common HTTP Codes

- `400` - Bad request (check parameters)
- `401` - Unauthorized (check API key)
- `404` - Not found (verify ID)
- `429` - Rate limited (wait and retry)
- `500` - Server error (retry later)

### BioMCP Error Patterns

- `1xxx` - Article errors
- `2xxx` - Trial errors
- `3xxx` - Variant errors
- `4xxx` - Gene/drug/disease errors
- `5xxx` - Authentication errors
- `6xxx` - Rate limit errors
- `7xxx` - Validation errors

## Tips and Tricks

### 1. Use Official Gene Symbols

```bash
# Wrong
biomcp article search --gene HER2  # ❌

# Right
biomcp article search --gene ERBB2  # ✅
```

### 2. Combine Multiple Searches

```bash
# Search multiple databases in parallel
(
  biomcp article search --gene BRAF --format json > articles.json &
  biomcp trial search --condition melanoma --format json > trials.json &
  biomcp variant search --gene BRAF --format json > variants.json &
  wait
)
```

### 3. Process Large Results

```bash
# Paginate through results
for page in {1..10}; do
  biomcp article search --gene TP53 --page $page --limit 100
done
```

### 4. Debug API Issues

```bash
# Enable debug logging
export BIOMCP_LOG_LEVEL=DEBUG
biomcp article search --gene BRAF --verbose
```

## Getting Help

```bash
# General help
biomcp --help

# Command help
biomcp article search --help

# Check documentation
open https://biomcp.org/

# Report issues
open https://github.com/genomoncology/biomcp/issues
```
