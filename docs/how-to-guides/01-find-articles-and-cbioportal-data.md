# How to Find Articles and cBioPortal Data

This guide walks you through searching biomedical literature with automatic cancer genomics integration from cBioPortal.

## Overview

When searching for articles about genes, BioMCP automatically enriches your results with:

- **cBioPortal Summary**: Mutation frequencies, hotspots, and cancer type distribution ([API Reference](../backend-services-reference/03-cbioportal.md))
- **PubMed Articles**: Peer-reviewed research with entity annotations ([PubTator3 Reference](../backend-services-reference/06-pubtator3.md))
- **Preprints**: Latest findings from bioRxiv and medRxiv

## Basic Article Search

### Search by Gene

Find articles about a specific gene:

```bash
# CLI
biomcp article search --gene BRAF --limit 5

# Python
articles = await client.articles.search(genes=["BRAF"], limit=5)

# MCP Tool
article_searcher(genes=["BRAF"], limit=5)
```

This automatically includes:

1. cBioPortal summary showing BRAF mutation frequency across cancers
2. Top mutation hotspots (e.g., V600E)
3. Recent articles mentioning BRAF

### Search by Disease

Find articles about a specific disease:

```bash
# CLI
biomcp article search --disease melanoma --limit 10

# Python
articles = await client.articles.search(diseases=["melanoma"])

# MCP Tool
article_searcher(diseases=["melanoma"])
```

## Advanced Search Techniques

### Combining Multiple Filters

Search for articles at the intersection of genes, diseases, and chemicals:

```bash
# CLI - EGFR mutations in lung cancer treated with erlotinib
biomcp article search \
  --gene EGFR \
  --disease "lung cancer" \
  --chemical erlotinib \
  --limit 20

# Python
articles = await client.articles.search(
    genes=["EGFR"],
    diseases=["lung cancer"],
    chemicals=["erlotinib"]
)
```

### Using OR Logic in Keywords

Find articles mentioning different notations of the same variant:

```bash
# CLI - Find any notation of BRAF V600E
biomcp article search \
  --gene BRAF \
  --keyword "V600E|p.V600E|c.1799T>A"

# Python - Different names for same concept
articles = await client.articles.search(
    diseases=["NSCLC|non-small cell lung cancer"],
    chemicals=["pembrolizumab|Keytruda|anti-PD-1"]
)
```

### Excluding Preprints

For peer-reviewed articles only:

```bash
# CLI
biomcp article search --gene TP53 --no-preprints

# Python
articles = await client.articles.search(
    genes=["TP53"],
    include_preprints=False
)
```

## Understanding cBioPortal Integration

### What cBioPortal Provides

When you search for a gene, the first result includes:

```markdown
### cBioPortal Summary for BRAF

- **Mutation Frequency**: 76.7% (368 mutations in 480 samples)
- **Studies**: 1 of 5 studies have mutations

**Top Hotspots:**

1. V600E: 310 mutations (84.2%)
2. V600K: 23 mutations (6.3%)
3. V600M: 12 mutations (3.3%)

**Cancer Type Distribution:**

- Skin Cancer, Non-Melanoma: 156 mutations
- Melanoma: 91 mutations
- Thyroid Cancer: 87 mutations
```

### Mutation-Specific Searches

Search for articles about specific mutations:

```python
# Search for BRAF V600E specifically
articles = await client.articles.search(
    genes=["BRAF"],
    keywords=["V600E"],
    include_cbioportal=True  # Default
)
```

The cBioPortal summary will highlight the specific mutation if found.

### Disabling cBioPortal

If you don't need cancer genomics data:

```bash
# CLI
biomcp article search --gene BRCA1 --no-cbioportal

# Python
articles = await client.articles.search(
    genes=["BRCA1"],
    include_cbioportal=False
)
```

## Practical Examples

### Example 1: Resistance Mechanism Research

Find articles about EGFR T790M resistance:

```python
# Using think tool first (for MCP)
think(
    thought="Researching EGFR T790M resistance mechanisms in lung cancer",
    thoughtNumber=1
)

# Search with multiple relevant terms
articles = await article_searcher(
    genes=["EGFR"],
    diseases=["lung cancer|NSCLC"],
    keywords=["T790M|p.T790M|resistance|resistant"],
    chemicals=["osimertinib|gefitinib|erlotinib"]
)
```

### Example 2: Combination Therapy Research

Research BRAF/MEK combination therapy:

```bash
# CLI approach
biomcp article search \
  --gene BRAF --gene MEK1 --gene MEK2 \
  --disease melanoma \
  --chemical dabrafenib --chemical trametinib \
  --keyword "combination therapy|combined treatment"
```

### Example 3: Biomarker Discovery

Find articles about potential biomarkers:

```python
# Search for PD-L1 as a biomarker
articles = await client.articles.search(
    genes=["CD274"],  # PD-L1 gene symbol
    keywords=["biomarker|predictive|prognostic"],
    diseases=["cancer"],
    limit=50
)

# Filter results programmatically
biomarker_articles = [
    a for a in articles
    if "biomarker" in a.title.lower() or "predictive" in a.abstract.lower()
]
```

## Working with Results

### Extracting Key Information

```python
# Process article results
for article in articles:
    print(f"Title: {article.title}")
    print(f"PMID: {article.pmid}")
    print(f"URL: {article.url}")

    # Extract annotated entities
    genes = article.metadata.get("genes", [])
    diseases = article.metadata.get("diseases", [])
    chemicals = article.metadata.get("chemicals", [])

    print(f"Genes mentioned: {', '.join(genes)}")
    print(f"Diseases: {', '.join(diseases)}")
    print(f"Chemicals: {', '.join(chemicals)}")
```

### Fetching Full Article Details

Get complete article information:

```python
# Get article by PMID
full_article = await client.articles.get("38768446")

# Access full abstract
print(full_article.abstract)

# Check for full text availability
if full_article.full_text_url:
    print(f"Full text: {full_article.full_text_url}")
```

## Tips for Effective Searches

### 1. Use Official Gene Symbols

```python
# ✅ Correct - Official HGNC symbol
articles = await search(genes=["ERBB2"])

# ❌ Avoid - Common name
articles = await search(genes=["HER2"])  # May miss results
```

### 2. Include Synonyms for Diseases

```python
# Cover all variations
articles = await search(
    diseases=["GIST|gastrointestinal stromal tumor|gastrointestinal stromal tumour"]
)
```

### 3. Leverage PubTator Annotations

PubTator automatically annotates articles with:

- Gene mentions (normalized to official symbols)
- Disease concepts (mapped to MeSH terms)
- Chemical/drug entities
- Genetic variants
- Species

### 4. Combine with Other Tools

```python
# 1. Find articles about a gene
articles = await article_searcher(genes=["ALK"])

# 2. Get gene details for context
gene_info = await gene_getter("ALK")

# 3. Find relevant trials
trials = await trial_searcher(
    other_terms=["ALK positive", "ALK rearrangement"]
)
```

## Troubleshooting

### No Results Found

1. **Check gene symbols**: Use [genenames.org](https://www.genenames.org)
2. **Broaden search**: Remove filters one by one
3. **Try synonyms**: Especially for diseases and drugs

### cBioPortal Data Missing

- Some genes may not have cancer genomics data
- Try searching for cancer-related genes
- Check if gene symbol is correct

### Preprint Issues

- Europe PMC may have delays in indexing
- Some preprints may not have DOIs
- Try searching by title keywords instead

## Next Steps

- Learn to [find trials with NCI and BioThings](02-find-trials-with-nci-and-biothings.md)
- Explore [variant annotations](03-get-comprehensive-variant-annotations.md)
- Set up [API keys](../getting-started/03-authentication-and-api-keys.md) for enhanced features
