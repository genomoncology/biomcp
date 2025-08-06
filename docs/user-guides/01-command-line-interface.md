# Command Line Interface Reference

BioMCP provides a comprehensive command-line interface for biomedical data retrieval and analysis. This guide covers all available commands, options, and usage patterns.

## Installation

```bash
# Using uv (recommended)
uv tool install biomcp

# Using pip
pip install biomcp-python
```

## Global Options

These options work with all commands:

```bash
biomcp [OPTIONS] COMMAND [ARGS]...

Options:
  --version  Show the version and exit
  --help     Show help message and exit
```

## Commands Overview

| Domain           | Commands             | Purpose                                         |
| ---------------- | -------------------- | ----------------------------------------------- |
| **article**      | search, get          | Search and retrieve biomedical literature       |
| **trial**        | search, get          | Find and fetch clinical trial information       |
| **variant**      | search, get, predict | Analyze genetic variants and predict effects    |
| **gene**         | get                  | Retrieve gene information and annotations       |
| **drug**         | get                  | Look up drug/chemical information               |
| **disease**      | get                  | Get disease definitions and synonyms            |
| **organization** | search               | Search NCI organization database                |
| **intervention** | search               | Find interventions (drugs, devices, procedures) |
| **biomarker**    | search               | Search biomarkers used in trials                |
| **health**       | check                | Monitor API status and system health            |

## Article Commands

For practical examples and workflows, see [How to Find Articles and cBioPortal Data](../how-to-guides/01-find-articles-and-cbioportal-data.md).

### article search

Search PubMed/PubTator3 for biomedical literature with automatic cBioPortal integration.

```bash
biomcp article search [OPTIONS]
```

**Options:**

- `--gene, -g TEXT`: Gene symbol(s) to search for
- `--variant, -v TEXT`: Genetic variant(s) to search for
- `--disease, -d TEXT`: Disease/condition(s) to search for
- `--chemical, -c TEXT`: Chemical/drug name(s) to search for
- `--keyword, -k TEXT`: Keyword(s) to search for (supports OR with `|`)
- `--pmid TEXT`: Specific PubMed ID(s) to retrieve
- `--limit INTEGER`: Maximum results to return (default: 10)
- `--no-preprints`: Exclude preprints from results
- `--no-cbioportal`: Disable automatic cBioPortal integration
- `--format [json|markdown]`: Output format (default: markdown)

**Examples:**

```bash
# Basic gene search with automatic cBioPortal data
biomcp article search --gene BRAF --disease melanoma

# Multiple filters
biomcp article search --gene EGFR --disease "lung cancer" --chemical erlotinib

# OR logic in keywords (find different variant notations)
biomcp article search --gene PTEN --keyword "R173|Arg173|p.R173"

# Exclude preprints
biomcp article search --gene TP53 --no-preprints --limit 20

# JSON output for programmatic use
biomcp article search --gene KRAS --format json > results.json
```

### article get

Retrieve a specific article by PubMed ID or DOI.

```bash
biomcp article get IDENTIFIER
```

**Arguments:**

- `IDENTIFIER`: PubMed ID (e.g., "38768446") or DOI (e.g., "10.1101/2024.01.20.23288905")

**Examples:**

```bash
# Get article by PubMed ID
biomcp article get 38768446

# Get preprint by DOI
biomcp article get "10.1101/2024.01.20.23288905"
```

## Trial Commands

For practical examples and workflows, see [How to Find Trials with NCI and BioThings](../how-to-guides/02-find-trials-with-nci-and-biothings.md).

### trial search

Search ClinicalTrials.gov or NCI CTS API for clinical trials.

```bash
biomcp trial search [OPTIONS]
```

**Basic Options:**

- `--condition TEXT`: Disease/condition to search
- `--intervention TEXT`: Treatment/intervention to search
- `--term TEXT`: General search terms
- `--nct-id TEXT`: Specific NCT ID(s)
- `--limit INTEGER`: Maximum results (default: 10)
- `--source [ctgov|nci]`: Data source (default: ctgov)
- `--api-key TEXT`: API key for NCI source

**Study Characteristics:**

- `--status TEXT`: Trial status (RECRUITING, ACTIVE_NOT_RECRUITING, etc.)
- `--study-type TEXT`: Type of study (INTERVENTIONAL, OBSERVATIONAL)
- `--phase TEXT`: Trial phase (EARLY_PHASE1, PHASE1, PHASE2, PHASE3, PHASE4)
- `--study-purpose TEXT`: Primary purpose (TREATMENT, PREVENTION, etc.)
- `--age-group TEXT`: Target age group (CHILD, ADULT, OLDER_ADULT)

**Location Options:**

- `--country TEXT`: Country name
- `--state TEXT`: State/province
- `--city TEXT`: City name
- `--latitude FLOAT`: Geographic latitude
- `--longitude FLOAT`: Geographic longitude
- `--distance INTEGER`: Search radius in miles

**Advanced Filters:**

- `--start-date TEXT`: Trial start date (YYYY-MM-DD)
- `--end-date TEXT`: Trial end date (YYYY-MM-DD)
- `--intervention-type TEXT`: Type of intervention
- `--sponsor-type TEXT`: Type of sponsor
- `--is-fda-regulated`: FDA-regulated trials only
- `--expanded-access`: Trials offering expanded access

**Examples:**

```bash
# Find recruiting melanoma trials
biomcp trial search --condition melanoma --status RECRUITING

# Search by location (requires coordinates)
biomcp trial search --condition "lung cancer" \
  --latitude 41.4993 --longitude -81.6944 --distance 50

# Use NCI source with advanced filters
biomcp trial search --condition melanoma --source nci \
  --required-mutations "BRAF V600E" --allow-brain-mets true \
  --api-key YOUR_KEY

# Multiple filters
biomcp trial search --condition "breast cancer" \
  --intervention "CDK4/6 inhibitor" --phase PHASE3 \
  --status RECRUITING --country "United States"
```

### trial get

Retrieve detailed information about a specific clinical trial.

```bash
biomcp trial get NCT_ID [OPTIONS]
```

**Arguments:**

- `NCT_ID`: Clinical trial identifier (e.g., NCT03006926)

**Options:**

- `--include TEXT`: Specific sections to include (Protocol, Locations, References, Outcomes)
- `--source [ctgov|nci]`: Data source (default: ctgov)
- `--api-key TEXT`: API key for NCI source

**Examples:**

```bash
# Get basic trial information
biomcp trial get NCT03006926

# Get specific sections
biomcp trial get NCT03006926 --include Protocol --include Locations

# Use NCI source
biomcp trial get NCT04280705 --source nci --api-key YOUR_KEY
```

## Variant Commands

For practical examples and workflows, see:

- [Get Comprehensive Variant Annotations](../how-to-guides/03-get-comprehensive-variant-annotations.md)
- [Predict Variant Effects with AlphaGenome](../how-to-guides/04-predict-variant-effects-with-alphagenome.md)

### variant search

Search MyVariant.info for genetic variant annotations.

```bash
biomcp variant search [OPTIONS]
```

**Options:**

- `--gene TEXT`: Gene symbol
- `--hgvs TEXT`: HGVS notation
- `--rsid TEXT`: dbSNP rsID
- `--chromosome TEXT`: Chromosome
- `--start INTEGER`: Genomic start position
- `--end INTEGER`: Genomic end position
- `--assembly [hg19|hg38]`: Genome assembly (default: hg38)
- `--significance TEXT`: Clinical significance
- `--min-frequency FLOAT`: Minimum allele frequency
- `--max-frequency FLOAT`: Maximum allele frequency
- `--min-cadd FLOAT`: Minimum CADD score
- `--polyphen TEXT`: PolyPhen prediction
- `--sift TEXT`: SIFT prediction
- `--sources TEXT`: Data sources to include
- `--limit INTEGER`: Maximum results (default: 10)
- `--no-cbioportal`: Disable cBioPortal integration

**Examples:**

```bash
# Search pathogenic BRCA1 variants
biomcp variant search --gene BRCA1 --significance pathogenic

# Search by HGVS notation
biomcp variant search --hgvs "NM_007294.4:c.5266dupC"

# Filter by frequency and prediction scores
biomcp variant search --gene TP53 --max-frequency 0.01 \
  --min-cadd 20 --polyphen possibly_damaging

# Search genomic region
biomcp variant search --chromosome 7 --start 140753336 --end 140753337
```

### variant get

Retrieve detailed information about a specific variant.

```bash
biomcp variant get VARIANT_ID
```

**Arguments:**

- `VARIANT_ID`: Variant identifier (HGVS, rsID, or genomic)

**Examples:**

```bash
# Get variant by HGVS
biomcp variant get "NM_007294.4:c.5266dupC"

# Get variant by rsID
biomcp variant get rs121913529

# Get variant by genomic coordinates
biomcp variant get "chr17:g.43082434G>A"
```

### variant predict

Predict variant effects using Google DeepMind's AlphaGenome (requires API key).

```bash
biomcp variant predict CHROMOSOME POSITION REFERENCE ALTERNATE [OPTIONS]
```

**Arguments:**

- `CHROMOSOME`: Chromosome (e.g., chr7)
- `POSITION`: Genomic position
- `REFERENCE`: Reference allele
- `ALTERNATE`: Alternate allele

**Options:**

- `--tissue TEXT`: Tissue type(s) using UBERON ontology
- `--interval INTEGER`: Analysis window size (default: 20000)
- `--api-key TEXT`: AlphaGenome API key

**Examples:**

```bash
# Basic prediction (requires ALPHAGENOME_API_KEY env var)
biomcp variant predict chr7 140753336 A T

# Tissue-specific prediction
biomcp variant predict chr7 140753336 A T \
  --tissue UBERON:0002367  # breast tissue

# With per-request API key
biomcp variant predict chr7 140753336 A T --api-key YOUR_KEY
```

## Gene/Drug/Disease Commands

For practical examples using BioThings integration, see [How to Find Trials with NCI and BioThings](../how-to-guides/02-find-trials-with-nci-and-biothings.md#biothings-integration-for-enhanced-search).

### gene get

Retrieve gene information from MyGene.info.

```bash
biomcp gene get GENE_NAME
```

**Examples:**

```bash
# Get gene information
biomcp gene get TP53
biomcp gene get BRAF
```

### drug get

Retrieve drug/chemical information from MyChem.info.

```bash
biomcp drug get DRUG_NAME
```

**Examples:**

```bash
# Get drug information
biomcp drug get imatinib
biomcp drug get pembrolizumab
```

### disease get

Retrieve disease information from MyDisease.info.

```bash
biomcp disease get DISEASE_NAME
```

**Examples:**

```bash
# Get disease information
biomcp disease get melanoma
biomcp disease get "non-small cell lung cancer"
```

## NCI-Specific Commands

These commands require an NCI API key. For setup instructions and usage examples, see:

- [Authentication and API Keys](../getting-started/03-authentication-and-api-keys.md#nci-clinical-trials-api)
- [How to Find Trials with NCI and BioThings](../how-to-guides/02-find-trials-with-nci-and-biothings.md#using-nci-api-advanced-features)

### organization search

Search NCI's organization database.

```bash
biomcp organization search [OPTIONS]
```

**Options:**

- `--name TEXT`: Organization name
- `--city TEXT`: City location
- `--state TEXT`: State/province
- `--country TEXT`: Country
- `--org-type TEXT`: Organization type
- `--api-key TEXT`: NCI API key

**Example:**

```bash
biomcp organization search --name "MD Anderson" \
  --city Houston --state TX --api-key YOUR_KEY
```

### intervention search

Search NCI's intervention database.

```bash
biomcp intervention search [OPTIONS]
```

**Options:**

- `--name TEXT`: Intervention name
- `--intervention-type TEXT`: Type (Drug, Device, Procedure, etc.)
- `--api-key TEXT`: NCI API key

**Example:**

```bash
biomcp intervention search --name pembrolizumab \
  --intervention-type Drug --api-key YOUR_KEY
```

### biomarker search

Search biomarkers used in clinical trials.

```bash
biomcp biomarker search [OPTIONS]
```

**Options:**

- `--gene TEXT`: Gene symbol
- `--biomarker-type TEXT`: Type of biomarker
- `--api-key TEXT`: NCI API key

**Example:**

```bash
biomcp biomarker search --gene EGFR \
  --biomarker-type mutation --api-key YOUR_KEY
```

## Health Command

For monitoring API status before bulk operations, see the [Performance Optimizations Guide](../developer-guides/07-performance-optimizations.md).

### health check

Monitor API endpoints and system health.

```bash
biomcp health check [OPTIONS]
```

**Options:**

- `--apis-only`: Check only API endpoints
- `--system-only`: Check only system resources
- `--verbose, -v`: Show detailed information

**Examples:**

```bash
# Full health check
biomcp health check

# Check APIs only
biomcp health check --apis-only

# Detailed system check
biomcp health check --system-only --verbose
```

## Output Formats

Most commands support both human-readable markdown and machine-readable JSON output:

```bash
# Default markdown output
biomcp article search --gene BRAF

# JSON for programmatic use
biomcp article search --gene BRAF --format json

# Save to file
biomcp trial search --condition melanoma --format json > trials.json
```

## Environment Variables

Configure default behavior with environment variables:

```bash
# API Keys
export NCI_API_KEY="your-nci-key"
export ALPHAGENOME_API_KEY="your-alphagenome-key"
export CBIO_TOKEN="your-cbioportal-token"

# Logging
export BIOMCP_LOG_LEVEL="DEBUG"
export BIOMCP_CACHE_DIR="/path/to/cache"
```

## Getting Help

Every command has a built-in help flag:

```bash
# General help
biomcp --help

# Command-specific help
biomcp article search --help
biomcp trial get --help
biomcp variant predict --help
```

## Tips and Best Practices

1. **Use Official Gene Symbols**: Always use HGNC-approved gene symbols (e.g., "TP53" not "p53")

2. **Combine Filters**: Most commands support multiple filters for precise results:

   ```bash
   biomcp article search --gene EGFR --disease "lung cancer" \
     --chemical erlotinib --keyword "resistance"
   ```

3. **Handle Large Results**: Use `--limit` and `--format json` for processing:

   ```bash
   biomcp article search --gene BRCA1 --limit 100 --format json | \
     jq '.results[] | {pmid: .pmid, title: .title}'
   ```

4. **Location Searches**: Always provide both latitude and longitude:

   ```bash
   # Find trials near Boston
   biomcp trial search --condition cancer \
     --latitude 42.3601 --longitude -71.0589 --distance 25
   ```

5. **Use OR Logic**: The pipe character enables flexible searches:

   ```bash
   # Find articles mentioning any form of a variant
   biomcp article search --gene BRAF --keyword "V600E|p.V600E|c.1799T>A"
   ```

6. **Check API Health**: Before bulk operations, verify API status:
   ```bash
   biomcp health check --apis-only
   ```

## Next Steps

- Set up [API keys](../getting-started/03-authentication-and-api-keys.md) for enhanced features
- Explore [MCP tools](02-mcp-tools-reference.md) for AI integration
- Read [how-to guides](../how-to-guides/01-find-articles-and-cbioportal-data.md) for complex workflows
